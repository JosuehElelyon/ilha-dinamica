//! Arrastar a ilha para qualquer lugar da tela, lembrar a posição e
//! decidir para que lado ela abre (âncora) conforme onde ela está.

use serde::Serialize;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, WebviewWindow};

use crate::data::DataState;

/// Para que lado a ilha cresce dentro da janela transparente
#[derive(Serialize, Clone, PartialEq)]
pub struct Anchor {
    pub x: &'static str, // "left" | "center" | "right"
    pub y: &'static str, // "top" | "bottom"
}

static MOVE_GEN: AtomicU64 = AtomicU64::new(0);

struct Bounds {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

fn monitor_bounds(win: &WebviewWindow, cx: i32, cy: i32) -> Option<Bounds> {
    let monitors = win.available_monitors().ok().unwrap_or_default();
    let hit = monitors.iter().find(|m| {
        let (p, s) = (m.position(), m.size());
        cx >= p.x && cx < p.x + s.width as i32 && cy >= p.y && cy < p.y + s.height as i32
    });
    let m = match hit {
        Some(m) => m.clone(),
        None => win.primary_monitor().ok().flatten()?,
    };
    Some(Bounds { x: m.position().x, y: m.position().y, w: m.size().width as i32, h: m.size().height as i32 })
}

fn window_rect(win: &WebviewWindow) -> Option<(i32, i32, i32, i32)> {
    let p = win.outer_position().ok()?;
    let s = win.outer_size().ok()?;
    Some((p.x, p.y, s.width as i32, s.height as i32))
}

/// Posição padrão: centro do topo do monitor principal
pub fn place_default(win: &WebviewWindow) {
    let monitor = win.primary_monitor().ok().flatten().or_else(|| win.current_monitor().ok().flatten());
    if let (Some(m), Some((_, _, w, _))) = (monitor, window_rect(win)) {
        let x = m.position().x + (m.size().width as i32 - w) / 2;
        let _ = win.set_position(PhysicalPosition::new(x, m.position().y));
    }
}

fn saved_position(app: &AppHandle) -> Option<(i32, i32)> {
    let state = app.state::<DataState>();
    let v = state.value.lock().unwrap();
    let p = &v["settings"]["position"];
    Some((p["x"].as_i64()? as i32, p["y"].as_i64()? as i32))
}

fn save_position(app: &AppHandle, pos: Option<(i32, i32)>) {
    let value = match pos {
        Some((x, y)) => json!({ "x": x, "y": y }),
        None => Value::Null,
    };
    crate::data::set_setting(app, "position", value);
}

/// Ao abrir o app: volta para onde estava (se ainda couber em algum monitor)
pub fn restore(app: &AppHandle, win: &WebviewWindow) {
    match (saved_position(app), window_rect(win)) {
        (Some((x, y)), Some((_, _, w, h))) if monitor_bounds(win, x + w / 2, y + h / 2).map_or(false, |b| {
            x + w / 2 >= b.x && x + w / 2 < b.x + b.w && y + h / 2 >= b.y && y + h / 2 < b.y + b.h
        }) => {
            let _ = win.set_position(PhysicalPosition::new(x, y));
        }
        _ => place_default(win),
    }
}

/// Calcula a âncora para a posição atual da janela
pub fn anchor(win: &WebviewWindow) -> Anchor {
    let Some((x, y, w, h)) = window_rect(win) else { return Anchor { x: "center", y: "top" } };
    let Some(b) = monitor_bounds(win, x + w / 2, y + h / 2) else { return Anchor { x: "center", y: "top" } };
    let rel_x = (x + w / 2 - b.x) as f64 / b.w as f64;
    let rel_y = (y + h / 2 - b.y) as f64 / b.h as f64;
    Anchor {
        x: if rel_x < 0.3 { "left" } else if rel_x > 0.7 { "right" } else { "center" },
        y: if rel_y > 0.5 { "bottom" } else { "top" },
    }
}

/// Depois que a janela para de se mexer: mantém dentro da tela, encaixa no
/// centro do topo se estiver perto, salva a posição e avisa a interface.
fn settle(app: &AppHandle, win: &WebviewWindow) {
    let Some((x0, y0, w, h)) = window_rect(win) else { return };
    let Some(b) = monitor_bounds(win, x0 + w / 2, y0 + h / 2) else { return };
    let scale = win.scale_factor().unwrap_or(1.0);
    let mut x = x0.clamp(b.x, (b.x + b.w - w).max(b.x));
    let mut y = y0.clamp(b.y, (b.y + b.h - h).max(b.y));

    // ímã: perto do centro do topo, encaixa na posição padrão
    let def_x = b.x + (b.w - w) / 2;
    let snap = (48.0 * scale) as i32;
    let is_default = (x - def_x).abs() < snap && y - b.y < snap;
    if is_default {
        x = def_x;
        y = b.y;
    }
    if (x, y) != (x0, y0) {
        let _ = win.set_position(PhysicalPosition::new(x, y));
    }
    save_position(app, if is_default { None } else { Some((x, y)) });
    let _ = app.emit("anchor", anchor(win));
}

/// Chamado a cada evento "Moved" da janela; só age quando o movimento termina.
pub fn on_moved(app: &AppHandle) {
    let gen = MOVE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(450));
        if MOVE_GEN.load(Ordering::SeqCst) != gen {
            return; // ainda se mexendo
        }
        if let Some(win) = app.get_webview_window("main") {
            settle(&app, &win);
        }
    });
}

/* ---------------- comandos ---------------- */

/// Começa a arrastar a janela (chamado ao segurar a barra do topo)
#[tauri::command]
pub fn start_drag(window: WebviewWindow) {
    let _ = window.start_dragging();
}

/// Volta para o centro do topo do monitor principal
#[tauri::command]
pub fn reset_position(app: AppHandle) {
    reset(&app);
}

pub fn reset(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        save_position(app, None);
        place_default(&win);
        let _ = app.emit("anchor", Anchor { x: "center", y: "top" });
    }
}
