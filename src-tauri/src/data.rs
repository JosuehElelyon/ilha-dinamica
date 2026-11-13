//! Notas, lembretes e configurações, salvos em %APPDATA%\com.josueh.ilhadinamica\data.json

use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_autostart::ManagerExt;

#[derive(Default)]
pub struct DataState {
    pub value: Mutex<Value>,
    pub path: Mutex<Option<PathBuf>>,
}

pub fn load(app: &AppHandle) {
    let state = app.state::<DataState>();
    let path = app.path().app_data_dir().ok().map(|d| d.join("data.json"));
    let mut v: Value = path
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({}));
    if !v.is_object() {
        v = json!({});
    }
    let o = v.as_object_mut().unwrap();
    o.entry("notes").or_insert(json!(""));
    o.entry("settings").or_insert(json!({}));
    o.entry("reminders").or_insert(json!([]));
    *state.value.lock().unwrap() = v;
    *state.path.lock().unwrap() = path;
}

fn persist(state: &DataState) {
    let path = state.path.lock().unwrap().clone();
    if let Some(p) = path {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let text = serde_json::to_string_pretty(&*state.value.lock().unwrap()).unwrap_or_default();
        let _ = std::fs::write(p, text);
    }
}

/// Altera uma configuração a partir do Rust (ex.: posição da janela) e salva
pub fn set_setting(app: &AppHandle, key: &str, value: Value) {
    let state = app.state::<DataState>();
    {
        let mut v = state.value.lock().unwrap();
        if value.is_null() {
            if let Some(o) = v["settings"].as_object_mut() {
                o.remove(key);
            }
        } else {
            v["settings"][key] = value;
        }
    }
    persist(&state);
}

#[tauri::command]
pub fn get_data(app: AppHandle, state: State<DataState>) -> Value {
    let mut v = state.value.lock().unwrap().clone();
    let autostart = app.autolaunch().is_enabled().unwrap_or(false);
    if let Some(o) = v.as_object_mut() {
        o.insert("platform".into(), json!(std::env::consts::OS));
        o.insert("isAdmin".into(), json!(crate::util::is_admin()));
        o.insert("autostart".into(), json!(autostart));
        o.insert("protectedNames".into(), json!(crate::system::PROTECTED));
        o.insert("selfName".into(), json!(crate::system::self_name()));
        o.insert("ownPids".into(), json!([std::process::id()]));
        o.insert("version".into(), json!(env!("CARGO_PKG_VERSION")));
        if let Some(win) = app.get_webview_window("main") {
            o.insert("anchor".into(), json!(crate::position::anchor(&win)));
        }
    }
    v
}

#[tauri::command]
pub fn save_notes(state: State<DataState>, text: String) {
    state.value.lock().unwrap()["notes"] = json!(text);
    persist(&state);
}

#[tauri::command]
pub fn save_reminders(state: State<DataState>, reminders: Value) {
    state.value.lock().unwrap()["reminders"] = reminders;
    persist(&state);
}

#[tauri::command]
pub fn save_setting(state: State<DataState>, key: String, value: Value) {
    state.value.lock().unwrap()["settings"][key] = value;
    persist(&state);
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> bool {
    let al = app.autolaunch();
    let _ = if enabled { al.enable() } else { al.disable() };
    crate::tray::refresh(&app);
    al.is_enabled().unwrap_or(false)
}
