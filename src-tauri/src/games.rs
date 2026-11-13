//! Jogos instalados (Steam, Epic Games, GOG) e amigos da Steam online.
//! A chave da Steam Web API fica no Gerenciador de Credenciais do Windows.

use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Game {
    id: String,
    name: String,
    source: &'static str,
    /// URL/comando para abrir o jogo (steam://, com.epicgames.launcher://, caminho do .exe)
    launch: String,
    image: String,
    appid: u64,
    last_played: u64,
    size: u64,
}

/* ---------------- VDF (formato de texto da Steam) ---------------- */

/// Lê pares "chave" "valor" de uma linha VDF
fn kv(line: &str) -> Option<(String, String)> {
    let p: Vec<&str> = line.split('"').collect();
    if p.len() >= 5 {
        Some((p[1].to_lowercase(), p[3].replace("\\\\", "\\")))
    } else {
        None
    }
}

#[cfg(windows)]
fn steam_path() -> Option<PathBuf> {
    use winreg::enums::HKEY_CURRENT_USER;
    let k = winreg::RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam").ok()?;
    let p: String = k.get_value("SteamPath").ok()?;
    Some(PathBuf::from(p.replace('/', "\\")))
}
#[cfg(not(windows))]
fn steam_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".steam/steam")).filter(|p| p.exists())
}

fn steam_libraries(steam: &Path) -> Vec<PathBuf> {
    let mut libs = vec![steam.to_path_buf()];
    if let Ok(text) = std::fs::read_to_string(steam.join("steamapps").join("libraryfolders.vdf")) {
        for line in text.lines() {
            if let Some((k, v)) = kv(line) {
                if k == "path" {
                    let p = PathBuf::from(v);
                    if !libs.contains(&p) {
                        libs.push(p);
                    }
                }
            }
        }
    }
    libs
}

/// Ferramentas da Steam que não são jogos
const STEAM_SKIP: &[u64] = &[228980, 1070560, 1391110, 1628350, 1493710, 1826330, 2180100, 961940];

fn steam_games() -> Vec<Game> {
    let Some(steam) = steam_path() else { return vec![] };
    let mut out = Vec::new();
    for lib in steam_libraries(&steam) {
        let Ok(rd) = std::fs::read_dir(lib.join("steamapps")) else { continue };
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().to_string();
            if !(fname.starts_with("appmanifest_") && fname.ends_with(".acf")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(e.path()) else { continue };
            let (mut appid, mut name, mut last, mut size) = (0u64, String::new(), 0u64, 0u64);
            for line in text.lines() {
                if let Some((k, v)) = kv(line) {
                    match k.as_str() {
                        "appid" => appid = v.parse().unwrap_or(0),
                        "name" => name = v,
                        "lastplayed" => last = v.parse().unwrap_or(0),
                        "sizeondisk" => size = v.parse().unwrap_or(0),
                        _ => {}
                    }
                }
            }
            let lower = name.to_lowercase();
            if appid == 0 || name.is_empty() || STEAM_SKIP.contains(&appid)
                || lower.contains("redistributable") || lower.contains("steam linux runtime") || lower.starts_with("proton")
            {
                continue;
            }
            out.push(Game {
                id: format!("steam-{appid}"),
                name,
                source: "steam",
                launch: format!("steam://rungameid/{appid}"),
                image: format!("https://cdn.cloudflare.steamstatic.com/steam/apps/{appid}/header.jpg"),
                appid,
                last_played: last,
                size,
            });
        }
    }
    out
}

fn epic_games() -> Vec<Game> {
    let dir = std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"))
        .join("Epic\\EpicGamesLauncher\\Data\\Manifests");
    let Ok(rd) = std::fs::read_dir(dir) else { return vec![] };
    let mut out = Vec::new();
    for e in rd.flatten() {
        if e.path().extension().and_then(|x| x.to_str()) != Some("item") {
            continue;
        }
        let Ok(v) = std::fs::read_to_string(e.path()).map(|t| serde_json::from_str::<Value>(&t).unwrap_or(Value::Null)) else { continue };
        let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
        let (ns, item, app) = (s("CatalogNamespace"), s("CatalogItemId"), s("AppName"));
        if s("DisplayName").is_empty() || app.is_empty() {
            continue;
        }
        // ignora complementos (DLC) instalados como item separado
        if v["bIsApplication"].as_bool() == Some(false) {
            continue;
        }
        out.push(Game {
            id: format!("epic-{app}"),
            name: s("DisplayName"),
            source: "epic",
            launch: format!("com.epicgames.launcher://apps/{ns}%3A{item}%3A{app}?action=launch&silent=true"),
            image: String::new(),
            appid: 0,
            last_played: 0,
            size: v["InstallSize"].as_u64().unwrap_or(0),
        });
    }
    out
}

#[cfg(windows)]
fn gog_games() -> Vec<Game> {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let Ok(root) = winreg::RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey("SOFTWARE\\WOW6432Node\\GOG.com\\Games") else { return vec![] };
    let mut out = Vec::new();
    for id in root.enum_keys().flatten() {
        let Ok(k) = root.open_subkey(&id) else { continue };
        let name: String = k.get_value("gameName").unwrap_or_default();
        let exe: String = k.get_value("exe").unwrap_or_default();
        if name.is_empty() || exe.is_empty() {
            continue;
        }
        out.push(Game {
            id: format!("gog-{id}"),
            name,
            source: "gog",
            launch: exe,
            image: String::new(),
            appid: 0,
            last_played: 0,
            size: 0,
        });
    }
    out
}
#[cfg(not(windows))]
fn gog_games() -> Vec<Game> {
    vec![]
}

