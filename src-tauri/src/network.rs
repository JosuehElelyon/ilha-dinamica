//! Gerenciamento de redes Wi-Fi (`netsh wlan`) e dispositivos Bluetooth (PnP / WinRT) no Windows.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Clone, Default)]
pub struct WifiStatus {
    pub connected: bool,
    pub ssid: String,
    pub signal: u8,
}

#[derive(Serialize, Clone)]
pub struct WifiNet {
    pub ssid: String,
    pub signal: u8,
    pub auth: String,
    pub connected: bool,
}

#[derive(Serialize, Clone)]
pub struct BtDevice {
    pub id: String,
    pub name: String,
    pub connected: bool,
    pub kind: String,
}

#[derive(Serialize, Clone)]
pub struct BtState {
    pub available: bool,
    pub enabled: bool,
    pub devices: Vec<BtDevice>,
}

fn parse_pct(s: &str) -> u8 {
    s.trim()
        .trim_end_matches('%')
        .trim()
        .parse::<u8>()
        .unwrap_or(0)
        .min(100)
}

#[tauri::command(async)]
pub fn wifi_status() -> WifiStatus {
    #[cfg(windows)]
    {
        let Ok(out) = crate::util::hidden("netsh")
            .args(["wlan", "show", "interfaces"])
            .output()
        else {
            return WifiStatus::default();
        };
        let text = String::from_utf8_lossy(&out.stdout);
        let mut connected = false;
        let mut ssid = String::new();
        let mut signal = 0u8;

        for line in text.lines() {
            let Some((k, v)) = line.split_once(':') else { continue };
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim();
            if key == "state" || key == "estado" {
                let vl = val.to_ascii_lowercase();
                connected = vl == "connected" || vl == "conectado";
            } else if key == "ssid" && !key.contains("bssid") {
                ssid = val.to_string();
            } else if key == "signal" || key == "sinal" {
                signal = parse_pct(val);
            }
        }
        if !connected {
            ssid.clear();
            signal = 0;
        }
        WifiStatus { connected, ssid, signal }
    }
    #[cfg(not(windows))]
    {
        WifiStatus::default()
    }
}

#[tauri::command(async)]
pub fn wifi_scan() -> Vec<WifiNet> {
    #[cfg(windows)]
    {
        let cur = wifi_status();
        let Ok(out) = crate::util::hidden("netsh")
            .args(["wlan", "show", "networks", "mode=bssid"])
            .output()
        else {
            return vec![];
        };
        let text = String::from_utf8_lossy(&out.stdout);
        let mut map: HashMap<String, WifiNet> = HashMap::new();
        let mut cur_ssid = String::new();
        let mut cur_auth = String::new();
        let mut cur_sig = 0u8;

        let flush = |ssid: &str, auth: &str, sig: u8, map: &mut HashMap<String, WifiNet>| {
            if ssid.is_empty() {
                return;
            }
            let is_conn = cur.connected && cur.ssid == ssid;
            let entry = map.entry(ssid.to_string()).or_insert(WifiNet {
                ssid: ssid.to_string(),
                signal: sig,
                auth: auth.to_string(),
                connected: is_conn,
            });
            if sig > entry.signal {
                entry.signal = sig;
            }
            if entry.auth.is_empty() && !auth.is_empty() {
                entry.auth = auth.to_string();
            }
        };

        for line in text.lines() {
            let Some((k, v)) = line.split_once(':') else { continue };
            let key = k.trim().to_ascii_lowercase();
            let val = v.trim();
            if key.starts_with("ssid ") && !key.contains("bssid") {
                flush(&cur_ssid, &cur_auth, cur_sig, &mut map);
                cur_ssid = val.to_string();
                cur_auth.clear();
                cur_sig = 0;
            } else if key.starts_with("authentication") || key.starts_with("autentica") {
                cur_auth = val.to_string();
            } else if key == "signal" || key == "sinal" {
                let s = parse_pct(val);
                if s > cur_sig {
                    cur_sig = s;
                }
            }
        }
        flush(&cur_ssid, &cur_auth, cur_sig, &mut map);

        let mut list: Vec<WifiNet> = map.into_values().collect();
        list.sort_by(|a, b| b.connected.cmp(&a.connected).then(b.signal.cmp(&a.signal)));
        list
    }
    #[cfg(not(windows))]
    {
        vec![]
    }
}

#[tauri::command(async)]
pub fn wifi_connect(ssid: String) -> Result<String, String> {
    #[cfg(windows)]
    {
        let s = ssid.trim();
        if s.is_empty() {
            return Err("Nome da rede inválido".into());
        }
        let out = crate::util::hidden("netsh")
            .args(["wlan", "connect", &format!("name={s}")])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(format!("Conectando a {s}…"))
        } else {
            let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Err(if msg.is_empty() {
                "Perfil não salvo — abra as Configurações do Windows para informar a senha.".into()
            } else {
                msg
            })
        }
    }
    #[cfg(not(windows))]
    {
        let _ = ssid;
        Err("Disponível apenas no Windows".into())
    }
}

