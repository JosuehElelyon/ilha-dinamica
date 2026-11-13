const {
  app, BrowserWindow, ipcMain, screen, clipboard, Tray, Menu,
  nativeImage, globalShortcut, Notification, shell, dialog
} = require('electron');
const path = require('path');
const fs = require('fs');
const os = require('os');
const { spawn, spawnSync, execFile } = require('child_process');
const fsp = fs.promises;

const W = 880;   // largura da janela (transparente)
const H = 400;   // altura da janela (transparente)
const isWin = process.platform === 'win32';

let win, tray, mediaProc, helperProc, toolsProc;
let isAdmin = false;
let clips = [], lastClip = '';

if (!app.requestSingleInstanceLock()) { app.quit(); }
app.setAppUserModelId('com.josueh.ilhadinamica');

/* ---------- dados persistidos (notas, lembretes e configurações) ---------- */
const dataFile = () => path.join(app.getPath('userData'), 'data.json');
function loadData() {
  let d = {};
  try { d = JSON.parse(fs.readFileSync(dataFile(), 'utf8')); } catch { }
  return { notes: '', settings: {}, reminders: [], ...d };
}
function saveData(d) {
  fs.mkdirSync(path.dirname(dataFile()), { recursive: true });
  fs.writeFileSync(dataFile(), JSON.stringify(d, null, 2));
}
let data;

/* ---------- janela ---------- */
function placeWindow() {
  const { bounds } = screen.getPrimaryDisplay();
  win.setBounds({ x: Math.round(bounds.x + (bounds.width - W) / 2), y: bounds.y, width: W, height: H });
}
function createWindow() {
  win = new BrowserWindow({
    width: W, height: H,
    frame: false, transparent: true, resizable: false, movable: false,
    alwaysOnTop: true, skipTaskbar: true, hasShadow: false, show: false,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  });
  placeWindow();
  win.setAlwaysOnTop(true, 'screen-saver');
  win.setVisibleOnAllWorkspaces(true);
  // Por padrão a área transparente deixa o clique passar para o que está atrás
  win.setIgnoreMouseEvents(true, { forward: true });
  win.loadFile(path.join(__dirname, 'index.html'));
  win.once('ready-to-show', () => win.showInactive());
}

const send = (ch, payload) => { if (win && !win.isDestroyed()) win.webContents.send(ch, payload); };

/* ---------- clipboard (só em memória, nunca salvo em disco) ---------- */
function pollClipboard() {
  const text = clipboard.readText();
  if (!text || text === lastClip) return;
  lastClip = text;
  clips = [{ text, t: Date.now() }, ...clips.filter(c => c.text !== text)].slice(0, 8);
  send('clips', clips);
}

/* ---------- processos PowerShell ---------- */
function runScript(name, onLine) {
  // copia o script para a pasta temp (funciona mesmo com o app empacotado em .asar)
  const src = path.join(__dirname, '..', 'scripts', name);
  const dst = path.join(os.tmpdir(), 'ilha-dinamica-' + name);
  fs.writeFileSync(dst, fs.readFileSync(src, 'utf8'));
  const proc = spawn('powershell.exe',
    ['-NoProfile', '-NonInteractive', '-ExecutionPolicy', 'Bypass', '-File', dst],
    { windowsHide: true });
  let buf = '';
  proc.stdout.setEncoding('utf8');
  proc.stdout.on('data', chunk => {
    buf += chunk;
    let i;
    while ((i = buf.indexOf('\n')) >= 0) {
      const line = buf.slice(0, i).trim(); buf = buf.slice(i + 1);
      if (!line) continue;
      try { onLine(JSON.parse(line)); } catch { /* ignora linha inválida */ }
    }
  });
  proc.stdin.on('error', () => { });
  return proc;
}

function startMedia() {
  if (!isWin) return;
  mediaProc = runScript('media.ps1', m => send('media', m));
  mediaProc.on('exit', () => { mediaProc = null; if (!app.isQuitting) setTimeout(startMedia, 5000); });
}

