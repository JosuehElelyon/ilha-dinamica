//! Orquestrador do Assistente Pessoal Desktop (núcleo hexagonal + cofre + proatividade).

use assistant_core::bridge::PhoneBridgePacket;
use assistant_core::db::PersonalMemoryDb;
use assistant_core::domain::{
    AccountRecord, AssistantSnapshot, DevWorkItem, ImportantDate, PairingInfo, PhoneTransport,
    ProviderKind,
};
use assistant_core::normalizer::short_id;
use assistant_core::proactive::build_proactive_alerts;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_notification::NotificationExt;

use crate::{connectors, google, phone_bridge, win_calendar};

pub struct AssistantState {
    pub db: Mutex<Option<PersonalMemoryDb>>,
    pub dev_items: Mutex<Vec<DevWorkItem>>,
    pub bridge: phone_bridge::BridgeRuntime,
}

impl Default for AssistantState {
    fn default() -> Self {
        Self {
            db: Mutex::new(None),
            dev_items: Mutex::new(Vec::new()),
            bridge: phone_bridge::BridgeRuntime::default(),
        }
    }
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn init(app: &AppHandle) {
    let state = app.state::<AssistantState>();
    let db_path = app
        .path()
        .app_data_dir()
        .ok()
        .map(|d| d.join("assistant_memory.db"));

    if let Some(p) = db_path {
        match PersonalMemoryDb::open(&p) {
            Ok(db) => {
                *state.db.lock().unwrap() = Some(db);
            }
            Err(e) => {
                crate::logger::log(format!("assistant db erro: {e}"));
            }
        }
    }

    let tok = phone_bridge::ensure_token();
    *state.bridge.token.lock().unwrap() = tok.clone();
    phone_bridge::start_listener(app.clone(), tok, &state.bridge.port);

    // Sincronização inicial + ticker proativo em background (Windows + contas conectadas)
    let app_bg = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(3));
        let _ = sync_windows_into_db(&app_bg);
        emit_snapshot_and_notify(&app_bg);

        let app_async = app_bg.clone();
        tauri::async_runtime::spawn(async move {
            sync_all_connected_accounts(&app_async).await;
        });

        let mut tick: u32 = 0;
        loop {
            std::thread::sleep(Duration::from_secs(60));
            tick = tick.wrapping_add(1);
            if tick % 15 == 0 {
                let _ = sync_windows_into_db(&app_bg);
                let app_async = app_bg.clone();
                tauri::async_runtime::spawn(async move {
                    sync_all_connected_accounts(&app_async).await;
                });
            }
            emit_snapshot_and_notify(&app_bg);
        }
    });
}

async fn sync_all_connected_accounts(app: &AppHandle) {
    if google::load_tokens().is_some() {
        let _ = assistant_google_sync(app.clone()).await;
    }
    if connectors::has_github_token() {
        if let Ok((acc, items)) = connectors::sync_github().await {
            {
                let state = app.state::<AssistantState>();
                let guard = state.db.lock().unwrap();
                if let Some(db) = guard.as_ref() {
                    let _ = db.upsert_account(&acc);
                }
                *state.dev_items.lock().unwrap() = items;
            }
            emit_snapshot_and_notify(app);
        }
    }
    if connectors::load_apple_creds().is_some() {
        if let Ok((acc, contacts)) = connectors::sync_apple_icloud().await {
            {
                let state = app.state::<AssistantState>();
                let guard = state.db.lock().unwrap();
                if let Some(db) = guard.as_ref() {
                    let _ = db.upsert_account(&acc);
                    for (person, dates) in &contacts {
                        let _ = db.upsert_person(person);
                        for d in dates {
                            let _ = db.upsert_important_date(d);
                        }
                    }
                }
            }
            emit_snapshot_and_notify(app);
        }
    }
}

fn sync_windows_into_db(app: &AppHandle) -> Result<usize, String> {
    let events = win_calendar::fetch_windows_appointments(21)?;
    let count = events.len();
    let state = app.state::<AssistantState>();
    if let Some(db) = state.db.lock().unwrap().as_ref() {
        let _ = db.replace_account_events("windows:local", &events);
        let _ = db.upsert_account(&AccountRecord {
            id: "windows:local".into(),
            provider: ProviderKind::Windows,
            label: "Calendário do Windows".into(),
            email: std::env::var("USERNAME").unwrap_or_else(|_| "Local".into()),
            is_primary: false,
            status: "connected".into(),
            last_sync_ts: now_ts(),
            details: format!("{count} eventos locais (WinRT)"),
        });
    }
    Ok(count)
}

