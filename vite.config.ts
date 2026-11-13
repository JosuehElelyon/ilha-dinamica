import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: 'chrome105',
    sourcemap: false
  }
})