function startHelper() {
  if (!isWin) return;
  helperProc = runScript('helper.ps1', m => {
    if (m.type === 'vol') send('vol', m);
    if (m.type === 'bright') send('bright', m);
    if (m.type === 'devices') send('devices', m);
    if (m.type === 'error') send('helper-error', m);
  });
  helperProc.on('exit', () => { helperProc = null; if (!app.isQuitting) setTimeout(startHelper, 5000); });
}
function helper(cmd) {
  if (helperProc && helperProc.stdin.writable) helperProc.stdin.write(cmd + '\n');
}

/* ---------- processo de ferramentas (processos, lixeira, memória) ---------- */
const toolWait = {};
function startTools() {
  if (!isWin) return;
  toolsProc = runScript('tools.ps1', m => {
    if (m.type === 'procs') send('procs', m.list || []);
    const q = toolWait[m.type];
    if (q && q.length) q.shift()(m);
  });
  toolsProc.on('exit', () => { toolsProc = null; if (!app.isQuitting) setTimeout(startTools, 5000); });
}
function tools(cmd, type, timeout = 120000) {
  return new Promise(res => {
    if (!toolsProc || !toolsProc.stdin.writable) return res(null);
    let done = false;
    const fn = (m) => { if (!done) { done = true; res(m); } };
    (toolWait[type] = toolWait[type] || []).push(fn);
    toolsProc.stdin.write(cmd + '\n');
    setTimeout(() => {
      if (done) return;
      done = true;
      const q = toolWait[type]; const i = q.indexOf(fn); if (i >= 0) q.splice(i, 1);
      res(null);
    }, timeout);
  });
}

/* ---------- limpeza ---------- */
const ENV = (k, d = '') => process.env[k] || d;
function chromiumCaches(base) {
  const out = [];
  let profiles = [];
  try {
    profiles = fs.readdirSync(base, { withFileTypes: true })
      .filter(d => d.isDirectory() && (d.name === 'Default' || /^Profile \d+$/.test(d.name) || d.name === 'Guest Profile'))
      .map(d => d.name);
  } catch { return out; }
  for (const p of profiles)
    for (const c of ['Cache', 'Code Cache', 'GPUCache', path.join('Service Worker', 'CacheStorage')])
      out.push(path.join(base, p, c));
  for (const c of ['ShaderCache', 'GrShaderCache']) out.push(path.join(base, c));
  return out;
}
function firefoxCaches() {
  const b = path.join(ENV('LOCALAPPDATA'), 'Mozilla', 'Firefox', 'Profiles');
  try { return fs.readdirSync(b).map(p => path.join(b, p, 'cache2')); } catch { return []; }
}
const CLEAN = {
  temp: { name: 'Arquivos temporários', icon: '🗂️',
    dirs: () => [os.tmpdir()], skip: n => n.startsWith('ilha-dinamica-') },
  browsers: { name: 'Cache dos navegadores', icon: '🌐',
    dirs: () => {
      const L = ENV('LOCALAPPDATA'), R = ENV('APPDATA');
      return [
        ...chromiumCaches(path.join(L, 'Google', 'Chrome', 'User Data')),
        ...chromiumCaches(path.join(L, 'Microsoft', 'Edge', 'User Data')),
        ...chromiumCaches(path.join(L, 'BraveSoftware', 'Brave-Browser', 'User Data')),
        path.join(R, 'Opera Software', 'Opera Stable', 'Cache'),
        path.join(L, 'Opera Software', 'Opera Stable', 'Cache'),
        path.join(R, 'Opera Software', 'Opera GX Stable', 'Cache'),
        path.join(L, 'Opera Software', 'Opera GX Stable', 'Cache'),
        ...firefoxCaches()
      ];
    } },
  apps: { name: 'Cache de apps (Discord, VS Code)', icon: '💬',
    dirs: () => {
      const R = ENV('APPDATA');
      return ['Cache', 'Code Cache', 'GPUCache'].map(c => path.join(R, 'discord', c))
        .concat([path.join(R, 'Code', 'Cache'), path.join(R, 'Code', 'CachedData'), path.join(R, 'Code', 'GPUCache')]);
    } },
  thumbs: { name: 'Miniaturas e ícones', icon: '🖼️',
    dirs: () => [path.join(ENV('LOCALAPPDATA'), 'Microsoft', 'Windows', 'Explorer')],
    only: n => /^(thumbcache|iconcache)_.*\.db$/i.test(n) },
  crash: { name: 'Relatórios de erro', icon: '💥',
    dirs: () => [path.join(ENV('LOCALAPPDATA'), 'CrashDumps'),
      path.join(ENV('LOCALAPPDATA'), 'Microsoft', 'Windows', 'WER'),
      path.join(ENV('ProgramData', 'C:\\ProgramData'), 'Microsoft', 'Windows', 'WER')] },
  wintemp: { name: 'Temp do Windows', icon: '🪟', admin: true,
    dirs: () => [path.join(ENV('SystemRoot', 'C:\\Windows'), 'Temp')] },
  update: { name: 'Cache do Windows Update', icon: '🔄', admin: true,
    dirs: () => [path.join(ENV('SystemRoot', 'C:\\Windows'), 'SoftwareDistribution', 'Download')] },
  recycle: { name: 'Lixeira', icon: '🗑️', special: true }
};

