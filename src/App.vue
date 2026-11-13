<script setup lang="ts">
// Ilha Dinâmica — componente raiz: expandir/recolher, tema, avisos e carregamento inicial.
// Copyright (c) 2026 Josueh — Licença MIT
import { computed, onMounted, ref, watch, type Component } from 'vue'
import { api } from './lib/api'
import { hexHue, hashHue, hueFromImage } from './lib/format'
import { loadWeather } from './lib/weather'
import { S, loadSettings, activeModules } from './store/settings'
import { island, live, now, checkAll, showToast, alertIsland, reminders, remDate } from './store/state'
import type { Devices, VolInfo } from './lib/types'

import IslandPill from './components/IslandPill.vue'
import IslandHeader from './components/IslandHeader.vue'
import NavBar from './components/NavBar.vue'
import HomeTab from './components/tabs/HomeTab.vue'
import ActionsTab from './components/tabs/ActionsTab.vue'
import ClipsTab from './components/tabs/ClipsTab.vue'
import NotesTab from './components/tabs/NotesTab.vue'
import TimerTab from './components/tabs/TimerTab.vue'
import CalendarTab from './components/tabs/CalendarTab.vue'
import WeatherTab from './components/tabs/WeatherTab.vue'
import AudioTab from './components/tabs/AudioTab.vue'
import SystemTab from './components/tabs/SystemTab.vue'
import CleanTab from './components/tabs/CleanTab.vue'
import ProcessesTab from './components/tabs/ProcessesTab.vue'
import ModulesTab from './components/tabs/ModulesTab.vue'
import SettingsTab from './components/tabs/SettingsTab.vue'

const TABS: Record<string, Component> = {
  home: HomeTab, actions: ActionsTab, clips: ClipsTab, notes: NotesTab, timer: TimerTab,
  cal: CalendarTab, weather: WeatherTab, audio: AudioTab, sys: SystemTab, clean: CleanTab,
  procs: ProcessesTab, modules: ModulesTab, settings: SettingsTab
}

const el = ref<HTMLElement>()
const ready = ref(false)

/* ---------------- expandir / recolher ---------------- */
let leaveTimer: number | undefined
function enter() {
  clearTimeout(leaveTimer)
  island.hovering = true
  api.setInteractive(true)
  island.expanded = true
}
function leave() {
  island.hovering = false
  clearTimeout(leaveTimer)
  leaveTimer = window.setTimeout(() => collapse(), 450)
}
function typing() {
  const a = document.activeElement
  return !!a && a.matches('textarea, input[type=text], input[type=time], input:not([type]), select')
}
function collapse(force = false) {
  if (island.pinned || island.hovering) return
  if (!force && typing()) return // não fecha enquanto você digita
  island.expanded = false
  island.menuOpen = false
  api.setInteractive(false)
}
function setPinned(v: boolean) {
  island.pinned = v
  island.expanded = v || island.hovering
  api.setInteractive(island.expanded)
  if (!v) collapse(true)
}
api.onHover(inside => (inside ? enter() : leave()))
api.onToggle(() => setPinned(!island.expanded))
window.addEventListener('blur', () => { if (!island.hovering) collapse(true) })
document.addEventListener('click', () => (island.menuOpen = false))
function onFocusOut() { setTimeout(() => { if (!island.hovering) collapse() }, 60) }

// informa ao Rust onde a ilha está, para o clique passar pelo resto da janela
let lastRect = ''
setInterval(() => {
  if (!el.value) return
  const r = el.value.getBoundingClientRect()
  const key = [r.x, r.y, r.width, r.height].map(Math.round).join(',')
  if (key !== lastRect) { lastRect = key; api.setHitRect(r.x, r.y, r.width, r.height) }
}, 100)

/* ---------------- tema ---------------- */
const themeStyle = computed(() => {
  const playing = live.media.status === 'Playing' && !!live.media.title
  let h = hexHue(S.accent)
  if (S.followMusic && live.media.title) h = live.coverHue ?? hashHue(live.media.title + live.media.artist)
  return {
    '--accent': S.accent,
    '--alpha': String(S.opacity / 100),
    '--scale': String({ compacta: 0.88, normal: 1, grande: 1.12 }[S.size] ?? 1),
    '--h': String(h),
    '--glow': String(S.followMusic && playing ? 1 : 0.35)
  }
})
watch(themeStyle, s => Object.entries(s).forEach(([k, v]) => document.documentElement.style.setProperty(k, v)), { immediate: true })

/* ---------------- dados ao vivo + avisos ---------------- */
let firstMedia = true
api.onMedia(m => {
  const changed = m.title + '|' + m.artist !== live.media.title + '|' + live.media.artist
  if (m.cover && m.cover !== live.media.cover) hueFromImage(m.cover).then(h => (live.coverHue = h))
  if (!m.cover && changed) live.coverHue = null
  live.media = m
  if (changed && !firstMedia && m.title && m.status === 'Playing' && S.alerts.track)
    showToast({ img: m.cover || undefined, icon: m.cover ? '' : '🎵', text: `${m.title}${m.artist ? ' — ' + m.artist : ''}`, ms: 3500 })
  firstMedia = false
})
api.onClips(l => (live.clips = l))

