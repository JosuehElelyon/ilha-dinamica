//! CPU, RAM, bateria, brilho e processos.

use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager, State};

/// Processos que não podem ser finalizados (travaria ou reiniciaria o Windows)
pub const PROTECTED: &[&str] = &[
    "system", "registry", "smss", "csrss", "wininit", "winlogon", "services", "lsass", "lsaiso",
    "svchost", "dwm", "fontdrvhost", "memory compression", "secure system", "sihost", "ctfmon",
    "msmpeng", "securityhealthservice", "spoolsv", "audiodg", "conhost", "runtimebroker",
    "taskhostw", "idle", "startmenuexperiencehost", "shellexperiencehost", "searchhost",
    "textinputhost", "wudfhost", "nissrv", "msedgewebview2",
];

pub fn self_name() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .unwrap_or_default()
}

fn clean_name(n: &str) -> String {
    n.strip_suffix(".exe").or_else(|| n.strip_suffix(".EXE")).unwrap_or(n).to_string()
}

#[derive(Default)]
pub struct SysState {
    procs: Mutex<Option<System>>,
}

#[derive(Serialize, Clone)]
struct Battery {
    level: u8,
    charging: bool,
}

#[derive(Serialize, Clone)]
struct SysInfo {
    cpu: u32,
    #[serde(rename = "memUsed")]
    mem_used: u64,
    #[serde(rename = "memTotal")]
    mem_total: u64,
    uptime: u64,
    #[serde(rename = "cpuName")]
    cpu_name: String,
    battery: Option<Battery>,
}

#[cfg(windows)]
fn battery() -> Option<Battery> {
    use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    let mut s = SYSTEM_POWER_STATUS::default();
    unsafe { GetSystemPowerStatus(&mut s).ok()? };
    // 128 = sem bateria, 255 = desconhecido
    if s.BatteryFlag & 128 != 0 || s.BatteryLifePercent == 255 {
        return None;
    }
    Some(Battery { level: s.BatteryLifePercent, charging: s.ACLineStatus == 1 })
}
#[cfg(not(windows))]
fn battery() -> Option<Battery> {
    None
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            let info = SysInfo {
                cpu: sys.global_cpu_usage().round() as u32,
                mem_used: sys.used_memory(),
                mem_total: sys.total_memory(),
                uptime: System::uptime(),
                cpu_name: sys.cpus().first().map(|c| c.brand().trim().to_string()).unwrap_or_default(),
                battery: battery(),
            };
            let _ = app.emit("sys", info);
            std::thread::sleep(Duration::from_secs(2));
        }
    });
}

/* ---------------- brilho (WMI, só telas de notebook) ---------------- */

#[tauri::command(async)]
pub fn get_brightness() -> Option<u32> {
    let out = crate::util::powershell(
        "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness",
    )
    .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

#[tauri::command(async)]
pub fn set_brightness(value: u32) -> bool {
    let v = value.min(100);
    crate::util::powershell(&format!(
        "$m = Get-WmiObject -Namespace root/WMI -Class WmiMonitorBrightnessMethods -ErrorAction Stop | Select-Object -First 1; $null = $m.WmiSetBrightness(1, {v})"
    ))
    .map(|o| o.status.success())
    .unwrap_or(false)
}

/* ---------------- processos ---------------- */

#[derive(Serialize)]
pub struct Proc {
    n: String,
    p: u32,
    c: f32,
    m: u64,
}

#[tauri::command(async)]
pub fn get_procs(state: State<SysState>) -> Vec<Proc> {
    let mut guard = state.procs.lock().unwrap();
    let first = guard.is_none();
    let sys = guard.get_or_insert_with(System::new);
    sys.refresh_cpu_usage();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    if first {
        // o uso de CPU é calculado entre duas leituras
        std::thread::sleep(Duration::from_millis(300));
        sys.refresh_processes(ProcessesToUpdate::All, true);
    }
    let cores = sys.cpus().len().max(1) as f32;
    sys.processes()
        .iter()
        .filter(|(pid, _)| pid.as_u32() != 0)
        .map(|(pid, p)| Proc {
            n: clean_name(&p.name().to_string_lossy()),
            p: pid.as_u32(),
            c: ((p.cpu_usage() / cores) * 10.0).round() / 10.0,
            m: p.memory(),
        })
        .collect()
}

#[derive(Serialize)]
pub struct KillResult {
    ok: bool,
    protected: bool,
}

#[tauri::command(async)]
pub fn kill_process(app: AppHandle, pid: Option<u32>, name: Option<String>) -> KillResult {
    let nm = name.clone().unwrap_or_default().to_lowercase();
    if PROTECTED.contains(&nm.as_str()) || nm == self_name() || pid == Some(std::process::id()) {
        return KillResult { ok: false, protected: true };
    }
    let state = app.state::<SysState>();
    let mut guard = state.procs.lock().unwrap();
    let sys = guard.get_or_insert_with(System::new);
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut ok = false;
    if let Some(pid) = pid {
        if let Some(p) = sys.process(Pid::from_u32(pid)) {
            ok = p.kill();
        }
    } else if !nm.is_empty() {
        for p in sys.processes().values() {
            if clean_name(&p.name().to_string_lossy()).to_lowercase() == nm && p.kill() {
                ok = true;
            }
        }
    }
    KillResult { ok, protected: false }
}
