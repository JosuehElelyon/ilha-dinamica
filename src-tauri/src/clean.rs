//! Limpeza: temporários, cache de navegadores e apps, miniaturas, relatórios de erro,
//! Temp do Windows, cache do Windows Update, Lixeira, memória e DNS.
//! Só apaga o CONTEÚDO de pastas de cache conhecidas; arquivos em uso são pulados.

use serde::Serialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use sysinfo::{Disks, System};

struct Cat {
    key: &'static str,
    name: &'static str,
    icon: &'static str,
    admin: bool,
}

const CATS: &[Cat] = &[
    Cat { key: "temp", name: "Arquivos temporários", icon: "🗂️", admin: false },
    Cat { key: "browsers", name: "Cache dos navegadores", icon: "🌐", admin: false },
    Cat { key: "apps", name: "Cache de apps (Discord, VS Code)", icon: "💬", admin: false },
    Cat { key: "thumbs", name: "Miniaturas e ícones", icon: "🖼️", admin: false },
    Cat { key: "crash", name: "Relatórios de erro", icon: "💥", admin: false },
    Cat { key: "wintemp", name: "Temp do Windows", icon: "🪟", admin: true },
    Cat { key: "update", name: "Cache do Windows Update", icon: "🔄", admin: true },
    Cat { key: "recycle", name: "Lixeira", icon: "🗑️", admin: false },
];

fn env(k: &str, default: &str) -> PathBuf {
    std::env::var_os(k).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(default))
}

fn chromium(base: PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&base) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            let is_profile = n == "Default" || n == "Guest Profile" || n.starts_with("Profile ");
            if is_profile && e.path().is_dir() {
                for c in ["Cache", "Code Cache", "GPUCache"] {
                    out.push(e.path().join(c));
                }
                out.push(e.path().join("Service Worker").join("CacheStorage"));
            }
        }
    }
    out.push(base.join("ShaderCache"));
    out.push(base.join("GrShaderCache"));
    out
}

fn dirs(key: &str) -> Vec<PathBuf> {
    let local = env("LOCALAPPDATA", "");
    let roaming = env("APPDATA", "");
    let windir = env("SystemRoot", "C:\\Windows");
    let progdata = env("ProgramData", "C:\\ProgramData");
    match key {
        "temp" => vec![std::env::temp_dir()],
        "browsers" => {
            let mut v = chromium(local.join("Google").join("Chrome").join("User Data"));
            v.extend(chromium(local.join("Microsoft").join("Edge").join("User Data")));
            v.extend(chromium(local.join("BraveSoftware").join("Brave-Browser").join("User Data")));
            for o in ["Opera Stable", "Opera GX Stable"] {
                v.push(roaming.join("Opera Software").join(o).join("Cache"));
                v.push(local.join("Opera Software").join(o).join("Cache"));
            }
            if let Ok(rd) = std::fs::read_dir(local.join("Mozilla").join("Firefox").join("Profiles")) {
                v.extend(rd.flatten().map(|e| e.path().join("cache2")));
            }
            v
        }
        "apps" => {
            let mut v: Vec<PathBuf> = ["Cache", "Code Cache", "GPUCache"].iter().map(|c| roaming.join("discord").join(c)).collect();
            v.extend(["Cache", "CachedData", "GPUCache"].iter().map(|c| roaming.join("Code").join(c)));
            v
        }
        "thumbs" => vec![local.join("Microsoft").join("Windows").join("Explorer")],
        "crash" => vec![
            local.join("CrashDumps"),
            local.join("Microsoft").join("Windows").join("WER"),
            progdata.join("Microsoft").join("Windows").join("WER"),
        ],
        "wintemp" => vec![windir.join("Temp")],
        "update" => vec![windir.join("SoftwareDistribution").join("Download")],
        _ => vec![],
    }
}

