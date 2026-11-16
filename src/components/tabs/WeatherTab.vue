<script setup lang="ts">
// Clima: agora, próximas 8 horas e próximos 6 dias
import { computed } from 'vue'
import { DIAS_CURTOS, wmo } from '../../lib/format'
import { loadWeather, weather } from '../../lib/weather'

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
      <template v-if="weather.loading && !cur">
        <div class="main">
          <span class="skel skel-wx-ico" />
          <span class="skel skel-wx-temp" />
        </div>
        <div class="skel skel-wx-line" style="width: 72%" />
        <div class="skel skel-wx-sub" style="width: 52%" />
        <div class="skel-wx-meta">
          <div class="skel skel-wx-sub" style="width: 82%" />
          <div class="skel skel-wx-sub" style="width: 68%" />
          <div class="skel skel-wx-sub" style="width: 74%" />
        </div>
      </template>
      <template v-else-if="weather.error && !cur">
        <div class="wx-state">
          <span class="desc">Sem previsão</span>
          <span class="mute small">{{ weather.error }}</span>
          <button class="pillbtn tiny" @click="loadWeather()">Tentar de novo</button>
        </div>
      </template>
      <template v-else-if="!cur">
        <div class="wx-state">
          <span class="desc">Previsão do tempo</span>
          <span class="mute small">Veja temperatura atual, próximas horas e 6 dias.</span>
          <button class="pillbtn primary tiny" @click="loadWeather()">Carregar clima</button>
        </div>
      </template>
      <template v-else>
        <div class="main"><span class="e">{{ wmo(cur.weather_code, cur.is_day)[0] }}</span>
          <span class="mono t">{{ Math.round(cur.temperature_2m) }}°</span></div>
        <div class="desc">{{ wmo(cur.weather_code, cur.is_day)[1] }}</div>
        <div class="mute">{{ weather.city }}</div>
        <div class="mute mono small">
          SENSAÇÃO {{ Math.round(cur.apparent_temperature) }}°<br>UMIDADE {{ cur.relative_humidity_2m }}%<br>VENTO {{ Math.round(cur.wind_speed_10m) }} KM/H
        </div>
      </template>
    </div>
    <div class="strip hours">
      <template v-if="weather.loading && !hours.length">
        <div v-for="n in 8" :key="'hs-' + n" class="cell" aria-hidden="true">
          <span class="skel skel-k" /><span class="skel skel-ce" /><span class="skel skel-v" /><span class="skel skel-k" />
        </div>
      </template>
      <div v-for="h in hours" v-else :key="h.label" class="cell">
        <span class="k">{{ h.label }}</span><span class="ce">{{ h.icon }}</span><span>{{ h.temp }}°</span><span class="r">{{ h.rain }}%</span>
      </div>
    </div>
    <div class="strip days">
      <template v-if="weather.loading && !days.length">
        <div v-for="n in 6" :key="'ds-' + n" class="cell" aria-hidden="true">
          <span class="skel skel-k" /><span class="skel skel-ce" /><span class="skel skel-v" /><span class="skel skel-k" />
        </div>
      </template>
      <div v-for="d in days" v-else :key="d.label" class="cell">
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
.tiny { padding: 3px 9px; font-size: 11px; }
.wx-state { flex: 1; display: flex; flex-direction: column; align-items: flex-start; justify-content: center; gap: 7px; }
.skel-wx-ico { width: 42px; height: 42px; border-radius: 12px; }
.skel-wx-temp { width: 68px; height: 38px; border-radius: 10px; }
.skel-wx-line { height: 14px; margin-top: 6px; }
.skel-wx-sub { height: 10px; margin-top: 3px; }
.skel-wx-meta { display: flex; flex-direction: column; gap: 4px; margin-top: 6px; }
.skel-k { width: 26px; height: 9px; }
.skel-ce { width: 20px; height: 20px; border-radius: 6px; margin: 2px 0; }
.skel-v { width: 28px; height: 13px; }
</style>
