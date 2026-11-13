// Trabalho: estado das VPNs e Merge Requests de cada órgão/cliente
import { reactive } from 'vue'
import { api } from '../lib/api'
import type { Adapter, MrList, Org } from '../lib/types'
import { S } from './settings'
import { alertIsland, showToast } from './state'

export const VPN_TYPES: { id: Org['vpn']['type']; label: string; nameLabel: string; namePh: string; adapter: string }[] = [
  { id: 'none', label: 'Sem VPN', nameLabel: '', namePh: '', adapter: '' },
  { id: 'openvpnconnect', label: 'OpenVPN Connect', nameLabel: 'Perfil (nome ou servidor)', namePh: 'vpnclient.orgao.gov.br', adapter: 'OpenVPN' },
  { id: 'cisco', label: 'Cisco Secure Client / AnyConnect', nameLabel: 'Servidor', namePh: 'vpn.orgao.gov.br/grupo', adapter: 'Cisco' },
  { id: 'forti', label: 'FortiClient', nameLabel: 'Servidor (opcional)', namePh: 'vpn.orgao.gov.br', adapter: 'Fortinet' },
  { id: 'globalprotect', label: 'GlobalProtect (Palo Alto)', nameLabel: 'Portal', namePh: 'vpn.orgao.gov.br', adapter: 'PANGP' },
  { id: 'windows', label: 'VPN do Windows', nameLabel: 'Nome da conexão', namePh: 'Como aparece em Configurações → VPN', adapter: '' },
  { id: 'openvpn', label: 'OpenVPN GUI (comunidade)', nameLabel: 'Arquivo de config', namePh: 'orgao (sem .ovpn)', adapter: 'OpenVPN' },
  { id: 'wireguard', label: 'WireGuard', nameLabel: 'Túnel (.conf)', namePh: 'C:\\VPN\\orgao.conf', adapter: '' },
  { id: 'custom', label: 'Comandos personalizados', nameLabel: '', namePh: '', adapter: '' }
]

export interface OrgStatus {
  vpnUp: boolean
  ip: string
  busy: boolean
  mrs: MrList | null
  mrError: string
  mrLoading: boolean
  hasToken: boolean
  hasPassword: boolean
}

export const work = reactive({
  adapters: [] as Adapter[],
  status: {} as Record<string, OrgStatus>
})

export function st(id: string): OrgStatus {
  if (!work.status[id]) {
    work.status[id] = { vpnUp: false, ip: '', busy: false, mrs: null, mrError: '', mrLoading: false, hasToken: false, hasPassword: false }
  }
  return work.status[id]
}

/** Palavra que identifica a placa de rede da VPN (pode ter alternativas separadas por |) */
export function adapterKey(o: Org) {
  if (o.vpn.adapter.trim()) return o.vpn.adapter.trim()
  if (o.vpn.type === 'windows') return o.vpn.name
  if (o.vpn.type === 'wireguard') return (o.vpn.name.split(/[\\/]/).pop() || '').replace(/\.conf$/i, '')
  return VPN_TYPES.find(t => t.id === o.vpn.type)?.adapter || ''
}

function match(o: Org): Adapter | undefined {
  const keys = adapterKey(o).toLowerCase().split('|').map(k => k.trim()).filter(Boolean)
  if (!keys.length || o.vpn.type === 'none') return undefined
  return work.adapters.find(a => a.up && keys.some(k => a.name.toLowerCase().includes(k) || a.desc.toLowerCase().includes(k)))
}

let firstAdapters = true
function applyAdapters(list: Adapter[]) {
  work.adapters = list
  for (const o of S.orgs) {
    const s = st(o.id)
    const a = match(o)
    const up = !!a
    if (up !== s.vpnUp && !firstAdapters) {
      showToast({ icon: up ? '🔐' : '🔓', text: `VPN ${o.name} ${up ? 'conectada' : 'desconectada'}`, ms: 2500 })
      if (up && o.gitlab.url) setTimeout(() => loadMrs(o), 3000)
    }
    s.vpnUp = up
    s.ip = a?.ip || ''
  }
  firstAdapters = false
}

