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

export interface Adapter { name: string; desc: string; up: boolean; ip: string }

export type VpnType = 'windows' | 'openvpnconnect' | 'openvpn' | 'wireguard' | 'cisco' | 'forti' | 'globalprotect' | 'custom' | 'none'
export interface OrgRepo { name: string; path: string }
export interface RepoGitStatus { path: string; is_git: boolean; branch: string; dirty: number }
export interface Org {
  id: string
  name: string
  color: string
  vpn: { type: VpnType; name: string; user: string; adapter: string; connect: string; disconnect: string }
  gitlab: { url: string; insecure: boolean }
  repos?: OrgRepo[]
}
export interface Meeting {
  id: string
  date: string
  startTime: string
  endTime: string
  startMs: number
  endMs: number
  summary: string
  joinUrl: string
  allDay: boolean
}
export interface Mr {
  id: number; iid: number; title: string; project: string; url: string; author: string
  updated: string; draft: boolean; conflicts: boolean; pipeline: string; merge_status: string
}
export interface MrList { user: string; review: Mr[]; mine: Mr[] }
export interface VpnResult { ok: boolean; mode: 'cmd' | 'app'; msg: string }

export interface Game { id: string; name: string; source: 'steam' | 'epic' | 'gog'; launch: string; image: string; appid: number; last_played: number; size: number }
export interface SteamInfo { installed: boolean; steamid: string; persona: string; has_key: boolean }
export interface Friend { steamid: string; name: string; avatar: string; state: number; game: string; gameid: string; lobby: string }
export interface WifiStatus { connected: boolean; ssid: string; signal: number }
export interface WifiNet { ssid: string; signal: number; auth: string; connected: boolean }
export interface BtDevice { id: string; name: string; connected: boolean; kind: string }
export interface BtState { available: boolean; enabled: boolean; devices: BtDevice[] }

export interface Anchor { x: 'left' | 'center' | 'right'; y: 'top' | 'bottom' }

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
  anchor?: Anchor
}

export type ProviderKind = 'google' | 'windows' | 'github' | 'gitlab' | 'apple' | 'local'
export type PhoneTransport = 'usb' | 'lan' | 'bluetooth'
export type AvatarMood = 'idle' | 'celebrate' | 'alert' | 'meeting' | 'briefing'

export interface AccountRecord {
  id: string
  provider: ProviderKind
  label: string
  email: string
  isPrimary: boolean
  status: string
  lastSyncTs: number
  details: string
}

export interface CanonicalEvent {
  id: string
  fingerprint: string
  title: string
  category: string
  date: string
  startTime: string
  endTime: string
  startTs: number
  endTs: number
  allDay: boolean
  joinUrl: string
  sourceProvider: ProviderKind
  accountId: string
}

export interface UpcomingDate {
  id: string
  title: string
  personName: string
  kind: 'birthday' | 'anniversary' | 'holiday' | 'custom' | string
  month: number
  day: number
  daysUntil: number
  turningAge: number | null
  sourceProvider: ProviderKind
}

export interface PhoneDevice {
  id: string
  name: string
  platform: string
  connected: boolean
  activeTransport: PhoneTransport
  batteryPct: number | null
  charging: boolean
  lastSeenTs: number
}

export interface PhoneSignalRecord {
  id: string
  deviceId: string
  kind: 'notification' | 'call' | 'battery' | 'clipboard' | string
  title: string
  body: string
  appName: string
  ts: number
}

export interface ProactiveAlert {
  key: string
  level: 'urgent' | 'celebrate' | 'normal' | string
  avatarMood: AvatarMood
  icon: string
  title: string
  body: string
  bubbleText: string
  actionLabel: string
  actionTarget: string
  createdTs: number
}

export interface DevWorkItem {
  id: string
  provider: ProviderKind
  repo: string
  title: string
  kind: string
  url: string
  updatedAt: string
}

export interface PairingInfo {
  qrUri: string
  host: string
  port: number
  token: string
  adbAvailable: boolean
  adbDevices: string[]
  btPairedPhones: string[]
}

export interface AssistantSnapshot {
  googleConnected: boolean
  accounts: AccountRecord[]
  upcomingDates: UpcomingDate[]
  todayEvents: CanonicalEvent[]
  allEvents: CanonicalEvent[]
  peopleCount: number
  phones: PhoneDevice[]
  recentSignals: PhoneSignalRecord[]
  alerts: ProactiveAlert[]
  devItems: DevWorkItem[]
}

