<script setup lang="ts">
// Mascotes chibi estilo anime kawaii para o topo da ilha e prévia de seleção.
// Renderização SVG leve com cel-shading (sem gradientes pesados) e animações em GPU.
import { computed } from 'vue'
import type { PetId } from '../store/settings'

export type PetMode = 'walk' | 'sit' | 'jump' | 'play' | 'peek'

const props = withDefaults(defineProps<{
  pet: PetId
  mode?: PetMode
  blink?: boolean
  busy?: boolean
  dir?: 1 | -1
  jumpY?: number
  size?: 'sm' | 'lg'
  hideShadow?: boolean
}>(), {
  mode: 'sit',
  blink: false,
  busy: false,
  dir: 1,
  jumpY: 0,
  size: 'sm',
  hideShadow: false
})

interface Palette {
  fur: string
  shade: string
  belly: string
  earIn: string
  stroke: string
  irisGlow: string
  accent: string
  nose: string
}

const PALETTES: Record<PetId, Palette> = {
  dog: {
    fur: '#f39e58',
    shade: '#d47c38',
    belly: '#fff0d6',
    earIn: '#ff9ab5',
    stroke: '#2b1b24',
    irisGlow: '#ffba6b',
    accent: '#ff647c',
    nose: '#2b1b24'
  },
  cat: {
    fur: '#9c93e8',
    shade: '#7970c8',
    belly: '#f7f2ff',
    earIn: '#ff9ab8',
    stroke: '#221a35',
    irisGlow: '#b8a8ff',
    accent: '#86e3c2',
    nose: '#ff85a8'
  },
  horse: {
    fur: '#c67346',
    shade: '#9d512c',
    belly: '#ffe6c7',
    earIn: '#ff9eb5',
    stroke: '#2a181e',
    irisGlow: '#ffd480',
    accent: '#ffd875',
    nose: '#3a2224'
  },
  panda: {
    fur: '#f5f4fa',
    shade: '#dedbe8',
    belly: '#2b253a',
    earIn: '#665c85',
    stroke: '#4e4569',
    irisGlow: '#8ce8c8',
    accent: '#7de2b8',
    nose: '#231d30'
  },
  chicken: {
    fur: '#fff2c9',
    shade: '#f0d484',
    belly: '#fffaf0',
    earIn: '#ff5d70',
    stroke: '#2c1e24',
    irisGlow: '#85d9ff',
    accent: '#ff5d70',
    nose: '#ff9838'
  }
}

const p = computed(() => PALETTES[props.pet] || PALETTES.cat)
const isSide = computed(() => props.mode === 'walk' || props.mode === 'jump' || props.mode === 'play')
const dims = computed(() => (props.size === 'lg' ? { w: 84, h: 63 } : { w: 46, h: 34 }))
</script>

