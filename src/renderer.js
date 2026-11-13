const $ = (s, r = document) => r.querySelector(s);
const $$ = (s, r = document) => [...r.querySelectorAll(s)];
// "api" é exposto pelo preload (contextBridge) como variável global
const root = document.documentElement;

const DIAS = ['DOM', 'SEG', 'TER', 'QUA', 'QUI', 'SEX', 'SÁB'];
const DIAS_CURTOS = ['Dom', 'Seg', 'Ter', 'Qua', 'Qui', 'Sex', 'Sáb'];
const MESES = ['JAN', 'FEV', 'MAR', 'ABR', 'MAI', 'JUN', 'JUL', 'AGO', 'SET', 'OUT', 'NOV', 'DEZ'];
const MESES_LONGOS = ['Janeiro', 'Fevereiro', 'Março', 'Abril', 'Maio', 'Junho', 'Julho', 'Agosto', 'Setembro', 'Outubro', 'Novembro', 'Dezembro'];
const pad = (n) => String(n).padStart(2, '0');
const ymd = (d) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
const esc = (s) => String(s).replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const fmt = (sec) => {
  sec = Math.max(0, Math.round(sec));
  const h = Math.floor(sec / 3600), m = Math.floor((sec % 3600) / 60), s = sec % 60;
  return h ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
};

/* =============== módulos (abas) disponíveis =============== */
const ICONS = {
  home: '<path d="M4 11l8-7 8 7v9h-5v-6H9v6H4z"/>',
  actions: '<path d="M13 3L5 14h6l-1 7 8-11h-6z"/>',
  clips: '<rect x="8" y="3" width="11" height="14" rx="2"/><path d="M5 7v12a2 2 0 0 0 2 2h9"/>',
  notes: '<path d="M6 3h9l4 4v14H6z"/><path d="M9 11h7M9 15h7"/>',
  timer: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l3 2M9 2h6"/>',
  cal: '<rect x="4" y="5" width="16" height="16" rx="2"/><path d="M4 10h16M9 3v4M15 3v4"/>',
  weather: '<path d="M7 18h10a4 4 0 0 0 0-8 6 6 0 0 0-11.5 1.5A3.3 3.3 0 0 0 7 18z"/>',
  audio: '<path d="M4 14v-2a8 8 0 0 1 16 0v2"/><rect x="3" y="14" width="4" height="6" rx="1.5"/><rect x="17" y="14" width="4" height="6" rx="1.5"/>',
  sys: '<rect x="7" y="7" width="10" height="10" rx="1"/><path d="M10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4"/>',
  clean: '<path d="M19 4l-7 7"/><path d="M12.5 10.5l1 1c.8.8.8 2 0 2.8L9 19H4v-5l4.5-4.5c.8-.8 2-.8 2.8 0z"/><path d="M7 15l2 2"/>',
  procs: '<path d="M4 6h16M4 12h16M4 18h10"/><circle cx="19" cy="18" r="1.5"/>'
};
const MODULES = {
  home: 'INÍCIO', actions: 'AÇÕES', clips: 'CLIPS', notes: 'NOTAS', timer: 'TIMER',
  cal: 'AGENDA', weather: 'CLIMA', audio: 'ÁUDIO', sys: 'SISTEMA', clean: 'LIMPEZA', procs: 'PROCESSOS'
};
const svg = (id) => `<svg viewBox="0 0 24 24">${ICONS[id]}</svg>`;

/* =============== configurações =============== */
const DEFAULT_ACTIONS = [
  { id: 1, emoji: '🌐', name: 'Google', type: 'url', target: 'https://www.google.com' },
  { id: 2, emoji: '▶️', name: 'YouTube', type: 'url', target: 'https://www.youtube.com' },
  { id: 3, emoji: '📁', name: 'Downloads', type: 'folder', target: '%USERPROFILE%\\Downloads' },
  { id: 4, emoji: '🧮', name: 'Calculadora', type: 'cmd', target: 'calc' },
  { id: 5, emoji: '⚙️', name: 'Config.', type: 'cmd', target: 'ms-settings:' }
];
const DEFAULTS = {
  accent: '#ff8a2b', followMusic: true, size: 'normal', opacity: 94, city: null,
  alerts: { vol: true, bat: true, net: true, track: true, dev: true },
  modules: ['home', 'actions', 'clips', 'notes', 'timer', 'cal', 'weather', 'audio', 'sys', 'clean', 'procs'],
  cleanSel: ['temp', 'browsers', 'apps', 'thumbs', 'crash', 'wintemp', 'update'],
  procSort: 'm',
  modulesOff: [],
  actions: DEFAULT_ACTIONS
};
let S = JSON.parse(JSON.stringify(DEFAULTS));
function saveSetting(k, v) { S[k] = v; api.saveSetting(k, v); }

const ACCENTS = ['#ff8a2b', '#ff4d6d', '#ffd23f', '#34d058', '#2fb8ff', '#8b7bff', '#f0f0f0'];
function hexHue(hex) {
  const n = parseInt(hex.slice(1), 16), r = (n >> 16) / 255, g = ((n >> 8) & 255) / 255, b = (n & 255) / 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), d = mx - mn;
  if (!d) return 18;
  let h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return Math.round((h * 60 + 360) % 360);
}

/* =============== expandir / recolher =============== */
const island = $('#island');
let expanded = false, pinned = false, leaveTimer = null;

function setExpanded(v) {
  expanded = v;
  island.classList.toggle('expanded', v);
  island.classList.toggle('collapsed', !v);
  if (!v) closeMenu();
}
function collapse(force) {
  if (pinned) return;
  const a = document.activeElement;
  if (!force && a && a.matches && a.matches('textarea, input[type=text], input[type=time], input:not([type]), select')) return;
  setExpanded(false);
  api.setInteractive(false);
}
island.addEventListener('mouseenter', () => {
  clearTimeout(leaveTimer);
  api.setInteractive(true);
  setExpanded(true);
});
island.addEventListener('mouseleave', () => {
  clearTimeout(leaveTimer);
  leaveTimer = setTimeout(collapse, 450);
});
island.addEventListener('focusout', () => setTimeout(() => { if (!island.matches(':hover')) collapse(); }, 60));
window.addEventListener('blur', () => { if (!island.matches(':hover')) collapse(true); });
api.onToggle(() => {
  pinned = !expanded;
  $('#optPin').checked = pinned;
  setExpanded(pinned);
  api.setInteractive(pinned);
});

