<script setup lang="ts">
// Topo da ilha: data/clima, "olhinhos", hora e menu •••
import { computed } from 'vue'
import HeaderPet from './HeaderPet.vue'
import { api } from '../lib/api'
import { DIAS, MESES, pad } from '../lib/format'
import { island, live, now, stopwatch, focus, timer, checkUpdate, installUpdate, nextMeeting } from '../store/state'
import { S, save } from '../store/settings'
import { assistant, openLoginModal } from '../store/assistant'

const emit = defineEmits<{ pin: [boolean] }>()
const meet = computed(() => nextMeeting())

const date = computed(() => {
  const d = new Date(now.value)
  return `${DIAS[d.getDay()]} ${pad(d.getDate())} ${MESES[d.getMonth()]}${live.tempText ? ' / ' + live.tempText : ''}`
})
const clock = computed(() => new Date(now.value).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }))
const busy = computed(() => stopwatch.running || focus.running || timer.running)

async function toggleAutostart(e: Event) {
  island.autostart = await api.setAutostart((e.target as HTMLInputElement).checked)
}
function open(tab: 'settings' | 'pets' | 'assistant') { island.menuOpen = false; island.tab = tab }

// segurar a barra do topo e arrastar move a ilha pela tela
function drag(e: MouseEvent) {
  if (e.button !== 0 || S.locked) return
  if ((e.target as HTMLElement).closest('button, input, label, .menu')) return
  island.menuOpen = false
  api.startDrag()
}
function center() { island.menuOpen = false; api.resetPosition() }
</script>

<template>
  <header :class="{ draggable: !S.locked }" :title="S.locked ? '' : 'Arraste para mover a ilha'" @mousedown="drag" @dblclick="!S.locked && center()">
    <span class="left-info">
      <span class="mono">{{ date }}</span>
      <button v-if="live.wifi.connected" class="wifi-btn" :title="`Wi-Fi: ${live.wifi.ssid} (${live.wifi.signal}%) — clique para gerenciar redes`"
              @click.stop="island.tab = 'net'">
        <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.1" stroke-linecap="round" stroke-linejoin="round">
          <path d="M5 12.55a11 11 0 0 1 14.08 0" />
          <path d="M1.42 9a16 16 0 0 1 21.16 0" />
          <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
          <circle cx="12" cy="19.5" r="1" fill="currentColor" />
        </svg>
      </button>
    </span>
    <HeaderPet :busy="busy" :active="island.expanded" />
    <span class="right">
      <button v-if="meet?.joinUrl" class="meetbtn" :title="`Entrar em: ${meet.summary}`" @click.stop="api.runAction(meet.joinUrl)">📹 Entrar</button>
      <button v-if="island.update.available" class="upd" :disabled="island.update.installing"
              :title="`Instalar a versão ${island.update.available}`" @click.stop="installUpdate()">
        {{ island.update.installing ? '⏳ Atualizando…' : `⬆ ${island.update.available}` }}</button>
      <button v-if="island.tab === 'home'" class="icon-btn kbtn" :class="{ on: island.homeEdit }"
              :title="island.homeEdit ? 'Concluir edição da Home' : 'Personalizar cards da Home'"
              @click.stop="island.homeEdit = !island.homeEdit">
        {{ island.homeEdit ? '✓ Cards' : '✎' }}
      </button>
      <button class="icon-btn kbtn" title="Paleta de comandos (Ctrl+Alt+K)" @click.stop="island.paletteOpen = !island.paletteOpen">⌘K</button>
      <span class="clock mono">{{ clock }}</span>
      <button class="icon-btn" title="Opções" @click.stop="island.menuOpen = !island.menuOpen">•••</button>
    </span>
    <div v-if="island.menuOpen" class="menu" @click.stop>
      <button @click="openLoginModal()">
        🔑 Login
        <span class="ver" :style="{ color: assistant.snap.googleConnected ? '#5be39a' : '' }">
          {{ assistant.snap.googleConnected ? 'Google' : 'Entrar' }}
        </span>
      </button>
      <button @click="open('assistant')">✨ Assistente Pessoal…</button>
      <button @click="open('settings')">🎨 Personalizar…</button>
      <button @click="open('pets')">🐾 Mascote do topo…</button>
      <label><input type="checkbox" :checked="island.autostart" @change="toggleAutostart"> Iniciar com o Windows</label>
      <label><input type="checkbox" :checked="island.pinned" @change="emit('pin', ($event.target as HTMLInputElement).checked)"> Manter aberta</label>
      <label><input type="checkbox" :checked="S.locked" @change="save('locked', ($event.target as HTMLInputElement).checked)"> Travar posição</label>
      <button @click="center">📍 Voltar ao centro do topo</button>
      <div class="sep" />
      <button v-if="island.update.available" :disabled="island.update.installing" @click="installUpdate()">
        ⬆️ {{ island.update.installing ? `Instalando ${island.update.available}…` : `Instalar v${island.update.available} agora` }}</button>
      <button v-else :disabled="island.update.checking || island.update.installing" @click="checkUpdate(true)">
        ⬆️ Verificar atualizações <span class="ver">v{{ island.version }}</span></button>
      <span v-if="island.update.status" class="status">{{ island.update.status }}</span>
      <label><input type="checkbox" :checked="S.autoUpdate" @change="save('autoUpdate', ($event.target as HTMLInputElement).checked)"> Atualizar automaticamente</label>
      <div class="sep" />
      <span v-if="island.isAdmin" class="note">🛡️ Rodando como administrador</span>
      <button v-else @click="island.menuOpen = false; api.relaunchAdmin()">🛡️ Reiniciar como administrador</button>
      <button class="danger" @click="api.quit()">Sair</button>
    </div>
  </header>
