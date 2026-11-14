<script setup lang="ts">
// Central do Assistente Pessoal: Resumo/Datas, Contas Conectadas e Ponte Multi-Transporte com Celular.
import { computed, onMounted, ref, watch } from 'vue'
import QRCode from 'qrcode'
import { api } from '../../lib/api'
import { pad } from '../../lib/format'
import { S, save } from '../../store/settings'
import {
  addImportantDate, assistant, connectApple, connectGitHub, connectGoogle,
  deleteImportantDate, disconnectAccount, enableUsbBridge, primaryPhone,
  refreshAssistant, rotatePairingToken, syncGoogle, syncWindowsCalendar,
  triggerTestPhoneSignal
} from '../../store/assistant'
import { island } from '../../store/state'

const qrDataUrl = ref('')
const phoneMode = ref<'lan' | 'usb' | 'bluetooth'>('lan')

// Formulário rápido para adicionar aniversário/data comemorativa
const showAddDate = ref(false)
const dateName = ref('')
const dateKind = ref<'birthday' | 'anniversary' | 'custom'>('birthday')
const dateMd = ref('')

// Formulários das contas
const openCfg = ref<'' | 'google' | 'github' | 'apple'>('')
const gClientId = ref(S.googleClientId || '')
const gSecret = ref('')
const ghToken = ref('')
const appleId = ref('')
const applePass = ref('')

const googleAcc = computed(() => assistant.snap.accounts.find(a => a.provider === 'google'))
const winAcc = computed(() => assistant.snap.accounts.find(a => a.provider === 'windows'))
const ghAcc = computed(() => assistant.snap.accounts.find(a => a.provider === 'github'))
const appleAcc = computed(() => assistant.snap.accounts.find(a => a.provider === 'apple'))
const gitlabCount = computed(() => S.orgs.filter(o => o.gitlab.url).length)

async function buildQr() {
  const uri = assistant.pairing?.qrUri
  if (!uri) { qrDataUrl.value = ''; return }
  try {
    qrDataUrl.value = await QRCode.toDataURL(uri, {
      width: 112,
      margin: 1,
      color: { dark: '#111116', light: '#ffffff' }
    })
  } catch {
    qrDataUrl.value = ''
  }
}

watch(() => assistant.pairing?.qrUri, buildQr, { immediate: true })
onMounted(() => { refreshAssistant() })

async function submitDate() {
  const [m, d] = dateMd.value.split('/').map(Number)
  if (!dateName.value.trim() || !m || !d) return
  await addImportantDate(dateName.value.trim(), dateKind.value, m, d, null)
  dateName.value = ''
  dateMd.value = ''
  showAddDate.value = false
}

async function doGoogleConnect() {
  save('googleClientId', gClientId.value.trim())
  await connectGoogle(gClientId.value.trim(), gSecret.value.trim() || undefined)
  if (assistant.snap.googleConnected) openCfg.value = ''
}

async function doGithubConnect() {
  await connectGitHub(ghToken.value.trim())
  if (!assistant.error) { ghToken.value = ''; openCfg.value = '' }
}

async function doAppleConnect() {
  await connectApple(appleId.value.trim(), applePass.value.trim())
  if (!assistant.error) { applePass.value = ''; openCfg.value = '' }
}

function daysBadge(n: number) {
  if (n === 0) return 'HOJE'
  if (n === 1) return 'AMANHÃ'
  return `em ${n}d`
}
</script>