/* =============== abas dinâmicas =============== */
let currentTab = 'home';
function renderNav() {
  const tabs = $('#tabs');
  tabs.innerHTML = '';
  S.modules.filter(id => MODULES[id] && !S.modulesOff.includes(id)).forEach(id => {
    const b = document.createElement('button');
    b.className = 'tab-btn' + (id === currentTab ? ' active' : '');
    b.dataset.tab = id; b.title = MODULES[id];
    b.innerHTML = `${svg(id)}<span>${MODULES[id]}</span>`;
    b.addEventListener('click', () => showTab(id));
    tabs.appendChild(b);
  });
  $('nav .plus').classList.toggle('active', currentTab === 'modules');
}
function showTab(t) {
  currentTab = t;
  $$('.tab').forEach(s => s.classList.toggle('active', s.dataset.tab === t));
  renderNav();
  if (t === 'cal') { renderCal(); renderDay(); }
  if (t === 'modules') renderModules();
  if (t === 'audio') api.refreshDevices();
  if (t === 'actions') closeActForm();
  if (t === 'procs') api.refreshProcs();
  if (t === 'clean' && !cleanData) runScan();
}
$('nav .plus').addEventListener('click', () => showTab(currentTab === 'modules' ? firstTab() : 'modules'));
const firstTab = () => S.modules.find(id => !S.modulesOff.includes(id)) || 'home';

function renderModules() {
  const g = $('#modGrid'); g.innerHTML = '';
  S.modules.forEach((id, i) => {
    const off = S.modulesOff.includes(id);
    const el = document.createElement('div');
    el.className = 'mod' + (off ? ' off' : '');
    el.innerHTML = `${svg(id)}<span class="nm">${MODULES[id]}</span>
      <button class="mv" data-d="-1" title="Mover para a esquerda">‹</button>
      <button class="mv" data-d="1" title="Mover para a direita">›</button>
      <button class="switch ${off ? '' : 'on'}" title="${off ? 'Adicionar' : 'Remover'}"></button>`;
    el.querySelector('.switch').addEventListener('click', () => {
      const ativos = S.modules.filter(m => !S.modulesOff.includes(m));
      if (!off && ativos.length <= 1) return; // mantém pelo menos uma aba
      saveSetting('modulesOff', off ? S.modulesOff.filter(m => m !== id) : [...S.modulesOff, id]);
      renderModules(); renderNav();
    });
    el.querySelectorAll('.mv').forEach(b => b.addEventListener('click', () => {
      const j = i + +b.dataset.d;
      if (j < 0 || j >= S.modules.length) return;
      const arr = [...S.modules];[arr[i], arr[j]] = [arr[j], arr[i]];
      saveSetting('modules', arr); renderModules(); renderNav();
    }));
    g.appendChild(el);
  });
}

/* =============== menu ••• =============== */
const menu = $('#menu');
function closeMenu() { menu.classList.remove('open'); }
$('#menuBtn').addEventListener('click', (e) => { e.stopPropagation(); menu.classList.toggle('open'); });
document.addEventListener('click', (e) => { if (!menu.contains(e.target)) closeMenu(); });
$('#optLogin').addEventListener('change', (e) => api.setLogin(e.target.checked));
$('#optPin').addEventListener('change', (e) => { pinned = e.target.checked; if (!pinned && !island.matches(':hover')) collapse(true); });
$('#optSettings').addEventListener('click', () => { closeMenu(); showTab('settings'); });
$('#optQuit').addEventListener('click', () => api.quit());
$('#optAdmin').addEventListener('click', () => { closeMenu(); api.relaunchAdmin(); });

/* =============== avisos na pílula =============== */
let toast = null, toastT = null;
function showToast(o) {
  toast = o;
  island.classList.add('toasting');
  island.classList.toggle('alert', !!o.alert);
  clearTimeout(toastT);
  toastT = setTimeout(() => { toast = null; island.classList.remove('toasting', 'alert'); }, o.ms || 2500);
  renderPill();
}
function alertIsland(title, body, icon = '🔔') {
  api.notify(title, body || '');
  showToast({ icon, text: body ? `${title} · ${body}` : title, alert: true, ms: 6000 });
}

/* =============== tema =============== */
function applyTheme() {
  root.style.setProperty('--accent', S.accent);
  root.style.setProperty('--alpha', S.opacity / 100);
  root.style.setProperty('--scale', { compacta: 0.88, normal: 1, grande: 1.12 }[S.size] || 1);
  applyHue();
  $$('#swatches .sw').forEach(b => b.classList.toggle('on', b.dataset.color === S.accent));
  $$('#sizeSeg button').forEach(b => b.classList.toggle('on', b.dataset.size === S.size));
  $('#opRange').value = S.opacity; setFill($('#opRange'));
  $('#opVal').textContent = S.opacity + '%';
}
function setFill(r) { r.style.setProperty('--p', ((r.value - r.min) / (r.max - r.min) * 100) + '%'); }

function renderSettings() {
  const sw = $('#swatches'); sw.innerHTML = '';
  ACCENTS.forEach(c => {
    const b = document.createElement('button');
    b.className = 'sw'; b.dataset.color = c; b.style.background = c;
    b.addEventListener('click', () => { saveSetting('accent', c); applyTheme(); });
    sw.appendChild(b);
  });
  $$('#sizeSeg button').forEach(b => b.onclick = () => { saveSetting('size', b.dataset.size); applyTheme(); });
  $('#opRange').oninput = (e) => { S.opacity = +e.target.value; applyTheme(); };
  $('#opRange').onchange = (e) => saveSetting('opacity', +e.target.value);
  $('#optFollow').checked = S.followMusic;
  $('#optFollow').onchange = (e) => { saveSetting('followMusic', e.target.checked); applyHue(); };
  $$('[data-alert]').forEach(c => {
    c.checked = S.alerts[c.dataset.alert] !== false;
    c.onchange = () => saveSetting('alerts', { ...S.alerts, [c.dataset.alert]: c.checked });
  });
  $('#cityInput').value = S.city ? S.city.name : '';
  $('#cityMsg').textContent = S.city ? `Fixa: ${S.city.name}` : 'Automática (pela sua conexão)';
  applyTheme();
}
$('#cityForm').addEventListener('submit', async (e) => {
  e.preventDefault();
  const q = $('#cityInput').value.trim();
  if (!q) { saveSetting('city', null); autoLoc = null; $('#cityMsg').textContent = 'Automática (pela sua conexão)'; loadWeather(); return; }
  $('#cityMsg').textContent = 'Buscando…';
  try {
    const j = await (await fetch(`https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=1&language=pt`)).json();
    const r = j.results && j.results[0];
    if (!r) { $('#cityMsg').textContent = 'Cidade não encontrada'; return; }
    saveSetting('city', { name: r.name, lat: r.latitude, lon: r.longitude });
    $('#cityMsg').textContent = `Fixa: ${r.name}${r.admin1 ? ', ' + r.admin1 : ''}`;
    loadWeather();
  } catch { $('#cityMsg').textContent = 'Sem conexão'; }
});

