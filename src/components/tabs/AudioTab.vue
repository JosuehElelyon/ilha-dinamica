<script setup lang="ts">
// Áudio: saída (fones/alto-falantes) e microfone — dispositivo padrão, volume e mudo
import { onMounted, ref, watch } from 'vue'
import { api } from '../../lib/api'
import { live } from '../../store/state'

const vol = ref(50), mic = ref(50)
let dragV = false, dragM = false, tV: number | undefined, tM: number | undefined
watch(() => live.vol, v => {
  if (!v) return
  if (!dragV) vol.value = v.level
  if (!dragM && v.mic >= 0) mic.value = v.mic
}, { immediate: true })
onMounted(() => api.refreshDevices())

function onVol() { dragV = true; clearTimeout(tV); tV = window.setTimeout(() => { api.setVol(vol.value); dragV = false }, 60) }
function onMic() { dragM = true; clearTimeout(tM); tM = window.setTimeout(() => { api.setMic(mic.value); dragM = false }, 60) }

function icon(name: string, isIn: boolean) {
  const n = name.toLowerCase()
  if (isIn) return /fone|headset|airpods|buds|bluetooth/.test(n) ? '🎧' : '🎙️'
  if (/fone|headphone|headset|airpods|buds|bluetooth/.test(n)) return '🎧'
  if (/hdmi|monitor|display|tv/.test(n)) return '🖥️'
  return '🔈'
}
</script>

<template>
  <div class="audio">
    <div class="col">
      <span class="set-label">🎧 SAÍDA</span>
      <div class="ctrl">
        <button class="icon-btn big-ico" title="Mudo" @click="api.toggleMute()">{{ live.vol?.mute ? '🔇' : '🔊' }}</button>
        <input v-model.number="vol" type="range" min="0" max="100" :style="{ '--p': vol + '%' }" @input="onVol">
        <span class="mono val">{{ live.vol?.mute ? 'Mudo' : vol + '%' }}</span>
      </div>
      <div class="list">
        <div v-if="!live.devices?.out.length" class="empty">Nenhuma saída encontrada</div>
        <button v-for="d in live.devices?.out" :key="d.id" class="dev" :class="{ def: d.def }"
                :title="d.def ? 'Dispositivo padrão' : 'Usar este dispositivo'" @click="!d.def && api.setDevice(d.id)">
          <span class="radio" /><span>{{ icon(d.name, false) }}</span><span class="nm">{{ d.name }}</span>
        </button>
      </div>
    </div>
    <div class="col">
      <span class="set-label">🎙️ MICROFONE</span>
      <div class="ctrl">
        <button class="icon-btn big-ico" :title="live.vol?.micMute ? 'Ativar microfone' : 'Silenciar microfone'"
                @click="api.toggleMic()">{{ live.vol?.micMute ? '🚫' : '🎙️' }}</button>
        <input v-model.number="mic" type="range" min="0" max="100" :disabled="(live.vol?.mic ?? -1) < 0"
               :style="{ '--p': mic + '%' }" @input="onMic">
        <span class="mono val">{{ (live.vol?.mic ?? -1) < 0 ? '—' : live.vol?.micMute ? 'Mudo' : mic + '%' }}</span>
      </div>
      <div class="list">
        <div v-if="!live.devices?.in.length" class="empty">Nenhum microfone encontrado</div>
        <button v-for="d in live.devices?.in" :key="d.id" class="dev" :class="{ def: d.def }"
                :title="d.def ? 'Dispositivo padrão' : 'Usar este dispositivo'" @click="!d.def && api.setDevice(d.id)">
          <span class="radio" /><span>{{ icon(d.name, true) }}</span><span class="nm">{{ d.name }}</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.audio { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; height: 100%; padding: 0 8px; }
.col { display: flex; flex-direction: column; gap: 8px; min-height: 0; }
.ctrl { display: grid; grid-template-columns: 34px 1fr 54px; gap: 8px; align-items: center; }
.val { font-size: 12px; text-align: right; }
.list { display: flex; flex-direction: column; gap: 3px; overflow-y: auto; min-height: 0; }
.dev { display: flex; align-items: center; gap: 10px; padding: 8px 10px; border-radius: 12px; background: rgba(255,255,255,.04);
  font-size: 12.5px; text-align: left; width: 100%; }
.dev:hover { background: rgba(255,255,255,.08); }
.radio { width: 14px; height: 14px; border-radius: 50%; box-shadow: inset 0 0 0 1.5px #6b6b74; flex: none; }
.dev.def { background: color-mix(in srgb, var(--accent), transparent 82%); }
.dev.def .radio { box-shadow: inset 0 0 0 4px var(--accent); }
.nm { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.empty { padding: 14px 4px; }
</style>
