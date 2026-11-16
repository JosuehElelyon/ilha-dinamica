//! Protocolo KDE Connect (v7) — serialização e tradução para `PhoneBridgePacket`.

use crate::bridge::PhoneBridgePacket;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdePacket {
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub body: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KdeIdentity {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub protocol_version: u32,
    pub tcp_port: Option<u16>,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

pub fn sanitize_device_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| !"\"',;:.!?()[]<>".contains(*c))
        .take(32)
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        "Ilha Dinamica".into()
    } else {
        trimmed.to_string()
    }
}

pub fn build_identity_packet(
    device_id: &str,
    device_name: &str,
    tcp_port: Option<u16>,
) -> KdePacket {
    let mut body = json!({
        "deviceId": device_id,
        "deviceName": sanitize_device_name(device_name),
        "deviceType": "desktop",
        "protocolVersion": 7,
        "incomingCapabilities": [
            "kdeconnect.battery",
            "kdeconnect.notification",
            "kdeconnect.telephony",
            "kdeconnect.clipboard",
            "kdeconnect.clipboard.connect",
            "kdeconnect.ping"
        ],
        "outgoingCapabilities": [
            "kdeconnect.findmyphone.request",
            "kdeconnect.clipboard",
            "kdeconnect.ping"
        ]
    });
    if let Some(p) = tcp_port {
        body["tcpPort"] = json!(p);
    }
    KdePacket {
        id: now_ms(),
        kind: "kdeconnect.identity".into(),
        body,
    }
}

pub fn parse_kde_line(line: &str) -> Result<KdePacket, String> {
    serde_json::from_str::<KdePacket>(line.trim()).map_err(|e| e.to_string())
}

pub fn serialize_kde_line(pkt: &KdePacket) -> Result<String, String> {
    let mut s = serde_json::to_string(pkt).map_err(|e| e.to_string())?;
    s.push('\n');
    Ok(s)
}

pub fn parse_identity(pkt: &KdePacket) -> Option<KdeIdentity> {
    if pkt.kind != "kdeconnect.identity" {
        return None;
    }
    let device_id = pkt.body["deviceId"].as_str()?.trim().to_string();
    if device_id.is_empty() {
        return None;
    }
    let device_name = pkt.body["deviceName"]
        .as_str()
        .unwrap_or("Celular")
        .trim()
        .to_string();
    let device_type = pkt.body["deviceType"]
        .as_str()
        .unwrap_or("phone")
        .to_string();
    let protocol_version = pkt.body["protocolVersion"].as_u64().unwrap_or(7) as u32;
    let tcp_port = pkt.body["tcpPort"].as_u64().map(|p| p as u16);
    Some(KdeIdentity {
        device_id,
        device_name,
        device_type,
        protocol_version,
        tcp_port,
    })
}

pub fn build_pair_packet(pair: bool) -> KdePacket {
    KdePacket {
        id: now_ms(),
        kind: "kdeconnect.pair".into(),
        body: json!({ "pair": pair }),
    }
}

pub fn build_findmyphone_packet() -> KdePacket {
    KdePacket {
        id: now_ms(),
        kind: "kdeconnect.findmyphone.request".into(),
        body: json!({}),
    }
}

pub fn build_clipboard_packet(text: &str) -> KdePacket {
    KdePacket {
        id: now_ms(),
        kind: "kdeconnect.clipboard".into(),
        body: json!({ "content": text }),
    }
}

pub fn build_ping_packet(message: &str) -> KdePacket {
    KdePacket {
        id: now_ms(),
        kind: "kdeconnect.ping".into(),
        body: if message.is_empty() {
            json!({})
        } else {
            json!({ "message": message })
        },
    }
}

