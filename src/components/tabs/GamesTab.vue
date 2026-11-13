<script setup lang="ts">
// Jogos: instalados (Steam, Epic, GOG) + amigos da Steam online, com "Entrar na partida"
import { computed, onMounted, reactive, ref } from 'vue'
import { api } from '../../lib/api'
import { fmtSize, hashHue } from '../../lib/format'
import type { Friend, Game } from '../../lib/types'
import { S, save } from '../../store/settings'
import { showToast } from '../../store/state'
import { games, loadGames, loadFriends, refreshSteam, ownedGame, joinUrl, STATE } from '../../store/games'

const q = ref('')
const source = ref<'all' | Game['source']>('all')
const config = ref(false)
const form = reactive({ key: '', steamId: S.steamId })
const brokenImg = reactive(new Set<string>())

const filtered = computed(() => {
  const t = q.value.trim().toLowerCase()
  return games.list.filter(g => (source.value === 'all' || g.source === source.value) && (!t || g.name.toLowerCase().includes(t)))
})
const sources = computed(() => {
  const c = (s: string) => games.list.filter(g => g.source === s).length
  return [
    { id: 'all' as const, label: `Todos ${games.list.length}` },
    ...(['steam', 'epic', 'gog'] as const).filter(s => c(s)).map(s => ({ id: s, label: `${{ steam: 'Steam', epic: 'Epic', gog: 'GOG' }[s]} ${c(s)}` }))
  ]
})

function play(g: Game) {
  if (!g.launch) return
  api.runAction(g.launch)
  showToast({ icon: '🎮', text: `Abrindo ${g.name}…`, ms: 2500 })
}
function join(f: Friend) {
  api.runAction(joinUrl(f))
  showToast({ icon: '🎮', text: `Entrando na partida de ${f.name}…`, ms: 3000 })
}
const chat = (f: Friend) => api.runAction(`steam://friends/message/${f.steamid}`)
const store = (f: Friend) => api.runAction(`steam://store/${f.gameid}`)

async function saveConfig() {
  if (form.key) await api.steamSetKey(form.key)
  save('steamId', form.steamId.trim())
  form.key = ''
  await refreshSteam()
  config.value = false
  loadFriends()
}
async function forgetKey() { await api.steamSetKey(''); await refreshSteam(); games.friends = [] }

const initials = (n: string) => n.split(/\s+/).filter(Boolean).slice(0, 2).map(w => w[0]).join('').toUpperCase()
const tile = (g: Game) => `linear-gradient(135deg, hsl(${hashHue(g.name)} 55% 28%), hsl(${(hashHue(g.name) + 40) % 360} 60% 16%))`
const badge = { steam: 'STEAM', epic: 'EPIC', gog: 'GOG' }

onMounted(() => { if (!games.list.length) loadGames() })
</script>

<template>
  <div class="games">
    <!-- ================= JOGOS INSTALADOS ================= -->
    <div class="left">
      <div class="toolbar">
        <input v-model="q" type="text" class="in grow" placeholder="Buscar jogo…">
        <button class="pillbtn" title="Abrir a Steam" @click="api.runAction('steam://open/main')">Steam</button>
        <button class="pillbtn" :disabled="games.loading" title="Procurar jogos de novo" @click="loadGames()">↻</button>
      </div>
      <div class="chips">
        <button v-for="s in sources" :key="s.id" class="chip" :class="{ on: source === s.id }" @click="source = s.id">{{ s.label }}</button>
      </div>
      <div class="grid">
        <div v-if="games.loading && !games.list.length" class="empty">Procurando jogos…</div>
        <div v-else-if="!filtered.length" class="empty">Nenhum jogo encontrado (Steam, Epic Games e GOG)</div>
        <button v-for="g in filtered" :key="g.id" class="game" :title="`${g.name}${g.size ? ' · ' + fmtSize(g.size) : ''} — clique para jogar`" @click="play(g)">
          <img v-if="g.image && !brokenImg.has(g.id)" :src="g.image" alt="" loading="lazy" @error="brokenImg.add(g.id)">
          <div v-else class="ph" :style="{ background: tile(g) }">{{ initials(g.name) }}</div>
          <span class="gname">{{ g.name }}</span>
          <span class="src" :class="g.source">{{ badge[g.source] }}</span>
          <span class="playbtn">▶</span>
        </button>
      </div>
    </div>

    <!-- ================= AMIGOS DA STEAM ================= -->
    <div class="right">
      <div class="toolbar">
        <span class="set-label grow">AMIGOS ONLINE{{ games.friends.length ? ` (${games.friends.length})` : '' }}</span>
        <button class="icon-btn" title="Lista de amigos da Steam" @click="api.runAction('steam://open/friends')">👥</button>
        <button class="icon-btn" title="Configurar" @click="config = !config">⚙</button>
      </div>

      <form v-if="config || !games.steam?.has_key" class="cfg" autocomplete="off" @submit.prevent="saveConfig">
        <span class="mute tiny">
          Para ver seus amigos, crie uma chave grátis em
          <a @click="api.runAction('https://steamcommunity.com/dev/apikey')">steamcommunity.com/dev/apikey</a>
          (domínio: <code>localhost</code>). Ela fica no Gerenciador de Credenciais do Windows.
        </span>
        <input v-model="form.key" type="password" class="in" :placeholder="games.steam?.has_key ? '•••••• chave salva (digite para trocar)' : 'Chave da Steam Web API'">
        <input v-model="form.steamId" class="in" :placeholder="games.steam?.steamid ? `SteamID: ${games.steam.steamid} (detectado)` : 'SteamID64'">
        <label class="chk tiny"><input type="checkbox" :checked="S.friendAlerts" @change="save('friendAlerts', ($event.target as HTMLInputElement).checked)"> Avisar quando um amigo começar a jogar algo que eu tenho</label>
        <div class="toolbar">
          <a v-if="games.steam?.has_key" class="tiny" @click="forgetKey">Esquecer chave</a>
          <span class="grow" />
          <button v-if="games.steam?.has_key" type="button" class="pillbtn" @click="config = false">Cancelar</button>
          <button class="pillbtn primary">Salvar</button>
        </div>
      </form>

      <div v-else class="friends">
        <div v-if="games.friendsError" class="empty small">{{ games.friendsError }}</div>
        <div v-else-if="!games.friends.length" class="empty small">{{ games.friendsLoading ? 'Carregando…' : 'Nenhum amigo online' }}</div>
        <div v-for="f in games.friends" :key="f.steamid" class="friend" :class="{ ingame: !!f.game, away: f.state > 1 }">
          <img v-if="f.avatar" :src="f.avatar" class="av" alt="">
          <div v-else class="av ph2">{{ initials(f.name) }}</div>
          <div class="fi">
            <span class="fn">{{ f.name }}</span>
            <span class="fs">{{ f.game ? '🎮 ' + f.game : STATE[f.state] }}</span>
          </div>
          <template v-if="f.game">
            <button v-if="ownedGame(f)" class="pillbtn primary join" title="Entrar na partida do amigo" @click="join(f)">Entrar</button>
            <button v-else class="pillbtn join" title="Você não tem este jogo instalado" @click="store(f)">Loja</button>
          </template>
          <button class="icon-btn" title="Conversar" @click="chat(f)">💬</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.games { display: grid; grid-template-columns: 1.75fr 1fr; gap: 12px; padding: 0 6px 6px; }
