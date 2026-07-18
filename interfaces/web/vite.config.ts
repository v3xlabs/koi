import path from "node:path";

import tailwindcss from "@tailwindcss/vite";
import tanstackRouter from "@tanstack/router-plugin/vite";
import devtools from "solid-devtools/vite";
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";

export default defineConfig({
  plugins: [tanstackRouter({
    target: "solid",
    autoCodeSplitting: true,
    quoteStyle: "double",
  }), devtools(), solidPlugin(), tailwindcss()],
  server: {
    port: 5173,
    host: "localhost",
    proxy: {
      "/bootstrap": {
        target: "http://localhost:7777",
      },
      "/rpc": {
        target: "http://localhost:7777",
        ws: true,
        rewriteWsOrigin: true,
      },
    },
  },
  build: {
    target: "esnext",
  },
  resolve: {
    alias: {
      "#": path.resolve(__dirname, "src"),
    },
  },
});