export async function toggleVpn(o: Org) {
  const s = st(o.id)
  if (o.vpn.type === 'none') return
  s.busy = true
  try {
    if (s.vpnUp) {
      const r = await api.vpnDisconnect(o.id, o.vpn)
      if (r.mode === 'app') showToast({ icon: '🔓', text: r.msg || `Desconecte pelo app da VPN`, ms: 3500 })
      else if (!r.ok) showToast({ icon: '⚠️', text: r.msg, ms: 3500 })
    } else {
      if (S.vpnExclusive) {
        for (const other of S.orgs) if (other.id !== o.id && st(other.id).vpnUp) await api.vpnDisconnect(other.id, other.vpn)
      }
      showToast({ icon: '🔐', text: `Conectando VPN ${o.name}…`, ms: 4000 })
      const r = await api.vpnConnect(o.id, o.vpn)
      if (r.mode === 'app') showToast({ icon: '🔐', text: r.msg || 'Conecte pelo app da VPN', ms: 4000 })
      else if (!r.ok) showToast({ icon: '⚠️', text: r.msg || 'Falha ao conectar', alert: true, ms: 4000 })
    }
  } finally {
    // o status real vem das placas de rede; dá um tempo para a VPN subir
    setTimeout(() => (s.busy = false), 4000)
  }
}

/* ---------------- GitLab ---------------- */

const seenReview: Record<string, Set<number>> = {}
const pipelines: Record<string, Record<number, string>> = {}

export async function loadMrs(o: Org, silent = false) {
  const s = st(o.id)
  if (!o.gitlab.url || !s.hasToken) return
  s.mrLoading = true
  try {
    const r = await api.gitlabMrs(o.id, o.gitlab.url, o.gitlab.insecure)
    s.mrError = ''
    // avisos: nova revisão pedida e pipeline que falhou
    const seen = seenReview[o.id]
    if (seen && !silent) {
      for (const m of r.review) if (!seen.has(m.id)) alertIsland(`Revisão pedida · ${o.name}`, m.title, '🔀')
      for (const m of r.mine) {
        const before = pipelines[o.id]?.[m.id]
        if (before && before !== m.pipeline && m.pipeline === 'failed') alertIsland(`Pipeline falhou · ${o.name}`, m.title, '❌')
        if (before && before !== m.pipeline && m.pipeline === 'success') showToast({ icon: '✅', text: `Pipeline ok: ${m.title}`, ms: 3500 })
      }
    }
    seenReview[o.id] = new Set(r.review.map(m => m.id))
    pipelines[o.id] = Object.fromEntries(r.mine.map(m => [m.id, m.pipeline]))
    s.mrs = r
  } catch (e) {
    s.mrError = String(e)
  } finally {
    s.mrLoading = false
  }
}

export async function refreshSecrets() {
  const ids = S.orgs.map(o => o.id)
  const [tokens, pws] = await Promise.all([api.gitlabHasTokens(ids), api.vpnHasPasswords(ids)])
  for (const o of S.orgs) {
    st(o.id).hasToken = tokens.includes(o.id)
    st(o.id).hasPassword = pws.includes(o.id)
  }
}

export async function initWork() {
  api.onAdapters(applyAdapters)
  applyAdapters(await api.listAdapters())
  await refreshSecrets()
  const all = () => S.orgs.forEach(o => loadMrs(o))
  all()
  setInterval(all, 3 * 60 * 1000)
}

export const pipeIcon = (p: string) =>
  ({ success: '✅', failed: '❌', running: '⏳', pending: '⏳', created: '⏳', canceled: '⛔', manual: '▶️', skipped: '⏭' } as Record<string, string>)[p] || ''
