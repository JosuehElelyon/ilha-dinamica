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
  <div class="mods-wrap">
    <div class="top"><span class="set-label">MÓDULOS — ative, desative e reordene as abas</span></div>
    <div class="grid">
      <div v-for="(id, i) in S.modules" :key="id" class="mod" :class="{ off: S.modulesOff.includes(id) }">
        <ModIcon :name="id" />
        <span class="nm">{{ MODULES[id] }}</span>
        <div class="acts">
          <button class="mv" :disabled="i === 0" title="Mover para a esquerda" @click="move(i, -1)">‹</button>
          <button class="mv" :disabled="i === S.modules.length - 1" title="Mover para a direita" @click="move(i, 1)">›</button>
          <button class="switch" :class="{ on: !S.modulesOff.includes(id) }"
                  :title="S.modulesOff.includes(id) ? 'Adicionar' : 'Remover'" @click="toggle(id)" />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mods-wrap {
  display: flex; flex-direction: column; justify-content: center; gap: 6px;
  overflow: hidden;
}
.top { text-align: center; }
.grid {
  display: grid; grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 5px 7px; width: 100%;
}
.mod {
  display: flex; align-items: center; gap: 5px; min-width: 0;
  padding: 4.5px 7px; border-radius: 10px;
  background: rgba(255,255,255,.045); border: 1px solid rgba(255,255,255,.06);
  font-size: 10px;
}
.mod :deep(.ico) { width: 12.5px; height: 12.5px; opacity: .9; }
.nm {
  flex: 1; min-width: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  font-weight: 500;
}
.off { opacity: .45; }
.acts { display: flex; align-items: center; gap: 1px; flex: none; }
.mv { color: var(--mute); width: 12px; height: 16px; border-radius: 5px; font-size: 12px; display: grid; place-items: center; }
.mv:hover:not(:disabled) { color: #fff; background: rgba(255,255,255,.1); }
.mv:disabled { opacity: .22; cursor: default; }
.switch { width: 24px; height: 14px; border-radius: 7px; margin-left: 2px; }
.switch::after { width: 10px; height: 10px; }
.switch.on::after { transform: translateX(10px); }
</style>
