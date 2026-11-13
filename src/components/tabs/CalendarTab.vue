<script setup lang="ts">
// Calendário + lembretes por dia
import { computed, ref } from 'vue'
import { DIAS_CURTOS, MESES_LONGOS, ymd } from '../../lib/format'
import { reminders, remDate, saveReminders } from '../../store/state'

const ref0 = new Date(); ref0.setDate(1)
const month = ref(new Date(ref0))
const sel = ref(ymd(new Date()))
const time = ref('09:00')
const text = ref('')

const title = computed(() => `${MESES_LONGOS[month.value.getMonth()]} ${month.value.getFullYear()}`)
const days = computed(() => {
  const y = month.value.getFullYear(), m = month.value.getMonth()
  const start = new Date(y, m, 1 - new Date(y, m, 1).getDay())
  const today = ymd(new Date())
  const withRem = new Set(reminders.value.filter(r => !r.fired).map(r => r.date))
  return Array.from({ length: 42 }, (_, i) => {
    const d = new Date(start); d.setDate(start.getDate() + i)
    const ds = ymd(d)
    return {
      ds, n: d.getDate(), out: d.getMonth() !== m, we: d.getDay() === 0 || d.getDay() === 6,
      today: ds === today, sel: ds === sel.value, has: withRem.has(ds)
    }
  })
})
const dayTitle = computed(() => {
  const d = new Date(sel.value + 'T12:00')
  if (sel.value === ymd(new Date())) return 'Lembretes · Hoje'
  return `Lembretes · ${DIAS_CURTOS[d.getDay()]} ${d.getDate()} ${MESES_LONGOS[d.getMonth()].slice(0, 3)}`
})
const items = computed(() => reminders.value.filter(r => r.date === sel.value).sort((a, b) => a.time.localeCompare(b.time)))

function shift(n: number) { const d = new Date(month.value); d.setMonth(d.getMonth() + n); month.value = d }
function add() {
  if (!text.value.trim()) return
  const r = { id: Date.now(), date: sel.value, time: time.value || '09:00', text: text.value.trim(), fired: false }
  r.fired = remDate(r) <= new Date()
  reminders.value.push(r)
  saveReminders()
  text.value = ''
}
function remove(id: number) { reminders.value = reminders.value.filter(r => r.id !== id); saveReminders() }
</script>

<template>
  <div class="wrap">
    <div class="cal">
      <div class="head">
        <button class="icon-btn" @click="shift(-1)">‹</button>
        <span>{{ title }}</span>
        <button class="icon-btn" @click="shift(1)">›</button>
      </div>
      <div class="grid">
        <span v-for="(d, i) in ['D', 'S', 'T', 'Q', 'Q', 'S', 'S']" :key="'w' + i" class="wd">{{ d }}</span>
        <span v-for="d in days" :key="d.ds" class="d" :class="{ out: d.out, we: d.we, today: d.today, sel: d.sel, has: d.has }"
              @click="sel = d.ds">{{ d.n }}</span>
      </div>
    </div>
    <div class="day">
      <div class="day-title">{{ dayTitle }}</div>
      <div class="list">
        <div v-if="!items.length" class="empty" style="padding: 14px 4px">Nenhum lembrete neste dia</div>
        <div v-for="r in items" :key="r.id" class="rem" :class="{ done: r.fired }">
          <span class="t">{{ r.time }}</span><span>{{ r.text }}</span>
          <button class="x" title="Excluir" @click="remove(r.id)">✕</button>
        </div>
      </div>
      <form class="form" autocomplete="off" @submit.prevent="add">
        <input v-model="time" type="time" class="in time">
        <input v-model="text" type="text" class="in grow" placeholder="Novo lembrete…" maxlength="120">
        <button class="add" title="Adicionar">+</button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.wrap { display: grid; grid-template-columns: 330px 1fr; gap: 18px; height: 100%; padding: 0 6px; }
.head { display: flex; justify-content: space-between; align-items: center; font-weight: 600; font-size: 13px; }
.grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 1px; text-align: center; font-size: 11.5px; }
.wd { color: var(--mute); font-family: var(--mono); font-size: 10px; padding: 2px 0; }
.d { position: relative; padding: 3px 0; border-radius: 8px; cursor: pointer; }
.d:hover { background: rgba(255,255,255,.07); }
.d.out { color: #4a4a52; }
.d.we { color: #b7b7c0; }
.d.sel { box-shadow: inset 0 0 0 1px var(--accent); }
.d.today { background: var(--accent); color: #111; font-weight: 700; }
.d.has::after { content: ''; position: absolute; bottom: 1px; left: 50%; width: 4px; height: 4px; margin-left: -2px; border-radius: 50%; background: var(--accent); }
.d.today.has::after { background: #111; }
.day { display: flex; flex-direction: column; gap: 6px; min-height: 0; }
.day-title { font-weight: 600; font-size: 13px; padding-top: 2px; }
.list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; min-height: 0; }
.rem { display: grid; grid-template-columns: 46px 1fr auto; gap: 8px; align-items: center; padding: 6px 8px; border-radius: 10px; background: rgba(255,255,255,.04); font-size: 13px; }
.rem .t { font-family: var(--mono); font-size: 11px; color: var(--accent-2); }
.rem.done { opacity: .45; }
.x { color: var(--mute); padding: 0 4px; border-radius: 6px; }
.x:hover { color: #ff6b5e; background: rgba(255,255,255,.06); }
.form { display: flex; gap: 6px; }
.time { width: 92px; }
</style>
