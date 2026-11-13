<script setup lang="ts">
// Snippets: trechos e comandos reutilizáveis, com variáveis {assim}
import { computed, reactive, ref } from 'vue'
import { api } from '../../lib/api'
import { fill, varsOf, type Snippet } from '../../lib/snippets'
import { S, save } from '../../store/settings'
import { showToast } from '../../store/state'

const q = ref('')
const cat = ref('Todos')
const editing = ref(false)
const filling = ref<Snippet | null>(null)
const values = reactive<Record<string, string>>({})
const form = reactive({ open: false, id: 0, title: '', cat: '', body: '' })

const cats = computed(() => ['Todos', ...new Set(S.snippets.map(s => s.cat).filter(Boolean))])
const list = computed(() => {
  const t = q.value.trim().toLowerCase()
  return S.snippets.filter(s =>
    (cat.value === 'Todos' || s.cat === cat.value) &&
    (!t || s.title.toLowerCase().includes(t) || s.body.toLowerCase().includes(t) || s.cat.toLowerCase().includes(t)))
})

function copy(text: string, title: string) {
  api.copy(text)
  showToast({ icon: '📋', text: `Copiado: ${title}`, ms: 1800 })
}
function pick(s: Snippet) {
  if (editing.value) return openForm(s)
  const vars = varsOf(s.body)
  if (!vars.length) return copy(s.body, s.title)
  for (const k of Object.keys(values)) delete values[k]
  vars.forEach(v => (values[v] = v === 'data' ? new Date().toLocaleDateString('pt-BR') : ''))
  filling.value = s
}
function confirmFill() {
  if (!filling.value) return
  copy(fill(filling.value.body, values), filling.value.title)
  filling.value = null
}
function openForm(s?: Snippet) {
  Object.assign(form, s ? { open: true, ...s } : { open: true, id: 0, title: '', cat: cat.value === 'Todos' ? '' : cat.value, body: '' })
}
function submit() {
  const item: Snippet = { id: form.id || Date.now(), title: form.title.trim(), cat: form.cat.trim() || 'Geral', body: form.body }
  save('snippets', form.id ? S.snippets.map(x => (x.id === form.id ? item : x)) : [...S.snippets, item])
  form.open = false
}
function remove(s: Snippet) { save('snippets', S.snippets.filter(x => x.id !== s.id)) }
const label = (v: string) => v.replace(/[_-]/g, ' ')
</script>

<template>
  <div class="snip">
    <!-- preencher variáveis -->
    <form v-if="filling" class="fill" autocomplete="off" @submit.prevent="confirmFill">
      <div class="toolbar">
        <b class="grow ttl">{{ filling.title }}</b>
        <button type="button" class="pillbtn" @click="filling = null">Cancelar</button>
        <button class="pillbtn primary">Copiar</button>
      </div>
      <div class="vars">
        <label v-for="(_, k) in values" :key="k" class="var">
          <span class="set-label">{{ label(String(k)).toUpperCase() }}</span>
          <input v-model="values[k]" type="text" class="in" :placeholder="String(k)">
        </label>
      </div>
      <pre class="preview">{{ fill(filling.body, values) }}</pre>
    </form>

    <!-- criar / editar -->
    <form v-else-if="form.open" class="fill" autocomplete="off" @submit.prevent="submit">
      <div class="toolbar">
        <input v-model="form.title" class="in grow" placeholder="Título" required maxlength="60">
        <input v-model="form.cat" class="in catin" placeholder="Categoria" maxlength="16" list="snip-cats">
        <datalist id="snip-cats"><option v-for="c in cats.slice(1)" :key="c" :value="c" /></datalist>
        <button type="button" class="pillbtn" @click="form.open = false">Cancelar</button>
        <button class="pillbtn primary">Salvar</button>
      </div>
      <textarea v-model="form.body" class="in body" spellcheck="false" required
                placeholder="Conteúdo. Use {variavel} para campos que você preenche na hora de copiar." />
      <span class="mute small">Dica: <code>{namespace}</code> vira um campo para preencher na hora de copiar</span>
    </form>

    <!-- lista -->
    <template v-else>
      <div class="toolbar">
        <input v-model="q" type="text" class="in grow" placeholder="Buscar snippet…">
        <button class="pillbtn" :class="{ on: editing }" @click="editing = !editing">{{ editing ? 'Concluir' : 'Editar' }}</button>
        <button class="pillbtn primary" @click="openForm()">+ Novo</button>
      </div>
      <div class="cats">
        <button v-for="c in cats" :key="c" class="chip" :class="{ on: cat === c }" @click="cat = c">{{ c }}</button>
      </div>
      <div class="grid" :class="{ editing }">
        <div v-if="!list.length" class="empty">Nenhum snippet encontrado</div>
        <button v-for="s in list" :key="s.id" class="card" :title="editing ? 'Clique para editar' : 'Clique para copiar'" @click="pick(s)">
          <span class="t">{{ s.title }}</span>
          <span class="b">{{ s.body.split('\n')[0] }}</span>
          <span class="c">{{ s.cat }}<template v-if="varsOf(s.body).length"> · {{ varsOf(s.body).length }} campo(s)</template></span>
          <span v-if="editing" class="del" title="Apagar" @click.stop="remove(s)">✕</span>
        </button>
      </div>
    </template>
  </div>
</template>

<style scoped>
.snip { display: flex; flex-direction: column; gap: 6px; padding: 0 6px 6px; }
.toolbar { display: flex; align-items: center; gap: 6px; }
.cats { display: flex; gap: 4px; flex-wrap: wrap; }
.chip { padding: 2px 9px; border-radius: 10px; font-size: 11px; background: rgba(255,255,255,.06); color: #c9c9cf; }
.chip.on { background: var(--accent); color: #111; font-weight: 700; }
.grid { flex: 1; min-height: 0; overflow-y: auto; display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; align-content: start; }
.card { position: relative; display: flex; flex-direction: column; gap: 2px; padding: 7px 10px; border-radius: 12px; background: rgba(255,255,255,.04); text-align: left; min-width: 0; }
.card:hover { background: rgba(255,255,255,.09); }
.t { font-size: 12.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.b { font: 10.5px var(--mono); color: #b9b9c2; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.c { font-size: 10px; color: var(--mute); }
.del { position: absolute; top: 4px; right: 6px; width: 18px; height: 18px; border-radius: 50%; background: #ff5b4e; color: #fff; font-size: 10px; line-height: 18px; text-align: center; }
.fill { display: flex; flex-direction: column; gap: 6px; flex: 1; min-height: 0; }
.ttl { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.vars { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; }
.var { display: flex; flex-direction: column; gap: 2px; }
.preview { flex: 1; min-height: 0; overflow: auto; margin: 0; padding: 8px 10px; border-radius: 12px; background: rgba(0,0,0,.25);
  font: 11px/1.45 var(--mono); white-space: pre-wrap; word-break: break-all; user-select: text; }
.catin { width: 130px; }
.body { flex: 1; min-height: 0; resize: none; font: 11.5px/1.45 var(--mono); }
code { font-family: var(--mono); color: var(--accent-2); }
</style>