<template>
  <svg
    class="pet-svg"
    :class="[
      `pet-${pet}`,
      `mode-${mode}`,
      { flip: isSide && dir === -1, blink, busy }
    ]"
    viewBox="0 0 48 36"
    :width="dims.w"
    :height="dims.h"
    aria-hidden="true"
  >
    <!-- Sombra no chão -->
    <ellipse
      v-if="!hideShadow && mode !== 'peek'"
      cx="24"
      :cy="33.8 - jumpY"
      :rx="mode === 'jump' ? 7.5 : isSide ? 11.5 : 9.5"
      ry="1.35"
      fill="#000"
      :opacity="mode === 'jump' ? 0.16 : 0.32"
    />

    <!-- ================================================================ -->
    <!-- POSES LATERAIS / 3-4: ANDANDO, PULANDO E BRINCANDO               -->
    <!-- ================================================================ -->
    <g v-if="isSide" class="body-rig">
      <!-- Rabinho / penas da cauda -->
      <g class="tail">
        <template v-if="pet === 'dog'">
          <path d="M 13 22 C 6 19 6 11 11 10.5 C 13.5 10.2 13.5 13.2 11 13.8" fill="none" :stroke="p.stroke" stroke-width="4.2" stroke-linecap="round" />
          <path d="M 13 22 C 6 19 6 11 11 10.5 C 13.5 10.2 13.5 13.2 11 13.8" fill="none" :stroke="p.fur" stroke-width="2.6" stroke-linecap="round" />
        </template>
        <template v-else-if="pet === 'cat'">
          <path d="M 13 22.5 C 6 20.5 5 12 9.5 8.5 C 11.5 7 12.8 9.2 10.8 10.6" fill="none" :stroke="p.stroke" stroke-width="3.8" stroke-linecap="round" />
          <path d="M 13 22.5 C 6 20.5 5 12 9.5 8.5 C 11.5 7 12.8 9.2 10.8 10.6" fill="none" :stroke="p.fur" stroke-width="2.3" stroke-linecap="round" />
        </template>
        <path v-else-if="pet === 'horse'" d="M 12.5 21 C 6.5 20.5 5.5 26 8 29 C 9.5 26.5 10.5 24 12.5 23.5 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".9" stroke-linejoin="round" />
        <circle v-else-if="pet === 'panda'" cx="12" cy="23.5" r="2.2" :fill="p.belly" :stroke="p.stroke" stroke-width=".9" />
        <path v-else-if="pet === 'chicken'" d="M 12.5 22 L 7.5 18.5 L 10 22 L 7.5 24.5 L 13 25 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".9" stroke-linejoin="round" />
      </g>

      <!-- Patinhas de trás -->
      <g class="paw paw-b">
        <rect v-if="pet !== 'chicken'" x="14.5" y="25" width="3.8" height="7" rx="1.9" :fill="pet === 'panda' ? p.belly : p.shade" :stroke="p.stroke" stroke-width=".85" />
        <rect v-if="pet === 'horse'" x="14.5" y="29.6" width="3.8" height="2.4" rx="1" fill="#3b2320" :stroke="p.stroke" stroke-width=".7" />
        <path v-if="pet === 'chicken'" d="M 20 28 L 19 32 M 17.5 32 L 20.8 32" fill="none" :stroke="p.nose" stroke-width="1.35" stroke-linecap="round" />
      </g>
      <g class="paw paw-a">
        <rect v-if="pet !== 'chicken'" x="25.5" y="25" width="3.8" height="7" rx="1.9" :fill="pet === 'panda' ? p.belly : p.shade" :stroke="p.stroke" stroke-width=".85" />
        <rect v-if="pet === 'horse'" x="25.5" y="29.6" width="3.8" height="2.4" rx="1" fill="#3b2320" :stroke="p.stroke" stroke-width=".7" />
      </g>

      <!-- Corpo chibi arredondado -->
      <ellipse
        cx="22"
        :cy="pet === 'chicken' ? 23.8 : 23.2"
        :rx="pet === 'chicken' ? 10 : 9.4"
        :ry="pet === 'chicken' ? 6.2 : 5.4"
        :fill="p.fur"
        :stroke="p.stroke"
        stroke-width="1"
      />
      <!-- Barriguinha / marcação no corpo -->
      <ellipse v-if="pet !== 'panda'" cx="23.5" cy="24.5" rx="5.5" ry="3.2" :fill="p.belly" opacity=".88" />
      <path v-if="pet === 'panda'" d="M 15.5 21.5 Q 22 23.5 28.5 21.5 Q 28 24.8 22 25.5 Q 16 24.8 15.5 21.5 Z" :fill="p.belly" />

      <!-- Patinhas da frente (ou asinha na galinha) -->
      <g v-if="pet === 'chicken'" class="wing">
        <path d="M 16.5 21.8 C 21 20 25 22 24 25.5 C 21.5 26.8 16.5 25.2 16.5 21.8 Z" :fill="p.shade" :stroke="p.stroke" stroke-width=".85" />
      </g>
      <g v-if="pet === 'chicken'" class="paw paw-a">
        <path d="M 24.5 28.5 L 25 32.3 M 23.5 32.3 L 26.8 32.3" fill="none" :stroke="p.nose" stroke-width="1.35" stroke-linecap="round" />
      </g>
      <template v-else>
        <g class="paw paw-a">
          <rect x="17.2" y="25.2" width="4" height="7.2" rx="2" :fill="pet === 'panda' ? p.belly : p.fur" :stroke="p.stroke" stroke-width=".85" />
          <ellipse v-if="pet === 'dog' || pet === 'cat'" cx="19.2" cy="31.5" rx="1.8" ry="1.1" :fill="p.belly" />
          <rect v-if="pet === 'horse'" x="17.2" y="29.8" width="4" height="2.6" rx="1" fill="#3b2320" :stroke="p.stroke" stroke-width=".7" />
        </g>
        <g :class="mode === 'play' ? 'play-paw' : 'paw paw-b'">
          <rect x="27.5" y="25" width="4" height="7.2" rx="2" :fill="pet === 'panda' ? p.belly : p.fur" :stroke="p.stroke" stroke-width=".85" />
          <ellipse v-if="pet === 'dog' || pet === 'cat'" cx="29.5" cy="31.3" rx="1.8" ry="1.1" :fill="p.belly" />
          <rect v-if="pet === 'horse'" x="27.5" y="29.6" width="4" height="2.6" rx="1" fill="#3b2320" :stroke="p.stroke" stroke-width=".7" />
        </g>
      </template>

      <!-- Coleira / lenço no pescoço -->
      <path v-if="pet === 'dog' || pet === 'cat'" d="M 22.5 19.6 Q 28.5 22 33.5 19.2" fill="none" :stroke="p.accent" stroke-width="2" stroke-linecap="round" />
      <circle v-if="pet === 'dog'" cx="30" cy="21.4" r="1.45" fill="#ffd24c" :stroke="p.stroke" stroke-width=".6" />
      <path v-if="pet === 'cat'" d="M 29.8 20.5 C 29.2 19.8 28.2 20.3 28.6 21.1 C 29 21.8 29.8 22.3 29.8 22.3 C 29.8 22.3 30.6 21.8 31 21.1 C 31.4 20.3 30.4 19.8 29.8 20.5 Z" fill="#ff6b90" :stroke="p.stroke" stroke-width=".45" />
      <path v-if="pet === 'panda'" d="M 22 19.8 Q 28.5 22.2 34.5 19.4" fill="none" :stroke="p.accent" stroke-width="2.4" stroke-linecap="round" />
      <path v-if="pet === 'panda'" d="M 25 21 L 23.2 24.4 L 26.2 24.2 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".6" stroke-linejoin="round" />

      <!-- Crina no pescoço do cavalinho -->
      <path v-if="pet === 'horse'" d="M 19.5 13 C 17 15 17 19.5 19.8 21.5 C 21.5 19.5 22 15.5 21.2 13 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".85" stroke-linejoin="round" />

      <!-- CABEÇA CHIBI (3/4 perfil) -->
      <g class="head-bob">
        <!-- Orelhas / Crista atrás -->
        <template v-if="pet === 'dog' || pet === 'cat'">
          <path d="M 22.5 10.2 L 21.2 3.2 L 27 7.6 Z" :fill="p.shade" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 31 7.6 L 35.5 3.2 L 36.2 10.2 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 32.1 7.6 L 34.7 4.7 L 35.1 8.8 Z" :fill="p.earIn" opacity=".85" />
        </template>
        <template v-else-if="pet === 'horse'">
          <path d="M 24.5 8.8 L 24 2.8 L 28.2 6.8 Z" :fill="p.shade" :stroke="p.stroke" stroke-width=".9" stroke-linejoin="round" />
          <path d="M 31 6.8 L 34.6 2.6 L 35.5 8.8 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".9" stroke-linejoin="round" />
          <path d="M 32.1 7 L 34 4.4 L 34.5 7.8 Z" :fill="p.earIn" opacity=".8" />
        </template>
        <template v-else-if="pet === 'panda'">
          <circle cx="22.8" cy="7.4" r="3.4" :fill="p.belly" :stroke="p.stroke" stroke-width=".95" />
          <circle cx="34.8" cy="7.4" r="3.4" :fill="p.belly" :stroke="p.stroke" stroke-width=".95" />
        </template>
        <template v-else-if="pet === 'chicken'">
          <path d="M 25.5 7.2 C 25.2 4 27.6 3.5 28.6 5.4 C 29.6 3 32.2 3.4 31.8 5.8 C 33.4 4.6 35 6 33.6 7.8 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".85" stroke-linejoin="round" />
        </template>

        <!-- Formato fofo da cabeça -->
        <ellipse cx="29.5" cy="14" rx="9" ry="7.6" :fill="p.fur" :stroke="p.stroke" stroke-width="1" />

        <!-- Franja fofa da crina do cavalinho -->
        <path v-if="pet === 'horse'" d="M 26 7.2 C 28 5.2 32.2 5.6 33.4 8.5 C 31.5 9.2 29.5 8.2 28.2 9.4 C 27 8.4 26 8.2 26 7.2 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".8" stroke-linejoin="round" />

        <!-- Focinho creme -->
        <ellipse v-if="pet === 'dog' || pet === 'cat' || pet === 'horse'" cx="32.8" cy="17.2" rx="4.6" ry="3.1" :fill="p.belly" />

        <!-- Manchas nos olhos do pandinha -->
        <template v-if="pet === 'panda'">
          <ellipse cx="27.4" cy="14.2" rx="2.7" ry="3.1" transform="rotate(-10 27.4 14.2)" :fill="p.belly" />
          <ellipse cx="34.2" cy="14.2" rx="2.5" ry="3" transform="rotate(10 34.2 14.2)" :fill="p.belly" />
        </template>

        <!-- Bochechas coradas estilo anime -->
        <ellipse cx="25.2" cy="16.8" rx="1.8" ry=".95" fill="#ff7b9c" opacity=".52" />
        <ellipse cx="35.6" cy="16.8" rx="1.7" ry=".95" fill="#ff7b9c" opacity=".52" />
        <path d="M 24.6 17.1 L 25.1 16.3 M 25.5 17.1 L 26 16.3" stroke="#ff5c86" stroke-width=".55" stroke-linecap="round" opacity=".75" />

        <!-- Bigodinhos no gatinho -->
        <path v-if="pet === 'cat'" d="M 23.2 15.8 L 19.6 15.2 M 23.2 17 L 19.8 17.6 M 36.8 15.8 L 40.2 15.2 M 36.8 17 L 40 17.6"
              stroke="#5c537d" stroke-width=".65" stroke-linecap="round" opacity=".8" />

        <!-- Olhos anime kawaii com brilho duplo -->
        <g class="eyes">
          <!-- Olho esquerdo -->
          <g class="eye-l">
            <ellipse cx="27.6" cy="13.8" rx="1.95" ry="2.4" fill="#1e1429" />
            <path d="M 26.2 14.5 A 1.5 1.3 0 0 0 29 14.5" :fill="p.irisGlow" opacity=".75" />
            <circle class="shine" cx="26.9" cy="12.7" r=".82" fill="#fff" />
            <circle class="shine" cx="28.3" cy="14.7" r=".42" fill="#fff" />
          </g>
          <path d="M 25.5 12.2 Q 27.6 10.6 29.6 12.1" fill="none" :stroke="p.stroke" stroke-width=".95" stroke-linecap="round" />

          <!-- Olho direito -->
          <g class="eye-r">
            <ellipse cx="33.8" cy="13.8" rx="1.8" ry="2.35" fill="#1e1429" />
            <path d="M 32.5 14.5 A 1.4 1.25 0 0 0 35.1 14.5" :fill="p.irisGlow" opacity=".75" />
            <circle class="shine" cx="33.1" cy="12.7" r=".78" fill="#fff" />
            <circle class="shine" cx="34.4" cy="14.7" r=".4" fill="#fff" />
          </g>
          <path d="M 32 12.1 Q 33.9 10.6 35.7 12.2" fill="none" :stroke="p.stroke" stroke-width=".95" stroke-linecap="round" />
        </g>

        <!-- Nariz / Bico e boquinha ":3" -->
        <template v-if="pet === 'chicken'">
          <polygon points="32,15.6 36.4,16.7 32,17.9" :fill="p.nose" :stroke="p.stroke" stroke-width=".7" stroke-linejoin="round" />
          <ellipse cx="32.4" cy="18.9" rx=".95" ry="1.25" :fill="p.accent" :stroke="p.stroke" stroke-width=".55" />
        </template>
        <template v-else>
          <ellipse cx="31.5" cy="16.2" rx="1.15" ry=".78" :fill="p.nose" />
          <path d="M 29.8 17.5 Q 30.6 18.4 31.5 17.4 Q 32.4 18.4 33.2 17.5" fill="none" :stroke="p.stroke" stroke-width=".8" stroke-linecap="round" />
        </template>
      </g>
    </g>

    <!-- ================================================================ -->
    <!-- POSES FRONTAIS: SENTADO E ESPIANDO A BORDA                       -->
    <!-- ================================================================ -->
    <g v-else class="front-rig" :class="{ peek: mode === 'peek' }">
      <!-- Rabinho lateral aparecendo ao sentar -->
      <g v-if="mode === 'sit'" class="tail-sit">
        <path v-if="pet === 'dog' || pet === 'cat'" d="M 29.5 30 Q 38 30 37 22 Q 36 18 33 19.5" fill="none" :stroke="p.stroke" stroke-width="3.6" stroke-linecap="round" />
        <path v-if="pet === 'dog' || pet === 'cat'" d="M 29.5 30 Q 38 30 37 22 Q 36 18 33 19.5" fill="none" :stroke="p.fur" stroke-width="2.1" stroke-linecap="round" />
        <path v-else-if="pet === 'horse'" d="M 30 27 Q 37 26 35.5 31 Q 32.5 31.5 30 29 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".85" stroke-linejoin="round" />
      </g>

      <!-- Corpinho sentado -->
      <g v-if="mode === 'sit'">
        <path d="M 16.5 27.5 C 16.5 21 19.5 19.5 24 19.5 C 28.5 19.5 31.5 21 31.5 27.5 C 31.5 31.2 28.8 32.5 24 32.5 C 19.2 32.5 16.5 31.2 16.5 27.5 Z"
              :fill="p.fur" :stroke="p.stroke" stroke-width="1" />
        <ellipse v-if="pet !== 'panda'" cx="24" cy="27.2" rx="4.2" ry="3.8" :fill="p.belly" opacity=".9" />
        <path v-if="pet === 'panda'" d="M 17 23.5 Q 24 25.5 31 23.5 L 31.2 26.5 Q 24 28 16.8 26.5 Z" :fill="p.belly" />
        <!-- Asinhas fofas da galinha sentada -->
        <ellipse v-if="pet === 'chicken'" cx="16.4" cy="26.5" rx="2" ry="3.2" transform="rotate(14 16.4 26.5)" :fill="p.shade" :stroke="p.stroke" stroke-width=".8" />
        <ellipse v-if="pet === 'chicken'" cx="31.6" cy="26.5" rx="2" ry="3.2" transform="rotate(-14 31.6 26.5)" :fill="p.shade" :stroke="p.stroke" stroke-width=".8" />

        <!-- Patinhas apoiadas -->
        <template v-if="pet === 'chicken'">
          <path d="M 20.5 31.5 L 20 33.2 M 18.6 33.2 L 21.8 33.2 M 27.5 31.5 L 28 33.2 M 26.2 33.2 L 29.4 33.2"
                fill="none" :stroke="p.nose" stroke-width="1.35" stroke-linecap="round" />
        </template>
        <template v-else>
          <ellipse cx="20.4" cy="31.8" rx="2.7" ry="1.75" :fill="pet === 'horse' ? '#3b2320' : pet === 'panda' ? p.belly : p.belly" :stroke="p.stroke" stroke-width=".85" />
          <ellipse cx="27.6" cy="31.8" rx="2.7" ry="1.75" :fill="pet === 'horse' ? '#3b2320' : pet === 'panda' ? p.belly : p.belly" :stroke="p.stroke" stroke-width=".85" />
        </template>

        <!-- Coleira / lenço -->
        <path v-if="pet === 'dog' || pet === 'cat'" d="M 18.5 21.4 Q 24 23.6 29.5 21.4" fill="none" :stroke="p.accent" stroke-width="2" stroke-linecap="round" />
        <circle v-if="pet === 'dog'" cx="24" cy="23.2" r="1.45" fill="#ffd24c" :stroke="p.stroke" stroke-width=".6" />
        <path v-if="pet === 'cat'" d="M 24 22.3 C 23.4 21.6 22.4 22.1 22.8 22.9 C 23.2 23.6 24 24.1 24 24.1 C 24 24.1 24.8 23.6 25.2 22.9 C 25.6 22.1 24.6 21.6 24 22.3 Z" fill="#ff6b90" :stroke="p.stroke" stroke-width=".45" />
        <path v-if="pet === 'panda'" d="M 18.5 21.6 Q 24 23.8 29.5 21.6" fill="none" :stroke="p.accent" stroke-width="2.4" stroke-linecap="round" />
        <path v-if="pet === 'panda'" d="M 26.2 22.5 L 27.8 26 L 24.8 25.8 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".6" stroke-linejoin="round" />
      </g>

      <!-- CABEÇA CHIBI FRONTAL -->
      <g class="head-bob">
        <!-- Orelhas / Crista -->
        <template v-if="pet === 'dog' || pet === 'cat'">
          <path d="M 15.6 11.4 L 13.8 3.6 L 20.6 8.2 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 27.4 8.2 L 34.2 3.6 L 32.4 11.4 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 16.2 10.2 L 15.2 5.4 L 19.3 8.4 Z" :fill="p.earIn" opacity=".85" />
          <path d="M 28.7 8.4 L 32.8 5.4 L 31.8 10.2 Z" :fill="p.earIn" opacity=".85" />
        </template>
        <template v-else-if="pet === 'horse'">
          <path d="M 16.5 10.5 L 15.2 3.4 L 20.6 7.8 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 27.4 7.8 L 32.8 3.4 L 31.5 10.5 Z" :fill="p.fur" :stroke="p.stroke" stroke-width=".95" stroke-linejoin="round" />
          <path d="M 17 9.6 L 16.2 5.2 L 19.5 7.9 Z" :fill="p.earIn" opacity=".8" />
          <path d="M 28.5 7.9 L 31.8 5.2 L 31 9.6 Z" :fill="p.earIn" opacity=".8" />
        </template>
        <template v-else-if="pet === 'panda'">
          <circle cx="16" cy="7.8" r="3.5" :fill="p.belly" :stroke="p.stroke" stroke-width=".95" />
          <circle cx="32" cy="7.8" r="3.5" :fill="p.belly" :stroke="p.stroke" stroke-width=".95" />
        </template>
        <template v-else-if="pet === 'chicken'">
          <path d="M 19.8 7.6 C 19.6 4.4 22 3.8 22.8 5.8 C 23.8 3.2 26.2 3.4 25.8 5.8 C 27.2 4.6 28.8 6 27.5 7.8 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".85" stroke-linejoin="round" />
        </template>

        <!-- Cabeça arredondada -->
        <ellipse cx="24" :cy="mode === 'peek' ? 16.2 : 14.5" rx="9.5" ry="7.8" :fill="p.fur" :stroke="p.stroke" stroke-width="1" />

        <!-- Franjinha do cavalinho -->
        <path v-if="pet === 'horse'" d="M 19.5 7.6 C 21.5 5.4 26.5 5.4 28.5 7.8 C 26.8 9.2 25 8 24 9.6 C 23 8 21.2 9.2 19.5 7.6 Z" :fill="p.accent" :stroke="p.stroke" stroke-width=".8" stroke-linejoin="round" />

        <!-- Focinho creme -->
        <ellipse v-if="pet === 'dog' || pet === 'cat' || pet === 'horse'" cx="24" :cy="mode === 'peek' ? 19.5 : 17.8" rx="4.8" ry="3.1" :fill="p.belly" />

        <!-- Manchas do pandinha -->
        <template v-if="pet === 'panda'">
          <ellipse cx="19.8" :cy="mode === 'peek' ? 16.4 : 14.7" rx="2.7" ry="3.1" transform="rotate(-14 19.8 15)" :fill="p.belly" />
          <ellipse cx="28.2" :cy="mode === 'peek' ? 16.4 : 14.7" rx="2.7" ry="3.1" transform="rotate(14 28.2 15)" :fill="p.belly" />
        </template>

        <!-- Bochechas coradas + tracinhos anime -->
        <ellipse cx="16.6" :cy="mode === 'peek' ? 18.8 : 17.1" rx="1.8" ry=".95" fill="#ff7b9c" opacity=".52" />
        <ellipse cx="31.4" :cy="mode === 'peek' ? 18.8 : 17.1" rx="1.8" ry=".95" fill="#ff7b9c" opacity=".52" />
        <path v-if="mode === 'sit'" d="M 15.8 17.4 L 16.3 16.6 M 16.7 17.4 L 17.2 16.6 M 30.8 17.4 L 31.3 16.6 M 31.7 17.4 L 32.2 16.6"
              stroke="#ff5c86" stroke-width=".55" stroke-linecap="round" opacity=".75" />

        <!-- Bigodinhos no gatinho -->
        <path v-if="pet === 'cat'" d="M 14.8 16.2 L 10.6 15.4 M 14.8 17.4 L 10.8 18.2 M 33.2 16.2 L 37.4 15.4 M 33.2 17.4 L 37.2 18.2"
              stroke="#5c537d" stroke-width=".65" stroke-linecap="round" opacity=".8" />

        <!-- Olhos anime kawaii frontais -->
        <g class="eyes" :transform="mode === 'peek' ? 'translate(0, 1.7)' : undefined">
          <g class="eye-l">
            <ellipse cx="20.1" cy="14.2" rx="2.05" ry="2.45" fill="#1e1429" />
            <path d="M 18.6 15 A 1.55 1.35 0 0 0 21.6 15" :fill="p.irisGlow" opacity=".78" />
            <circle class="shine" cx="19.3" cy="13.1" r=".85" fill="#fff" />
            <circle class="shine" cx="20.8" cy="15.1" r=".42" fill="#fff" />
          </g>
          <path d="M 17.9 12.6 Q 20.1 10.9 22.3 12.5" fill="none" :stroke="p.stroke" stroke-width=".95" stroke-linecap="round" />

          <g class="eye-r">
            <ellipse cx="27.9" cy="14.2" rx="2.05" ry="2.45" fill="#1e1429" />
            <path d="M 26.4 15 A 1.55 1.35 0 0 0 29.4 15" :fill="p.irisGlow" opacity=".78" />
            <circle class="shine" cx="27.1" cy="13.1" r=".85" fill="#fff" />
            <circle class="shine" cx="28.6" cy="15.1" r=".42" fill="#fff" />
          </g>
          <path d="M 25.7 12.5 Q 27.9 10.9 30.1 12.6" fill="none" :stroke="p.stroke" stroke-width=".95" stroke-linecap="round" />
        </g>

        <!-- Nariz / Bico e boca ":3" -->
        <g :transform="mode === 'peek' ? 'translate(0, 1.7)' : undefined">
          <template v-if="pet === 'chicken'">
            <polygon points="22.4,15.8 25.6,15.8 24,17.8" :fill="p.nose" :stroke="p.stroke" stroke-width=".7" stroke-linejoin="round" />
            <ellipse cx="24" cy="18.7" rx=".95" ry="1.05" :fill="p.accent" :stroke="p.stroke" stroke-width=".55" />
          </template>
          <template v-else>
            <ellipse cx="24" cy="16.5" rx="1.2" ry=".82" :fill="p.nose" />
            <path d="M 22.1 17.8 Q 23 18.8 24 17.7 Q 25 18.8 25.9 17.8" fill="none" :stroke="p.stroke" stroke-width=".8" stroke-linecap="round" />
          </template>
        </g>
      </g>

      <!-- Patinhas apoiadas na borda no modo peek -->
      <g v-if="mode === 'peek'" class="peek-paws">
        <ellipse class="lpaw" cx="17.2" cy="27.4" rx="3.2" ry="2.4" :fill="pet === 'horse' ? '#3b2320' : pet === 'panda' ? p.belly : p.fur" :stroke="p.stroke" stroke-width=".85" />
        <ellipse class="rpaw" cx="30.8" cy="27.4" rx="3.2" ry="2.4" :fill="pet === 'horse' ? '#3b2320' : pet === 'panda' ? p.belly : p.fur" :stroke="p.stroke" stroke-width=".85" />
        <ellipse v-if="pet !== 'horse' && pet !== 'chicken'" cx="17.2" cy="27.8" rx="1.3" ry=".85" fill="#ff9ab8" opacity=".65" />
        <ellipse v-if="pet !== 'horse' && pet !== 'chicken'" cx="30.8" cy="27.8" rx="1.3" ry=".85" fill="#ff9ab8" opacity=".65" />
      </g>
    </g>
  </svg>
