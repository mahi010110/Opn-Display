import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Tauri attend le serveur de développement sur un port fixe.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: { target: 'es2021' },
})
