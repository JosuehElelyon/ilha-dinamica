<script setup lang="ts">
// Redes: gerenciamento de Wi-Fi (conectar/desconectar/sinal) e Bluetooth (rádio e dispositivos pareados)
import { onMounted, ref } from 'vue'
import { api } from '../../lib/api'
import type { BtState, WifiNet } from '../../lib/types'
import { live, refreshWifi, showToast } from '../../store/state'

const nets = ref<WifiNet[]>([])
const bt = ref<BtState>({ available: false, enabled: false, devices: [] })
const scanning = ref(false)
const busySsid = ref('')
const togglingBt = ref(false)

async function loadAll() {
  scanning.value = true
  try {
    const [w, b] = await Promise.all([api.wifiScan(), api.btState()])
    nets.value = w
    bt.value = b
    await refreshWifi()
  } finally {
    scanning.value = false
  }
}

async function connectWifi(ssid: string) {
  busySsid.value = ssid
  try {
    const msg = await api.wifiConnect(ssid)
    showToast({ icon: '📶', text: msg, ms: 2500 })
    setTimeout(loadAll, 1500)
  } catch (e) {
    showToast({ icon: '⚠️', text: String(e), alert: true, ms: 4500 })
  } finally {
    busySsid.value = ''
  }
}

async function disconnectWifi() {
  await api.wifiDisconnect()
  showToast({ icon: '📶', text: 'Wi-Fi desconectado', ms: 2000 })
  setTimeout(loadAll, 600)
}

async function toggleBt() {
  if (togglingBt.value) return
  togglingBt.value = true
  const next = !bt.value.enabled
  try {
    await api.btToggleRadio(next)
    bt.value.enabled = next
    showToast({ icon: '🔵', text: next ? 'Bluetooth ligado' : 'Bluetooth desligado', ms: 2000 })
    setTimeout(loadAll, 800)
  } catch (e) {
    showToast({ icon: '⚠️', text: String(e) || 'Não foi possível alternar o Bluetooth', alert: true, ms: 3500 })
  } finally {
    togglingBt.value = false
  }
}

onMounted(loadAll)
</script>