fn build_snapshot(app: &AppHandle) -> AssistantSnapshot {
    let state = app.state::<AssistantState>();
    let dev_items = state.dev_items.lock().unwrap().clone();
    let guard = state.db.lock().unwrap();
    let Some(db) = guard.as_ref() else {
        return AssistantSnapshot {
            google_connected: false,
            accounts: vec![],
            upcoming_dates: vec![],
            today_events: vec![],
            all_events: vec![],
            people_count: 0,
            phones: vec![],
            recent_signals: vec![],
            alerts: vec![],
            dev_items,
        };
    };

    let accounts = db.list_accounts().unwrap_or_default();
    let google_connected = accounts
        .iter()
        .any(|a| a.provider == ProviderKind::Google && a.status == "connected");
    let today = PersonalMemoryDb::today_local();
    let today_ymd = today.format("%Y-%m-%d").to_string();
    let upcoming_dates = db.list_upcoming_dates(today, 30).unwrap_or_default();
    let today_events = db.list_events_for_date(&today_ymd).unwrap_or_default();
    let all_events = db.list_upcoming_events(&today_ymd, 60).unwrap_or_default();
    let people_count = db.people_count().unwrap_or(0);
    let phones = db.list_phones().unwrap_or_default();
    let recent_signals = db.list_recent_signals(15).unwrap_or_default();
    let alerts = build_proactive_alerts(
        now_ts(),
        &upcoming_dates,
        &today_events,
        &phones,
        &recent_signals,
    );

    AssistantSnapshot {
        google_connected,
        accounts,
        upcoming_dates,
        today_events,
        all_events,
        people_count,
        phones,
        recent_signals,
        alerts,
        dev_items,
    }
}

fn emit_snapshot_and_notify(app: &AppHandle) {
    let snap = build_snapshot(app);
    let state = app.state::<AssistantState>();
    let now = now_ts();
    let cooldown_since = now - 12 * 3600;

    if let Some(db) = state.db.lock().unwrap().as_ref() {
        for alert in &snap.alerts {
            if !db.was_notified_recently(&alert.key, cooldown_since).unwrap_or(true) {
                let _ = db.mark_notified(&alert.key, now, "auto");
                let _ = app
                    .notification()
                    .builder()
                    .title(format!("{} {}", alert.icon, alert.title))
                    .body(&alert.body)
                    .show();
                let _ = app.emit("assistant-alert", alert);
            }
        }
    }
    let _ = app.emit("assistant-update", &snap);
}

pub fn ingest_phone_packet(app: &AppHandle, pkt: PhoneBridgePacket) {
    {
        let state = app.state::<AssistantState>();
        let guard = state.db.lock().unwrap();
        if let Some(db) = guard.as_ref() {
            phone_bridge::apply_packet_to_db(db, &pkt, app);
        }
    }
    emit_snapshot_and_notify(app);
}

/* ==================== COMANDOS TAURI ==================== */

#[tauri::command]
pub fn assistant_snapshot(app: AppHandle) -> AssistantSnapshot {
    build_snapshot(&app)
}

