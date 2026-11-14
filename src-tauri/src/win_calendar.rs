//! Conector zero-config do Calendário nativo do Windows (WinRT AppointmentManager).
//! Lê eventos locais sincronizados no Windows (Outlook, conta Microsoft, feriados) sem login extra.

use assistant_core::domain::{CanonicalEvent, ProviderKind};
use assistant_core::normalizer::{event_fingerprint, short_id};

#[cfg(windows)]
pub fn fetch_windows_appointments(days_ahead: i64) -> Result<Vec<CanonicalEvent>, String> {
    use chrono::{DateTime, Local, TimeZone};
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct RawWinAppt {
        subject: String,
        start_ms: i64,
        duration_sec: i64,
        all_day: bool,
        location: String,
    }

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.ApplicationModel.Appointments.AppointmentManager, Windows.ApplicationModel.Appointments, ContentType=WindowsRuntime]
$null = [Windows.ApplicationModel.Appointments.FindAppointmentsOptions, Windows.ApplicationModel.Appointments, ContentType=WindowsRuntime]
$asTaskGeneric = ([System.WindowsRuntimeSystemExtensions].GetMethods() | Where-Object {{ $_.Name -eq 'AsTask' -and $_.GetParameters().Count -eq 1 -and $_.GetParameters()[0].ParameterType.Name -eq 'IAsyncOperation`1' }})[0]
function Await($op, $type) {{
  $m = $asTaskGeneric.MakeGenericMethod($type)
  $t = $m.Invoke($null, @($op))
  $t.Wait(-1) | Out-Null
  return $t.Result
}}
$store = Await ([Windows.ApplicationModel.Appointments.AppointmentManager]::RequestStoreAsync([Windows.ApplicationModel.Appointments.AppointmentStoreAccessType]::AllCalendarsReadOnly)) ([Windows.ApplicationModel.Appointments.AppointmentStore])
$start = [DateTimeOffset]::Now.Date
$span = [TimeSpan]::FromDays({days_ahead})
$opts = New-Object Windows.ApplicationModel.Appointments.FindAppointmentsOptions
$opts.IncludeHidden = $false
$opts.MaxCount = 80
$list = Await ($store.FindAppointmentsAsync($start, $span, $opts)) ([System.Collections.Generic.IReadOnlyList[Windows.ApplicationModel.Appointments.Appointment]])
$out = @()
foreach ($a in $list) {{
  $out += [PSCustomObject]@{{
    subject = [string]$a.Subject
    start_ms = [long]$a.StartTime.ToUnixTimeMilliseconds()
    duration_sec = [long]$a.Duration.TotalSeconds
    all_day = [bool]$a.AllDay
    location = [string]$a.Location
  }}
}}
$out | ConvertTo-Json -Compress
"#
    );

    let out = crate::util::powershell(&script).map_err(|e| e.to_string())?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(if err.is_empty() {
            "Calendário do Windows indisponível".into()
        } else {
            err
        });
    }
    let raw = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if raw.is_empty() || raw == "null" {
        return Ok(vec![]);
    }

    let items: Vec<RawWinAppt> = if raw.starts_with('[') {
        serde_json::from_str(&raw).map_err(|e| e.to_string())?
    } else {
        vec![serde_json::from_str::<RawWinAppt>(&raw).map_err(|e| e.to_string())?]
    };

    let mut events = Vec::new();
    for item in items {
        let title = if item.subject.trim().is_empty() {
            "(Sem título)".to_string()
        } else {
            item.subject.trim().to_string()
        };
        let start_ts = item.start_ms / 1000;
        let end_ts = start_ts + item.duration_sec.max(1800);
        let st = DateTime::from_timestamp(start_ts, 0)
            .map(|d| d.with_timezone(&Local))
            .unwrap_or_else(|| Local.timestamp_opt(start_ts, 0).unwrap());
        let et = DateTime::from_timestamp(end_ts, 0)
            .map(|d| d.with_timezone(&Local))
            .unwrap_or(st);
        let date = st.format("%Y-%m-%d").to_string();
        let start_time = if item.all_day { "Dia".into() } else { st.format("%H:%M").to_string() };
        let end_time = if item.all_day { "".into() } else { et.format("%H:%M").to_string() };
        let join_url = extract_meeting_url(&item.location);
        let fp = event_fingerprint(&date, &start_time, &title);
        events.push(CanonicalEvent {
            id: short_id("win", &fp),
            fingerprint: fp,
            title,
            category: if join_url.is_empty() { "personal".into() } else { "meeting".into() },
            date,
            start_time,
            end_time,
            start_ts,
            end_ts,
            all_day: item.all_day,
            join_url,
            source_provider: ProviderKind::Windows,
            account_id: "windows:local".into(),
        });
    }
    Ok(events)
}

#[cfg(not(windows))]
pub fn fetch_windows_appointments(_days_ahead: i64) -> Result<Vec<CanonicalEvent>, String> {
    Ok(vec![])
}

#[cfg(windows)]
fn extract_meeting_url(text: &str) -> String {
    for tok in text.split_whitespace() {
        let clean = tok.trim_matches(|c: char| !c.is_alphanumeric() && !":/?&=._-%#".contains(c));
        if clean.starts_with("https://")
            && (clean.contains("teams.microsoft.com")
                || clean.contains("meet.google.com")
                || clean.contains("zoom.us"))
        {
            return clean.to_string();
        }
    }
    String::new()
}
