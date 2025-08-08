/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  // Set default environment variables based on mode
  const defaultEnv = {
    development: {
      VITE_API_URL: 'http://localhost:3000',
      VITE_PORT: '5173',
      VITE_HOST: 'localhost',
    },
    production: {
      VITE_API_URL: 'https://docs.islahlabs.com',
    }
  };

  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: {
        "@": "/src",
      },
    },
    server: {
      port: parseInt(process.env.VITE_PORT || defaultEnv.development.VITE_PORT),
      host: process.env.VITE_HOST || defaultEnv.development.VITE_HOST,
    },
    define: {
      // Ensure environment variables are available at build time
      'import.meta.env.VITE_API_URL': JSON.stringify(
        process.env.VITE_API_URL || 
        (mode === 'production' ? defaultEnv.production.VITE_API_URL : defaultEnv.development.VITE_API_URL)
      ),
    },
    test: {
      globals: true,
      environment: 'jsdom',
      setupFiles: ['./src/test/setup.ts'],
    },
  }
})
