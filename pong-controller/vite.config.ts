import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import UnoCSS from 'unocss/vite'
import fs from 'fs'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    UnoCSS()
  ],
  server: {
    https: {
      key: fs.readFileSync('./keys/server.key'),
      cert: fs.readFileSync('./keys/server.crt'),
    },
    host: 'dev.local',
    port: 5173,
  }
})
