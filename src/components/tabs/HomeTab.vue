<script setup lang="ts">
// Início: cronômetro, mídia tocando (com capa), foco/pomodoro e mini porta-retrato Google Fotos
import { computed, onMounted, onUnmounted, reactive } from 'vue'
import { api } from '../../lib/api'
import { fmt } from '../../lib/format'
import { live, stopwatch, swElapsed, focus, focusLeft, FOCO, showToast } from '../../store/state'

const has = computed(() => !!live.media.title)
const playing = computed(() => live.media.status === 'Playing')

let clickT: number | undefined
function swClick() {
  clearTimeout(clickT)
  clickT = window.setTimeout(() => {
    if (stopwatch.running) { stopwatch.acc += Date.now() - stopwatch.start; stopwatch.running = false }
    else { stopwatch.start = Date.now(); stopwatch.running = true }
  }, 220)
}
function swReset() { clearTimeout(clickT); stopwatch.running = false; stopwatch.acc = 0 }

function focusToggle() {
  if (focus.running) { focus.left = (focus.endAt - Date.now()) / 1000; focus.running = false }
  else { focus.endAt = Date.now() + focus.left * 1000; focus.running = true }
}
function focusReset() { focus.running = false; focus.mode = 'focus'; focus.left = FOCO }

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
  <div class="grid">
    <div class="card sw" :class="{ running: stopwatch.running }" title="Clique: iniciar/pausar · Duplo clique: zerar"
         @click="swClick" @dblclick="swReset">
      <span class="label mono">CRONÔMETRO</span>
      <div class="big mono">{{ fmt(swElapsed()) }}</div>
      <span class="sub">{{ stopwatch.running ? 'Rodando' : stopwatch.acc ? 'Pausado' : 'Parado' }}</span>
    </div>

    <div class="card media" :class="{ 'has-cover': !!live.media.cover }">
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

    <div class="card focus" :class="{ break: focus.mode === 'break' }" title="Duplo clique: reiniciar" @dblclick="focusReset">
      <span class="label mono">{{ focus.mode === 'focus' ? 'FOCO' : 'PAUSA' }}</span>
      <div class="big mono">{{ fmt(focusLeft()) }}</div>
      <div class="row">
        <span class="sub">{{ focus.running ? (focus.mode === 'focus' ? 'Focando' : 'Descansando') : 'Pronto' }}</span>
        <button class="round" title="Iniciar/pausar" @click.stop="focusToggle">{{ focus.running ? '❚❚' : '▶' }}</button>
      </div>
    </div>

    <div class="card photos" @mouseenter="gp.hover = true" @mouseleave="gp.hover = false">
      <!-- Exibição das fotos -->
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

      <!-- Configuração / conexão -->
      <template v-else>
        <div class="phead">
          <span class="label mono">GOOGLE FOTOS</span>
          <button v-if="gp.cfgOpen && gp.list.length" class="pbtn" title="Voltar" @click="gp.cfgOpen = false">✕</button>
        </div>

        <template v-if="!gp.connected || gp.editClient">
          <div class="psetup">
            <template v-if="!gp.hasClient || gp.editClient">
              <input v-model="gp.clientId" class="pin" placeholder="Client ID OAuth (.apps.googleusercontent.com)">
              <input v-model="gp.clientSecret" type="password" class="pin" placeholder="Client Secret (App p/ computador)">
              <a class="plink" @click="api.runAction('https://console.cloud.google.com/apis/credentials')">Google Cloud Console ↗</a>
            </template>
            <span v-else class="psub">Ative a <b>Photos Picker API</b> e conecte sua conta.</span>
            <button class="paction" :disabled="gp.busy || ((!gp.hasClient || gp.editClient) && !gp.clientId.trim())" @click="connectGoogle">
              {{ gp.busy ? 'Aguardando…' : 'Conectar Google' }}
            </button>
            <a v-if="gp.hasClient && !gp.editClient" class="plink" @click="gp.editClient = true">Alterar Client ID</a>
          </div>
        </template>

        <template v-else>
          <div class="psetup">
            <span class="psub">{{ gp.list.length ? `${gp.list.length} fotos salvas` : 'Conta vinculada! Escolha suas fotos:' }}</span>
            <button class="paction" :disabled="gp.busy" @click="pickPhotos">{{ gp.busy ? 'Abrindo…' : 'Escolher fotos' }}</button>
            <div class="prow">
              <a class="plink" @click="gp.editClient = true">Client ID</a>
              <a class="plink danger" @click="disconnectGoogle">Desvincular</a>
            </div>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>

<style scoped>
.grid { display: grid; grid-template-columns: 1fr 1.75fr 1fr 1.15fr; gap: 8px; height: 100%; }
.card { position: relative; border-radius: 20px; padding: 14px 16px; overflow: hidden; display: flex; flex-direction: column; }
.label { font-size: 10.5px; opacity: .85; }
.big { font-size: 32px; font-weight: 600; margin-top: auto; letter-spacing: 0; }
.sub { font-size: 12px; opacity: .85; }
.row { display: flex; align-items: center; justify-content: space-between; }
.sw { background: linear-gradient(160deg, color-mix(in srgb, var(--accent), black 62%), var(--accent)); cursor: pointer; }
.sw.running { box-shadow: inset 0 0 0 1px rgba(255,255,255,.3); }
.focus { background: linear-gradient(160deg, #24205a, #4a3fc0 80%); }
.focus.break { background: linear-gradient(160deg, #0f3d2e, #1f9d6b 80%); }
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
</style>
