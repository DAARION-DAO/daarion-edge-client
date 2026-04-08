import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { VitePWA } from "vite-plugin-pwa";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig(async () => ({
  plugins: [
    react(),
    VitePWA({
      // Use 'autoUpdate' so the SW updates silently in the background
      registerType: "autoUpdate",
      // Include generated assets in the manifest
      includeAssets: ["icon-192.png", "icon-512.png", "favicon.ico", "fallback_registry.json"],

      manifest: {
        name: "DAARION Edge",
        short_name: "DAARION",
        description:
          "Sovereign Edge Client — суверенний вузол мережі DAARION. Онбординг агентів та протокол Голосової Церемонії.",
        start_url: "/",
        scope: "/",
        display: "standalone",
        orientation: "portrait-primary",
        theme_color: "#020202",
        background_color: "#020202",
        lang: "uk",
        categories: ["productivity", "utilities"],
        icons: [
          {
            src: "/icon-192.png",
            sizes: "192x192",
            type: "image/png",
            purpose: "any maskable",
          },
          {
            src: "/icon-512.png",
            sizes: "512x512",
            type: "image/png",
            purpose: "any maskable",
          },
        ],
        shortcuts: [
          {
            name: "Sovereign Genesis",
            short_name: "Genesis",
            description: "Запустити онбординг нового агента",
            url: "/?launch=genesis",
            icons: [{ src: "/icon-192.png", sizes: "192x192" }],
          },
        ],
      },
      // Workbox strategy: network-first for API, cache-first for assets
      workbox: {
        globPatterns: ["**/*.{js,css,html,ico,png,svg,woff2,json}"],
        runtimeCaching: [
          {
            // Cache Genesis API responses briefly
            urlPattern: /^https:\/\/api\.daarion\.city\/genesis\/.*/i,
            handler: "NetworkFirst",
            options: {
              cacheName: "genesis-api-cache",
              expiration: { maxEntries: 20, maxAgeSeconds: 60 },
            },
          },
        ],
      },
      // Suppress Tauri-specific warnings — vite-plugin-pwa is PWA-only
      devOptions: {
        enabled: false, // disable SW in tauri dev to avoid conflicts
      },
    }),
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent Vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));