#[tauri::command]
pub async fn assistant_google_connect(
    app: AppHandle,
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<AssistantSnapshot, String> {
    google::start_oauth_pkce(client_id, client_secret).await?;
    assistant_google_sync(app).await
}

#[tauri::command]
pub async fn assistant_google_sync(app: AppHandle) -> Result<AssistantSnapshot, String> {
    let bundle = google::sync_google().await?;
    {
        let state = app.state::<AssistantState>();
        let guard = state.db.lock().unwrap();
        if let Some(db) = guard.as_ref() {
            db.upsert_account(&bundle.account)?;
            db.replace_account_events(&bundle.account.id, &bundle.events)?;
            for (person, dates) in &bundle.contacts {
                let _ = db.upsert_person(person);
                for d in dates {
                    let _ = db.upsert_important_date(d);
                }
            }
        }
    }
    emit_snapshot_and_notify(&app);
    Ok(build_snapshot(&app))
}

#[tauri::command]
pub fn assistant_google_disconnect(app: AppHandle) -> AssistantSnapshot {
    google::clear_tokens();
    {
        let state = app.state::<AssistantState>();
        let guard = state.db.lock().unwrap();
        if let Some(db) = guard.as_ref() {
            if let Ok(accs) = db.list_accounts() {
                for a in accs.into_iter().filter(|a| a.provider == ProviderKind::Google) {
                    let _ = db.remove_account(&a.id);
                }
            }
        }
    }
    emit_snapshot_and_notify(&app);
    build_snapshot(&app)
}

#[tauri::command(async)]
pub fn assistant_sync_windows(app: AppHandle) -> Result<AssistantSnapshot, String> {
    sync_windows_into_db(&app)?;
    emit_snapshot_and_notify(&app);
    Ok(build_snapshot(&app))
}

#[tauri::command]
pub async fn assistant_github_connect(
    app: AppHandle,
    token: String,
) -> Result<AssistantSnapshot, String> {
    connectors::save_github_token(&token)?;
    let (acc, items) = connectors::sync_github().await?;
    {
        let state = app.state::<AssistantState>();
        if let Some(db) = state.db.lock().unwrap().as_ref() {
            let _ = db.upsert_account(&acc);
        }
        *state.dev_items.lock().unwrap() = items;
    }
    emit_snapshot_and_notify(&app);
    Ok(build_snapshot(&app))
}

#[tauri::command]
pub async fn assistant_github_device_start(
    client_id: String,
) -> Result<connectors::GithubDeviceChallenge, String> {
    let ch = connectors::start_github_device_flow(&client_id).await?;
    let _ = crate::actions::run_action(ch.verification_uri.clone());
    Ok(ch)
}

#[tauri::command]
pub async fn assistant_github_device_poll(
    app: AppHandle,
    client_id: String,
    device_code: String,
) -> Result<Option<AssistantSnapshot>, String> {
    let Some(_tok) = connectors::poll_github_device_flow(&client_id, &device_code).await? else {
        return Ok(None);
    };
    let (acc, items) = connectors::sync_github().await?;
    {
        let state = app.state::<AssistantState>();
        if let Some(db) = state.db.lock().unwrap().as_ref() {
            let _ = db.upsert_account(&acc);
        }
        *state.dev_items.lock().unwrap() = items;
    }
    emit_snapshot_and_notify(&app);
    Ok(Some(build_snapshot(&app)))
}

#[tauri::command]
pub async fn assistant_apple_connect(
    app: AppHandle,
    apple_id: String,
    app_password: String,
) -> Result<AssistantSnapshot, String> {
    connectors::save_apple_creds(&apple_id, &app_password)?;
    let (acc, contacts) = connectors::sync_apple_icloud().await?;
    {
        let state = app.state::<AssistantState>();
        let guard = state.db.lock().unwrap();
        if let Some(db) = guard.as_ref() {
            let _ = db.upsert_account(&acc);
            for (person, dates) in &contacts {
                let _ = db.upsert_person(person);
                for d in dates {
                    let _ = db.upsert_important_date(d);
                }
            }
        }
    }
    emit_snapshot_and_notify(&app);
    Ok(build_snapshot(&app))
}

#[tauri::command]
pub fn assistant_disconnect_account(app: AppHandle, account_id: String) -> AssistantSnapshot {
    if account_id.starts_with("google:") {
        google::clear_tokens();
    } else if account_id.starts_with("github:") {
        let _ = connectors::save_github_token("");
        app.state::<AssistantState>().dev_items.lock().unwrap().clear();
    } else if account_id.starts_with("apple:") {
        let _ = connectors::save_apple_creds("", "");
    }
    if let Some(db) = app.state::<AssistantState>().db.lock().unwrap().as_ref() {
        let _ = db.remove_account(&account_id);
    }
    emit_snapshot_and_notify(&app);
    build_snapshot(&app)
}

#[tauri::command]
pub fn assistant_add_important_date(
    app: AppHandle,
    name: String,
    kind: String,
    month: u32,
    day: u32,
    year_optional: Option<i32>,
) -> Result<AssistantSnapshot, String> {
    let clean = name.trim().to_string();
    if clean.is_empty() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err("Informe um nome e uma data válida (dia/mês)".into());
    }
    let id = short_id("custom", &format!("{clean}-{kind}-{month}-{day}"));
    let label = if kind == "birthday" {
        format!("Aniversário de {clean}")
    } else {
        clean.clone()
    };
    if let Some(db) = app.state::<AssistantState>().db.lock().unwrap().as_ref() {
        db.upsert_important_date(&ImportantDate {
            id,
            person_id: None,
            person_name: if kind == "birthday" { clean } else { "".into() },
            kind,
            month,
            day,
            year_optional,
            label,
            source_provider: ProviderKind::Local,
        })?;
    }
    emit_snapshot_and_notify(&app);
    Ok(build_snapshot(&app))
}

