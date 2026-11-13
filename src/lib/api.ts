// Ponte entre a interface (Vue) e o backend em Rust (Tauri).
// Fora do Tauri (ex.: `npm run dev:web` no navegador) usa um simulador com dados de exemplo.
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { listen as tauriListen } from '@tauri-apps/api/event'
import type {
  Anchor, AppData, Clip, Devices, MediaInfo, Proc, ScanResult, SysInfo, VolInfo
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
  pickPath: (kind: 'file' | 'folder') => invoke<string | null>('pick_path', { kind }),
  notify: (title: string, body: string) => invoke<void>('notify', { title, body }),
  relaunchAdmin: () => invoke<boolean>('relaunch_admin'),
  quit: () => invoke<void>('quit_app'),
  checkUpdate: () => invoke<{ available: boolean; version: string; current: string; notes: string }>('check_update'),
  installUpdate: () => invoke<void>('install_update'),

  // eventos vindos do Rust
  onHover: (cb: (inside: boolean) => void) => on('hover', cb),
  onToggle: (cb: () => void) => on('toggle', cb),
  onClips: (cb: (l: Clip[]) => void) => on('clips', cb),
  onMedia: (cb: (m: MediaInfo) => void) => on('media', cb),
  onVol: (cb: (v: VolInfo) => void) => on('vol', cb),
  onDevices: (cb: (d: Devices) => void) => on('devices', cb),
  onSys: (cb: (s: SysInfo) => void) => on('sys', cb),
  onAnchor: (cb: (a: Anchor) => void) => on('anchor', cb),
  onAutostart: (cb: (on: boolean) => void) => on('autostart', cb)
}
