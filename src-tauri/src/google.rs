//! Conector Google (OAuth 2.0 Desktop + PKCE loopback em 127.0.0.1).
//! Consome Google Calendar, Google Contacts (aniversários e datas especiais) e perfil do usuário.

use assistant_core::domain::{AccountRecord, CanonicalEvent, ImportantDate, PersonRecord, ProviderKind};
use assistant_core::normalizer::{parse_google_calendar_events, parse_google_contacts};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::vault;

const GOOGLE_VAULT_KEY: &str = "google:oauth";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokens {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub email: String,
    pub name: String,
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn pkce_pair() -> (String, String) {
    let seed = format!(
        "{}-{}-{}",
        now_ts(),
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(1)
    );
    let mut h1 = Sha256::new();
    h1.update(seed.as_bytes());
    let verifier = URL_SAFE_NO_PAD.encode(h1.finalize());

    let mut h2 = Sha256::new();
    h2.update(verifier.as_bytes());
    let challenge = URL_SAFE_NO_PAD.encode(h2.finalize());
    (verifier, challenge)
}

fn parse_query_param(req: &str, key: &str) -> Option<String> {
    let first_line = req.lines().next()?;
    let path = first_line.split_whitespace().nth(1)?;
    let query = path.split_once('?')?.1;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=')?;
        if k == key {
            return Some(url_decode(v));
        }
    }
    None
}

fn url_decode(s: &str) -> String {
    let mut bytes = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(hex) = std::str::from_utf8(&b[i + 1..i + 3]) {
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    bytes.push(v);
                    i += 3;
                    continue;
                }
            }
        }
        bytes.push(if b[i] == b'+' { b' ' } else { b[i] });
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

pub fn load_tokens() -> Option<GoogleTokens> {
    let raw = vault::get_secret(GOOGLE_VAULT_KEY)?;
    serde_json::from_str(&raw).ok()
}

pub fn save_tokens(t: &GoogleTokens) -> Result<(), String> {
    let raw = serde_json::to_string(t).map_err(|e| e.to_string())?;
    vault::set_secret(GOOGLE_VAULT_KEY, &raw)
}

pub fn clear_tokens() {
    vault::delete_secret(GOOGLE_VAULT_KEY);
}

