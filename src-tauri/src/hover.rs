//! Faz a área transparente da janela deixar o clique passar para o que está atrás,
//! e avisa a interface quando o mouse entra/sai da ilha.
//!
//! O Tauri não tem o "forward" de eventos de mouse do Electron, então acompanhamos
//! a posição do cursor e ligamos/desligamos o click-through conforme ele está
//! dentro do retângulo da ilha (informado pela interface).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Default)]
pub struct HoverState {
    /// Retângulo da ilha em pixels CSS, relativo à janela: x, y, largura, altura
    rect: Mutex<(f64, f64, f64, f64)>,
    /// Interface pediu para a janela inteira aceitar cliques (ilha aberta / fixada)
    force: AtomicBool,
}

#[tauri::command]
pub fn set_hit_rect(state: State<HoverState>, x: f64, y: f64, w: f64, h: f64) {
    *state.rect.lock().unwrap() = (x, y, w, h);
}

#[tauri::command]
pub fn set_interactive(state: State<HoverState>, value: bool) {
    state.force.store(value, Ordering::Relaxed);
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut was_inside = false;
        let mut last_ignore: Option<bool> = None;
        loop {
            std::thread::sleep(Duration::from_millis(40));
            let Some(win) = app.get_webview_window("main") else { continue };
            let state = app.state::<HoverState>();
            let (x, y, w, h) = *state.rect.lock().unwrap();

            let inside = match (app.cursor_position(), win.inner_position(), win.scale_factor()) {
                (Ok(c), Ok(p), Ok(s)) => {
                    let lx = (c.x - p.x as f64) / s;
                    let ly = (c.y - p.y as f64) / s;
                    w > 0.0 && lx >= x && lx <= x + w && ly >= y && ly <= y + h
                }
                _ => false,
            };
            if inside != was_inside {
                was_inside = inside;
                let _ = win.emit("hover", inside);
            }
            let ignore = !(inside || state.force.load(Ordering::Relaxed));
            if last_ignore != Some(ignore) {
                let _ = win.set_ignore_cursor_events(ignore);
                last_ignore = Some(ignore);
            }
        }
    });
}
