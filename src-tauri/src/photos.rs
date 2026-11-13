//! Integração com Google Fotos via OAuth 2.0 (loopback) + Google Photos Picker API.
//! Credenciais e tokens ficam no Gerenciador de Credenciais do Windows;
//! as fotos escolhidas são salvas localmente em cache para exibição instantânea na Home.

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::Serialize;
use serde_json::Value;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const SERVICE: &str = "ilha-dinamica-gphotos";
const PICKER_SCOPE: &str = "https://www.googleapis.com/auth/photospicker.mediaitems.readonly";

#[derive(Serialize)]
pub struct GPhotosStatus {
    pub connected: bool,
    pub has_client: bool,
    pub count: usize,
}

fn get_secret(key: &str) -> Option<String> {
    keyring::Entry::new(SERVICE, key).ok()?.get_password().ok().filter(|s| !s.trim().is_empty())
}

fn resolve_client_id() -> Option<String> {
    get_secret("client_id").or_else(|| {
        option_env!("GPHOTOS_CLIENT_ID")
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
    })
}

fn resolve_client_secret() -> String {
    get_secret("client_secret")
        .or_else(|| {
            option_env!("GPHOTOS_CLIENT_SECRET")
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
        })
        .unwrap_or_default()
}

fn set_secret(key: &str, val: &str) -> Result<(), String> {
    let e = keyring::Entry::new(SERVICE, key).map_err(|e| e.to_string())?;
    if val.trim().is_empty() {
        let _ = e.delete_credential();
    } else {
        e.set_password(val.trim()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn photos_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?.join("photos");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn list_cached_files(app: &AppHandle) -> Vec<PathBuf> {
    let Ok(dir) = photos_dir(app) else { return vec![] };
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut files: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && matches!(p.extension().and_then(|s| s.to_str()), Some("jpg" | "jpeg" | "png" | "webp")))
        .collect();
    files.sort();
    files
}

fn read_data_urls(app: &AppHandle) -> Vec<String> {
    list_cached_files(app)
        .into_iter()
        .take(50)
        .filter_map(|p| {
            let bytes = std::fs::read(&p).ok()?;
            let mime = match p.extension().and_then(|s| s.to_str()) {
                Some("png") => "image/png",
                Some("webp") => "image/webp",
                _ => "image/jpeg",
            };
            Some(format!("data:{mime};base64,{}", B64.encode(bytes)))
        })
        .collect()
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn url_decode(s: &str) -> String {
    let s = s.replace('+', " ");
    let mut bytes = Vec::with_capacity(s.len());
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or(""), 16) {
                bytes.push(v);
                i += 3;
                continue;
            }
        }
        bytes.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).to_string()
}

async fn refresh_access_token() -> Result<String, String> {
    let client_id = resolve_client_id().ok_or("Client ID do Google não configurado")?;
    let client_secret = resolve_client_secret();
    let refresh_token = get_secret("refresh_token").ok_or("Conta Google não vinculada")?;

    let mut params = vec![
        ("client_id", client_id.as_str()),
        ("refresh_token", refresh_token.as_str()),
        ("grant_type", "refresh_token"),
    ];
    if !client_secret.is_empty() {
        params.push(("client_secret", client_secret.as_str()));
    }

    let c = http_client()?;
    let resp = c
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let msg = json["error_description"].as_str().or(json["error"].as_str()).unwrap_or("erro ao renovar token");
        return Err(format!("Google OAuth: {msg}"));
    }
    let token = json["access_token"].as_str().ok_or("access_token ausente")?.to_string();
    let _ = set_secret("access_token", &token);
    Ok(token)
}

#[tauri::command]
pub fn gphotos_status(app: AppHandle) -> GPhotosStatus {
    GPhotosStatus {
        connected: get_secret("refresh_token").is_some(),
        has_client: resolve_client_id().is_some(),
        count: list_cached_files(&app).len(),
    }
}