async function sizeOf(p) {
  let st;
  try { st = await fsp.lstat(p); } catch { return 0; }
  if (st.isSymbolicLink()) return 0;
  if (!st.isDirectory()) return st.size;
  let ents;
  try { ents = await fsp.readdir(p); } catch { return 0; }
  let total = 0;
  for (let i = 0; i < ents.length; i += 64) {
    const part = await Promise.all(ents.slice(i, i + 64).map(e => sizeOf(path.join(p, e))));
    total += part.reduce((a, b) => a + b, 0);
  }
  return total;
}
// itens de primeiro nível que podem ser apagados (nunca a pasta raiz em si)
async function entriesOf(cat) {
  const out = []; let denied = false;
  for (const d of cat.dirs()) {
    let ents;
    try { ents = await fsp.readdir(d); }
    catch (e) { if (e.code === 'EPERM' || e.code === 'EACCES') denied = true; continue; }
    for (const n of ents) {
      if (cat.skip && cat.skip(n)) continue;
      if (cat.only && !cat.only(n)) continue;
      out.push(path.join(d, n));
    }
  }
  return { list: out, denied };
}
async function scanCat(key) {
  const cat = CLEAN[key];
  if (cat.special) {
    const r = await tools('recycle', 'recycle');
    return { size: r ? r.size : 0, count: r ? r.count : 0 };
  }
  const { list, denied } = await entriesOf(cat);
  let size = 0;
  for (const e of list) size += await sizeOf(e);
  return { size, count: list.length, denied };
}
async function cleanCat(key) {
  const cat = CLEAN[key];
  if (cat.special) { await tools('emptyrecycle', 'recycle'); return; }
  const { list } = await entriesOf(cat);
  for (const e of list) {
    try { await fsp.rm(e, { recursive: true, force: true }); } catch { /* em uso ou sem permissão: pula */ }
  }
}
async function diskInfo() {
  try {
    const st = await fsp.statfs(isWin ? (ENV('SystemDrive', 'C:') + '\\') : '/');
    return { free: st.bavail * st.bsize, total: st.blocks * st.bsize };
  } catch { return null; }
}

/* ---------- processos protegidos (finalizar pode travar ou reiniciar o Windows) ---------- */
const PROTECTED = new Set(['system', 'registry', 'smss', 'csrss', 'wininit', 'winlogon', 'services', 'lsass', 'lsaiso',
  'svchost', 'dwm', 'fontdrvhost', 'memory compression', 'secure system', 'sihost', 'ctfmon', 'msmpeng',
  'securityhealthservice', 'spoolsv', 'audiodg', 'conhost', 'runtimebroker', 'taskhostw', 'idle',
  'startmenuexperiencehost', 'shellexperiencehost', 'searchhost', 'textinputhost', 'wudfhost', 'nissrv']);
const selfName = () => path.basename(process.execPath, '.exe').toLowerCase();
const ownPids = () => [process.pid, mediaProc && mediaProc.pid, helperProc && helperProc.pid, toolsProc && toolsProc.pid].filter(Boolean);

const MEDIA_KEYS = { toggle: 'B3', next: 'B0', prev: 'B1' };

