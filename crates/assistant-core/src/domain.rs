use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Google,
    Windows,
    #[serde(rename = "github")]
    GitHub,
    #[serde(rename = "gitlab")]
    GitLab,
    Apple,
    Local,
}

impl ProviderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Windows => "windows",
            Self::GitHub => "github",
            Self::GitLab => "gitlab",
            Self::Apple => "apple",
            Self::Local => "local",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "google" => Self::Google,
            "windows" => Self::Windows,
            "github" => Self::GitHub,
            "gitlab" => Self::GitLab,
            "apple" => Self::Apple,
            _ => Self::Local,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecord {
    pub id: String,
    pub provider: ProviderKind,
    pub label: String,
    pub email: String,
    pub is_primary: bool,
    pub status: String, // "connected" | "syncing" | "error" | "disconnected"
    pub last_sync_ts: i64,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalEvent {
    pub id: String,
    pub fingerprint: String,
    pub title: String,
    pub category: String, // "meeting" | "reminder" | "dev" | "holiday" | "personal"
    pub date: String,     // YYYY-MM-DD
    pub start_time: String,
    pub end_time: String,
    pub start_ts: i64,
    pub end_ts: i64,
    pub all_day: bool,
    pub join_url: String,
    pub source_provider: ProviderKind,
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonRecord {
    pub id: String,
    pub name: String,
    pub relation: String,
    pub email: String,
    pub phone: String,
    pub source_provider: ProviderKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportantDate {
    pub id: String,
    pub person_id: Option<String>,
    pub person_name: String,
    pub kind: String, // "birthday" | "anniversary" | "holiday" | "custom"
    pub month: u32,
    pub day: u32,
    pub year_optional: Option<i32>,
    pub label: String,
    pub source_provider: ProviderKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpcomingDate {
    pub id: String,
    pub title: String,
    pub person_name: String,
    pub kind: String,
    pub month: u32,
    pub day: u32,
    pub days_until: i64,
    pub turning_age: Option<i32>,
    pub source_provider: ProviderKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PhoneTransport {
    Usb,
    Lan,
    Bluetooth,
}

impl PhoneTransport {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Usb => "usb",
            Self::Lan => "lan",
            Self::Bluetooth => "bluetooth",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "usb" => Self::Usb,
            "bluetooth" | "bt" => Self::Bluetooth,
            _ => Self::Lan,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneDevice {
    pub id: String,
    pub name: String,
    pub platform: String, // "android" | "ios"
    pub connected: bool,
    pub active_transport: PhoneTransport,
    pub battery_pct: Option<u8>,
    pub charging: bool,
    pub last_seen_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneSignalRecord {
    pub id: String,
    pub device_id: String,
    pub kind: String, // "notification" | "call" | "battery" | "clipboard"
    pub title: String,
    pub body: String,
    pub app_name: String,
    pub ts: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AvatarMood {
    Idle,
    Celebrate,
    Alert,
    Meeting,
    Briefing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProactiveAlert {
    pub key: String,
    pub level: String, // "urgent" | "celebrate" | "normal"
    pub avatar_mood: AvatarMood,
    pub icon: String,
    pub title: String,
    pub body: String,
    pub bubble_text: String,
    pub action_label: String,
    pub action_target: String,
    pub created_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DevWorkItem {
    pub id: String,
    pub provider: ProviderKind,
    pub repo: String,
    pub title: String,
    pub kind: String, // "review" | "pr" | "issue" | "notification"
    pub url: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairingInfo {
    pub qr_uri: String,
    pub companion_url: String,
    pub webhook_url: String,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub adb_available: bool,
    pub adb_devices: Vec<String>,
    pub bt_paired_phones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantSnapshot {
    pub google_connected: bool,
    pub accounts: Vec<AccountRecord>,
    pub upcoming_dates: Vec<UpcomingDate>,
    pub today_events: Vec<CanonicalEvent>,
    pub all_events: Vec<CanonicalEvent>,
    pub people_count: usize,
    pub phones: Vec<PhoneDevice>,
    pub recent_signals: Vec<PhoneSignalRecord>,
    pub alerts: Vec<ProactiveAlert>,
    pub dev_items: Vec<DevWorkItem>,
}
