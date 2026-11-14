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
  { name: 'Empresa A', desc: 'WAN Miniport (IKEv2)', up: false, ip: '' },
  { name: 'Ethernet 3', desc: 'Cisco AnyConnect Secure Mobility Client Virtual Miniport Adapter', up: false, ip: '' }
]
const mr = (id: number, title: string, project: string, extra: object = {}) => ({
  id, iid: id % 1000, title, project, url: 'https://gitlab.example.com/' + project + '/-/merge_requests/' + id,
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
        { id: 'org1', name: 'Empresa A', color: '#2fb8ff', vpn: { type: 'openvpnconnect', name: 'vpn.exemplo.com', user: '', adapter: 'Fortinet', connect: '', disconnect: '' }, gitlab: { url: 'gitlab.exemplo.com', insecure: false } },
        { id: 'org2', name: 'Cliente B', color: '#34d058', vpn: { type: 'windows', name: 'Empresa A', user: '', adapter: '', connect: '', disconnect: '' }, gitlab: { url: 'git.cliente.com', insecure: false } }
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
  open_in_antigravity: () => true,
  open_terminal: () => true,
  git_status: (a) => ((a.paths as string[]) || []).map((path, i) => ({
    path, is_git: true, branch: i % 2 === 0 ? 'main' : 'feat/ajuste-api', dirty: i % 2 === 0 ? 0 : 2
  })),
  fetch_ics: () => {
    const d = new Date()
    const pad2 = (n: number) => String(n).padStart(2, '0')
    const ds = `${d.getFullYear()}${pad2(d.getMonth() + 1)}${pad2(d.getDate())}`
    return [
      'BEGIN:VCALENDAR',
      'VERSION:2.0',
      'BEGIN:VEVENT',
      'UID:mock-teams-1',
      `DTSTART:${ds}T150000`,
      `DTEND:${ds}T153000`,
      'SUMMARY:Daily — Alinhamento Técnico',
      'LOCATION:Microsoft Teams Meeting',
      'X-MICROSOFT-SKYPETEAMSMEETINGURL:https://teams.microsoft.com/l/meetup-join/mock',
      'END:VEVENT',
      'END:VCALENDAR'
    ].join('\r\n')
  },
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
  gitlab_has_tokens: () => ['org1', 'org2'],
  gitlab_mrs: (a) => {
    if (a.org === 'org1') return {
      user: 'usuario',
      review: [mr(101, 'feat: integra SSO no portal', 'empresa-a/portal'), mr(102, 'fix: timeout na emissão de relatórios', 'empresa-a/api')],
      mine: [mr(201, 'refactor: separa módulo de cadastros (ADR-014)', 'empresa-a/api', { pipeline: 'success' })]
    }
    return {
      user: 'usuario',
      review: [mr(301, 'feat: novo cálculo de repasse', 'cliente-b/repasse')],
      mine: [mr(401, 'chore: atualiza Spring Boot 3.5', 'cliente-b/backend', { pipeline: 'failed' }), mr(402, 'docs: C4 do barramento de eventos', 'cliente-b/docs', { pipeline: 'running', draft: true })]
    }
  },
  wifi_status: () => ({ connected: true, ssid: 'Casa_5G', signal: 88 }),
  wifi_scan: () => [
    { ssid: 'Casa_5G', signal: 88, auth: 'WPA2-Personal', connected: true },
    { ssid: 'Casa_2.4G', signal: 74, auth: 'WPA2-Personal', connected: false },
    { ssid: 'Vizinho_Fibra', signal: 42, auth: 'WPA2-Personal', connected: false }
  ],
  wifi_connect: (a) => `Conectando a ${String(a.ssid)}…`,
  wifi_disconnect: () => true,
  bt_state: () => ({
    available: true,
    enabled: true,
    devices: [
      { id: 'bt-1', name: 'Galaxy Buds2 Pro', connected: true, kind: 'BT' },
      { id: 'bt-2', name: 'MX Master 3S', connected: true, kind: 'LE' },
      { id: 'bt-3', name: 'JBL Flip 6', connected: false, kind: 'BT' }
    ]
  }),
  bt_toggle_radio: (a) => !!a.enable,
  assistant_snapshot: () => mockAssistantSnapshot(),
  assistant_google_connect: () => {
    mockGoogleConnected = true
    return mockAssistantSnapshot()
  },
  assistant_google_sync: () => mockAssistantSnapshot(),
  assistant_google_disconnect: () => {
    mockGoogleConnected = false
    return mockAssistantSnapshot()
  },
  assistant_sync_windows: () => {
    mockWinConnected = true
    return mockAssistantSnapshot()
  },
  assistant_github_connect: () => {
    mockGithubConnected = true
    return mockAssistantSnapshot()
  },
  assistant_apple_connect: () => {
    mockAppleConnected = true
    return mockAssistantSnapshot()
  },
  assistant_disconnect_account: (a) => {
    const id = String(a.accountId || '')
    if (id.startsWith('google:')) mockGoogleConnected = false
    if (id.startsWith('github:')) mockGithubConnected = false
    if (id.startsWith('apple:')) mockAppleConnected = false
    return mockAssistantSnapshot()
  },
  assistant_add_important_date: (a) => {
    mockDates.unshift({
      id: 'custom-' + Date.now(),
      title: a.kind === 'birthday' ? `Aniversário de ${String(a.name)}` : String(a.name),
      personName: String(a.name),
      kind: String(a.kind),
      month: Number(a.month),
      day: Number(a.day),
      daysUntil: 0,
      turningAge: a.yearOptional ? new Date().getFullYear() - Number(a.yearOptional) : null,
      sourceProvider: 'local'
    })
    return mockAssistantSnapshot()
  },
  assistant_delete_important_date: (a) => {
    mockDates = mockDates.filter(d => d.id !== String(a.id))
    return mockAssistantSnapshot()
  },
  assistant_github_device_start: () => ({
    deviceCode: 'dev_mock_123',
    userCode: 'ILHA-2026',
    verificationUri: 'https://github.com/login/device',
    expiresIn: 900,
    interval: 5
  }),
  assistant_github_device_poll: () => {
    mockGithubConnected = true
    return mockAssistantSnapshot()
  },
  phone_pairing_info: () => ({
    qrUri: `ilha://pair?v=1&host=192.168.0.15&port=47821&token=${mockPairToken}&pc=DESKTOP-JOSUEH`,
    companionUrl: `http://192.168.0.15:47821/?token=${mockPairToken}`,
    webhookUrl: `http://192.168.0.15:47821/api/packet?token=${mockPairToken}`,
    host: '192.168.0.15',
    port: 47821,
    token: mockPairToken,
    adbAvailable: true,
    adbDevices: ['R58X30ABCD'],
    btPairedPhones: ['Galaxy S24 Ultra (conectado)', 'iPhone de Josueh']
  }),
  phone_rotate_token: () => {
    mockPairToken = 'tok_' + Math.random().toString(36).slice(2, 12)
    return handlers.phone_pairing_info({})
  },
  phone_enable_usb_bridge: () => ['R58X30ABCD'],
  phone_ingest_demo_signal: (a) => {
    const kind = String(a.kind || 'notification')
    const title = String(a.title || 'Nova mensagem')
    const body = String(a.body || '')
    const tr = String(a.transport || 'lan') as 'lan' | 'usb' | 'bluetooth'
    mockPhone.activeTransport = tr
    if (kind === 'battery') mockPhone.batteryPct = Number(title) || 12
    else {
      mockSignals.unshift({
        id: 'sig-' + Date.now(),
        deviceId: mockPhone.id,
        kind,
        title: kind === 'call' ? title : title,
        body,
        appName: kind === 'call' ? 'Telefone' : 'WhatsApp',
        ts: Math.floor(Date.now() / 1000)
      })
    }
    return mockAssistantSnapshot()
  },
  check_update: () => ({ available: true, version: '2.2.0', current: '2.1.0', notes: '' }),
  install_update: () => undefined,
  reset_position: () => emit('anchor', { x: 'center', y: 'top' })
}

let mockGoogleConnected = true
let mockWinConnected = true
let mockGithubConnected = true
let mockAppleConnected = false
let mockPairToken = '9f4a21c7e8b3d019a52c81ef'

const curDate = new Date()
const curM = curDate.getMonth() + 1
const curD = curDate.getDate()
let mockDates: import('./types').UpcomingDate[] = [
  { id: 'b-amanda', title: 'Aniversário de Amanda Silva', personName: 'Amanda Silva', kind: 'birthday', month: curM, day: curD, daysUntil: 0, turningAge: 29, sourceProvider: 'google' },
  { id: 'b-marcos', title: 'Aniversário de Marcos Paulo', personName: 'Marcos Paulo', kind: 'birthday', month: curM, day: Math.min(28, curD + 2), daysUntil: 2, turningAge: 34, sourceProvider: 'google' },
  { id: 'h-natal', title: 'Natal', personName: '', kind: 'holiday', month: 12, day: 25, daysUntil: 42, turningAge: null, sourceProvider: 'local' }
]
const mockPhone = { id: 'px-s24', name: 'Galaxy S24 Ultra', platform: 'android', connected: true, activeTransport: 'lan' as 'lan' | 'usb' | 'bluetooth', batteryPct: 82, charging: false, lastSeenTs: Math.floor(Date.now() / 1000) }
const mockSignals = [
  { id: 's1', deviceId: 'px-s24', kind: 'notification', title: 'Amanda Silva', body: 'Oi! Confirmando nosso almoço hoje às 12h30 😊', appName: 'WhatsApp', ts: Math.floor(Date.now() / 1000) - 180 },
  { id: 's2', deviceId: 'px-s24', kind: 'notification', title: 'Nubank', body: 'Pix recebido de R$ 150,00', appName: 'Nubank', ts: Math.floor(Date.now() / 1000) - 920 }
]

function mockAssistantSnapshot() {
  const nowSec = Math.floor(Date.now() / 1000)
  const accounts = []
  if (mockGoogleConnected) accounts.push({ id: 'google:josueh@gmail.com', provider: 'google', label: 'Josueh Elelyon', email: 'josueh@gmail.com', isPrimary: true, status: 'connected', lastSyncTs: nowSec - 120, details: '4 eventos · 128 contatos' })
  if (mockWinConnected) accounts.push({ id: 'windows:local', provider: 'windows', label: 'Calendário do Windows', email: 'WinRT Local', isPrimary: false, status: 'connected', lastSyncTs: nowSec - 90, details: '3 eventos locais (WinRT)' })
  if (mockGithubConnected) accounts.push({ id: 'github:JosuehElelyon', provider: 'github', label: '@JosuehElelyon', email: 'josueh@gmail.com', isPrimary: false, status: 'connected', lastSyncTs: nowSec - 300, details: '2 PRs para revisar' })
  if (mockAppleConnected) accounts.push({ id: 'apple:josueh@icloud.com', provider: 'apple', label: 'Apple iCloud', email: 'josueh@icloud.com', isPrimary: false, status: 'connected', lastSyncTs: nowSec - 400, details: 'CalDAV/CardDAV ativo' })

  const events = [
    ...(mockGoogleConnected ? [
      { id: 'ev2', fingerprint: 'fp2', title: 'Almoço Aniversário Amanda', category: 'personal', date: today, startTime: '12:30', endTime: '13:30', startTs: nowSec + 1800, endTs: nowSec + 5400, allDay: false, joinUrl: '', sourceProvider: 'google', accountId: 'google:josueh@gmail.com' },
      { id: 'ev1', fingerprint: 'fp1', title: 'Alinhamento de Arquitetura', category: 'meeting', date: today, startTime: '14:00', endTime: '14:45', startTs: nowSec + 3600, endTs: nowSec + 6300, allDay: false, joinUrl: 'https://meet.google.com/abc-defg-hij', sourceProvider: 'google', accountId: 'google:josueh@gmail.com' }
    ] : []),
    ...(mockWinConnected ? [
      { id: 'ev3', fingerprint: 'fp3', title: 'Daily — Equipe Core', category: 'meeting', date: today, startTime: '16:00', endTime: '16:20', startTs: nowSec + 7200, endTs: nowSec + 8400, allDay: false, joinUrl: 'https://teams.microsoft.com/l/meetup-join/mock', sourceProvider: 'windows', accountId: 'windows:local' }
    ] : [])
  ]

  const alerts = []
  const callSig = mockSignals.find(s => s.kind === 'call' && nowSec - s.ts <= 90)
  if (callSig) {
    alerts.push({ key: 'call:' + callSig.id, level: 'urgent', avatarMood: 'alert', icon: '📞', title: `Ligação: ${callSig.title}`, body: callSig.body || 'Chamada no celular', bubbleText: `📞 ${callSig.title}`, actionLabel: 'Celular', actionTarget: 'tab:assistant:celular', createdTs: nowSec })
  }
  const bdayToday = mockDates.find(d => d.daysUntil === 0)
  if (bdayToday) {
    alerts.push({ key: 'date:' + bdayToday.id, level: 'celebrate', avatarMood: 'celebrate', icon: '🎂', title: `Hoje: ${bdayToday.title}${bdayToday.turningAge ? ` (${bdayToday.turningAge} anos)` : ''}`, body: 'Não esqueça de dar os parabéns!', bubbleText: `🎂 ${bdayToday.personName || bdayToday.title}!`, actionLabel: 'Ver Datas', actionTarget: 'tab:assistant:hoje', createdTs: nowSec })
  }

  return {
    googleConnected: mockGoogleConnected,
    accounts,
    upcomingDates: mockDates,
    todayEvents: events,
    allEvents: events,
    peopleCount: mockGoogleConnected ? 128 : 0,
    phones: [mockPhone],
    recentSignals: mockSignals,
    alerts,
    devItems: mockGithubConnected ? [
      { id: 'gh1', provider: 'github', repo: 'JosuehElelyon/ilha-dinamica', title: 'feat: ponte PhoneBridge LAN/USB/BT', kind: 'review', url: 'https://github.com/JosuehElelyon/ilha-dinamica', updatedAt: new Date().toISOString() }
    ] : []
  }
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