#[tauri::command(async)]
pub fn wifi_disconnect() -> bool {
    #[cfg(windows)]
    {
        crate::util::hidden("netsh")
            .args(["wlan", "disconnect"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[tauri::command(async)]
pub fn bt_state() -> BtState {
    #[cfg(windows)]
    {
        let ps = r#"
$ErrorActionPreference = 'SilentlyContinue'
$enabled = $false
$available = $false
try {
  Add-Type -AssemblyName System.Runtime.WindowsRuntime | Out-Null
  $asTask = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object { $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' })[0]
  $op = [Windows.Devices.Radios.Radio, Windows.System.Devices, ContentType=WindowsRuntime]::GetRadiosAsync()
  $task = $asTask.MakeGenericMethod([System.Collections.Generic.IReadOnlyList[Windows.Devices.Radios.Radio]]).Invoke($null, @($op))
  $radios = $task.Result
  $bt = $radios | Where-Object { $_.Kind -eq 'Bluetooth' } | Select-Object -First 1
  if ($bt) { $available = $true; $enabled = ($bt.State -eq 'On') }
} catch {}
$devs = @()
if ($available -or (Get-PnpDevice -Class Bluetooth | Select-Object -First 1)) {
  $available = $true
  $pnp = Get-PnpDevice -Class Bluetooth | Where-Object { $_.InstanceId -like 'BTHENUM\DEV_*' -or $_.InstanceId -like 'BTHLE\DEV_*' }
  foreach ($d in $pnp) {
    $conn = $false
    try {
      $prop = Get-PnpDeviceProperty -InstanceId $d.InstanceId -KeyName '{83DA6326-97A6-4088-9453-A1923F573B29} 15'
      if ($prop -and $prop.Data -eq $true) { $conn = $true }
    } catch {}
    $name = if ($d.FriendlyName) { $d.FriendlyName } else { 'Dispositivo Bluetooth' }
    $kind = if ($d.InstanceId -like 'BTHLE*') { 'LE' } else { 'BT' }
    $devs += [PSCustomObject]@{ id = $d.InstanceId; name = $name; connected = $conn; kind = $kind }
  }
}
[PSCustomObject]@{ available = $available; enabled = $enabled; devices = @($devs | Sort-Object -Property @{Expression='connected';Descending=$true}, 'name') } | ConvertTo-Json -Depth 3 -Compress
"#;
        let Ok(out) = crate::util::hidden("powershell.exe")
            .args(["-NoProfile", "-Command", ps])
            .output()
        else {
            return BtState { available: false, enabled: false, devices: vec![] };
        };
        let s = String::from_utf8_lossy(&out.stdout);
        let Ok(v) = serde_json::from_str::<serde_json::Value>(s.trim()) else {
            return BtState { available: false, enabled: false, devices: vec![] };
        };
        let available = v["available"].as_bool().unwrap_or(false);
        let enabled = v["enabled"].as_bool().unwrap_or(false);
        let devices = v["devices"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|d| BtDevice {
                        id: d["id"].as_str().unwrap_or("").to_string(),
                        name: d["name"].as_str().unwrap_or("Dispositivo").to_string(),
                        connected: d["connected"].as_bool().unwrap_or(false),
                        kind: d["kind"].as_str().unwrap_or("BT").to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        BtState { available, enabled, devices }
    }
    #[cfg(not(windows))]
    {
        BtState { available: false, enabled: false, devices: vec![] }
    }
}

#[tauri::command(async)]
pub fn bt_toggle_radio(enable: bool) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let target = if enable { "On" } else { "Off" };
        let ps = format!(
            r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime | Out-Null
$asTask = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {{ $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }})[0]
$op = [Windows.Devices.Radios.Radio, Windows.System.Devices, ContentType=WindowsRuntime]::GetRadiosAsync()
$task = $asTask.MakeGenericMethod([System.Collections.Generic.IReadOnlyList[Windows.Devices.Radios.Radio]]).Invoke($null, @($op))
$bt = $task.Result | Where-Object {{ $_.Kind -eq 'Bluetooth' }} | Select-Object -First 1
if (-not $bt) {{ throw 'Adaptador Bluetooth não encontrado' }}
$op2 = $bt.SetStateAsync([Windows.Devices.Radios.RadioState]::{target})
$t2 = $asTask.MakeGenericMethod([Windows.Devices.Radios.RadioAccessStatus]).Invoke($null, @($op2))
$res = $t2.Result
Write-Output $res
"#
        );
        let out = crate::util::hidden("powershell.exe")
            .args(["-NoProfile", "-Command", &ps])
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(enable)
        } else {
            Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
        }
    }
    #[cfg(not(windows))]
    {
        let _ = enable;
        Err("Disponível apenas no Windows".into())
    }
}
