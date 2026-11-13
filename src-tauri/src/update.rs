//! Atualização automática pelas Releases do GitHub (assinadas).

use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

#[derive(Serialize)]
pub struct UpdateInfo {
    available: bool,
    version: String,
    current: String,
    notes: String,
}

#[tauri::command]
pub async fn check_update(app: AppHandle) -> Result<UpdateInfo, String> {
    let current = app.package_info().version.to_string();
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| {
        crate::logger::log(format!("verificar atualização: {e}"));
        e.to_string()
    })?;
    Ok(match update {
        Some(u) => UpdateInfo { available: true, version: u.version.clone(), current, notes: u.body.clone().unwrap_or_default() },
        None => UpdateInfo { available: false, version: String::new(), current, notes: String::new() },
    })
}

/// Baixa, instala e reinicia o app
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let update = app.updater().map_err(|e| e.to_string())?.check().await.map_err(|e| e.to_string())?;
    if let Some(u) = update {
        crate::logger::log(format!("instalando atualização {}", u.version));
        u.download_and_install(|_, _| {}, || {}).await.map_err(|e| {
            crate::logger::log(format!("instalar atualização: {e}"));
            e.to_string()
        })?;
        app.restart();
    }
    Ok(())
}
