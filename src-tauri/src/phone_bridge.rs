//! Ponte multi-transporte com o celular (Mesma Rede LAN + Cabo USB ADB + Bluetooth).
//! Unifica todos os transportes em pacotes `PhoneBridgePacket` para o núcleo do assistente.

use assistant_core::bridge::{
    build_companion_url, build_qr_pairing_uri, build_webhook_url, generate_pairing_token,
    merge_handshake_device, packet_to_signal, parse_bridge_packet, PhoneBridgePacket,
};
use assistant_core::domain::{PairingInfo, PhoneDevice, PhoneTransport};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream, UdpSocket};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

use crate::vault;

const BRIDGE_TOKEN_KEY: &str = "phone:bridge_token";
const DEFAULT_PORT: u16 = 47821;
const COMPANION_HTML: &str = include_str!("companion.html");
const COMPANION_MANIFEST: &str = r##"{"name":"Ilha Dinâmica Companion","short_name":"Ilha Companion","start_url":"/","display":"standalone","background_color":"#0f1016","theme_color":"#0f1016"}"##;

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
        companion_url: build_companion_url(&host, port, token),
        webhook_url: build_webhook_url(&host, port, token),
        host,
        port,
        token: token.to_string(),
        adb_available,
        adb_devices,
        bt_paired_phones,
    }
}

fn live_token(app: &AppHandle) -> String {
    app.state::<crate::assistant::AssistantState>()
        .bridge
        .token
        .lock()
        .unwrap()
        .clone()
}

fn query_token(target: &str) -> Option<String> {
    let (_, query) = target.split_once('?')?;
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == "token" {
                return Some(v.to_string());
            }
        }
    }
    None
}

fn write_http(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type, Authorization\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

fn handle_http_connection(mut reader: BufReader<TcpStream>, first_line: String, app: AppHandle) {
    let mut parts = first_line.split_whitespace();
    let method = parts.next().unwrap_or("GET").to_string();
    let target = parts.next().unwrap_or("/").to_string();

    let mut content_len: usize = 0;
    let mut bearer_token: Option<String> = None;
    let mut hdr_line = String::new();
    while let Ok(n) = reader.read_line(&mut hdr_line) {
        if n == 0 || hdr_line == "\r\n" || hdr_line == "\n" {
            break;
        }
        if let Some((k, v)) = hdr_line.split_once(':') {
            let kl = k.trim().to_ascii_lowercase();
            let vt = v.trim();
            if kl == "content-length" {
                content_len = vt.parse::<usize>().unwrap_or(0).min(262_144);
            } else if kl == "authorization" && vt.len() > 7 && vt[..7].eq_ignore_ascii_case("bearer ") {
                bearer_token = Some(vt[7..].trim().to_string());
            }
        }
        hdr_line.clear();
    }

    let mut body = vec![0u8; content_len];
    if content_len > 0 {
        let _ = reader.read_exact(&mut body);
    }

    let mut stream = reader.into_inner();
    let path = target.split('?').next().unwrap_or("/");
    let expected = live_token(&app);

    if method == "OPTIONS" {
        write_http(&mut stream, "204 No Content", "text/plain", b"");
        return;
    }

    if method == "GET" && path == "/manifest.webmanifest" {
        write_http(
            &mut stream,
            "200 OK",
            "application/manifest+json; charset=utf-8",
            COMPANION_MANIFEST.as_bytes(),
        );
        return;
    }

    if method == "GET" && path == "/health" {
        write_http(&mut stream, "200 OK", "application/json", b"{\"ok\":true}");
        return;
    }

    if method == "GET" && (path == "/" || path == "/index.html") {
        write_http(
            &mut stream,
            "200 OK",
            "text/html; charset=utf-8",
            COMPANION_HTML.as_bytes(),
        );
        return;
    }

    if method == "POST" && (path == "/api/packet" || path == "/api/signal") {
        let raw_json = String::from_utf8_lossy(&body);
        let Ok(pkt) = parse_bridge_packet(&raw_json) else {
            write_http(
                &mut stream,
                "400 Bad Request",
                "application/json",
                b"{\"ok\":false,\"err\":\"json_invalido\"}",
            );
            return;
        };
        let supplied = query_token(&target).or(bearer_token).or_else(|| match &pkt {
            PhoneBridgePacket::Handshake { token, .. } => Some(token.clone()),
            _ => None,
        });
        if supplied.as_deref() != Some(expected.as_str()) {
            write_http(
                &mut stream,
                "401 Unauthorized",
                "application/json",
                b"{\"ok\":false,\"err\":\"token_invalido\"}",
            );
            return;
        }
        crate::assistant::ingest_phone_packet(&app, pkt);
        write_http(&mut stream, "200 OK", "application/json", b"{\"ok\":true}");
        return;
    }

    write_http(
        &mut stream,
        "404 Not Found",
        "application/json",
        b"{\"ok\":false,\"err\":\"not_found\"}",
    );
}

/// Inicia o servidor unificado HTTP (Companion PWA / Webhook) + TCP/NDJSON da ponte do celular.
pub fn start_listener(app: AppHandle, _expected_token: String, port_slot: &Mutex<u16>) {
    let listener = TcpListener::bind(("0.0.0.0", DEFAULT_PORT))
        .or_else(|_| TcpListener::bind(("0.0.0.0", 0)));

    let Ok(listener) = listener else {
        crate::logger::log("phone_bridge: falha ao abrir porta TCP");
        return;
    };
    let port = listener.local_addr().map(|a| a.port()).unwrap_or(DEFAULT_PORT);
    *port_slot.lock().unwrap() = port;
    crate::logger::log(format!("phone_bridge (HTTP+NDJSON) escutando em 0.0.0.0:{port}"));

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let app_c = app.clone();
            std::thread::spawn(move || {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(30)));
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                let Ok(n) = reader.read_line(&mut line) else {
                    return;
                };
                if n == 0 {
                    return;
                }
                let trimmed = line.trim();
                if trimmed.starts_with("GET ")
                    || trimmed.starts_with("POST ")
                    || trimmed.starts_with("OPTIONS ")
                    || trimmed.starts_with("HEAD ")
                {
                    handle_http_connection(reader, trimmed.to_string(), app_c);
                    return;
                }

                // Conexão persistente NDJSON TCP
                loop {
                    let t = line.trim();
                    if !t.is_empty() && t.starts_with('{') {
                        if let Ok(pkt) = parse_bridge_packet(t) {
                            let tok_c = live_token(&app_c);
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
                    match reader.read_line(&mut line) {
                        Ok(0) | Err(_) => break,
                        Ok(_) => {}
                    }
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
