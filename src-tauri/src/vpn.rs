//! VPNs: status pelas placas de rede (funciona com qualquer cliente de VPN),
//! ligar/desligar conforme o tipo e teste de acesso a um host (ex.: o GitLab do órgão).

use serde::{Deserialize, Serialize};
use std::net::{TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone, PartialEq)]
pub struct Adapter {
    pub name: String,
    pub desc: String,
    pub up: bool,
    pub ip: String,
}

#[cfg(windows)]
pub fn adapters() -> Vec<Adapter> {
    use windows::Win32::NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_SKIP_MULTICAST,
        IP_ADAPTER_ADDRESSES_LH,
    };
    use windows::Win32::NetworkManagement::Ndis::IfOperStatusUp;
    use windows::Win32::Networking::WinSock::{AF_INET, AF_UNSPEC, SOCKADDR_IN};

    let mut size: u32 = 32 * 1024;
    let mut buf: Vec<u64>;
    loop {
        buf = vec![0u64; (size as usize / 8) + 1];
        let r = unsafe {
            GetAdaptersAddresses(
                AF_UNSPEC.0 as u32,
                GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER,
                None,
                Some(buf.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut size,
            )
        };
        if r == 111 {
            continue; // ERROR_BUFFER_OVERFLOW: tenta de novo com o tamanho pedido
        }
        if r != 0 {
            return vec![];
        }
        break;
    }
    let mut out = Vec::new();
    let mut p = buf.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
    while !p.is_null() {
        let a = unsafe { &*p };
        let name = unsafe { a.FriendlyName.to_string().unwrap_or_default() };
        let desc = unsafe { a.Description.to_string().unwrap_or_default() };
        let mut ip = String::new();
        let mut u = a.FirstUnicastAddress;
        while !u.is_null() {
            let ua = unsafe { &*u };
            let sa = ua.Address.lpSockaddr;
            if !sa.is_null() && unsafe { (*sa).sa_family } == AF_INET {
                let sin = unsafe { &*(sa as *const SOCKADDR_IN) };
                let b = unsafe { sin.sin_addr.S_un.S_un_b };
                ip = format!("{}.{}.{}.{}", b.s_b1, b.s_b2, b.s_b3, b.s_b4);
                break;
            }
            u = ua.Next;
        }
        out.push(Adapter { name, desc, up: a.OperStatus == IfOperStatusUp, ip });
        p = a.Next;
    }
    out
}
#[cfg(not(windows))]
pub fn adapters() -> Vec<Adapter> {
    vec![]
}

/// Envia a lista de placas de rede para a interface sempre que algo muda
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: Vec<Adapter> = Vec::new();
        loop {
            let now = adapters();
            if now != last {
                let _ = app.emit("adapters", now.clone());
                last = now;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

#[tauri::command(async)]
pub fn list_adapters() -> Vec<Adapter> {
    adapters()
}

/* ---------------- ligar / desligar ---------------- */

#[derive(Deserialize)]
pub struct VpnProfile {
    /// windows | openvpnconnect | openvpn | wireguard | cisco | forti | globalprotect | custom
    #[serde(rename = "type")]
    kind: String,
    /// nome da conexão / config / túnel / servidor, conforme o tipo
    #[serde(default)]
    name: String,
    /// usuário (Cisco)
    #[serde(default)]
    user: String,
    /// id do órgão, para achar a senha salva no Gerenciador de Credenciais
    #[serde(default)]
    org: String,
    #[serde(default)]
    connect: String,
    #[serde(default)]
    disconnect: String,
}

fn first_existing(paths: &[&str]) -> Option<PathBuf> {
    let expand = |p: &str| {
        let mut s = p.to_string();
        for k in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Ok(v) = std::env::var(k) {
                s = s.replace(&format!("%{k}%"), &v);
            }
        }
        PathBuf::from(s)
    };
    paths.iter().map(|p| expand(p)).find(|p| p.exists())
}

fn run(program: &PathBuf, args: &[&str]) -> Result<(), String> {
    crate::util::hidden(&program.to_string_lossy()).args(args).spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Roda um comando do usuário pelo cmd (para o tipo "personalizado")
fn shell(cmd: &str) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        crate::util::hidden("cmd.exe").raw_arg(format!("/d /s /c \"{cmd}\"")).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("sh").arg("-c").arg(cmd).spawn().map(|_| ()).map_err(|e| e.to_string())
    }
}

