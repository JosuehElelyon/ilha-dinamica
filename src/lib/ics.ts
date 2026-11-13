// Parser leve de arquivos .ics (Outlook / Microsoft Teams) sem dependências externas.
import { pad, ymd } from './format'
import type { Meeting } from './types'

const DAY_CODES: Record<string, number> = { SU: 0, MO: 1, TU: 2, WE: 3, TH: 4, FR: 5, SA: 6 }

function unescapeIcs(v: string) {
  return v.replace(/\\n/gi, '\n').replace(/\\,/g, ',').replace(/\\;/g, ';').replace(/\\\\/g, '\\').trim()
}

function parseIcsDate(raw: string): { date: Date; allDay: boolean } | null {
  const s = raw.trim()
  const mDay = /^(\d{4})(\d{2})(\d{2})$/.exec(s)
  if (mDay) {
    return { date: new Date(+mDay[1], +mDay[2] - 1, +mDay[3], 0, 0, 0), allDay: true }
  }
  const mDt = /^(\d{4})(\d{2})(\d{2})T(\d{2})(\d{2})(\d{2})(Z?)$/.exec(s)
  if (!mDt) return null
  const [, y, mo, d, h, mi, se, z] = mDt
  const date = z === 'Z'
    ? new Date(Date.UTC(+y, +mo - 1, +d, +h, +mi, +se))
    : new Date(+y, +mo - 1, +d, +h, +mi, +se)
  return { date, allDay: false }
}

