import { defineConfig } from "vite";

// M6.2: devUrl in ../crates/xoft-testbed/tauri.conf.json must match this port.
export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
});
