<script setup lang="ts">
// Início: cards personalizáveis (mover, remover, adicionar), cronômetro com meta/som, mídia, foco ajustável e Google Fotos
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { api } from '../../lib/api'
import { fmt, ymd } from '../../lib/format'
import { DEFAULT_HOME_CARDS, HOME_CARDS, S, save, type HomeCardId } from '../../store/settings'
import {
  live, stopwatch, swElapsed, focus, focusLeft, setFocusMinutes,
  timer, timerLeft, meetings, now, island, showToast
} from '../../store/state'

const has = computed(() => !!live.media.title)
const playing = computed(() => live.media.status === 'Playing')

/* ---------------- Personalização dos cards ---------------- */
const editMode = ref(false)
const availableToAdd = computed(() =>
  (Object.keys(HOME_CARDS) as HomeCardId[]).filter(id => !S.homeCards.includes(id))
)
const gridStyle = computed(() => ({
  gridTemplateColumns: S.homeCards.map(id => (id === 'media' ? '1.75fr' : id === 'photos' ? '1.15fr' : '1fr')).join(' ')
}))

function moveCard(idx: number, delta: number) {
  const to = idx + delta
  if (to < 0 || to >= S.homeCards.length) return
  const next = [...S.homeCards]
  ;[next[idx], next[to]] = [next[to], next[idx]]
  save('homeCards', next)
}
function removeCard(id: HomeCardId) {
  if (S.homeCards.length <= 1) return showToast({ icon: 'ℹ️', text: 'Mantenha pelo menos 1 card na Home', ms: 2000 })
  save('homeCards', S.homeCards.filter(c => c !== id))
}
function addCard(id: HomeCardId) {
  if (S.homeCards.includes(id)) return
  save('homeCards', [...S.homeCards, id])
}
function resetCards() {
  save('homeCards', [...DEFAULT_HOME_CARDS])
}

/* ---------------- Cronômetro (com meta opcional + som) ---------------- */
const SW_GOALS = [0, 60, 300, 600, 900, 1800] // 0, 1m, 5m, 10m, 15m, 30m
function cycleGoal() {
  const cur = SW_GOALS.indexOf(stopwatch.goalSec)
  stopwatch.goalSec = SW_GOALS[(cur + 1) % SW_GOALS.length]
  stopwatch.goalFired = stopwatch.goalSec > 0 && swElapsed() >= stopwatch.goalSec
}
let clickT: number | undefined
function swClick() {
  if (editMode.value) return
  clearTimeout(clickT)
  clickT = window.setTimeout(() => {
    if (stopwatch.running) { stopwatch.acc += Date.now() - stopwatch.start; stopwatch.running = false }
    else { stopwatch.start = Date.now(); stopwatch.running = true }
  }, 220)
}
function swReset() {
  clearTimeout(clickT)
  stopwatch.running = false
  stopwatch.acc = 0
  stopwatch.goalFired = false
}

/* ---------------- Foco ajustável ---------------- */
function focusToggle() {
  if (focus.running) { focus.left = (focus.endAt - Date.now()) / 1000; focus.running = false }
  else { focus.endAt = Date.now() + focus.left * 1000; focus.running = true }
}
function focusReset() { focus.running = false; focus.mode = 'focus'; focus.left = S.focusMinutes * 60 }
function adjustFocus(deltaMin: number) {
  setFocusMinutes(S.focusMinutes + deltaMin)
}

/* ---------------- Timer rápido ---------------- */
function timerPreset(min: number) { timer.running = false; timer.total = timer.left = min * 60 }
function timerToggle() {
  if (timer.running) { timer.left = timerLeft(); timer.running = false }
  else if (timer.left > 0) { timer.endAt = Date.now() + timer.left * 1000; timer.running = true }
}
function timerReset() { timer.running = false; timer.left = timer.total }

/* ---------------- Próxima reunião ---------------- */
const nextMeetToday = computed(() => {
  const today = ymd(new Date(now.value))
  return meetings.value.find(m => m.date === today && m.endMs > now.value) || null
})

