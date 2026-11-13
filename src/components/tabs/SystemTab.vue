<script setup lang="ts">
// Sistema: brilho, CPU, RAM, bateria e tempo ligado
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from '../../lib/api'
import { live } from '../../store/state'

const bright = ref<number | null>(null)
const value = ref(50)
let poll: number | undefined, t: number | undefined, dragging = false

async function readBright() {
  if (dragging) return
  bright.value = await api.getBrightness()
  if (bright.value !== null) value.value = bright.value
}
onMounted(() => { readBright(); poll = window.setInterval(readBright, 10000) })
onUnmounted(() => clearInterval(poll))
function onBright() {
  dragging = true
  clearTimeout(t)
  t = window.setTimeout(async () => { await api.setBrightness(value.value); dragging = false }, 250)
}

const gb = (b: number) => (b / 1024 ** 3).toFixed(1)
const s = computed(() => live.sys)
const ramPct = computed(() => (s.value ? (100 * s.value.memUsed) / s.value.memTotal : 0))
const uptime = computed(() => {
  if (!s.value) return ''
  const h = Math.floor(s.value.uptime / 3600), m = Math.floor((s.value.uptime % 3600) / 60)
  return `LIGADO HÁ ${h}H ${m}MIN`
})
</script>

<template>
  <div class="sys">
    <div class="stat">
      <span class="big-ico">☀️</span>
      <input v-model.number="value" type="range" min="0" max="100" :disabled="bright === null"
             :title="bright === null ? 'Monitores externos normalmente não aceitam ajuste de brilho pelo Windows' : ''"
             :style="{ '--p': value + '%' }" @input="onBright">
      <span class="val mono">{{ bright === null ? 'Indisponível' : value + '%' }}</span>
    </div>
    <div class="stat"><span class="label mono">CPU</span><div class="bar"><b :style="{ width: (s?.cpu ?? 0) + '%' }" /></div><span class="val mono">{{ s?.cpu ?? 0 }}%</span></div>
    <div class="stat"><span class="label mono">RAM</span><div class="bar ram"><b :style="{ width: ramPct + '%' }" /></div>
      <span class="val mono">{{ s ? `${gb(s.memUsed)}/${gb(s.memTotal)} GB` : '—' }}</span></div>
    <div class="stat"><span class="label mono">BATERIA</span><div class="bar bat"><b :style="{ width: (s?.battery?.level ?? 0) + '%' }" /></div>
      <span class="val mono">{{ s?.battery ? `${s.battery.level}%${s.battery.charging ? ' ⚡' : ''}` : 'Sem bateria' }}</span></div>
    <div class="foot mono"><span>{{ s?.cpuName }}</span><span>{{ uptime }}</span></div>
  </div>
</template>

<style scoped>
.sys { display: flex; flex-direction: column; gap: 12px; padding: 8px 24px; }
.ram b { background: linear-gradient(90deg, #6f62ff, #a59bff); }
.bat b { background: linear-gradient(90deg, #22b765, #5be39a); }
.foot { display: flex; justify-content: space-between; font-size: 10px; color: var(--mute); }
</style>