const OPENVPN_GUI: &[&str] = &["%ProgramFiles%\\OpenVPN\\bin\\openvpn-gui.exe", "%ProgramFiles%\\OpenVPN Connect\\OpenVPNConnect.exe"];
const WIREGUARD: &[&str] = &["%ProgramFiles%\\WireGuard\\wireguard.exe"];
const CISCO_CLI: &[&str] = &[
    "%ProgramFiles(x86)%\\Cisco\\Cisco Secure Client\\vpncli.exe",
    "%ProgramFiles(x86)%\\Cisco\\Cisco AnyConnect Secure Mobility Client\\vpncli.exe",
];
const CISCO_UI: &[&str] = &[
    "%ProgramFiles(x86)%\\Cisco\\Cisco Secure Client\\UI\\csc_ui.exe",
    "%ProgramFiles(x86)%\\Cisco\\Cisco AnyConnect Secure Mobility Client\\vpnui.exe",
];
const OVPN_CONNECT: &[&str] = &["%ProgramFiles%\\OpenVPN Connect\\OpenVPNConnect.exe"];
const FORTI: &[&str] = &["%ProgramFiles%\\Fortinet\\FortiClient\\FortiClient.exe"];
const GLOBALPROTECT: &[&str] = &["%ProgramFiles%\\Palo Alto Networks\\GlobalProtect\\PanGPA.exe"];
/// CLI oficial das versões novas do GlobalProtect (instaladas por MSI)
const GP_CLI: &[&str] = &["%ProgramFiles%\\Palo Alto Networks\\GlobalProtect\\pangpcli.exe"];

/// Resultado: "ok" (comando enviado) ou "app" (abrimos o programa da VPN para você entrar)
#[derive(Serialize)]
pub struct VpnResult {
    ok: bool,
    mode: &'static str,
    msg: String,
}

fn open_app(paths: &[&str], label: &str) -> VpnResult {
    match first_existing(paths) {
        Some(p) => match run(&p, &[]) {
            Ok(_) => VpnResult { ok: true, mode: "app", msg: format!("Abrimos o {label} para você conectar") },
            Err(e) => VpnResult { ok: false, mode: "app", msg: e },
        },
        None => VpnResult { ok: false, mode: "app", msg: format!("{label} não encontrado neste PC") },
    }
}

/* ---------------- senha da VPN (Gerenciador de Credenciais) ---------------- */

const PW_SERVICE: &str = "ilha-dinamica-vpn";

fn password(org: &str) -> Option<String> {
    keyring::Entry::new(PW_SERVICE, org).ok()?.get_password().ok()
}

#[tauri::command(async)]
pub fn vpn_set_password(org: String, password: String) -> Result<bool, String> {
    let e = keyring::Entry::new(PW_SERVICE, &org).map_err(|e| e.to_string())?;
    if password.is_empty() {
        let _ = e.delete_credential();
        return Ok(false);
    }
    e.set_password(&password).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command(async)]
pub fn vpn_has_passwords(orgs: Vec<String>) -> Vec<String> {
    orgs.into_iter().filter(|o| password(o).is_some()).collect()
}

/* ---------------- OpenVPN Connect (v3) ---------------- */

/// Acha o id do perfil no OpenVPN Connect pelo nome ou servidor (ex.: vpnclient.enap.gov.br)
fn ovpn_connect_profile_id(exe: &PathBuf, wanted: &str) -> Option<String> {
    let out = crate::util::hidden(&exe.to_string_lossy()).arg("--list-profiles").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let start = text.find('[').or_else(|| text.find('{'))?;
    let v: serde_json::Value = serde_json::from_str(text[start..].trim()).ok()?;
    let list = v.as_array().cloned().or_else(|| v["message"].as_array().cloned())?;
    let w = wanted.to_lowercase();
    let pick = list.iter().find(|p| {
        let f = |k: &str| p[k].as_str().unwrap_or("").to_lowercase();
        w.is_empty() || f("name").contains(&w) || f("host").contains(&w)
    })?;
    pick["id"].as_str().map(String::from).or_else(|| pick["id"].as_u64().map(|n| n.to_string()))
}