/* ---------------- Google Fotos ---------------- */
const gp = reactive({
  connected: false,
  hasClient: false,
  list: [] as string[],
  idx: 0,
  busy: false,
  cfgOpen: false,
  editClient: false,
  hover: false,
  clientId: '',
  clientSecret: ''
})

async function loadGPhotos() {
  try {
    const st = await api.gphotosStatus()
    gp.connected = st.connected
    gp.hasClient = st.has_client
    if (st.count > 0) gp.list = await api.gphotosList()
    else gp.list = []
    if (gp.idx >= gp.list.length) gp.idx = 0
  } catch { /* ignora fora do Tauri */ }
}

async function connectGoogle() {
  gp.busy = true
  try {
    if (gp.clientId.trim()) {
      gp.hasClient = await api.gphotosSetClient(gp.clientId, gp.clientSecret)
      gp.editClient = false
    }
    showToast({ icon: '🔑', text: 'Conclua o login do Google no navegador…', ms: 6000 })
    await api.gphotosConnect()
    gp.connected = true
    gp.cfgOpen = false
    showToast({ icon: '✅', text: 'Google conectado! Escolha suas fotos.', ms: 3000 })
    await pickPhotos()
  } catch (e) {
    showToast({ icon: '⚠️', text: String(e), alert: true, ms: 5000 })
  } finally {
    gp.busy = false
  }
}

async function pickPhotos() {
  gp.busy = true
  try {
    showToast({ icon: '🖼️', text: 'Selecione as fotos no navegador…', ms: 6000 })
    const imgs = await api.gphotosPick()
    gp.list = imgs
    gp.idx = 0
    gp.cfgOpen = false
    showToast(imgs.length
      ? { icon: '🖼️', text: `${imgs.length} fotos carregadas na Home`, ms: 2500 }
      : { icon: 'ℹ️', text: 'Nenhuma foto selecionada', ms: 2500 })
  } catch (e) {
    showToast({ icon: '⚠️', text: String(e), alert: true, ms: 5000 })
  } finally {
    gp.busy = false
  }
}

async function pickFolder() {
  gp.busy = true
  try {
    const imgs = await api.gphotosPickFolder()
    if (imgs.length) {
      gp.list = imgs
      gp.idx = 0
      gp.cfgOpen = false
      showToast({ icon: '🖼️', text: `${imgs.length} fotos carregadas da pasta`, ms: 2500 })
    }
  } finally {
    gp.busy = false
  }
}

async function disconnectGoogle() {
  await api.gphotosDisconnect()
  gp.connected = false
  gp.list = []
  gp.idx = 0
  gp.cfgOpen = false
  showToast({ icon: '🔓', text: 'Conta Google desvinculada', ms: 2200 })
}

function stepPhoto(delta: number) {
  if (!gp.list.length) return
  gp.idx = (gp.idx + delta + gp.list.length) % gp.list.length
}

let slideTimer: number | undefined
onMounted(() => {
  loadGPhotos()
  slideTimer = window.setInterval(() => {
    if (!gp.hover && !gp.cfgOpen && gp.list.length > 1) stepPhoto(1)
  }, 8000)
})
onUnmounted(() => clearInterval(slideTimer))
</script>