/* ---------- uso de CPU / RAM ---------- */
let prevCpu = os.cpus();
function sysStats() {
  const cur = os.cpus();
  let idle = 0, total = 0;
  cur.forEach((c, i) => {
    const p = prevCpu[i] ? prevCpu[i].times : { user: 0, nice: 0, sys: 0, idle: 0, irq: 0 };
    const t = c.times;
    total += (t.user - p.user) + (t.nice - p.nice) + (t.sys - p.sys) + (t.idle - p.idle) + (t.irq - p.irq);
    idle += t.idle - p.idle;
  });
  prevCpu = cur;
  send('sys', {
    cpu: total ? Math.round(100 * (1 - idle / total)) : 0,
    memUsed: os.totalmem() - os.freemem(),
    memTotal: os.totalmem(),
    uptime: os.uptime(),
    cpuName: (cur[0] && cur[0].model || '').trim()
  });
}

/* ---------- bandeja ---------- */
function createTray() {
  tray = new Tray(nativeImage.createFromPath(path.join(__dirname, '..', 'assets', 'tray.png')));
  tray.setToolTip('Ilha Dinâmica');
  tray.rebuild = () => tray.setContextMenu(Menu.buildFromTemplate([
    { label: 'Abrir / fechar ilha  (Ctrl+Alt+Espaço)', click: () => send('toggle') },
    { label: 'Iniciar com o Windows', type: 'checkbox', checked: !!data.settings.openAtLogin,
      click: (it) => setLogin(it.checked) },
    { type: 'separator' },
    { label: 'Sair', click: () => app.quit() }
  ]));
  tray.rebuild();
}

function setLogin(v) {
  data.settings.openAtLogin = !!v; saveData(data);
  app.setLoginItemSettings({ openAtLogin: !!v });
  if (tray) tray.rebuild();
}

/* ---------- IPC ---------- */
ipcMain.on('interactive', (_e, v) => { if (win) win.setIgnoreMouseEvents(!v, { forward: true }); });
ipcMain.on('copy', (_e, text) => { lastClip = text; clipboard.writeText(text); });
ipcMain.on('clear-clips', () => { clips = []; send('clips', clips); });
ipcMain.on('media-cmd', (_e, cmd) => { if (MEDIA_KEYS[cmd]) helper('key ' + MEDIA_KEYS[cmd]); });
ipcMain.on('set-vol', (_e, v) => helper('setvol ' + Math.max(0, Math.min(100, Math.round(v)))));
ipcMain.on('toggle-mute', () => helper('mute'));
ipcMain.on('set-mic', (_e, v) => helper('setmic ' + Math.max(0, Math.min(100, Math.round(v)))));
ipcMain.on('toggle-mic', () => helper('micmute'));
ipcMain.on('set-device', (_e, id) => { if (/^[\w{}.\-]+$/.test(id)) helper('setdev ' + id); });
ipcMain.on('refresh-devices', () => helper('devices'));

