import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 5173, strictPort: true, watch: { ignored: ['**/src-tauri/**', '**/*.gurps.json', '**/*.tmp'] } },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: { target: 'es2021', outDir: 'dist', emptyOutDir: true },
});
