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
mod position;
mod system;
mod tray;
mod util;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{Builder as ShortcutBuilder, ShortcutState};

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
            position::restore(&handle, &win);
            let _ = win.set_ignore_cursor_events(true);
            let _ = win.show();

            tray::create(&handle)?;
            hover::start(handle.clone());
            clipboard::start(handle.clone());
            media::start(handle.clone());
            audio::start(handle.clone());
            system::start(handle.clone());

            // ao arrastar: mantém dentro da tela, salva a posição e ajusta a âncora;
            // se a resolução / escala mudar, volta para onde estava
            let (h2, w2) = (handle.clone(), win.clone());
            win.on_window_event(move |e| match e {
                tauri::WindowEvent::Moved(_) => position::on_moved(&h2),
                tauri::WindowEvent::ScaleFactorChanged { .. } => position::restore(&h2, &w2),
                _ => {}
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            hover::set_hit_rect,
            hover::set_interactive,
            position::start_drag,
            position::reset_position,
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
