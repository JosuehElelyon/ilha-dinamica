// Ponte entre a interface (Vue) e o backend em Rust (Tauri).
// Fora do Tauri (ex.: `npm run dev:web` no navegador) usa um simulador com dados de exemplo.
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen } from '@tauri-apps/api/event'
import type {
  Adapter, Anchor, AssistantSnapshot, BtState, Friend, Game, PairingInfo, PhoneSignalRecord, ProactiveAlert,
  SteamInfo, AppData, Clip, MrList, Org, RepoGitStatus, VpnResult, Devices, MediaInfo, Proc, ScanResult,
  SysInfo, VolInfo, WifiNet, WifiStatus
} from './types'
import { mock } from './mock'

export const inTauri = '__TAURI_INTERNALS__' in window
// no navegador, deixa o simulador acessível pelo console (ex.: __ilhaMock.emit('anchor', {x:'left', y:'bottom'}))
if (!inTauri) (window as unknown as { __ilhaMock: typeof mock }).__ilhaMock = mock

function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return inTauri ? tauriInvoke<T>(cmd, args) : mock.invoke<T>(cmd, args)
}

function on<T>(event: string, cb: (payload: T) => void) {
  if (inTauri) tauriListen<T>(event, e => cb(e.payload))
  else mock.listen(event, cb as (p: unknown) => void)
}

