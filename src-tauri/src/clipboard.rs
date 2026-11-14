//! Histórico das últimas 8 coisas copiadas — só em memória, nunca salvo em disco.

use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Clone, Serialize)]
pub struct Clip {
    text: String,
    t: u64,
}

#[derive(Default)]
pub struct ClipState {
    clips: Mutex<Vec<Clip>>,
    last: Mutex<String>,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut cb = match arboard::Clipboard::new() {
            Ok(c) => c,
            Err(_) => return,
        };
        {
            let st = app.state::<ClipState>();
            *st.last.lock().unwrap() = cb.get_text().unwrap_or_default();
        }
        loop {
            std::thread::sleep(Duration::from_millis(700));
            let Ok(text) = cb.get_text() else { continue };
            if text.is_empty() {
                continue;
            }
            let st = app.state::<ClipState>();
            {
                let mut last = st.last.lock().unwrap();
                if *last == text {
                    continue;
                }
                *last = text.clone();
            }
            let list = {
                let mut clips = st.clips.lock().unwrap();
                clips.retain(|c| c.text != text);
                clips.insert(0, Clip { text, t: now_ms() });
                clips.truncate(8);
                clips.clone()
            };
            let _ = app.emit("clips", list);
        }
    });
}

pub fn set_text_from_app(app: &AppHandle, text: String) {
    let state = app.state::<ClipState>();
    *state.last.lock().unwrap() = text.clone();
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

#[tauri::command]
pub fn copy_text(state: State<ClipState>, text: String) {
    *state.last.lock().unwrap() = text.clone();
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(text);
    }
}

#[tauri::command]
pub fn clear_clips(app: AppHandle, state: State<ClipState>) {
    state.clips.lock().unwrap().clear();
    let _ = app.emit("clips", Vec::<Clip>::new());
}
