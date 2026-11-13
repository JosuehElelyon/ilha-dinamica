<script setup lang="ts">
// Clima: agora, próximas 8 horas e próximos 6 dias
import { computed } from 'vue'
import { DIAS_CURTOS, wmo } from '../../lib/format'
import { weather } from '../../lib/weather'

const cur = computed(() => weather.data?.current)
const hours = computed(() => {
  const d = weather.data
  if (!d) return []
  const nowKey = d.current.time.slice(0, 13)
  let i0 = d.hourly.time.findIndex(t => t.slice(0, 13) >= nowKey)
  if (i0 < 0) i0 = 0
  return d.hourly.time.slice(i0, i0 + 8).map((t, k) => ({
    label: k === 0 ? 'AGORA' : t.slice(11, 13) + 'H',
    icon: wmo(d.hourly.weather_code[i0 + k])[0],
    temp: Math.round(d.hourly.temperature_2m[i0 + k]),
    rain: d.hourly.precipitation_probability[i0 + k] ?? 0
  }))
})
const days = computed(() => {
  const d = weather.data
  if (!d) return []
  return d.daily.time.map((t, i) => ({
    label: i === 0 ? 'HOJE' : DIAS_CURTOS[new Date(t + 'T12:00').getDay()].toUpperCase(),
    icon: wmo(d.daily.weather_code[i])[0],
    max: Math.round(d.daily.temperature_2m_max[i]),
    min: Math.round(d.daily.temperature_2m_min[i]),
    rain: d.daily.precipitation_probability_max[i] ?? 0
  }))
})
</script>

<template>
  <div class="wx">
    <div class="now">
      <div class="main"><span class="e">{{ cur ? wmo(cur.weather_code, cur.is_day)[0] : '🌡️' }}</span>
        <span class="mono t">{{ cur ? Math.round(cur.temperature_2m) + '°' : '--°' }}</span></div>
      <div class="desc">{{ cur ? wmo(cur.weather_code, cur.is_day)[1] : (weather.error || 'Carregando…') }}</div>
      <div class="mute">{{ weather.city }}</div>
      <div v-if="cur" class="mute mono small">
        SENSAÇÃO {{ Math.round(cur.apparent_temperature) }}°<br>UMIDADE {{ cur.relative_humidity_2m }}%<br>VENTO {{ Math.round(cur.wind_speed_10m) }} KM/H
      </div>
    </div>
    <div class="strip hours">
      <div v-for="h in hours" :key="h.label" class="cell">
        <span class="k">{{ h.label }}</span><span class="ce">{{ h.icon }}</span><span>{{ h.temp }}°</span><span class="r">{{ h.rain }}%</span>
      </div>
    </div>
    <div class="strip days">
      <div v-for="d in days" :key="d.label" class="cell">
        <span class="k">{{ d.label }}</span><span class="ce">{{ d.icon }}</span>
        <span>{{ d.max }}° <span class="mute">{{ d.min }}°</span></span><span class="r">{{ d.rain }}%</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.wx { display: grid; grid-template-columns: 200px 1fr; grid-template-rows: 1fr 1fr; gap: 8px; height: 100%; }
.wx > div { background: rgba(255,255,255,.04); border-radius: 18px; }
.now { grid-row: span 2; padding: 14px 16px; display: flex; flex-direction: column; gap: 3px; font-size: 13px; }
.main { display: flex; align-items: center; gap: 8px; margin-bottom: auto; }
.e, .t { font-size: 40px; }
.t { font-weight: 600; letter-spacing: 0; }
.desc { font-weight: 600; }
.strip { display: grid; align-items: center; text-align: center; padding: 6px; }
.hours { grid-template-columns: repeat(8, 1fr); }
.days { grid-template-columns: repeat(6, 1fr); }
.cell { display: flex; flex-direction: column; gap: 1px; align-items: center; font-size: 13px; }
.k { font-family: var(--mono); font-size: 10px; color: var(--mute); }
.ce { font-size: 18px; }
.r { font-size: 10px; color: #6cb6ff; font-family: var(--mono); }
</style>