</template>

<style scoped>
.pet-svg { display: block; overflow: visible; }
.pet-svg.flip { transform: scaleX(-1); }

/* Piscar GPU-friendly */
.eye-l, .eye-r {
  transform-box: fill-box;
  transform-origin: center;
  transition: transform .08s ease;
}
.blink .eye-l, .blink .eye-r { transform: scaleY(.08); }
.blink .shine { opacity: 0; }

/* Animações suaves em GPU */
.mode-sit .body-rig, .mode-sit .front-rig { animation: breathe 2.4s ease-in-out infinite; }
.mode-play .body-rig { animation: playBob .42s ease-in-out infinite alternate; }
.head-bob { transform-box: fill-box; transform-origin: center bottom; }
.mode-walk .head-bob { animation: headWalk .48s ease-in-out infinite alternate; }
.mode-play .head-bob { animation: headPlay .42s ease-in-out infinite alternate; }

.tail { transform-box: fill-box; transform-origin: right center; animation: wag 1s ease-in-out infinite alternate; }
.mode-play .tail { animation-duration: .36s; }
.tail-sit { transform-box: fill-box; transform-origin: bottom left; animation: tailSway 2.1s ease-in-out infinite alternate; }

.paw-a { animation: stepA .45s ease-in-out infinite alternate; }
.paw-b { animation: stepB .45s ease-in-out infinite alternate; }
.mode-jump .paw-a, .mode-jump .paw-b { animation: none; transform: translateY(-2px) rotate(-14deg); }
.play-paw { transform-box: fill-box; transform-origin: top center; animation: batPaw .34s ease-in-out infinite alternate; }
.wing { transform-box: fill-box; transform-origin: left center; animation: flap .42s ease-in-out infinite alternate; }
.mode-jump .wing, .mode-play .wing { animation-duration: .22s; }