pub fn kde_packet_to_bridge_packet(device_id: &str, pkt: &KdePacket) -> Option<PhoneBridgePacket> {
    match pkt.kind.as_str() {
        "kdeconnect.battery" => {
            let charge = pkt.body["currentCharge"].as_i64().unwrap_or(-1);
            if charge < 0 {
                return None;
            }
            let level = charge.clamp(0, 100) as u8;
            let charging = pkt.body["isCharging"].as_bool().unwrap_or(false);
            Some(PhoneBridgePacket::Battery {
                device_id: device_id.to_string(),
                level,
                charging,
            })
        }
        "kdeconnect.notification" => {
            if pkt.body["isCancel"].as_bool().unwrap_or(false) {
                return None;
            }
            let app = pkt.body["appName"]
                .as_str()
                .unwrap_or("Celular")
                .to_string();
            let title = pkt.body["title"]
                .as_str()
                .or_else(|| pkt.body["ticker"].as_str())
                .unwrap_or("Notificação")
                .to_string();
            let body = pkt.body["text"].as_str().unwrap_or("").to_string();
            Some(PhoneBridgePacket::Notification {
                device_id: device_id.to_string(),
                app,
                title,
                body,
            })
        }
        "kdeconnect.telephony" => {
            let ev = pkt.body["event"].as_str().unwrap_or("");
            if ev != "ringing" {
                return None;
            }
            let number = pkt.body["phoneNumber"].as_str().unwrap_or("").to_string();
            let caller = pkt.body["contactName"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or(&number)
                .to_string();
            Some(PhoneBridgePacket::IncomingCall {
                device_id: device_id.to_string(),
                caller,
                number,
            })
        }
        "kdeconnect.clipboard" | "kdeconnect.clipboard.connect" => {
            let text = pkt.body["content"].as_str().unwrap_or("").trim().to_string();
            if text.is_empty() {
                return None;
            }
            Some(PhoneBridgePacket::Clipboard {
                device_id: device_id.to_string(),
                text,
            })
        }
        "kdeconnect.ping" => Some(PhoneBridgePacket::Ping {
            device_id: device_id.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_identity_and_translates_core_packets() {
        let id_pkt = build_identity_packet("ilha_dinamica_pc_0000000000000001", "Ilha (PC)!", Some(1716));
        let line = serialize_kde_line(&id_pkt).unwrap();
        let parsed = parse_kde_line(&line).unwrap();
        let ident = parse_identity(&parsed).unwrap();
        assert_eq!(ident.device_name, "Ilha PC");
        assert_eq!(ident.tcp_port, Some(1716));

        let bat = parse_kde_line(r#"{"id":1,"type":"kdeconnect.battery","body":{"currentCharge":84,"isCharging":true}}"#).unwrap();
        match kde_packet_to_bridge_packet("dev1", &bat).unwrap() {
            PhoneBridgePacket::Battery { level, charging, .. } => {
                assert_eq!(level, 84);
                assert!(charging);
            }
            _ => panic!("expected battery"),
        }

        let notif = parse_kde_line(r#"{"id":2,"type":"kdeconnect.notification","body":{"appName":"WhatsApp","title":"Amanda","text":"Oi!"}}"#).unwrap();
        match kde_packet_to_bridge_packet("dev1", &notif).unwrap() {
            PhoneBridgePacket::Notification { app, title, body, .. } => {
                assert_eq!(app, "WhatsApp");
                assert_eq!(title, "Amanda");
                assert_eq!(body, "Oi!");
            }
            _ => panic!("expected notification"),
        }

        let call = parse_kde_line(r#"{"id":3,"type":"kdeconnect.telephony","body":{"event":"ringing","contactName":"Mãe","phoneNumber":"+5511999990000"}}"#).unwrap();
        match kde_packet_to_bridge_packet("dev1", &call).unwrap() {
            PhoneBridgePacket::IncomingCall { caller, number, .. } => {
                assert_eq!(caller, "Mãe");
                assert_eq!(number, "+5511999990000");
            }
            _ => panic!("expected incoming call"),
        }
    }
}