/* =============== clima (Open-Meteo, sem chave) =============== */
const WMO = (c, day = 1) => {
  if (c === 0) return day ? ['☀️', 'Céu limpo'] : ['🌙', 'Céu limpo'];
  if (c === 1) return day ? ['🌤️', 'Poucas nuvens'] : ['🌙', 'Poucas nuvens'];
  if (c === 2) return ['⛅', 'Parcialmente nublado'];
  if (c === 3) return ['☁️', 'Nublado'];
  if (c === 45 || c === 48) return ['🌫️', 'Neblina'];
  if (c >= 51 && c <= 57) return ['🌦️', 'Garoa'];
  if (c >= 61 && c <= 67) return ['🌧️', 'Chuva'];
  if ((c >= 71 && c <= 77) || c === 85 || c === 86) return ['🌨️', 'Neve'];
  if (c >= 80 && c <= 82) return ['🌦️', 'Pancadas de chuva'];
  if (c >= 95) return ['⛈️', 'Tempestade'];
  return ['🌡️', '—'];
};
let wx = null, wxCity = '', autoLoc = null, tempText = '';
async function getLocation() {
  if (S.city) return S.city;
  if (autoLoc) return autoLoc;
  try {
    const j = await (await fetch('https://ipapi.co/json/')).json();
    if (j.latitude) return (autoLoc = { name: j.city, lat: j.latitude, lon: j.longitude });
  } catch { }
  return { name: 'São Paulo', lat: -23.55, lon: -46.63 };
}
async function loadWeather() {
  const loc = await getLocation();
  wxCity = loc.name;
  try {
    const u = `https://api.open-meteo.com/v1/forecast?latitude=${loc.lat}&longitude=${loc.lon}` +
      `&current=temperature_2m,apparent_temperature,relative_humidity_2m,wind_speed_10m,weather_code,is_day` +
      `&hourly=temperature_2m,weather_code,precipitation_probability` +
      `&daily=weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max&timezone=auto&forecast_days=6`;
    wx = await (await fetch(u)).json();
    renderWeather();
  } catch { $('#wxDesc').textContent = 'Sem conexão'; }
}
function renderWeather() {
  if (!wx || !wx.current) return;
  const c = wx.current, [ic, desc] = WMO(c.weather_code, c.is_day);
  tempText = `${ic} ${Math.round(c.temperature_2m)}°`;
  $('#wxIcon').textContent = ic;
  $('#wxTemp').textContent = `${Math.round(c.temperature_2m)}°`;
  $('#wxDesc').textContent = desc;
  $('#wxCity').textContent = wxCity;
  $('#wxDetails').innerHTML = `SENSAÇÃO ${Math.round(c.apparent_temperature)}°<br>UMIDADE ${c.relative_humidity_2m}%<br>VENTO ${Math.round(c.wind_speed_10m)} KM/H`;

  const h = wx.hourly, now = c.time.slice(0, 13);
  let i0 = h.time.findIndex(t => t.slice(0, 13) >= now); if (i0 < 0) i0 = 0;
  $('#wxHours').innerHTML = h.time.slice(i0, i0 + 8).map((t, k) => {
    const i = i0 + k;
    return `<div class="wx-cell"><span class="k">${k === 0 ? 'AGORA' : t.slice(11, 13) + 'H'}</span>
      <span class="e">${WMO(h.weather_code[i])[0]}</span><span>${Math.round(h.temperature_2m[i])}°</span>
      <span class="r">${h.precipitation_probability[i] ?? 0}%</span></div>`;
  }).join('');
  const d = wx.daily;
  $('#wxDays').innerHTML = d.time.map((t, i) => `<div class="wx-cell">
      <span class="k">${i === 0 ? 'HOJE' : DIAS_CURTOS[new Date(t + 'T12:00').getDay()].toUpperCase()}</span>
      <span class="e">${WMO(d.weather_code[i])[0]}</span>
      <span>${Math.round(d.temperature_2m_max[i])}° <span class="mn">${Math.round(d.temperature_2m_min[i])}°</span></span>
      <span class="r">${d.precipitation_probability_max[i] ?? 0}%</span></div>`).join('');
}
setInterval(loadWeather, 30 * 60 * 1000);

/* =============== cronômetro =============== */
const sw = { running: false, start: 0, acc: 0 };
const swElapsed = () => (sw.acc + (sw.running ? Date.now() - sw.start : 0)) / 1000;
let swClickT = null;
$('#swCard').addEventListener('click', () => {
  clearTimeout(swClickT);
  swClickT = setTimeout(() => {
    if (sw.running) { sw.acc += Date.now() - sw.start; sw.running = false; }
    else { sw.start = Date.now(); sw.running = true; }
  }, 220);
});
$('#swCard').addEventListener('dblclick', () => { clearTimeout(swClickT); sw.running = false; sw.acc = 0; });

/* =============== foco (pomodoro) =============== */
const FOCO = 25 * 60, PAUSA = 5 * 60;
const focus = { mode: 'focus', left: FOCO, running: false, endAt: 0 };
$('#focusBtn').addEventListener('click', (e) => {
  e.stopPropagation();
  if (focus.running) { focus.left = (focus.endAt - Date.now()) / 1000; focus.running = false; }
  else { focus.endAt = Date.now() + focus.left * 1000; focus.running = true; }
});
$('#focusCard').addEventListener('dblclick', () => { focus.running = false; focus.mode = 'focus'; focus.left = FOCO; });
const focusLeft = () => focus.running ? (focus.endAt - Date.now()) / 1000 : focus.left;
function checkFocus() {
  if (!focus.running || focusLeft() > 0) return;
  if (focus.mode === 'focus') { focus.mode = 'break'; focus.left = PAUSA; alertIsland('Foco concluído!', 'Pausa de 5 minutos', '🎯'); }
  else { focus.mode = 'focus'; focus.left = FOCO; alertIsland('Pausa encerrada', 'Bora para mais um ciclo', '💪'); }
  focus.endAt = Date.now() + focus.left * 1000;
  focus.running = focus.mode === 'break';
}

/* =============== timer =============== */
const tm = { total: 5 * 60, left: 5 * 60, running: false, endAt: 0 };
const tmLeft = () => tm.running ? (tm.endAt - Date.now()) / 1000 : tm.left;
$$('.presets [data-min]').forEach(b => b.addEventListener('click', () => { tm.running = false; tm.total = tm.left = +b.dataset.min * 60; }));
$('.presets [data-add]').addEventListener('click', () => { if (tm.running) tm.endAt += 60000; else tm.left += 60; });
$('#tmStart').addEventListener('click', () => {
  if (tm.running) { tm.left = tmLeft(); tm.running = false; }
  else if (tm.left > 0) { tm.endAt = Date.now() + tm.left * 1000; tm.running = true; }
});
$('#tmReset').addEventListener('click', () => { tm.running = false; tm.left = tm.total; });
function checkTimer() {
  if (tm.running && tmLeft() <= 0) { tm.running = false; tm.left = 0; alertIsland('Timer finalizado', `${fmt(tm.total)} se passaram`, '⏰'); }
}

