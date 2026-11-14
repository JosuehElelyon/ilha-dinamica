// Estado reativo do Assistente Pessoal: contas, datas importantes, ponte do celular e alertas no avatar.
import { computed, reactive, ref } from 'vue'
import { api } from '../lib/api'
import type { AssistantSnapshot, GithubDeviceChallenge, PairingInfo, ProactiveAlert } from '../lib/types'
import { alertIsland, island, showToast } from './state'

export type AssistantSubTab = 'hoje' | 'contas' | 'celular'

const EMPTY_SNAP: AssistantSnapshot = {
  googleConnected: false,
  accounts: [],
  upcomingDates: [],
  todayEvents: [],
  allEvents: [],
  peopleCount: 0,
  phones: [],
  recentSignals: [],
  alerts: [],
  devItems: []
}

export const assistant = reactive({
  subTab: 'hoje' as AssistantSubTab,
  loading: false,
  syncing: false,
  error: '',
  onboardingDismissed: false,
  autoPauseOnCall: true,
  snap: structuredClone(EMPTY_SNAP) as AssistantSnapshot,
  pairing: null as PairingInfo | null,
  githubDevice: null as GithubDeviceChallenge | null
})

export const topAlert = computed<ProactiveAlert | null>(() => assistant.snap.alerts[0] || null)
export const primaryPhone = computed(() => assistant.snap.phones.find(p => p.connected) || assistant.snap.phones[0] || null)

export function openAssistant(subTab: AssistantSubTab = 'hoje') {
  assistant.subTab = subTab
  island.tab = 'assistant'
  island.menuOpen = false
  island.loginOpen = false
}

export function openLoginModal() {
  assistant.error = ''
  island.menuOpen = false
  island.loginOpen = true
}

export function handleAlertAction(alert: ProactiveAlert) {
  if (alert.actionTarget.startsWith('tab:assistant:')) {
    const sub = alert.actionTarget.split(':')[2] as AssistantSubTab
    openAssistant(sub || 'hoje')
    return
  }
  if (alert.actionTarget.startsWith('http')) {
    api.runAction(alert.actionTarget)
    return
  }
  openAssistant('hoje')
}

export async function refreshAssistant() {
  assistant.loading = true
  try {
    assistant.snap = await api.assistantSnapshot()
    assistant.pairing = await api.phonePairingInfo()
    assistant.error = ''
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.loading = false
  }
}

