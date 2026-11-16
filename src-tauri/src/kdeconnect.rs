//! Implementação do protocolo KDE Connect (v7) sobre UDP 1716 + TCP/TLS.
//! Permite parear diretamente com o app oficial KDE Connect (Android / iOS)
//! sem precisar instalar o KDE Connect Desktop no Windows.

use assistant_core::bridge::PhoneBridgePacket;
use assistant_core::domain::PhoneTransport;
use assistant_core::kdeconnect::{
    build_clipboard_packet, build_findmyphone_packet, build_identity_packet, build_pair_packet,
    build_ping_packet, kde_packet_to_bridge_packet, parse_identity, parse_kde_line,
    serialize_kde_line, KdePacket,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{DigitallySignedStruct, DistinguishedName, Error as RustlsError, SignatureScheme};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::mpsc;
use tokio_rustls::{TlsAcceptor, TlsConnector};

use crate::vault;

const VAULT_KDE_ID: &str = "kde:device_id";
const VAULT_KDE_CERT: &str = "kde:cert_der";
const VAULT_KDE_KEY: &str = "kde:key_der";
const VAULT_KDE_PAIRED: &str = "kde:paired_ids";
const KDE_DEFAULT_PORT: u16 = 1716;

static CRYPTO_INIT: OnceLock<()> = OnceLock::new();
fn ensure_crypto_provider() {
    CRYPTO_INIT.get_or_init(|| {
        let _ = rustls::crypto::ring::default_provider().install_default();
    });
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KdeDeviceInfo {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub ip: String,
    pub tcp_port: u16,
    pub paired: bool,
    pub pair_requested_by_peer: bool,
    pub connected: bool,
    pub battery_pct: Option<u8>,
    pub charging: bool,
    pub last_seen_ts: i64,
}

pub struct KdeConnectRuntime {
    pub device_id: Mutex<String>,
    pub device_name: Mutex<String>,
    pub listen_port: Mutex<u16>,
    pub cert_der: Mutex<Vec<u8>>,
    pub key_der: Mutex<Vec<u8>>,
    pub paired_ids: Mutex<HashSet<String>>,
    pub pending_outgoing_pair: Mutex<HashSet<String>>,
    pub devices: Mutex<HashMap<String, KdeDeviceInfo>>,
    pub senders: Mutex<HashMap<String, mpsc::UnboundedSender<String>>>,
}

impl Default for KdeConnectRuntime {
    fn default() -> Self {
        Self {
            device_id: Mutex::new(String::new()),
            device_name: Mutex::new("Ilha Dinamica".into()),
            listen_port: Mutex::new(KDE_DEFAULT_PORT),
            cert_der: Mutex::new(Vec::new()),
            key_der: Mutex::new(Vec::new()),
            paired_ids: Mutex::new(HashSet::new()),
            pending_outgoing_pair: Mutex::new(HashSet::new()),
            devices: Mutex::new(HashMap::new()),
            senders: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug)]
struct AcceptAnyPeerCert;

impl ServerCertVerifier for AcceptAnyPeerCert {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

impl ClientCertVerifier for AcceptAnyPeerCert {
    fn offer_client_auth(&self) -> bool {
        true
    }

    fn client_auth_mandatory(&self) -> bool {
        false
    }

    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, RustlsError> {
        Ok(ClientCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &rustls::crypto::ring::default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        rustls::crypto::ring::default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn load_or_create_identity(rt: &KdeConnectRuntime) {
    let dev_id = vault::get_secret(VAULT_KDE_ID)
        .filter(|s| s.len() >= 32)
        .unwrap_or_else(|| {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(format!("ilha-kde-{}-{}", now_ts(), std::process::id()).as_bytes());
            let hex = format!("{:x}", h.finalize());
            let id = hex[..32].to_string();
            let _ = vault::set_secret(VAULT_KDE_ID, &id);
            id
        });

    let pc_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .map(|n| format!("Ilha {n}"))
        .unwrap_or_else(|_| "Ilha Dinamica".into());

    let (cert_der, key_der) = match (
        vault::get_secret(VAULT_KDE_CERT).and_then(|s| B64.decode(s).ok()),
        vault::get_secret(VAULT_KDE_KEY).and_then(|s| B64.decode(s).ok()),
    ) {
        (Some(c), Some(k)) if !c.is_empty() && !k.is_empty() => (c, k),
        _ => {
            let mut params = rcgen::CertificateParams::new(vec![dev_id.clone()])
                .unwrap_or_else(|_| rcgen::CertificateParams::default());
            params
                .distinguished_name
                .push(rcgen::DnType::CommonName, dev_id.clone());
            let kp = rcgen::KeyPair::generate().expect("kde keypair");
            let cert = params.self_signed(&kp).expect("kde self-signed");
            let c = cert.der().to_vec();
            let k = kp.serialize_der();
            let _ = vault::set_secret(VAULT_KDE_CERT, &B64.encode(&c));
            let _ = vault::set_secret(VAULT_KDE_KEY, &B64.encode(&k));
            (c, k)
        }
    };

    let paired: HashSet<String> = vault::get_secret(VAULT_KDE_PAIRED)
        .map(|raw| {
            raw.split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default();

    *rt.device_id.lock().unwrap() = dev_id;
    *rt.device_name.lock().unwrap() = pc_name;
    *rt.cert_der.lock().unwrap() = cert_der;
    *rt.key_der.lock().unwrap() = key_der;
    *rt.paired_ids.lock().unwrap() = paired;
}

fn persist_paired_ids(rt: &KdeConnectRuntime) {
    let joined = rt
        .paired_ids
        .lock()
        .unwrap()
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(",");
    let _ = vault::set_secret(VAULT_KDE_PAIRED, &joined);
}

fn build_tls_server_config(rt: &KdeConnectRuntime) -> Result<Arc<rustls::ServerConfig>, String> {
    let cert = CertificateDer::from(rt.cert_der.lock().unwrap().clone());
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(rt.key_der.lock().unwrap().clone()));
    let verifier = Arc::new(AcceptAnyPeerCert);
    rustls::ServerConfig::builder()
        .with_client_cert_verifier(verifier)
        .with_single_cert(vec![cert], key)
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

fn build_tls_client_config(rt: &KdeConnectRuntime) -> Result<Arc<rustls::ClientConfig>, String> {
    let cert = CertificateDer::from(rt.cert_der.lock().unwrap().clone());
    let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(rt.key_der.lock().unwrap().clone()));
    let verifier = Arc::new(AcceptAnyPeerCert);
    rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(verifier)
        .with_client_auth_cert(vec![cert], key)
        .map(Arc::new)
        .map_err(|e| e.to_string())
}

fn build_my_identity(rt: &KdeConnectRuntime, include_port: bool) -> KdePacket {
    let id = rt.device_id.lock().unwrap().clone();
    let name = rt.device_name.lock().unwrap().clone();
    let port = if include_port {
        Some(*rt.listen_port.lock().unwrap())
    } else {
        None
    };
    build_identity_packet(&id, &name, port)
}

fn send_packet_to_peer(app: &AppHandle, peer_id: &str, pkt: &KdePacket) -> Result<(), String> {
    let line = serialize_kde_line(pkt)?;
    let state = app.state::<crate::assistant::AssistantState>();
    let guard = state.kde.senders.lock().unwrap();
    let tx = guard
        .get(peer_id)
        .ok_or_else(|| "Dispositivo KDE Connect não conectado no momento".to_string())?;
    tx.send(line).map_err(|e| e.to_string())
}

fn emit_devices_update(app: &AppHandle) {
    let list = list_devices(app);
    let _ = app.emit("kdeconnect-update", &list);
}

pub fn list_devices(app: &AppHandle) -> Vec<KdeDeviceInfo> {
    let state = app.state::<crate::assistant::AssistantState>();
    let mut list: Vec<KdeDeviceInfo> = state.kde.devices.lock().unwrap().values().cloned().collect();
    list.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(b.last_seen_ts.cmp(&a.last_seen_ts))
    });
    list
}

pub fn start(app: AppHandle) {
    ensure_crypto_provider();
    {
        let state = app.state::<crate::assistant::AssistantState>();
        load_or_create_identity(&state.kde);
    }

    tauri::async_runtime::spawn(async move {
        // 1. Inicia listener TCP (1716 ou próximo livre até 1764)
        let mut listener = None;
        let mut bound_port = KDE_DEFAULT_PORT;
        for port in KDE_DEFAULT_PORT..=1764 {
            if let Ok(l) = TcpListener::bind(("0.0.0.0", port)).await {
                bound_port = port;
                listener = Some(l);
                break;
            }
        }
        {
            let state = app.state::<crate::assistant::AssistantState>();
            *state.kde.listen_port.lock().unwrap() = bound_port;
        }
        crate::logger::log(format!("kdeconnect: TCP ouvindo na porta {bound_port}"));

        if let Some(tcp_listener) = listener {
            let app_tcp = app.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Ok((stream, addr)) = tcp_listener.accept().await {
                        let app_c = app_tcp.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = handle_incoming_tcp(app_c, stream, addr).await;
                        });
                    }
                }
            });
        }

        // 2. Inicia UDP discovery na porta 1716
        if let Ok(udp) = UdpSocket::bind(("0.0.0.0", KDE_DEFAULT_PORT)).await {
            let _ = udp.set_broadcast(true);
            let app_udp = app.clone();
            let udp_arc = Arc::new(udp);
            let udp_rx = udp_arc.clone();

            // Broadcast inicial
            let _ = send_udp_broadcast(&app, &udp_arc).await;

            tauri::async_runtime::spawn(async move {
                let mut buf = vec![0u8; 8192];
                loop {
                    let Ok((len, src)) = udp_rx.recv_from(&mut buf).await else {
                        continue;
                    };
                    let txt = String::from_utf8_lossy(&buf[..len]);
                    let Ok(pkt) = parse_kde_line(&txt) else {
                        continue;
                    };
                    let Some(ident) = parse_identity(&pkt) else {
                        continue;
                    };
                    let my_id = {
                        let state = app_udp.state::<crate::assistant::AssistantState>();
                        state.kde.device_id.lock().unwrap().clone()
                    };
                    if ident.device_id == my_id {
                        continue;
                    }
                    let peer_ip = src.ip().to_string();
                    let peer_port = ident.tcp_port.unwrap_or(KDE_DEFAULT_PORT);
                    let already_connected = {
                        let state = app_udp.state::<crate::assistant::AssistantState>();
                        let paired = state.kde.paired_ids.lock().unwrap().contains(&ident.device_id);
                        let mut devs = state.kde.devices.lock().unwrap();
                        let entry = devs.entry(ident.device_id.clone()).or_insert_with(|| KdeDeviceInfo {
                            device_id: ident.device_id.clone(),
                            device_name: ident.device_name.clone(),
                            device_type: ident.device_type.clone(),
                            ip: peer_ip.clone(),
                            tcp_port: peer_port,
                            paired,
                            pair_requested_by_peer: false,
                            connected: false,
                            battery_pct: None,
                            charging: false,
                            last_seen_ts: now_ts(),
                        });
                        entry.device_name = ident.device_name.clone();
                        entry.ip = peer_ip.clone();
                        entry.tcp_port = peer_port;
                        entry.paired = paired;
                        entry.last_seen_ts = now_ts();
                        state.kde.senders.lock().unwrap().contains_key(&ident.device_id)
                    };
                    emit_devices_update(&app_udp);

                    if !already_connected {
                        let app_conn = app_udp.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = connect_to_peer(app_conn, peer_ip, peer_port).await;
                        });
                    }
                }
            });
        } else {
            crate::logger::log("kdeconnect: porta UDP 1716 ocupada (outro daemon ativo?)");
        }
    });
}

