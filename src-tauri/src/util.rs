use std::process::{Command, Output};

/// Cria um comando que não abre janela de console no Windows.
pub fn hidden(program: &str) -> Command {
    #[allow(unused_mut)]
    let mut c = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        c.creation_flags(CREATE_NO_WINDOW);
    }
    c
}

/// Executa um comando PowerShell curto e devolve a saída.
pub fn powershell(script: &str) -> std::io::Result<Output> {
    hidden("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script])
        .output()
}

/// O app está rodando como administrador?
pub fn is_admin() -> bool {
    #[cfg(windows)]
    {
        hidden("net").arg("session").output().map(|o| o.status.success()).unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        false
    }
}