</template>

<style scoped>
header { position: relative; z-index: 5; display: flex; align-items: center; justify-content: space-between;
  padding: 12px 20px 8px; font-size: 11px; color: #c9c9cf; }
header.draggable { cursor: grab; }
header.draggable:active { cursor: grabbing; }
.left-info { display: flex; align-items: center; gap: 7px; }
.wifi-btn {
  display: inline-flex; align-items: center; justify-content: center;
  width: 22px; height: 22px; border-radius: 7px; color: #5be39a;
  background: rgba(91, 227, 154, .12); transition: background .15s, transform .15s;
}
.wifi-btn:hover { background: rgba(91, 227, 154, .24); transform: scale(1.06); }
.right { display: flex; align-items: center; gap: 10px; }
.clock { font-size: 12px; color: var(--txt); font-weight: 600; }
.menu { position: absolute; right: 16px; top: 32px; z-index: 20; display: flex; flex-direction: column; gap: 1px;
  padding: 5px; min-width: 222px; max-height: 278px; overflow-y: auto; background: #1d1d22; border: 1px solid var(--line); border-radius: 14px;
  box-shadow: 0 10px 30px rgba(0,0,0,.5); font-size: 11px; }
.menu label, .menu button { display: flex; gap: 8px; align-items: center; padding: 3.8px 8px; border-radius: 8px; text-align: left; cursor: pointer; }
.menu label:hover, .menu button:hover { background: rgba(255,255,255,.06); }
.note { padding: 4px 8px; color: #5be39a; }
.sep { height: 1px; background: var(--line); margin: 2px 4px; }
.ver { margin-left: auto; color: var(--mute); font-family: var(--mono); font-size: 10.5px; }
.status { padding: 0 8px 4px 32px; color: var(--mute); font-size: 11px; }
.upd { padding: 2px 9px; border-radius: 10px; background: var(--accent); color: #111; font-size: 11px; font-weight: 700; }
.upd:disabled { opacity: .7; }
.meetbtn { padding: 2px 9px; border-radius: 10px; background: #5b7cff; color: #fff; font-size: 11px; font-weight: 700; }
.meetbtn:hover { background: #7390ff; }
.kbtn { font-family: var(--mono); font-size: 10.5px; color: var(--mute); padding: 2px 6px; border-radius: 6px; border: 1px solid var(--line); }
.kbtn:hover { color: #fff; border-color: rgba(255,255,255,.25); }
.kbtn.on { background: var(--accent); border-color: transparent; color: #111; font-weight: 700; }
</style>