async fn send_udp_broadcast(app: &AppHandle, udp: &UdpSocket) -> Result<(), String> {
    let pkt = {
        let state = app.state::<crate::assistant::AssistantState>();
        build_my_identity(&state.kde, true)
    };
    let line = serialize_kde_line(&pkt)?;
    let _ = udp.send_to(line.as_bytes(), ("255.255.255.255", KDE_DEFAULT_PORT)).await;
    Ok(())
}

async fn handle_incoming_tcp(
    app: AppHandle,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut first_line = String::new();
    let n = tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut first_line))
        .await
        .map_err(|_| "Timeout aguardando identidade KDE Connect".to_string())?
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("Conexão encerrada".into());
    }
    let pkt = parse_kde_line(&first_line)?;
    let ident = parse_identity(&pkt).ok_or("Pacote inicial não é kdeconnect.identity")?;

    let tls_cfg = {
        let state = app.state::<crate::assistant::AssistantState>();
        build_tls_client_config(&state.kde)?
    };
    let connector = TlsConnector::from(tls_cfg);
    let server_name = ServerName::try_from(ident.device_id.clone())
        .unwrap_or_else(|_| ServerName::try_from("kdeconnect.local".to_string()).unwrap());
    let tls_stream = connector
        .connect(server_name, reader.into_inner())
        .await
        .map_err(|e| format!("TLS client handshake falhou: {e}"))?;

    run_tls_session(app, tls_stream, ident.device_id, ident.device_name, ident.device_type, addr.ip().to_string(), ident.tcp_port.unwrap_or(KDE_DEFAULT_PORT)).await
}

