<script setup lang="ts">
// Estado recolhido: mostra aviso > música > foco > timer > cronômetro > hora
import { computed } from 'vue'
import { fmt } from '../lib/format'
import { island, live, now, focus, focusLeft, timer, timerLeft, stopwatch, swElapsed } from '../store/state'
import { S } from '../store/settings'
import { st } from '../store/work'

// órgãos com VPN ligada aparecem como etiquetas coloridas na pílula
const vpns = computed(() => S.orgs.filter(o => st(o.id).vpnUp))

const view = computed(() => {
  const hora = new Date(now.value).toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' })
  const t = island.toast
  if (t) return { text: t.text, icon: t.img ? '' : t.icon || '', img: t.img || '', progress: t.progress ?? null, bars: false, dot: '' }
  const base = { icon: '', img: '', progress: null as number | null, bars: false, dot: '' }
  if (live.media.status === 'Playing' && live.media.title) return { ...base, text: live.media.title, bars: true }
  if (focus.running) return { ...base, text: `${focus.mode === 'focus' ? 'Foco' : 'Pausa'} ${fmt(focusLeft())}`, dot: 'focus' }
  if (timer.running) return { ...base, text: `Timer ${fmt(timerLeft())}`, dot: 'timer' }
  if (stopwatch.running) return { ...base, text: `⏱ ${fmt(swElapsed())}`, dot: 'on' }
  return { ...base, text: hora }
})
</script>

<template>
  <div class="pill">
    <img v-if="view.img" class="pimg" :src="view.img" alt="">
    <span v-if="view.icon">{{ view.icon }}</span>
    <span v-if="view.bars" class="bars"><i /><i /><i /><i /></span>
    <span class="ptext">{{ view.text }}</span>
    <span v-if="view.progress !== null" class="pprog"><b :style="{ width: view.progress + '%' }" /></span>
    <span v-if="view.dot" class="dot" :class="view.dot" />
    <span v-for="o in vpns" v-show="!island.toast" :key="o.id" class="vpn" :style="{ color: o.color, borderColor: o.color }" :title="'VPN ' + o.name + ' ligada'">{{ o.name }}</span>
  </div>
</template>

<style scoped>
.pill {
  position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; gap: 9px;
  padding: 0 16px; font-size: 13px; font-weight: 600; white-space: nowrap; transition: opacity .2s;
}
.ptext { max-width: 170px; overflow: hidden; text-overflow: ellipsis; }
.pimg { width: 24px; height: 24px; border-radius: 6px; object-fit: cover; }
.pprog { width: 70px; height: 5px; border-radius: 3px; background: rgba(255,255,255,.15); overflow: hidden; }
.pprog b { display: block; height: 100%; background: #fff; border-radius: 3px; transition: width .2s; }
.vpn { font-size: 9.5px; font-weight: 800; letter-spacing: .04em; padding: 1px 5px; border: 1px solid; border-radius: 6px; line-height: 1.3; }
.dot { width: 7px; height: 7px; border-radius: 50%; background: #34d058; }
.dot.focus { background: #8b7bff; }
.dot.timer { background: var(--accent); }
.bars { display: inline-flex; gap: 2px; align-items: flex-end; height: 12px; }
.bars i { width: 3px; border-radius: 2px; background: hsl(var(--h) 85% 62%); animation: eq .9s ease-in-out infinite; }
.bars i:nth-child(2) { animation-delay: -.3s } .bars i:nth-child(3) { animation-delay: -.6s } .bars i:nth-child(4) { animation-delay: -.15s }
@keyframes eq { 0%,100% { height: 3px } 50% { height: 12px } }
</style>
