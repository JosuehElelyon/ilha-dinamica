<script setup lang="ts">
// Limpeza: cache, temporários, Lixeira, memória e DNS
import { computed, onMounted, ref } from 'vue'
import { api } from '../../lib/api'
import { fmtSize } from '../../lib/format'
import type { ScanResult } from '../../lib/types'
import { S, save } from '../../store/settings'
import { island, showToast } from '../../store/state'

const data = ref<ScanResult | null>(null)
const busy = ref(false)
const working = ref<string[]>([])
const msg = ref('Clique em Analisar para ver quanto espaço dá para liberar')
const confirmRecycle = ref(false)
const ramBusy = ref(false)

const disk = computed(() => data.value?.disk)
const diskPct = computed(() => (disk.value ? (100 * (disk.value.total - disk.value.free)) / disk.value.total : 0))
const total = computed(() => {
  if (!data.value) return 0
  return S.cleanSel.reduce((a, k) => a + (data.value!.cats[k]?.size || 0), 0)
})

async function scan() {
  if (busy.value) return
  busy.value = true
  working.value = data.value ? [...data.value.order] : []
  msg.value = 'Analisando…'
  try {
    data.value = await api.cleanScan()
    const tot = Object.values(data.value.cats).reduce((a, c) => a + c.size, 0)
    msg.value = `${fmtSize(tot)} encontrados no total`
  } catch { msg.value = 'Não foi possível analisar' }
  busy.value = false
  working.value = []
}
onMounted(scan)

function toggle(k: string, on: boolean) {
  save('cleanSel', on ? [...new Set([...S.cleanSel, k])] : S.cleanSel.filter(x => x !== k))
}

async function run() {
  if (busy.value || !S.cleanSel.length) return
  const rb = data.value?.cats.recycle
  if (S.cleanSel.includes('recycle') && rb && rb.count > 0 && !confirmRecycle.value) {
    confirmRecycle.value = true
    setTimeout(() => (confirmRecycle.value = false), 3500)
    return
  }
  confirmRecycle.value = false
  busy.value = true
  working.value = [...S.cleanSel]
  msg.value = 'Limpando… arquivos em uso são pulados'
  const r = await api.cleanRun([...S.cleanSel])
  busy.value = false
  const text = r.freed ? `${fmtSize(r.freed)} liberados` : 'Limpeza concluída'
  showToast({ icon: '🧹', text, ms: 3500 })
  await scan()
  msg.value = `✓ ${text}`
}

async function ram() {
  ramBusy.value = true
  const r = await api.trimRam()
  ramBusy.value = false
  const text = r.freed > 50 * 1024 ** 2 ? `${fmtSize(r.freed)} de RAM liberados` : 'Memória otimizada'
  msg.value = `✓ ${text}`
  showToast({ icon: '🧠', text, ms: 3000 })
}
async function dns() {
  const ok = await api.flushDns()
  msg.value = ok ? '✓ Cache DNS limpo' : 'Não foi possível limpar o DNS'
  showToast({ icon: '🌐', text: ok ? 'Cache DNS limpo' : 'Não foi possível limpar o DNS', ms: 2500 })
}
</script>

<template>
  <div class="clean">
    <div class="top">
      <span class="set-label">DISCO</span>
      <div class="bar disk" :class="{ full: diskPct > 90 }"><b :style="{ width: diskPct + '%' }" /></div>
      <span class="mono small">{{ disk ? `${fmtSize(disk.free)} livres de ${fmtSize(disk.total)}` : '—' }}</span>
      <button class="pillbtn" :disabled="busy" @click="scan">Analisar</button>
      <button class="pillbtn primary" :disabled="busy || !S.cleanSel.length" @click="run">
        {{ confirmRecycle ? 'Esvaziar lixeira também?' : total ? `Limpar ${fmtSize(total)}` : 'Limpar' }}
      </button>
    </div>
    <div class="grid">
      <template v-if="data">
        <label v-for="k in data.order" :key="k" class="row" :class="{ busy: working.includes(k) }"
               :title="data.cats[k].admin && !island.isAdmin ? 'Precisa do app aberto como administrador (menu •••)'
                 : k === 'recycle' ? 'Apaga definitivamente o que está na lixeira' : `${data.cats[k].count} itens`">
          <input type="checkbox" :checked="S.cleanSel.includes(k)" @change="toggle(k, ($event.target as HTMLInputElement).checked)">
          <span>{{ data.cats[k].icon }}</span>
          <span class="nm">{{ data.cats[k].name }}{{ data.cats[k].admin && !island.isAdmin ? ' 🔒' : '' }}</span>
          <span class="sz" :class="{ big: data.cats[k].size > 200 * 1024 ** 2 }">
            {{ data.cats[k].denied && !data.cats[k].size ? 'sem acesso' : fmtSize(data.cats[k].size) }}</span>
        </label>
      </template>
    </div>
    <div class="foot">
      <button class="pillbtn" :disabled="ramBusy" @click="ram">🧠 {{ ramBusy ? 'Otimizando…' : 'Otimizar memória' }}</button>
      <button class="pillbtn" @click="dns">🌐 Limpar cache DNS</button>
      <span class="mute small msg">{{ msg }}</span>
    </div>
  </div>
</template>

<style scoped>
.clean { display: flex; flex-direction: column; gap: 8px; height: 100%; padding: 0 8px; }
.top { display: grid; grid-template-columns: auto 1fr auto auto auto; align-items: center; gap: 10px; }
.disk b { background: linear-gradient(90deg, #6f62ff, #a59bff); }
.disk.full b { background: linear-gradient(90deg, #ff5b4e, #ff8a7e); }
.grid { display: grid; grid-template-columns: 1fr 1fr; gap: 4px 10px; min-height: 120px; }
.row { display: grid; grid-template-columns: 16px 20px 1fr auto; align-items: center; gap: 6px; padding: 5px 8px;
  border-radius: 10px; background: rgba(255,255,255,.04); font-size: 12.5px; cursor: pointer; }
.row:hover { background: rgba(255,255,255,.08); }
.nm { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.sz { font-family: var(--mono); font-size: 11px; color: var(--mute); }
.sz.big { color: var(--accent-2); }
.row.busy .sz { animation: pulse 1s infinite; }
@keyframes pulse { 50% { opacity: .3 } }
.foot { display: flex; align-items: center; gap: 8px; }
.msg { margin-left: auto; text-align: right; }
</style>
