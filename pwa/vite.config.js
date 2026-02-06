import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const strict = !!process.env.SVELTE_STRICT;
const svelteWarnings = [];

export default defineConfig({
  plugins: [
    svelte({
      onwarn(warning, handler) {
        if (strict) {
          svelteWarnings.push(warning);
        }
        handler(warning);
      }
    }),
    strict && {
      name: 'svelte-strict',
      buildEnd() {
        if (svelteWarnings.length > 0) {
          const msgs = svelteWarnings.map(w => `  ${w.code}: ${w.message}`).join('\n');
          this.error(`${svelteWarnings.length} Svelte warning(s):\n${msgs}`);
        }
      }
    }
  ].filter(Boolean),
  publicDir: 'static',
  server: {
    proxy: {
      '/api': 'http://localhost:8080'
    }
  }
});
