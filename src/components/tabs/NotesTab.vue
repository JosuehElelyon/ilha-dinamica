<script setup lang="ts">
// Bloco de notas que salva sozinho
import { onMounted, ref } from 'vue'
import { api } from '../../lib/api'

const text = ref('')
let t: number | undefined
onMounted(async () => { text.value = (await api.getData()).notes || '' })
function onInput() {
  clearTimeout(t)
  t = window.setTimeout(() => api.saveNotes(text.value), 400)
}
</script>

<template>
  <div>
    <textarea v-model="text" placeholder="Anote algo rápido… (salva automaticamente)" @input="onInput" />
  </div>
</template>

<style scoped>
div { height: 100%; }
textarea { width: 100%; height: 100%; resize: none; border: 1px solid var(--line); border-radius: 16px; padding: 12px 14px;
  background: rgba(255,255,255,.03); color: var(--txt); font: 14px/1.5 "Segoe UI", system-ui; outline: none; user-select: text; }
textarea:focus { border-color: rgba(255,255,255,.18); }
</style>
