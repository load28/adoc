import tailwindcss from "@tailwindcss/vite";
import { tanstackStart } from "@tanstack/react-start/plugin/vite";
import viteReact from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig(({ command }) => ({
  build: {
    assetsInlineLimit: 0,
  },
  server: {
    port: 3000,
  },
  resolve: {
    tsconfigPaths: true,
  },
  ...(command === "build" ? { ssr: { noExternal: true } } : {}),
  plugins: [tanstackStart(), tailwindcss(), viteReact()],
}));
