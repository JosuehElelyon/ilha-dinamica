<script setup lang="ts">
// Paleta de comandos global (Ctrl+Alt+K ou botão ⌘K): busca rápida em tudo da ilha.
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { api } from '../lib/api'
import { TOOLS } from '../lib/devtools'
import { ymd } from '../lib/format'
import { games } from '../store/games'
import { MODULES, S, activeModules, type ModuleId } from '../store/settings'
import { island, meetings, openTool, showToast } from '../store/state'
import { st, toggleVpn } from '../store/work'

interface Item {
  id: string
  icon: string
  title: string
  sub: string
  run: () => void
}

const q = ref('')
const idx = ref(0)
const inputEl = ref<HTMLInputElement>()

onMounted(() => nextTick(() => inputEl.value?.focus()))
watch(q, () => (idx.value = 0))

function close() {
  island.paletteOpen = false
}

const allItems = computed<Item[]>(() => {
  const out: Item[] = []
  const today = ymd(new Date())

  // Reuniões de hoje
  for (const m of meetings.value) {
    if (m.date !== today) continue
    out.push({
      id: 'meet-' + m.id,
      icon: '📹',
      title: m.summary,
      sub: m.joinUrl ? `Reunião ${m.startTime} · Entrar` : `Reunião ${m.startTime}`,
      run: () => (m.joinUrl ? api.runAction(m.joinUrl) : (island.tab = 'cal'))
    })
  }

  // Repositórios dos órgãos (abrir no Antigravity)
  for (const o of S.orgs) {
    for (const r of o.repos || []) {
      out.push({
        id: `repo-${o.id}-${r.path}`,
        icon: '🚀',
        title: `${r.name} (Antigravity)`,
        sub: `${o.name} · ${r.path}`,
        run: async () => {
          const ok = await api.openInAntigravity(r.path)
          showToast(ok ? { icon: '🚀', text: `Abrindo ${r.name}…`, ms: 2000 } : { icon: '⚠️', text: 'Antigravity não encontrado', alert: true, ms: 3000 })
        }
      })
    }
  }

  // VPNs dos órgãos
  for (const o of S.orgs) {
    if (o.vpn.type === 'none') continue
    const up = st(o.id).vpnUp
    out.push({
      id: 'vpn-' + o.id,
      icon: up ? '🔓' : '🔐',
      title: `${up ? 'Desligar' : 'Ligar'} VPN ${o.name}`,
      sub: up ? `Conectada (${st(o.id).ip})` : 'Trabalho · VPN',
      run: () => toggleVpn(o)
    })
  }

  // Ações rápidas
  for (const a of S.actions) {
    out.push({
      id: 'act-' + a.id,
      icon: a.emoji || '⚡',
      title: a.name,
      sub: `Ação · ${a.target}`,
      run: () => api.runAction(a.target)
    })
  }

  // Snippets
  for (const s of S.snippets) {
    out.push({
      id: 'snip-' + s.id,
      icon: '📋',
      title: s.title,
      sub: `Snippet · ${s.cat}`,
      run: () => {
        if (/\{[a-zA-Z0-9_]+\}/.test(s.body)) {
          island.tab = 'snippets'
        } else {
          api.copy(s.body)
          showToast({ icon: '📋', text: `Copiado: ${s.title}`, ms: 1800 })
        }
      }
    })
  }

  // Ferramentas de dev
  for (const t of TOOLS) {
    out.push({
      id: 'tool-' + t.id,
      icon: '🔧',
      title: t.label,
      sub: 'Ferramenta',
      run: () => openTool(t.id, '')
    })
  }

  // Abas
  for (const m of activeModules()) {
    out.push({
      id: 'tab-' + m,
      icon: '📑',
      title: `Ir para ${MODULES[m]}`,
      sub: 'Aba',
      run: () => (island.tab = m as ModuleId)
    })
  }

  // Jogos
  for (const g of games.list) {
    if (!g.launch) continue
    out.push({
      id: 'game-' + g.id,
      icon: '🎮',
      title: g.name,
      sub: `Jogo · ${g.source.toUpperCase()}`,
      run: () => api.runAction(g.launch)
    })
  }

  return out
})

const filtered = computed(() => {
  const s = q.value.trim().toLowerCase()
  if (!s) return allItems.value.slice(0, 12)
  const terms = s.split(/\s+/)
  return allItems.value
    .filter(it => {
      const hay = `${it.title} ${it.sub}`.toLowerCase()
      return terms.every(t => hay.includes(t))
    })
    .slice(0, 16)
})

function exec(it?: Item) {
  if (!it) return
  close()
  it.run()
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') {
    e.preventDefault()
    close()
  } else if (e.key === 'ArrowDown') {
    e.preventDefault()
    if (filtered.value.length) idx.value = (idx.value + 1) % filtered.value.length
  } else if (e.key === 'ArrowUp') {
    e.preventDefault()
    if (filtered.value.length) idx.value = (idx.value - 1 + filtered.value.length) % filtered.value.length
  } else if (e.key === 'Enter') {
    e.preventDefault()
    exec(filtered.value[idx.value])
  }
}
</script>

<template>
  <div class="pal-bg" @click.self="close" @keydown="onKey">
    <div class="pal">
      <input ref="inputEl" v-model="q" type="text" class="pal-in" placeholder="Buscar repositório, VPN, snippet, reunião, ação… (Esc fecha)">
      <div class="pal-list">
        <button v-for="(it, i) in filtered" :key="it.id" class="pal-item" :class="{ sel: i === idx }"
                @mouseenter="idx = i" @click="exec(it)">
          <span class="pico">{{ it.icon }}</span>
          <span class="ptxt">
            <b>{{ it.title }}</b>
            <span class="mute">{{ it.sub }}</span>
          </span>
        </button>
        <div v-if="!filtered.length" class="empty small">Nada encontrado para “{{ q }}”</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.pal-bg {
  position: absolute; inset: 0; z-index: 40; display: flex; align-items: flex-start; justify-content: center;
  padding: 28px 18px 14px; background: rgba(8, 8, 12, .72); backdrop-filter: blur(8px); border-radius: inherit;
}
.pal {
  width: 100%; max-width: 560px; max-height: 100%; display: flex; flex-direction: column; gap: 6px;
  padding: 10px; border-radius: 16px; background: #19191f; border: 1px solid rgba(255,255,255,.14);
  box-shadow: 0 18px 40px rgba(0,0,0,.6);
}
.pal-in {
  width: 100%; padding: 8px 12px; border-radius: 10px; border: 1px solid rgba(255,255,255,.12);
  background: rgba(255,255,255,.06); color: #fff; font-size: 13px; outline: none;
}
.pal-in:focus { border-color: var(--accent); }
.pal-list { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 2px; max-height: 240px; }
.pal-item {
  display: flex; align-items: center; gap: 10px; padding: 6px 10px; border-radius: 10px; text-align: left; width: 100%;
}
.pal-item.sel { background: rgba(255,255,255,.1); }
.pico { font-size: 15px; flex: none; }
.ptxt { display: flex; flex-direction: column; min-width: 0; }
.ptxt b { font-size: 12.5px; color: #fff; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ptxt span { font-size: 10.5px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.empty.small { padding: 14px; font-size: 12px; }
</style>
