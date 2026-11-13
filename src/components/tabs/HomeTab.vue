<script setup lang="ts">
// Início: cards personalizáveis (mover, remover, adicionar), cronômetro com meta/som, mídia e foco ajustável
import { computed, ref } from 'vue'
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
  gridTemplateColumns: S.homeCards.map(id => (id === 'media' ? '1.85fr' : '1fr')).join(' ')
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
            <button class="pill-chip" title="Definir meta com alerta sonoro" @click.stop="cycleGoal">
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
            <button class="ctl" title="Anterior" @click="api.mediaCmd('prev')">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor">
                <rect x="4.5" y="5.5" width="2.4" height="13" rx="1.2" />
                <path d="M18.2 6.4a1.1 1.1 0 0 0-1.7-.9l-8.2 5.6a1.1 1.1 0 0 0 0 1.8l8.2 5.6a1.1 1.1 0 0 0 1.7-.9V6.4z" />
              </svg>
            </button>
            <button class="ctl play" title="Tocar/pausar" @click="api.mediaCmd('toggle')">
              <svg v-if="playing" viewBox="0 0 24 24" width="15" height="15" fill="currentColor">
                <rect x="6.5" y="5.5" width="3.8" height="13" rx="1.4" />
                <rect x="13.7" y="5.5" width="3.8" height="13" rx="1.4" />
              </svg>
              <svg v-else viewBox="0 0 24 24" width="16" height="16" fill="currentColor" style="margin-left:1.5px">
                <path d="M7.5 5.6a1.2 1.2 0 0 1 1.8-1l10.2 6.4a1.2 1.2 0 0 1 0 2l-10.2 6.4a1.2 1.2 0 0 1-1.8-1V5.6z" />
              </svg>
            </button>
            <button class="ctl" title="Próxima" @click="api.mediaCmd('next')">
              <svg viewBox="0 0 24 24" width="15" height="15" fill="currentColor">
                <path d="M5.8 6.4a1.1 1.1 0 0 1 1.7-.9l8.2 5.6a1.1 1.1 0 0 1 0 1.8l-8.2 5.6a1.1 1.1 0 0 1-1.7-.9V6.4z" />
                <rect x="17.1" y="5.5" width="2.4" height="13" rx="1.2" />
              </svg>
            </button>
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
            <div v-if="!focus.running && focus.mode === 'focus'" class="stepper" @click.stop>
              <button class="step-btn" title="Diminuir 5 min" @click="adjustFocus(-5)">
                <svg viewBox="0 0 16 16" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <path d="M3.5 8h9" />
                </svg>
              </button>
              <span class="step-val mono">{{ S.focusMinutes }}m</span>
              <button class="step-btn" title="Aumentar 5 min" @click="adjustFocus(5)">
                <svg viewBox="0 0 16 16" width="11" height="11" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <path d="M8 3.5v9M3.5 8h9" />
                </svg>
              </button>
            </div>
          </div>
          <div class="big mono">{{ fmt(focusLeft()) }}</div>
          <div v-if="!focus.running && focus.mode === 'focus'" class="segbar" @click.stop>
            <button v-for="m in [15, 25, 45, 60]" :key="m" :class="{ on: S.focusMinutes === m }" @click="setFocusMinutes(m)">{{ m }}m</button>
          </div>
          <div class="row">
            <span class="sub">{{ focus.running ? (focus.mode === 'focus' ? 'Focando' : 'Descansando') : 'Pronto' }}</span>
            <button class="round" title="Iniciar/pausar" @click.stop="focusToggle">
              <svg v-if="focus.running" viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
                <rect x="6.5" y="5.5" width="3.8" height="13" rx="1.4" />
                <rect x="13.7" y="5.5" width="3.8" height="13" rx="1.4" />
              </svg>
              <svg v-else viewBox="0 0 24 24" width="12" height="12" fill="currentColor" style="margin-left:1.5px">
                <path d="M7.5 5.6a1.2 1.2 0 0 1 1.8-1l10.2 6.4a1.2 1.2 0 0 1 0 2l-10.2 6.4a1.2 1.2 0 0 1-1.8-1V5.6z" />
              </svg>
            </button>
          </div>
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
          <div v-if="!timer.running" class="segbar" @click.stop>
            <button v-for="m in [1, 5, 10, 15]" :key="m" :class="{ on: timer.total === m * 60 }" @click="timerPreset(m)">{{ m }}m</button>
          </div>
          <div class="row">
            <span class="sub">{{ timer.running ? 'Contando' : 'Parado' }}</span>
            <button class="round" title="Iniciar/pausar" @click.stop="timerToggle">
              <svg v-if="timer.running" viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
                <rect x="6.5" y="5.5" width="3.8" height="13" rx="1.4" />
                <rect x="13.7" y="5.5" width="3.8" height="13" rx="1.4" />
              </svg>
              <svg v-else viewBox="0 0 24 24" width="12" height="12" fill="currentColor" style="margin-left:1.5px">
                <path d="M7.5 5.6a1.2 1.2 0 0 1 1.8-1l10.2 6.4a1.2 1.2 0 0 1 0 2l-10.2 6.4a1.2 1.2 0 0 1-1.8-1V5.6z" />
              </svg>
            </button>
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
            <button v-if="nextMeetToday.joinUrl" class="meet-btn" @click="api.runAction(nextMeetToday.joinUrl)">Entrar ↗</button>
          </template>
          <template v-else>
            <div class="msub" style="margin-top:auto">Sem reuniões hoje</div>
            <button class="pill-chip" style="align-self:flex-start;margin-top:6px" @click="island.tab = 'cal'">Abrir Agenda</button>
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
  position: absolute; top: -6px; right: 2px; z-index: 15; padding: 2px 8px; border-radius: 999px;
  background: rgba(255,255,255,.08); border: 1px solid rgba(255,255,255,.1); color: var(--mute); font-size: 10.5px; font-weight: 600;
}
.edit-toggle:hover, .edit-toggle.active { background: var(--accent); border-color: transparent; color: #111; }

.grid { flex: 1; min-height: 0; display: grid; gap: 9px; }
.card {
  position: relative; border-radius: 22px; padding: 14px 16px; overflow: hidden;
  display: flex; flex-direction: column; min-width: 0;
  border: 1px solid rgba(255,255,255,.11);
  box-shadow: inset 0 1px 0 rgba(255,255,255,.16), 0 8px 20px rgba(0,0,0,.22);
}
.card.editing { box-shadow: inset 0 0 0 1.5px rgba(255,255,255,.35); }
.econtrols {
  position: relative; z-index: 12; display: flex; gap: 4px; justify-content: flex-end; margin-bottom: 4px;
}
.econtrols button {
  width: 20px; height: 20px; border-radius: 7px; background: rgba(0,0,0,.45); border: 1px solid rgba(255,255,255,.12);
  color: #fff; font-size: 10px; display: grid; place-items: center;
}
.econtrols button:disabled { opacity: .3; }
.econtrols button.del { color: #ff8a7e; }

.addbar {
  display: flex; align-items: center; gap: 5px; flex-wrap: nowrap; overflow-x: auto;
  padding: 5px 9px; border-radius: 14px; background: rgba(255,255,255,.06); border: 1px solid rgba(255,255,255,.08);
}
.addchip { padding: 3px 9px; border-radius: 999px; background: rgba(255,255,255,.1); font-size: 11px; white-space: nowrap; }
.addchip:hover { background: rgba(255,255,255,.2); }
.addchip.reset { margin-left: auto; color: var(--mute); }
.tiny { font-size: 10.5px; white-space: nowrap; }

.label { font-size: 10.5px; opacity: .82; letter-spacing: .04em; }
.big { font-size: 32px; font-weight: 600; margin-top: auto; letter-spacing: -.01em; line-height: 1.05; }
.sub { font-size: 12px; opacity: .82; }
.row { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.pill-chip {
  padding: 2px 8px; border-radius: 999px; background: rgba(0,0,0,.24);
  border: 1px solid rgba(255,255,255,.12); color: #fff; font-size: 10px;
}
.pill-chip:hover { background: rgba(0,0,0,.42); }

.sw { background: linear-gradient(155deg, color-mix(in srgb, var(--accent), black 58%), var(--accent)); cursor: pointer; }
.sw.running { box-shadow: inset 0 0 0 1px rgba(255,255,255,.32), 0 8px 20px rgba(0,0,0,.22); }
.focus { background: linear-gradient(155deg, #211d54, #493ec2 85%); }
.focus.break { background: linear-gradient(155deg, #0d3a2c, #1d9a68 85%); }

/* Stepper (− | 25m | +) integrado ao card como cápsula única */
.stepper {
  display: inline-flex; align-items: center;
  background: rgba(0,0,0,.24); border: 1px solid rgba(255,255,255,.14);
  border-radius: 999px; overflow: hidden; height: 22px;
}
.step-btn {
  width: 22px; height: 22px; display: grid; place-items: center; color: rgba(255,255,255,.9);
  transition: background .15s;
}
.step-btn:hover { background: rgba(255,255,255,.16); color: #fff; }
.step-val {
  padding: 0 5px; font-size: 10px; color: rgba(255,255,255,.88);
  border-left: 1px solid rgba(255,255,255,.1); border-right: 1px solid rgba(255,255,255,.1);
  line-height: 22px;
}

/* Barra segmentada unificada para presets de tempo */
.segbar {
  display: grid; grid-auto-flow: column; grid-auto-columns: 1fr; gap: 2px;
  padding: 2px; margin: 4px 0 5px; border-radius: 10px;
  background: rgba(0,0,0,.22); border: 1px solid rgba(255,255,255,.09);
}
.segbar button {
  padding: 2px 0; border-radius: 8px; font-size: 10px; color: rgba(255,255,255,.75); font-weight: 500;
  transition: background .15s, color .15s;
}
.segbar button:hover { color: #fff; background: rgba(255,255,255,.08); }
.segbar button.on { background: rgba(255,255,255,.94); color: #111; font-weight: 700; box-shadow: 0 1px 4px rgba(0,0,0,.25); }

.round {
  width: 28px; height: 28px; border-radius: 50%; background: rgba(255,255,255,.95); color: #111;
  display: grid; place-items: center; box-shadow: 0 3px 10px rgba(0,0,0,.25);
  transition: transform .15s;
}
.round:hover { transform: scale(1.07); }

/* Card de mídia */
.media { background: linear-gradient(180deg, hsl(var(--h) 55% 20%), hsl(var(--h) 70% 38%)); justify-content: flex-end; }
.sun { position: absolute; right: 26%; top: -30px; width: 110px; height: 110px; border-radius: 50%;
  background: hsl(calc(var(--h) + 10) 90% 82%); opacity: .85; }
.hills { position: absolute; left: 0; right: 0; bottom: 0; height: 55%; background: linear-gradient(180deg, transparent, rgba(0,0,0,.35));
  clip-path: polygon(0 60%, 18% 30%, 35% 55%, 55% 20%, 75% 50%, 100% 25%, 100% 100%, 0 100%); }
.cover-bg { position: absolute; inset: -30px; width: calc(100% + 60px); height: calc(100% + 60px);
  object-fit: cover; filter: blur(26px) saturate(1.4) brightness(.55); }
.cover { position: absolute; top: 14px; left: 16px; width: 70px; height: 70px; border-radius: 14px; object-fit: cover; box-shadow: 0 8px 24px rgba(0,0,0,.5); }
.info { position: relative; max-width: 62%; }
.title { font-size: 23px; font-weight: 800; line-height: 1.1; letter-spacing: -.02em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.has-cover .title { font-size: 19px; }
.artist { font-size: 12.5px; opacity: .85; margin-top: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }

/* Controles de mídia em cápsula de vidro */
.ctrls {
  position: absolute; right: 12px; bottom: 12px;
  display: flex; align-items: center; gap: 3px; padding: 3px;
  border-radius: 999px; background: rgba(0,0,0,.28); border: 1px solid rgba(255,255,255,.12);
  backdrop-filter: blur(10px);
}
.ctl {
  width: 28px; height: 28px; border-radius: 50%; color: rgba(255,255,255,.88);
  display: grid; place-items: center; transition: background .15s, color .15s, transform .15s;
}
.ctl:hover { background: rgba(255,255,255,.16); color: #fff; }
.ctl.play {
  width: 34px; height: 34px; background: #fff; color: #111;
  box-shadow: 0 3px 10px rgba(0,0,0,.28);
}
.ctl.play:hover { background: #fff; color: #111; transform: scale(1.06); }

.htimer { background: linear-gradient(155deg, #192636, #274469); }
.hweather { background: linear-gradient(155deg, #142a3b, #1d648b); cursor: pointer; }
.hmeet { background: linear-gradient(155deg, #1d2140, #313b7a); }
.mhour { font-size: 22px; font-weight: 700; margin-top: 8px; color: #9fb3ff; }
.msub { font-size: 12px; opacity: .9; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; }
.meet-btn { margin-top: auto; padding: 6px 10px; border-radius: 10px; background: var(--accent); color: #111; font-size: 11.5px; font-weight: 700; }
</style>
