import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  base: process.env.GITHUB_PAGES ? '/file-forge/' : '/',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
});