export async function connectGoogle(clientId?: string, clientSecret?: string) {
  assistant.syncing = true
  assistant.error = ''
  try {
    assistant.snap = await api.assistantGoogleConnect(clientId, clientSecret)
    showToast({ icon: '✅', text: 'Google conectado ao Assistente!', ms: 3200 })
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

export async function syncGoogle() {
  assistant.syncing = true
  assistant.error = ''
  try {
    assistant.snap = await api.assistantGoogleSync()
    showToast({ icon: '↻', text: 'Agenda e contatos Google atualizados', ms: 2400 })
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

export async function syncWindowsCalendar() {
  assistant.syncing = true
  assistant.error = ''
  try {
    assistant.snap = await api.assistantSyncWindows()
    showToast({ icon: '🪟', text: 'Calendário do Windows sincronizado', ms: 2400 })
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

export async function connectGitHub(token: string) {
  assistant.syncing = true
  assistant.error = ''
  try {
    assistant.snap = await api.assistantGithubConnect(token)
    showToast({ icon: '🐙', text: 'GitHub conectado', ms: 2500 })
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

let ghPollTimer: number | null = null

export function cancelGithubDeviceFlow() {
  if (ghPollTimer !== null) {
    clearTimeout(ghPollTimer)
    ghPollTimer = null
  }
  assistant.githubDevice = null
}

export async function startGithubDeviceFlow(clientId: string) {
  cancelGithubDeviceFlow()
  assistant.error = ''
  assistant.syncing = true
  try {
    const ch = await api.assistantGithubDeviceStart(clientId)
    assistant.githubDevice = ch
    await api.copy(ch.userCode)
    showToast({ icon: '🐙', text: `Código ${ch.userCode} copiado! Autorize no GitHub.`, ms: 4000 })
    const waitMs = Math.max(5, ch.interval || 5) * 1000
    const poll = async () => {
      if (!assistant.githubDevice) return
      try {
        const snap = await api.assistantGithubDevicePoll(clientId, ch.deviceCode)
        if (snap) {
          assistant.snap = snap
          cancelGithubDeviceFlow()
          showToast({ icon: '🐙', text: 'GitHub conectado via OAuth!', ms: 2800 })
          return
        }
        ghPollTimer = window.setTimeout(poll, waitMs)
      } catch (e) {
        assistant.error = String(e)
        cancelGithubDeviceFlow()
      }
    }
    ghPollTimer = window.setTimeout(poll, waitMs)
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

export async function connectApple(appleId: string, appPassword: string) {
  assistant.syncing = true
  assistant.error = ''
  try {
    assistant.snap = await api.assistantAppleConnect(appleId, appPassword)
    showToast({ icon: '🍎', text: 'Apple iCloud conectado', ms: 2500 })
  } catch (e) {
    assistant.error = String(e)
  } finally {
    assistant.syncing = false
  }
}

export async function disconnectAccount(accountId: string) {
  assistant.snap = await api.assistantDisconnectAccount(accountId)
  showToast({ icon: '🔌', text: 'Conta desconectada', ms: 2200 })
}

export async function addImportantDate(name: string, kind: string, month: number, day: number, yearOptional?: number | null) {
  assistant.error = ''
  try {
    assistant.snap = await api.assistantAddImportantDate(name, kind, month, day, yearOptional)
    showToast({ icon: '🎂', text: `Data salva: ${name}`, ms: 2400 })
  } catch (e) {
    assistant.error = String(e)
  }
}

export async function deleteImportantDate(id: string) {
  assistant.snap = await api.assistantDeleteImportantDate(id)
}

export async function rotatePairingToken() {
  assistant.pairing = await api.phoneRotateToken()
  showToast({ icon: '🔐', text: 'Novo código de pareamento gerado', ms: 2200 })
}

export async function enableUsbBridge() {
  assistant.error = ''
  try {
    const devs = await api.phoneEnableUsbBridge()
    assistant.pairing = await api.phonePairingInfo()
    showToast({ icon: '🔌', text: `Ponte USB ativa (${devs[0]})`, ms: 3000 })
  } catch (e) {
    assistant.error = String(e)
  }
}

export async function triggerTestPhoneSignal(kind: string, title: string, body: string, transport: string) {
  assistant.snap = await api.phoneIngestDemoSignal(kind, title, body, transport)
}

export async function sendClipboardToPhone() {
  assistant.error = ''
  try {
    const preview = await api.phoneSendClipboard()
    showToast({ icon: '📋', text: `Enviado ao celular: ${preview}`, ms: 2600 })
  } catch (e) {
    assistant.error = String(e)
  }
}

export async function ringPhone() {
  await api.phoneRing()
  showToast({ icon: '🔔', text: 'Chamado enviado para o celular!', ms: 2400 })
}

export async function toggleAutoPauseOnCall() {
  const next = !assistant.autoPauseOnCall
  assistant.autoPauseOnCall = await api.phoneSetAutoPauseOnCall(next)
  showToast({
    icon: assistant.autoPauseOnCall ? '🔇' : '🔊',
    text: assistant.autoPauseOnCall ? 'Pausar mídia ao receber ligação: ON' : 'Pausar mídia ao receber ligação: OFF',
    ms: 2400
  })
}

export function sendBirthdayWish(name: string) {
  const msg = `Feliz aniversário, ${name}! 🎉🎂 Tudo de bom!`
  api.copy(msg)
  showToast({ icon: '🎉', text: `Parabéns copiado para ${name}!`, ms: 2600 })
  api.runAction(`https://wa.me/?text=${encodeURIComponent(msg)}`)
}

const listenersAttached = ref(false)
export function initAssistant() {
  refreshAssistant()
  if (listenersAttached.value) return
  listenersAttached.value = true
  api.onAssistantUpdate(s => {
    assistant.snap = s
  })
  api.onAssistantAlert(a => {
    alertIsland(a.title, a.body, a.icon)
  })
}