#[tauri::command]
pub async fn games_list() -> Vec<Game> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut all = steam_games();
        all.extend(epic_games());
        all.extend(gog_games());
        all.sort_by(|a, b| b.last_played.cmp(&a.last_played).then(a.name.to_lowercase().cmp(&b.name.to_lowercase())));
        all
    })
    .await
    .unwrap_or_default()
}

/* ---------------- Steam: conta local e amigos ---------------- */

#[derive(Serialize)]
pub struct SteamInfo {
    installed: bool,
    steamid: String,
    persona: String,
    has_key: bool,
}

/// SteamID64 e nome do último usuário logado (config/loginusers.vdf)
fn local_user() -> Option<(String, String)> {
    let text = std::fs::read_to_string(steam_path()?.join("config").join("loginusers.vdf")).ok()?;
    let (mut cur, mut persona, mut best) = (String::new(), String::new(), None);
    for line in text.lines() {
        let t = line.trim().trim_matches('"');
        if t.len() == 17 && t.chars().all(|c| c.is_ascii_digit()) {
            cur = t.to_string();
            persona.clear();
            continue;
        }
        if let Some((k, v)) = kv(line) {
            if k == "personaname" {
                persona = v.clone();
            }
            if k == "mostrecent" && v == "1" && !cur.is_empty() {
                best = Some((cur.clone(), persona.clone()));
            }
        }
        if best.is_none() && !cur.is_empty() && !persona.is_empty() && line.trim() == "}" {
            best = Some((cur.clone(), persona.clone()));
        }
    }
    best
}

const KEY_SERVICE: &str = "ilha-dinamica-steam";

fn api_key() -> Option<String> {
    keyring::Entry::new(KEY_SERVICE, "webapi").ok()?.get_password().ok()
}

#[tauri::command(async)]
pub fn steam_info() -> SteamInfo {
    let user = local_user();
    SteamInfo {
        installed: steam_path().is_some(),
        steamid: user.as_ref().map(|u| u.0.clone()).unwrap_or_default(),
        persona: user.map(|u| u.1).unwrap_or_default(),
        has_key: api_key().is_some(),
    }
}

#[tauri::command(async)]
pub fn steam_set_key(key: String) -> Result<bool, String> {
    let e = keyring::Entry::new(KEY_SERVICE, "webapi").map_err(|e| e.to_string())?;
    if key.trim().is_empty() {
        let _ = e.delete_credential();
        return Ok(false);
    }
    e.set_password(key.trim()).map_err(|e| e.to_string())?;
    Ok(true)
}

#[derive(Serialize)]
pub struct Friend {
    steamid: String,
    name: String,
    avatar: String,
    /// 0 offline, 1 online, 2 ocupado, 3 ausente, 4 soneca, 5 quer trocar, 6 quer jogar
    state: u64,
    game: String,
    gameid: String,
    lobby: String,
}

#[tauri::command]
pub async fn steam_friends(steamid: String) -> Result<Vec<Friend>, String> {
    let key = api_key().ok_or("Cadastre sua chave da Steam Web API")?;
    let id = if steamid.trim().is_empty() { local_user().map(|u| u.0).unwrap_or_default() } else { steamid.trim().to_string() };
    if id.is_empty() {
        return Err("Não encontrei seu SteamID — informe nas configurações".into());
    }
    let c = reqwest::Client::builder().timeout(Duration::from_secs(12)).build().map_err(|e| e.to_string())?;
    let base = "https://api.steampowered.com/ISteamUser";
    let r = c
        .get(format!("{base}/GetFriendList/v1/?key={key}&steamid={id}&relationship=friend"))
        .send()
        .await
        .map_err(|_| "Sem conexão com a Steam".to_string())?;
    match r.status().as_u16() {
        401 => return Err("Sua lista de amigos está privada (Steam → Perfil → Privacidade → Lista de amigos: Pública)".into()),
        403 => return Err("Chave da Steam Web API inválida".into()),
        s if s >= 400 => return Err(format!("Steam respondeu {s}")),
        _ => {}
    }
    let list: Value = r.json().await.map_err(|e| e.to_string())?;
    let ids: Vec<String> = list["friendslist"]["friends"]
        .as_array()
        .map(|a| a.iter().filter_map(|f| f["steamid"].as_str().map(String::from)).collect())
        .unwrap_or_default();

    let mut out = Vec::new();
    for chunk in ids.chunks(100) {
        let url = format!("{base}/GetPlayerSummaries/v2/?key={key}&steamids={}", chunk.join(","));
        let Ok(resp) = c.get(url).send().await else { continue };
        let Ok(v) = resp.json::<Value>().await else { continue };
        for p in v["response"]["players"].as_array().cloned().unwrap_or_default() {
            let s = |k: &str| p[k].as_str().unwrap_or("").to_string();
            let state = p["personastate"].as_u64().unwrap_or(0);
            let game = s("gameextrainfo");
            if state == 0 && game.is_empty() {
                continue; // offline
            }
            out.push(Friend {
                steamid: s("steamid"),
                name: s("personaname"),
                avatar: s("avatarmedium"),
                state,
                game,
                gameid: s("gameid"),
                lobby: s("lobbysteamid"),
            });
        }
    }
    // quem está jogando primeiro, depois online, depois ausente
    out.sort_by_key(|f| (f.game.is_empty(), f.state != 1, f.name.to_lowercase()));
    Ok(out)
}
