use crate::domain::{PhoneDevice, PhoneSignalRecord, PhoneTransport};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PhoneBridgePacket {
    Handshake {
        device_id: String,
        device_name: String,
        platform: String,
        transport: PhoneTransport,
        token: String,
    },
    Battery {
        device_id: String,
        level: u8,
        charging: bool,
    },
    Notification {
        device_id: String,
        app: String,
        title: String,
        body: String,
    },
    IncomingCall {
        device_id: String,
        caller: String,
        number: String,
    },
    Clipboard {
        device_id: String,
        text: String,
    },
    Ping {
        device_id: String,
    },
}

pub fn build_qr_pairing_uri(host: &str, port: u16, token: &str, pc_name: &str) -> String {
    let encoded_name = pc_name.replace(' ', "%20");
    format!("ilha://pair?v=1&host={host}&port={port}&token={token}&pc={encoded_name}")
}

pub fn generate_pairing_token(seed: &str) -> String {
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    let hex = format!("{:x}", h.finalize());
    hex[..24].to_string()
}

pub fn parse_bridge_packet(raw: &str) -> Result<PhoneBridgePacket, String> {
    serde_json::from_str::<PhoneBridgePacket>(raw.trim()).map_err(|e| e.to_string())
}

pub fn packet_to_signal(pkt: &PhoneBridgePacket, ts: i64) -> Option<PhoneSignalRecord> {
    let uid = |kind: &str, dev: &str, txt: &str| {
        let mut h = Sha256::new();
        h.update(format!("{kind}|{dev}|{ts}|{txt}").as_bytes());
        format!("sig-{}", &format!("{:x}", h.finalize())[..12])
    };
    match pkt {
        PhoneBridgePacket::Notification { device_id, app, title, body } => Some(PhoneSignalRecord {
            id: uid("notif", device_id, title),
            device_id: device_id.clone(),
            kind: "notification".into(),
            title: title.clone(),
            body: body.clone(),
            app_name: app.clone(),
            ts,
        }),
        PhoneBridgePacket::IncomingCall { device_id, caller, number } => Some(PhoneSignalRecord {
            id: uid("call", device_id, caller),
            device_id: device_id.clone(),
            kind: "call".into(),
            title: if caller.is_empty() { number.clone() } else { caller.clone() },
            body: number.clone(),
            app_name: "Telefone".into(),
            ts,
        }),
        PhoneBridgePacket::Clipboard { device_id, text } => Some(PhoneSignalRecord {
            id: uid("clip", device_id, text),
            device_id: device_id.clone(),
            kind: "clipboard".into(),
            title: "Área de transferência do celular".into(),
            body: text.clone(),
            app_name: "Clipboard".into(),
            ts,
        }),
        PhoneBridgePacket::Battery { .. } | PhoneBridgePacket::Handshake { .. } | PhoneBridgePacket::Ping { .. } => None,
    }
}

pub fn merge_handshake_device(
    device_id: String,
    device_name: String,
    platform: String,
    transport: PhoneTransport,
    ts: i64,
) -> PhoneDevice {
    PhoneDevice {
        id: device_id,
        name: device_name,
        platform,
        connected: true,
        active_transport: transport,
        battery_pct: None,
        charging: false,
        last_seen_ts: ts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_incoming_call_and_qr_uri() {
        let raw = r#"{"type":"incoming_call","device_id":"px9","caller":"Mãe","number":"+5511999990000"}"#;
        let pkt = parse_bridge_packet(raw).unwrap();
        let sig = packet_to_signal(&pkt, 1700000000).unwrap();
        assert_eq!(sig.kind, "call");
        assert_eq!(sig.title, "Mãe");

        let qr = build_qr_pairing_uri("192.168.0.15", 47821, "abc123", "Meu PC");
        assert!(qr.starts_with("ilha://pair?v=1&host=192.168.0.15&port=47821"));
    }
}
