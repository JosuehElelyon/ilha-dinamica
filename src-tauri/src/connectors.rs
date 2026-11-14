//! Conectores secundários: GitHub (PRs e notificações) e Apple iCloud (CalDAV + CardDAV).

use assistant_core::domain::{AccountRecord, DevWorkItem, ImportantDate, PersonRecord, ProviderKind};
use assistant_core::normalizer::parse_vcard_contacts;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::vault;

const GITHUB_VAULT_KEY: &str = "github:oauth";
const APPLE_VAULT_KEY: &str = "apple:icloud";

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppleCreds {
    apple_id: String,
    app_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GithubDeviceChallenge {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

/* ==================== GITHUB ==================== */

pub fn save_github_token(token: &str) -> Result<(), String> {
    if token.trim().is_empty() {
        vault::delete_secret(GITHUB_VAULT_KEY);
        return Ok(());
    }
    vault::set_secret(GITHUB_VAULT_KEY, token.trim())
}

pub fn has_github_token() -> bool {
    vault::get_secret(GITHUB_VAULT_KEY).is_some()
}

pub async fn start_github_device_flow(client_id: &str) -> Result<GithubDeviceChallenge, String> {
    let cid = client_id.trim();
    if cid.is_empty() {
        return Err("Informe o Client ID do GitHub OAuth App (Device Flow)".into());
    }
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    let val: Value = http
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", cid), ("scope", "read:user repo notifications")])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let device_code = val["device_code"]
        .as_str()
        .ok_or_else(|| {
            val["error_description"]
                .as_str()
                .unwrap_or("Falha ao iniciar GitHub Device Flow")
                .to_string()
        })?
        .to_string();

    Ok(GithubDeviceChallenge {
        device_code,
        user_code: val["user_code"].as_str().unwrap_or("").to_string(),
        verification_uri: val["verification_uri"]
            .as_str()
            .unwrap_or("https://github.com/login/device")
            .to_string(),
        expires_in: val["expires_in"].as_u64().unwrap_or(900),
        interval: val["interval"].as_u64().unwrap_or(5),
    })
}

pub async fn poll_github_device_flow(
    client_id: &str,
    device_code: &str,
) -> Result<Option<String>, String> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    let val: Value = http
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", client_id.trim()),
            ("device_code", device_code.trim()),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ])
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(tok) = val["access_token"].as_str() {
        save_github_token(tok)?;
        return Ok(Some(tok.to_string()));
    }
    match val["error"].as_str() {
        Some("authorization_pending") | Some("slow_down") => Ok(None),
        Some(err) => Err(val["error_description"]
            .as_str()
            .unwrap_or(err)
            .to_string()),
        None => Ok(None),
    }
}

pub async fn sync_github() -> Result<(AccountRecord, Vec<DevWorkItem>), String> {
    let tok = vault::get_secret(GITHUB_VAULT_KEY).ok_or("Token GitHub não configurado")?;
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    let me: Value = http
        .get("https://api.github.com/user")
        .bearer_auth(&tok)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let login = me["login"].as_str().unwrap_or("").to_string();
    if login.is_empty() {
        return Err("Token GitHub inválido ou expirado".into());
    }

    let mut items = Vec::new();
    let q = format!("https://api.github.com/search/issues?q=is:open+is:pr+review-requested:{login}&per_page=10");
    if let Ok(resp) = http
        .get(&q)
        .bearer_auth(&tok)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        if let Ok(val) = resp.json::<Value>().await {
            if let Some(arr) = val["items"].as_array() {
                for it in arr {
                    let url = it["html_url"].as_str().unwrap_or("").to_string();
                    let repo = it["repository_url"]
                        .as_str()
                        .and_then(|u| u.rsplit('/').take(2).collect::<Vec<_>>().into_iter().rev().collect::<Vec<_>>().join("/").into())
                        .unwrap_or_default();
                    items.push(DevWorkItem {
                        id: format!("gh-pr-{}", it["id"].as_u64().unwrap_or(0)),
                        provider: ProviderKind::GitHub,
                        repo,
                        title: it["title"].as_str().unwrap_or("PR").to_string(),
                        kind: "review".into(),
                        url,
                        updated_at: it["updated_at"].as_str().unwrap_or("").to_string(),
                    });
                }
            }
        }
    }

    if let Ok(resp) = http
        .get("https://api.github.com/notifications?per_page=8")
        .bearer_auth(&tok)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
    {
        if let Ok(Value::Array(arr)) = resp.json::<Value>().await {
            for n in arr {
                let title = n["subject"]["title"].as_str().unwrap_or("Notificação").to_string();
                let repo = n["repository"]["full_name"].as_str().unwrap_or("").to_string();
                items.push(DevWorkItem {
                    id: format!("gh-notif-{}", n["id"].as_str().unwrap_or("0")),
                    provider: ProviderKind::GitHub,
                    repo: repo.clone(),
                    title,
                    kind: "notification".into(),
                    url: format!("https://github.com/{repo}"),
                    updated_at: n["updated_at"].as_str().unwrap_or("").to_string(),
                });
            }
        }
    }

    let acc = AccountRecord {
        id: format!("github:{login}"),
        provider: ProviderKind::GitHub,
        label: format!("@{login}"),
        email: me["email"].as_str().unwrap_or("").to_string(),
        is_primary: false,
        status: "connected".into(),
        last_sync_ts: now_ts(),
        details: format!("{} pendências/notificações", items.len()),
    };
    Ok((acc, items))
}

