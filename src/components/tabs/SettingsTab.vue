<script setup lang="ts">
// Personalizar: cor, tamanho, opacidade, cidade do clima, avisos + "Sobre" (licença)
import { ref } from 'vue'
import { ACCENTS, S, save, type Settings } from '../../store/settings'
import { island } from '../../store/state'
import { loadWeather, resetAutoLocation, searchCity } from '../../lib/weather'

const city = ref(S.city?.name ?? '')
const cityMsg = ref(S.city ? `Fixa: ${S.city.name}` : 'Automática (pela sua conexão)')
const ALERTS: [keyof Settings['alerts'], string][] = [
  ['vol', 'Volume'], ['bat', 'Bateria'], ['net', 'Internet'], ['track', 'Música nova'], ['dev', 'Fones / microfone']
]

async function setCity() {
  const q = city.value.trim()
  if (!q) {
    save('city', null); resetAutoLocation()
    cityMsg.value = 'Automática (pela sua conexão)'
    return loadWeather()
  }
  cityMsg.value = 'Buscando…'
  try {
    const r = await searchCity(q)
    if (!r) { cityMsg.value = 'Cidade não encontrada'; return }
    save('city', { name: r.name, lat: r.lat, lon: r.lon })
    cityMsg.value = `Fixa: ${r.name}${r.region ? ', ' + r.region : ''}`
    loadWeather()
  } catch { cityMsg.value = 'Sem conexão' }
}
</script>

<template>
  <div class="settings">
    <div class="set">
      <span class="set-label">COR DE DESTAQUE</span>
      <div class="swatches">
        <button v-for="c in ACCENTS" :key="c" class="sw" :class="{ on: S.accent === c }" :style="{ background: c }" @click="save('accent', c)" />
      </div>
    </div>
    <div class="set">
      <span class="set-label">TAMANHO</span>
      <div class="seg">
        <button v-for="s in (['compacta', 'normal', 'grande'] as const)" :key="s" :class="{ on: S.size === s }" @click="save('size', s)">
          {{ s[0].toUpperCase() + s.slice(1) }}</button>
      </div>
    </div>
    <div class="set">
      <span class="set-label">OPACIDADE <b>{{ S.opacity }}%</b></span>
      <input v-model.number="S.opacity" type="range" min="55" max="100" :style="{ '--p': ((S.opacity - 55) / 45 * 100) + '%' }"
             @change="save('opacity', S.opacity)">
    </div>
    <div class="set">
      <span class="set-label">CIDADE DO CLIMA</span>
      <form class="frow" autocomplete="off" @submit.prevent="setCity">
        <input v-model="city" type="text" class="in grow" placeholder="Vazio = automática">
        <button class="add" title="Buscar">↵</button>
      </form>
      <span class="mute small">{{ cityMsg }}</span>
    </div>
    <div class="set">
      <span class="set-label">VISUAL</span>
      <label class="chk"><input type="checkbox" :checked="S.followMusic" @change="save('followMusic', ($event.target as HTMLInputElement).checked)"> Cor acompanha a música</label>
      <span class="set-label about-label">SOBRE</span>
      <span class="small mute">Ilha Dinâmica v{{ island.version }} · © 2026 Josueh<br>Software livre sob a <b class="lic">Licença MIT</b></span>
    </div>
    <div class="set">
      <span class="set-label">AVISOS NA PÍLULA</span>
      <div class="chks">
        <label v-for="[k, label] in ALERTS" :key="k" class="chk">
          <input type="checkbox" :checked="S.alerts[k]" @change="save('alerts', { ...S.alerts, [k]: ($event.target as HTMLInputElement).checked })"> {{ label }}
        </label>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings { display: grid; grid-template-columns: 1fr 1fr; gap: 6px 28px; padding: 0 20px; }
.set { display: flex; flex-direction: column; gap: 4px; }
.swatches { display: flex; gap: 8px; }
.sw { width: 22px; height: 22px; border-radius: 50%; box-shadow: inset 0 0 0 1px rgba(255,255,255,.2); }
.sw.on { box-shadow: 0 0 0 2px #0e0e11, 0 0 0 4px #fff; }
.frow { display: flex; gap: 6px; }
.chks { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 10px; }
.about-label { margin-top: 4px; }
.lic { color: var(--txt); font-weight: 600; }
</style>
