<script setup lang="ts">
// Ícones (traço) das abas
defineProps<{ name: string }>()
const PATHS: Record<string, string> = {
  home: '<path d="M4 11l8-7 8 7v9h-5v-6H9v6H4z"/>',
  actions: '<path d="M13 3L5 14h6l-1 7 8-11h-6z"/>',
  clips: '<rect x="8" y="3" width="11" height="14" rx="2"/><path d="M5 7v12a2 2 0 0 0 2 2h9"/>',
  notes: '<path d="M6 3h9l4 4v14H6z"/><path d="M9 11h7M9 15h7"/>',
  timer: '<circle cx="12" cy="13" r="8"/><path d="M12 9v4l3 2M9 2h6"/>',
  cal: '<rect x="4" y="5" width="16" height="16" rx="2"/><path d="M4 10h16M9 3v4M15 3v4"/>',
  weather: '<path d="M7 18h10a4 4 0 0 0 0-8 6 6 0 0 0-11.5 1.5A3.3 3.3 0 0 0 7 18z"/>',
  audio: '<path d="M4 14v-2a8 8 0 0 1 16 0v2"/><rect x="3" y="14" width="4" height="6" rx="1.5"/><rect x="17" y="14" width="4" height="6" rx="1.5"/>',
  sys: '<rect x="7" y="7" width="10" height="10" rx="1"/><path d="M10 3v4M14 3v4M10 17v4M14 17v4M3 10h4M3 14h4M17 10h4M17 14h4"/>',
  clean: '<path d="M19 4l-7 7"/><path d="M12.5 10.5l1 1c.8.8.8 2 0 2.8L9 19H4v-5l4.5-4.5c.8-.8 2-.8 2.8 0z"/><path d="M7 15l2 2"/>',
  procs: '<path d="M4 6h16M4 12h16M4 18h10"/><circle cx="19" cy="18" r="1.5"/>',
  work: '<rect x="3" y="7" width="18" height="13" rx="2"/><path d="M9 7V5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2M3 13h18"/>',
  games: '<path d="M6 11h4M8 9v4M15 12h.01M18 10h.01"/><path d="M17.3 5H6.7a4 4 0 0 0-3.95 3.4L2 14.6A2.6 2.6 0 0 0 6.6 16.6L9 14h6l2.4 2.6a2.6 2.6 0 0 0 4.6-2L21.25 8.4A4 4 0 0 0 17.3 5z"/>',
  tools: '<path d="M14.7 6.3a4 4 0 0 0-5.4 5.4L4 17l3 3 5.3-5.3a4 4 0 0 0 5.4-5.4l-2.6 2.6-2.4-.6-.6-2.4z"/>',
  snippets: '<path d="M8 6l-5 6 5 6M16 6l5 6-5 6M13.5 4l-3 16"/>',
  plus: '<path d="M12 5v14M5 12h14"/>'
}
</script>

<template>
  <svg class="ico" viewBox="0 0 24 24" v-html="PATHS[name]" />
</template>