function extractJoinUrl(fields: string[]): string {
  const patterns = [
    /https:\/\/teams\.microsoft\.com\/l\/meetup-join\/[^\s<>"]+/i,
    /https:\/\/teams\.live\.com\/meet\/[^\s<>"]+/i,
    /https:\/\/meet\.google\.com\/[a-z-]+/i,
    /https:\/\/[\w.-]*zoom\.us\/j\/[^\s<>"]+/i,
    /https:\/\/[\w.-]*webex\.com\/[^\s<>"]*meet[^\s<>"]*/i
  ]
  for (const f of fields) {
    if (!f) continue
    for (const p of patterns) {
      const m = p.exec(f)
      if (m) return m[0].replace(/[).,;]+$/, '')
    }
  }
  return ''
}

function hm(d: Date) {
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`
}

export function parseIcs(raw: string, rangeStartMs?: number, rangeEndMs?: number): Meeting[] {
  const now = Date.now()
  const winStart = rangeStartMs ?? now - 7 * 86_400_000
  const winEnd = rangeEndMs ?? now + 45 * 86_400_000

  const unfolded = raw.replace(/\r\n[ \t]/g, '').replace(/\n[ \t]/g, '')
  const lines = unfolded.split(/\r?\n/)

  const out: Meeting[] = []
  let inEvent = false
  let uid = ''
  let summary = ''
  let dtstartRaw = ''
  let dtendRaw = ''
  let rrule = ''
  let teamsUrl = ''
  let url = ''
  let location = ''
  let desc = ''
  let exdates = new Set<string>()

  const flush = () => {
    if (!dtstartRaw || !summary) return
    const st = parseIcsDate(dtstartRaw)
    if (!st) return
    const en = dtendRaw ? parseIcsDate(dtendRaw) : null
    const dur = en ? Math.max(0, en.date.getTime() - st.date.getTime()) : 30 * 60_000
    const joinUrl = extractJoinUrl([teamsUrl, url, location, desc])
    const baseUid = uid || `${summary}-${st.date.getTime()}`

    const pushOcc = (startDate: Date) => {
      const sMs = startDate.getTime()
      const eMs = sMs + dur
      if (eMs < winStart || sMs > winEnd) return
      if (exdates.has(ymd(startDate))) return
      const endDate = new Date(eMs)
      out.push({
        id: `${baseUid}_${sMs}`,
        date: ymd(startDate),
        startTime: st.allDay ? '00:00' : hm(startDate),
        endTime: st.allDay ? '23:59' : hm(endDate),
        startMs: sMs,
        endMs: eMs,
        summary,
        joinUrl,
        allDay: st.allDay
      })
    }

    if (!rrule) {
      pushOcc(st.date)
      return
    }

    const parts = Object.fromEntries(
      rrule.split(';').map(p => p.split('=')).filter(a => a.length === 2)
    ) as Record<string, string>
    const freq = parts.FREQ
    const interval = Math.max(1, parseInt(parts.INTERVAL || '1', 10) || 1)
    const count = parts.COUNT ? parseInt(parts.COUNT, 10) : Infinity
    const until = parts.UNTIL ? parseIcsDate(parts.UNTIL)?.date.getTime() ?? Infinity : Infinity
    const maxEnd = Math.min(until, winEnd)

    if (freq === 'DAILY') {
      let cur = new Date(st.date)
      let n = 0
      while (cur.getTime() <= maxEnd && n < count && n < 400) {
        pushOcc(cur)
        n++
        cur = new Date(cur)
        cur.setDate(cur.getDate() + interval)
      }
    } else if (freq === 'WEEKLY') {
      const byDay = (parts.BYDAY || '').split(',').map(d => DAY_CODES[d.trim()]).filter(d => d !== undefined)
      const days = byDay.length ? byDay : [st.date.getDay()]
      const weekAnchor = new Date(st.date)
      weekAnchor.setDate(weekAnchor.getDate() - weekAnchor.getDay())
      let n = 0
      for (let w = 0; w < 100 && n < count; w += interval) {
        for (const wd of days.slice().sort((a, b) => a - b)) {
          const occ = new Date(weekAnchor)
          occ.setDate(weekAnchor.getDate() + w * 7 + wd)
          occ.setHours(st.date.getHours(), st.date.getMinutes(), st.date.getSeconds(), 0)
          if (occ.getTime() < st.date.getTime()) continue
          if (occ.getTime() > maxEnd || n >= count) break
          pushOcc(occ)
          n++
        }
        const nextWeek = new Date(weekAnchor)
        nextWeek.setDate(weekAnchor.getDate() + (w + interval) * 7)
        if (nextWeek.getTime() > maxEnd) break
      }
    } else {
      pushOcc(st.date)
    }
  }

  for (const line of lines) {
    if (line === 'BEGIN:VEVENT') {
      inEvent = true
      uid = ''; summary = ''; dtstartRaw = ''; dtendRaw = ''; rrule = ''
      teamsUrl = ''; url = ''; location = ''; desc = ''; exdates = new Set()
      continue
    }
    if (line === 'END:VEVENT') {
      if (inEvent) flush()
      inEvent = false
      continue
    }
    if (!inEvent) continue
    const idx = line.indexOf(':')
    if (idx < 0) continue
    const keyPart = line.slice(0, idx).toUpperCase()
    const key = keyPart.split(';')[0]
    const val = line.slice(idx + 1)
    if (key === 'UID') uid = val.trim()
    else if (key === 'SUMMARY') summary = unescapeIcs(val)
    else if (key === 'DTSTART') dtstartRaw = val.trim()
    else if (key === 'DTEND') dtendRaw = val.trim()
    else if (key === 'RRULE') rrule = val.trim().toUpperCase()
    else if (key === 'X-MICROSOFT-SKYPETEAMSMEETINGURL') teamsUrl = val.trim()
    else if (key === 'URL') url = val.trim()
    else if (key === 'LOCATION') location = unescapeIcs(val)
    else if (key === 'DESCRIPTION') desc = unescapeIcs(val)
    else if (key === 'EXDATE') {
      for (const item of val.split(',')) {
        const parsed = parseIcsDate(item)
        if (parsed) exdates.add(ymd(parsed.date))
      }
    }
  }

  return out.sort((a, b) => a.startMs - b.startMs)
}
