<script setup lang="ts">
// Barra de abas (dinâmica) + botão "+" que abre o gerenciador de módulos
import ModIcon from './ModIcon.vue'
import { MODULES, activeModules } from '../store/settings'
import { island } from '../store/state'

function togglePlus() {
  island.tab = island.tab === 'modules' ? (activeModules()[0] || 'home') : 'modules'
}
</script>

<template>
  <nav>
    <button class="tab-btn plus" :class="{ active: island.tab === 'modules' }" title="Adicionar ou remover módulos" @click="togglePlus">
      <ModIcon name="plus" />
    </button>
    <div class="tabs">
      <button v-for="m in activeModules()" :key="m" class="tab-btn" :class="{ active: island.tab === m }"
              :title="MODULES[m]" @click="island.tab = m">
        <ModIcon :name="m" /><span>{{ MODULES[m] }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
nav { position: relative; display: flex; justify-content: center; align-items: center; gap: 2px; padding: 12px 12px 14px; }
.tabs { display: flex; gap: 2px; }
.tab-btn { display: flex; align-items: center; gap: 5px; height: 30px; padding: 0 7.5px; border-radius: 15px;
  color: #a9a9b2; transition: background .2s, color .2s; }
.tab-btn span { display: none; font-family: var(--mono); font-size: 10px; letter-spacing: .08em; font-weight: 600; }
.tab-btn:hover { color: #fff; background: rgba(255,255,255,.05); }
.tab-btn.active { color: #fff; background: rgba(255,255,255,.1); padding: 0 11px; }
.tab-btn.active span { display: inline; }
.plus { position: absolute; left: 14px; color: #8b8b94; }
.plus.active { padding: 0 7.5px; }
</style>
