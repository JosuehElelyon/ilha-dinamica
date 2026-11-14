//! Volume, microfone e dispositivos de áudio (Core Audio do Windows).
//! Tudo roda numa thread própria; os comandos chegam por um canal.

use serde::Serialize;
use std::sync::mpsc::{channel, Sender};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager, State};

pub enum Cmd {
    SetVol(u32),
    ToggleMute,
    SetMic(u32),
    ToggleMic,
    SetDevice(String),
    Refresh,
}

#[derive(Default)]
pub struct AudioState {
    tx: Mutex<Option<Sender<Cmd>>>,
}

#[derive(Clone, Serialize, PartialEq, Default)]
pub struct VolInfo {
    level: i32,
    mute: bool,
    mic: i32,
    #[serde(rename = "micMute")]
    mic_mute: bool,
}

#[derive(Clone, Serialize, PartialEq)]
pub struct Device {
    id: String,
    name: String,
    def: bool,
}

#[derive(Clone, Serialize, PartialEq, Default)]
pub struct Devices {
    out: Vec<Device>,
    #[serde(rename = "in")]
    inp: Vec<Device>,
}

#[cfg(windows)]
#[allow(non_snake_case)]
mod imp {
    use super::{Device, Devices, VolInfo};
    use windows::core::{interface, Interface, IUnknown, IUnknown_Vtbl, GUID, HRESULT, PCWSTR};
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
    use windows::Win32::Media::Audio::{
        eCapture, eCommunications, eConsole, eMultimedia, eRender, EDataFlow, ERole, IMMDevice,
        IMMDeviceEnumerator, MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
    };

    // Interface não documentada usada por apps como EarTrumpet para trocar o dispositivo padrão
    #[interface("f8679f50-850a-41cf-9c72-430f290290c8")]
    unsafe trait IPolicyConfig: IUnknown {
        fn GetMixFormat(&self) -> HRESULT;
        fn GetDeviceFormat(&self) -> HRESULT;
        fn ResetDeviceFormat(&self) -> HRESULT;
        fn SetDeviceFormat(&self) -> HRESULT;
        fn GetProcessingPeriod(&self) -> HRESULT;
        fn SetProcessingPeriod(&self) -> HRESULT;
        fn GetShareMode(&self) -> HRESULT;
        fn SetShareMode(&self) -> HRESULT;
        fn GetPropertyValue(&self) -> HRESULT;
        fn SetPropertyValue(&self) -> HRESULT;
        fn SetDefaultEndpoint(&self, id: PCWSTR, role: ERole) -> HRESULT;
    }
    const CLSID_POLICY_CONFIG: GUID = GUID::from_u128(0x870af99c_171d_4f9e_af0d_e63df40c2bc9);

    pub fn init_thread() {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
    }

    fn enumerator() -> windows::core::Result<IMMDeviceEnumerator> {
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
    }

    fn endpoint(flow: EDataFlow) -> windows::core::Result<IAudioEndpointVolume> {
        unsafe {
            let dev = enumerator()?.GetDefaultAudioEndpoint(flow, eConsole)?;
            dev.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
        }
    }

    fn read_level(flow: EDataFlow) -> (i32, bool) {
        match endpoint(flow) {
            Ok(v) => unsafe {
                let l = v.GetMasterVolumeLevelScalar().map(|x| (x * 100.0).round() as i32).unwrap_or(-1);
                let m = v.GetMute().map(|b| b.as_bool()).unwrap_or(false);
                (l, m)
            },
            Err(_) => (-1, false),
        }
    }

    pub fn vol_info() -> VolInfo {
        let (level, mute) = read_level(eRender);
        let (mic, mic_mute) = read_level(eCapture);
        VolInfo { level, mute, mic, mic_mute }
    }

    pub fn set_level(flow: EDataFlow, v: u32) {
        if let Ok(e) = endpoint(flow) {
            unsafe {
                let _ = e.SetMasterVolumeLevelScalar(v.min(100) as f32 / 100.0, std::ptr::null());
                if flow == eRender {
                    let _ = e.SetMute(false, std::ptr::null());
                }
            }
        }
    }

    pub fn toggle_mute(flow: EDataFlow) {
        if let Ok(e) = endpoint(flow) {
            unsafe {
                let m = e.GetMute().map(|b| b.as_bool()).unwrap_or(false);
                let _ = e.SetMute(!m, std::ptr::null());
            }
        }
    }

    fn device_id(dev: &IMMDevice) -> String {
        unsafe {
            match dev.GetId() {
                Ok(p) => {
                    let s = p.to_string().unwrap_or_default();
                    CoTaskMemFree(Some(p.0 as _));
                    s
                }
                Err(_) => String::new(),
            }
        }
    }

