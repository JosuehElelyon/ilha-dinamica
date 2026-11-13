// Ilha Dinâmica — Dynamic Island para Windows
// Copyright (c) 2026 Josueh — Licença MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod audio;
mod clean;
mod clipboard;
mod data;
mod hover;
mod media;
mod system;
mod tray;
mod util;

use tauri::{Emitter, Manager, PhysicalPosition};
use tauri_plugin_global_shortcut::{Builder as ShortcutBuilder, ShortcutState};

/// Largura e altura (em pixels lógicos) da janela transparente onde a ilha é desenhada.
pub const WIN_W: f64 = 880.0;

fn place_window(win: &tauri::WebviewWindow) {
    let monitor = win.primary_monitor().ok().flatten().or_else(|| win.current_monitor().ok().flatten());
    if let Some(m) = monitor {
        let scale = m.scale_factor();
        let w = (WIN_W * scale) as i32;
        let x = m.position().x + (m.size().width as i32 - w) / 2;
        let _ = win.set_position(PhysicalPosition::new(x, m.position().y));
    }
}

fn main() {
    // Quando reiniciamos como administrador, damos tempo para a instância antiga fechar
    if std::env::args().any(|a| a == "--elevated-restart") {
        std::thread::sleep(std::time::Duration::from_millis(1500));
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.emit("toggle", ());
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            ShortcutBuilder::new()
                .with_shortcuts(["ctrl+alt+space"])
                .expect("atalho inválido")
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app.emit("toggle", ());
                    }
                })
                .build(),
        )
        .manage(data::DataState::default())
        .manage(hover::HoverState::default())
        .manage(clipboard::ClipState::default())
        .manage(system::SysState::default())
        .manage(audio::AudioState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            data::load(&handle);

            let win = app.get_webview_window("main").expect("janela principal");
            place_window(&win);
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.show();

            tray::create(&handle)?;
            hover::start(handle.clone());
            clipboard::start(handle.clone());
            media::start(handle.clone());
            audio::start(handle.clone());
            system::start(handle.clone());

            // reposiciona se a resolução / escala mudar
            let w2 = win.clone();
            win.on_window_event(move |e| {
                if let tauri::WindowEvent::ScaleFactorChanged { .. } = e {
                    place_window(&w2);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hover::set_hit_rect,
            hover::set_interactive,
            data::get_data,
            data::save_notes,
            data::save_reminders,
            data::save_setting,
            data::set_autostart,
            clipboard::copy_text,
            clipboard::clear_clips,
            media::media_cmd,
            audio::set_vol,
            audio::toggle_mute,
            audio::set_mic,
            audio::toggle_mic,
            audio::set_device,
            audio::refresh_devices,
            system::get_brightness,
            system::set_brightness,
            system::get_procs,
            system::kill_process,
            clean::clean_scan,
            clean::clean_run,
            clean::trim_ram,
            clean::flush_dns,
            actions::run_action,
            actions::pick_path,
            actions::notify,
            actions::relaunch_admin,
            actions::quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar a Ilha Dinâmica");

}
