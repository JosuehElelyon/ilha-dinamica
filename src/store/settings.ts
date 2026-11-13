import { reactive } from 'vue'
import { api } from '../lib/api'
import type { Action, Org } from '../lib/types'
import { DEFAULT_SNIPPETS, type Snippet } from '../lib/snippets'

export type ModuleId =
  | 'home' | 'work' | 'games' | 'actions' | 'tools' | 'snippets' | 'clips' | 'notes' | 'timer' | 'cal'
  | 'weather' | 'audio' | 'net' | 'sys' | 'clean' | 'procs'

export type HomeCardId = 'stopwatch' | 'media' | 'focus' | 'timer' | 'weather' | 'meeting'

export const HOME_CARDS: Record<HomeCardId, { label: string; icon: string }> = {
  stopwatch: { label: 'Cronômetro', icon: '⏱️' },
  media: { label: 'Música', icon: '🎵' },
  focus: { label: 'Foco', icon: '🎯' },
  timer: { label: 'Timer', icon: '⏳' },
  weather: { label: 'Clima', icon: '⛅' },
  meeting: { label: 'Reunião', icon: '📹' }
}

export const DEFAULT_HOME_CARDS: HomeCardId[] = ['stopwatch', 'media', 'focus']

export const MODULES: Record<ModuleId, string> = {
  home: 'INÍCIO', work: 'TRABALHO', games: 'JOGOS', actions: 'AÇÕES', tools: 'FERRAMENTAS', snippets: 'SNIPPETS', clips: 'CLIPS', notes: 'NOTAS', timer: 'TIMER',
  cal: 'AGENDA', weather: 'CLIMA', audio: 'ÁUDIO', net: 'REDES', sys: 'SISTEMA', clean: 'LIMPEZA', procs: 'PROCESSOS'
}

export const ACCENTS = ['#ff8a2b', '#ff4d6d', '#ffd23f', '#34d058', '#2fb8ff', '#8b7bff', '#f0f0f0']

export const DEFAULT_ACTIONS: Action[] = [
  { id: 1, emoji: '🌐', name: 'Google', type: 'url', target: 'https://www.google.com' },
  { id: 2, emoji: '▶️', name: 'YouTube', type: 'url', target: 'https://www.youtube.com' },
  { id: 3, emoji: '📁', name: 'Downloads', type: 'folder', target: '%USERPROFILE%\\Downloads' },
  { id: 4, emoji: '🧮', name: 'Calculadora', type: 'cmd', target: 'calc' },
  { id: 5, emoji: '⚙️', name: 'Config.', type: 'cmd', target: 'ms-settings:' }
]

export function newOrg(id: string, name: string, color: string, vpn: Partial<Org['vpn']> = {}): Org {
  return {
    id, name, color,
    vpn: { type: 'none', name: '', user: '', adapter: '', connect: '', disconnect: '', ...vpn },
    gitlab: { url: '', insecure: false }
  }
}

export interface Settings {
  accent: string
  followMusic: boolean
  size: 'compacta' | 'normal' | 'grande'
  opacity: number
  city: { name: string; lat: number; lon: number } | null
  alerts: Record<'vol' | 'bat' | 'net' | 'track' | 'dev', boolean>
  modules: ModuleId[]
  modulesOff: ModuleId[]
  actions: Action[]
  cleanSel: string[]
  procSort: 'c' | 'm'
  locked: boolean
  autoUpdate: boolean
  snippets: Snippet[]
  orgs: Org[]
  vpnExclusive: boolean
  steamId: string
  friendAlerts: boolean
  icsUrl: string
  homeCards: HomeCardId[]
  focusMinutes: number
}

const DEFAULTS: Settings = {
  accent: '#ff8a2b',
  followMusic: true,
  size: 'normal',
  opacity: 94,
  city: null,
  alerts: { vol: true, bat: true, net: true, track: true, dev: true },
  modules: Object.keys(MODULES) as ModuleId[],
  modulesOff: [],
  actions: DEFAULT_ACTIONS,
  cleanSel: ['temp', 'browsers', 'apps', 'thumbs', 'crash', 'wintemp', 'update'],
  procSort: 'm',
  locked: false,
  autoUpdate: true,
  snippets: DEFAULT_SNIPPETS,
  orgs: [],
  vpnExclusive: false,
  steamId: '',
  friendAlerts: true,
  icsUrl: '',
  homeCards: [...DEFAULT_HOME_CARDS],
  focusMinutes: 25
}

export const S = reactive<Settings>(structuredClone(DEFAULTS))

/** Altera e salva uma configuração */
export function save<K extends keyof Settings>(key: K, value: Settings[K]) {
  S[key] = value
  api.saveSetting(key, JSON.parse(JSON.stringify(value)))
}

/** Mescla as configurações salvas com os padrões */
export function loadSettings(saved: Record<string, unknown>) {
  Object.assign(S, structuredClone(DEFAULTS), saved)
  S.alerts = { ...DEFAULTS.alerts, ...((saved.alerts as object) || {}) }
  const known = Object.keys(MODULES) as ModuleId[]
  const mods = (Array.isArray(saved.modules) ? saved.modules : []).filter((m): m is ModuleId => known.includes(m))
  // módulos novos entram logo depois do módulo que vem antes deles na ordem padrão
  const merged = [...mods]
  known.forEach((m, i) => {
    if (merged.includes(m)) return
    const prev = known.slice(0, i).reverse().find(p => merged.includes(p))
    merged.splice(prev ? merged.indexOf(prev) + 1 : 0, 0, m)
  })
  S.modules = merged
  if (!Array.isArray(S.actions)) S.actions = DEFAULT_ACTIONS
  if (!Array.isArray(S.cleanSel)) S.cleanSel = DEFAULTS.cleanSel
  if (!Array.isArray(S.snippets)) S.snippets = DEFAULT_SNIPPETS
  if (!Array.isArray(S.orgs)) S.orgs = structuredClone(DEFAULTS.orgs)
  S.orgs = S.orgs.map(o => ({ ...o, repos: Array.isArray(o.repos) ? o.repos : [] }))
  const knownCards = Object.keys(HOME_CARDS) as HomeCardId[]
  const cards = (Array.isArray(saved.homeCards) ? saved.homeCards : DEFAULT_HOME_CARDS).filter((c): c is HomeCardId => knownCards.includes(c))
  S.homeCards = cards.length ? cards : [...DEFAULT_HOME_CARDS]
  S.focusMinutes = Math.min(180, Math.max(1, Number(S.focusMinutes) || 25))
}

export const activeModules = () => S.modules.filter(m => !S.modulesOff.includes(m))