async fn connect_to_peer(app: AppHandle, ip: String, port: u16) -> Result<(), String> {
    let mut stream = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect((ip.as_str(), port)))
        .await
        .map_err(|_| "Timeout TCP ao conectar ao celular".to_string())?
        .map_err(|e| e.to_string())?;

    let (my_id_line, tls_cfg) = {
        let state = app.state::<crate::assistant::AssistantState>();
        let id_pkt = build_my_identity(&state.kde, false);
        (serialize_kde_line(&id_pkt)?, build_tls_server_config(&state.kde)?)
    };
    stream
        .write_all(my_id_line.as_bytes())
        .await
        .map_err(|e| e.to_string())?;

    let acceptor = TlsAcceptor::from(tls_cfg);
    let tls_stream = acceptor
        .accept(stream)
        .await
        .map_err(|e| format!("TLS server handshake falhou: {e}"))?;

    // Descobre device_id já conhecido pelo IP ou usa IP temporário até chegar pacote/identidade
    let (peer_id, peer_name, peer_type) = {
        let state = app.state::<crate::assistant::AssistantState>();
        let devs = state.kde.devices.lock().unwrap();
        devs.values()
            .find(|d| d.ip == ip)
            .map(|d| (d.device_id.clone(), d.device_name.clone(), d.device_type.clone()))
            .unwrap_or_else(|| (format!("kde-{ip}"), format!("Celular ({ip})"), "phone".into()))
    };

    run_tls_session(app, tls_stream, peer_id, peer_name, peer_type, ip, port).await
}