<template>
  <div class="assist-wrap">
    <!-- Barra superior de navegação interna + status rápidos -->
    <div class="topbar">
      <div class="seg tabs-seg">
        <button :class="{ on: assistant.subTab === 'hoje' }" @click="assistant.subTab = 'hoje'">✨ Hoje</button>
        <button :class="{ on: assistant.subTab === 'contas' }" @click="assistant.subTab = 'contas'">
          🔐 Contas
          <span class="dot-s" :class="assistant.snap.googleConnected ? 'ok' : 'warn'" />
        </button>
        <button :class="{ on: assistant.subTab === 'celular' }" @click="assistant.subTab = 'celular'">
          📱 Celular
          <span v-if="primaryPhone?.batteryPct != null" class="bat-tag">{{ primaryPhone.batteryPct }}%</span>
        </button>
      </div>

      <div class="status-chips">
        <span class="chip" :class="assistant.snap.googleConnected ? 'ok' : 'warn'">
          {{ assistant.snap.googleConnected ? '● Google ativo' : '⚠ Google pendente' }}
        </span>
        <span class="chip muted">{{ assistant.snap.peopleCount }} contatos</span>
        <button class="pillbtn tiny" :disabled="assistant.syncing" title="Sincronizar fontes" @click="syncWindowsCalendar(); if (assistant.snap.googleConnected) syncGoogle()">
          {{ assistant.syncing ? '…' : '↻ Sincronizar' }}
        </button>
      </div>
    </div>

    <div v-if="assistant.error" class="err-bar">{{ assistant.error }}</div>

    <!-- ================= ABA HOJE ================= -->
    <div v-if="assistant.subTab === 'hoje'" class="hoje-grid">
      <div v-if="!assistant.snap.googleConnected && !assistant.onboardingDismissed" class="google-banner">
        <div>
          <strong>Conecte sua conta Google como base do Assistente</strong>
          <span class="mute small"> — libera aniversários dos contatos, agenda e lembretes inteligentes.</span>
        </div>
        <div class="banner-acts">
          <button class="pillbtn primary tiny" @click="assistant.subTab = 'contas'; openCfg = 'google'">Conectar Google</button>
          <button class="pillbtn tiny" @click="assistant.onboardingDismissed = true">Depois</button>
        </div>
      </div>

      <div class="cols">
        <!-- Coluna 1: Aniversários & Datas Comemorativas -->
        <section class="panel-box">
          <div class="box-head">
            <span class="set-label">DATAS & ANIVERSÁRIOS</span>
            <button class="pillbtn tiny" @click="showAddDate = !showAddDate">{{ showAddDate ? '✕' : '+ Data' }}</button>
          </div>

          <form v-if="showAddDate" class="add-date" @submit.prevent="submitDate">
            <input v-model="dateName" type="text" class="in tin grow" placeholder="Nome / evento">
            <input v-model="dateMd" type="text" class="in tin md-in" placeholder="MM/DD" maxlength="5">
            <select v-model="dateKind" class="in tin">
              <option value="birthday">Aniversário</option>
              <option value="anniversary">Especial</option>
              <option value="custom">Lembrete</option>
            </select>
            <button type="submit" class="pillbtn primary tiny">Salvar</button>
          </form>

          <div class="scroll-list">
            <div v-for="d in assistant.snap.upcomingDates" :key="d.id" class="row-item" :class="{ today: d.daysUntil === 0 }">
              <span class="ico-badge">{{ d.kind === 'birthday' ? '🎂' : '🎉' }}</span>
              <div class="row-txt">
                <div class="row-title">
                  {{ d.title }}
                  <span v-if="d.turningAge" class="mute">({{ d.turningAge }}a)</span>
                </div>
                <div class="row-sub mute">{{ pad(d.day) }}/{{ pad(d.month) }} · {{ d.sourceProvider }}</div>
              </div>
              <span class="when-pill" :class="{ hot: d.daysUntil === 0 }">{{ daysBadge(d.daysUntil) }}</span>
              <button v-if="d.id.startsWith('custom-')" class="del-x" title="Remover" @click="deleteImportantDate(d.id)">✕</button>
            </div>
            <div v-if="!assistant.snap.upcomingDates.length" class="empty-sm">Sem datas nos próximos 30 dias</div>
          </div>
        </section>

        <!-- Coluna 2: Agenda Unificada & Pendências Dev -->
        <section class="panel-box">
          <div class="box-head">
            <span class="set-label">AGENDA HOJE & PENDÊNCIAS</span>
            <span class="mute small">{{ assistant.snap.todayEvents.length }} eventos · {{ assistant.snap.devItems.length }} dev</span>
          </div>

          <div class="scroll-list">
            <div v-for="ev in assistant.snap.todayEvents" :key="ev.id" class="row-item">
              <span class="time-pill mono">{{ ev.allDay ? 'DIA' : ev.startTime }}</span>
              <div class="row-txt">
                <div class="row-title">{{ ev.title }}</div>
                <div class="row-sub mute">{{ ev.sourceProvider }} {{ ev.endTime ? '· até ' + ev.endTime : '' }}</div>
              </div>
              <button v-if="ev.joinUrl" class="pillbtn primary tiny" @click="api.runAction(ev.joinUrl)">Entrar ↗</button>
            </div>

            <div v-for="dv in assistant.snap.devItems" :key="dv.id" class="row-item dev">
              <span class="ico-badge">🐙</span>
              <div class="row-txt">
                <div class="row-title">{{ dv.title }}</div>
                <div class="row-sub mute">{{ dv.repo }} · review pendente</div>
              </div>
              <button class="pillbtn tiny" @click="api.runAction(dv.url)">Abrir ↗</button>
            </div>

            <div v-if="!assistant.snap.todayEvents.length && !assistant.snap.devItems.length" class="empty-sm">
              Nenhum evento ou pendência para hoje
            </div>
          </div>
        </section>
      </div>
    </div>

    <!-- ================= ABA CONTAS ================= -->
    <div v-else-if="assistant.subTab === 'contas'" class="contas-grid">
      <!-- Google (Obrigatória / Principal) -->
      <div class="acc-card primary-card">
        <div class="acc-head">
          <span class="acc-name">🔵 Google <b class="req-tag">Principal</b></span>
          <span class="chip" :class="googleAcc ? 'ok' : 'warn'">{{ googleAcc ? 'Conectada' : 'Obrigatória' }}</span>
        </div>
        <div class="acc-desc mute">
          {{ googleAcc ? `${googleAcc.email} · ${googleAcc.details}` : 'Agenda, contatos, aniversários e perfil via OAuth PKCE.' }}
        </div>
        <div v-if="openCfg === 'google'" class="acc-form">
          <input v-model="gClientId" type="text" class="in tin" placeholder="Google OAuth Desktop Client ID">
          <input v-model="gSecret" type="password" class="in tin" placeholder="Client Secret (opcional)">
          <div class="form-btns">
            <button class="pillbtn primary tiny" :disabled="!gClientId.trim() || assistant.syncing" @click="doGoogleConnect">Autorizar no Navegador</button>
            <button class="pillbtn tiny" @click="openCfg = ''">Cancelar</button>
          </div>
        </div>
        <div v-else class="acc-acts">
          <button v-if="!googleAcc" class="pillbtn primary tiny" @click="openCfg = 'google'">Conectar Google</button>
          <template v-else>
            <button class="pillbtn tiny" :disabled="assistant.syncing" @click="syncGoogle">↻ Sincronizar</button>
            <button class="pillbtn tiny danger" @click="disconnectAccount(googleAcc.id)">Desconectar</button>
          </template>
        </div>
      </div>

      <!-- Windows Calendário Nativo -->
      <div class="acc-card">
        <div class="acc-head">
          <span class="acc-name">🪟 Calendário do Windows</span>
          <span class="chip" :class="winAcc ? 'ok' : 'muted'">{{ winAcc ? 'Ativo' : 'Pronto' }}</span>
        </div>
        <div class="acc-desc mute">
          {{ winAcc ? winAcc.details : 'Lê compromissos locais do Windows (Outlook/Microsoft) via WinRT sem login.' }}
        </div>
        <div class="acc-acts">
          <button class="pillbtn tiny" :disabled="assistant.syncing" @click="syncWindowsCalendar">↻ Ler Calendário Local</button>
        </div>
      </div>

      <!-- Apple iCloud -->
      <div class="acc-card">
        <div class="acc-head">
          <span class="acc-name">🍎 Apple iCloud</span>
          <span class="chip" :class="appleAcc ? 'ok' : 'muted'">{{ appleAcc ? 'Conectada' : 'Opcional' }}</span>
        </div>
        <div class="acc-desc mute">
          {{ appleAcc ? `${appleAcc.email} · ${appleAcc.details}` : 'CalDAV + CardDAV com senha específica de app no cofre Windows.' }}
        </div>
        <div v-if="openCfg === 'apple'" class="acc-form">
          <input v-model="appleId" type="text" class="in tin" placeholder="seu@icloud.com">
          <input v-model="applePass" type="password" class="in tin" placeholder="Senha específica de app">
          <div class="form-btns">
            <button class="pillbtn primary tiny" :disabled="!appleId.trim() || !applePass.trim() || assistant.syncing" @click="doAppleConnect">Conectar</button>
            <button class="pillbtn tiny" @click="openCfg = ''">Cancelar</button>
          </div>
        </div>
        <div v-else class="acc-acts">
          <button v-if="!appleAcc" class="pillbtn tiny" @click="openCfg = 'apple'">Configurar iCloud</button>
          <button v-else class="pillbtn tiny danger" @click="disconnectAccount(appleAcc.id)">Desconectar</button>
        </div>
      </div>

      <!-- GitHub + GitLab -->
      <div class="acc-card">
        <div class="acc-head">
          <span class="acc-name">🐙 GitHub & 🦊 GitLab</span>
          <span class="chip" :class="ghAcc || gitlabCount ? 'ok' : 'muted'">{{ ghAcc ? ghAcc.label : `${gitlabCount} orgs GitLab` }}</span>
        </div>
        <div class="acc-desc mute">
          {{ ghAcc ? ghAcc.details : 'PRs para revisar, notificações e MRs das organizações do Trabalho.' }}
        </div>
        <div v-if="openCfg === 'github'" class="acc-form">
          <input v-model="ghToken" type="password" class="in tin" placeholder="Token GitHub (repo, notifications)">
          <div class="form-btns">
            <button class="pillbtn primary tiny" :disabled="!ghToken.trim() || assistant.syncing" @click="doGithubConnect">Salvar no Cofre</button>
            <button class="pillbtn tiny" @click="openCfg = ''">Cancelar</button>
          </div>
        </div>
        <div v-else class="acc-acts">
          <button v-if="!ghAcc" class="pillbtn tiny" @click="openCfg = 'github'">Conectar GitHub</button>
          <button v-else class="pillbtn tiny danger" @click="disconnectAccount(ghAcc.id)">Desconectar GitHub</button>
          <button class="pillbtn tiny" @click="island.tab = 'work'">GitLab ({{ gitlabCount }}) ↗</button>
        </div>
      </div>
    </div>

    <!-- ================= ABA CELULAR ================= -->
    <div v-else class="phone-grid">
      <!-- Lado Esquerdo: Ponte Multi-Transporte -->
      <section class="panel-box">
        <div class="box-head">
          <span class="set-label">CONEXÃO COM CELULAR</span>
          <div class="seg mini-seg">
            <button :class="{ on: phoneMode === 'lan' }" @click="phoneMode = 'lan'">Wi-Fi</button>
            <button :class="{ on: phoneMode === 'usb' }" @click="phoneMode = 'usb'">Cabo USB</button>
            <button :class="{ on: phoneMode === 'bluetooth' }" @click="phoneMode = 'bluetooth'">Bluetooth</button>
          </div>
        </div>

        <div v-if="phoneMode === 'lan'" class="lan-box">
          <img v-if="qrDataUrl" :src="qrDataUrl" class="qr-img" alt="QR Code Pareamento">
          <div class="lan-info">
            <div class="row-title">Mesma Rede Wi-Fi</div>
            <div class="mono tiny-mono">{{ assistant.pairing?.host || '127.0.0.1' }}:{{ assistant.pairing?.port || 47821 }}</div>
            <div class="mute small">Escaneie o QR no app companheiro para parear via canal seguro.</div>
            <button class="pillbtn tiny" @click="rotatePairingToken">↻ Novo Token</button>
          </div>
        </div>

        <div v-else-if="phoneMode === 'usb'" class="transport-box">
          <div class="row-title">Cabo USB (Baixa Latência)</div>
          <div class="mute small">
            ADB: <b>{{ assistant.pairing?.adbAvailable ? 'Disponível' : 'Não encontrado' }}</b> ·
            Aparelhos: <b>{{ assistant.pairing?.adbDevices.length ? assistant.pairing.adbDevices.join(', ') : 'Nenhum' }}</b>
          </div>
          <div class="mute small">Usa <code>adb reverse</code> para trafegar os mesmos dados pela porta local sem firewall.</div>
          <button class="pillbtn primary tiny" @click="enableUsbBridge">🔌 Ativar Ponte USB</button>
        </div>

        <div v-else class="transport-box">
          <div class="row-title">Bluetooth (Proximidade / ANCS)</div>
          <div class="mute small">
            Pareados: <b>{{ assistant.pairing?.btPairedPhones.length ? assistant.pairing.btPairedPhones.join(', ') : 'Nenhum celular pareado' }}</b>
          </div>
          <div class="form-btns">
            <button class="pillbtn tiny" @click="island.tab = 'net'">Gerenciar Bluetooth ↗</button>
          </div>
        </div>
      </section>

      <!-- Lado Direito: Aparelho & Feed em Tempo Real -->
      <section class="panel-box">
        <div class="box-head">
          <span class="set-label">SINAIS & NOTIFICAÇÕES</span>
          <span v-if="primaryPhone" class="chip ok">
            {{ primaryPhone.name }} · {{ primaryPhone.activeTransport.toUpperCase() }} · {{ primaryPhone.batteryPct ?? '--' }}%
          </span>
          <span v-else class="chip muted">Aguardando celular</span>
        </div>

        <div class="scroll-list">
          <div v-for="sig in assistant.snap.recentSignals" :key="sig.id" class="row-item">
            <span class="ico-badge">{{ sig.kind === 'call' ? '📞' : sig.kind === 'clipboard' ? '📋' : '💬' }}</span>
            <div class="row-txt">
              <div class="row-title">{{ sig.title }} <span class="mute">· {{ sig.appName }}</span></div>
              <div class="row-sub mute">{{ sig.body }}</div>
            </div>
          </div>
          <div v-if="!assistant.snap.recentSignals.length" class="empty-sm">Nenhuma notificação recebida do celular</div>
        </div>

        <div class="test-row">
          <span class="mute small">Simular sinal no avatar:</span>
          <button class="pillbtn tiny" @click="triggerTestPhoneSignal('call', 'Mãe', '+55 11 99999-0000', phoneMode)">📞 Ligação</button>
          <button class="pillbtn tiny" @click="triggerTestPhoneSignal('notification', 'WhatsApp', 'Reunião movida para 15h', phoneMode)">💬 Msg</button>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.assist-wrap { display: flex; flex-direction: column; gap: 8px; overflow: hidden; }
