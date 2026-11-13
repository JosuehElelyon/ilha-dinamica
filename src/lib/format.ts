export const DIAS = ['DOM', 'SEG', 'TER', 'QUA', 'QUI', 'SEX', 'SÁB']
export const DIAS_CURTOS = ['Dom', 'Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sáb']
export const MESES = ['JAN', 'FEV', 'MAR', 'ABR', 'MAI', 'JUN', 'JUL', 'AGO', 'SET', 'OUT', 'NOV', 'DEZ']
export const MESES_LONGOS = ['Janeiro', 'Fevereiro', 'Março', 'Abril', 'Maio', 'Junho', 'Julho', 'Agosto', 'Setembro', 'Outubro', 'Novembro', 'Dezembro']

export const pad = (n: number) => String(n).padStart(2, '0')
export const ymd = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`

/** segundos → m:ss ou h:mm:ss */
export function fmt(sec: number) {
  sec = Math.max(0, Math.round(sec))
  const h = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), s = sec % 60
  return h ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`
}

/** bytes → texto amigável em pt-BR */
export function fmtSize(b: number) {
  if (!b) return '0 MB'
  if (b >= 1024 ** 3) return (b / 1024 ** 3).toFixed(1).replace('.', ',') + ' GB'
  if (b >= 1024 ** 2) return Math.round(b / 1024 ** 2) + ' MB'
  return Math.max(1, Math.round(b / 1024)) + ' KB'
}

export function hexHue(hex: string) {
  const n = parseInt(hex.slice(1), 16)
  const r = (n >> 16) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn
  if (!d) return 18
  const h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4
  return Math.round((h * 60 + 360) % 360)
}

export const hashHue = (s: string) => {
  let h = 0
  for (const c of s) h = (h * 31 + c.charCodeAt(0)) % 360
  return h
}

function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
  r /= 255; g /= 255; b /= 255
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2, d = mx - mn
  if (!d) return [0, 0, l]
  const s = d / (1 - Math.abs(2 * l - 1))
  const h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4
  return [(h * 60 + 360) % 360, s, l]
}

/** Matiz predominante de uma imagem (para a ilha assumir a cor da capa) */
export function hueFromImage(src: string): Promise<number | null> {
  return new Promise(res => {
    const im = new Image()
    im.onload = () => {
      const c = document.createElement('canvas'); c.width = c.height = 24
      const x = c.getContext('2d')!
      x.drawImage(im, 0, 0, 24, 24)
      const d = x.getImageData(0, 0, 24, 24).data
      let sx = 0, sy = 0, w = 0
      for (let i = 0; i < d.length; i += 4) {
        const [h, s, l] = rgbToHsl(d[i], d[i + 1], d[i + 2])
        const wt = s * (1 - Math.abs(l - 0.5) * 2)
        if (wt < 0.08) continue
        sx += Math.cos(h * Math.PI / 180) * wt; sy += Math.sin(h * Math.PI / 180) * wt; w += wt
      }
      res(w ? Math.round((Math.atan2(sy, sx) * 180 / Math.PI + 360) % 360) : null)
    }
    im.onerror = () => res(null)
    im.src = src
  })
}

/** Ícone e descrição do código de tempo WMO (Open-Meteo) */
export function wmo(c: number, day = 1): [string, string] {
  if (c === 0) return day ? ['☀️', 'Céu limpo'] : ['🌙', 'Céu limpo']
  if (c === 1) return day ? ['🌤️', 'Poucas nuvens'] : ['🌙', 'Poucas nuvens']
  if (c === 2) return ['⛅', 'Parcialmente nublado']
  if (c === 3) return ['☁️', 'Nublado']
  if (c === 45 || c === 48) return ['🌫️', 'Neblina']
  if (c >= 51 && c <= 57) return ['🌦️', 'Garoa']
  if (c >= 61 && c <= 67) return ['🌧️', 'Chuva']
  if ((c >= 71 && c <= 77) || c === 85 || c === 86) return ['🌨️', 'Neve']
  if (c >= 80 && c <= 82) return ['🌦️', 'Pancadas de chuva']
  if (c >= 95) return ['⛈️', 'Tempestade']
  return ['🌡️', '—']
}

export const ago = (t: number) => {
  const s = (Date.now() - t) / 1000
  if (s < 60) return 'AGORA'
  if (s < 3600) return `${Math.floor(s / 60)} MIN`
  return `${Math.floor(s / 3600)} H`
}
