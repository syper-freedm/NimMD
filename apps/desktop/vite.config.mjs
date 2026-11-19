import { fileURLToPath } from "node:url";
import { searchForWorkspaceRoot } from "vite";
import { defineConfig } from "vitest/config";

export default defineConfig({
  clearScreen: false,
  server: {
    host: "127.0.0.1", port: 5173, strictPort: true,
    fs: {
      allow: [searchForWorkspaceRoot(import.meta.dirname), fileURLToPath(new URL("../../packages/theme", import.meta.url))],
    },
  },
  build: { rolldownOptions: { input: ["index.html", "view.html"] } },
  test: { environment: "jsdom", include: ["src/**/*.test.ts"] },
});