.topbar { display: flex; align-items: center; justify-content: space-between; gap: 10px; flex: none; }
.tabs-seg { width: 290px; }
.tabs-seg button { display: inline-flex; align-items: center; justify-content: center; gap: 5px; font-size: 11.5px; }
.dot-s { width: 6px; height: 6px; border-radius: 50%; }
.dot-s.ok { background: #5be39a; }
.dot-s.warn { background: #ffd23f; }
.bat-tag { font-family: var(--mono); font-size: 10px; color: #5be39a; }

.status-chips { display: flex; align-items: center; gap: 6px; }
.chip { font-size: 10.5px; padding: 2px 8px; border-radius: 999px; background: rgba(255,255,255,.07); white-space: nowrap; }
.chip.ok { color: #5be39a; background: rgba(91,227,154,.12); }
.chip.warn { color: #ffd23f; background: rgba(255,210,63,.14); }
.chip.muted { color: var(--mute); }
.tiny { padding: 3px 8px; font-size: 10.5px; }
.err-bar { padding: 4px 10px; border-radius: 8px; background: rgba(255,107,94,.16); color: #ffb3ac; font-size: 11px; flex: none; }

.hoje-grid { flex: 1; min-height: 0; display: flex; flex-direction: column; gap: 7px; }
.google-banner {
  display: flex; align-items: center; justify-content: space-between; gap: 10px; flex: none;
  padding: 6px 10px; border-radius: 11px;
  background: rgba(91,124,255,.14); border: 1px solid rgba(115,144,255,.32); font-size: 11.5px;
}
.banner-acts { display: flex; gap: 5px; flex: none; }

.cols, .phone-grid { flex: 1; min-height: 0; display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }
.panel-box {
  display: flex; flex-direction: column; gap: 6px; min-height: 0; min-width: 0;
  padding: 8px 10px; border-radius: 13px;
  background: rgba(255,255,255,.04); border: 1px solid var(--line);
}
.box-head { display: flex; align-items: center; justify-content: space-between; gap: 6px; flex: none; }
.scroll-list { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 4px; }

.add-date { display: flex; gap: 4px; align-items: center; flex: none; }
.tin { padding: 3px 7px; font-size: 11px; border-radius: 8px; }
.md-in { width: 62px; flex: none; text-align: center; }

.row-item {
  display: flex; align-items: center; gap: 7px; min-width: 0;
  padding: 4.5px 7px; border-radius: 9px; background: rgba(255,255,255,.035); font-size: 11.5px;
}
.row-item.today { background: rgba(255,210,63,.11); border: 1px solid rgba(255,210,63,.24); }
.row-item.dev { border-left: 2px solid #8b7bff; }
.ico-badge { font-size: 13px; flex: none; }
.time-pill { font-size: 10px; padding: 1px 6px; border-radius: 6px; background: rgba(91,124,255,.18); color: #cad6ff; flex: none; }
.row-txt { flex: 1; min-width: 0; }
.row-title { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 600; font-size: 11.5px; }
.row-sub { white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 10px; }
.when-pill { font-family: var(--mono); font-size: 9.5px; padding: 1px 6px; border-radius: 999px; background: rgba(255,255,255,.08); color: var(--mute); flex: none; }
.when-pill.hot { background: #ffd23f; color: #111; font-weight: 800; }
.del-x { color: var(--mute); font-size: 11px; padding: 0 3px; }
.del-x:hover { color: #ff6b5e; }
.empty-sm { padding: 14px 4px; text-align: center; color: var(--mute); font-size: 11.5px; }

.contas-grid {
  flex: 1; min-height: 0; overflow-y: auto;
  display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); grid-auto-rows: minmax(0, 1fr); gap: 6px;
}
.acc-card {
  display: flex; flex-direction: column; justify-content: space-between; gap: 3px; min-height: 0;
  padding: 6px 9px; border-radius: 11px;
  background: rgba(255,255,255,.04); border: 1px solid var(--line);
}
.primary-card { border-color: rgba(91,124,255,.35); background: rgba(91,124,255,.07); }
.acc-head { display: flex; align-items: center; justify-content: space-between; gap: 6px; }
.acc-name { font-weight: 700; font-size: 11.5px; display: flex; align-items: center; gap: 5px; }
.req-tag { font-size: 9px; padding: 1px 5px; border-radius: 5px; background: rgba(255,210,63,.16); color: #ffd23f; }
.acc-desc { font-size: 10px; line-height: 1.25; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.acc-acts, .form-btns { display: flex; gap: 5px; align-items: center; flex-wrap: wrap; }
.acc-form { display: flex; flex-direction: column; gap: 3px; }

.mini-seg { width: 176px; }
.mini-seg button { font-size: 10px; padding: 2.5px 0; }
.lan-box { flex: 1; display: flex; align-items: center; gap: 10px; min-height: 0; }
.qr-img { width: 88px; height: 88px; border-radius: 8px; background: #fff; padding: 3px; flex: none; }
.lan-info, .transport-box { flex: 1; display: flex; flex-direction: column; gap: 5px; align-items: flex-start; justify-content: center; min-width: 0; }
.tiny-mono { font-size: 11px; color: var(--accent-2); }
.test-row { display: flex; align-items: center; gap: 5px; flex: none; border-top: 1px solid var(--line); padding-top: 5px; }
</style>
