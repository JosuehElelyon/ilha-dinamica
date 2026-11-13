//! Log simples em %LOCALAPPDATA%\com.josueh.ilhadinamica\ilha.log
//! (ajuda a descobrir por que o app não abriu, já que ele não tem console).

use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("com.josueh.ilhadinamica").join("ilha.log")
}

pub fn log(msg: impl AsRef<str>) {
    let p = path();
    if let Some(dir) = p.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    // mantém o arquivo pequeno
    if std::fs::metadata(&p).map(|m| m.len() > 256 * 1024).unwrap_or(false) {
        let _ = std::fs::remove_file(&p);
    }
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&p) {
        let _ = writeln!(f, "[{ts}] {}", msg.as_ref());
    }
}

/// Registra qualquer pânico no log antes de o app fechar
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log(format!("PÂNICO: {info}"));
        default(info);
    }));
}
