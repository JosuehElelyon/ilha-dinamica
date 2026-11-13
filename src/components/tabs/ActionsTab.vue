<script setup lang="ts">
// Ações personalizadas: atalhos para sites, programas, pastas e comandos
import { reactive, ref } from 'vue'
import { api } from '../../lib/api'
import type { Action } from '../../lib/types'
import { S, save } from '../../store/settings'

const editing = ref(false)
const form = reactive({ open: false, id: 0, emoji: '', name: '', type: 'url' as Action['type'], target: '' })
const PLACEHOLDER: Record<Action['type'], string> = {
  url: 'https://…', file: 'C:\\Caminho\\programa.exe', folder: 'C:\\Users\\…\\Documentos', cmd: 'calc, notepad, ms-settings:, spotify:'
}
const DEFAULT_EMOJI: Record<Action['type'], string> = { url: '🌐', file: '🚀', folder: '📁', cmd: '⚡' }

function openForm(a?: Action) {
  Object.assign(form, a ? { open: true, ...a } : { open: true, id: 0, emoji: '', name: '', type: 'url', target: '' })
}
function click(a: Action, ev: MouseEvent) {
  if ((ev.target as HTMLElement).classList.contains('del')) {
    save('actions', S.actions.filter(x => x.id !== a.id))
    return
  }
  if (editing.value) return openForm(a)
  api.runAction(a.target)
  ;(ev.currentTarget as HTMLElement).animate([{ transform: 'scale(.9)' }, { transform: 'scale(1)' }], { duration: 200 })
}
async function pick() {
  const p = await api.pickPath(form.type === 'folder' ? 'folder' : 'file')
  if (!p) return
  form.target = p
  if (!form.name) form.name = p.split(/[\\/]/).pop()!.replace(/\.(exe|lnk)$/i, '')
}
function submit() {
  let target = form.target.trim()
  if (form.type === 'url' && !/^[a-z][\w+.-]*:/i.test(target)) target = 'https://' + target
  const item: Action = {
    id: form.id || Date.now(), emoji: form.emoji.trim() || DEFAULT_EMOJI[form.type],
    name: form.name.trim(), type: form.type, target
  }
  save('actions', form.id ? S.actions.map(x => (x.id === form.id ? item : x)) : [...S.actions, item])
  form.open = false
}
</script>

<template>
  <div :class="{ editing }">
    <div class="top">
      <span class="set-label">SUAS AÇÕES</span>
      <button class="pillbtn" :class="{ on: editing }" @click="editing = !editing; form.open = false">{{ editing ? 'Concluir' : 'Editar' }}</button>
    </div>
    <form v-if="form.open" class="form" autocomplete="off" @submit.prevent="submit">
      <div class="frow">
        <input v-model="form.emoji" class="in emoji" maxlength="4" placeholder="⚡">
        <input v-model="form.name" class="in grow" maxlength="24" placeholder="Nome (ex.: Spotify)" required>
        <select v-model="form.type" class="in">
          <option value="url">Site</option>
          <option value="file">Programa / arquivo</option>
          <option value="folder">Pasta</option>
          <option value="cmd">Comando / app</option>
        </select>
      </div>
      <div class="frow">
        <input v-model="form.target" class="in grow" :placeholder="PLACEHOLDER[form.type]" required>
        <button v-if="form.type === 'file' || form.type === 'folder'" type="button" class="pillbtn" @click="pick">Procurar…</button>
        <button type="button" class="pillbtn" @click="form.open = false">Cancelar</button>
        <button class="pillbtn primary">Salvar</button>
      </div>
    </form>
    <div v-else class="grid">
      <button v-for="a in S.actions" :key="a.id" class="act" :title="editing ? 'Clique para editar' : a.target" @click="click(a, $event)">
        <span class="em">{{ a.emoji || '⚡' }}</span><span class="nm">{{ a.name }}</span>
        <span class="del" title="Remover">✕</span>
      </button>
      <button class="act new" @click="openForm()"><span class="em">＋</span><span class="nm">Nova ação</span></button>
    </div>
  </div>
</template>

<style scoped>
.top { display: flex; justify-content: space-between; align-items: center; padding: 2px 8px 8px; }
.grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(82px, 1fr)); gap: 8px; padding: 0 8px; max-height: 160px; overflow-y: auto; }
.act { position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 4px;
  height: 74px; border-radius: 16px; background: rgba(255,255,255,.05); font-size: 11.5px; }
.act:hover { background: rgba(255,255,255,.1); }
.em { font-size: 24px; line-height: 1; }
.nm { max-width: 76px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.new { border: 1.5px dashed rgba(255,255,255,.18); background: none; color: var(--mute); }
.del { display: none; position: absolute; top: -4px; right: -4px; width: 20px; height: 20px; border-radius: 50%;
  background: #ff5b4e; color: #fff; font-size: 11px; font-weight: 800; line-height: 20px; }
.editing .act:not(.new) .del { display: block; }
.editing .act:not(.new) { animation: wiggle .3s ease-in-out infinite alternate; }
@keyframes wiggle { from { transform: rotate(-1.5deg) } to { transform: rotate(1.5deg) } }
.form { display: flex; flex-direction: column; gap: 8px; padding: 6px 8px; }
.frow { display: flex; gap: 6px; }
.emoji { width: 46px; text-align: center; font-size: 16px; }
</style>
