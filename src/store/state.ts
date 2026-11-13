import { reactive, ref } from 'vue'
import { api } from '../lib/api'
import type { Anchor, Clip, Devices, MediaInfo, Reminder, SysInfo, VolInfo } from '../lib/types'
import type { ModuleId } from './settings'

export type TabId = ModuleId | 'modules' | 'settings'

export interface Toast {
  icon?: string
  img?: string
  text: string
  progress?: number
  alert?: boolean
  ms?: number
}

/** Estado geral da ilha */
export const island = reactive({
  expanded: false,
  pinned: false,
  hovering: false,
  tab: 'home' as TabId,
  menuOpen: false,
  toast: null as Toast | null,
  // vindos do backend
  isAdmin: false,
  autostart: false,
  platform: 'windows',
  version: '',
  protectedNames: new Set<string>(),
  selfName: '',
  ownPids: [] as number[],
  anchor: { x: 'center', y: 'top' } as Anchor,
  update: { checking: false, installing: false, available: '', status: '' }
})

/* ---------- caixa de ferramentas (compartilhada com o Clips) ---------- */
export const toolbox = reactive({ input: '', tool: 'json' as import('../lib/devtools').ToolId })
export function openTool(tool: import('../lib/devtools').ToolId, input: string) {
  toolbox.tool = tool
  toolbox.input = input
  island.tab = 'tools'
}

/* ---------- atualização automática ---------- */
export async function checkUpdate(manual = false) {
  if (island.update.checking || island.update.installing) return
  island.update.checking = true
  island.update.status = 'Verificando…'
  try {
    const u = await api.checkUpdate()
    island.update.available = u.available ? u.version : ''
    island.update.status = u.available ? `Nova versão ${u.version} disponível` : 'Você está na versão mais recente'
    if (u.available) {
      if (!manual && autoUpdateEnabled()) return installUpdate()
      showToast({ icon: '⬆️', text: `Atualização ${u.version} disponível`, ms: 5000 })
    } else if (manual) showToast({ icon: '✅', text: 'Ilha Dinâmica atualizada', ms: 2500 })
  } catch {
    island.update.status = 'Não foi possível verificar agora'
    if (manual) showToast({ icon: '⚠️', text: 'Não foi possível verificar atualizações', ms: 3000 })
  } finally {
    island.update.checking = false
  }
}
export async function installUpdate() {
  island.update.installing = true
  island.update.status = `Instalando ${island.update.available}…`
  showToast({ icon: '⬆️', text: `Atualizando para ${island.update.available}…`, ms: 15000 })
  try {
    await api.installUpdate() // o app reinicia sozinho ao terminar
  } catch {
    island.update.installing = false
    island.update.status = 'Falha ao instalar — tente de novo'
    showToast({ icon: '⚠️', text: 'Falha ao atualizar', alert: true, ms: 4000 })
  }
}
let autoUpdateEnabled = () => true
export function setAutoUpdateGetter(fn: () => boolean) { autoUpdateEnabled = fn }

/** Relógio compartilhado (atualiza 4x por segundo) */
export const now = ref(Date.now())
setInterval(() => (now.value = Date.now()), 250)

let toastTimer: number | undefined
export function showToast(t: Toast) {
  island.toast = t
  clearTimeout(toastTimer)
  toastTimer = window.setTimeout(() => (island.toast = null), t.ms ?? 2500)
}

/** Aviso importante: notificação do Windows + ilha piscando */
export function alertIsland(title: string, body = '', icon = '🔔') {
  api.notify(title, body)
  showToast({ icon, text: body ? `${title} · ${body}` : title, alert: true, ms: 6000 })
}

/* ---------- dados ao vivo vindos do Rust ---------- */
export const live = reactive({
  clips: [] as Clip[],
  media: { title: '', artist: '', status: 'None', app: '', cover: '' } as MediaInfo,
  coverHue: null as number | null,
  vol: null as VolInfo | null,
  devices: null as Devices | null,
  sys: null as SysInfo | null,
  tempText: ''
})

/* ---------- cronômetro, foco (pomodoro) e timer ---------- */
export const FOCO = 25 * 60
export const PAUSA = 5 * 60

export const stopwatch = reactive({ running: false, start: 0, acc: 0 })
export const swElapsed = () => (stopwatch.acc + (stopwatch.running ? now.value - stopwatch.start : 0)) / 1000

export const focus = reactive({ mode: 'focus' as 'focus' | 'break', left: FOCO, running: false, endAt: 0 })
export const focusLeft = () => (focus.running ? (focus.endAt - now.value) / 1000 : focus.left)

export const timer = reactive({ total: 5 * 60, left: 5 * 60, running: false, endAt: 0 })
export const timerLeft = () => (timer.running ? (timer.endAt - now.value) / 1000 : timer.left)

/* ---------- lembretes ---------- */
export const reminders = ref<Reminder[]>([])
export const remDate = (r: Reminder) => new Date(`${r.date}T${r.time}:00`)
export const saveReminders = () => api.saveReminders(JSON.parse(JSON.stringify(reminders.value)))

/** Verificações que rodam a cada tique do relógio */
export function checkAll() {
  if (focus.running && focusLeft() <= 0) {
    if (focus.mode === 'focus') {
      focus.mode = 'break'; focus.left = PAUSA
      alertIsland('Foco concluído!', 'Pausa de 5 minutos', '🎯')
    } else {
      focus.mode = 'focus'; focus.left = FOCO
      alertIsland('Pausa encerrada', 'Bora para mais um ciclo', '💪')
    }
    focus.endAt = Date.now() + focus.left * 1000
    focus.running = focus.mode === 'break' // a pausa começa sozinha; o foco espera você
  }
  if (timer.running && timerLeft() <= 0) {
    timer.running = false; timer.left = 0
    alertIsland('Timer finalizado', 'O tempo acabou', '⏰')
  }
  let changed = false
  for (const r of reminders.value) {
    if (!r.fired && remDate(r).getTime() <= now.value) {
      r.fired = true; changed = true
      alertIsland('Lembrete', r.text, '⏰')
    }
  }
  if (changed) saveReminders()
}
