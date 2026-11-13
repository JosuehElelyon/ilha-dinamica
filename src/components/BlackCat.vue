<script setup lang="ts">
// Gatinho preto animado no topo da ilha:
// - Anda de um lado para o outro com patinhas e rabinho animados
// - Para, senta de frente, olha para você e pisca
// - Quando você interage com a ilha abaixo do topo, ele se apoia olhando para baixo e segue o cursor
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { island } from '../store/state'

defineProps<{ busy?: boolean }>()

type Mode = 'walk' | 'sit' | 'peek'

const stage = ref<HTMLElement>()
const pos = ref(50) // % horizontal dentro do palco
const dir = ref<1 | -1>(1)
const baseMode = ref<'walk' | 'sit'>('walk')
const blinking = ref(false)
const interacting = ref(false)
const mouse = ref({ x: 0, y: 0 })

const mode = computed<Mode>(() => (interacting.value && island.expanded ? 'peek' : baseMode.value))

// Vetor normalizado do olhar em direção ao cursor
const look = computed(() => {
  const el = stage.value
  if (!el) return { dx: 0, dy: 0 }
  const r = el.getBoundingClientRect()
  const catX = r.left + (r.width * pos.value) / 100
  const catY = r.top + r.height * 0.55
  const vx = mouse.value.x - catX
  const vy = mouse.value.y - catY
  const dist = Math.hypot(vx, vy) || 1
  const max = 2.1
  return {
    dx: Math.max(-max, Math.min(max, (vx / dist) * max)),
    dy: Math.max(-max, Math.min(max, (vy / dist) * max))
  }
})

function onMouseMove(e: MouseEvent) {
  mouse.value = { x: e.clientX, y: e.clientY }
  const el = stage.value
  if (!el) return
  const r = el.getBoundingClientRect()
  const belowHeader = e.clientY > r.bottom + 4
  interacting.value = island.expanded && belowHeader
  if (interacting.value && r.width > 0) {
    // Segue suavemente a posição horizontal do cursor dentro dos limites do palco
    const targetPct = ((e.clientX - r.left) / r.width) * 100
    const clamped = Math.max(8, Math.min(92, targetPct))
    if (Math.abs(clamped - pos.value) > 1.5) {
      dir.value = clamped >= pos.value ? 1 : -1
      pos.value += (clamped - pos.value) * 0.14
    }
  }
}

function onMouseLeave() {
  interacting.value = false
}

let animId = 0
let lastTs = 0
let stateTimer = 0
let blinkTimer: number | undefined

function tick(ts: number) {
  if (!lastTs) lastTs = ts
  const dt = Math.min(0.05, (ts - lastTs) / 1000)
  lastTs = ts

  if (mode.value === 'walk') {
    pos.value += dir.value * dt * 11 // velocidade suave (~11%/s)
    if (pos.value >= 90) { pos.value = 90; dir.value = -1 }
    if (pos.value <= 10) { pos.value = 10; dir.value = 1 }
  }

  stateTimer += dt
  if (baseMode.value === 'walk' && stateTimer > 6.2 && !interacting.value) {
    baseMode.value = 'sit'
    stateTimer = 0
  } else if (baseMode.value === 'sit' && stateTimer > 3.6) {
    baseMode.value = 'walk'
    stateTimer = 0
  }

  animId = requestAnimationFrame(tick)
}

function scheduleBlink() {
  const delay = 2600 + Math.random() * 2800
  blinkTimer = window.setTimeout(() => {
    blinking.value = true
    setTimeout(() => {
      blinking.value = false
      scheduleBlink()
    }, 150)
  }, delay)
}

onMounted(() => {
  window.addEventListener('mousemove', onMouseMove, { passive: true })
  document.addEventListener('mouseleave', onMouseLeave)
  animId = requestAnimationFrame(tick)
  scheduleBlink()
})

onUnmounted(() => {
  window.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseleave', onMouseLeave)
  cancelAnimationFrame(animId)
  clearTimeout(blinkTimer)
})
</script>

