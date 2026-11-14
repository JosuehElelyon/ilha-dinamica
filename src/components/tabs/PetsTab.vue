<script setup lang="ts">
// Seleção do mascote animado no topo da ilha (Cachorro, Gatinho, Cavalinho, Pandinha e Galinha)
import { computed, onMounted, onUnmounted, ref } from 'vue'
import PetSprite from '../PetSprite.vue'
import { PETS, S, save, type PetId } from '../../store/settings'

const hoverId = ref<PetId | null>(null)
const blinkId = ref<PetId | null>(null)
let blinkT: number | undefined

const activePet = computed(() => PETS.find(p => p.id === S.pet) || PETS[1])

function select(id: PetId) {
  save('pet', id)
}

function scheduleBlink() {
  blinkT = window.setTimeout(() => {
    const pick = PETS[Math.floor(Math.random() * PETS.length)].id
    blinkId.value = pick
    setTimeout(() => {
      blinkId.value = null
      scheduleBlink()
    }, 150)
  }, 1400 + Math.random() * 1400)
}

onMounted(scheduleBlink)
onUnmounted(() => clearTimeout(blinkT))
</script>

<template>
  <div class="pets-wrap">
    <div class="top">
      <div class="titles">
        <span class="set-label">MASCOTE DO TOPO</span>
        <span class="sub mute">Selecione o bichinho animado no cabeçalho da ilha</span>
      </div>
      <span class="badge mono">ATIVO · {{ activePet.name.toUpperCase() }}</span>
    </div>

    <div class="grid">
      <button
        v-for="p in PETS"
        :key="p.id"
        class="card"
        :class="{ active: S.pet === p.id }"
        @mouseenter="hoverId = p.id"
        @mouseleave="hoverId = null"
        @click="select(p.id)"
      >
        <span v-if="S.pet === p.id" class="chk-pill">✓</span>
        <span class="tag mono">{{ p.tag }}</span>
        <div class="stage">
          <PetSprite
            :pet="p.id"
            :mode="hoverId === p.id || S.pet === p.id ? 'play' : 'sit'"
            :blink="blinkId === p.id"
            size="lg"
          />
        </div>
        <div class="meta">
          <span class="name">{{ p.name }}</span>
          <span class="desc mute">{{ p.desc }}</span>
        </div>
      </button>
    </div>
  </div>
</template>

<style scoped>
.pets-wrap {
  display: flex;
  flex-direction: column;
  gap: 10px;
  height: 100%;
  overflow: hidden;
}
.top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  flex: none;
}
.titles {
  display: flex;
  align-items: baseline;
  gap: 10px;
  min-width: 0;
}
.sub {
  font-size: 11.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.badge {
  font-size: 10px;
  padding: 3px 9px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--accent), transparent 82%);
  color: var(--accent-2);
  border: 1px solid color-mix(in srgb, var(--accent), transparent 65%);
  flex: none;
}
.grid {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: 10px;
}
.card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-between;
  padding: 9px 8px 10px;
  border-radius: 18px;
  background: rgba(255, 255, 255, .04);
  border: 1px solid rgba(255, 255, 255, .08);
  transition: transform .18s var(--ease), background .18s, border-color .18s, box-shadow .18s;
  min-width: 0;
}
.card:hover {
  transform: translateY(-2px);
  background: rgba(255, 255, 255, .08);
  border-color: rgba(255, 255, 255, .18);
}
.card.active {
  background: color-mix(in srgb, var(--accent), rgba(255, 255, 255, .05) 86%);
  border-color: color-mix(in srgb, var(--accent), transparent 30%);
  box-shadow: 0 6px 20px color-mix(in srgb, var(--accent), transparent 82%);
}
.tag {
  align-self: flex-start;
  font-size: 9px;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(255, 255, 255, .07);
  color: var(--mute);
}
.card.active .tag {
  background: color-mix(in srgb, var(--accent), transparent 78%);
  color: var(--accent-2);
}
.chk-pill {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: var(--accent);
  color: #111;
  font-size: 11px;
  font-weight: 800;
  display: grid;
  place-items: center;
}
.stage {
  flex: 1;
  display: grid;
  place-items: center;
  min-height: 0;
}
.meta {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
  width: 100%;
  min-width: 0;
}
.name {
  font-size: 12.5px;
  font-weight: 700;
  color: var(--txt);
}
.desc {
  font-size: 10px;
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 100%;
}
</style>
