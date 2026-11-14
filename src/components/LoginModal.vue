<script setup lang="ts">
// Modal de Login da Ilha Dinâmica — fluxo direto com Google OAuth sem exigir Client ID na tela principal.
import { computed, ref } from 'vue'
import { S, save } from '../store/settings'
import { island } from '../store/state'
import {
  assistant, connectGoogle, disconnectAccount, openAssistant, syncGoogle
} from '../store/assistant'

const showAdv = ref(false)
const advClientId = ref(S.googleClientId || '')
const advSecret = ref('')

const googleAcc = computed(() => assistant.snap.accounts.find(a => a.provider === 'google'))

async function loginWithGoogle() {
  const cid = advClientId.value.trim() || S.googleClientId || undefined
  if (advClientId.value.trim()) save('googleClientId', advClientId.value.trim())
  await connectGoogle(cid, advSecret.value.trim() || undefined)
  if (assistant.snap.googleConnected) {
    showAdv.value = false
  }
}

function close() {
  island.loginOpen = false
}

function goAccounts() {
  openAssistant('contas')
}
</script>

<template>
  <div class="login-bg" @click="close" @keydown.esc="close">
    <div class="login-modal" @click.stop>
      <div class="lm-head">
        <div class="lm-title">🔑 Login da Ilha Dinâmica</div>
        <button class="lm-close" title="Fechar" @click="close">✕</button>
      </div>

      <template v-if="googleAcc">
        <div class="lm-connected">
          <div class="lm-avatar">G</div>
          <div class="lm-user">
            <div class="lm-name">{{ googleAcc.label }}</div>
            <div class="lm-email">{{ googleAcc.email }} · {{ googleAcc.details }}</div>
          </div>
          <span class="lm-ok">Conectado</span>
        </div>

        <div class="lm-actions">
          <button class="pillbtn primary" :disabled="assistant.syncing" @click="syncGoogle">
            {{ assistant.syncing ? 'Sincronizando…' : '↻ Sincronizar Google' }}
          </button>
          <button class="pillbtn danger" @click="disconnectAccount(googleAcc.id)">Sair</button>
        </div>
      </template>

      <template v-else>
        <p class="lm-desc">
          Entre com sua conta Google para sincronizar agenda, contatos, aniversários e lembretes no avatar.
        </p>

        <button class="google-btn" :disabled="assistant.syncing" @click="loginWithGoogle">
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true">
            <path fill="#4285F4" d="M23.5 12.3c0-.8-.1-1.6-.2-2.3H12v4.4h6.5c-.3 1.5-1.1 2.8-2.4 3.6v3h3.9c2.3-2.1 3.5-5.2 3.5-8.7z" />
            <path fill="#34A853" d="M12 24c3.2 0 6-1.1 8-2.9l-3.9-3c-1.1.7-2.5 1.2-4.1 1.2-3.1 0-5.8-2.1-6.7-5H1.3v3.1C3.3 21.3 7.3 24 12 24z" />
            <path fill="#FBBC05" d="M5.3 14.3c-.2-.7-.4-1.5-.4-2.3s.1-1.6.4-2.3V6.6H1.3C.5 8.2 0 10 0 12s.5 3.8 1.3 5.4l4-3.1z" />
            <path fill="#EA4335" d="M12 4.7c1.8 0 3.3.6 4.6 1.8l3.4-3.4C18 1.1 15.2 0 12 0 7.3 0 3.3 2.7 1.3 6.6l4 3.1c.9-2.9 3.6-5 6.7-5z" />
          </svg>
          <span>{{ assistant.syncing ? 'Abrindo navegador…' : 'Continuar com Google' }}</span>
        </button>

        <div v-if="assistant.error" class="lm-err">
          {{ assistant.error }}
        </div>

        <div v-if="showAdv" class="lm-adv">
          <input v-model="advClientId" type="text" class="in tin" placeholder="Client ID OAuth próprio (opcional)">
          <input v-model="advSecret" type="password" class="in tin" placeholder="Client Secret (opcional)">
        </div>
      </template>

      <div class="lm-foot">
        <button class="link-btn" @click="goAccounts">Outras contas (GitHub, Apple, Windows) ↗</button>
        <button v-if="!googleAcc" class="link-btn" @click="showAdv = !showAdv">
          {{ showAdv ? 'Ocultar OAuth próprio' : 'OAuth próprio' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-bg {
  position: absolute; inset: 0; z-index: 60;
  background: rgba(8, 9, 14, .68); backdrop-filter: blur(8px);
  display: grid; place-items: center; padding: 14px;
}
.login-modal {
  width: 100%; max-width: 390px;
  background: #1a1c24; border: 1px solid rgba(255,255,255,.15); border-radius: 18px;
  box-shadow: 0 18px 46px rgba(0,0,0,.65);
  padding: 14px 16px; display: flex; flex-direction: column; gap: 10px;
}
.lm-head { display: flex; align-items: center; justify-content: space-between; }
.lm-title { font-size: 13px; font-weight: 700; color: #fff; }
.lm-close { color: var(--mute); font-size: 13px; padding: 2px 6px; border-radius: 6px; }
.lm-close:hover { background: rgba(255,255,255,.08); color: #fff; }
.lm-desc { margin: 0; font-size: 11.5px; line-height: 1.4; color: var(--mute); }

.google-btn {
  display: flex; align-items: center; justify-content: center; gap: 9px;
  width: 100%; padding: 9px 14px; border-radius: 11px;
  background: #fff; color: #1f1f1f; font-size: 12.5px; font-weight: 700;
  box-shadow: 0 4px 14px rgba(0,0,0,.28); transition: transform .12s, background .15s;
}
.google-btn:hover:not(:disabled) { background: #f4f6fb; transform: translateY(-1px); }
.google-btn:disabled { opacity: .65; cursor: wait; }

.lm-connected {
  display: flex; align-items: center; gap: 10px;
  padding: 9px 11px; border-radius: 12px;
  background: rgba(91,227,154,.08); border: 1px solid rgba(91,227,154,.26);
}
.lm-avatar {
  width: 32px; height: 32px; border-radius: 50%; flex: none;
  display: grid; place-items: center; background: #4285F4; color: #fff; font-weight: 800; font-size: 14px;
}
.lm-user { flex: 1; min-width: 0; }
.lm-name { font-size: 12px; font-weight: 700; color: #fff; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.lm-email { font-size: 10.5px; color: var(--mute); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.lm-ok { font-size: 10px; padding: 2px 7px; border-radius: 999px; background: rgba(91,227,154,.16); color: #5be39a; font-weight: 700; }
.lm-actions { display: flex; gap: 8px; justify-content: flex-end; }
.lm-err { padding: 6px 9px; border-radius: 9px; background: rgba(255,107,94,.16); color: #ffb3ac; font-size: 10.5px; line-height: 1.35; }
.lm-adv { display: flex; flex-direction: column; gap: 5px; }
.tin { padding: 5px 8px; font-size: 11px; border-radius: 8px; }
.lm-foot {
  display: flex; align-items: center; justify-content: space-between; gap: 8px;
  border-top: 1px solid var(--line); padding-top: 7px;
}
.link-btn { font-size: 10.5px; color: var(--mute); }
.link-btn:hover { color: #fff; text-decoration: underline; }
</style>