/* ---------- ações personalizadas ---------- */
ipcMain.on('run-action', (_e, a) => {
  if (!a || !a.target) return;
  const t = String(a.target).trim();
  if (/^https?:\/\//i.test(t)) { shell.openExternal(t); return; }
  if (!isWin) { shell.openPath(t); return; }
  // "start" abre programas do PATH (calc, notepad), pastas, arquivos, ms-settings: etc.
  spawn('cmd.exe', ['/d /s /c start "" "' + t.replace(/"/g, '') + '"'],
    { windowsHide: true, windowsVerbatimArguments: true, detached: true, stdio: 'ignore' }).unref();
});
ipcMain.handle('pick-path', async (_e, kind) => {
  win.setAlwaysOnTop(false);
  const r = await dialog.showOpenDialog(win, {
    title: kind === 'folder' ? 'Escolha uma pasta' : 'Escolha um programa ou arquivo',
    properties: [kind === 'folder' ? 'openDirectory' : 'openFile']
  });
  win.setAlwaysOnTop(true, 'screen-saver');
  return r.canceled ? null : r.filePaths[0];
});
ipcMain.on('set-bright', (_e, v) => helper('setbright ' + Math.max(0, Math.min(100, Math.round(v)))));
ipcMain.handle('get-data', () => ({ ...data, platform: process.platform, isAdmin,
  selfName: selfName(), ownPids: ownPids(), protectedNames: [...PROTECTED] }));

/* ---------- limpeza / processos ---------- */
ipcMain.handle('clean-scan', async () => {
  const cats = {};
  await Promise.all(Object.keys(CLEAN).map(async k => {
    const c = CLEAN[k];
    cats[k] = { name: c.name, icon: c.icon, admin: !!c.admin, ...(isWin ? await scanCat(k) : { size: 0, count: 0 }) };
  }));
  return { cats, disk: await diskInfo(), isAdmin };
});
ipcMain.handle('clean-run', async (_e, keys) => {
  if (!isWin) return { freed: 0 };
  const before = await diskInfo();
  for (const k of (keys || []).filter(k => CLEAN[k])) await cleanCat(k);
  const after = await diskInfo();
  return { freed: before && after ? Math.max(0, after.free - before.free) : 0, disk: after };
});
ipcMain.handle('trim-ram', async () => {
  const before = os.freemem();
  const r = await tools('trimram', 'trimram');
  await new Promise(r => setTimeout(r, 1200));
  return { freed: Math.max(0, os.freemem() - before), n: r ? r.n : 0 };
});
ipcMain.handle('flush-dns', () => new Promise(res => {
  if (!isWin) return res({ ok: false });
  execFile('ipconfig', ['/flushdns'], { windowsHide: true }, (err) => res({ ok: !err }));
}));
ipcMain.on('procs-refresh', () => { if (toolsProc) tools('procs', 'procs', 15000); });
ipcMain.handle('kill', (_e, { pid, name }) => new Promise(res => {
  if (!isWin) return res({ ok: false });
  const nm = String(name || '').toLowerCase();
  if (PROTECTED.has(nm) || nm === selfName()) return res({ ok: false, protected: true });
  let args;
  if (pid) {
    pid = parseInt(pid, 10);
    if (!pid || ownPids().includes(pid)) return res({ ok: false, protected: true });
    args = ['/PID', String(pid), '/T', '/F'];
  } else {
    if (!/^[\w .()+\-]+$/.test(name)) return res({ ok: false });
    args = ['/IM', name + '.exe', '/T', '/F'];
  }
  execFile('taskkill', args, { windowsHide: true }, (err) => res({ ok: !err }));
}));
ipcMain.on('relaunch-admin', () => {
  if (!isWin || isAdmin) return;
  const exe = process.execPath.replace(/'/g, "''");
  const argList = app.isPackaged ? '' : ` -ArgumentList '"${app.getAppPath().replace(/'/g, "''")}"'`;
  app.releaseSingleInstanceLock();
  const ps = spawn('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command',
    `Start-Process -FilePath '${exe}'${argList} -Verb RunAs`], { windowsHide: true });
  ps.on('exit', (code) => {
    if (code === 0) app.quit();
    else app.requestSingleInstanceLock(); // usuário recusou o UAC: continua rodando normalmente
  });
});
ipcMain.on('save-notes', (_e, t) => { data.notes = t; saveData(data); });
ipcMain.on('save-reminders', (_e, r) => { data.reminders = r; saveData(data); });
ipcMain.on('save-setting', (_e, { key, value }) => { data.settings[key] = value; saveData(data); });
ipcMain.on('notify', (_e, { title, body }) => {
  if (Notification.isSupported()) new Notification({ title, body }).show();
});
ipcMain.on('set-login', (_e, v) => setLogin(v));
ipcMain.on('quit', () => app.quit());

/* ---------- ciclo de vida ---------- */
app.whenReady().then(() => {
  data = loadData();
  createWindow();
  createTray();
  lastClip = clipboard.readText() || '';
  setInterval(pollClipboard, 700);
  setInterval(sysStats, 2000);
  startMedia();
  startHelper();
  startTools();
  if (isWin) { try { isAdmin = spawnSync('net', ['session'], { windowsHide: true }).status === 0; } catch { } }
  setInterval(() => helper('vol'), 800);
  setInterval(() => { helper('bright'); helper('devices'); }, 5000);
  helper('bright'); helper('devices');
  globalShortcut.register('Control+Alt+Space', () => send('toggle'));
  screen.on('display-metrics-changed', placeWindow);
});

app.on('before-quit', () => {
  app.isQuitting = true;
  if (mediaProc) mediaProc.kill();
  if (helperProc) helperProc.kill();
  if (toolsProc) toolsProc.kill();
});
app.on('will-quit', () => globalShortcut.unregisterAll());
app.on('window-all-closed', () => app.quit());
