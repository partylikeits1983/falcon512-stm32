import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    fs: {
      allow: [".."],
    },
  },
  optimizeDeps: {
    exclude: ["falcon_wasm"],
  },
  build: {
    target: "esnext",
    rollupOptions: {
      external: [],
      output: {
        manualChunks: undefined,
      },
    },
  },
  worker: {
    format: "es",
  },
  assetsInclude: ["**/*.wasm"],
});