/// Inicia o fluxo OAuth 2.0 PKCE em servidor loopback local (`127.0.0.1`),
/// abre o navegador do usuário e troca o `code` por `access_token` + `refresh_token`.
pub async fn start_oauth_pkce(
    client_id: Option<String>,
    client_secret: Option<String>,
) -> Result<GoogleTokens, String> {
    let cid = client_id
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| option_env!("ILHA_GOOGLE_CLIENT_ID").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
        .or_else(|| std::env::var("ILHA_GOOGLE_CLIENT_ID").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
        .ok_or_else(|| "OAuth Google do aplicativo não configurado neste build. Abra Config. avançada na tela de Login para informar um Client ID.".to_string())?;

    let client_secret = client_secret
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| option_env!("ILHA_GOOGLE_CLIENT_SECRET").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()))
        .or_else(|| std::env::var("ILHA_GOOGLE_CLIENT_SECRET").ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()));

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/oauth/callback");
    let (verifier, challenge) = pkce_pair();

    let scope = "openid email profile https://www.googleapis.com/auth/calendar.readonly https://www.googleapis.com/auth/contacts.readonly";
    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt=consent",
        url_encode(&cid),
        url_encode(&redirect_uri),
        url_encode(scope),
        url_encode(&challenge),
    );

    if !crate::actions::run_action(auth_url) {
        return Err("Não foi possível abrir o navegador para autenticação Google".into());
    }

    // Aguarda retorno do navegador por até 180 segundos em thread dedicada
    let code = tokio::task::spawn_blocking(move || -> Result<String, String> {
        listener.set_nonblocking(false).map_err(|e| e.to_string())?;
        let started = SystemTime::now();
        loop {
            if started.elapsed().unwrap_or_default() > Duration::from_secs(180) {
                return Err("Tempo esgotado aguardando confirmação no navegador".into());
            }
            if let Ok((mut stream, _)) = listener.accept() {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]);
                if let Some(err) = parse_query_param(&req, "error") {
                    let html = "<html><body style='font-family:sans-serif;background:#111;color:#fff;padding:32px'><h2>Autenticacao cancelada</h2><p>Voce pode fechar esta aba e voltar para a Ilha Dinamica.</p></body></html>";
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{html}");
                    return Err(format!("Google OAuth cancelado: {err}"));
                }
                if let Some(code) = parse_query_param(&req, "code") {
                    let html = "<html><body style='font-family:sans-serif;background:#111;color:#5be39a;padding:32px'><h2>Google conectado a Ilha Dinamica!</h2><p style='color:#ddd'>Voce ja pode fechar esta aba.</p></body></html>";
                    let _ = write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{html}");
                    return Ok(code);
                }
                let _ = write!(stream, "HTTP/1.1 404 Not Found\r\n\r\n");
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())?;

    let mut form = vec![
        ("code", code),
        ("client_id", cid.clone()),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code".into()),
        ("code_verifier", verifier),
    ];
    if let Some(sec) = client_secret.as_ref().filter(|s| !s.trim().is_empty()) {
        form.push(("client_secret", sec.trim().to_string()));
    }

    let tok_val: Value = http
        .post("https://oauth2.googleapis.com/token")
        .form(&form)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let access_token = tok_val["access_token"]
        .as_str()
        .ok_or_else(|| {
            tok_val["error_description"]
                .as_str()
                .unwrap_or("Falha ao obter token Google")
                .to_string()
        })?
        .to_string();

    let refresh_token = tok_val["refresh_token"].as_str().map(|s| s.to_string());
    let expires_in = tok_val["expires_in"].as_i64().unwrap_or(3600);

    let profile: Value = http
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(&access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();

    let tokens = GoogleTokens {
        client_id: cid,
        client_secret,
        access_token,
        refresh_token,
        expires_at: now_ts() + expires_in - 60,
        email: profile["email"].as_str().unwrap_or("Conta Google").to_string(),
        name: profile["name"].as_str().unwrap_or("Google").to_string(),
    };
    save_tokens(&tokens)?;
    Ok(tokens)
}

async fn ensure_fresh_token(http: &reqwest::Client) -> Result<GoogleTokens, String> {
    let mut t = load_tokens().ok_or("Conta Google não conectada")?;
    if now_ts() < t.expires_at {
        return Ok(t);
    }
    let Some(ref rt) = t.refresh_token else {
        return Ok(t);
    };

    let mut form = vec![
        ("client_id", t.client_id.clone()),
        ("grant_type", "refresh_token".into()),
        ("refresh_token", rt.clone()),
    ];
    if let Some(sec) = t.client_secret.as_ref().filter(|s| !s.trim().is_empty()) {
        form.push(("client_secret", sec.trim().to_string()));
    }

    let val: Value = http
        .post("https://oauth2.googleapis.com/token")
        .form(&form)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    if let Some(at) = val["access_token"].as_str() {
        t.access_token = at.to_string();
        t.expires_at = now_ts() + val["expires_in"].as_i64().unwrap_or(3600) - 60;
        save_tokens(&t)?;
    }
    Ok(t)
}

pub struct GoogleSyncBundle {
    pub account: AccountRecord,
    pub events: Vec<CanonicalEvent>,
    pub contacts: Vec<(PersonRecord, Vec<ImportantDate>)>,
}

pub async fn sync_google() -> Result<GoogleSyncBundle, String> {
    let http = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    let tok = ensure_fresh_token(&http).await?;
    let account_id = format!("google:{}", tok.email);

    let time_min = (Utc::now() - ChronoDuration::days(1)).to_rfc3339();
    let time_max = (Utc::now() + ChronoDuration::days(30)).to_rfc3339();
    let cal_url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/primary/events?singleEvents=true&orderBy=startTime&maxResults=80&timeMin={}&timeMax={}",
        url_encode(&time_min),
        url_encode(&time_max)
    );
    let cal_json: Value = http
        .get(&cal_url)
        .bearer_auth(&tok.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();

    let events = parse_google_calendar_events(&cal_json, &account_id);

    let people_url = "https://people.googleapis.com/v1/people/me/connections?personFields=names,birthdays,events,emailAddresses,phoneNumbers&pageSize=300";
    let people_json: Value = http
        .get(people_url)
        .bearer_auth(&tok.access_token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();

    let contacts = parse_google_contacts(&people_json);

    let account = AccountRecord {
        id: account_id,
        provider: ProviderKind::Google,
        label: if tok.name.is_empty() { "Google".into() } else { tok.name },
        email: tok.email,
        is_primary: true,
        status: "connected".into(),
        last_sync_ts: now_ts(),
        details: format!("{} eventos · {} contatos", events.len(), contacts.len()),
    };

    Ok(GoogleSyncBundle {
        account,
        events,
        contacts,
    })
}
