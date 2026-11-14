<script setup lang="ts">
// Mascote autônomo no topo da ilha: reaproveita a lógica independente de estados
// (anda, senta e pisca, pula, brinca e espia) com transform direto na GPU (sem re-render por frame)
// e pausa automaticamente quando a ilha está recolhida.
import { onMounted, onUnmounted, ref, watch } from 'vue'
import PetSprite, { type PetMode } from './PetSprite.vue'
import { S } from '../store/settings'

const props = withDefaults(defineProps<{ busy?: boolean; active?: boolean }>(), {
  busy: false,
  active: true
})

const stageEl = ref<HTMLElement>()
const petEl = ref<HTMLElement>()
const shadowEl = ref<HTMLElement>()

const dir = ref<1 | -1>(1)
const mode = ref<PetMode>('walk')
const blinking = ref(false)

let animId = 0
let lastTs = 0
let posPct = 50
let jumpY = 0
let stateTimer = 0
let stateDuration = 4.2
let walkSpeed = 11
let blinkTimer: number | undefined
let stageWidth = 340

function syncDom() {
  const x = (posPct / 100) * stageWidth
  if (petEl.value) {
    const peekOff = mode.value === 'peek' ? 2 : 0
    petEl.value.style.transform = `translate3d(${x.toFixed(2)}px, ${(jumpY + peekOff).toFixed(2)}px, 0) translateX(-50%)`
  }
  if (shadowEl.value) {
    const scale = Math.max(0.55, 1 + jumpY / 14)
    shadowEl.value.style.transform = `translate3d(${x.toFixed(2)}px, 0, 0) translateX(-50%) scaleX(${scale.toFixed(2)})`
    shadowEl.value.style.opacity = mode.value === 'peek' ? '0' : String((0.32 * scale).toFixed(2))
  }
}

function pickNextState() {
  const prev = mode.value
  const r = Math.random()

  if (prev === 'jump') {
    if (r < 0.55) {
      mode.value = 'walk'
      stateDuration = 2.5 + Math.random() * 4
      walkSpeed = 9 + Math.random() * 7
      if (Math.random() < 0.35) dir.value = dir.value === 1 ? -1 : 1
    } else if (r < 0.85) {
      mode.value = 'play'
      stateDuration = 1.6 + Math.random() * 1.4
    } else {
      mode.value = 'sit'
      stateDuration = 2.2 + Math.random() * 2.2
    }
  } else if (prev === 'walk') {
    if (r < 0.28) {
      mode.value = 'jump'
      stateDuration = 0.62
    } else if (r < 0.52) {
      mode.value = 'sit'
      stateDuration = 2.0 + Math.random() * 2.6
    } else if (r < 0.74) {
      mode.value = 'play'
      stateDuration = 1.6 + Math.random() * 1.5
    } else if (r < 0.88) {
      mode.value = 'peek'
      stateDuration = 2.0 + Math.random() * 1.8
    } else {
      mode.value = 'walk'
      dir.value = dir.value === 1 ? -1 : 1
      walkSpeed = 8 + Math.random() * 9
      stateDuration = 2.5 + Math.random() * 3.5
    }
  } else {
    if (r < 0.25) {
      mode.value = 'jump'
      stateDuration = 0.62
    } else if (r < 0.82) {
      mode.value = 'walk'
      if (Math.random() < 0.45) dir.value = dir.value === 1 ? -1 : 1
      walkSpeed = 9 + Math.random() * 8
      stateDuration = 2.8 + Math.random() * 4.2
    } else {
      mode.value = prev === 'play' ? 'sit' : 'play'
      stateDuration = 1.8 + Math.random() * 1.6
    }
  }
  stateTimer = 0
}

function tick(ts: number) {
  if (!props.active || document.hidden) {
    animId = 0
    lastTs = 0
    return
  }
  if (!lastTs) {
    lastTs = ts
    stageWidth = stageEl.value?.clientWidth || 340
  }
  const dt = Math.min(0.05, (ts - lastTs) / 1000)
  lastTs = ts
  stateTimer += dt

  if (mode.value === 'walk') {
    posPct += dir.value * dt * walkSpeed
    if (posPct >= 88) { posPct = 88; dir.value = -1 }
    if (posPct <= 12) { posPct = 12; dir.value = 1 }
    jumpY = 0
  } else if (mode.value === 'jump') {
    const p = Math.min(1, stateTimer / stateDuration)
    jumpY = -Math.sin(p * Math.PI) * 9
    posPct += dir.value * dt * 15
    if (posPct >= 88) { posPct = 88; dir.value = -1 }
    if (posPct <= 12) { posPct = 12; dir.value = 1 }
  } else {
    jumpY = 0
  }

  if (stateTimer >= stateDuration) {
    jumpY = 0
    pickNextState()
  }

  syncDom()
  animId = requestAnimationFrame(tick)
}

function scheduleBlink() {
  clearTimeout(blinkTimer)
  if (!props.active) return
  const delay = 2200 + Math.random() * 2800
  blinkTimer = window.setTimeout(() => {
    blinking.value = true
    window.setTimeout(() => {
      blinking.value = false
      scheduleBlink()
    }, 150)
  }, delay)
}

function startLoop() {
  if (!props.active || document.hidden) return
  stageWidth = stageEl.value?.clientWidth || 340
  syncDom()
  if (!animId) {
    lastTs = 0
    animId = requestAnimationFrame(tick)
  }
  scheduleBlink()
}

function stopLoop() {
  if (animId) {
    cancelAnimationFrame(animId)
    animId = 0
  }
  lastTs = 0
  clearTimeout(blinkTimer)
  blinking.value = false
}

watch(() => props.active, on => (on ? startLoop() : stopLoop()))

function onVis() {
  if (document.hidden) stopLoop()
  else if (props.active) startLoop()
}

onMounted(() => {
  document.addEventListener('visibilitychange', onVis)
  if (props.active) startLoop()
})

onUnmounted(() => {
  document.removeEventListener('visibilitychange', onVis)
  stopLoop()
})
</script>

<template>
  <div ref="stageEl" class="pet-stage" aria-hidden="true">
    <div ref="shadowEl" class="pet-shadow" />
    <div ref="petEl" class="pet-pos">
      <PetSprite :pet="S.pet" :mode="mode" :blink="blinking" :busy="busy" :dir="dir" :hide-shadow="true" />
    </div>
  </div>
</template>

<style scoped>
.pet-stage {
  position: absolute; left: 190px; right: 220px; top: 1px; height: 35px;
  pointer-events: none; overflow: hidden;
}
.pet-pos {
  position: absolute; top: 0; left: 0;
  will-change: transform;
}
.pet-shadow {
  position: absolute; bottom: 1px; left: 0;
  width: 24px; height: 3px; border-radius: 50%;
  background: #000; opacity: .3;
  will-change: transform, opacity;
}
</style>
