import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

// Two pages: the kiosk (index.html) and the staff admin page (admin/index.html).
// `npm run dev` serves them with live reload and passes /api calls on to the
// Rust server, which must be running on port 8080.
export default defineConfig({
  plugins: [svelte()],
  build: {
    rollupOptions: {
      input: { kiosk: 'index.html', admin: 'admin/index.html' },
    },
  },
  server: {
    proxy: { '/api': 'http://localhost:8080' },
  },
});
