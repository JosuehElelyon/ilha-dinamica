<script setup lang="ts">
// Trabalho: um cartão por órgão com VPN (liga/desliga) e Merge Requests do GitLab
import { computed, reactive, ref } from 'vue'
import { api } from '../../lib/api'
import type { Mr, Org, OrgRepo } from '../../lib/types'
import { ACCENTS, S, save, newOrg } from '../../store/settings'
import { showToast } from '../../store/state'
import { VPN_TYPES, work, st, toggleVpn, loadMrs, refreshSecrets, adapterKey, pipeIcon } from '../../store/work'

type View = { kind: 'cards' } | { kind: 'config'; id: string } | { kind: 'mrs'; id: string } | { kind: 'repos'; id: string }
const view = ref<View>({ kind: 'cards' })
const org = computed(() => (view.value.kind === 'cards' ? null : S.orgs.find(o => o.id === (view.value as { id: string }).id) || null))

/* ---------- configuração ---------- */
const form = reactive({ org: null as Org | null, token: '', password: '', testing: false, testMsg: '', showAdapters: false })
const typeInfo = computed(() => VPN_TYPES.find(t => t.id === form.org?.vpn.type) || VPN_TYPES[0])

function openConfig(o: Org) {
  const clone: Org = structuredClone(JSON.parse(JSON.stringify(o)))
  if (!Array.isArray(clone.repos)) clone.repos = []
  form.org = clone
  form.token = ''; form.password = ''; form.testMsg = ''; form.showAdapters = false
  view.value = { kind: 'config', id: o.id }
}
async function addRepo() {
  if (!form.org) return
  const path = await api.pickPath('folder')
  if (!path) return
  const name = path.split(/[\\/]/).filter(Boolean).pop() || 'Projeto'
  form.org.repos = [...(form.org.repos || []), { name, path }]
}
function removeRepo(idx: number) {
  if (!form.org?.repos) return
  form.org.repos = form.org.repos.filter((_, i) => i !== idx)
}
async function openRepo(r: OrgRepo) {
  const ok = await api.openInAntigravity(r.path)
  showToast(ok
    ? { icon: '🚀', text: `Abrindo ${r.name} no Antigravity…`, ms: 2200 }
    : { icon: '⚠️', text: 'Antigravity não encontrado — verifique a instalação', alert: true, ms: 3500 })
}
function openReposBtn(o: Org) {
  const list = o.repos || []
  if (list.length === 1) return openRepo(list[0])
  if (list.length > 1) { view.value = { kind: 'repos', id: o.id }; return }
  openConfig(o)
}
async function saveConfig() {
  const o = form.org!
  save('orgs', S.orgs.map(x => (x.id === o.id ? o : x)))
  if (form.token) await api.gitlabSetToken(o.id, form.token)
  if (form.password) await api.vpnSetPassword(o.id, form.password)
  await refreshSecrets()
  showToast({ icon: '✅', text: `${o.name} salvo`, ms: 1800 })
  view.value = { kind: 'cards' }
  loadMrs(S.orgs.find(x => x.id === o.id)!)
}
async function test() {
  const o = form.org!
  if (form.token) { await api.gitlabSetToken(o.id, form.token); await refreshSecrets(); form.token = '' }
  form.testing = true; form.testMsg = 'Testando…'
  try {
    const r = await api.gitlabMrs(o.id, o.gitlab.url, o.gitlab.insecure)
    form.testMsg = `✅ Conectado como @${r.user} · ${r.review.length} p/ revisar · ${r.mine.length} seus`
  } catch (e) { form.testMsg = `❌ ${e}` }
  form.testing = false
}
async function forget(kind: 'token' | 'password') {
  const o = form.org!
  if (kind === 'token') await api.gitlabSetToken(o.id, '')
  else await api.vpnSetPassword(o.id, '')
  await refreshSecrets()
}
function removeOrg() {
  const o = form.org!
  save('orgs', S.orgs.filter(x => x.id !== o.id))
  api.gitlabSetToken(o.id, ''); api.vpnSetPassword(o.id, '')
  view.value = { kind: 'cards' }
}
function addOrg() {
  const id = 'org' + Date.now().toString(36)
  const o = newOrg(id, 'Novo', ACCENTS[S.orgs.length % ACCENTS.length])
  save('orgs', [...S.orgs, o])
  openConfig(o)
}

