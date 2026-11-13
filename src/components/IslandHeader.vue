<script setup lang="ts">
// Topo da ilha: data/clima, "olhinhos", hora e menu •••
import { computed } from 'vue'
import { api } from '../lib/api'
import { DIAS, MESES, pad } from '../lib/format'
import { island, live, now, stopwatch, focus, timer } from '../store/state'
import { S, save } from '../store/settings'

const emit = defineEmits<{ pin: [boolean] }>()

const date = computed(() => {
  const d = new Date(now.value)
  return `${DIAS[d.getDay()]} ${pad(d.getDate())} ${MESES[d.getMonth()]}${live.tempText ? ' / ' + live.tempText : ''}`
})
const clock = computed(() => new Date(now.value).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' }))
const busy = computed(() => stopwatch.running || focus.running || timer.running)

async function toggleAutostart(e: Event) {
  island.autostart = await api.setAutostart((e.target as HTMLInputElement).checked)
}
function open(tab: 'settings') { island.menuOpen = false; island.tab = tab }

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
    <span class="mono">{{ date }}</span>
    <span class="eyes" :class="{ busy }"><i /><i /></span>
    <span class="right">
      <span class="clock mono">{{ clock }}</span>
      <button class="icon-btn" title="Opções" @click.stop="island.menuOpen = !island.menuOpen">•••</button>
    </span>
    <div v-if="island.menuOpen" class="menu" @click.stop>
      <button @click="open('settings')">🎨 Personalizar…</button>
      <label><input type="checkbox" :checked="island.autostart" @change="toggleAutostart"> Iniciar com o Windows</label>
      <label><input type="checkbox" :checked="island.pinned" @change="emit('pin', ($event.target as HTMLInputElement).checked)"> Manter aberta</label>
      <label><input type="checkbox" :checked="S.locked" @change="save('locked', ($event.target as HTMLInputElement).checked)"> Travar posição</label>
      <button @click="center">📍 Voltar ao centro do topo</button>
      <span v-if="island.isAdmin" class="note">🛡️ Rodando como administrador</span>
      <button v-else @click="island.menuOpen = false; api.relaunchAdmin()">🛡️ Reiniciar como administrador</button>
      <button class="danger" @click="api.quit()">Sair</button>
    </div>
  </header>
</template>

<style scoped>
header { position: relative; z-index: 5; display: flex; align-items: center; justify-content: space-between;
  padding: 12px 20px 6px; font-size: 11px; color: #c9c9cf; }
header.draggable { cursor: grab; }
header.draggable:active { cursor: grabbing; }
.right { display: flex; align-items: center; gap: 12px; }
.clock { font-size: 12px; color: var(--txt); font-weight: 600; }
.eyes { position: absolute; left: 50%; transform: translateX(-50%); display: flex; gap: 6px; }
.eyes i { width: 5px; height: 8px; border-radius: 3px; background: #fff; animation: blink 5s infinite; }
.eyes.busy i { background: var(--accent); }
@keyframes blink { 0%,94%,100% { transform: scaleY(1) } 97% { transform: scaleY(.1) } }
.menu { position: absolute; right: 16px; top: 36px; z-index: 20; display: flex; flex-direction: column; gap: 2px;
  padding: 8px; min-width: 220px; background: #1d1d22; border: 1px solid var(--line); border-radius: 14px;
  box-shadow: 0 10px 30px rgba(0,0,0,.5); font-size: 12px; }
.menu label, .menu button { display: flex; gap: 8px; align-items: center; padding: 6px 8px; border-radius: 8px; text-align: left; cursor: pointer; }
.menu label:hover, .menu button:hover { background: rgba(255,255,255,.06); }
.note { padding: 6px 8px; color: #5be39a; }
</style>
