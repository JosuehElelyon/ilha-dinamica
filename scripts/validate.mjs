// Validação local contínua a cada commit:
// 1) cargo test (crates/assistant-core)
// 2) vue-tsc --noEmit && vite build (npm run build:web)
// 3) Smoke test real em Chromium headless (quando disponível no ambiente)
import { execSync, spawn } from 'node:child_process'
import fs from 'node:fs'

const skipUi = process.argv.includes('--skip-ui')
const run = (cmd) => {
  console.log(`\n▶ ${cmd}`)
  execSync(cmd, { stdio: 'inherit' })
}

run('cargo test --manifest-path crates/assistant-core/Cargo.toml')
run('npm run build:web')

const chromeBin = ['/usr/bin/chromium', '/usr/bin/google-chrome', '/usr/bin/chromium-browser'].find(p => fs.existsSync(p))
if (skipUi || !chromeBin) {
  console.log('\n✓ Validação concluída (sem browser headless nesta máquina).')
  process.exit(0)
}

console.log(`\n▶ Smoke test UI em ${chromeBin}…`)
const sleep = (ms) => new Promise(r => setTimeout(r, ms))
const port = 4195
const cdpPort = 9241
const userData = `/tmp/ilha-validate-${Date.now()}`

const preview = spawn('npx', ['vite', 'preview', '--port', String(port), '--host', '127.0.0.1'], { stdio: 'ignore' })
const chrome = spawn(chromeBin, [
  '--headless=new', '--no-sandbox', '--disable-gpu',
  `--remote-debugging-port=${cdpPort}`,
  `--user-data-dir=${userData}`,
  '--window-size=840,370',
  'about:blank'
], { stdio: 'ignore' })

async function cdpClient(wsUrl) {
  const ws = new WebSocket(wsUrl)
  await new Promise(res => ws.addEventListener('open', res))
  let id = 0
  const pending = new Map()
  const runtimeErrors = []
  ws.addEventListener('message', ev => {
    const msg = JSON.parse(ev.data)
    if (msg.method === 'Runtime.exceptionThrown') {
      runtimeErrors.push(msg.params?.exceptionDetails?.text || 'Uncaught exception')
    }
    if (msg.method === 'Runtime.consoleAPICalled' && msg.params?.type === 'error') {
      const txt = (msg.params.args || []).map(a => a.value ?? a.description ?? '').join(' ').trim()
      if (txt) runtimeErrors.push(`console.error: ${txt}`)
    }
    if (msg.method === 'Log.entryAdded' && msg.params?.entry?.level === 'error') {
      const entry = msg.params.entry
      const url = entry.url || ''
      const isExternal = url.startsWith('https://') && !url.includes('127.0.0.1')
      const isFavicon = url.endsWith('/favicon.ico')
      if (!isExternal && !isFavicon && entry.source !== 'network') {
        runtimeErrors.push(entry.text)
      }
    }
    if (msg.id && pending.has(msg.id)) {
      const { res, rej } = pending.get(msg.id)
      pending.delete(msg.id)
      if (msg.error) rej(new Error(msg.error.message))
      else res(msg.result)
    }
  })
  const send = (method, params = {}) => new Promise((res, rej) => {
    const cid = ++id
    pending.set(cid, { res, rej })
    ws.send(JSON.stringify({ id: cid, method, params }))
  })
  return { send, runtimeErrors, close: () => ws.close() }
}

try {
  for (let i = 0; i < 30; i++) {
    try { await fetch(`http://127.0.0.1:${port}/`); break } catch {}
    await sleep(150)
  }
  let wsUrl = ''
  for (let i = 0; i < 30; i++) {
    try {
      const list = await fetch(`http://127.0.0.1:${cdpPort}/json/list`).then(r => r.json())
      const page = list.find(t => t.type === 'page')
      if (page?.webSocketDebuggerUrl) { wsUrl = page.webSocketDebuggerUrl; break }
    } catch {}
    await sleep(150)
  }
  if (!wsUrl) throw new Error('Falha ao conectar ao Chromium CDP')

  const { send, runtimeErrors, close } = await cdpClient(wsUrl)
  await send('Page.enable')
  await send('Runtime.enable')
  await send('Log.enable')
  await send('Emulation.setDeviceMetricsOverride', { width: 840, height: 370, deviceScaleFactor: 1, mobile: false })
  await send('Page.navigate', { url: `http://127.0.0.1:${port}/` })
  await sleep(600)

  await send('Runtime.evaluate', { expression: 'if (window.__ilhaMock) window.__ilhaMock.emit("toggle")' })
  await sleep(300)

  const checkTabs = ['INÍCIO', 'ASSISTENTE', 'AGENDA', 'REDES', 'ÁUDIO', 'MÓDULOS']
  for (const title of checkTabs) {
    await send('Runtime.evaluate', {
      expression: `document.querySelector('nav button[title="${title}"]')?.click()`
    })
    await sleep(120)
    const ok = await send('Runtime.evaluate', {
      returnByValue: true,
      expression: `(() => {
        const m = document.querySelector('main');
        const nav = document.querySelector('nav');
        if (!m || !nav) return 'missing main/nav';
        const mr = m.getBoundingClientRect(), nr = nav.getBoundingClientRect();
        if (mr.bottom > nr.top + 1) return 'main overlaps nav: ' + mr.bottom + ' > ' + nr.top;
        return 'ok';
      })()`
    })
    if (ok?.result?.value !== 'ok') {
      throw new Error(`Falha layout na aba ${title}: ${ok?.result?.value}`)
    }
  }

  if (runtimeErrors.length) {
    throw new Error(`Erros JS no navegador: ${runtimeErrors.join('; ')}`)
  }
  close()
  console.log('✓ Smoke test UI em Chromium passou (0 erros JS, 0 sobreposições)!')
} finally {
  chrome.kill()
  preview.kill()
  try { fs.rmSync(userData, { recursive: true, force: true }) } catch {}
}