<template>
  <div ref="stage" class="cat-stage" aria-hidden="true">
    <div class="cat-pos" :class="[mode, { busy, blink: blinking }]" :style="{ left: pos + '%' }">
      <!-- ========== MODO ANDANDO (PERFIL) ========== -->
      <svg v-if="mode === 'walk'" class="cat-svg" :class="{ flip: dir === -1 }" viewBox="0 0 46 32" width="42" height="30">
        <!-- Rabinho balançando -->
        <path class="tail" d="M 10 19 Q 3 13 6 7" fill="none" stroke="#0c0c12" stroke-width="3.2" stroke-linecap="round" />
        <path class="tail" d="M 10 19 Q 3 13 6 7" fill="none" stroke="#2c2c38" stroke-width="1" stroke-linecap="round" />
        <!-- Patinhas traseiras e dianteiras -->
        <rect class="paw paw-b" x="13" y="22" width="3.2" height="7" rx="1.6" fill="#0a0a0e" stroke="#2a2a36" stroke-width=".6" />
        <rect class="paw paw-a" x="17" y="22" width="3.2" height="7" rx="1.6" fill="#0a0a0e" stroke="#2a2a36" stroke-width=".6" />
        <rect class="paw paw-a" x="25" y="22" width="3.2" height="7" rx="1.6" fill="#0a0a0e" stroke="#2a2a36" stroke-width=".6" />
        <rect class="paw paw-b" x="29" y="22" width="3.2" height="7" rx="1.6" fill="#0a0a0e" stroke="#2a2a36" stroke-width=".6" />
        <!-- Corpo -->
        <ellipse cx="21" cy="19.5" rx="10" ry="5.5" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <!-- Cabeça e orelhas -->
        <polygon points="27,10 28.5,3.5 32.5,8.5" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="34,8.5 37.5,3.5 39,10" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="28.6,8.5 29.3,5.3 31.3,8" fill="#e87a9c" opacity=".65" />
        <polygon points="35.1,8 37.1,5.3 37.8,8.5" fill="#e87a9c" opacity=".65" />
        <circle cx="33" cy="13.5" r="6.2" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <!-- Olhinhos -->
        <ellipse class="eye" cx="31.3" cy="13.2" rx="1.35" ry="1.8" />
        <ellipse class="eye" cx="35.3" cy="13.2" rx="1.35" ry="1.8" />
        <!-- Narizinho -->
        <polygon points="33.1,15.3 34.3,15.3 33.7,16" fill="#f08aa8" />
      </svg>

      <!-- ========== MODO SENTADO (DE FRENTE, PISCANDO) ========== -->
      <svg v-else-if="mode === 'sit'" class="cat-svg sit-bob" viewBox="0 0 40 34" width="36" height="31">
        <!-- Rabo enrolado ao lado -->
        <path class="tail-sit" d="M 25 28 Q 34 28 33 21 Q 32 17 29 19" fill="none" stroke="#0a0a0e" stroke-width="3.2" stroke-linecap="round" />
        <!-- Corpo sentado -->
        <ellipse cx="20" cy="23.5" rx="7.5" ry="7.2" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <!-- Patinhas juntas na frente -->
        <ellipse cx="17" cy="29.5" rx="2.3" ry="1.6" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".7" />
        <ellipse cx="23" cy="29.5" rx="2.3" ry="1.6" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".7" />
        <!-- Orelhas -->
        <polygon points="13.2,11 12.2,3.8 17.5,8.2" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="22.5,8.2 27.8,3.8 26.8,11" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="14,9.8 13.4,5.6 16.5,8.3" fill="#e87a9c" opacity=".7" />
        <polygon points="23.5,8.3 26.6,5.6 26,9.8" fill="#e87a9c" opacity=".7" />
        <!-- Cabeça -->
        <circle cx="20" cy="13.5" r="6.8" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <!-- Bigodinhos -->
        <path d="M 12 14 L 8 13.2 M 12 15.5 L 8 16.2 M 28 14 L 32 13.2 M 28 15.5 L 32 16.2" stroke="#454556" stroke-width=".7" stroke-linecap="round" />
        <!-- Olhos com pupila seguindo levemente o mouse -->
        <ellipse class="eye" cx="17.3" cy="13.2" rx="1.6" ry="2.1" />
        <ellipse class="eye" cx="22.7" cy="13.2" rx="1.6" ry="2.1" />
        <circle class="pupil" :cx="17.3 + look.dx * 0.3" :cy="13.2 + look.dy * 0.3" r=".75" fill="#0a0a0e" />
        <circle class="pupil" :cx="22.7 + look.dx * 0.3" :cy="13.2 + look.dy * 0.3" r=".75" fill="#0a0a0e" />
        <polygon points="19.3,15.5 20.7,15.5 20,16.3" fill="#f08aa8" />
      </svg>

      <!-- ========== MODO APOIADO PARA BAIXO (SEGUINDO O CURSOR) ========== -->
      <svg v-else class="cat-svg peek" viewBox="0 0 42 32" width="38" height="30">
        <!-- Orelhas atentas -->
        <g :style="{ transform: `translate(${look.dx * 0.6}px, ${look.dy * 0.4}px)` }">
          <polygon points="13,12 11.5,4.5 17.8,9" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
          <polygon points="24.2,9 30.5,4.5 29,12" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" stroke-linejoin="round" />
          <polygon points="13.8,10.6 13,6.4 16.6,9.2" fill="#e87a9c" opacity=".7" />
          <polygon points="25.4,9.2 29,6.4 28.2,10.6" fill="#e87a9c" opacity=".7" />
          <!-- Cabeça apoiada olhando para baixo -->
          <ellipse cx="21" cy="15.5" rx="7.6" ry="6.6" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
          <!-- Bigodinhos -->
          <path d="M 12.5 16 L 8.5 15.5 M 12.5 17.5 L 8.5 18.2 M 29.5 16 L 33.5 15.5 M 29.5 17.5 L 33.5 18.2" stroke="#454556" stroke-width=".7" stroke-linecap="round" />
          <!-- Olhos grandes curiosos seguindo o mouse -->
          <ellipse class="eye" cx="18" cy="15.3" rx="1.8" ry="2.2" />
          <ellipse class="eye" cx="24" cy="15.3" rx="1.8" ry="2.2" />
          <circle class="pupil" :cx="18 + look.dx * 0.45" :cy="15.6 + look.dy * 0.45" r=".85" fill="#0a0a0e" />
          <circle class="pupil" :cx="24 + look.dx * 0.45" :cy="15.6 + look.dy * 0.45" r=".85" fill="#0a0a0e" />
          <polygon points="20.3,17.8 21.7,17.8 21,18.6" fill="#f08aa8" />
        </g>
        <!-- Duas patinhas apoiadas na borda inferior do header -->
        <ellipse class="lpaw" cx="15" cy="24.5" rx="3.1" ry="2.4" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <ellipse class="rpaw" cx="27" cy="24.5" rx="3.1" ry="2.4" fill="#0a0a0e" stroke="#2c2c38" stroke-width=".8" />
        <!-- Almofadinhas sutis nas patinhas -->
        <circle cx="14.2" cy="24.8" r=".55" fill="#3a3a48" />
        <circle cx="15.8" cy="24.8" r=".55" fill="#3a3a48" />
        <circle cx="26.2" cy="24.8" r=".55" fill="#3a3a48" />
        <circle cx="27.8" cy="24.8" r=".55" fill="#3a3a48" />
      </svg>
    </div>
  </div>