/// Itens de primeiro nível que podem ser apagados (nunca a pasta raiz em si)
fn entries(key: &str) -> (Vec<PathBuf>, bool) {
    let mut out = Vec::new();
    let mut denied = false;
    for d in dirs(key) {
        match std::fs::read_dir(&d) {
            Ok(rd) => {
                for e in rd.flatten() {
                    let n = e.file_name().to_string_lossy().to_lowercase();
                    if key == "temp" && n.starts_with("ilha-dinamica") {
                        continue;
                    }
                    if key == "thumbs" && !((n.starts_with("thumbcache_") || n.starts_with("iconcache_")) && n.ends_with(".db")) {
                        continue;
                    }
                    out.push(e.path());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => denied = true,
            Err(_) => {}
        }
    }
    (out, denied)
}

fn size_of(p: &Path) -> u64 {
    let Ok(md) = std::fs::symlink_metadata(p) else { return 0 };
    if md.file_type().is_symlink() {
        return 0;
    }
    if !md.is_dir() {
        return md.len();
    }
    std::fs::read_dir(p)
        .map(|rd| rd.flatten().map(|e| size_of(&e.path())).sum())
        .unwrap_or(0)
}

/* ---------------- Lixeira ---------------- */

#[cfg(windows)]
fn recycle_info() -> (u64, u64) {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{SHQueryRecycleBinW, SHQUERYRBINFO};
    let mut info = SHQUERYRBINFO { cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32, ..Default::default() };
    unsafe {
        if SHQueryRecycleBinW(PCWSTR::null(), &mut info).is_ok() {
            return (info.i64Size.max(0) as u64, info.i64NumItems.max(0) as u64);
        }
    }
    (0, 0)
}
#[cfg(not(windows))]
fn recycle_info() -> (u64, u64) {
    (0, 0)
}

#[cfg(windows)]
fn empty_recycle() {
    use windows::core::PCWSTR;
    use windows::Win32::UI::Shell::{SHEmptyRecycleBinW, SHERB_NOCONFIRMATION, SHERB_NOPROGRESSUI, SHERB_NOSOUND};
    unsafe {
        let _ = SHEmptyRecycleBinW(None, PCWSTR::null(), SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND);
    }
}
#[cfg(not(windows))]
fn empty_recycle() {}

/* ---------------- Disco ---------------- */

#[derive(Serialize, Clone)]
pub struct Disk {
    free: u64,
    total: u64,
}

fn disk() -> Option<Disk> {
    let sysdrive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".into());
    let disks = Disks::new_with_refreshed_list();
    disks
        .list()
        .iter()
        .find(|d| {
            let mp = d.mount_point().to_string_lossy().to_uppercase();
            mp.starts_with(&sysdrive.to_uppercase()) || mp == "/"
        })
        .map(|d| Disk { free: d.available_space(), total: d.total_space() })
}

/* ---------------- Comandos ---------------- */

#[derive(Serialize)]
pub struct CatInfo {
    name: &'static str,
    icon: &'static str,
    admin: bool,
    size: u64,
    count: u64,
    denied: bool,
}

#[derive(Serialize)]
pub struct ScanResult {
    cats: BTreeMap<&'static str, CatInfo>,
    order: Vec<&'static str>,
    disk: Option<Disk>,
}

#[tauri::command]
pub async fn clean_scan() -> ScanResult {
    tauri::async_runtime::spawn_blocking(|| {
        let mut cats = BTreeMap::new();
        for c in CATS {
            let (size, count, denied) = if c.key == "recycle" {
                let (s, n) = recycle_info();
                (s, n, false)
            } else {
                let (list, denied) = entries(c.key);
                (list.iter().map(|p| size_of(p)).sum(), list.len() as u64, denied)
            };
            cats.insert(c.key, CatInfo { name: c.name, icon: c.icon, admin: c.admin, size, count, denied });
        }
        ScanResult { cats, order: CATS.iter().map(|c| c.key).collect(), disk: disk() }
    })
    .await
    .unwrap_or(ScanResult { cats: BTreeMap::new(), order: vec![], disk: None })
}

#[derive(Serialize)]
pub struct RunResult {
    freed: u64,
    disk: Option<Disk>,
}

#[tauri::command]
pub async fn clean_run(keys: Vec<String>) -> RunResult {
    tauri::async_runtime::spawn_blocking(move || {
        let before = disk().map(|d| d.free).unwrap_or(0);
        for k in &keys {
            if k == "recycle" {
                empty_recycle();
                continue;
            }
            if !CATS.iter().any(|c| c.key == k) {
                continue;
            }
            for p in entries(k).0 {
                // arquivos em uso ou sem permissão são simplesmente pulados
                let _ = if p.is_dir() { std::fs::remove_dir_all(&p) } else { std::fs::remove_file(&p) };
            }
        }
        let after = disk();
        let freed = after.as_ref().map(|d| d.free.saturating_sub(before)).unwrap_or(0);
        RunResult { freed, disk: after }
    })
    .await
    .unwrap_or(RunResult { freed: 0, disk: None })
}

#[derive(Serialize)]
pub struct RamResult {
    freed: u64,
    n: u32,
}

/// Pede ao Windows para devolver a memória ociosa de cada processo (EmptyWorkingSet)
#[tauri::command]
pub async fn trim_ram() -> RamResult {
    tauri::async_runtime::spawn_blocking(|| {
        let mut sys = System::new();
        sys.refresh_memory();
        let before = sys.available_memory();
        #[allow(unused_mut)]
        let mut n = 0u32;
        #[cfg(windows)]
        {
            use sysinfo::ProcessesToUpdate;
            use windows::Win32::Foundation::CloseHandle;
            use windows::Win32::System::ProcessStatus::K32EmptyWorkingSet;
            use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_QUOTA};
            sys.refresh_processes(ProcessesToUpdate::All, false);
            for pid in sys.processes().keys() {
                unsafe {
                    if let Ok(h) = OpenProcess(PROCESS_SET_QUOTA | PROCESS_QUERY_LIMITED_INFORMATION, false, pid.as_u32()) {
                        if K32EmptyWorkingSet(h).as_bool() {
                            n += 1;
                        }
                        let _ = CloseHandle(h);
                    }
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1200));
        sys.refresh_memory();
        RamResult { freed: sys.available_memory().saturating_sub(before), n }
    })
    .await
    .unwrap_or(RamResult { freed: 0, n: 0 })
}

#[tauri::command(async)]
pub fn flush_dns() -> bool {
    crate::util::hidden("ipconfig").arg("/flushdns").output().map(|o| o.status.success()).unwrap_or(false)
}