/* ==================== APPLE iCLOUD (CardDAV / CalDAV) ==================== */

pub fn save_apple_creds(apple_id: &str, app_password: &str) -> Result<(), String> {
    if apple_id.trim().is_empty() || app_password.trim().is_empty() {
        vault::delete_secret(APPLE_VAULT_KEY);
        return Ok(());
    }
    let creds = AppleCreds {
        apple_id: apple_id.trim().to_string(),
        app_password: app_password.trim().to_string(),
    };
    let raw = serde_json::to_string(&creds).map_err(|e| e.to_string())?;
    vault::set_secret(APPLE_VAULT_KEY, &raw)
}

pub fn load_apple_creds() -> Option<(String, String)> {
    let raw = vault::get_secret(APPLE_VAULT_KEY)?;
    let c: AppleCreds = serde_json::from_str(&raw).ok()?;
    Some((c.apple_id, c.app_password))
}

pub async fn sync_apple_icloud() -> Result<(AccountRecord, Vec<(PersonRecord, Vec<ImportantDate>)>), String> {
    let (apple_id, app_password) =
        load_apple_creds().ok_or("Credenciais Apple iCloud não configuradas")?;

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    // CardDAV principal discovery no iCloud
    let propfind_body = r#"<?xml version="1.0" encoding="utf-8" ?>
<d:propfind xmlns:d="DAV:"><d:prop><d:current-user-principal/></d:prop></d:propfind>"#;

    let resp = http
        .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), "https://contacts.icloud.com/")
        .basic_auth(&apple_id, Some(&app_password))
        .header("Depth", "0")
        .header("Content-Type", "application/xml; charset=utf-8")
        .body(propfind_body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status().as_u16() == 401 {
        return Err("Apple ID ou Senha de App inválida".into());
    }
    if !resp.status().is_success() && resp.status().as_u16() != 207 {
        return Err(format!("iCloud respondeu HTTP {}", resp.status().as_u16()));
    }

    let xml = resp.text().await.unwrap_or_default();
    let mut contacts = Vec::new();
    // Se o XML trouxer caminho do principal, tenta buscar o catálogo de contatos vCard
    if let Some(principal) = extract_xml_href(&xml) {
        let addr_url = format!("https://contacts.icloud.com{}/card/", principal.trim_end_matches('/'));
        let report_body = r#"<?xml version="1.0" encoding="utf-8" ?>
<card:addressbook-query xmlns:d="DAV:" xmlns:card="urn:ietf:params:xml:ns:carddav">
  <d:prop><d:getetag/><card:address-data/></d:prop>
</card:addressbook-query>"#;
        if let Ok(r2) = http
            .request(reqwest::Method::from_bytes(b"REPORT").unwrap(), &addr_url)
            .basic_auth(&apple_id, Some(&app_password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(report_body)
            .send()
            .await
        {
            if let Ok(body2) = r2.text().await {
                contacts = parse_vcard_contacts(&body2, ProviderKind::Apple);
            }
        }
    }

    let acc = AccountRecord {
        id: format!("apple:{apple_id}"),
        provider: ProviderKind::Apple,
        label: "Apple iCloud".into(),
        email: apple_id,
        is_primary: false,
        status: "connected".into(),
        last_sync_ts: now_ts(),
        details: format!("CalDAV/CardDAV ativo · {} contatos", contacts.len()),
    };
    Ok((acc, contacts))
}

fn extract_xml_href(xml: &str) -> Option<String> {
    let start = xml.find("<href").or_else(|| xml.find("<D:href")).or_else(|| xml.find("<d:href"))?;
    let gt = xml[start..].find('>')? + start + 1;
    let end = xml[gt..].find("</")? + gt;
    let val = xml[gt..end].trim();
    if val.is_empty() { None } else { Some(val.to_string()) }
}
