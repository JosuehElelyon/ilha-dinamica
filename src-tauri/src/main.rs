// Ilha Dinâmica — Dynamic Island para Windows
// Copyright (c) 2026 Josueh — Licença MIT
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod audio;
mod calendar;
mod clean;
mod clipboard;
mod data;
mod games;
mod gitlab;
mod hover;
mod logger;
mod media;
mod network;
mod position;
mod system;
mod tray;
mod update;
mod util;
mod vpn;

use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn main() {
    logger::install_panic_hook();
    logger::log(format!("iniciando v{}", env!("CARGO_PKG_VERSION")));
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
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

            // o atalho pode estar ocupado por outro programa (ex.: a versão antiga em Electron):
            // nesse caso a ilha abre normalmente, só sem o atalho
            if let Err(e) = app.global_shortcut().on_shortcut("ctrl+alt+space", |app, _s, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = app.emit("toggle", ());
                }
            }) {
                logger::log(format!("atalho Ctrl+Alt+Espaço indisponível: {e}"));
            }
            if let Err(e) = app.global_shortcut().on_shortcut("ctrl+alt+k", |app, _s, event| {
                if event.state == ShortcutState::Pressed {
                    let _ = app.emit("palette", ());
                }
            }) {
                logger::log(format!("atalho Ctrl+Alt+K indisponível: {e}"));
            }
            if let Err(e) = tray::create(&handle) {
                logger::log(format!("ícone da bandeja: {e}"));
            }
            hover::start(handle.clone());
            clipboard::start(handle.clone());
            media::start(handle.clone());
            audio::start(handle.clone());
            system::start(handle.clone());
            vpn::start(handle.clone());

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
            actions::open_in_antigravity,
            actions::open_terminal,
            actions::git_status,
            actions::pick_path,
            actions::notify,
            actions::relaunch_admin,
            actions::quit_app,
            calendar::fetch_ics,
            update::check_update,
            vpn::list_adapters,
            vpn::vpn_connect,
            vpn::vpn_disconnect,
            vpn::probe_host,
            vpn::vpn_set_password,
            vpn::vpn_has_passwords,
            gitlab::gitlab_set_token,
            gitlab::gitlab_has_tokens,
            gitlab::gitlab_mrs,
            games::games_list,
            games::steam_info,
            games::steam_set_key,
            games::steam_friends,
            network::wifi_status,
            network::wifi_scan,
            network::wifi_connect,
            network::wifi_disconnect,
            network::bt_state,
            network::bt_toggle_radio,
            update::install_update,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            logger::log(format!("erro ao iniciar: {e}"));
            panic!("erro ao iniciar a Ilha Dinâmica: {e}");
        });

}