    fn device_name(dev: &IMMDevice) -> String {
        unsafe {
            dev.OpenPropertyStore(STGM_READ)
                .and_then(|s| s.GetValue(&PKEY_Device_FriendlyName))
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "Dispositivo".into())
        }
    }

    fn list(flow: EDataFlow) -> Vec<Device> {
        let mut out = Vec::new();
        let Ok(en) = enumerator() else { return out };
        unsafe {
            let def = en.GetDefaultAudioEndpoint(flow, eConsole).map(|d| device_id(&d)).unwrap_or_default();
            let Ok(col) = en.EnumAudioEndpoints(flow, DEVICE_STATE_ACTIVE) else { return out };
            let n = col.GetCount().unwrap_or(0);
            for i in 0..n {
                if let Ok(dev) = col.Item(i) {
                    let id = device_id(&dev);
                    out.push(Device { name: device_name(&dev), def: id == def, id });
                }
            }
        }
        out
    }

    pub fn devices() -> Devices {
        Devices { out: list(eRender), inp: list(eCapture) }
    }

    pub fn set_default(id: &str) -> windows::core::Result<()> {
        let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let pc: IUnknown = CoCreateInstance(&CLSID_POLICY_CONFIG, None, CLSCTX_ALL)?;
            let pc: IPolicyConfig = pc.cast()?;
            for role in [eConsole, eMultimedia, eCommunications] {
                pc.SetDefaultEndpoint(PCWSTR(wide.as_ptr()), role).ok()?;
            }
        }
        Ok(())
    }

    pub use windows::Win32::Media::Audio::{eCapture as CAPTURE, eRender as RENDER};
}

pub fn start(app: AppHandle) {
    let (tx, rx) = channel::<Cmd>();
    *app.state::<AudioState>().tx.lock().unwrap() = Some(tx);

    #[cfg(windows)]
    std::thread::spawn(move || {
        imp::init_thread();
        let mut last_vol: Option<VolInfo> = None;
        let mut last_dev: Option<Devices> = None;
        let mut tick = 0u32;
        loop {
            let mut force_dev = false;
            if let Ok(cmd) = rx.recv_timeout(Duration::from_millis(400)) {
                match cmd {
                    Cmd::SetVol(v) => imp::set_level(imp::RENDER, v),
                    Cmd::ToggleMute => imp::toggle_mute(imp::RENDER),
                    Cmd::SetMic(v) => imp::set_level(imp::CAPTURE, v),
                    Cmd::ToggleMic => imp::toggle_mute(imp::CAPTURE),
                    Cmd::SetDevice(id) => {
                        let _ = imp::set_default(&id);
                        force_dev = true;
                    }
                    Cmd::Refresh => {
                        force_dev = true;
                        last_vol = None;
                    }
                }
                // junta vários comandos seguidos (ex.: arrastar o slider)
                while let Ok(c) = rx.try_recv() {
                    match c {
                        Cmd::SetVol(v) => imp::set_level(imp::RENDER, v),
                        Cmd::SetMic(v) => imp::set_level(imp::CAPTURE, v),
                        Cmd::ToggleMute => imp::toggle_mute(imp::RENDER),
                        Cmd::ToggleMic => imp::toggle_mute(imp::CAPTURE),
                        Cmd::SetDevice(id) => {
                            let _ = imp::set_default(&id);
                            force_dev = true;
                        }
                        Cmd::Refresh => force_dev = true,
                    }
                }
            }
            let v = imp::vol_info();
            if last_vol.as_ref() != Some(&v) {
                let _ = app.emit("vol", v.clone());
                last_vol = Some(v);
            }
            tick += 1;
            if force_dev || tick % 8 == 0 {
                let d = imp::devices();
                if force_dev || last_dev.as_ref() != Some(&d) {
                    let _ = app.emit("devices", d.clone());
                    last_dev = Some(d);
                }
            }
        }
    });
    #[cfg(not(windows))]
    std::thread::spawn(move || {
        let _ = (rx, app.emit("devices", Devices::default()));
    });
}

fn send(state: &State<AudioState>, cmd: Cmd) {
    if let Some(tx) = state.tx.lock().unwrap().as_ref() {
        let _ = tx.send(cmd);
    }
}

#[tauri::command]
pub fn set_vol(state: State<AudioState>, value: u32) {
    send(&state, Cmd::SetVol(value));
}
#[tauri::command]
pub fn toggle_mute(state: State<AudioState>) {
    send(&state, Cmd::ToggleMute);
}
#[tauri::command]
pub fn set_mic(state: State<AudioState>, value: u32) {
    send(&state, Cmd::SetMic(value));
}
#[tauri::command]
pub fn toggle_mic(state: State<AudioState>) {
    send(&state, Cmd::ToggleMic);
}
#[tauri::command]
pub fn set_device(state: State<AudioState>, id: String) {
    send(&state, Cmd::SetDevice(id));
}
#[tauri::command]
pub fn refresh_devices(state: State<AudioState>) {
    send(&state, Cmd::Refresh);
}

pub fn toggle_mute_app(app: &AppHandle) {
    let state = app.state::<AudioState>();
    send(&state, Cmd::ToggleMute);
}