/* ---------------- Cisco Secure Client ---------------- */

fn kill_cisco_ui() {
    for exe in ["csc_ui.exe", "vpnui.exe"] {
        let _ = crate::util::hidden("taskkill").args(["/IM", exe, "/F"]).output();
    }
}

/// Conecta pelo vpncli enviando usuário e senha pela entrada padrão (nunca na linha de comando)
fn cisco_connect(server: &str, user: &str, pass: &str) -> VpnResult {
    use std::io::Write;
    use std::process::Stdio;
    let Some(cli) = first_existing(CISCO_CLI) else {
        return VpnResult { ok: false, mode: "cmd", msg: "Cisco Secure Client não encontrado".into() };
    };
    kill_cisco_ui(); // o vpncli não conecta com a janela do Cisco aberta
    let child = crate::util::hidden(&cli.to_string_lossy())
        .arg("-s")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(c) => c,
        Err(e) => return VpnResult { ok: false, mode: "cmd", msg: e.to_string() },
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = write!(stdin, "connect {server}\n{user}\n{pass}\ny\nexit\n");
    }
    let out = child.wait_with_output();
    // reabre a janela/bandeja do Cisco já conectado
    if let Some(ui) = first_existing(CISCO_UI) {
        let _ = run(&ui, &[]);
    }
    let text = out.map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase()).unwrap_or_default();
    if text.contains("login failed") || text.contains("falha no login") {
        return VpnResult { ok: false, mode: "cmd", msg: "Usuário ou senha da VPN incorretos".into() };
    }
    VpnResult { ok: true, mode: "cmd", msg: String::new() }
}

#[tauri::command(async)]
pub fn vpn_connect(profile: VpnProfile) -> VpnResult {
    let name = profile.name.trim();
    let done = |r: Result<(), String>| match r {
        Ok(_) => VpnResult { ok: true, mode: "cmd", msg: String::new() },
        Err(e) => VpnResult { ok: false, mode: "cmd", msg: e },
    };
    match profile.kind.as_str() {
        // VPN nativa do Windows (Configurações → Rede → VPN), usa a senha salva
        "windows" => {
            let out = crate::util::hidden("rasdial").arg(name).output();
            match out {
                Ok(o) if o.status.success() => VpnResult { ok: true, mode: "cmd", msg: String::new() },
                _ => {
                    // sem senha salva: abre a tela de VPN do Windows
                    let _ = shell("start \"\" ms-settings:network-vpn");
                    VpnResult { ok: true, mode: "app", msg: "Abrimos as configurações de VPN do Windows".into() }
                }
            }
        }
        // OpenVPN Connect v3 (app novo): conecta pelo id do perfil
        "openvpnconnect" => match first_existing(OVPN_CONNECT) {
            Some(p) => match ovpn_connect_profile_id(&p, name) {
                Some(id) => done(run(&p, &[&format!("--connect-shortcut={id}")])),
                None => open_app(OVPN_CONNECT, "OpenVPN Connect"),
            },
            None => VpnResult { ok: false, mode: "cmd", msg: "OpenVPN Connect não encontrado".into() },
        },
        "openvpn" => match first_existing(OPENVPN_GUI) {
            Some(p) if p.ends_with("openvpn-gui.exe") => done(run(&p, &["--command", "connect", name])),
            _ => open_app(OPENVPN_GUI, "OpenVPN"),
        },
        "wireguard" => {
            if crate::util::is_admin() && !name.is_empty() {
                match first_existing(WIREGUARD) {
                    Some(p) => done(run(&p, &["/installtunnelservice", name])),
                    None => VpnResult { ok: false, mode: "cmd", msg: "WireGuard não encontrado".into() },
                }
            } else {
                open_app(WIREGUARD, "WireGuard")
            }
        }
        // Cisco: com senha salva conecta direto; sem senha, abre o app oficial
        "cisco" => match password(&profile.org) {
            Some(pw) if !name.is_empty() && !profile.user.trim().is_empty() => cisco_connect(name, profile.user.trim(), &pw),
            _ => open_app(CISCO_UI, "Cisco Secure Client"),
        },
        "forti" => open_app(FORTI, "FortiClient"),
        "globalprotect" => match first_existing(GP_CLI) {
            Some(cli) if !name.is_empty() => done(run(&cli, &["-start", "-portal", name])),
            Some(cli) => done(run(&cli, &["-start"])),
            None => open_app(GLOBALPROTECT, "GlobalProtect"),
        },
        "custom" if !profile.connect.trim().is_empty() => done(shell(profile.connect.trim())),
        _ => VpnResult { ok: false, mode: "cmd", msg: "Configure a VPN deste órgão primeiro".into() },
    }
}

