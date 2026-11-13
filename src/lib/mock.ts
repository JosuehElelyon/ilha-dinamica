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
    notes: '', settings: {}, platform: 'windows', isAdmin: false, autostart: false, version: '2.0.0',
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
  relaunch_admin: () => false
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
        { text: 'Reunião remarcada para 15h', t: Date.now() },
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
