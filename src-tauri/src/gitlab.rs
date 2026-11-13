//! Merge Requests do GitLab de cada órgão. O token de acesso fica no
//! Gerenciador de Credenciais do Windows (nunca em arquivo nem na interface).

use serde::Serialize;
use serde_json::Value;
use std::time::Duration;

const SERVICE: &str = "ilha-dinamica-gitlab";

fn entry(org: &str) -> Result<keyring::Entry, String> {
    keyring::Entry::new(SERVICE, org).map_err(|e| e.to_string())
}

fn token(org: &str) -> Option<String> {
    entry(org).ok()?.get_password().ok()
}

/// Salva (ou apaga, se vazio) o token do GitLab de um órgão
#[tauri::command(async)]
pub fn gitlab_set_token(org: String, token: String) -> Result<bool, String> {
    let e = entry(&org)?;
    if token.trim().is_empty() {
        let _ = e.delete_credential();
        return Ok(false);
    }
    e.set_password(token.trim()).map_err(|e| e.to_string())?;
    Ok(true)
}

/// Quais órgãos já têm token salvo
#[tauri::command(async)]
pub fn gitlab_has_tokens(orgs: Vec<String>) -> Vec<String> {
    orgs.into_iter().filter(|o| token(o).is_some()).collect()
}

fn client(insecure: bool) -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        // alguns GitLabs internos usam certificado próprio do órgão
        .danger_accept_invalid_certs(insecure)
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())
}

fn base(url: &str) -> String {
    let u = url.trim().trim_end_matches('/');
    if u.starts_with("http") { u.to_string() } else { format!("https://{u}") }
}

async fn get(c: &reqwest::Client, url: &str, tok: &str) -> Result<Value, String> {
    let r = c.get(url).header("PRIVATE-TOKEN", tok).send().await.map_err(|e| {
        if e.is_connect() || e.is_timeout() {
            "Sem acesso ao GitLab — a VPN está ligada?".to_string()
        } else {
            e.to_string()
        }
    })?;
    let status = r.status();
    if status.as_u16() == 401 {
        return Err("Token inválido ou expirado".into());
    }
    if !status.is_success() {
        return Err(format!("GitLab respondeu {status}"));
    }
    r.json::<Value>().await.map_err(|e| e.to_string())
}

#[derive(Serialize)]
pub struct Mr {
    id: u64,
    iid: u64,
    title: String,
    project: String,
    url: String,
    author: String,
    updated: String,
    draft: bool,
    conflicts: bool,
    pipeline: String,
    merge_status: String,
}

fn to_mr(v: &Value) -> Mr {
    let s = |k: &str| v[k].as_str().unwrap_or("").to_string();
    let reference = v["references"]["full"].as_str().unwrap_or("");
    Mr {
        id: v["id"].as_u64().unwrap_or(0),
        iid: v["iid"].as_u64().unwrap_or(0),
        title: s("title"),
        project: reference.split('!').next().unwrap_or("").to_string(),
        url: s("web_url"),
        author: v["author"]["name"].as_str().unwrap_or("").to_string(),
        updated: s("updated_at"),
        draft: v["draft"].as_bool().or(v["work_in_progress"].as_bool()).unwrap_or(false),
        conflicts: v["has_conflicts"].as_bool().unwrap_or(false),
        pipeline: v["head_pipeline"]["status"].as_str().unwrap_or("").to_string(),
        merge_status: v["detailed_merge_status"].as_str().unwrap_or("").to_string(),
    }
}

#[derive(Serialize)]
pub struct MrList {
    user: String,
    review: Vec<Mr>,
    mine: Vec<Mr>,
}

/// Lista MRs onde você é revisor/responsável e os MRs que você abriu (com pipeline)
#[tauri::command]
pub async fn gitlab_mrs(org: String, url: String, insecure: bool) -> Result<MrList, String> {
    let tok = token(&org).ok_or("Cadastre o token do GitLab nas configurações do órgão")?;
    let c = client(insecure)?;
    let api = format!("{}/api/v4", base(&url));
    let me = get(&c, &format!("{api}/user"), &tok).await?;
    let uid = me["id"].as_u64().unwrap_or(0);
    let user = me["username"].as_str().unwrap_or("").to_string();

    let q = "state=opened&per_page=30&scope=all";
    let mut review: Vec<Mr> = Vec::new();
    for url in [format!("{api}/merge_requests?{q}&reviewer_id={uid}"), format!("{api}/merge_requests?state=opened&per_page=30&scope=assigned_to_me")] {
        if let Ok(Value::Array(list)) = get(&c, &url, &tok).await {
            for v in &list {
                let m = to_mr(v);
                if !review.iter().any(|r| r.id == m.id) && v["author"]["id"].as_u64() != Some(uid) {
                    review.push(m);
                }
            }
        }
    }

    let mut mine = Vec::new();
    if let Value::Array(list) = get(&c, &format!("{api}/merge_requests?state=opened&per_page=15&scope=created_by_me"), &tok).await? {
        for v in list.iter().take(10) {
            // o detalhe traz o pipeline e o status de merge
            let pid = v["project_id"].as_u64().unwrap_or(0);
            let iid = v["iid"].as_u64().unwrap_or(0);
            let detail = get(&c, &format!("{api}/projects/{pid}/merge_requests/{iid}"), &tok).await.unwrap_or_else(|_| v.clone());
            mine.push(to_mr(&detail));
        }
    }
    Ok(MrList { user, review, mine })
}
