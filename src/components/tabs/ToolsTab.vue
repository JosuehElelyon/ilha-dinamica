<script setup lang="ts">
// Caixa de ferramentas de dev: JSON, JWT, Base64, URL, datas, UUID, hash, case, contagem
import { onMounted, ref, watch } from 'vue'
import { api } from '../../lib/api'
import { TOOLS, detect, run, type ToolResult } from '../../lib/devtools'
import { live, showToast, toolbox } from '../../store/state'

const result = ref<ToolResult>({ ok: true, out: '' })
const minify = ref(false)

async function compute() {
  result.value = await run(toolbox.tool, toolbox.input, { minify: minify.value })
}
watch(() => [toolbox.tool, toolbox.input, minify.value], compute)

function paste() {
  const t = live.clips[0]?.text
  if (!t) return showToast({ icon: '📋', text: 'Nada no histórico do clipboard', ms: 2000 })
  toolbox.input = t
  const d = detect(t)
  if (d) toolbox.tool = d
}
function copyOut() {
  if (!result.value.out) return
  api.copy(result.value.out)
  showToast({ icon: '✅', text: 'Resultado copiado', ms: 1500 })
}
function useOut() { toolbox.input = result.value.out }
onMounted(() => {
  if (!toolbox.input && live.clips[0]) paste()
  compute()
})
</script>

<template>
  <div class="tools">
    <div class="chips">
      <button v-for="t in TOOLS" :key="t.id" class="chip" :class="{ on: toolbox.tool === t.id }" :title="t.hint" @click="toolbox.tool = t.id">
        {{ t.label }}</button>
    </div>
    <div class="io">
      <div class="col">
        <textarea v-model="toolbox.input" class="area" spellcheck="false"
                  :placeholder="toolbox.tool === 'ts' ? 'Timestamp ou data (vazio = agora)' : toolbox.tool === 'uuid' ? '(não precisa de entrada)' : 'Cole aqui (ou use o botão)'" />
        <div class="toolbar">
          <button class="pillbtn" @click="paste">📋 Último copiado</button>
          <button class="pillbtn" @click="toolbox.input = ''">Limpar</button>
          <label v-if="toolbox.tool === 'json'" class="chk"><input v-model="minify" type="checkbox"> Minificar</label>
        </div>
      </div>
      <div class="col">
        <pre class="area out" :class="{ err: !result.ok }">{{ result.out }}</pre>
        <div class="toolbar">
          <span class="info mute small">{{ result.info }}</span>
          <button class="pillbtn" title="Usar o resultado como nova entrada" @click="useOut">↺</button>
          <button class="pillbtn primary" @click="copyOut">Copiar</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tools { display: flex; flex-direction: column; gap: 6px; padding: 0 6px 6px; }
.chips { display: flex; gap: 4px; flex-wrap: wrap; }
.chip { padding: 3px 9px; border-radius: 10px; font-size: 11px; background: rgba(255,255,255,.06); color: #c9c9cf; font-family: var(--mono); }
.chip:hover { background: rgba(255,255,255,.12); }
.chip.on { background: var(--accent); color: #111; font-weight: 700; }
.io { flex: 1; min-height: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
.col { display: flex; flex-direction: column; gap: 5px; min-height: 0; }
.area { flex: 1; min-height: 0; resize: none; overflow: auto; border: 1px solid var(--line); border-radius: 12px; padding: 8px 10px;
  background: rgba(255,255,255,.03); color: var(--txt); font: 11.5px/1.45 var(--mono); outline: none; user-select: text; white-space: pre-wrap; word-break: break-all; }
.out { background: rgba(0,0,0,.25); }
.out.err { color: #ff8a7e; }
.toolbar { display: flex; align-items: center; gap: 6px; }
.info { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
</style>
