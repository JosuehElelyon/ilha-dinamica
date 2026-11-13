export interface Clip { text: string; t: number }

export interface MediaInfo { title: string; artist: string; status: string; app: string; cover: string }

export interface VolInfo { level: number; mute: boolean; mic: number; micMute: boolean }

export interface Device { id: string; name: string; def: boolean }
export interface Devices { out: Device[]; in: Device[] }

export interface SysInfo {
  cpu: number
  memUsed: number
  memTotal: number
  uptime: number
  cpuName: string
  battery: { level: number; charging: boolean } | null
}

export interface Proc { n: string; p: number; c: number; m: number }

export interface CleanCat { name: string; icon: string; admin: boolean; size: number; count: number; denied: boolean }
export interface Disk { free: number; total: number }
export interface ScanResult { cats: Record<string, CleanCat>; order: string[]; disk: Disk | null }

export interface Reminder { id: number; date: string; time: string; text: string; fired: boolean }

export interface Action { id: number; emoji: string; name: string; type: 'url' | 'file' | 'folder' | 'cmd'; target: string }

export interface AppData {
  notes: string
  settings: Record<string, unknown>
  reminders: Reminder[]
  platform: string
  isAdmin: boolean
  autostart: boolean
  protectedNames: string[]
  selfName: string
  ownPids: number[]
  version: string
}
