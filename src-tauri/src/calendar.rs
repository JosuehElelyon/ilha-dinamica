//! Busca calendário ICS (Outlook / Teams) sem problemas de CORS no WebView.

use std::time::Duration;

#[tauri::command]
pub async fn fetch_ics(url: String) -> Result<String, String> {
    let mut u = url.trim().to_string();
    if u.starts_with("webcal://") {
        u = format!("https://{}", &u["webcal://".len()..]);
    }
    if !u.starts_with("https://") && !u.starts_with("http://") {
        return Err("URL do calendário deve começar com https:// ou webcal://".into());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("ilha-dinamica")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&u).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {} ao buscar calendário", resp.status().as_u16()));
    }
    let text = resp.text().await.map_err(|e| e.to_string())?;
    if !text.contains("BEGIN:VCALENDAR") {
        return Err("O link não retornou um arquivo .ics válido".into());
    }
    Ok(text)
}
