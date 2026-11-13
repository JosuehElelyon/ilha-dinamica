// Caixa de ferramentas de dev: funções puras usadas pela aba "Ferramentas"
// e pela detecção automática no clipboard.

export type ToolId =
  | 'json' | 'jwt' | 'b64enc' | 'b64dec' | 'urlenc' | 'urldec'
  | 'ts' | 'uuid' | 'sha256' | 'case' | 'count'

export interface ToolResult { ok: boolean; out: string; info?: string }

export const TOOLS: { id: ToolId; label: string; hint: string }[] = [
  { id: 'json', label: 'JSON', hint: 'Formatar e validar JSON' },
  { id: 'jwt', label: 'JWT', hint: 'Decodificar token JWT (header, payload e validade)' },
  { id: 'b64dec', label: 'Base64 →', hint: 'Decodificar Base64' },
  { id: 'b64enc', label: '→ Base64', hint: 'Codificar em Base64' },
  { id: 'urldec', label: 'URL →', hint: 'Decodificar URL (%20 → espaço)' },
  { id: 'urlenc', label: '→ URL', hint: 'Codificar para URL' },
  { id: 'ts', label: 'Data/Epoch', hint: 'Timestamp Unix ↔ data' },
  { id: 'uuid', label: 'UUID', hint: 'Gerar UUIDs v4' },
  { id: 'sha256', label: 'SHA-256', hint: 'Hash SHA-256 / SHA-1' },
  { id: 'case', label: 'Case', hint: 'camelCase, snake_case, kebab-case, PascalCase…' },
  { id: 'count', label: 'Contar', hint: 'Caracteres, palavras, linhas e bytes' }
]

const utf8 = new TextEncoder()
const fromUtf8 = new TextDecoder('utf-8', { fatal: true })

function b64ToBytes(s: string) {
  let t = s.trim().replace(/\s+/g, '').replace(/-/g, '+').replace(/_/g, '/')
  while (t.length % 4) t += '='
  const bin = atob(t)
  return Uint8Array.from(bin, c => c.charCodeAt(0))
}
function bytesToB64(b: Uint8Array) {
  let bin = ''
  b.forEach(x => (bin += String.fromCharCode(x)))
  return btoa(bin)
}

const pretty = (v: unknown) => JSON.stringify(v, null, 2)

function fmtDate(d: Date) {
  return d.toLocaleString('pt-BR', { dateStyle: 'short', timeStyle: 'medium' })
}

function relative(ms: number) {
  const s = Math.round(ms / 1000), a = Math.abs(s)
  const txt = a < 60 ? `${a}s` : a < 3600 ? `${Math.round(a / 60)} min` : a < 86400 ? `${Math.round(a / 3600)} h` : `${Math.round(a / 86400)} dias`
  return s >= 0 ? `daqui a ${txt}` : `há ${txt}`
}

/* ---------------- detecção ---------------- */

const JWT_RE = /^[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]*$/
const UUID_RE = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