<template>
  <div class="home-wrap">
    <button class="edit-toggle" :class="{ active: editMode }" :title="editMode ? 'Concluir edição da Home' : 'Personalizar cards da Home'"
            @click="editMode = !editMode">
      {{ editMode ? '✓ Concluir' : '✎' }}
    </button>

    <div class="grid" :style="gridStyle">
      <template v-for="(cid, idx) in S.homeCards" :key="cid">
        <!-- ================= CRONÔMETRO ================= -->
        <div v-if="cid === 'stopwatch'" class="card sw" :class="{ running: stopwatch.running, editing: editMode }"
             title="Clique: iniciar/pausar · Duplo clique: zerar" @click="swClick" @dblclick="swReset">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <div class="row">
            <span class="label mono">CRONÔMETRO</span>
            <button class="chip" title="Definir meta com alerta sonoro" @click.stop="cycleGoal">
              {{ stopwatch.goalSec ? `🎯 ${stopwatch.goalSec / 60}m` : '🎯 meta' }}
            </button>
          </div>
          <div class="big mono">{{ fmt(swElapsed()) }}</div>
          <span class="sub">{{ stopwatch.running ? 'Rodando' : stopwatch.acc ? 'Pausado' : 'Parado' }}</span>
        </div>

        <!-- ================= MÍDIA ================= -->
        <div v-else-if="cid === 'media'" class="card media" :class="{ 'has-cover': !!live.media.cover, editing: editMode }">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <template v-if="live.media.cover">
            <img class="cover-bg" :src="live.media.cover" alt="">
            <img class="cover" :src="live.media.cover" alt="">
          </template>
          <template v-else><div class="sun" /><div class="hills" /></template>
          <div class="info">
            <div class="title">{{ has ? live.media.title : 'Nada tocando' }}</div>
            <div class="artist">{{ has ? (live.media.artist || live.media.app) : 'Abra o Spotify, YouTube…' }}</div>
          </div>
          <div class="ctrls">
            <button class="ctl" title="Anterior" @click="api.mediaCmd('prev')">⏮</button>
            <button class="ctl play" title="Tocar/pausar" @click="api.mediaCmd('toggle')">{{ playing ? '❚❚' : '▶' }}</button>
            <button class="ctl" title="Próxima" @click="api.mediaCmd('next')">⏭</button>
          </div>
        </div>

        <!-- ================= FOCO ================= -->
        <div v-else-if="cid === 'focus'" class="card focus" :class="{ break: focus.mode === 'break', editing: editMode }"
             title="Duplo clique: reiniciar" @dblclick="focusReset">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <div class="row">
            <span class="label mono">{{ focus.mode === 'focus' ? 'FOCO' : 'PAUSA' }}</span>
            <div v-if="!focus.running && focus.mode === 'focus'" class="fadj" @click.stop>
              <button title="Diminuir 5 min" @click="adjustFocus(-5)">−</button>
              <button title="Aumentar 5 min" @click="adjustFocus(5)">+</button>
            </div>
          </div>
          <div class="big mono">{{ fmt(focusLeft()) }}</div>
          <div v-if="!focus.running && focus.mode === 'focus'" class="fpresets" @click.stop>
            <button v-for="m in [15, 25, 45, 60]" :key="m" :class="{ on: S.focusMinutes === m }" @click="setFocusMinutes(m)">{{ m }}m</button>
          </div>
          <div class="row">
            <span class="sub">{{ focus.running ? (focus.mode === 'focus' ? 'Focando' : 'Descansando') : 'Pronto' }}</span>
            <button class="round" title="Iniciar/pausar" @click.stop="focusToggle">{{ focus.running ? '❚❚' : '▶' }}</button>
          </div>
        </div>

        <!-- ================= GOOGLE FOTOS ================= -->
        <div v-else-if="cid === 'photos'" class="card photos" :class="{ editing: editMode }"
             @mouseenter="gp.hover = true" @mouseleave="gp.hover = false">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <template v-if="gp.list.length && !gp.cfgOpen">
            <img class="pframe" :src="gp.list[gp.idx]" alt="Google Fotos">
            <div class="pgrad" />
            <span class="pcount mono">{{ gp.idx + 1 }}/{{ gp.list.length }}</span>
            <div class="pctrls">
              <button class="pbtn" title="Anterior" @click.stop="stepPhoto(-1)">◀</button>
              <button class="pbtn" title="Próxima" @click.stop="stepPhoto(1)">▶</button>
              <button class="pbtn" :disabled="gp.busy" title="Escolher fotos no Google Fotos" @click.stop="pickPhotos">🖼️</button>
              <button class="pbtn" title="Configurar conta Google" @click.stop="gp.cfgOpen = true">⚙</button>
            </div>
          </template>
          <template v-else>
            <div class="phead">
              <span class="label mono">GOOGLE FOTOS</span>
              <button v-if="gp.cfgOpen && gp.list.length" class="pbtn" title="Voltar" @click="gp.cfgOpen = false">✕</button>
            </div>
        <template v-if="!gp.connected || gp.editClient">
          <div class="psetup">
            <template v-if="!gp.hasClient || gp.editClient">
              <input v-model="gp.clientId" class="pin" placeholder="Client ID OAuth (opcional se usar pasta)">
              <input v-model="gp.clientSecret" type="password" class="pin" placeholder="Client Secret">
            </template>
            <span v-else class="psub">Conecte sua conta Google ou escolha uma pasta.</span>
            <button class="paction" :disabled="gp.busy || ((!gp.hasClient || gp.editClient) && !gp.clientId.trim())" @click="connectGoogle">
              {{ gp.busy ? 'Aguardando…' : 'Conectar Google' }}
            </button>
            <div class="prow">
              <a class="plink" @click="pickFolder">📁 Pasta do PC</a>
              <a v-if="gp.hasClient && !gp.editClient" class="plink" @click="gp.editClient = true">Client ID</a>
              <a v-else class="plink" @click="api.runAction('https://console.cloud.google.com/apis/credentials')">Cloud Console ↗</a>
            </div>
          </div>
        </template>
        <template v-else>
          <div class="psetup">
            <span class="psub">{{ gp.list.length ? `${gp.list.length} fotos salvas` : 'Conta vinculada! Escolha suas fotos:' }}</span>
            <button class="paction" :disabled="gp.busy" @click="pickPhotos">{{ gp.busy ? 'Abrindo…' : 'Escolher fotos' }}</button>
            <div class="prow">
              <a class="plink" @click="pickFolder">📁 Pasta</a>
              <a class="plink" @click="gp.editClient = true">Client ID</a>
              <a class="plink danger" @click="disconnectGoogle">Desvincular</a>
            </div>
          </div>
        </template>
          </template>
        </div>

        <!-- ================= TIMER RÁPIDO ================= -->
        <div v-else-if="cid === 'timer'" class="card htimer" :class="{ editing: editMode }" @dblclick="timerReset">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <span class="label mono">TIMER</span>
          <div class="big mono">{{ fmt(timerLeft()) }}</div>
          <div v-if="!timer.running" class="fpresets" @click.stop>
            <button v-for="m in [1, 5, 10, 15]" :key="m" :class="{ on: timer.total === m * 60 }" @click="timerPreset(m)">{{ m }}m</button>
          </div>
          <div class="row">
            <span class="sub">{{ timer.running ? 'Contando' : 'Parado' }}</span>
            <button class="round" title="Iniciar/pausar" @click.stop="timerToggle">{{ timer.running ? '❚❚' : '▶' }}</button>
          </div>
        </div>

        <!-- ================= CLIMA ================= -->
        <div v-else-if="cid === 'weather'" class="card hweather" :class="{ editing: editMode }" @click="!editMode && (island.tab = 'weather')">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <span class="label mono">CLIMA</span>
          <div class="big">{{ live.tempText || '—' }}</div>
          <span class="sub">{{ S.city?.name || 'Localização atual' }}</span>
        </div>

        <!-- ================= PRÓXIMA REUNIÃO ================= -->
        <div v-else-if="cid === 'meeting'" class="card hmeet" :class="{ editing: editMode }">
          <div v-if="editMode" class="econtrols" @click.stop>
            <button :disabled="idx === 0" title="Mover para esquerda" @click="moveCard(idx, -1)">◀</button>
            <button :disabled="idx === S.homeCards.length - 1" title="Mover para direita" @click="moveCard(idx, 1)">▶</button>
            <button class="del" title="Remover card" @click="removeCard(cid)">✕</button>
          </div>
          <span class="label mono">REUNIÃO</span>
          <template v-if="nextMeetToday">
            <div class="mhour mono">{{ nextMeetToday.startTime }}</div>
            <div class="msub">{{ nextMeetToday.summary }}</div>
            <button v-if="nextMeetToday.joinUrl" class="paction" style="margin-top:auto" @click="api.runAction(nextMeetToday.joinUrl)">Entrar ↗</button>
          </template>
          <template v-else>
            <div class="msub" style="margin-top:auto">Sem reuniões hoje</div>
            <button class="chip" style="align-self:flex-start;margin-top:6px" @click="island.tab = 'cal'">Abrir Agenda</button>
          </template>
        </div>
      </template>
    </div>

    <!-- Barra de adição no modo edição -->
    <div v-if="editMode" class="addbar">
      <span class="mute tiny">Adicionar:</span>
      <button v-for="id in availableToAdd" :key="id" class="addchip" @click="addCard(id)">
        + {{ HOME_CARDS[id].icon }} {{ HOME_CARDS[id].label }}
      </button>
      <span v-if="!availableToAdd.length" class="mute tiny">Todos os cards já estão na Home</span>
      <button class="addchip reset" @click="resetCards">Restaurar padrão</button>
    </div>
  </div>