/* =============== mídia + capa do álbum =============== */
let media = { title: '', artist: '', status: 'None', cover: '' }, lastTrack = null, coverHue = null, lastCover = '';
const hashHue = (s) => { let h = 0; for (const c of s) h = (h * 31 + c.charCodeAt(0)) % 360; return h; };
function rgbToHsl(r, g, b) {
  r /= 255; g /= 255; b /= 255;
  const mx = Math.max(r, g, b), mn = Math.min(r, g, b), l = (mx + mn) / 2, d = mx - mn;
  if (!d) return [0, 0, l];
  const s = d / (1 - Math.abs(2 * l - 1));
  let h = mx === r ? ((g - b) / d) % 6 : mx === g ? (b - r) / d + 2 : (r - g) / d + 4;
  return [(h * 60 + 360) % 360, s, l];
}
function hueFromImage(src) {
  return new Promise(res => {
    const im = new Image();
    im.onload = () => {
      const c = document.createElement('canvas'); c.width = c.height = 24;
      const x = c.getContext('2d'); x.drawImage(im, 0, 0, 24, 24);
      const d = x.getImageData(0, 0, 24, 24).data;
      let sx = 0, sy = 0, w = 0;
      for (let i = 0; i < d.length; i += 4) {
        const [h, s, l] = rgbToHsl(d[i], d[i + 1], d[i + 2]);
        const wt = s * (1 - Math.abs(l - 0.5) * 2);
        if (wt < 0.08) continue;
        sx += Math.cos(h * Math.PI / 180) * wt; sy += Math.sin(h * Math.PI / 180) * wt; w += wt;
      }
      res(w ? Math.round((Math.atan2(sy, sx) * 180 / Math.PI + 360) % 360) : null);
    };
    im.onerror = () => res(null);
    im.src = src;
  });
}
function applyHue() {
  const playing = media.status === 'Playing' && media.title;
  let h = hexHue(S.accent);
  if (S.followMusic && media.title) h = coverHue ?? hashHue(media.title + media.artist);
  root.style.setProperty('--h', h);
  root.style.setProperty('--glow', S.followMusic && playing ? 1 : 0.35);
}
api.onMedia((m) => {
  const key = m.title + '|' + m.artist;
  const changed = lastTrack !== null && key !== lastTrack;
  const first = lastTrack === null;
  lastTrack = key; media = m;
  const has = !!m.title;
  $('#mTitle').textContent = has ? m.title : 'Nada tocando';
  $('#mArtist').textContent = has ? (m.artist || m.app) : 'Abra o Spotify, YouTube…';
  $('#mPlay').textContent = m.status === 'Playing' ? '❚❚' : '▶';
  const card = $('#mediaCard');
  if (m.cover) {
    if (m.cover !== lastCover) {
      lastCover = m.cover;
      $('#mCover').src = m.cover; $('#mCoverBg').src = m.cover;
      hueFromImage(m.cover).then(h => { coverHue = h; applyHue(); });
    }
    card.classList.add('has-cover');
  } else {
    if (changed || !has) { card.classList.remove('has-cover'); coverHue = null; lastCover = ''; }
  }
  applyHue();
  if (changed && has && m.status === 'Playing' && S.alerts.track !== false && !first) {
    showToast({ img: m.cover, icon: m.cover ? '' : '🎵', text: `${m.title}${m.artist ? ' — ' + m.artist : ''}`, ms: 3500 });
  }
});
$$('.ctl').forEach(b => b.addEventListener('click', () => api.mediaCmd(b.dataset.cmd)));

/* =============== volume, microfone e dispositivos =============== */
let vol = null, volDrag = false, micDrag = false;
function throttled(fn, ms = 70) { let t = null, last; return (v) => { last = v; if (!t) t = setTimeout(() => { t = null; fn(last); }, ms); }; }
const sendVol = throttled(v => api.setVol(v));
const sendMic = throttled(v => api.setMic(v));
const sendBright = throttled(v => api.setBright(v), 150);

api.onVol((v) => {
  const prev = vol; vol = v;
  if (!volDrag) { $('#volRange').value = v.level; setFill($('#volRange')); }
  $('#volVal').textContent = v.mute ? 'Mudo' : v.level + '%';
  $('#muteBtn').textContent = v.mute ? '🔇' : '🔊';
  const hasMic = v.mic >= 0;
  $('#micRange').disabled = !hasMic;
  if (!micDrag && hasMic) { $('#micRange').value = v.mic; setFill($('#micRange')); }
  $('#micVal').textContent = !hasMic ? '—' : v.micMute ? 'Mudo' : v.mic + '%';
  $('#micBtn').textContent = v.micMute ? '🚫' : '🎙️';
  $('#micBtn').title = v.micMute ? 'Ativar microfone' : 'Silenciar microfone';
  if (prev && !expanded && S.alerts.vol !== false) {
    if (prev.level !== v.level || prev.mute !== v.mute)
      showToast({ icon: v.mute ? '🔇' : '🔊', text: v.mute ? 'Mudo' : `Volume ${v.level}%`, progress: v.mute ? 0 : v.level, ms: 1800 });
    else if (prev.micMute !== v.micMute)
      showToast({ icon: v.micMute ? '🚫' : '🎙️', text: v.micMute ? 'Microfone silenciado' : 'Microfone ativo', ms: 2200 });
  }
});
$('#volRange').addEventListener('input', (e) => { volDrag = true; setFill(e.target); $('#volVal').textContent = e.target.value + '%'; sendVol(+e.target.value); });
$('#volRange').addEventListener('change', () => { volDrag = false; });
$('#muteBtn').addEventListener('click', () => api.toggleMute());
$('#micRange').addEventListener('input', (e) => { micDrag = true; setFill(e.target); $('#micVal').textContent = e.target.value + '%'; sendMic(+e.target.value); });
$('#micRange').addEventListener('change', () => { micDrag = false; });
$('#micBtn').addEventListener('click', () => api.toggleMic());

