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
        if t.contains("://") || t.starts_with("ms-") || t.starts_with("mailto:") {
            use windows::core::{w, HSTRING, PCWSTR};
            use windows::Win32::UI::Shell::ShellExecuteW;
            use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
            let url = HSTRING::from(&t);
            let r = unsafe { ShellExecuteW(None, w!("open"), &url, PCWSTR::null(), PCWSTR::null(), SW_SHOWNORMAL) };
            if r.0 as isize > 32 {
                return true;
            }
        }
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

#[derive(serde::Serialize)]
pub struct RepoGitStatus {
    pub path: String,
    pub is_git: bool,
    pub branch: String,
    pub dirty: usize,
}

/// Abre o Terminal (Windows Terminal ou PowerShell) diretamente na pasta do repositório
#[tauri::command]
pub fn open_terminal(path: String) -> bool {
    let p = path.trim().replace('"', "");
    if p.is_empty() {
        return false;
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        if std::process::Command::new("wt.exe")
            .args(["-d", &p])
            .spawn()
            .is_ok()
        {
            return true;
        }
        std::process::Command::new("cmd.exe")
            .raw_arg(format!("/d /s /c start \"\" powershell.exe -NoExit -Command \"Set-Location -LiteralPath '{p}'\""))
            .spawn()
            .is_ok()
    }
    #[cfg(not(windows))]
    {
        std::process::Command::new("x-terminal-emulator")
            .current_dir(&p)
            .spawn()
            .is_ok()
    }
}

/// Consulta branch atual e quantidade de alterações não commitadas para uma lista de pastas
#[tauri::command(async)]
pub fn git_status(paths: Vec<String>) -> Vec<RepoGitStatus> {
    paths
        .into_iter()
        .map(|raw| {
            let p = raw.trim().to_string();
            let br = crate::util::hidden("git")
                .args(["-C", &p, "rev-parse", "--abbrev-ref", "HEAD"])
                .output();
            let Ok(br_out) = br else {
                return RepoGitStatus { path: p, is_git: false, branch: String::new(), dirty: 0 };
            };
            if !br_out.status.success() {
                return RepoGitStatus { path: p, is_git: false, branch: String::new(), dirty: 0 };
            }
            let branch = String::from_utf8_lossy(&br_out.stdout).trim().to_string();
            let dirty = crate::util::hidden("git")
                .args(["-C", &p, "status", "--porcelain"])
                .output()
                .map(|o| {
                    if !o.status.success() {
                        0
                    } else {
                        String::from_utf8_lossy(&o.stdout)
                            .lines()
                            .filter(|l| !l.trim().is_empty())
                            .count()
                    }
                })
                .unwrap_or(0);
            RepoGitStatus { path: p, is_git: true, branch, dirty }
        })
        .collect()
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
