import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    // Cargo's workspace output sits below Vite's root. Windows locks its
    // running executables/PDBs; only Tauri needs to watch the Rust sources.
    watch: { ignored: ["**/target/**", "**/src-tauri/**"] },
  },
});
