use crate::domain::{CanonicalEvent, ImportantDate, PersonRecord, ProviderKind};
use chrono::{DateTime, Local, NaiveDate, TimeZone};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub fn event_fingerprint(date: &str, start_time: &str, title: &str) -> String {
    let mut h = Sha256::new();
    h.update(date.trim().as_bytes());
    h.update(b"|");
    h.update(start_time.trim().as_bytes());
    h.update(b"|");
    h.update(title.trim().to_ascii_lowercase().as_bytes());
    format!("{:x}", h.finalize())
}

pub fn short_id(prefix: &str, seed: &str) -> String {
    let mut h = Sha256::new();
    h.update(seed.as_bytes());
    let hex = format!("{:x}", h.finalize());
    format!("{prefix}-{}", &hex[..16])
}

/// Converte resposta do Google Calendar API v3 (`events.list`) em eventos canônicos.
pub fn parse_google_calendar_events(json: &Value, account_id: &str) -> Vec<CanonicalEvent> {
    let mut out = Vec::new();
    let Some(items) = json["items"].as_array() else {
        return out;
    };

    for item in items {
        if item["status"].as_str() == Some("cancelled") {
            continue;
        }
        let title = item["summary"].as_str().unwrap_or("(Sem título)").trim().to_string();
        let start_dt = item["start"]["dateTime"].as_str();
        let start_d = item["start"]["date"].as_str();
        let end_dt = item["end"]["dateTime"].as_str();
        let end_d = item["end"]["date"].as_str();

        let (date, start_time, start_ts, end_ts, end_time, all_day) = if let Some(sdt) = start_dt {
            let Ok(st) = DateTime::parse_from_rfc3339(sdt) else { continue };
            let et = end_dt
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .unwrap_or(st + chrono::Duration::minutes(30));
            let local_st = st.with_timezone(&Local);
            let local_et = et.with_timezone(&Local);
            (
                local_st.format("%Y-%m-%d").to_string(),
                local_st.format("%H:%M").to_string(),
                st.timestamp(),
                et.timestamp(),
                local_et.format("%H:%M").to_string(),
                false,
            )
        } else if let Some(sd) = start_d {
            let Ok(nd) = NaiveDate::parse_from_str(sd, "%Y-%m-%d") else { continue };
            let end_nd = end_d
                .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok())
                .unwrap_or(nd);
            let st_ts = Local
                .from_local_datetime(&nd.and_hms_opt(0, 0, 0).unwrap())
                .single()
                .map(|d| d.timestamp())
                .unwrap_or(0);
            let et_ts = Local
                .from_local_datetime(&end_nd.and_hms_opt(23, 59, 59).unwrap())
                .single()
                .map(|d| d.timestamp())
                .unwrap_or(st_ts);
            (sd.to_string(), "Dia".into(), st_ts, et_ts, "".into(), true)
        } else {
            continue;
        };

        let join_url = item["hangoutLink"]
            .as_str()
            .or_else(|| {
                item["conferenceData"]["entryPoints"]
                    .as_array()
                    .and_then(|arr| arr.iter().find_map(|ep| ep["uri"].as_str()))
            })
            .unwrap_or("")
            .to_string();

        let fp = event_fingerprint(&date, &start_time, &title);
        let id = item["id"]
            .as_str()
            .map(|s| format!("gcal-{s}"))
            .unwrap_or_else(|| short_id("gcal", &fp));

        out.push(CanonicalEvent {
            id,
            fingerprint: fp,
            title,
            category: if join_url.is_empty() { "personal".into() } else { "meeting".into() },
            date,
            start_time,
            end_time,
            start_ts,
            end_ts,
            all_day,
            join_url,
            source_provider: ProviderKind::Google,
            account_id: account_id.to_string(),
        });
    }
    out
}

