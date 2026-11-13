//! Ações personalizadas, seletor de arquivos, notificações e reinício como administrador.

use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_notification::NotificationExt;

/// Abre site, programa, pasta, arquivo ou comando (calc, ms-settings:, spotify: ...)
#[tauri::command]
pub fn run_action(target: String) -> bool {
    let t = target.trim().replace('"', "");
    if t.is_empty() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // "start" resolve programas do PATH, pastas, URLs e protocolos
        crate::util::hidden("cmd.exe")
            .raw_arg(format!("/d /s /c start \"\" \"{t}\""))
            .spawn()
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("xdg-open").arg(t).spawn().is_ok()
    }
}

/// Abre uma pasta diretamente no Antigravity (procura no PATH e nos caminhos padrão de instalação)
#[tauri::command]
pub fn open_in_antigravity(path: String) -> bool {
    let p = path.trim().replace('"', "");
    if p.is_empty() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        use std::path::PathBuf;
        let candidates = [
            std::env::var("LOCALAPPDATA")
                .ok()
                .map(|d| PathBuf::from(d).join("Programs\\Antigravity\\Antigravity.exe")),
            std::env::var("LOCALAPPDATA")
                .ok()
                .map(|d| PathBuf::from(d).join("Programs\\Antigravity\\bin\\antigravity.cmd")),
            std::env::var("ProgramFiles")
                .ok()
                .map(|d| PathBuf::from(d).join("Antigravity\\Antigravity.exe")),
        ];
        for exe in candidates.into_iter().flatten() {
            if exe.exists() {
                if exe.extension().and_then(|s| s.to_str()) == Some("cmd") {
                    return crate::util::hidden("cmd.exe")
                        .raw_arg(format!("/d /s /c \"\"{}\" \"{}\"\"", exe.display(), p))
                        .spawn()
                        .is_ok();
                }
                return crate::util::hidden(&exe.to_string_lossy()).arg(&p).spawn().is_ok();
            }
        }
        crate::util::hidden("cmd.exe")
            .raw_arg(format!("/d /s /c antigravity \"{p}\""))
            .spawn()
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("antigravity").arg(&p).spawn().is_ok()
    }
}

#[tauri::command]
pub async fn pick_path(app: AppHandle, kind: String) -> Option<String> {
    let win = app.get_webview_window("main");
    if let Some(w) = &win {
        let _ = w.set_always_on_top(false);
    }
    let dialog = app.dialog().file().set_title(if kind == "folder" { "Escolha uma pasta" } else { "Escolha um programa ou arquivo" });
    let picked = if kind == "folder" { dialog.blocking_pick_folder() } else { dialog.blocking_pick_file() };
    if let Some(w) = &win {
        let _ = w.set_always_on_top(true);
    }
    picked.and_then(|p| p.into_path().ok()).map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn notify(app: AppHandle, title: String, body: String) {
    let _ = app.notification().builder().title(title).body(body).show();
}

#[tauri::command]
pub fn relaunch_admin(app: AppHandle) -> bool {
    #[cfg(windows)]
    {
        use windows::core::{w, HSTRING, PCWSTR};
        use windows::Win32::UI::Shell::ShellExecuteW;
        use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
        let Ok(exe) = std::env::current_exe() else { return false };
        let exe = HSTRING::from(exe.as_os_str());
        let args = HSTRING::from("--elevated-restart");
        let r = unsafe { ShellExecuteW(None, w!("runas"), &exe, &args, PCWSTR::null(), SW_SHOWNORMAL) };
        // valores > 32 significam sucesso; se o usuário recusar o UAC, seguimos rodando
        if r.0 as isize > 32 {
            app.exit(0);
            return true;
        }
        false
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        false
    }
}

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
