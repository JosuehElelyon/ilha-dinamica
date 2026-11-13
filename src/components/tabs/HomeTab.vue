<script setup lang="ts">
// Início: cronômetro, mídia tocando (com capa) e foco/pomodoro
import { computed } from 'vue'
import { api } from '../../lib/api'
import { fmt } from '../../lib/format'
import { live, stopwatch, swElapsed, focus, focusLeft, FOCO } from '../../store/state'

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
  </div>
</template>

<style scoped>
.grid { display: grid; grid-template-columns: 1fr 2.1fr 1fr; gap: 8px; height: 100%; }
.card { position: relative; border-radius: 20px; padding: 14px 16px; overflow: hidden; display: flex; flex-direction: column; }
.label { font-size: 10.5px; opacity: .85; }
.big { font-size: 34px; font-weight: 600; margin-top: auto; letter-spacing: 0; }
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
.cover { position: absolute; top: 14px; left: 16px; width: 74px; height: 74px; border-radius: 12px; object-fit: cover; box-shadow: 0 8px 24px rgba(0,0,0,.5); }
.info { position: relative; max-width: 68%; }
.title { font-size: 26px; font-weight: 800; line-height: 1.1; letter-spacing: -.02em; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.has-cover .title { font-size: 21px; }
.artist { font-size: 13px; opacity: .85; margin-top: 3px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ctrls { position: absolute; right: 14px; bottom: 14px; display: flex; align-items: center; gap: 4px; }
.ctl { width: 30px; height: 30px; border-radius: 50%; font-size: 12px; color: #fff; opacity: .85; }
.ctl:hover { background: rgba(255,255,255,.15); opacity: 1; }
.ctl.play { width: 40px; height: 40px; background: #fff; color: #111; font-size: 14px; opacity: 1; }
</style>