let devices = null;
function devIcon(name, isIn) {
  const n = name.toLowerCase();
  if (isIn) return /fone|headset|airpods|buds|bluetooth/.test(n) ? '🎧' : '🎙️';
  if (/fone|headphone|headset|airpods|buds|bluetooth/.test(n)) return '🎧';
  if (/hdmi|monitor|display|tv/.test(n)) return '🖥️';
  return '🔈';
}
function renderDevList(el, list, isIn) {
  if (!list || !list.length) { el.innerHTML = `<div class="empty">${isIn ? 'Nenhum microfone encontrado' : 'Nenhuma saída encontrada'}</div>`; return; }
  el.innerHTML = '';
  list.forEach(d => {
    const b = document.createElement('button');
    b.className = 'dev' + (d.def ? ' def' : '');
    b.title = d.def ? 'Dispositivo padrão' : 'Usar este dispositivo';
    b.innerHTML = `<span class="radio"></span><span>${devIcon(d.name, isIn)}</span><span class="nm"></span>`;
    b.querySelector('.nm').textContent = d.name;
    b.addEventListener('click', () => { if (!d.def) api.setDevice(d.id); });
    el.appendChild(b);
  });
}
api.onDevices((d) => {
  const prev = devices; devices = d;
  renderDevList($('#outList'), d.out, false);
  renderDevList($('#inList'), d.in, true);
  if (!prev || S.alerts.dev === false) return;
  const def = (l) => (l.find(x => x.def) || {});
  const ids = (l) => l.map(x => x.id);
  const novoOut = d.out.find(x => !ids(prev.out).includes(x.id));
  const saiuOut = prev.out.find(x => !ids(d.out).includes(x.id));
  if (novoOut) showToast({ icon: devIcon(novoOut.name), text: `Conectado: ${novoOut.name}`, ms: 3000 });
  else if (saiuOut) showToast({ icon: '🔌', text: `Desconectado: ${saiuOut.name}`, ms: 3000 });
  else if (def(prev.out).id !== def(d.out).id && def(d.out).name) showToast({ icon: devIcon(def(d.out).name), text: `Saída: ${def(d.out).name}`, ms: 2500 });
  else if (def(prev.in).id !== def(d.in).id && def(d.in).name) showToast({ icon: '🎙️', text: `Microfone: ${def(d.in).name}`, ms: 2500 });
});

api.onBright((b) => {
  const ok = b.level !== null && b.level !== undefined;
  $('#brightRange').disabled = !ok;
  $('#brightVal').textContent = ok ? b.level + '%' : 'Indisponível';
  $('#brightRange').title = ok ? '' : 'Monitores externos normalmente não aceitam ajuste de brilho pelo Windows';
  if (ok && document.activeElement !== $('#brightRange')) { $('#brightRange').value = b.level; setFill($('#brightRange')); }
});
$('#brightRange').addEventListener('input', (e) => { setFill(e.target); $('#brightVal').textContent = e.target.value + '%'; sendBright(+e.target.value); });

/* =============== ações personalizadas =============== */
let actEditing = false, actEditId = null;
const PLACEHOLDER = { url: 'https://…', file: 'C:\\Caminho\\programa.exe', folder: 'C:\\Users\\…\\Documentos', cmd: 'calc, notepad, ms-settings:, spotify:' };
function renderActions() {
  const g = $('#actGrid'); g.innerHTML = '';
  $('[data-tab=actions]').classList.toggle('editing', actEditing);
  $('#actEdit').textContent = actEditing ? 'Concluir' : 'Editar';
  $('#actEdit').classList.toggle('on', actEditing);
  S.actions.forEach(a => {
    const b = document.createElement('button');
    b.className = 'act';
    b.innerHTML = `<span class="em"></span><span class="nm"></span><span class="del" title="Remover">✕</span>`;
    b.querySelector('.em').textContent = a.emoji || '⚡';
    b.querySelector('.nm').textContent = a.name;
    b.title = actEditing ? 'Clique para editar' : a.target;
    b.addEventListener('click', (e) => {
      if (e.target.classList.contains('del')) {
        saveSetting('actions', S.actions.filter(x => x.id !== a.id)); renderActions(); return;
      }
      if (actEditing) { openActForm(a); return; }
      api.runAction(a);
      b.animate([{ transform: 'scale(.9)' }, { transform: 'scale(1)' }], { duration: 200 });
    });
    g.appendChild(b);
  });
  const add = document.createElement('button');
  add.className = 'act new';
  add.innerHTML = '<span class="em">＋</span><span class="nm">Nova ação</span>';
  add.addEventListener('click', () => openActForm(null));
  g.appendChild(add);
}
function openActForm(a) {
  actEditId = a ? a.id : null;
  $('#actEmoji').value = a ? a.emoji : '';
  $('#actName').value = a ? a.name : '';
  $('#actType').value = a ? a.type : 'url';
  $('#actTarget').value = a ? a.target : '';
  updateActType();
  $('[data-tab=actions]').classList.add('adding');
  $('#actForm').classList.add('open');
  $('#actName').focus();
}
function closeActForm() {
  $('[data-tab=actions]').classList.remove('adding');
  $('#actForm').classList.remove('open');
}
function updateActType() {
  const t = $('#actType').value;
  $('#actTarget').placeholder = PLACEHOLDER[t];
  $('#actPick').style.display = (t === 'file' || t === 'folder') ? '' : 'none';
}
$('#actType').addEventListener('change', updateActType);
$('#actEdit').addEventListener('click', () => { actEditing = !actEditing; closeActForm(); renderActions(); });
$('#actCancel').addEventListener('click', closeActForm);
$('#actPick').addEventListener('click', async () => {
  const p = await api.pickPath($('#actType').value === 'folder' ? 'folder' : 'file');
  if (p) {
    $('#actTarget').value = p;
    if (!$('#actName').value) $('#actName').value = p.split(/[\\/]/).pop().replace(/\.(exe|lnk)$/i, '');
  }
});
$('#actForm').addEventListener('submit', (e) => {
  e.preventDefault();
  let target = $('#actTarget').value.trim();
  const type = $('#actType').value;
  if (type === 'url' && !/^[a-z][\w+.-]*:/i.test(target)) target = 'https://' + target;
  const emoji = $('#actEmoji').value.trim() || { url: '🌐', file: '🚀', folder: '📁', cmd: '⚡' }[type];
  const item = { id: actEditId || Date.now(), emoji, name: $('#actName').value.trim(), type, target };
  const list = actEditId ? S.actions.map(x => x.id === actEditId ? item : x) : [...S.actions, item];
  saveSetting('actions', list);
  closeActForm(); renderActions();
});

