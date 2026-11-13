<script setup lang="ts">
// Processos: CPU/RAM de cada programa, busca e finalizar (com confirmação)
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api } from '../../lib/api'
import { fmtSize } from '../../lib/format'
import type { Proc } from '../../lib/types'
import { S, save } from '../../store/settings'
import { island, showToast } from '../../store/state'

const procs = ref<Proc[]>([])
const filter = ref('')
const group = ref(true)
const confirmKey = ref<string | null>(null)
let poll: number | undefined, confirmT: number | undefined

async function refresh() { procs.value = await api.getProcs() }
onMounted(() => { refresh(); poll = window.setInterval(refresh, 3000) })
onUnmounted(() => clearInterval(poll))

interface Row { n: string; c: number; m: number; count: number; pids: number[] }
const rows = computed<Row[]>(() => {
  let list: Row[]
  if (group.value) {
    const map = new Map<string, Row>()
    for (const p of procs.value) {
      const k = p.n.toLowerCase()
      const g = map.get(k) || { n: p.n, c: 0, m: 0, count: 0, pids: [] }
      g.c += p.c; g.m += p.m; g.count++; g.pids.push(p.p)
      map.set(k, g)
    }
    list = [...map.values()]
  } else list = procs.value.map(p => ({ n: p.n, c: p.c, m: p.m, count: 1, pids: [p.p] }))
  const q = filter.value.trim().toLowerCase()
  if (q) list = list.filter(r => r.n.toLowerCase().includes(q) || r.pids.some(p => String(p) === q))
  return list.sort((a, b) => b[S.procSort] - a[S.procSort]).slice(0, 80)
})

const keyOf = (r: Row) => (group.value ? 'n:' + r.n.toLowerCase() : 'p:' + r.pids[0])
const isProtected = (r: Row) => {
  const n = r.n.toLowerCase()
  return island.protectedNames.has(n) || n === island.selfName || (!group.value && island.ownPids.includes(r.pids[0]))
}

async function kill(r: Row) {
  const k = keyOf(r)
  if (confirmKey.value !== k) {
    confirmKey.value = k
    clearTimeout(confirmT)
    confirmT = window.setTimeout(() => (confirmKey.value = null), 3000)
    return
  }
  confirmKey.value = null
  const res = await api.kill(group.value && r.count > 1 ? null : r.pids[0], r.n)
  showToast(res.ok
    ? { icon: '✅', text: `${r.n} finalizado`, ms: 2500 }
    : { icon: '⚠️', text: res.protected ? `${r.n} é protegido` : `Não deu para finalizar ${r.n}${island.isAdmin ? '' : ' (tente como admin)'}`, ms: 3500 })
  setTimeout(refresh, 400)
}
</script>

<template>
  <div class="procs">
    <div class="top">
      <input v-model="filter" type="text" class="in grow" placeholder="Buscar processo…" autocomplete="off">
      <div class="seg">
        <button :class="{ on: S.procSort === 'c' }" @click="save('procSort', 'c')">CPU</button>
        <button :class="{ on: S.procSort === 'm' }" @click="save('procSort', 'm')">RAM</button>
      </div>
      <label class="chk"><input v-model="group" type="checkbox" @change="confirmKey = null"> Agrupar</label>
    </div>
    <div class="row head mono"><span>NOME</span><span>CPU</span><span>RAM</span><span>{{ procs.length }} ATIVOS</span></div>
    <div class="list">
      <div v-if="!procs.length" class="empty">Carregando processos…</div>
      <div v-else-if="!rows.length" class="empty">Nenhum processo encontrado</div>
      <div v-for="r in rows" :key="keyOf(r)" class="row"
           :title="group ? `PIDs: ${r.pids.slice(0, 12).join(', ')}${r.pids.length > 12 ? '…' : ''}` : `PID ${r.pids[0]}`">
        <span class="nm">{{ r.n }}<span v-if="r.count > 1" class="cnt">×{{ r.count }}</span></span>
        <span class="c" :class="{ hot: r.c >= 10 }">{{ r.c.toFixed(1).replace('.', ',') }}%</span>
        <span class="m" :class="{ hot: r.m >= 1024 ** 3 }">{{ fmtSize(r.m) }}</span>
        <span v-if="isProtected(r)" class="lock" title="Processo essencial do Windows">🔒 sistema</span>
        <button v-else class="kill" :class="{ confirm: confirmKey === keyOf(r) }" @click="kill(r)">
          {{ confirmKey === keyOf(r) ? 'Confirmar' : 'Finalizar' }}</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.procs { display: flex; flex-direction: column; gap: 6px; height: 100%; padding: 0 8px; }
.top { display: flex; align-items: center; gap: 8px; }
.top .seg { width: 110px; }
.row { display: grid; grid-template-columns: 1fr 64px 84px 92px; align-items: center; gap: 8px; padding: 4px 8px; font-size: 12.5px; border-radius: 8px; }
.head { font-size: 10px; color: var(--mute); padding-top: 0; padding-bottom: 0; }
.head span:nth-child(n+2), .c, .m { text-align: right; }
.list { flex: 1; min-height: 0; overflow-y: auto; }
.list .row:hover { background: rgba(255,255,255,.05); }
.nm { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.cnt { color: var(--mute); font-size: 11px; margin-left: 4px; }
.c, .m { font-family: var(--mono); font-size: 11.5px; }
.hot { color: #ffb070; }
.kill { justify-self: end; padding: 3px 10px; border-radius: 10px; font-size: 11px; background: rgba(255,255,255,.07); color: #ddd; }
.kill:hover { background: rgba(255,91,78,.25); color: #fff; }
.kill.confirm { background: #ff5b4e; color: #fff; font-weight: 700; }
.lock { justify-self: end; font-size: 11px; color: var(--mute); padding-right: 6px; }
</style>
