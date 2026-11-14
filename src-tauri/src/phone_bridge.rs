//! Ponte multi-transporte com o celular (Mesma Rede LAN + Cabo USB ADB + Bluetooth).
//! Unifica todos os transportes em pacotes `PhoneBridgePacket` para o núcleo do assistente.

use assistant_core::bridge::{
    build_qr_pairing_uri, generate_pairing_token, merge_handshake_device, packet_to_signal,
    parse_bridge_packet, PhoneBridgePacket,
};
use assistant_core::domain::{PairingInfo, PhoneDevice, PhoneTransport};
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, UdpSocket};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

use crate::vault;

const BRIDGE_TOKEN_KEY: &str = "phone:bridge_token";
const DEFAULT_PORT: u16 = 47821;

pub struct BridgeRuntime {
    pub port: Mutex<u16>,
    pub token: Mutex<String>,
}

impl Default for BridgeRuntime {
    fn default() -> Self {
        Self {
            port: Mutex::new(DEFAULT_PORT),
            token: Mutex::new(String::new()),
        }
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn ensure_token() -> String {
    if let Some(t) = vault::get_secret(BRIDGE_TOKEN_KEY) {
        if !t.is_empty() {
            return t;
        }
    }
    let tok = generate_pairing_token(&format!("ilha-{}-{}", now_ts(), std::process::id()));
    let _ = vault::set_secret(BRIDGE_TOKEN_KEY, &tok);
    tok
}

pub fn rotate_token() -> String {
    let tok = generate_pairing_token(&format!("ilha-rot-{}-{}", now_ts(), std::process::id()));
    let _ = vault::set_secret(BRIDGE_TOKEN_KEY, &tok);
    tok
}

pub fn detect_lan_ip() -> String {
    if let Ok(sock) = UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = sock.local_addr() {
                return addr.ip().to_string();
            }
        }
    }
    "127.0.0.1".into()
}

pub fn list_adb_devices() -> (bool, Vec<String>) {
    let Ok(out) = crate::util::hidden("adb").arg("devices").output() else {
        return (false, vec![]);
    };
    if !out.status.success() {
        return (false, vec![]);
    }
    let txt = String::from_utf8_lossy(&out.stdout);
    let devs = txt
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?;
            let status = parts.next()?;
            if status == "device" {
                Some(id.to_string())
            } else {
                None
            }
        })
        .collect();
    (true, devs)
}

pub fn enable_usb_reverse(port: u16) -> Result<Vec<String>, String> {
    let (ok, devs) = list_adb_devices();
    if !ok {
        return Err("ADB não encontrado no PATH (instale Android Platform Tools para cabo USB)".into());
    }
    if devs.is_empty() {
        return Err("Nenhum celular Android detectado no cabo USB com Depuração USB ativa".into());
    }
    let spec = format!("tcp:{port}");
    let out = crate::util::hidden("adb")
        .args(["reverse", &spec, &spec])
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(devs)
}

pub fn list_bt_paired_phones() -> Vec<String> {
    let bt = crate::network::bt_state();
    bt.devices
        .into_iter()
        .map(|d| if d.connected { format!("{} (conectado)", d.name) } else { d.name })
        .collect()
}

pub fn get_pairing_info(port: u16, token: &str) -> PairingInfo {
    let host = detect_lan_ip();
    let pc_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "Ilha Dinâmica PC".into());
    let (adb_available, adb_devices) = list_adb_devices();
    let bt_paired_phones = list_bt_paired_phones();
    PairingInfo {
        qr_uri: build_qr_pairing_uri(&host, port, token, &pc_name),
        host,
        port,
        token: token.to_string(),
        adb_available,
        adb_devices,
        bt_paired_phones,
    }
}

/// Inicia o servidor TCP/NDJSON da ponte do celular em thread de fundo (LAN + USB reverse).
pub fn start_listener(app: AppHandle, expected_token: String, port_slot: &Mutex<u16>) {
    let listener = TcpListener::bind(("0.0.0.0", DEFAULT_PORT))
        .or_else(|_| TcpListener::bind(("0.0.0.0", 0)));

    let Ok(listener) = listener else {
        crate::logger::log("phone_bridge: falha ao abrir porta TCP");
        return;
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(DEFAULT_PORT);
    *port_slot.lock().unwrap() = port;
    crate::logger::log(format!("phone_bridge escutando em 0.0.0.0:{port}"));

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let app_c = app.clone();
            let tok_c = expected_token.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                while let Ok(n) = reader.read_line(&mut line) {
                    if n == 0 {
                        break;
                    }
                    let trimmed = line.trim();
                    if !trimmed.is_empty() && trimmed.starts_with('{') {
                        if let Ok(pkt) = parse_bridge_packet(trimmed) {
                            let valid = match &pkt {
                                PhoneBridgePacket::Handshake { token, .. } => token == &tok_c,
                                _ => true,
                            };
                            if valid {
                                crate::assistant::ingest_phone_packet(&app_c, pkt);
                                let _ = reader.get_mut().write_all(b"{\"ok\":true}\n");
                            } else {
                                let _ = reader.get_mut().write_all(b"{\"ok\":false,\"err\":\"token\"}\n");
                            }
                        }
                    }
                    line.clear();
                }
            });
        }
    });
}

pub fn apply_packet_to_db(
    db: &assistant_core::db::PersonalMemoryDb,
    pkt: &PhoneBridgePacket,
    app: &AppHandle,
) {
    let ts = now_ts();
    match pkt {
        PhoneBridgePacket::Handshake {
            device_id,
            device_name,
            platform,
            transport,
            ..
        } => {
            let dev = merge_handshake_device(
                device_id.clone(),
                device_name.clone(),
                platform.clone(),
                *transport,
                ts,
            );
            let _ = db.upsert_phone(&dev);
        }
        PhoneBridgePacket::Battery {
            device_id,
            level,
            charging,
        } => {
            let existing = db
                .list_phones()
                .ok()
                .and_then(|l| l.into_iter().find(|d| &d.id == device_id));
            let mut dev = existing.unwrap_or(PhoneDevice {
                id: device_id.clone(),
                name: "Meu Celular".into(),
                platform: "android".into(),
                connected: true,
                active_transport: PhoneTransport::Lan,
                battery_pct: Some(*level),
                charging: *charging,
                last_seen_ts: ts,
            });
            dev.connected = true;
            dev.battery_pct = Some(*level);
            dev.charging = *charging;
            dev.last_seen_ts = ts;
            let _ = db.upsert_phone(&dev);
        }
        PhoneBridgePacket::Clipboard { text, .. } => {
            crate::clipboard::set_text_from_app(app, text.clone());
        }
        PhoneBridgePacket::Ping { .. } => {
            let _ = app.emit("phone-ping", ts);
        }
        _ => {}
    }
    if let Some(sig) = packet_to_signal(pkt, ts) {
        let _ = db.record_phone_signal(&sig);
        let _ = app.emit("phone-signal", &sig);
    }
}
