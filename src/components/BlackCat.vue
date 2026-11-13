<script setup lang="ts">
// Gatinho preto animado no topo da ilha:
// - Anda de um lado para o outro com patinhas, orelhas e rabinho animados
// - Para, senta de frente, olha para você e pisca
// - Quando você interage com a ilha abaixo do topo, ele se apoia na borda olhando para baixo
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

const mode = computed<Mode>(() => (interacting.value && island.expanded ? 'peek' : baseMode.value))

function onMouseMove(e: MouseEvent) {
  const el = stage.value
  if (!el) return
  const r = el.getBoundingClientRect()
  interacting.value = island.expanded && e.clientY > r.bottom + 4
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
    pos.value += dir.value * dt * 10.5
    if (pos.value >= 88) { pos.value = 88; dir.value = -1 }
    if (pos.value <= 12) { pos.value = 12; dir.value = 1 }
  }

  stateTimer += dt
  if (baseMode.value === 'walk' && stateTimer > 6.5 && !interacting.value) {
    baseMode.value = 'sit'
    stateTimer = 0
  } else if (baseMode.value === 'sit' && stateTimer > 3.8) {
    baseMode.value = 'walk'
    stateTimer = 0
  }

  animId = requestAnimationFrame(tick)
}

function scheduleBlink() {
  const delay = 2500 + Math.random() * 2800
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
      <!-- ========== MODO ANDANDO (PERFIL DETALHADO) ========== -->
      <svg v-if="mode === 'walk'" class="cat-svg" :class="{ flip: dir === -1 }" viewBox="0 0 52 34" width="46" height="31">
        <defs>
          <linearGradient id="catFurW" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#1e1e2a" />
            <stop offset="100%" stop-color="#09090e" />
          </linearGradient>
        </defs>
        <!-- Sombra suave no chão -->
        <ellipse cx="24" cy="31.2" rx="13" ry="1.4" fill="#000" opacity=".35" />
        <!-- Rabinho curvo em S -->
        <path class="tail" d="M 12 20 C 5 18 4 10 8 6 C 10 4.5 11.5 6.5 9.5 8" fill="none" stroke="#111118" stroke-width="3.2" stroke-linecap="round" />
        <path class="tail" d="M 12 20 C 5 18 4 10 8 6" fill="none" stroke="#363648" stroke-width=".9" stroke-linecap="round" />
        <!-- Patinhas de trás (mais escuras) -->
        <path class="paw paw-b" d="M 14 23 L 13 29.5 Q 13 31 15.2 31 Q 16.5 31 16.5 29.5 L 17 23 Z" fill="#08080c" stroke="#2a2a38" stroke-width=".6" />
        <path class="paw paw-a" d="M 27 23 L 26.5 29.5 Q 26.5 31 28.7 31 Q 30 31 30 29.5 L 30 23 Z" fill="#08080c" stroke="#2a2a38" stroke-width=".6" />
        <!-- Corpo orgânico com peito fofo -->
        <path d="M 12 21 C 12 15.5 18 14 25 15 C 30 15.6 33 18 33 21.5 C 33 25 28.5 26 22 26 C 15.5 26 12 24.5 12 21 Z"
              fill="url(#catFurW)" stroke="#38384a" stroke-width=".85" />
        <!-- Patinhas da frente -->
        <path class="paw paw-a" d="M 17.5 23.5 L 17 29.8 Q 17 31.2 19.2 31.2 Q 20.6 31.2 20.6 29.8 L 20.8 23.5 Z" fill="url(#catFurW)" stroke="#38384a" stroke-width=".7" />
        <path class="paw paw-b" d="M 29.5 23.5 L 29.5 29.8 Q 29.5 31.2 31.8 31.2 Q 33.2 31.2 33.2 29.8 L 32.8 23.5 Z" fill="url(#catFurW)" stroke="#38384a" stroke-width=".7" />
        <!-- Orelha de trás e orelha da frente -->
        <polygon points="31,10.5 32.2,3.5 36.2,8.8" fill="#12121a" stroke="#38384a" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="36.5,9 40.5,3.2 42,10.5" fill="url(#catFurW)" stroke="#38384a" stroke-width=".8" stroke-linejoin="round" />
        <polygon points="37.6,8.6 40,4.9 40.9,9.2" fill="#f08aa8" opacity=".75" />
        <!-- Cabeça arredondada com bochechinha -->
        <path d="M 29.5 14.5 C 29.5 10.2 33 8 37.2 8 C 41.5 8 44.5 10.8 44.5 14.5 C 44.5 18 41.8 20.2 37.2 20.2 C 32.5 20.2 29.5 18 29.5 14.5 Z"
              fill="url(#catFurW)" stroke="#38384a" stroke-width=".85" />
        <!-- Brilho sutil na testa -->
        <path d="M 34.5 10 Q 37.5 9.2 40 10.2" fill="none" stroke="#484860" stroke-width=".8" stroke-linecap="round" opacity=".7" />
        <!-- Bigodinhos -->
        <path d="M 32.5 15.8 L 28 15.2 M 32.5 17.2 L 28.2 17.8 M 42.2 15.8 L 46.5 15.2 M 42.2 17.2 L 46.2 17.8"
              stroke="#5a5a72" stroke-width=".65" stroke-linecap="round" />
        <!-- Olhos expressivos com pupila e ponto de brilho -->
        <g class="eye-group">
          <ellipse class="eye" cx="35.6" cy="14" rx="1.65" ry="2.05" />
          <ellipse class="eye" cx="40.6" cy="14" rx="1.65" ry="2.05" />
          <circle class="pupil" cx="35.9" cy="14.1" r=".88" fill="#09090d" />
          <circle class="pupil" cx="40.9" cy="14.1" r=".88" fill="#09090d" />
          <circle class="shine" cx="35.2" cy="13.2" r=".55" fill="#fff" />
          <circle class="shine" cx="40.2" cy="13.2" r=".55" fill="#fff" />
        </g>
        <!-- Narizinho e boquinha -->
        <polygon points="37.6,16.1 39.2,16.1 38.4,17" fill="#f28aa8" />
        <path d="M 37 17.6 Q 38.4 18.4 39.8 17.6" fill="none" stroke="#7a7a96" stroke-width=".65" stroke-linecap="round" />
      </svg>

      <!-- ========== MODO SENTADO (DE FRENTE, PISCANDO) ========== -->
      <svg v-else-if="mode === 'sit'" class="cat-svg sit-bob" viewBox="0 0 44 35" width="40" height="32">
        <defs>
          <linearGradient id="catFurS" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#20202d" />
            <stop offset="100%" stop-color="#09090e" />
          </linearGradient>
        </defs>
        <ellipse cx="22" cy="32.2" rx="10.5" ry="1.5" fill="#000" opacity=".35" />
        <!-- Rabo enrolado ao lado com ponta suave -->
        <path class="tail-sit" d="M 27 29.5 Q 37 29.5 36 22 Q 35.2 17.5 31.8 19.5" fill="none" stroke="#111118" stroke-width="3.2" stroke-linecap="round" />
        <path class="tail-sit" d="M 27 29.5 Q 37 29.5 36 22 Q 35.2 17.5 31.8 19.5" fill="none" stroke="#38384a" stroke-width=".8" stroke-linecap="round" />
        <!-- Corpo sentado -->
        <path d="M 14 26 C 14 19.5 17.5 17.5 22 17.5 C 26.5 17.5 30 19.5 30 26 C 30 30.2 27 31.5 22 31.5 C 17 31.5 14 30.2 14 26 Z"
              fill="url(#catFurS)" stroke="#38384a" stroke-width=".85" />
        <!-- Mancha fofa no peito -->
        <path d="M 19.2 21.5 Q 22 20.5 24.8 21.5 Q 24 25.5 22 26.2 Q 20 25.5 19.2 21.5 Z" fill="#2a2a3a" opacity=".75" />
        <!-- Patinhas dianteiras juntas -->
        <ellipse cx="18.6" cy="30.6" rx="2.6" ry="1.7" fill="url(#catFurS)" stroke="#38384a" stroke-width=".75" />
        <ellipse cx="25.4" cy="30.6" rx="2.6" ry="1.7" fill="url(#catFurS)" stroke="#38384a" stroke-width=".75" />
        <path d="M 17.8 30.8 L 17.8 31.8 M 19.4 30.8 L 19.4 31.8 M 24.6 30.8 L 24.6 31.8 M 26.2 30.8 L 26.2 31.8"
              stroke="#48485c" stroke-width=".6" stroke-linecap="round" />
        <!-- Orelhas pontudas com interior rosado -->
        <polygon points="14.2,11.5 12.6,3.4 19,8.4" fill="url(#catFurS)" stroke="#38384a" stroke-width=".85" stroke-linejoin="round" />
        <polygon points="25,8.4 31.4,3.4 29.8,11.5" fill="url(#catFurS)" stroke="#38384a" stroke-width=".85" stroke-linejoin="round" />
        <polygon points="15,10.2 14.1,5.3 17.8,8.5" fill="#f08aa8" opacity=".75" />
        <polygon points="26.2,8.5 29.9,5.3 29,10.2" fill="#f08aa8" opacity=".75" />
        <!-- Cabeça fofa -->
        <ellipse cx="22" cy="13.8" rx="7.6" ry="6.7" fill="url(#catFurS)" stroke="#38384a" stroke-width=".85" />
        <!-- Bochechinhas rosadas sutis -->
        <ellipse cx="16.4" cy="16.2" rx="1.3" ry=".75" fill="#f08aa8" opacity=".28" />
        <ellipse cx="27.6" cy="16.2" rx="1.3" ry=".75" fill="#f08aa8" opacity=".28" />
        <!-- Bigodinhos -->
        <path d="M 13.8 14.5 L 8.8 13.6 M 13.8 16 L 8.8 16.8 M 30.2 14.5 L 35.2 13.6 M 30.2 16 L 35.2 16.8"
              stroke="#5a5a72" stroke-width=".65" stroke-linecap="round" />
        <!-- Olhos grandes com brilho -->
        <g class="eye-group">
          <ellipse class="eye" cx="19" cy="13.5" rx="1.8" ry="2.25" />
          <ellipse class="eye" cx="25" cy="13.5" rx="1.8" ry="2.25" />
          <circle class="pupil" cx="19" cy="13.6" r=".95" fill="#09090d" />
          <circle class="pupil" cx="25" cy="13.6" r=".95" fill="#09090d" />
          <circle class="shine" cx="18.3" cy="12.6" r=".6" fill="#fff" />
          <circle class="shine" cx="24.3" cy="12.6" r=".6" fill="#fff" />
        </g>
        <!-- Nariz e boquinha :3 -->
        <polygon points="21.2,15.7 22.8,15.7 22,16.5" fill="#f28aa8" />
        <path d="M 20.2 17.2 Q 21.1 18 22 17.1 Q 22.9 18 23.8 17.2" fill="none" stroke="#8585a2" stroke-width=".65" stroke-linecap="round" />
      </svg>

      <!-- ========== MODO APOIADO NA BORDA (OLHANDO PARA BAIXO) ========== -->
      <svg v-else class="cat-svg peek" viewBox="0 0 46 34" width="42" height="31">
        <defs>
          <linearGradient id="catFurP" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#222230" />
            <stop offset="100%" stop-color="#09090e" />
          </linearGradient>
        </defs>
        <!-- Pontinha do rabinho atrás -->
        <path class="tail-sit" d="M 32 18 Q 39 14 37 8" fill="none" stroke="#181822" stroke-width="2.6" stroke-linecap="round" />
        <!-- Orelhas atentas -->
        <polygon points="13.5,12 11.8,3.6 18.8,8.8" fill="url(#catFurP)" stroke="#3c3c50" stroke-width=".85" stroke-linejoin="round" />
        <polygon points="27.2,8.8 34.2,3.6 32.5,12" fill="url(#catFurP)" stroke="#3c3c50" stroke-width=".85" stroke-linejoin="round" />
        <polygon points="14.5,10.6 13.4,5.6 17.5,8.8" fill="#f08aa8" opacity=".78" />
        <polygon points="28.5,8.8 32.6,5.6 31.5,10.6" fill="#f08aa8" opacity=".78" />
        <!-- Cabeça apoiada olhando para baixo -->
        <ellipse cx="23" cy="15.2" rx="8.6" ry="7.2" fill="url(#catFurP)" stroke="#3c3c50" stroke-width=".85" />
        <!-- Bochechas rosadas -->
        <ellipse cx="16.8" cy="17.8" rx="1.4" ry=".8" fill="#f08aa8" opacity=".3" />
        <ellipse cx="29.2" cy="17.8" rx="1.4" ry=".8" fill="#f08aa8" opacity=".3" />
        <!-- Bigodinhos -->
        <path d="M 13.8 16 L 8.5 15.2 M 13.8 17.6 L 8.5 18.4 M 32.2 16 L 37.5 15.2 M 32.2 17.6 L 37.5 18.4"
              stroke="#5e5e78" stroke-width=".65" stroke-linecap="round" />
        <!-- Olhos grandes curiosos -->
        <g class="eye-group">
          <ellipse class="eye" cx="19.6" cy="15.1" rx="1.95" ry="2.35" />
          <ellipse class="eye" cx="26.4" cy="15.1" rx="1.95" ry="2.35" />
          <circle class="pupil" cx="19.6" cy="15.6" r="1.05" fill="#09090d" />
          <circle class="pupil" cx="26.4" cy="15.6" r="1.05" fill="#09090d" />
          <circle class="shine" cx="18.9" cy="14.2" r=".65" fill="#fff" />
          <circle class="shine" cx="25.7" cy="14.2" r=".65" fill="#fff" />
        </g>
        <!-- Narizinho e boquinha :3 -->
        <polygon points="22.1,17.7 23.9,17.7 23,18.6" fill="#f28aa8" />
        <path d="M 21.2 19.3 Q 22.1 20 23 19.2 Q 23.9 20 24.8 19.3" fill="none" stroke="#8585a2" stroke-width=".65" stroke-linecap="round" />
        <!-- Duas patinhas apoiadas na borda com almofadinhas rosadas -->
        <g class="lpaw">
          <ellipse cx="16.2" cy="25.2" rx="3.4" ry="2.6" fill="url(#catFurP)" stroke="#3c3c50" stroke-width=".8" />
          <ellipse cx="16.2" cy="25.8" rx="1.1" ry=".75" fill="#f08aa8" opacity=".65" />
          <circle cx="14.7" cy="24.5" r=".55" fill="#f08aa8" opacity=".65" />
          <circle cx="16.2" cy="24.1" r=".55" fill="#f08aa8" opacity=".65" />
          <circle cx="17.7" cy="24.5" r=".55" fill="#f08aa8" opacity=".65" />
        </g>
        <g class="rpaw">
          <ellipse cx="29.8" cy="25.2" rx="3.4" ry="2.6" fill="url(#catFurP)" stroke="#3c3c50" stroke-width=".8" />
          <ellipse cx="29.8" cy="25.8" rx="1.1" ry=".75" fill="#f08aa8" opacity=".65" />
          <circle cx="28.3" cy="24.5" r=".55" fill="#f08aa8" opacity=".65" />
          <circle cx="29.8" cy="24.1" r=".55" fill="#f08aa8" opacity=".65" />
          <circle cx="31.3" cy="24.5" r=".55" fill="#f08aa8" opacity=".65" />
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
.cat-stage {
  position: absolute; left: 170px; right: 210px; top: 1px; height: 34px;
  pointer-events: none; overflow: hidden;
}
.cat-pos {
  position: absolute; top: 0; transform: translateX(-50%);
  transition: top .22s ease;
}
.cat-pos.peek { top: 2px; }
.cat-svg { display: block; overflow: visible; }
.cat-svg.flip { transform: scaleX(-1); }

.eye {
  fill: #ffd966;
  transform-box: fill-box;
  transform-origin: center;
  transition: transform .09s ease;
}
.cat-pos.busy .eye { fill: var(--accent); }
.cat-pos.blink .eye { transform: scaleY(.08); }
.cat-pos.blink .pupil,
.cat-pos.blink .shine { opacity: 0; }

.tail {
  transform-box: fill-box;
  transform-origin: bottom right;
  animation: wag 1.1s ease-in-out infinite alternate;
}
.tail-sit {
  transform-box: fill-box;
  transform-origin: bottom left;
  animation: tailSway 2.2s ease-in-out infinite alternate;
}
.paw-a { animation: stepA .48s ease-in-out infinite alternate; }
.paw-b { animation: stepB .48s ease-in-out infinite alternate; }
.sit-bob { animation: breathe 2.4s ease-in-out infinite; }
.lpaw { animation: pawTap 2.4s ease-in-out infinite; }
.rpaw { animation: pawTap 2.4s ease-in-out .35s infinite; }

@keyframes wag { from { transform: rotate(-9deg); } to { transform: rotate(12deg); } }
@keyframes tailSway { from { transform: rotate(-4deg); } to { transform: rotate(6deg); } }
@keyframes stepA { from { transform: translateY(0) rotate(-9deg); } to { transform: translateY(-2px) rotate(11deg); } }
@keyframes stepB { from { transform: translateY(-2px) rotate(11deg); } to { transform: translateY(0) rotate(-9deg); } }
@keyframes breathe { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-1px); } }
@keyframes pawTap { 0%, 88%, 100% { transform: translateY(0); } 94% { transform: translateY(-1.5px); } }
</style>
