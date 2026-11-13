<script setup lang="ts">
// Contagem regressiva
import { fmt } from '../../lib/format'
import { timer, timerLeft } from '../../store/state'

function preset(min: number) { timer.running = false; timer.total = timer.left = min * 60 }
function addMin() { if (timer.running) timer.endAt += 60000; else timer.left += 60 }
function toggle() {
  if (timer.running) { timer.left = timerLeft(); timer.running = false }
  else if (timer.left > 0) { timer.endAt = Date.now() + timer.left * 1000; timer.running = true }
}
function reset() { timer.running = false; timer.left = timer.total }
</script>

<template>
  <div class="wrap">
    <div class="big mono" :class="{ running: timer.running }">{{ fmt(timerLeft()) }}</div>
    <div class="row">
      <button v-for="m in [1, 5, 10, 15, 30]" :key="m" @click="preset(m)">{{ m }} min</button>
      <button @click="addMin">+1 min</button>
    </div>
    <div class="row">
      <button class="primary" @click="toggle">{{ timer.running ? 'Pausar' : 'Iniciar' }}</button>
      <button @click="reset">Zerar</button>
    </div>
  </div>
</template>

<style scoped>
.wrap { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 10px; }
.big { font-size: 52px; font-weight: 600; letter-spacing: 0; }
.big.running { color: var(--accent-2); }
.row { display: flex; gap: 6px; }
.row button { padding: 6px 12px; border-radius: 14px; background: rgba(255,255,255,.07); font-size: 12px; }
.row button:hover { background: rgba(255,255,255,.13); }
.row .primary { background: var(--accent); color: #111; font-weight: 700; }
.row .primary:hover { background: var(--accent-2); }
</style>