#[tauri::command]
pub fn assistant_delete_important_date(app: AppHandle, id: String) -> AssistantSnapshot {
    if let Some(db) = app.state::<AssistantState>().db.lock().unwrap().as_ref() {
        let _ = db.delete_important_date(&id);
    }
    emit_snapshot_and_notify(&app);
    build_snapshot(&app)
}

#[tauri::command]
pub fn phone_pairing_info(state: State<AssistantState>) -> PairingInfo {
    let port = *state.bridge.port.lock().unwrap();
    let tok = state.bridge.token.lock().unwrap().clone();
    phone_bridge::get_pairing_info(port, &tok)
}

#[tauri::command]
pub fn phone_rotate_token(state: State<AssistantState>) -> PairingInfo {
    let tok = phone_bridge::rotate_token();
    *state.bridge.token.lock().unwrap() = tok.clone();
    let port = *state.bridge.port.lock().unwrap();
    phone_bridge::get_pairing_info(port, &tok)
}

#[tauri::command(async)]
pub fn phone_enable_usb_bridge(state: State<AssistantState>) -> Result<Vec<String>, String> {
    let port = *state.bridge.port.lock().unwrap();
    phone_bridge::enable_usb_reverse(port)
}

#[tauri::command]
pub fn phone_ingest_demo_signal(
    app: AppHandle,
    kind: String,
    title: String,
    body: String,
    transport: String,
) -> AssistantSnapshot {
    let tr = PhoneTransport::from_str(&transport);
    let dev_id = "demo-phone-1".to_string();
    ingest_phone_packet(
        &app,
        PhoneBridgePacket::Handshake {
            device_id: dev_id.clone(),
            device_name: "Smartphone Conectado".into(),
            platform: "android".into(),
            transport: tr,
            token: String::new(),
        },
    );
    let pkt = match kind.as_str() {
        "call" => PhoneBridgePacket::IncomingCall {
            device_id: dev_id,
            caller: title,
            number: body,
        },
        "battery" => PhoneBridgePacket::Battery {
            device_id: dev_id,
            level: title.parse::<u8>().unwrap_or(14),
            charging: false,
        },
        "clipboard" => PhoneBridgePacket::Clipboard {
            device_id: dev_id,
            text: body,
        },
        _ => PhoneBridgePacket::Notification {
            device_id: dev_id,
            app: "WhatsApp".into(),
            title,
            body,
        },
    };
    ingest_phone_packet(&app, pkt);
    build_snapshot(&app)
}

#[tauri::command]
pub fn phone_send_clipboard_to_phone(app: AppHandle) -> Result<String, String> {
    let text = arboard::Clipboard::new()
        .and_then(|mut c| c.get_text())
        .map_err(|e| format!("Falha ao ler clipboard do PC: {e}"))?;
    if text.trim().is_empty() {
        return Err("Área de transferência do PC está vazia.".into());
    }
    let preview: String = text.chars().take(48).collect();
    phone_bridge::enqueue_command(
        &app,
        assistant_core::bridge::PcToPhoneCommand::Clipboard {
            text,
            ts: now_ts(),
        },
    );
    Ok(preview)
}

#[tauri::command]
pub fn phone_ring_phone(app: AppHandle, message: Option<String>) {
    phone_bridge::enqueue_command(
        &app,
        assistant_core::bridge::PcToPhoneCommand::Ping {
            message: message.unwrap_or_else(|| "Ilha Dinâmica chamando este celular!".into()),
            ts: now_ts(),
        },
    );
}

#[tauri::command]
pub fn phone_set_auto_pause_on_call(state: State<AssistantState>, enabled: bool) -> bool {
    *state.bridge.auto_pause_on_call.lock().unwrap() = enabled;
    enabled
}
