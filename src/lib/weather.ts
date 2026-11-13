// Clima pela Open-Meteo (gratuita, sem chave)
import { reactive } from 'vue'
import { S } from '../store/settings'
import { live } from '../store/state'
import { wmo } from './format'

export interface Forecast {
  current: {
    time: string; temperature_2m: number; apparent_temperature: number
    relative_humidity_2m: number; wind_speed_10m: number; weather_code: number; is_day: number
  }
  hourly: { time: string[]; temperature_2m: number[]; weather_code: number[]; precipitation_probability: number[] }
  daily: {
    time: string[]; weather_code: number[]; temperature_2m_max: number[]
    temperature_2m_min: number[]; precipitation_probability_max: number[]
  }
}

export const weather = reactive({ data: null as Forecast | null, city: '', error: '' })

let autoLoc: { name: string; lat: number; lon: number } | null = null

async function location() {
  if (S.city) return S.city
  if (autoLoc) return autoLoc
  try {
    const j = await (await fetch('https://ipapi.co/json/')).json()
    if (j.latitude) return (autoLoc = { name: j.city, lat: j.latitude, lon: j.longitude })
  } catch { /* sem conexão */ }
  return { name: 'São Paulo', lat: -23.55, lon: -46.63 }
}

export async function loadWeather() {
  const loc = await location()
  weather.city = loc.name
  try {
    const u = `https://api.open-meteo.com/v1/forecast?latitude=${loc.lat}&longitude=${loc.lon}` +
      '&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code,is_day' +
      '&hourly=temperature_2m,weather_code,precipitation_probability' +
      '&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone=auto&forecast_days=6'
    weather.data = await (await fetch(u)).json()
    weather.error = ''
    const c = weather.data!.current
    live.tempText = `${wmo(c.weather_code, c.is_day)[0]} ${Math.round(c.temperature_2m)}°`
  } catch {
    weather.error = 'Sem conexão'
  }
}

export function resetAutoLocation() { autoLoc = null }

export async function searchCity(q: string) {
  const j = await (await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=1&language=pt`)).json()
  const r = j.results && j.results[0]
  return r ? { name: r.name as string, region: (r.admin1 || r.country || '') as string, lat: r.latitude as number, lon: r.longitude as number } : null
}