/* ---------- cartões ---------- */
function mrSummary(o: Org) {
  const s = st(o.id)
  if (!o.gitlab.url) return { text: 'GitLab não configurado', warn: false }
  if (!s.hasToken) return { text: 'Falta o token do GitLab', warn: true }
  if (s.mrError) return { text: s.mrError, warn: true }
  if (!s.mrs) return { text: s.mrLoading ? 'Carregando…' : '—', warn: false }
  const failed = s.mrs.mine.filter(m => m.pipeline === 'failed').length
  return {
    text: `${s.mrs.review.length} p/ revisar · ${s.mrs.mine.length} meus${failed ? ` · ${failed} ❌` : ''}`,
    warn: false, hot: s.mrs.review.length > 0 || failed > 0
  }
}
const openUrl = (u: string) => api.runAction(u)
const gitlabHome = (o: Org) => (o.gitlab.url.startsWith('http') ? o.gitlab.url : 'https://' + o.gitlab.url)
const ago = (iso: string) => {
  const m = Math.round((Date.now() - new Date(iso).getTime()) / 60000)
  return m < 60 ? `${m} min` : m < 1440 ? `${Math.round(m / 60)} h` : `${Math.round(m / 1440)} d`
}
const mrClass = (m: Mr) => ({ draft: m.draft })
</script>