.left, .right { display: flex; flex-direction: column; gap: 6px; min-height: 0; }
.toolbar { display: flex; align-items: center; gap: 6px; }
.chips { display: flex; gap: 4px; }
.chip { padding: 2px 9px; border-radius: 10px; font-size: 11px; background: rgba(255,255,255,.06); color: #c9c9cf; }
.chip.on { background: var(--accent); color: #111; font-weight: 700; }
.grid { flex: 1; min-height: 0; overflow-y: auto; display: grid; grid-template-columns: repeat(3, 1fr); grid-auto-rows: 66px; gap: 6px; align-content: start; }
.game { position: relative; height: 66px; border-radius: 10px; overflow: hidden; background: #1a1a1f; }
.game img, .ph { position: absolute; inset: 0; width: 100%; height: 100%; object-fit: cover; }
.ph { display: grid; place-items: center; font-weight: 800; font-size: 20px; color: rgba(255,255,255,.75); letter-spacing: .05em; }
.gname { position: absolute; left: 0; right: 0; bottom: 0; padding: 10px 6px 3px; font-size: 10.5px; font-weight: 600; text-align: left;
  background: linear-gradient(transparent, rgba(0,0,0,.85)); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.src { position: absolute; top: 4px; left: 4px; font: 700 8px var(--mono); padding: 1px 4px; border-radius: 4px; background: rgba(0,0,0,.6); letter-spacing: .06em; }
.src.epic { color: #ddd; } .src.steam { color: #66c0f4; } .src.gog { color: #c38aff; }
.playbtn { position: absolute; inset: 0; display: grid; place-items: center; font-size: 20px; background: rgba(0,0,0,.45); opacity: 0; transition: opacity .15s; }
.game:hover .playbtn { opacity: 1; }
.game:hover { box-shadow: 0 0 0 2px var(--accent); }
.friends { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
.friend { display: flex; align-items: center; gap: 7px; padding: 4px 6px; border-radius: 10px; background: rgba(255,255,255,.04); }
.friend.ingame { background: rgba(144, 186, 60, .12); }
.friend.away { opacity: .6; }
.av { width: 28px; height: 28px; border-radius: 6px; flex: none; object-fit: cover; }
.ph2 { display: grid; place-items: center; background: #2a475e; font-size: 10px; font-weight: 700; }
.ingame .av { box-shadow: 0 0 0 1.5px #90ba3c; }
.fi { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.fn { font-size: 12px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.fs { font-size: 10.5px; color: var(--mute); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.ingame .fs { color: #a4d007; }
.join { padding: 3px 9px; font-size: 11px; }
.cfg { display: flex; flex-direction: column; gap: 5px; overflow-y: auto; }
.tiny { font-size: 10.5px; }
.empty.small { padding: 12px 4px; font-size: 12px; }
a { color: var(--accent-2); cursor: pointer; text-decoration: underline; }
code { font-family: var(--mono); color: var(--accent-2); }
</style>