/** Adivinha a ferramenta mais útil para o texto (ou null) */
export function detect(text: string): ToolId | null {
  const t = text.trim()
  if (!t) return null
  if (JWT_RE.test(t.replace(/^Bearer\s+/i, ''))) return 'jwt'
  if (/^[[{]/.test(t)) { try { JSON.parse(t); return 'json' } catch { /* não é JSON */ } }
  if (/^\d{10}(\d{3})?$/.test(t)) return 'ts'
  if (/%[0-9A-F]{2}/i.test(t) && !/\s/.test(t)) return 'urldec'
  if (t.length >= 12 && t.length % 4 === 0 && /^[A-Za-z0-9+/]+={0,2}$/.test(t) && /[a-z]/.test(t) && /[A-Z0-9]/.test(t)) {
    try { fromUtf8.decode(b64ToBytes(t)); return 'b64dec' } catch { /* binário */ }
  }
  return null
}

export const DETECT_LABEL: Partial<Record<ToolId, string>> = {
  jwt: 'Decodificar JWT', json: 'Formatar JSON', ts: 'Converter data', urldec: 'Decodificar URL', b64dec: 'Decodificar Base64'
}

/* ---------------- ferramentas ---------------- */

function decodeJwt(input: string): ToolResult {
  const t = input.trim().replace(/^Bearer\s+/i, '')
  const parts = t.split('.')
  if (parts.length < 2) return { ok: false, out: 'Não parece um JWT (esperado: header.payload.assinatura)' }
  try {
    const header = JSON.parse(fromUtf8.decode(b64ToBytes(parts[0])))
    const payload = JSON.parse(fromUtf8.decode(b64ToBytes(parts[1])))
    const lines: string[] = []
    const now = Date.now()
    for (const k of ['iat', 'nbf', 'exp'] as const) {
      if (typeof payload[k] === 'number') {
        const d = new Date(payload[k] * 1000)
        lines.push(`${k}: ${fmtDate(d)} (${relative(d.getTime() - now)})`)
      }
    }
    let info = 'Sem data de expiração'
    if (typeof payload.exp === 'number') info = payload.exp * 1000 < now ? '⛔ Token EXPIRADO' : '✅ Token válido (pela data)'
    const out = `// HEADER\n${pretty(header)}\n\n// PAYLOAD\n${pretty(payload)}` + (lines.length ? `\n\n// DATAS\n${lines.join('\n')}` : '')
    return { ok: true, out, info: `${info} · alg ${header.alg ?? '?'} · assinatura não verificada` }
  } catch {
    return { ok: false, out: 'JWT inválido: não foi possível decodificar o header/payload' }
  }
}

function json(input: string, minify: boolean): ToolResult {
  try {
    const v = JSON.parse(input)
    const keys = typeof v === 'object' && v ? Object.keys(v).length : 0
    return { ok: true, out: minify ? JSON.stringify(v) : pretty(v), info: `✅ JSON válido · ${Array.isArray(v) ? `${v.length} itens` : `${keys} chaves`}` }
  } catch (e) {
    const msg = (e as Error).message
    const m = /position (\d+)/.exec(msg)
    let where = ''
    if (m) {
      const pos = +m[1], before = input.slice(0, pos)
      where = ` (linha ${before.split('\n').length}, coluna ${pos - before.lastIndexOf('\n')})`
    }
    return { ok: false, out: `❌ JSON inválido${where}\n${msg}` }
  }
}

function timestamp(input: string): ToolResult {
  const t = input.trim()
  if (!t) {
    const n = Date.now()
    return { ok: true, out: `Agora\nsegundos:      ${Math.floor(n / 1000)}\nmilissegundos: ${n}\nISO 8601:      ${new Date(n).toISOString()}\nlocal:         ${fmtDate(new Date(n))}` }
  }
  let d: Date
  if (/^-?\d+(\.\d+)?$/.test(t)) {
    const n = Number(t)
    d = new Date(t.replace('-', '').split('.')[0].length > 11 ? n : n * 1000)
  } else d = new Date(t)
  if (isNaN(d.getTime())) return { ok: false, out: 'Não reconheci como data nem timestamp.\nExemplos: 1763064000 · 1763064000000 · 2026-11-13T17:00:00Z' }
  return {
    ok: true,
    out: `local:         ${fmtDate(d)}\nISO 8601:      ${d.toISOString()}\nsegundos:      ${Math.floor(d.getTime() / 1000)}\nmilissegundos: ${d.getTime()}\nrelativo:      ${relative(d.getTime() - Date.now())}`
  }
}

function words(s: string) {
  return s
    .replace(/([a-z0-9])([A-Z])/g, '$1 $2')
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')
    .normalize('NFD').replace(/[̀-ͯ]/g, '')
    .split(/[^A-Za-z0-9]+/).filter(Boolean).map(w => w.toLowerCase())
}
function cases(input: string): ToolResult {
  const w = words(input)
  if (!w.length) return { ok: false, out: 'Digite um texto (ex.: nome do serviço de pagamentos)' }
  const cap = (x: string) => x[0].toUpperCase() + x.slice(1)
  return {
    ok: true,
    out: [
      `camelCase:      ${w[0] + w.slice(1).map(cap).join('')}`,
      `PascalCase:     ${w.map(cap).join('')}`,
      `snake_case:     ${w.join('_')}`,
      `CONSTANT_CASE:  ${w.join('_').toUpperCase()}`,
      `kebab-case:     ${w.join('-')}`,
      `dot.case:       ${w.join('.')}`,
      `Título:         ${w.map(cap).join(' ')}`
    ].join('\n')
  }
}

function count(input: string): ToolResult {
  const lines = input ? input.split(/\r?\n/).length : 0
  const wordsN = (input.trim().match(/\S+/g) || []).length
  return { ok: true, out: `caracteres: ${[...input].length}\nsem espaços: ${input.replace(/\s/g, '').length}\npalavras:   ${wordsN}\nlinhas:     ${lines}\nbytes UTF-8: ${utf8.encode(input).length}` }
}

async function hash(input: string): Promise<ToolResult> {
  const hex = async (alg: string) =>
    [...new Uint8Array(await crypto.subtle.digest(alg, utf8.encode(input)))].map(b => b.toString(16).padStart(2, '0')).join('')
  return { ok: true, out: `SHA-256: ${await hex('SHA-256')}\nSHA-1:   ${await hex('SHA-1')}\nSHA-512: ${await hex('SHA-512')}` }
}

export async function run(tool: ToolId, input: string, opts: { minify?: boolean; count?: number } = {}): Promise<ToolResult> {
  switch (tool) {
    case 'json': return json(input, !!opts.minify)
    case 'jwt': return decodeJwt(input)
    case 'b64enc': return { ok: true, out: bytesToB64(utf8.encode(input)) }
    case 'b64dec':
      try { return { ok: true, out: fromUtf8.decode(b64ToBytes(input)) } } catch { return { ok: false, out: 'Base64 inválido (ou o conteúdo não é texto)' } }
    case 'urlenc': return { ok: true, out: encodeURIComponent(input) }
    case 'urldec':
      try { return { ok: true, out: decodeURIComponent(input.replace(/\+/g, ' ')) } } catch { return { ok: false, out: 'URL com codificação inválida' } }
    case 'ts': return timestamp(input)
    case 'uuid': return { ok: true, out: Array.from({ length: opts.count || 5 }, () => crypto.randomUUID()).join('\n'), info: UUID_RE.test(input.trim()) ? 'O texto de entrada já é um UUID válido' : undefined }
    case 'sha256': return hash(input)
    case 'case': return cases(input)
    case 'count': return count(input)
  }
}