/// Extrai contatos e datas comemorativas/aniversários da Google People API (`connections`).
pub fn parse_google_contacts(json: &Value) -> Vec<(PersonRecord, Vec<ImportantDate>)> {
    let mut out = Vec::new();
    let Some(conns) = json["connections"].as_array() else {
        return out;
    };

    for c in conns {
        let name = c["names"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|n| n["displayName"].as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let email = c["emailAddresses"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|e| e["value"].as_str())
            .unwrap_or("")
            .to_string();
        let phone = c["phoneNumbers"]
            .as_array()
            .and_then(|a| a.first())
            .and_then(|p| p["value"].as_str())
            .unwrap_or("")
            .to_string();

        let resource = c["resourceName"].as_str().unwrap_or(&name);
        let pid = short_id("gperson", resource);
        let person = PersonRecord {
            id: pid.clone(),
            name: name.clone(),
            relation: "".into(),
            email,
            phone,
            source_provider: ProviderKind::Google,
        };

        let mut dates = Vec::new();
        if let Some(bdays) = c["birthdays"].as_array() {
            for b in bdays {
                let m = b["date"]["month"].as_u64().unwrap_or(0) as u32;
                let d = b["date"]["day"].as_u64().unwrap_or(0) as u32;
                if (1..=12).contains(&m) && (1..=31).contains(&d) {
                    let y = b["date"]["year"].as_i64().map(|v| v as i32);
                    dates.push(ImportantDate {
                        id: short_id("gbday", &format!("{pid}-{m}-{d}")),
                        person_id: Some(pid.clone()),
                        person_name: name.clone(),
                        kind: "birthday".into(),
                        month: m,
                        day: d,
                        year_optional: y,
                        label: format!("Aniversário de {name}"),
                        source_provider: ProviderKind::Google,
                    });
                }
            }
        }
        if let Some(evs) = c["events"].as_array() {
            for (idx, ev) in evs.iter().enumerate() {
                let m = ev["date"]["month"].as_u64().unwrap_or(0) as u32;
                let d = ev["date"]["day"].as_u64().unwrap_or(0) as u32;
                if (1..=12).contains(&m) && (1..=31).contains(&d) {
                    let y = ev["date"]["year"].as_i64().map(|v| v as i32);
                    let typ = ev["formattedType"]
                        .as_str()
                        .or_else(|| ev["type"].as_str())
                        .unwrap_or("Data especial");
                    dates.push(ImportantDate {
                        id: short_id("gev", &format!("{pid}-{idx}-{m}-{d}")),
                        person_id: Some(pid.clone()),
                        person_name: name.clone(),
                        kind: "anniversary".into(),
                        month: m,
                        day: d,
                        year_optional: y,
                        label: format!("{typ} · {name}"),
                        source_provider: ProviderKind::Google,
                    });
                }
            }
        }
        out.push((person, dates));
    }
    out
}

/// Faz parse de vCard (.vcf) vindo do Apple CardDAV para extrair nomes e aniversários (BDAY).
pub fn parse_vcard_contacts(vcf_text: &str, provider: ProviderKind) -> Vec<(PersonRecord, Vec<ImportantDate>)> {
    let mut out = Vec::new();
    let mut in_card = false;
    let mut name = String::new();
    let mut email = String::new();
    let mut phone = String::new();
    let mut bday = String::new();

    for raw in vcf_text.lines() {
        let line = raw.trim();
        if line.eq_ignore_ascii_case("BEGIN:VCARD") {
            in_card = true;
            name.clear();
            email.clear();
            phone.clear();
            bday.clear();
            continue;
        }
        if line.eq_ignore_ascii_case("END:VCARD") {
            if in_card && !name.is_empty() {
                let pid = short_id("vcard", &format!("{}:{}", provider.as_str(), name));
                let person = PersonRecord {
                    id: pid.clone(),
                    name: name.clone(),
                    relation: "".into(),
                    email: email.clone(),
                    phone: phone.clone(),
                    source_provider: provider,
                };
                let mut dates = Vec::new();
                if let Some((m, d, y)) = parse_vcard_date(&bday) {
                    dates.push(ImportantDate {
                        id: short_id("vbday", &format!("{pid}-{m}-{d}")),
                        person_id: Some(pid),
                        person_name: name.clone(),
                        kind: "birthday".into(),
                        month: m,
                        day: d,
                        year_optional: y,
                        label: format!("Aniversário de {name}"),
                        source_provider: provider,
                    });
                }
                out.push((person, dates));
            }
            in_card = false;
            continue;
        }
        if !in_card {
            continue;
        }
        let Some((left, val)) = line.split_once(':') else { continue };
        let prop = left.split(';').next().unwrap_or("").to_ascii_uppercase();
        match prop.as_str() {
            "FN" if name.is_empty() => name = val.trim().to_string(),
            "EMAIL" if email.is_empty() => email = val.trim().to_string(),
            "TEL" if phone.is_empty() => phone = val.trim().to_string(),
            "BDAY" if bday.is_empty() => bday = val.trim().to_string(),
            _ => {}
        }
    }
    out
}

fn parse_vcard_date(s: &str) -> Option<(u32, u32, Option<i32>)> {
    let clean = s.trim();
    if clean.is_empty() {
        return None;
    }
    // Formatos: YYYY-MM-DD, YYYYMMDD, --MM-DD, --MMDD
    if let Some(rest) = clean.strip_prefix("--") {
        let digits: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 4 {
            let m = digits[0..2].parse::<u32>().ok()?;
            let d = digits[2..4].parse::<u32>().ok()?;
            return Some((m, d, None));
        }
        return None;
    }
    let digits: String = clean.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 8 {
        let y = digits[0..4].parse::<i32>().ok()?;
        let m = digits[4..6].parse::<u32>().ok()?;
        let d = digits[6..8].parse::<u32>().ok()?;
        return Some((m, d, Some(y)));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_google_contacts_birthdays() {
        let payload = json!({
            "connections": [{
                "resourceName": "people/c123",
                "names": [{"displayName": "Carlos Eduardo"}],
                "emailAddresses": [{"value": "carlos@gmail.com"}],
                "birthdays": [{"date": {"year": 1994, "month": 5, "day": 22}}]
            }]
        });
        let res = parse_google_contacts(&payload);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].0.name, "Carlos Eduardo");
        assert_eq!(res[0].1.len(), 1);
        assert_eq!(res[0].1[0].month, 5);
        assert_eq!(res[0].1[0].day, 22);
        assert_eq!(res[0].1[0].year_optional, Some(1994));
    }

    #[test]
    fn parses_apple_vcard_bday() {
        let vcf = "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Juliana Costa\r\nBDAY:--08-19\r\nEMAIL:ju@icloud.com\r\nEND:VCARD\r\n";
        let res = parse_vcard_contacts(vcf, ProviderKind::Apple);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].1[0].month, 8);
        assert_eq!(res[0].1[0].day, 19);
        assert_eq!(res[0].1[0].year_optional, None);
    }
}
