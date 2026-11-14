//! Música/vídeo tocando agora (Spotify, YouTube no navegador...) pela API de
//! controle de mídia do Windows, incluindo a capa do álbum.

use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize, PartialEq, Default)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub status: String,
    pub app: String,
    pub cover: String,
}

#[cfg(windows)]
mod imp {
    use super::MediaInfo;
    use base64::Engine;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionMediaProperties as Props,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    };
    use windows::Storage::Streams::DataReader;

    pub fn init_thread() {
        unsafe {
            let _ = windows::Win32::System::WinRT::RoInitialize(
                windows::Win32::System::WinRT::RO_INIT_MULTITHREADED,
            );
        }
    }

    pub fn manager() -> windows::core::Result<Manager> {
        Manager::RequestAsync()?.get()
    }

    fn status_name(s: Status) -> &'static str {
        match s {
            Status::Playing => "Playing",
            Status::Paused => "Paused",
            Status::Stopped => "Stopped",
            Status::Changing => "Changing",
            Status::Opened => "Opened",
            _ => "Closed",
        }
    }

    pub fn cover(props: &Props) -> windows::core::Result<String> {
        let stream = props.Thumbnail()?.OpenReadAsync()?.get()?;
        let size = stream.Size()? as u32;
        if size == 0 {
            return Ok(String::new());
        }
        let reader = DataReader::CreateDataReader(&stream.GetInputStreamAt(0)?)?;
        reader.LoadAsync(size)?.get()?;
        let mut buf = vec![0u8; size as usize];
        reader.ReadBytes(&mut buf)?;
        let mut ct = stream.ContentType()?.to_string();
        if ct.is_empty() {
            ct = "image/png".into();
        }
        Ok(format!("data:{};base64,{}", ct, base64::engine::general_purpose::STANDARD.encode(buf)))
    }

    /// Lê a sessão atual. `want_cover` evita reler a imagem a cada segundo.
    pub fn read(mgr: &Manager, want_cover: bool) -> (MediaInfo, Option<Props>) {
        let Ok(session) = mgr.GetCurrentSession() else {
            return (MediaInfo { status: "None".into(), ..Default::default() }, None);
        };
        let props = session.TryGetMediaPropertiesAsync().and_then(|op| op.get()).ok();
        let status = session
            .GetPlaybackInfo()
            .and_then(|i| i.PlaybackStatus())
            .map(status_name)
            .unwrap_or("None");
        let mut info = MediaInfo {
            status: status.into(),
            app: session.SourceAppUserModelId().map(|s| s.to_string()).unwrap_or_default(),
            ..Default::default()
        };
        if let Some(p) = &props {
            info.title = p.Title().map(|s| s.to_string()).unwrap_or_default();
            info.artist = p.Artist().map(|s| s.to_string()).unwrap_or_default();
            if want_cover {
                info.cover = cover(p).unwrap_or_default();
            }
        }
        (info, props)
    }

    pub fn command(cmd: &str) -> windows::core::Result<()> {
        init_thread();
        let session = manager()?.GetCurrentSession()?;
        match cmd {
            "toggle" => { session.TryTogglePlayPauseAsync()?.get()?; }
            "next" => { session.TrySkipNextAsync()?.get()?; }
            "prev" => { session.TrySkipPreviousAsync()?.get()?; }
            _ => {}
        }
        Ok(())
    }
}

pub fn start(app: AppHandle) {
    #[cfg(windows)]
    std::thread::spawn(move || {
        imp::init_thread();
        let mut mgr = None;
        let mut last = MediaInfo::default();
        let mut key = String::new();
        let mut cover = String::new();
        let mut tries = 0;
        loop {
            if mgr.is_none() {
                mgr = imp::manager().ok();
            }
            if let Some(m) = &mgr {
                let (mut info, props) = imp::read(m, false);
                let k = format!("{}|{}", info.title, info.artist);
                if k != key {
                    key = k;
                    cover.clear();
                    tries = 0;
                }
                // a capa às vezes chega um pouco depois da troca de faixa
                if cover.is_empty() && tries < 4 {
                    tries += 1;
                    if let Some(p) = &props {
                        cover = imp::cover(p).unwrap_or_default();
                    }
                }
                info.cover = cover.clone();
                if info != last {
                    last = info.clone();
                    let _ = app.emit("media", info);
                }
            }
            std::thread::sleep(Duration::from_millis(1000));
        }
    });
    #[cfg(not(windows))]
    let _ = app;
}

#[tauri::command(async)]
pub fn media_cmd(cmd: String) {
    #[cfg(windows)]
    let _ = imp::command(&cmd);
    #[cfg(not(windows))]
    let _ = cmd;
}

pub fn pause_if_playing() {
    #[cfg(windows)]
    {
        imp::init_thread();
        if let Ok(mgr) = imp::manager() {
            if let Ok(session) = mgr.GetCurrentSession() {
                if let Ok(info) = session.GetPlaybackInfo() {
                    if info.PlaybackStatus().ok()
                        == Some(windows::Media::Control::GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
                    {
                        let _ = session.TryPauseAsync().and_then(|op| op.get());
                    }
                }
            }
        }
    }
}
