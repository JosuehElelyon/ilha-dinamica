<script setup lang="ts">
// Calendário + lembretes por dia
import { computed, ref } from 'vue'
import { api } from '../../lib/api'
import { DIAS_CURTOS, MESES_LONGOS, ymd } from '../../lib/format'
import { S, save } from '../../store/settings'
import { meetings, meetingsState, now, refreshMeetings, reminders, remDate, saveReminders } from '../../store/state'

const ref0 = new Date(); ref0.setDate(1)
const month = ref(new Date(ref0))
const sel = ref(ymd(new Date()))
const time = ref('09:00')
const text = ref('')
const cfgOpen = ref(false)
const icsInput = ref(S.icsUrl)

const title = computed(() => `${MESES_LONGOS[month.value.getMonth()]} ${month.value.getFullYear()}`)
const days = computed(() => {
  const y = month.value.getFullYear(), m = month.value.getMonth()
  const start = new Date(y, m, 1 - new Date(y, m, 1).getDay())
  const today = ymd(new Date())
  const withRem = new Set([
    ...reminders.value.filter(r => !r.fired).map(r => r.date),
    ...meetings.value.map(mt => mt.date)
  ])
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
  if (sel.value === ymd(new Date())) return 'Agenda · Hoje'
  return `Agenda · ${DIAS_CURTOS[d.getDay()]} ${d.getDate()} ${MESES_LONGOS[d.getMonth()].slice(0, 3)}`
})
const dayMeetings = computed(() => meetings.value.filter(m => m.date === sel.value))
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
async function saveIcs() {
  const u = icsInput.value.trim()
  save('icsUrl', u)
  await refreshMeetings(u)
  if (!meetingsState.error) cfgOpen.value = false
}
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
      <div class="day-head">
        <span class="day-title">{{ dayTitle }}</span>
        <button v-if="S.icsUrl" class="pillbtn tinybtn" :disabled="meetingsState.loading" title="Atualizar reuniões" @click="refreshMeetings(S.icsUrl)">{{ meetingsState.loading ? '…' : '↻' }}</button>
        <button class="pillbtn tinybtn" :class="{ primary: cfgOpen }" title="Configurar calendário Teams / Outlook (.ics)" @click="cfgOpen = !cfgOpen; icsInput = S.icsUrl">📹 Teams</button>
      </div>
      <form v-if="cfgOpen" class="ics-cfg" autocomplete="off" @submit.prevent="saveIcs">
        <input v-model="icsInput" type="text" class="in" placeholder="Link .ics do Outlook/Teams (https://… ou webcal://…)">
        <div class="ics-row">
          <span class="mute tiny">Outlook Web → Config. → Calendário → Publicar calendário → ICS</span>
          <button type="submit" class="pillbtn primary tinybtn" :disabled="meetingsState.loading">Salvar</button>
        </div>
        <span v-if="meetingsState.error" class="err tiny">{{ meetingsState.error }}</span>
      </form>
      <div v-else class="list">
        <div v-for="m in dayMeetings" :key="m.id" class="rem meet" :class="{ done: !m.allDay && m.endMs < now }">
          <span class="t">{{ m.allDay ? 'Dia' : m.startTime }}</span>
          <span class="mtxt" :title="`${m.startTime}–${m.endTime} · ${m.summary}`">📹 {{ m.summary }}</span>
          <button v-if="m.joinUrl" class="pillbtn join" title="Entrar na reunião" @click="api.runAction(m.joinUrl)">Entrar ↗</button>
          <span v-else class="mute tiny">{{ m.allDay ? '' : m.endTime }}</span>
        </div>
        <div v-if="!dayMeetings.length && !items.length" class="empty" style="padding: 14px 4px">Nenhum evento neste dia</div>
        <div v-for="r in items" :key="r.id" class="rem" :class="{ done: r.fired }">
          <span class="t">{{ r.time }}</span><span>{{ r.text }}</span>
          <button class="x" title="Excluir" @click="remove(r.id)">✕</button>
        </div>
      </div>
      <form v-if="!cfgOpen" class="form" autocomplete="off" @submit.prevent="add">
        <input v-model="time" type="time" class="in time">
        <input v-model="text" type="text" class="in grow" placeholder="Novo lembrete…" maxlength="120">
        <button class="add" title="Adicionar">+</button>
      </form>
    </div>
  </div>
</template>

<style scoped>
.wrap {
  display: grid; grid-template-columns: 320px minmax(0, 1fr); grid-template-rows: minmax(0, 1fr);
  gap: 16px; padding: 0 4px; overflow: hidden;
}
.cal { display: flex; flex-direction: column; justify-content: space-between; min-height: 0; }
.head { display: flex; justify-content: space-between; align-items: center; font-weight: 600; font-size: 12.5px; }
.grid { display: grid; grid-template-columns: repeat(7, 1fr); gap: 1px; text-align: center; font-size: 11px; }
.wd { color: var(--mute); font-family: var(--mono); font-size: 9.5px; padding: 1px 0; }
.d { position: relative; padding: 2px 0; border-radius: 7px; cursor: pointer; line-height: 1.35; }
.d:hover { background: rgba(255,255,255,.07); }
.d.out { color: #4a4a52; }
.d.we { color: #b7b7c0; }
.d.sel { box-shadow: inset 0 0 0 1px var(--accent); }
.d.today { background: var(--accent); color: #111; font-weight: 700; }
.d.has::after { content: ''; position: absolute; bottom: 1px; left: 50%; width: 3.5px; height: 3.5px; margin-left: -1.75px; border-radius: 50%; background: var(--accent); }
.d.today.has::after { background: #111; }
.day { display: flex; flex-direction: column; gap: 6px; min-height: 0; overflow: hidden; }
.day-head { display: flex; align-items: center; gap: 6px; flex: none; }
.day-title { font-weight: 600; font-size: 13px; margin-right: auto; }
.tinybtn { padding: 3px 8px; font-size: 11px; }
.ics-cfg { flex: 1; display: flex; flex-direction: column; gap: 6px; justify-content: center; }
.ics-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.tiny { font-size: 10.5px; }
.err { color: #ff8a7e; }
.list { flex: 1; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; min-height: 0; }
.rem { display: grid; grid-template-columns: 46px 1fr auto; gap: 8px; align-items: center; padding: 5px 8px; border-radius: 10px; background: rgba(255,255,255,.04); font-size: 12.5px; }
.rem.meet { border-left: 2px solid #5b7cff; }
.mtxt { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.join { padding: 2px 8px; font-size: 11px; background: rgba(91, 124, 255, .25); color: #cfd8ff; }
.join:hover { background: rgba(91, 124, 255, .45); color: #fff; }
.rem .t { font-family: var(--mono); font-size: 11px; color: var(--accent-2); }
.rem.done { opacity: .45; }
.x { color: var(--mute); padding: 0 4px; border-radius: 6px; }
.x:hover { color: #ff6b5e; background: rgba(255,255,255,.06); }
.form { display: flex; gap: 6px; flex: none; }
.form .in { padding: 5px 8px; font-size: 12px; }
.time { width: auto; min-width: 92px; flex: none; }
</style>