export const api = {
  // janela / mouse
  setHitRect: (x: number, y: number, w: number, h: number) => invoke<void>('set_hit_rect', { x, y, w, h }),
  setInteractive: (value: boolean) => invoke<void>('set_interactive', { value }),
  startDrag: () => invoke<void>('start_drag'),
  resetPosition: () => invoke<void>('reset_position'),

  // dados
  getData: () => invoke<AppData>('get_data'),
  saveNotes: (text: string) => invoke<void>('save_notes', { text }),
  saveReminders: (reminders: unknown) => invoke<void>('save_reminders', { reminders }),
  saveSetting: (key: string, value: unknown) => invoke<void>('save_setting', { key, value }),
  setAutostart: (enabled: boolean) => invoke<boolean>('set_autostart', { enabled }),

  // clipboard
  copy: (text: string) => invoke<void>('copy_text', { text }),
  clearClips: () => invoke<void>('clear_clips'),

  // mídia e áudio
  mediaCmd: (cmd: 'toggle' | 'next' | 'prev') => invoke<void>('media_cmd', { cmd }),
  setVol: (value: number) => invoke<void>('set_vol', { value }),
  toggleMute: () => invoke<void>('toggle_mute'),
  setMic: (value: number) => invoke<void>('set_mic', { value }),
  toggleMic: () => invoke<void>('toggle_mic'),
  setDevice: (id: string) => invoke<void>('set_device', { id }),
  refreshDevices: () => invoke<void>('refresh_devices'),

  // sistema
  getBrightness: () => invoke<number | null>('get_brightness'),
  setBrightness: (value: number) => invoke<boolean>('set_brightness', { value }),
  getProcs: () => invoke<Proc[]>('get_procs'),
  kill: (pid: number | null, name: string) => invoke<{ ok: boolean; protected: boolean }>('kill_process', { pid, name }),

  // limpeza
  cleanScan: () => invoke<ScanResult>('clean_scan'),
  cleanRun: (keys: string[]) => invoke<{ freed: number; disk: ScanResult['disk'] }>('clean_run', { keys }),
  trimRam: () => invoke<{ freed: number; n: number }>('trim_ram'),
  flushDns: () => invoke<boolean>('flush_dns'),

  // ações e app
  runAction: (target: string) => invoke<boolean>('run_action', { target }),
  openInAntigravity: (path: string) => invoke<boolean>('open_in_antigravity', { path }),
  openTerminal: (path: string) => invoke<boolean>('open_terminal', { path }),
  gitStatus: (paths: string[]) => invoke<RepoGitStatus[]>('git_status', { paths }),
  pickPath: (kind: 'file' | 'folder') => invoke<string | null>('pick_path', { kind }),
  fetchIcs: (url: string) => invoke<string>('fetch_ics', { url }),
  notify: (title: string, body: string) => invoke<void>('notify', { title, body }),
  relaunchAdmin: () => invoke<boolean>('relaunch_admin'),
  quit: () => invoke<void>('quit_app'),
  checkUpdate: () => invoke<{ available: boolean; version: string; current: string; notes: string }>('check_update'),
  installUpdate: () => invoke<void>('install_update'),

  // trabalho: VPNs e GitLab
  listAdapters: () => invoke<Adapter[]>('list_adapters'),
  vpnConnect: (org: string, vpn: Org['vpn']) => invoke<VpnResult>('vpn_connect', { profile: { ...vpn, org } }),
  vpnDisconnect: (org: string, vpn: Org['vpn']) => invoke<VpnResult>('vpn_disconnect', { profile: { ...vpn, org } }),
  vpnSetPassword: (org: string, password: string) => invoke<boolean>('vpn_set_password', { org, password }),
  vpnHasPasswords: (orgs: string[]) => invoke<string[]>('vpn_has_passwords', { orgs }),
  probeHost: (host: string) => invoke<{ ok: boolean; ms: number }>('probe_host', { host }),
  gitlabSetToken: (org: string, token: string) => invoke<boolean>('gitlab_set_token', { org, token }),
  gitlabHasTokens: (orgs: string[]) => invoke<string[]>('gitlab_has_tokens', { orgs }),
  gitlabMrs: (org: string, url: string, insecure: boolean) => invoke<MrList>('gitlab_mrs', { org, url, insecure }),

  // jogos
  gamesList: () => invoke<Game[]>('games_list'),
  steamInfo: () => invoke<SteamInfo>('steam_info'),
  steamSetKey: (key: string) => invoke<boolean>('steam_set_key', { key }),
  steamFriends: (steamid: string) => invoke<Friend[]>('steam_friends', { steamid }),

  // redes (Wi-Fi e Bluetooth)
  wifiStatus: () => invoke<WifiStatus>('wifi_status'),
  wifiScan: () => invoke<WifiNet[]>('wifi_scan'),
  wifiConnect: (ssid: string) => invoke<string>('wifi_connect', { ssid }),
  wifiDisconnect: () => invoke<boolean>('wifi_disconnect'),
  btState: () => invoke<BtState>('bt_state'),
  btToggleRadio: (enable: boolean) => invoke<boolean>('bt_toggle_radio', { enable }),

  // assistente pessoal & ponte com celular
  assistantSnapshot: () => invoke<AssistantSnapshot>('assistant_snapshot'),
  assistantGoogleConnect: (clientId?: string, clientSecret?: string) =>
    invoke<AssistantSnapshot>('assistant_google_connect', { clientId: clientId || null, clientSecret: clientSecret || null }),
  assistantGoogleSync: () => invoke<AssistantSnapshot>('assistant_google_sync'),
  assistantGoogleDisconnect: () => invoke<AssistantSnapshot>('assistant_google_disconnect'),
  assistantSyncWindows: () => invoke<AssistantSnapshot>('assistant_sync_windows'),
  assistantGithubConnect: (token: string) => invoke<AssistantSnapshot>('assistant_github_connect', { token }),
  assistantGithubDeviceStart: (clientId: string) =>
    invoke<import('./types').GithubDeviceChallenge>('assistant_github_device_start', { clientId }),
  assistantGithubDevicePoll: (clientId: string, deviceCode: string) =>
    invoke<AssistantSnapshot | null>('assistant_github_device_poll', { clientId, deviceCode }),
  assistantAppleConnect: (appleId: string, appPassword: string) =>
    invoke<AssistantSnapshot>('assistant_apple_connect', { appleId, appPassword }),
  assistantDisconnectAccount: (accountId: string) => invoke<AssistantSnapshot>('assistant_disconnect_account', { accountId }),
  assistantAddImportantDate: (name: string, kind: string, month: number, day: number, yearOptional?: number | null) =>
    invoke<AssistantSnapshot>('assistant_add_important_date', { name, kind, month, day, yearOptional: yearOptional ?? null }),
  assistantDeleteImportantDate: (id: string) => invoke<AssistantSnapshot>('assistant_delete_important_date', { id }),
  phonePairingInfo: () => invoke<PairingInfo>('phone_pairing_info'),
  phoneRotateToken: () => invoke<PairingInfo>('phone_rotate_token'),
  phoneEnableUsbBridge: () => invoke<string[]>('phone_enable_usb_bridge'),
  phoneIngestDemoSignal: (kind: string, title: string, body: string, transport: string) =>
    invoke<AssistantSnapshot>('phone_ingest_demo_signal', { kind, title, body, transport }),
  phoneSendClipboard: () => invoke<string>('phone_send_clipboard_to_phone'),
  phoneRing: (message?: string) => invoke<void>('phone_ring_phone', { message: message ?? null }),
  phoneSetAutoPauseOnCall: (enabled: boolean) => invoke<boolean>('phone_set_auto_pause_on_call', { enabled }),

  // eventos vindos do Rust
  onHover: (cb: (inside: boolean) => void) => on('hover', cb),
  onToggle: (cb: () => void) => on('toggle', cb),
  onPalette: (cb: () => void) => on('palette', cb),
  onClips: (cb: (l: Clip[]) => void) => on('clips', cb),
  onMedia: (cb: (m: MediaInfo) => void) => on('media', cb),
  onVol: (cb: (v: VolInfo) => void) => on('vol', cb),
  onDevices: (cb: (d: Devices) => void) => on('devices', cb),
  onSys: (cb: (s: SysInfo) => void) => on('sys', cb),
  onAnchor: (cb: (a: Anchor) => void) => on('anchor', cb),
  onAdapters: (cb: (a: Adapter[]) => void) => on('adapters', cb),
  onAutostart: (cb: (on: boolean) => void) => on('autostart', cb),
  onAssistantUpdate: (cb: (s: AssistantSnapshot) => void) => on('assistant-update', cb),
  onAssistantAlert: (cb: (a: ProactiveAlert) => void) => on('assistant-alert', cb),
  onPhoneSignal: (cb: (s: PhoneSignalRecord) => void) => on('phone-signal', cb)
}