<template>
  <!-- ================= CARTÕES ================= -->
  <div v-if="view.kind === 'cards'" class="work">
    <div class="top">
      <span class="set-label">ÓRGÃOS</span>
      <label class="chk" title="Ao ligar uma VPN, desliga as outras"><input type="checkbox" :checked="S.vpnExclusive"
             @change="save('vpnExclusive', ($event.target as HTMLInputElement).checked)"> Uma VPN por vez</label>
      <button class="pillbtn" @click="addOrg">+ Órgão</button>
    </div>
    <div class="cards" :style="{ gridTemplateColumns: `repeat(${Math.min(Math.max(S.orgs.length, 1), 4)}, 1fr)` }">
      <div v-for="o in S.orgs" :key="o.id" class="card" :style="{ '--c': o.color }">
        <div class="head">
          <span class="name">{{ o.name }}</span>
          <button class="icon-btn gear" title="Configurar" @click="openConfig(o)">⚙</button>
        </div>

        <div class="vpn">
          <template v-if="o.vpn.type !== 'none'">
            <span class="dot" :class="{ on: st(o.id).vpnUp }" />
            <span class="vtxt">
              <b>{{ st(o.id).vpnUp ? 'VPN conectada' : 'VPN desligada' }}</b>
              <span class="mute mono small">{{ st(o.id).vpnUp ? st(o.id).ip : VPN_TYPES.find(t => t.id === o.vpn.type)?.label }}</span>
            </span>
            <button class="switch" :class="{ on: st(o.id).vpnUp, busy: st(o.id).busy }" :disabled="st(o.id).busy"
                    :title="st(o.id).vpnUp ? 'Desligar VPN' : 'Ligar VPN'" @click="toggleVpn(o)" />
          </template>
          <span v-else class="mute small">Sem VPN · <a @click="openConfig(o)">configurar</a></span>
        </div>

        <div class="gl" :class="{ warn: mrSummary(o).warn, hot: mrSummary(o).hot }" @click="o.gitlab.url && st(o.id).hasToken ? (view = { kind: 'mrs', id: o.id }) : openConfig(o)">
          <span>🔀</span><span class="gtxt">{{ mrSummary(o).text }}</span>
        </div>

        <div class="actions">
          <button class="pillbtn" :disabled="!st(o.id).mrs" @click="view = { kind: 'mrs', id: o.id }">MRs</button>
          <button class="pillbtn" :title="(o.repos?.length || 0) === 1 ? `Abrir ${o.repos![0].name} no Antigravity` : (o.repos?.length ? `Repositórios (${o.repos.length})` : 'Configurar repositórios no Antigravity')" @click="openReposBtn(o)">🚀{{ (o.repos?.length || 0) > 1 ? ` ${o.repos!.length}` : '' }}</button>
          <button class="pillbtn" :disabled="!o.gitlab.url" title="Abrir o GitLab no navegador" @click="openUrl(gitlabHome(o))">GitLab ↗</button>
          <button class="pillbtn" :disabled="!st(o.id).hasToken || st(o.id).mrLoading" title="Atualizar" @click="loadMrs(o, true)">↻</button>
        </div>
      </div>
      <div v-if="!S.orgs.length" class="empty">Nenhum órgão. Clique em “+ Órgão”.</div>
    </div>
  </div>

  <!-- ================= REPOSITÓRIOS (ANTIGRAVITY) ================= -->
  <div v-else-if="view.kind === 'repos' && org" class="work" :style="{ '--c': org.color }">
    <div class="top">
      <button class="pillbtn" @click="view = { kind: 'cards' }">‹ Voltar</button>
      <b class="ttl">{{ org.name }} · Repositórios</b>
      <button class="pillbtn" @click="openConfig(org)">⚙ Configurar</button>
    </div>
    <div class="repolist">
      <div v-for="(r, idx) in (org.repos || [])" :key="idx" class="repocard">
        <div class="repometa">
          <b>{{ r.name }}</b>
          <span class="mute mono tiny">{{ r.path }}</span>
        </div>
        <button class="pillbtn" title="Abrir pasta no Explorer" @click="openUrl(r.path)">📂</button>
        <button class="pillbtn primary" @click="openRepo(r)">🚀 Antigravity</button>
      </div>
      <div v-if="!org.repos?.length" class="empty small">Nenhum repositório configurado.</div>
    </div>
  </div>

  <!-- ================= MERGE REQUESTS ================= -->
  <div v-else-if="view.kind === 'mrs' && org" class="work" :style="{ '--c': org.color }">
    <div class="top">
      <button class="pillbtn" @click="view = { kind: 'cards' }">‹ Voltar</button>
      <b class="ttl">{{ org.name }} · Merge Requests</b>
      <span class="mute small">{{ st(org.id).mrs?.user ? '@' + st(org.id).mrs?.user : '' }}</span>
      <button class="pillbtn" :disabled="st(org.id).mrLoading" @click="loadMrs(org, true)">{{ st(org.id).mrLoading ? '…' : '↻' }}</button>
    </div>
    <div v-if="st(org.id).mrError" class="empty">{{ st(org.id).mrError }}</div>
    <div v-else class="mrcols">
      <div class="mrcol">
        <span class="set-label">PARA REVISAR ({{ st(org.id).mrs?.review.length ?? 0 }})</span>
        <div class="mrlist">
          <div v-if="!st(org.id).mrs?.review.length" class="empty small">Nada para revisar 🎉</div>
          <button v-for="m in st(org.id).mrs?.review" :key="m.id" class="mr" :class="mrClass(m)" :title="m.url" @click="openUrl(m.url)">
            <span class="mt">{{ m.draft ? '[Draft] ' : '' }}{{ m.title }}</span>
            <span class="mm">{{ m.project }} !{{ m.iid }} · {{ m.author }} · {{ ago(m.updated) }}</span>
          </button>
        </div>
      </div>
      <div class="mrcol">
        <span class="set-label">MEUS ({{ st(org.id).mrs?.mine.length ?? 0 }})</span>
        <div class="mrlist">
          <div v-if="!st(org.id).mrs?.mine.length" class="empty small">Nenhum MR aberto</div>
          <button v-for="m in st(org.id).mrs?.mine" :key="m.id" class="mr" :class="mrClass(m)" :title="m.url" @click="openUrl(m.url)">
            <span class="mt"><span v-if="m.pipeline" :title="'Pipeline: ' + m.pipeline">{{ pipeIcon(m.pipeline) }} </span>{{ m.draft ? '[Draft] ' : '' }}{{ m.title }}</span>
            <span class="mm">{{ m.project }} !{{ m.iid }} · {{ ago(m.updated) }}<b v-if="m.conflicts" class="conf"> · conflito</b></span>
          </button>
        </div>
      </div>
    </div>
  </div>

  <!-- ================= CONFIGURAÇÃO ================= -->
  <form v-else-if="view.kind === 'config' && form.org" class="work cfg" autocomplete="off" @submit.prevent="saveConfig">
    <div class="top">
      <button type="button" class="pillbtn" @click="view = { kind: 'cards' }">‹ Voltar</button>
      <input v-model="form.org.name" class="in oname" maxlength="16" required>
      <div class="sws">
        <button v-for="c in ACCENTS" :key="c" type="button" class="sw" :class="{ on: form.org.color === c }" :style="{ background: c }" @click="form.org.color = c" />
      </div>
      <span class="grow" />
      <button type="button" class="pillbtn danger" @click="removeOrg">Remover</button>
      <button class="pillbtn primary">Salvar</button>
    </div>
    <div class="cfgcols">
      <div class="sec">
        <span class="set-label">🔐 VPN</span>
        <select v-model="form.org.vpn.type" class="in">
          <option v-for="t in VPN_TYPES" :key="t.id" :value="t.id">{{ t.label }}</option>
        </select>
        <template v-if="form.org.vpn.type === 'custom'">
          <input v-model="form.org.vpn.connect" class="in" placeholder="Comando para ligar">
          <input v-model="form.org.vpn.disconnect" class="in" placeholder="Comando para desligar">
        </template>
        <input v-else-if="typeInfo.nameLabel" v-model="form.org.vpn.name" class="in" :placeholder="typeInfo.nameLabel + ' — ' + typeInfo.namePh" :title="typeInfo.nameLabel">
        <template v-if="form.org.vpn.type === 'cisco'">
          <div class="row2">
            <input v-model="form.org.vpn.user" class="in" placeholder="Usuário (ex.: dominio\usuario)">
            <input v-model="form.password" type="password" class="in"
                   :placeholder="st(form.org.id).hasPassword ? '•••••• (salva)' : 'Senha (opcional)'">
          </div>
          <span class="mute tiny">Com a senha salva (no Gerenciador de Credenciais do Windows) a ilha conecta sozinha; sem ela, abre o Cisco.
            <a v-if="st(form.org.id).hasPassword" @click="forget('password')">Esquecer senha</a></span>
        </template>
        <div v-if="form.org.vpn.type !== 'none'" class="row2">
          <input v-model="form.org.vpn.adapter" class="in" :placeholder="'Placa de rede (padrão: ' + (adapterKey({ ...form.org, vpn: { ...form.org.vpn, adapter: '' } }) || '—') + ')'"
                 title="Parte do nome da placa de rede que aparece quando a VPN está ligada. Use | para alternativas.">
          <button type="button" class="pillbtn" @click="form.showAdapters = !form.showAdapters">Detectar</button>
        </div>
        <div v-if="form.showAdapters" class="adapters">
          <span class="mute tiny">Ligue a VPN e escolha a placa que apareceu:</span>
          <button v-for="a in work.adapters" :key="a.name" type="button" class="ad" :class="{ up: a.up }"
                  @click="form.org.vpn.adapter = a.desc || a.name; form.showAdapters = false">
            <span class="dot" :class="{ on: a.up }" /> {{ a.name }} <span class="mute">· {{ a.desc }}</span>
          </button>
        </div>
      </div>
      <div class="sec">
        <span class="set-label">🦊 GITLAB</span>
        <input v-model="form.org.gitlab.url" class="in" placeholder="Endereço (ex.: gitlab.orgao.gov.br)">
        <input v-model="form.token" type="password" class="in"
               :placeholder="st(form.org.id).hasToken ? '•••••• token salvo (digite para trocar)' : 'Token de acesso pessoal (escopo read_api)'">
        <label class="chk tiny"><input v-model="form.org.gitlab.insecure" type="checkbox"> Aceitar certificado interno do órgão</label>
        <div class="row2">
          <button type="button" class="pillbtn" :disabled="!form.org.gitlab.url || form.testing" @click="test">Testar conexão</button>
          <a v-if="st(form.org.id).hasToken" class="tiny" @click="forget('token')">Esquecer token</a>
        </div>
        <span class="tiny" :class="form.testMsg.startsWith('❌') ? 'danger' : 'mute'">{{ form.testMsg }}</span>
      </div>
      <div class="sec span2">
        <div class="row2">
          <span class="set-label">🚀 REPOSITÓRIOS (ANTIGRAVITY)</span>
          <button type="button" class="pillbtn" @click="addRepo">+ Pasta</button>
        </div>
        <div v-for="(r, idx) in (form.org.repos || [])" :key="idx" class="row2">
          <input v-model="r.name" class="in rname" placeholder="Nome" maxlength="32">
          <input v-model="r.path" class="in mono" placeholder="C:\projetos\meu-repo">
          <button type="button" class="pillbtn danger" title="Remover" @click="removeRepo(idx)">✕</button>
        </div>
        <span v-if="!form.org.repos?.length" class="mute tiny">Adicione pastas locais para abrir direto no Antigravity pelo cartão do órgão.</span>
      </div>
    </div>
  </form>