/* =============== clipboard =============== */
let clips = [];
function kindOf(t) {
  t = t.trim();
  if (/^\S+@\S+\.\S+$/.test(t)) return 'MAIL';
  if (/^https?:\/\//i.test(t)) return 'LINK';
  if (/^[\d\s().+-]{8,}$/.test(t)) return 'NÚMERO';
  return 'TEXTO';
}
function ago(t) {
  const s = (Date.now() - t) / 1000;
  if (s < 60) return 'AGORA';
  if (s < 3600) return `${Math.floor(s / 60)} MIN`;
  return `${Math.floor(s / 3600)} H`;
}
function renderClips() {
  const list = $('#clipList');
  if (!clips.length) { list.innerHTML = '<div class="empty">Copie algo (Ctrl+C) e aparece aqui. Fica só na memória.</div>'; return; }
  list.innerHTML = '';
  clips.forEach(c => {
    const el = document.createElement('div');
    el.className = 'clip';
    el.innerHTML = `<span class="kind"></span><span class="txt"></span><span class="ago"></span>`;
    el.querySelector('.kind').textContent = kindOf(c.text);
    el.querySelector('.txt').textContent = c.text.replace(/\s+/g, ' ');
    el.querySelector('.ago').textContent = ago(c.t);
    el.title = 'Clique para copiar de novo';
    el.addEventListener('click', () => {
      api.copy(c.text);
      el.classList.add('copied'); el.querySelector('.ago').textContent = 'COPIADO';
      setTimeout(renderClips, 1200);
    });
    list.appendChild(el);
  });
}
api.onClips((l) => { clips = l; renderClips(); });
$('#clearClips').addEventListener('click', () => api.clearClips());
renderClips();
setInterval(renderClips, 60000);

/* =============== notas =============== */
const notes = $('#notes');
let notesT;
notes.addEventListener('input', () => { clearTimeout(notesT); notesT = setTimeout(() => api.saveNotes(notes.value), 400); });

/* =============== agenda + lembretes =============== */
let calRef = new Date(); calRef.setDate(1);
let selDay = ymd(new Date());
let reminders = [];
const remDate = (r) => new Date(`${r.date}T${r.time}:00`);
function saveRem() { api.saveReminders(reminders); }
function renderCal() {
  const y = calRef.getFullYear(), m = calRef.getMonth();
  $('#calTitle').textContent = `${MESES_LONGOS[m]} ${y}`;
  const grid = $('#calGrid'); grid.innerHTML = '';
  ['D', 'S', 'T', 'Q', 'Q', 'S', 'S'].forEach(d => grid.insertAdjacentHTML('beforeend', `<span class="wd">${d}</span>`));
  const start = new Date(y, m, 1 - new Date(y, m, 1).getDay());
  const today = ymd(new Date());
  const withRem = new Set(reminders.filter(r => !r.fired).map(r => r.date));
  for (let i = 0; i < 42; i++) {
    const d = new Date(start); d.setDate(start.getDate() + i);
    const ds = ymd(d), cls = ['d'];
    if (d.getMonth() !== m) cls.push('out');
    if (d.getDay() === 0 || d.getDay() === 6) cls.push('we');
    if (ds === today) cls.push('today');
    if (ds === selDay) cls.push('sel');
    if (withRem.has(ds)) cls.push('has');
    const el = document.createElement('span');
    el.className = cls.join(' '); el.textContent = d.getDate();
    el.addEventListener('click', () => { selDay = ds; renderCal(); renderDay(); });
    grid.appendChild(el);
  }
}
function renderDay() {
  const d = new Date(selDay + 'T12:00');
  const isToday = selDay === ymd(new Date());
  $('#dayTitle').textContent = `Lembretes · ${isToday ? 'Hoje' : DIAS_CURTOS[d.getDay()] + ' ' + d.getDate() + ' ' + MESES_LONGOS[d.getMonth()].slice(0, 3)}`;
  const list = $('#remList');
  const items = reminders.filter(r => r.date === selDay).sort((a, b) => a.time.localeCompare(b.time));
  if (!items.length) { list.innerHTML = '<div class="empty" style="padding:14px 4px">Nenhum lembrete neste dia</div>'; return; }
  list.innerHTML = '';
  items.forEach(r => {
    const el = document.createElement('div');
    el.className = 'rem' + (r.fired ? ' done' : '');
    el.innerHTML = `<span class="t">${r.time}</span><span class="tx"></span><button class="x" title="Excluir">✕</button>`;
    el.querySelector('.tx').textContent = r.text;
    el.querySelector('.x').addEventListener('click', () => {
      reminders = reminders.filter(x => x.id !== r.id); saveRem(); renderCal(); renderDay();
    });
    list.appendChild(el);
  });
}
$('#remForm').addEventListener('submit', (e) => {
  e.preventDefault();
  const text = $('#remText').value.trim(), time = $('#remTime').value || '09:00';
  if (!text) return;
  const r = { id: Date.now(), date: selDay, time, text, fired: false };
  r.fired = remDate(r) <= new Date();
  reminders.push(r); saveRem();
  $('#remText').value = '';
  renderCal(); renderDay();
});
$('#calPrev').addEventListener('click', () => { calRef.setMonth(calRef.getMonth() - 1); renderCal(); });
$('#calNext').addEventListener('click', () => { calRef.setMonth(calRef.getMonth() + 1); renderCal(); });
function checkReminders() {
  const now = new Date();
  let changed = false;
  reminders.forEach(r => {
    if (!r.fired && remDate(r) <= now) { r.fired = true; changed = true; alertIsland('Lembrete', r.text, '⏰'); }
  });
  if (changed) { saveRem(); if (currentTab === 'cal') { renderCal(); renderDay(); } }
}

/* =============== sistema =============== */
const gb = (b) => (b / 1024 ** 3).toFixed(1);
api.onSys((s) => {
  $('#cpuBar').style.width = s.cpu + '%'; $('#cpuVal').textContent = s.cpu + '%';
  $('#ramBar').style.width = (100 * s.memUsed / s.memTotal) + '%';
  $('#ramVal').textContent = `${gb(s.memUsed)}/${gb(s.memTotal)} GB`;
  $('#cpuName').textContent = s.cpuName;
  const h = Math.floor(s.uptime / 3600), mi = Math.floor((s.uptime % 3600) / 60);
  $('#uptime').textContent = `LIGADO HÁ ${h}H ${mi}MIN`;
});
if (navigator.getBattery) {
  navigator.getBattery().then(b => {
    let warned20 = false, warned10 = false;
    const upd = () => {
      const p = Math.round(b.level * 100);
      $('#batBar').style.width = p + '%';
      $('#batVal').textContent = `${p}%${b.charging ? ' ⚡' : ''}`;
      if (b.charging) { warned20 = warned10 = false; return; }
      if (S.alerts.bat === false) return;
      if (p <= 10 && !warned10) { warned10 = warned20 = true; alertIsland('Bateria crítica', `${p}% — conecte o carregador`, '🪫'); }
      else if (p <= 20 && !warned20) { warned20 = true; alertIsland('Bateria fraca', `${p}% restantes`, '🪫'); }
    };
    upd();
    b.addEventListener('levelchange', upd);
    b.addEventListener('chargingchange', () => {
      upd();
      if (S.alerts.bat === false) return;
      const p = Math.round(b.level * 100);
      showToast({ icon: b.charging ? '⚡' : '🔋', text: b.charging ? `Carregando · ${p}%` : `Na bateria · ${p}%`, progress: p, ms: 2500 });
    });
  });
}


/* =============== limpeza =============== */
const fmtSize = (b) => {
  if (!b) return '0 MB';
  if (b >= 1024 ** 3) return (b / 1024 ** 3).toFixed(1).replace('.', ',') + ' GB';
  if (b >= 1024 ** 2) return Math.round(b / 1024 ** 2) + ' MB';
  return Math.max(1, Math.round(b / 1024)) + ' KB';
};
let cleanData = null, cleanBusy = false, isAdmin = false;
function renderDisk(d) {
  if (!d) return;
  const used = d.total - d.free, pct = 100 * used / d.total;
  $('#diskBar').style.width = pct + '%';
  $('#diskBar').parentElement.classList.toggle('full', pct > 90);
  $('#diskVal').textContent = `${fmtSize(d.free)} livres de ${fmtSize(d.total)}`;
}
function renderClean(busyKeys = []) {
  const g = $('#cleanGrid'); g.innerHTML = '';
  if (!cleanData) return;
  let total = 0;
  Object.entries(cleanData.cats).forEach(([k, c]) => {
    const on = S.cleanSel.includes(k);
    if (on) total += c.size || 0;
    const el = document.createElement('label');
    el.className = 'crow' + (busyKeys.includes(k) ? ' busy' : '');
    const lock = c.admin && !isAdmin ? ' 🔒' : '';
    el.title = c.admin && !isAdmin ? 'Precisa do app aberto como administrador (menu •••)' :
      k === 'recycle' ? 'Apaga definitivamente o que está na lixeira' : `${c.count || 0} itens`;
    el.innerHTML = `<input type="checkbox" ${on ? 'checked' : ''}><span>${c.icon}</span><span class="nm"></span><span class="sz"></span>`;
    el.querySelector('.nm').textContent = c.name + lock;
    const sz = el.querySelector('.sz');
    sz.textContent = c.denied && !c.size ? 'sem acesso' : fmtSize(c.size);
    sz.classList.toggle('big', (c.size || 0) > 200 * 1024 ** 2);
    el.querySelector('input').addEventListener('change', (e) => {
      saveSetting('cleanSel', e.target.checked ? [...new Set([...S.cleanSel, k])] : S.cleanSel.filter(x => x !== k));
      renderClean();
    });
    g.appendChild(el);
  });
  $('#cleanRun').textContent = total ? `Limpar ${fmtSize(total)}` : 'Limpar';
  $('#cleanRun').disabled = cleanBusy || !S.cleanSel.length;
}
async function runScan() {
  if (cleanBusy) return;
  cleanBusy = true;
  $('#cleanScan').disabled = true; $('#cleanRun').disabled = true;
  $('#cleanMsg').textContent = 'Analisando…';
  if (cleanData) renderClean(Object.keys(cleanData.cats));
  try {
    cleanData = await api.cleanScan();
    renderDisk(cleanData.disk);
    const tot = Object.values(cleanData.cats).reduce((a, c) => a + (c.size || 0), 0);
    $('#cleanMsg').textContent = `${fmtSize(tot)} encontrados no total`;
  } catch { $('#cleanMsg').textContent = 'Não foi possível analisar'; }
  cleanBusy = false;
  $('#cleanScan').disabled = false;
  renderClean();
}
$('#cleanScan').addEventListener('click', runScan);
$('#cleanRun').addEventListener('click', async () => {
  if (cleanBusy || !S.cleanSel.length) return;
  if (S.cleanSel.includes('recycle') && cleanData && cleanData.cats.recycle && cleanData.cats.recycle.count > 0 &&
      $('#cleanRun').dataset.confirm !== '1') {
    $('#cleanRun').dataset.confirm = '1';
    $('#cleanRun').textContent = 'Esvaziar lixeira também?';
    setTimeout(() => { $('#cleanRun').dataset.confirm = ''; renderClean(); }, 3500);
    return;
  }
  $('#cleanRun').dataset.confirm = '';
  cleanBusy = true;
  $('#cleanScan').disabled = true; $('#cleanRun').disabled = true;
  $('#cleanMsg').textContent = 'Limpando… arquivos em uso são pulados';
  renderClean(S.cleanSel);
  const r = await api.cleanRun(S.cleanSel);
  cleanBusy = false;
  const msg = r && r.freed ? `${fmtSize(r.freed)} liberados` : 'Limpeza concluída';
  showToast({ icon: '🧹', text: msg, ms: 3500 });
  cleanData = null;
  await runScan();
  $('#cleanMsg').textContent = `✓ ${msg}`;
});
$('#ramBtn').addEventListener('click', async () => {
  const b = $('#ramBtn'); b.disabled = true; b.textContent = '🧠 Otimizando…';
  const r = await api.trimRam();
  b.disabled = false; b.textContent = '🧠 Otimizar memória';
  const msg = r && r.freed > 50 * 1024 ** 2 ? `${fmtSize(r.freed)} de RAM liberados` : 'Memória otimizada';
  $('#cleanMsg').textContent = `✓ ${msg}`;
  showToast({ icon: '🧠', text: msg, ms: 3000 });
});
$('#dnsBtn').addEventListener('click', async () => {
  const r = await api.flushDns();
  const msg = r && r.ok ? 'Cache DNS limpo' : 'Não foi possível limpar o DNS';
  $('#cleanMsg').textContent = (r && r.ok ? '✓ ' : '') + msg;
  showToast({ icon: '🌐', text: msg, ms: 2500 });
});

/* =============== processos =============== */
let procs = [], protectedNames = new Set(), ownPids = [], selfName = '';
let confirmKey = null, confirmT = null, procFilter = '';
const isProtected = (name, pid) => {
  const n = name.toLowerCase();
  return protectedNames.has(n) || n === selfName || (pid && ownPids.includes(pid));
};
function renderProcs() {
  const list = $('#procList');
  $$('#procSort button').forEach(b => b.classList.toggle('on', b.dataset.s === S.procSort));
  if (!procs.length) { list.innerHTML = '<div class="empty">Carregando processos…</div>'; return; }
  let rows;
  if ($('#procGroup').checked) {
    const map = new Map();
    procs.forEach(p => {
      const k = p.n.toLowerCase();
      const g = map.get(k) || { n: p.n, c: 0, m: 0, count: 0, pids: [] };
      g.c += p.c; g.m += p.m; g.count++; g.pids.push(p.p);
      map.set(k, g);
    });
    rows = [...map.values()];
  } else rows = procs.map(p => ({ ...p, count: 1, pids: [p.p] }));
  const q = procFilter.toLowerCase();
  if (q) rows = rows.filter(r => r.n.toLowerCase().includes(q) || r.pids.some(p => String(p) === q));
  rows.sort((a, b) => b[S.procSort] - a[S.procSort]);
  $('#procCount').textContent = `${procs.length} ATIVOS`;
  list.innerHTML = '';
  rows.slice(0, 80).forEach(r => {
    const grouped = $('#procGroup').checked;
    const key = grouped ? 'n:' + r.n.toLowerCase() : 'p:' + r.pids[0];
    const prot = grouped ? isProtected(r.n) : isProtected(r.n, r.pids[0]);
    const el = document.createElement('div');
    el.className = 'proc-row';
    el.title = grouped ? `PIDs: ${r.pids.slice(0, 12).join(', ')}${r.pids.length > 12 ? '…' : ''}` : `PID ${r.pids[0]}`;
    el.innerHTML = `<span class="nm"></span><span class="c ${r.c >= 10 ? 'hot' : ''}">${r.c.toFixed(1).replace('.', ',')}%</span>
      <span class="m ${r.m >= 1024 ** 3 ? 'hot' : ''}">${fmtSize(r.m)}</span>` +
      (prot ? '<span class="lock" title="Processo essencial do Windows">🔒 sistema</span>'
            : `<button class="kill ${confirmKey === key ? 'confirm' : ''}">${confirmKey === key ? 'Confirmar' : 'Finalizar'}</button>`);
    el.querySelector('.nm').innerHTML = `${esc(r.n)}${r.count > 1 ? `<span class="cnt">×${r.count}</span>` : ''}`;
    const btn = el.querySelector('.kill');
    if (btn) btn.addEventListener('click', async () => {
      if (confirmKey !== key) {
        confirmKey = key; clearTimeout(confirmT);
        confirmT = setTimeout(() => { confirmKey = null; renderProcs(); }, 3000);
        renderProcs(); return;
      }
      confirmKey = null; clearTimeout(confirmT);
      btn.textContent = '…'; btn.disabled = true;
      const res = await api.kill(grouped && r.count > 1 ? { name: r.n } : { pid: r.pids[0], name: r.n });
      showToast(res && res.ok
        ? { icon: '✅', text: `${r.n} finalizado`, ms: 2500 }
        : { icon: '⚠️', text: res && res.protected ? `${r.n} é protegido` : `Não deu para finalizar ${r.n}${isAdmin ? '' : ' (tente como admin)'}`, ms: 3500 });
      setTimeout(() => api.refreshProcs(), 400);
    });
    list.appendChild(el);
  });
  if (!rows.length) list.innerHTML = '<div class="empty">Nenhum processo encontrado</div>';
}
api.onProcs((l) => { procs = l; if (currentTab === 'procs') renderProcs(); });
setInterval(() => { if (expanded && currentTab === 'procs') api.refreshProcs(); }, 3000);
$('#procSearch').addEventListener('input', (e) => { procFilter = e.target.value.trim(); renderProcs(); });
$$('#procSort button').forEach(b => b.addEventListener('click', () => { saveSetting('procSort', b.dataset.s); renderProcs(); }));
$('#procGroup').addEventListener('change', () => { confirmKey = null; renderProcs(); });

/* =============== internet =============== */
window.addEventListener('offline', () => { if (S.alerts.net !== false) showToast({ icon: '📡', text: 'Sem conexão com a internet', alert: true, ms: 5000 }); });
window.addEventListener('online', () => { if (S.alerts.net !== false) showToast({ icon: '🛜', text: 'Conectado de novo', ms: 2500 }); loadWeather(); });

/* =============== pílula =============== */
function renderPill() {
  const d = new Date();
  const hora = d.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  const playing = media.status === 'Playing' && media.title;
  const dot = $('#pillDot');
  dot.className = 'pill-dot';
  let txt = hora, icon = '', img = '', prog = null, bars = false;
  if (toast) { txt = toast.text; icon = toast.icon || ''; img = toast.img || ''; prog = toast.progress ?? null; }
  else if (playing) { txt = media.title; bars = true; }
  else if (focus.running) { txt = `${focus.mode === 'focus' ? 'Foco' : 'Pausa'} ${fmt(focusLeft())}`; dot.classList.add('on', 'focus'); }
  else if (tm.running) { txt = `Timer ${fmt(tmLeft())}`; dot.classList.add('on', 'timer'); }
  else if (sw.running) { txt = `⏱ ${fmt(swElapsed())}`; dot.classList.add('on'); }
  $('#pillText').textContent = txt;
  $('#pillIcon').textContent = img ? '' : icon;
  const pi = $('#pillImg');
  if (img) { if (pi.src !== img) pi.src = img; pi.classList.add('on'); } else pi.classList.remove('on');
  $('#pillBars').classList.toggle('on', bars);
  $('#pillProgWrap').classList.toggle('on', prog !== null);
  if (prog !== null) $('#pillProg').style.width = prog + '%';
}

/* =============== loop principal =============== */
function tick() {
  const d = new Date();
  $('#clock').textContent = d.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  $('#date').textContent = `${DIAS[d.getDay()]} ${pad(d.getDate())} ${MESES[d.getMonth()]}${tempText ? ' / ' + tempText : ''}`;

  checkFocus(); checkTimer(); checkReminders();

  $('#swTime').textContent = fmt(swElapsed());
  $('#swState').textContent = sw.running ? 'Rodando' : (sw.acc ? 'Pausado' : 'Parado');
  $('#swCard').classList.toggle('running', sw.running);

  $('#focusTime').textContent = fmt(focusLeft());
  $('#focusLabel').textContent = focus.mode === 'focus' ? 'FOCO' : 'PAUSA';
  $('#focusState').textContent = focus.running ? (focus.mode === 'focus' ? 'Focando' : 'Descansando') : 'Pronto';
  $('#focusBtn').textContent = focus.running ? '❚❚' : '▶';
  $('#focusCard').classList.toggle('break', focus.mode === 'break');

  $('#tmTime').textContent = fmt(tmLeft());
  $('#tmTime').classList.toggle('running', tm.running);
  $('#tmStart').textContent = tm.running ? 'Pausar' : 'Iniciar';

  $('.eyes').classList.toggle('busy', sw.running || focus.running || tm.running);
  renderPill();
}

/* =============== inicialização =============== */
api.getData().then(d => {
  const s = d.settings || {};
  S = { ...S, ...s, alerts: { ...DEFAULTS.alerts, ...(s.alerts || {}) } };
  // módulos novos de versões futuras entram automaticamente no fim da lista
  S.modules = [...(S.modules || []).filter(m => MODULES[m]), ...Object.keys(MODULES).filter(m => !(S.modules || []).includes(m))];
  if (!Array.isArray(S.actions)) S.actions = DEFAULT_ACTIONS;
  notes.value = d.notes || '';
  $('#optLogin').checked = !!s.openAtLogin;
  reminders = Array.isArray(d.reminders) ? d.reminders : [];
  // lembretes antigos (app estava fechado há muito tempo) são marcados sem disparar
  const limite = Date.now() - 10 * 60 * 1000;
  reminders.forEach(r => { if (!r.fired && remDate(r).getTime() < limite) r.fired = true; });
  isAdmin = !!d.isAdmin;
  island.classList.toggle('admin', isAdmin);
  protectedNames = new Set(d.protectedNames || []);
  ownPids = d.ownPids || []; selfName = (d.selfName || '').toLowerCase();
  if (!Array.isArray(S.cleanSel)) S.cleanSel = DEFAULTS.cleanSel;
  if (d.platform !== 'win32') {
    $('#mArtist').textContent = 'Leitura de mídia só funciona no Windows';
    $('#outList').innerHTML = $('#inList').innerHTML = '<div class="empty">Disponível só no Windows</div>';
  }
  renderSettings();
  renderActions();
  currentTab = firstTab();
  showTab(currentTab);
  loadWeather();
  setInterval(tick, 250);
  tick();
});
