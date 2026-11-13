// Simulador usado quando a interface roda fora do Tauri (navegador comum),
// para desenvolver e testar o visual sem compilar o Rust.
type Cb = (p: unknown) => void
const listeners: Record<string, Cb[]> = {}
const emit = (ev: string, p: unknown) => (listeners[ev] || []).forEach(cb => cb(p))

let vol = { level: 45, mute: false, mic: 70, micMute: false }
let devices = {
  out: [
    { id: 'a', name: 'Alto-falantes (Realtek Audio)', def: false },
    { id: 'b', name: 'Fones de ouvido (JBL Tune 510BT)', def: true },
    { id: 'c', name: 'LG ULTRAGEAR (HDMI)', def: false }
  ],
  in: [
    { id: 'd', name: 'Microfone (Realtek Audio)', def: true },
    { id: 'e', name: 'Headset (JBL Tune 510BT)', def: false }
  ]
}
const today = new Date().toISOString().slice(0, 10)
let adapters = [
  { name: 'Wi-Fi', desc: 'Intel(R) Wi-Fi 6E AX211', up: true, ip: '192.168.0.12' },
  { name: 'Ethernet 2', desc: 'Fortinet SSL VPN Virtual Ethernet Adapter', up: true, ip: '10.70.4.21' },
  { name: 'FNDE', desc: 'WAN Miniport (IKEv2)', up: false, ip: '' },
  { name: 'Ethernet 3', desc: 'Cisco AnyConnect Secure Mobility Client Virtual Miniport Adapter', up: false, ip: '' }
]
const mr = (id: number, title: string, project: string, extra: object = {}) => ({
  id, iid: id % 1000, title, project, url: 'https://gitlab.example.gov.br/' + project + '/-/merge_requests/' + id,
  author: 'Maria Souza', updated: new Date(Date.now() - id * 6e5).toISOString(), draft: false, conflicts: false,
  pipeline: '', merge_status: 'mergeable', ...extra
})
const coverSvg = 'data:image/svg+xml;base64,' + btoa(
  '<svg xmlns="http://www.w3.org/2000/svg" width="120" height="120"><rect width="120" height="120" fill="#1e3c8c"/><circle cx="60" cy="60" r="40" fill="#f07828"/></svg>'
)

const procs = () => [
  ...Array.from({ length: 18 }, (_, i) => ({ n: 'chrome', p: 1000 + i, c: i < 2 ? 6.2 : 0.3, m: 1.4e8 })),
  { n: 'Code', p: 2001, c: 3.1, m: 6.2e8 }, { n: 'Code', p: 2002, c: 0.4, m: 3e8 },
  { n: 'Spotify', p: 3001, c: 1.2, m: 3.4e8 }, { n: 'Discord', p: 4001, c: 0.8, m: 4.1e8 },
  { n: 'explorer', p: 5001, c: 0.5, m: 1.2e8 }, { n: 'svchost', p: 600, c: 0.9, m: 2.5e8 },
  { n: 'dwm', p: 700, c: 2.4, m: 1.1e8 }, { n: 'MsMpEng', p: 800, c: 0.2, m: 2.9e8 },
  { n: 'ilha-dinamica', p: 9001, c: 0.3, m: 3.1e7 }, { n: 'Steam', p: 9100, c: 0.1, m: 2.2e8 }
]

