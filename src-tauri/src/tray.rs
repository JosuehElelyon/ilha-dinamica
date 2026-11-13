//! Ícone na bandeja (perto do relógio).

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt;

pub struct TrayItems {
    autostart: CheckMenuItem<Wry>,
}

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let toggle = MenuItem::with_id(app, "toggle", "Abrir / fechar ilha  (Ctrl+Alt+Espaço)", true, None::<&str>)?;
    let autostart = CheckMenuItem::with_id(
        app,
        "autostart",
        "Iniciar com o Windows",
        true,
        app.autolaunch().is_enabled().unwrap_or(false),
        None::<&str>,
    )?;
    let center = MenuItem::with_id(app, "center", "Voltar ao centro do topo", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle, &center, &autostart, &sep, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Ilha Dinâmica")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "toggle" => {
                let _ = app.emit("toggle", ());
            }
            "center" => crate::position::reset(app),
            "autostart" => {
                let al = app.autolaunch();
                let on = al.is_enabled().unwrap_or(false);
                let _ = if on { al.disable() } else { al.enable() };
                refresh(app);
                let _ = app.emit("autostart", !on);
            }
            "quit" => app.exit(0),
            _ => {}
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    app.manage(TrayItems { autostart });
    Ok(())
}

pub fn refresh(app: &AppHandle) {
    if let Some(items) = app.try_state::<TrayItems>() {
        let _ = items.autostart.set_checked(app.autolaunch().is_enabled().unwrap_or(false));
    }
}