.lpaw { animation: pawTap 2.4s ease-in-out infinite; }
.rpaw { animation: pawTap 2.4s ease-in-out .35s infinite; }

.busy .head-bob { animation-duration: .34s; }

@keyframes wag { from { transform: rotate(-9deg); } to { transform: rotate(12deg); } }
@keyframes tailSway { from { transform: rotate(-4deg); } to { transform: rotate(7deg); } }
@keyframes stepA { from { transform: translateY(0) rotate(-9deg); } to { transform: translateY(-2px) rotate(11deg); } }
@keyframes stepB { from { transform: translateY(-2px) rotate(11deg); } to { transform: translateY(0) rotate(-9deg); } }
@keyframes breathe { 0%, 100% { transform: translateY(0); } 50% { transform: translateY(-1px); } }
@keyframes playBob { from { transform: translateY(0); } to { transform: translateY(-1.8px); } }
@keyframes headWalk { from { transform: translateY(0) rotate(-1.5deg); } to { transform: translateY(-.8px) rotate(1.5deg); } }
@keyframes headPlay { from { transform: rotate(-3deg); } to { transform: rotate(3deg); } }
@keyframes batPaw { from { transform: rotate(-18deg); } to { transform: rotate(22deg); } }
@keyframes flap { from { transform: rotate(-8deg); } to { transform: rotate(16deg); } }
@keyframes pawTap { 0%, 88%, 100% { transform: translateY(0); } 94% { transform: translateY(-1.5px); } }
</style>