async fn run_tls_session<S>(
    app: AppHandle,
    stream: S,
    peer_id: String,
    peer_name: String,
    peer_type: String,
    ip: String,
    tcp_port: u16,
) -> Result<(), String>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    let (read_half, mut write_half) = tokio::io::split(stream);
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let is_paired = {
        let state = app.state::<crate::assistant::AssistantState>();
        let paired = state.kde.paired_ids.lock().unwrap().contains(&peer_id);
        let mut devs = state.kde.devices.lock().unwrap();
        let entry = devs.entry(peer_id.clone()).or_insert_with(|| KdeDeviceInfo {
            device_id: peer_id.clone(),
            device_name: peer_name.clone(),
            device_type: peer_type.clone(),
            ip: ip.clone(),
            tcp_port,
            paired,
            pair_requested_by_peer: false,
            connected: true,
            battery_pct: None,
            charging: false,
            last_seen_ts: now_ts(),
        });
        entry.connected = true;
        entry.paired = paired;
        entry.ip = ip.clone();
        entry.last_seen_ts = now_ts();
        state.kde.senders.lock().unwrap().insert(peer_id.clone(), tx);
        paired
    };

    if is_paired {
        crate::assistant::ingest_phone_packet(
            &app,
            PhoneBridgePacket::Handshake {
                device_id: peer_id.clone(),
                device_name: peer_name.clone(),
                platform: if peer_type == "phone" { "android".into() } else { peer_type.clone() },
                transport: PhoneTransport::Lan,
                token: "kdeconnect".into(),
            },
        );
    }
    emit_devices_update(&app);

    let write_task = tauri::async_runtime::spawn(async move {
        while let Some(line) = rx.recv().await {
            if write_half.write_all(line.as_bytes()).await.is_err() {
                break;
            }
            let _ = write_half.flush().await;
        }
    });

    let mut lines = BufReader::new(read_half).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let Ok(pkt) = parse_kde_line(&line) else {
            continue;
        };
        if pkt.kind == "kdeconnect.pair" {
            let pair = pkt.body["pair"].as_bool().unwrap_or(false);
            if pair {
                let accept_auto = {
                    let state = app.state::<crate::assistant::AssistantState>();
                    let was_pending = state.kde.pending_outgoing_pair.lock().unwrap().remove(&peer_id);
                    let already = state.kde.paired_ids.lock().unwrap().contains(&peer_id);
                    if was_pending || already {
                        state.kde.paired_ids.lock().unwrap().insert(peer_id.clone());
                        persist_paired_ids(&state.kde);
                        if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&peer_id) {
                            d.paired = true;
                            d.pair_requested_by_peer = false;
                        }
                        true
                    } else {
                        if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&peer_id) {
                            d.pair_requested_by_peer = true;
                        }
                        false
                    }
                };
                if accept_auto {
                    let _ = send_packet_to_peer(&app, &peer_id, &build_pair_packet(true));
                    crate::assistant::ingest_phone_packet(
                        &app,
                        PhoneBridgePacket::Handshake {
                            device_id: peer_id.clone(),
                            device_name: peer_name.clone(),
                            platform: "android".into(),
                            transport: PhoneTransport::Lan,
                            token: "kdeconnect".into(),
                        },
                    );
                } else {
                    let _ = app.emit("kdeconnect-pair-request", &peer_id);
                }
            } else {
                let state = app.state::<crate::assistant::AssistantState>();
                state.kde.paired_ids.lock().unwrap().remove(&peer_id);
                persist_paired_ids(&state.kde);
                if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&peer_id) {
                    d.paired = false;
                    d.pair_requested_by_peer = false;
                }
            }
            emit_devices_update(&app);
            continue;
        }

        let paired_now = {
            let state = app.state::<crate::assistant::AssistantState>();
            state.kde.paired_ids.lock().unwrap().contains(&peer_id)
        };
        if !paired_now {
            continue;
        }

        if let Some(bridge_pkt) = kde_packet_to_bridge_packet(&peer_id, &pkt) {
            if let PhoneBridgePacket::Battery { level, charging, .. } = &bridge_pkt {
                let state = app.state::<crate::assistant::AssistantState>();
                if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&peer_id) {
                    d.battery_pct = Some(*level);
                    d.charging = *charging;
                    d.last_seen_ts = now_ts();
                }
                emit_devices_update(&app);
            }
            if let PhoneBridgePacket::IncomingCall { .. } = &bridge_pkt {
                let auto_pause = {
                    let state = app.state::<crate::assistant::AssistantState>();
                    let v = *state.bridge.auto_pause_on_call.lock().unwrap();
                    v
                };
                if auto_pause {
                    crate::media::media_cmd("pause".into());
                }
            }
            crate::assistant::ingest_phone_packet(&app, bridge_pkt);
        }
    }

    write_task.abort();
    {
        let state = app.state::<crate::assistant::AssistantState>();
        state.kde.senders.lock().unwrap().remove(&peer_id);
        if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&peer_id) {
            d.connected = false;
        }
    }
    emit_devices_update(&app);
    Ok(())
}

