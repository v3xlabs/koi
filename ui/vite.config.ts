import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';
import solidPlugin from 'vite-plugin-solid';
import devtools from 'solid-devtools/vite';
import path from "node:path";
import tanstackRouter from "@tanstack/router-plugin/vite";

export default defineConfig({
  plugins: [tanstackRouter({
    target: "solid",
    autoCodeSplitting: true,
    quoteStyle: "double",
  }), devtools(), solidPlugin(), tailwindcss()],
  server: {
    port: 5173,
    host: "127.0.0.1",
    proxy: {
      "/api": {
        target: "http://0.0.0.0:7777",
        changeOrigin: true,
      },
    },
  },
  build: {
    target: 'esnext',
  },
  resolve: {
    alias: {
      "#": path.resolve(__dirname, "src"),
    }
  }
});