#[tauri::command]
pub fn gphotos_set_client(client_id: String, client_secret: String) -> Result<bool, String> {
    set_secret("client_id", &client_id)?;
    set_secret("client_secret", &client_secret)?;
    Ok(resolve_client_id().is_some())
}

#[tauri::command]
pub fn gphotos_list(app: AppHandle) -> Vec<String> {
    read_data_urls(&app)
}

#[tauri::command]
pub async fn gphotos_pick_folder(app: AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let win = app.get_webview_window("main");
    if let Some(w) = &win {
        let _ = w.set_always_on_top(false);
    }
    let picked = app.dialog().file().set_title("Escolha a pasta com suas fotos").blocking_pick_folder();
    if let Some(w) = &win {
        let _ = w.set_always_on_top(true);
    }
    let Some(folder) = picked.and_then(|p| p.into_path().ok()) else {
        return Ok(read_data_urls(&app));
    };
    let dir = photos_dir(&app)?;
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let _ = std::fs::remove_file(e.path());
        }
    }
    if let Ok(rd) = std::fs::read_dir(folder) {
        let mut imgs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && p.extension()
                        .and_then(|s| s.to_str())
                        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"))
                        .unwrap_or(false)
            })
            .collect();
        imgs.sort();
        for (i, src) in imgs.into_iter().take(40).enumerate() {
            let ext = src.extension().and_then(|s| s.to_str()).unwrap_or("jpg").to_ascii_lowercase();
            let _ = std::fs::copy(&src, dir.join(format!("local_{i:02}.{ext}")));
        }
    }
    Ok(read_data_urls(&app))
}