/* ==================== COMANDOS TAURI ==================== */

#[tauri::command]
pub fn kdeconnect_list_devices(app: AppHandle) -> Vec<KdeDeviceInfo> {
    list_devices(&app)
}

#[tauri::command]
pub async fn kdeconnect_scan(app: AppHandle) -> Result<Vec<KdeDeviceInfo>, String> {
    if let Ok(udp) = UdpSocket::bind("0.0.0.0:0").await {
        let _ = udp.set_broadcast(true);
        send_udp_broadcast(&app, &udp).await?;
    }
    Ok(list_devices(&app))
}

#[tauri::command]
pub async fn kdeconnect_connect_ip(
    app: AppHandle,
    ip: String,
    port: Option<u16>,
) -> Result<Vec<KdeDeviceInfo>, String> {
    let clean_ip = ip.trim().to_string();
    if clean_ip.is_empty() {
        return Err("Informe o IP do celular".into());
    }
    let p = port.unwrap_or(KDE_DEFAULT_PORT);
    let app_c = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = connect_to_peer(app_c, clean_ip, p).await;
    });
    tokio::time::sleep(Duration::from_millis(600)).await;
    Ok(list_devices(&app))
}

#[tauri::command]
pub fn kdeconnect_pair(
    app: AppHandle,
    device_id: String,
    accept: bool,
) -> Result<Vec<KdeDeviceInfo>, String> {
    let (was_requested_by_peer, device_name) = {
        let state = app.state::<crate::assistant::AssistantState>();
        let devs = state.kde.devices.lock().unwrap();
        devs.get(&device_id)
            .map(|d| (d.pair_requested_by_peer, d.device_name.clone()))
            .unwrap_or((false, "Celular".into()))
    };

    if accept {
        let state = app.state::<crate::assistant::AssistantState>();
        if was_requested_by_peer {
            state.kde.paired_ids.lock().unwrap().insert(device_id.clone());
            persist_paired_ids(&state.kde);
            if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&device_id) {
                d.paired = true;
                d.pair_requested_by_peer = false;
            }
            let _ = send_packet_to_peer(&app, &device_id, &build_pair_packet(true));
            crate::assistant::ingest_phone_packet(
                &app,
                PhoneBridgePacket::Handshake {
                    device_id: device_id.clone(),
                    device_name,
                    platform: "android".into(),
                    transport: PhoneTransport::Lan,
                    token: "kdeconnect".into(),
                },
            );
        } else {
            state.kde.pending_outgoing_pair.lock().unwrap().insert(device_id.clone());
            send_packet_to_peer(&app, &device_id, &build_pair_packet(true))?;
        }
    } else {
        let state = app.state::<crate::assistant::AssistantState>();
        state.kde.paired_ids.lock().unwrap().remove(&device_id);
        state.kde.pending_outgoing_pair.lock().unwrap().remove(&device_id);
        persist_paired_ids(&state.kde);
        if let Some(d) = state.kde.devices.lock().unwrap().get_mut(&device_id) {
            d.paired = false;
            d.pair_requested_by_peer = false;
        }
        let _ = send_packet_to_peer(&app, &device_id, &build_pair_packet(false));
    }
    emit_devices_update(&app);
    Ok(list_devices(&app))
}

#[tauri::command]
pub fn kdeconnect_find_my_phone(app: AppHandle, device_id: String) -> Result<(), String> {
    send_packet_to_peer(&app, &device_id, &build_findmyphone_packet())
}

#[tauri::command]
pub fn kdeconnect_send_clipboard(app: AppHandle, device_id: String) -> Result<String, String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let text = cb.get_text().map_err(|e| format!("Clipboard vazio: {e}"))?;
    if text.trim().is_empty() {
        return Err("Nenhum texto copiado no PC".into());
    }
    send_packet_to_peer(&app, &device_id, &build_clipboard_packet(&text))?;
    Ok(text.chars().take(40).collect())
}

#[tauri::command]
pub fn kdeconnect_ping(app: AppHandle, device_id: String, message: Option<String>) -> Result<(), String> {
    send_packet_to_peer(
        &app,
        &device_id,
        &build_ping_packet(message.as_deref().unwrap_or("Olá da Ilha Dinâmica!")),
    )
}