const handlers: Record<string, (a: Record<string, unknown>) => unknown> = {
  get_data: () => ({
    notes: '', platform: 'windows',
    settings: {
      orgs: [
        { id: 'enap', name: 'ENAP', color: '#2fb8ff', vpn: { type: 'openvpnconnect', name: 'vpnclient.enap.gov.br', user: '', adapter: 'Fortinet', connect: '', disconnect: '' }, gitlab: { url: 'gitlab.enap.gov.br', insecure: false } },
        { id: 'fnde', name: 'FNDE', color: '#34d058', vpn: { type: 'windows', name: 'FNDE', user: '', adapter: '', connect: '', disconnect: '' }, gitlab: { url: 'gitlab.fnde.gov.br', insecure: false } },
        { id: 'basa', name: 'BASA', color: '#ffd23f', vpn: { type: 'cisco', name: 'vpn.basa.com.br/parceiros', user: '', adapter: 'Cisco', connect: '', disconnect: '' }, gitlab: { url: '', insecure: false } }
      ]
    }, isAdmin: false, autostart: false, version: '2.0.0',
    protectedNames: ['svchost', 'dwm', 'msmpeng', 'system'], selfName: 'ilha-dinamica', ownPids: [],
    reminders: [
      { id: 1, date: today, time: '23:30', text: 'Ligar para a Amanda', fired: false },
      { id: 2, date: today, time: '08:00', text: 'Academia', fired: true }
    ]
  }),
  set_vol: (a) => { vol = { ...vol, level: a.value as number, mute: false }; emit('vol', vol) },
  toggle_mute: () => { vol = { ...vol, mute: !vol.mute }; emit('vol', vol) },
  set_mic: (a) => { vol = { ...vol, mic: a.value as number }; emit('vol', vol) },
  toggle_mic: () => { vol = { ...vol, micMute: !vol.micMute }; emit('vol', vol) },
  set_device: (a) => {
    const isOut = devices.out.some(d => d.id === a.id)
    const k = isOut ? 'out' : 'in'
    devices = { ...devices, [k]: devices[k].map(d => ({ ...d, def: d.id === a.id })) }
    emit('devices', devices)
  },
  refresh_devices: () => emit('devices', devices),
  get_brightness: () => null,
  get_procs: () => procs(),
  kill_process: () => ({ ok: true, protected: false }),
  clean_scan: async () => {
    await new Promise(r => setTimeout(r, 300))
    return {
      disk: { free: 118e9, total: 476e9 },
      order: ['temp', 'browsers', 'apps', 'thumbs', 'crash', 'wintemp', 'update', 'recycle'],
      cats: {
        temp: { name: 'Arquivos temporários', icon: '🗂️', admin: false, size: 1.84e9, count: 812, denied: false },
        browsers: { name: 'Cache dos navegadores', icon: '🌐', admin: false, size: 2.3e9, count: 40, denied: false },
        apps: { name: 'Cache de apps (Discord, VS Code)', icon: '💬', admin: false, size: 3.1e8, count: 9, denied: false },
        thumbs: { name: 'Miniaturas e ícones', icon: '🖼️', admin: false, size: 9.2e7, count: 12, denied: false },
        crash: { name: 'Relatórios de erro', icon: '💥', admin: false, size: 4.5e7, count: 3, denied: false },
        wintemp: { name: 'Temp do Windows', icon: '🪟', admin: true, size: 0, count: 0, denied: true },
        update: { name: 'Cache do Windows Update', icon: '🔄', admin: true, size: 0, count: 0, denied: true },
        recycle: { name: 'Lixeira', icon: '🗑️', admin: false, size: 6.4e8, count: 25, denied: false }
      }
    }
  },
  clean_run: () => ({ freed: 4.1e9, disk: { free: 122e9, total: 476e9 } }),
  trim_ram: () => ({ freed: 8e8, n: 120 }),
  flush_dns: () => true,
  set_autostart: (a) => a.enabled,
  pick_path: () => null,
  relaunch_admin: () => false,
  list_adapters: () => adapters,
  vpn_connect: (a) => {
    const p = a.profile as { type: string; name: string }
    adapters = adapters.map(x => (p.type === 'windows' && x.name === p.name) || (p.type === 'cisco' && x.desc.includes('Cisco')) ? { ...x, up: true, ip: '10.9.0.5' } : x)
    setTimeout(() => emit('adapters', adapters), 300)
    return { ok: true, mode: p.type === 'cisco' ? 'app' : 'cmd', msg: p.type === 'cisco' ? 'Abrimos o Cisco Secure Client para você conectar' : '' }
  },
  vpn_disconnect: (a) => {
    const p = a.profile as { type: string; name: string }
    adapters = adapters.map(x => (p.type === 'windows' && x.name === p.name) || (p.type === 'forti' && x.desc.includes('Fortinet')) || (p.type === 'cisco' && x.desc.includes('Cisco')) ? { ...x, up: false, ip: '' } : x)
    setTimeout(() => emit('adapters', adapters), 300)
    return { ok: true, mode: 'cmd', msg: '' }
  },
  probe_host: () => ({ ok: true, ms: 38 }),
  vpn_set_password: (a) => !!a.password,
  vpn_has_passwords: () => [],
  games_list: () => [
    { id: 'steam-730', name: 'Counter-Strike 2', source: 'steam', launch: 'steam://rungameid/730', image: '', appid: 730, last_played: 1763050000, size: 3.4e10 },
    { id: 'steam-570', name: 'Dota 2', source: 'steam', launch: 'steam://rungameid/570', image: '', appid: 570, last_played: 1762900000, size: 4e10 },
    { id: 'steam-1086940', name: "Baldur's Gate 3", source: 'steam', launch: 'steam://rungameid/1086940', image: '', appid: 1086940, last_played: 1762000000, size: 1.5e11 },
    { id: 'steam-1172470', name: 'Apex Legends', source: 'steam', launch: 'steam://rungameid/1172470', image: '', appid: 1172470, last_played: 1761000000, size: 7e10 },
    { id: 'epic-Fortnite', name: 'Fortnite', source: 'epic', launch: '', image: '', appid: 0, last_played: 0, size: 3e10 },
    { id: 'epic-RocketLeague', name: 'Rocket League', source: 'epic', launch: '', image: '', appid: 0, last_played: 0, size: 2e10 },
    { id: 'gog-1', name: 'The Witcher 3: Wild Hunt', source: 'gog', launch: '', image: '', appid: 0, last_played: 0, size: 5e10 }
  ],
  steam_info: () => ({ installed: true, steamid: '76561198000000001', persona: 'Josueh', has_key: true }),
  steam_set_key: (a) => !!a.key,
  steam_friends: () => [
    { steamid: '1', name: 'Marcão', avatar: '', state: 1, game: 'Counter-Strike 2', gameid: '730', lobby: '109775241000000000' },
    { steamid: '2', name: 'Ana Clara', avatar: '', state: 1, game: 'Dota 2', gameid: '570', lobby: '' },
    { steamid: '3', name: 'Rafa_BR', avatar: '', state: 1, game: 'Elden Ring', gameid: '1245620', lobby: '' },
    { steamid: '4', name: 'dudu', avatar: '', state: 3, game: '', gameid: '', lobby: '' },
    { steamid: '5', name: 'Pedro H.', avatar: '', state: 1, game: '', gameid: '', lobby: '' }
  ],
  gitlab_set_token: (a) => !!String(a.token).trim(),
  gitlab_has_tokens: () => ['enap', 'fnde'],
  gitlab_mrs: (a) => {
    if (a.org === 'enap') return {
      user: 'josueh',
      review: [mr(101, 'feat: integra SSO gov.br no portal de cursos', 'enap/escola-virtual/portal'), mr(102, 'fix: timeout na emissão de certificados', 'enap/certificados/api')],
      mine: [mr(201, 'refactor: separa módulo de matrículas (ADR-014)', 'enap/escola-virtual/api', { pipeline: 'success' })]
    }
    return {
      user: 'josueh',
      review: [mr(301, 'feat: novo cálculo de repasse PNAE', 'fnde/sigpc/repasse')],
      mine: [mr(401, 'chore: atualiza Spring Boot 3.5', 'fnde/sigpc/backend', { pipeline: 'failed' }), mr(402, 'docs: C4 do barramento de eventos', 'fnde/arquitetura/docs', { pipeline: 'running', draft: true })]
    }
  },
  check_update: () => ({ available: true, version: '2.2.0', current: '2.1.0', notes: '' }),
  install_update: () => undefined,
  reset_position: () => emit('anchor', { x: 'center', y: 'top' })
}

