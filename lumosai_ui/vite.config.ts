import react from '@vitejs/plugin-react';
import { resolve } from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [react()],
  server: {
    host: true,
    port: 3000,
    open: true
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
      '@lumosai/client-js': resolve(__dirname, '../packages/client-js/src'),
      '@lumosai/ui-components': resolve(__dirname, '../packages/ui-components/src')
    },
  },
  css: {
    postcss: './postcss.config.mjs',
  },
  clearScreen: false
});