</template>

<style scoped>
.work { display: flex; flex-direction: column; gap: 6px; padding: 0 6px 6px; }
.top { display: flex; align-items: center; gap: 8px; }
.top .set-label { margin-right: auto; }
.ttl { margin-right: auto; font-size: 13px; }
.cards { flex: 1; min-height: 0; display: grid; gap: 8px; }
.card { display: flex; flex-direction: column; gap: 8px; padding: 10px 12px; border-radius: 16px; min-width: 0;
  background: linear-gradient(180deg, color-mix(in srgb, var(--c), transparent 82%), rgba(255,255,255,.03) 60%);
  box-shadow: inset 0 2px 0 var(--c); }
.head { display: flex; align-items: center; justify-content: space-between; }
.name { font-weight: 800; font-size: 15px; letter-spacing: .02em; color: var(--c); }
.gear { opacity: .7; } .gear:hover { opacity: 1; }
.vpn { display: flex; align-items: center; gap: 8px; min-height: 30px; }
.vtxt { display: flex; flex-direction: column; flex: 1; min-width: 0; font-size: 12px; }
.vtxt .mono { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.dot { width: 9px; height: 9px; border-radius: 50%; background: #55555e; flex: none; }
.dot.on { background: #34d058; box-shadow: 0 0 8px #34d058; }
.switch.on { background: #34d058; }
.switch.busy { opacity: .6; animation: pulse 1s infinite; }
@keyframes pulse { 50% { opacity: .3 } }
.gl { display: flex; gap: 6px; align-items: center; padding: 6px 8px; border-radius: 10px; background: rgba(0,0,0,.2); font-size: 12px; cursor: pointer; min-height: 30px; }
.gl:hover { background: rgba(0,0,0,.32); }
.gl.warn { color: #ffb070; }
.gl.hot .gtxt { color: #fff; font-weight: 600; }
.gtxt { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.actions { display: flex; gap: 4px; margin-top: auto; }
.actions .pillbtn { padding: 4px 9px; font-size: 11px; }
a { color: var(--accent-2); cursor: pointer; text-decoration: underline; }
.tiny { font-size: 10.5px; }

.mrcols { flex: 1; min-height: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 10px; }
.mrcol { display: flex; flex-direction: column; gap: 4px; min-height: 0; }
.mrlist { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 3px; }
.mr { display: flex; flex-direction: column; gap: 1px; padding: 6px 9px; border-radius: 10px; background: rgba(255,255,255,.04); text-align: left;
  border-left: 2px solid var(--c); }
.mr:hover { background: rgba(255,255,255,.09); }
.mr.draft { opacity: .6; }
.mt { font-size: 12.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.mm { font-size: 10.5px; color: var(--mute); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.conf { color: #ff8a7e; }
.empty.small { padding: 10px 4px; font-size: 12px; }

.cfg .oname { width: 110px; font-weight: 700; }
.sws { display: flex; gap: 5px; }
.sw { width: 16px; height: 16px; border-radius: 50%; }
.sw.on { box-shadow: 0 0 0 2px #0e0e11, 0 0 0 3px #fff; }
.cfgcols { flex: 1; min-height: 0; display: grid; grid-template-columns: 1fr 1fr; gap: 14px; overflow-y: auto; }
.sec { display: flex; flex-direction: column; gap: 5px; min-width: 0; }
.row2 { display: flex; gap: 6px; align-items: center; }
.row2 .in { flex: 1; min-width: 0; }
.adapters { display: flex; flex-direction: column; gap: 2px; max-height: 90px; overflow-y: auto; padding: 4px; border-radius: 8px; background: rgba(0,0,0,.25); }
.ad { display: flex; align-items: center; gap: 6px; padding: 3px 6px; border-radius: 6px; font-size: 11px; text-align: left; white-space: nowrap; overflow: hidden; }
.ad:hover { background: rgba(255,255,255,.08); }
.span2 { grid-column: 1 / -1; }
.rname { max-width: 150px; flex: 0 0 150px !important; }
.repolist { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 6px; }
.repocard { display: flex; align-items: center; gap: 8px; padding: 8px 10px; border-radius: 12px; background: rgba(255,255,255,.04); border-left: 2px solid var(--c); }
.repometa { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
.repometa b { font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.repometa span { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
</style>