api.onVol((v: VolInfo) => {
  const prev = live.vol
  live.vol = v
  if (!prev || island.expanded || !S.alerts.vol) return
  if (prev.level !== v.level || prev.mute !== v.mute)
    showToast({ icon: v.mute ? '🔇' : '🔊', text: v.mute ? 'Mudo' : `Volume ${v.level}%`, progress: v.mute ? 0 : v.level, ms: 1800 })
  else if (prev.micMute !== v.micMute)
    showToast({ icon: v.micMute ? '🚫' : '🎙️', text: v.micMute ? 'Microfone silenciado' : 'Microfone ativo', ms: 2200 })
})

const devIcon = (n: string) => /fone|headphone|headset|airpods|buds|bluetooth/i.test(n) ? '🎧' : /hdmi|monitor|display|tv/i.test(n) ? '🖥️' : '🔈'
api.onDevices((d: Devices) => {
  const prev = live.devices
  live.devices = d
  if (!prev || !S.alerts.dev) return
  const ids = (l: { id: string }[]) => l.map(x => x.id)
  const def = (l: Devices['out']) => l.find(x => x.def)
  const novo = d.out.find(x => !ids(prev.out).includes(x.id))
  const saiu = prev.out.find(x => !ids(d.out).includes(x.id))
  if (novo) showToast({ icon: devIcon(novo.name), text: `Conectado: ${novo.name}`, ms: 3000 })
  else if (saiu) showToast({ icon: '🔌', text: `Desconectado: ${saiu.name}`, ms: 3000 })
  else if (def(prev.out)?.id !== def(d.out)?.id && def(d.out)) showToast({ icon: devIcon(def(d.out)!.name), text: `Saída: ${def(d.out)!.name}`, ms: 2500 })
  else if (def(prev.in)?.id !== def(d.in)?.id && def(d.in)) showToast({ icon: '🎙️', text: `Microfone: ${def(d.in)!.name}`, ms: 2500 })
})

let warned20 = false, warned10 = false
api.onSys(s => {
  const prev = live.sys?.battery
  live.sys = s
  const b = s.battery
  if (!b || !S.alerts.bat) return
  if (prev && prev.charging !== b.charging)
    showToast({ icon: b.charging ? '⚡' : '🔋', text: b.charging ? `Carregando · ${b.level}%` : `Na bateria · ${b.level}%`, progress: b.level, ms: 2500 })
  if (b.charging) { warned20 = warned10 = false; return }
  if (b.level <= 10 && !warned10) { warned10 = warned20 = true; alertIsland('Bateria crítica', `${b.level}% — conecte o carregador`, '🪫') }
  else if (b.level <= 20 && !warned20) { warned20 = true; alertIsland('Bateria fraca', `${b.level}% restantes`, '🪫') }
})

window.addEventListener('offline', () => { if (S.alerts.net) showToast({ icon: '📡', text: 'Sem conexão com a internet', alert: true, ms: 5000 }) })
window.addEventListener('online', () => { if (S.alerts.net) showToast({ icon: '🛜', text: 'Conectado de novo', ms: 2500 }); loadWeather() })

watch(now, checkAll)

/* ---------------- inicialização ---------------- */
onMounted(async () => {
  const d = await api.getData()
  loadSettings(d.settings || {})
  Object.assign(island, {
    isAdmin: d.isAdmin, autostart: d.autostart, platform: d.platform, version: d.version,
    protectedNames: new Set(d.protectedNames || []), selfName: (d.selfName || '').toLowerCase(), ownPids: d.ownPids || []
  })
  // lembretes antigos (app estava fechado há muito tempo) são marcados sem disparar
  const limite = Date.now() - 10 * 60 * 1000
  reminders.value = (d.reminders || []).map(r => (!r.fired && remDate(r).getTime() < limite ? { ...r, fired: true } : r))
  island.tab = activeModules()[0] || 'home'
  ready.value = true
  loadWeather()
  setInterval(loadWeather, 30 * 60 * 1000)
})
</script>

<template>
  <div
    id="island" ref="el"
    :class="[island.expanded ? 'expanded' : 'collapsed', { toasting: !!island.toast, alert: island.toast?.alert }]"
    @mouseenter="enter" @mouseleave="leave" @focusout="onFocusOut"
  >
    <IslandPill />
    <div class="panel">
      <div class="glow" />
      <IslandHeader @pin="setPinned" />
      <main v-if="ready">
        <component :is="TABS[island.tab]" :key="island.tab" class="tab" />
      </main>
      <NavBar />
    </div>
  </div>
</template>