#[tauri::command(async)]
pub fn vpn_disconnect(profile: VpnProfile) -> VpnResult {
    let name = profile.name.trim();
    let done = |r: Result<(), String>| match r {
        Ok(_) => VpnResult { ok: true, mode: "cmd", msg: String::new() },
        Err(e) => VpnResult { ok: false, mode: "cmd", msg: e },
    };
    match profile.kind.as_str() {
        "windows" => done(crate::util::hidden("rasdial").args([name, "/disconnect"]).output().map(|_| ()).map_err(|e| e.to_string())),
        "openvpnconnect" => match first_existing(OVPN_CONNECT) {
            Some(p) => {
                let arg = match ovpn_connect_profile_id(&p, name) {
                    Some(id) => format!("--disconnect-shortcut={id}"),
                    None => "--disconnect-shortcut".into(),
                };
                done(run(&p, &[&arg]))
            }
            None => VpnResult { ok: false, mode: "cmd", msg: "OpenVPN Connect não encontrado".into() },
        },
        "openvpn" => match first_existing(OPENVPN_GUI) {
            Some(p) if p.ends_with("openvpn-gui.exe") => done(run(&p, &["--command", "disconnect", name])),
            _ => open_app(OPENVPN_GUI, "OpenVPN"),
        },
        "wireguard" => {
            let tunnel = std::path::Path::new(name).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            match first_existing(WIREGUARD) {
                Some(p) if crate::util::is_admin() => done(run(&p, &["/uninstalltunnelservice", &tunnel])),
                _ => open_app(WIREGUARD, "WireGuard"),
            }
        }
        // o Cisco desconecta sem pedir senha pela linha de comando
        "cisco" => match first_existing(CISCO_CLI) {
            Some(p) => done(run(&p, &["disconnect"])),
            None => open_app(CISCO_UI, "Cisco Secure Client"),
        },
        "forti" => open_app(FORTI, "FortiClient"),
        "globalprotect" => match first_existing(GP_CLI) {
            Some(cli) => done(run(&cli, &["-disconnect"])),
            None => open_app(GLOBALPROTECT, "GlobalProtect"),
        },
        "custom" if !profile.disconnect.trim().is_empty() => done(shell(profile.disconnect.trim())),
        _ => VpnResult { ok: false, mode: "cmd", msg: "Sem comando para desligar".into() },
    }
}

/* ---------------- teste de acesso ---------------- */

#[derive(Serialize)]
pub struct Probe {
    ok: bool,
    ms: u32,
}

/// Tenta abrir uma conexão TCP com o host (porta 443 por padrão). Ex.: gitlab.orgao.gov.br
#[tauri::command(async)]
pub fn probe_host(host: String) -> Probe {
    let h = host
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string();
    if h.is_empty() {
        return Probe { ok: false, ms: 0 };
    }
    let target = if h.contains(':') { h } else { format!("{h}:443") };
    let start = Instant::now();
    let ok = target
        .to_socket_addrs()
        .ok()
        .and_then(|mut a| a.next())
        .map(|addr| TcpStream::connect_timeout(&addr, Duration::from_secs(3)).is_ok())
        .unwrap_or(false);
    Probe { ok, ms: start.elapsed().as_millis() as u32 }
}
