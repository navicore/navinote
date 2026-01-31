import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  publicDir: 'static',
  server: {
    proxy: {
      '/api': 'http://localhost:8080'
    }
  }
});
