// Jogos instalados e amigos da Steam
import { reactive } from 'vue'
import { api } from '../lib/api'
import type { Friend, Game, SteamInfo } from '../lib/types'
import { S } from './settings'
import { showToast } from './state'

export const games = reactive({
  list: [] as Game[],
  loading: false,
  steam: null as SteamInfo | null,
  friends: [] as Friend[],
  friendsError: '',
  friendsLoading: false
})

export async function loadGames() {
  games.loading = true
  try { games.list = await api.gamesList() } finally { games.loading = false }
}

const playing: Record<string, string> = {}
let firstFriends = true

export async function loadFriends() {
  if (!games.steam?.has_key) return
  games.friendsLoading = true
  try {
    const list = await api.steamFriends(S.steamId || games.steam.steamid)
    games.friendsError = ''
    // aviso: amigo começou a jogar algo que você tem instalado
    if (!firstFriends && S.friendAlerts) {
      for (const f of list) {
        if (f.game && playing[f.steamid] !== f.game && games.list.some(g => g.appid && String(g.appid) === f.gameid)) {
          showToast({ icon: '🎮', img: f.avatar || undefined, text: `${f.name} começou a jogar ${f.game}`, ms: 5000 })
        }
      }
    }
    for (const k of Object.keys(playing)) delete playing[k]
    for (const f of list) playing[f.steamid] = f.game
    firstFriends = false
    games.friends = list
  } catch (e) {
    games.friendsError = String(e)
  } finally {
    games.friendsLoading = false
  }
}

export async function refreshSteam() {
  games.steam = await api.steamInfo()
}

export async function initGames() {
  await Promise.all([loadGames(), refreshSteam()])
  loadFriends()
  setInterval(loadFriends, 60_000)
}

/** Tem o jogo do amigo instalado? */
export const ownedGame = (f: Friend) => games.list.find(g => g.appid && String(g.appid) === f.gameid)

/** Entrar na partida do amigo: lobby direto quando a Steam informa, senão "Entrar no jogo" do Steam */
export function joinUrl(f: Friend) {
  return f.lobby ? `steam://joinlobby/${f.gameid}/${f.lobby}/${f.steamid}` : `steam://friends/joingame/${f.steamid}`
}

export const STATE: Record<number, string> = { 0: 'Offline', 1: 'Online', 2: 'Ocupado', 3: 'Ausente', 4: 'Soneca', 5: 'Quer trocar', 6: 'Quer jogar' }