<template>
  <div class="net-grid">
    <!-- ================= WI-FI ================= -->
    <section class="panel-col">
      <div class="col-head">
        <div class="col-title">
          <svg class="hico" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M5 12.55a11 11 0 0 1 14.08 0" />
            <path d="M1.42 9a16 16 0 0 1 21.16 0" />
            <path d="M8.53 16.11a6 6 0 0 1 6.95 0" />
            <circle cx="12" cy="19.5" r="1" fill="currentColor" />
          </svg>
          <span class="mono">WI-FI</span>
          <span v-if="live.wifi.connected" class="badge on">{{ live.wifi.ssid }} · {{ live.wifi.signal }}%</span>
          <span v-else class="badge">Desconectado</span>
        </div>
        <div class="col-acts">
          <button class="sbtn" :disabled="scanning" title="Atualizar redes" @click="loadAll">{{ scanning ? '…' : '↻' }}</button>
          <button class="sbtn" title="Configurações de Wi-Fi do Windows" @click="api.runAction('ms-settings:network-wifi')">⚙</button>
        </div>
      </div>

      <div class="list">
        <div v-if="!nets.length && !scanning" class="empty-sm">Nenhuma rede Wi-Fi encontrada</div>
        <div v-for="n in nets" :key="n.ssid" class="item" :class="{ active: n.connected }">
          <div class="imain">
            <div class="iname">{{ n.ssid }}</div>
            <div class="isub mono">{{ n.signal }}% · {{ n.auth || 'Aberta' }}</div>
          </div>
          <button v-if="n.connected" class="ibtn off" @click="disconnectWifi">Desconectar</button>
          <button v-else class="ibtn" :disabled="busySsid === n.ssid" @click="connectWifi(n.ssid)">
            {{ busySsid === n.ssid ? '…' : 'Conectar' }}
          </button>
        </div>
      </div>
    </section>

    <!-- ================= BLUETOOTH ================= -->
    <section class="panel-col">
      <div class="col-head">
        <div class="col-title">
          <svg class="hico bt" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="m7 7 10 10-5 5V2l5 5L7 17" />
          </svg>
          <span class="mono">BLUETOOTH</span>
        </div>
        <div class="col-acts">
          <button v-if="bt.available" class="switch" :class="{ on: bt.enabled }" :disabled="togglingBt"
                  :title="bt.enabled ? 'Desligar Bluetooth' : 'Ligar Bluetooth'" @click="toggleBt" />
          <button class="sbtn" title="Configurações de Bluetooth do Windows" @click="api.runAction('ms-settings:bluetooth')">⚙</button>
        </div>
      </div>

      <div class="list">
        <div v-if="!bt.available" class="empty-sm">Adaptador Bluetooth não detectado</div>
        <div v-else-if="!bt.enabled" class="empty-sm">Bluetooth desligado — ative na chave acima</div>
        <div v-else-if="!bt.devices.length" class="empty-sm">Nenhum dispositivo pareado</div>
        <div v-for="d in bt.devices" :key="d.id" class="item" :class="{ active: d.connected }">
          <div class="imain">
            <div class="iname">{{ d.name }}</div>
            <div class="isub mono">{{ d.connected ? 'Conectado' : 'Pareado' }} · {{ d.kind }}</div>
          </div>
          <span class="dot" :class="{ on: d.connected }" :title="d.connected ? 'Conectado' : 'Desconectado'" />
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.net-grid { display: grid; grid-template-columns: 1.15fr 1fr; gap: 12px; height: 100%; }
.panel-col {
  display: flex; flex-direction: column; min-height: 0; padding: 10px 12px;
  border-radius: 18px; background: rgba(255,255,255,.04); border: 1px solid rgba(255,255,255,.08);
}
.col-head { display: flex; align-items: center; justify-content: space-between; gap: 6px; margin-bottom: 8px; }
.col-title { display: flex; align-items: center; gap: 6px; min-width: 0; font-size: 11px; color: var(--mute); }
.hico { width: 14px; height: 14px; color: var(--accent); flex: none; }
.hico.bt { color: #5aa9ff; }
.badge {
  font-size: 10px; padding: 1px 7px; border-radius: 999px; background: rgba(255,255,255,.07);
  color: var(--mute); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; max-width: 150px;
}
.badge.on { background: rgba(52, 208, 88, .18); color: #5be39a; }
.col-acts { display: flex; align-items: center; gap: 6px; flex: none; }
.sbtn {
  width: 24px; height: 24px; border-radius: 8px; background: rgba(255,255,255,.07);
  color: var(--txt); font-size: 12px; display: grid; place-items: center;
}
.sbtn:hover { background: rgba(255,255,255,.14); }
.sbtn:disabled { opacity: .45; }

.list { flex: 1; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; gap: 5px; padding-right: 2px; }
.empty-sm { margin: auto; font-size: 12px; color: var(--mute); text-align: center; padding: 12px; }
.item {
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
  padding: 6px 9px; border-radius: 12px; background: rgba(255,255,255,.04); border: 1px solid transparent;
}
.item.active { background: rgba(255,255,255,.08); border-color: rgba(255,255,255,.12); }
.imain { min-width: 0; }
.iname { font-size: 12.5px; font-weight: 600; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.isub { font-size: 10px; color: var(--mute); }
.ibtn {
  padding: 3px 9px; border-radius: 999px; background: var(--accent); color: #111;
  font-size: 10.5px; font-weight: 700; flex: none;
}
.ibtn.off { background: rgba(255,255,255,.12); color: #fff; }
.ibtn:hover { opacity: .9; }
.dot { width: 8px; height: 8px; border-radius: 50%; background: rgba(255,255,255,.2); flex: none; }
.dot.on { background: #5be39a; box-shadow: 0 0 8px rgba(91,227,154,.6); }
</style>
