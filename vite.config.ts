import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { resolve } from "path";
import DefineOptions from "unplugin-vue-define-options/vite";

export default defineConfig({
  plugins: [vue(), DefineOptions()],
  // Tauri 推荐配置
  clearScreen: false,
  server: {
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: ["es2021", "chrome100", "safari13"],
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  resolve: {
    alias: {
      "@": resolve(__dirname, "src"),
    },
  },
});
