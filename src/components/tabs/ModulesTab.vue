<script setup lang="ts">
// Gerenciador de módulos: ativar, desativar e reordenar as abas
import ModIcon from '../ModIcon.vue'
import { MODULES, S, save, activeModules, type ModuleId } from '../../store/settings'

function toggle(id: ModuleId) {
  const off = S.modulesOff.includes(id)
  if (!off && activeModules().length <= 1) return // mantém pelo menos uma aba
  save('modulesOff', off ? S.modulesOff.filter(m => m !== id) : [...S.modulesOff, id])
}
function move(i: number, d: number) {
  const j = i + d
  if (j < 0 || j >= S.modules.length) return
  const arr = [...S.modules];
  [arr[i], arr[j]] = [arr[j], arr[i]]
  save('modules', arr)
}
</script>

<template>
  <div>
    <div class="top"><span class="set-label">MÓDULOS — ative, desative e reordene as abas</span></div>
    <div class="grid">
      <div v-for="(id, i) in S.modules" :key="id" class="mod" :class="{ off: S.modulesOff.includes(id) }">
        <ModIcon :name="id" /><span class="nm">{{ MODULES[id] }}</span>
        <button class="mv" title="Mover para a esquerda" @click="move(i, -1)">‹</button>
        <button class="mv" title="Mover para a direita" @click="move(i, 1)">›</button>
        <button class="switch" :class="{ on: !S.modulesOff.includes(id) }"
                :title="S.modulesOff.includes(id) ? 'Adicionar' : 'Remover'" @click="toggle(id)" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.top { padding: 2px 8px 8px; }
.grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; padding: 0 8px; }
.mod { display: flex; align-items: center; gap: 8px; padding: 7px 10px; border-radius: 12px; background: rgba(255,255,255,.04); font-size: 12.5px; }
.nm { flex: 1; }
.off { opacity: .45; }
.mv { color: var(--mute); padding: 0 3px; border-radius: 6px; font-size: 13px; }
.mv:hover { color: #fff; background: rgba(255,255,255,.08); }
</style>