</template>

<style scoped>
.cat-stage {
  position: absolute; left: 170px; right: 210px; top: 2px; height: 32px;
  pointer-events: none; overflow: hidden;
}
.cat-pos {
  position: absolute; top: 0; transform: translateX(-50%);
  transition: top .2s ease;
}
.cat-pos.peek { top: 3px; }
.cat-svg { display: block; overflow: visible; }
.cat-svg.flip { transform: scaleX(-1); }

.eye {
  fill: #fff;
  transform-box: fill-box;
  transform-origin: center;
  transition: transform .08s ease;
}
.cat-pos.busy .eye { fill: var(--accent); }
.cat-pos.blink .eye { transform: scaleY(.08); }
.cat-pos.blink .pupil { opacity: 0; }

.tail {
  transform-box: fill-box;
  transform-origin: bottom right;
  animation: wag 1.1s ease-in-out infinite alternate;
}
.paw-a { animation: stepA .48s ease-in-out infinite alternate; }
.paw-b { animation: stepB .48s ease-in-out infinite alternate; }
.sit-bob { animation: breathe 2.4s ease-in-out infinite; }
.lpaw { animation: pawTap 2.2s ease-in-out infinite; }
.rpaw { animation: pawTap 2.2s ease-in-out .3s infinite; }

@keyframes wag { from { transform: rotate(-10deg); } to { transform: rotate(14deg); } }
@keyframes stepA { from { transform: translateY(0) rotate(-10deg); } to { transform: translateY(-2px) rotate(12deg); } }
@keyframes stepB { from { transform: translateY(-2px) rotate(12deg); } to { transform: translateY(0) rotate(-10deg); } }
@keyframes breathe { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-1px); } }
@keyframes pawTap { 0%, 88%, 100% { transform: translateY(0); } 94% { transform: translateY(-1.5px); } }
</style>