</template>

<style scoped>
.home-wrap { position: relative; display: flex; flex-direction: column; gap: 6px; height: 100%; }
.edit-toggle {
  position: absolute; top: -6px; right: 2px; z-index: 15; padding: 2px 7px; border-radius: 9px;
  background: rgba(255,255,255,.08); color: var(--mute); font-size: 10.5px; font-weight: 600;
}
.edit-toggle:hover, .edit-toggle.active { background: var(--accent); color: #111; }

.grid { flex: 1; min-height: 0; display: grid; gap: 8px; }
.card { position: relative; border-radius: 20px; padding: 14px 16px; overflow: hidden; display: flex; flex-direction: column; min-width: 0; }
.card.editing { box-shadow: inset 0 0 0 1px rgba(255,255,255,.28); }
.econtrols {
  position: relative; z-index: 12; display: flex; gap: 4px; justify-content: flex-end; margin-bottom: 4px;
}
.econtrols button {
  width: 20px; height: 20px; border-radius: 6px; background: rgba(0,0,0,.55); color: #fff; font-size: 10px;
  display: grid; place-items: center;
}
.econtrols button:disabled { opacity: .3; }
.econtrols button.del { color: #ff8a7e; }

.addbar {
  display: flex; align-items: center; gap: 5px; flex-wrap: nowrap; overflow-x: auto;
  padding: 4px 8px; border-radius: 12px; background: rgba(255,255,255,.05);
}
.addchip { padding: 3px 8px; border-radius: 8px; background: rgba(255,255,255,.09); font-size: 11px; white-space: nowrap; }
.addchip:hover { background: rgba(255,255,255,.18); }
.addchip.reset { margin-left: auto; color: var(--mute); }
.tiny { font-size: 10.5px; white-space: nowrap; }

.label { font-size: 10.5px; opacity: .85; }
.big { font-size: 32px; font-weight: 600; margin-top: auto; letter-spacing: 0; }
.sub { font-size: 12px; opacity: .85; }
.row { display: flex; align-items: center; justify-content: space-between; gap: 4px; }
.chip { padding: 1px 6px; border-radius: 7px; background: rgba(0,0,0,.25); color: #fff; font-size: 10px; }
.chip:hover { background: rgba(0,0,0,.45); }

.sw { background: linear-gradient(160deg, color-mix(in srgb, var(--accent), black 62%), var(--accent)); cursor: pointer; }
.sw.running { box-shadow: inset 0 0 0 1px rgba(255,255,255,.3); }
.focus { background: linear-gradient(160deg, #24205a, #4a3fc0 80%); }
.focus.break { background: linear-gradient(160deg, #0f3d2e, #1f9d6b 80%); }
.fadj { display: flex; gap: 3px; }
.fadj button { width: 20px; height: 20px; border-radius: 6px; background: rgba(255,255,255,.15); font-size: 13px; line-height: 1; }
.fadj button:hover { background: rgba(255,255,255,.28); }
.fpresets { display: flex; gap: 3px; margin: 2px 0 4px; }
.fpresets button { padding: 1px 5px; border-radius: 6px; background: rgba(255,255,255,.1); font-size: 10px; color: #ddd; }
.fpresets button.on { background: #fff; color: #111; font-weight: 700; }

.round { width: 28px; height: 28px; border-radius: 50%; background: #fff; color: #111; font-size: 11px; display: grid; place-items: center; }
.round:hover { transform: scale(1.06); }
.media { background: linear-gradient(180deg, hsl(var(--h) 55% 20%), hsl(var(--h) 70% 38%)); justify-content: flex-end; }
.sun { position: absolute; right: 26%; top: -30px; width: 110px; height: 110px; border-radius: 50%;
  background: hsl(calc(var(--h) + 10) 90% 82%); opacity: .85; }
.hills { position: absolute; left: 0; right: 0; bottom: 0; height: 55%; background: linear-gradient(180deg, transparent, rgba(0,0,0,.35));
  clip-path: polygon(0 60%, 18% 30%, 35% 55%, 55% 20%, 75% 50%, 100% 25%, 100% 100%, 0 100%); }
.cover-bg { position: absolute; inset: -30px; width: calc(100% + 60px); height: calc(100% + 60px);
  object-fit: cover; filter: blur(26px) saturate(1.4) brightness(.55); }
.cover { position: absolute; top: 14px; left: 16px; width: 70px; height: 70px; border-radius: 12px; object-fit: cover; box-shadow: 0 8px 24px rgba(0,0,0,.5); }
.info { position: relative; max-width: 66%; }
.title { font-size: 24px; font-weight: 800; line-height: 1.1; letter-spacing: -.02em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.has-cover .title { font-size: 19px; }
.artist { font-size: 12.5px; opacity: .85; margin-top: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ctrls { position: absolute; right: 12px; bottom: 12px; display: flex; align-items: center; gap: 3px; }
.ctl { width: 28px; height: 28px; border-radius: 50%; font-size: 11px; color: #fff; opacity: .85; }
.ctl:hover { background: rgba(255,255,255,.15); opacity: 1; }
.ctl.play { width: 36px; height: 36px; background: #fff; color: #111; font-size: 13px; opacity: 1; }

.photos { background: linear-gradient(160deg, #1b1c24, #282b3a); padding: 12px 13px; }
.pframe { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
.pgrad { position: absolute; inset: 0; background: linear-gradient(180deg, rgba(0,0,0,.35), transparent 35%, rgba(0,0,0,.65)); pointer-events: none; }
.pcount { position: relative; align-self: flex-start; font-size: 10px; padding: 2px 6px; border-radius: 8px; background: rgba(0,0,0,.45); }
.pctrls { position: relative; margin-top: auto; display: flex; justify-content: space-between; gap: 4px; opacity: 0; transition: opacity .2s; }
.photos:hover .pctrls { opacity: 1; }
.pbtn { width: 24px; height: 24px; border-radius: 8px; background: rgba(0,0,0,.55); color: #fff; font-size: 11px; display: grid; place-items: center; }
.pbtn:hover { background: rgba(0,0,0,.85); }
.phead { display: flex; align-items: center; justify-content: space-between; }
.psetup { margin-top: auto; display: flex; flex-direction: column; gap: 5px; }
.psub { font-size: 11px; color: var(--mute); line-height: 1.3; }
.pin { width: 100%; padding: 4px 7px; border-radius: 8px; border: 1px solid var(--line); background: rgba(0,0,0,.3); color: #fff; font-size: 11px; outline: none; }
.pin:focus { border-color: var(--accent); }
.paction { padding: 6px 10px; border-radius: 10px; background: var(--accent); color: #111; font-size: 11.5px; font-weight: 700; }
.paction:disabled { opacity: .5; }
.prow { display: flex; justify-content: space-between; }
.plink { font-size: 10.5px; color: var(--accent-2); cursor: pointer; text-decoration: underline; }
.plink.danger { color: #ff8a7e; }

.htimer { background: linear-gradient(160deg, #1b2838, #274469); }
.hweather { background: linear-gradient(160deg, #152b3c, #1d648b); cursor: pointer; }
.hmeet { background: linear-gradient(160deg, #1e2242, #313b7a); }
.mhour { font-size: 22px; font-weight: 700; margin-top: 8px; color: #9fb3ff; }
.msub { font-size: 12px; opacity: .9; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }
</style>