#[tauri::command]
pub fn gphotos_disconnect(app: AppHandle) -> Result<(), String> {
    let _ = set_secret("refresh_token", "");
    let _ = set_secret("access_token", "");
    if let Ok(dir) = photos_dir(&app) {
        if let Ok(rd) = std::fs::read_dir(dir) {
            for e in rd.flatten() {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn gphotos_connect() -> Result<(), String> {
    let client_id = resolve_client_id().ok_or("Informe o Client ID do Google Cloud primeiro")?;
    let client_secret = resolve_client_secret();

    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let redirect_uri = format!("http://127.0.0.1:{port}");

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&access_type=offline&prompt=consent",
        url_encode(&client_id),
        url_encode(&redirect_uri),
        url_encode(PICKER_SCOPE)
    );
    crate::actions::run_action(auth_url);

    // aguarda o callback OAuth no loopback em thread bloqueante
    let code = tauri::async_runtime::spawn_blocking(move || -> Result<String, String> {
        let deadline = Instant::now() + Duration::from_secs(180);
        loop {
            if Instant::now() > deadline {
                return Err("Tempo esgotado aguardando login do Google".into());
            }
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 4096];
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    let first_line = req.lines().next().unwrap_or("");
                    let path = first_line.split_whitespace().nth(1).unwrap_or("");
                    let query = path.split_once('?').map(|x| x.1).unwrap_or("");
                    let mut code = None;
                    let mut err = None;
                    for pair in query.split('&') {
                        if let Some((k, v)) = pair.split_once('=') {
                            if k == "code" { code = Some(url_decode(v)); }
                            if k == "error" { err = Some(url_decode(v)); }
                        }
                    }
                    let html = if code.is_some() {
                        "<html><body style='font-family:sans-serif;background:#111;color:#fff;display:grid;place-items:center;height:100vh'><div><h2>✅ Conta Google vinculada!</h2><p>Você pode fechar esta aba e voltar para a Ilha Dinâmica.</p></div></body></html>"
                    } else {
                        "<html><body style='font-family:sans-serif;background:#111;color:#ff8a7e;display:grid;place-items:center;height:100vh'><div><h2>⚠️ Autorização cancelada</h2><p>Volte para a Ilha Dinâmica para tentar novamente.</p></div></body></html>"
                    };
                    let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", html.len(), html);
                    let _ = stream.write_all(resp.as_bytes());
                    let _ = stream.flush();
                    if let Some(c) = code { return Ok(c); }
                    if let Some(e) = err { return Err(format!("Autorização negada: {e}")); }
                    return Err("Código OAuth não recebido".into());
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(120));
                }
                Err(e) => return Err(e.to_string()),
            }
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut params = vec![
        ("code", code.as_str()),
        ("client_id", client_id.as_str()),
        ("redirect_uri", redirect_uri.as_str()),
        ("grant_type", "authorization_code"),
    ];
    if !client_secret.is_empty() {
        params.push(("client_secret", client_secret.as_str()));
    }
    let c = http_client()?;
    let resp = c
        .post("https://oauth2.googleapis.com/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status();
    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    if !status.is_success() {
        let msg = json["error_description"].as_str().or(json["error"].as_str()).unwrap_or("falha ao trocar código");
        return Err(format!("Google OAuth: {msg}"));
    }
    if let Some(at) = json["access_token"].as_str() {
        set_secret("access_token", at)?;
    }
    if let Some(rt) = json["refresh_token"].as_str() {
        set_secret("refresh_token", rt)?;
    } else if get_secret("refresh_token").is_none() {
        return Err("Google não retornou refresh_token. Remova o acesso do app em myaccount.google.com/permissions e tente novamente.".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn gphotos_pick(app: AppHandle) -> Result<Vec<String>, String> {
    let token = refresh_access_token().await?;
    let c = http_client()?;

    // 1. Cria sessão no Google Photos Picker
    let sess: Value = c
        .post("https://photospicker.googleapis.com/v1/sessions")
        .bearer_auth(&token)
        .json(&serde_json::json!({ "pickingConfig": { "maxItemCount": 30 } }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let sess_id = sess["id"].as_str().ok_or_else(|| {
        let msg = sess["error"]["message"].as_str().unwrap_or("falha ao criar sessão do Google Photos Picker");
        msg.to_string()
    })?.to_string();
    let picker_uri = sess["pickerUri"].as_str().ok_or("pickerUri ausente")?.to_string();

    // Abre o seletor no navegador do usuário
    crate::actions::run_action(format!("{picker_uri}/autoclose"));

    // 2. Aguarda o usuário concluir a seleção (até 3 min)
    let deadline = Instant::now() + Duration::from_secs(180);
    loop {
        if Instant::now() > deadline {
            return Err("Tempo esgotado aguardando seleção de fotos".into());
        }
        let _ = tauri::async_runtime::spawn_blocking(|| std::thread::sleep(Duration::from_millis(2500))).await;
        let poll: Value = c
            .get(format!("https://photospicker.googleapis.com/v1/sessions/{sess_id}"))
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        if poll["mediaItemsSet"].as_bool() == Some(true) {
            break;
        }
    }

    // 3. Lista itens selecionados e baixa para cache local
    let items_resp: Value = c
        .get("https://photospicker.googleapis.com/v1/mediaItems")
        .query(&[("sessionId", sess_id.as_str()), ("pageSize", "30")])
        .bearer_auth(&token)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let dir = photos_dir(&app)?;
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let _ = std::fs::remove_file(e.path());
        }
    }

    if let Some(arr) = items_resp["mediaItems"].as_array() {
        for (i, item) in arr.iter().enumerate() {
            if item["type"].as_str() != Some("PHOTO") {
                continue;
            }
            let Some(base_url) = item["mediaFile"]["baseUrl"].as_str() else { continue };
            let url = format!("{base_url}=w720-h720");
            if let Ok(resp) = c.get(&url).bearer_auth(&token).send().await {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes().await {
                        let _ = std::fs::write(dir.join(format!("photo_{i:02}.jpg")), &bytes);
                    }
                }
            }
        }
    }

    // Encerra sessão no Google (boa prática)
    let _ = c
        .delete(format!("https://photospicker.googleapis.com/v1/sessions/{sess_id}"))
        .bearer_auth(&token)
        .send()
        .await;

    Ok(read_data_urls(&app))
}