export const mock = {
  async invoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
    const h = handlers[cmd]
    return (h ? await h(args) : undefined) as T
  },
  listen(ev: string, cb: Cb) {
    (listeners[ev] = listeners[ev] || []).push(cb)
    // dados iniciais de exemplo
    setTimeout(() => {
      if (ev === 'clips') cb([
        { text: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJqb3N1ZWgiLCJvcmciOiJGTkRFIiwiZXhwIjo0MTAyNDQ0ODAwfQ.c2lnbmF0dXJl', t: Date.now() },
        { text: 'Reunião remarcada para 15h', t: Date.now() - 30000 },
        { text: 'hello@example.com', t: Date.now() - 120000 },
        { text: 'https://github.com/JosuehElelyon/ilha-dinamica', t: Date.now() - 4e6 }
      ])
      if (ev === 'media') cb({ title: 'Golden Hour', artist: 'Nova Lane', status: 'Playing', app: 'Spotify', cover: coverSvg })
      if (ev === 'vol') cb(vol)
      if (ev === 'devices') cb(devices)
      if (ev === 'sys') cb({ cpu: 23, memUsed: 9e9, memTotal: 16e9, uptime: 18000, cpuName: 'Intel Core i7', battery: { level: 76, charging: false } })
    }, 60)
  },
  emit
}
