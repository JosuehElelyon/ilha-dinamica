<script setup lang="ts">
// Histórico do clipboard (só em memória)
import { ref } from 'vue'
import { api } from '../../lib/api'
import { ago } from '../../lib/format'
import { live, now } from '../../store/state'

const copied = ref<number | null>(null)
// lê `now` para o "há X min" se atualizar sozinho
const agoText = (t: number) => (now.value, ago(t))
function kind(t: string) {
  t = t.trim()
  if (/^\S+@\S+\.\S+$/.test(t)) return 'MAIL'
  if (/^https?:\/\//i.test(t)) return 'LINK'
  if (/^[\d\s().+-]{8,}$/.test(t)) return 'NÚMERO'
  return 'TEXTO'
}
function copy(text: string, t: number) {
  api.copy(text)
  copied.value = t
  setTimeout(() => (copied.value = null), 1200)
}
</script>

<template>
  <div>
    <div class="list">
      <div v-if="!live.clips.length" class="empty">Copie algo (Ctrl+C) e aparece aqui. Fica só na memória.</div>
      <div v-for="c in live.clips" :key="c.t + c.text" class="clip" :class="{ copied: copied === c.t }"
           title="Clique para copiar de novo" @click="copy(c.text, c.t)">
        <span class="kind">{{ kind(c.text) }}</span>
        <span class="txt">{{ c.text.replace(/\s+/g, ' ') }}</span>
        <span class="ago">{{ copied === c.t ? 'COPIADO' : agoText(c.t) }}</span>
      </div>
    </div>
    <button class="link danger" @click="api.clearClips()">Limpar histórico</button>
  </div>
</template>

<style scoped>
.list { display: flex; flex-direction: column; max-height: 150px; overflow-y: auto; }
.clip { display: grid; grid-template-columns: 60px 1fr auto; align-items: center; gap: 10px; padding: 9px 8px;
  border-bottom: 1px solid var(--line); cursor: pointer; border-radius: 8px; }
.clip:hover { background: rgba(255,255,255,.05); }
.kind, .ago { font-family: var(--mono); font-size: 10px; color: var(--mute); letter-spacing: .1em; }
.txt { font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.copied .ago { color: #34d058; }
.link { margin: 10px 8px; font-size: 13px; font-weight: 600; }
</style>
