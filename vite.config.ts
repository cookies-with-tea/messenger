import { fileURLToPath, URL } from "node:url";
import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");

  return {
    plugins: [vue(), tailwindcss()],

    resolve: {
      alias: {
        "@": fileURLToPath(new URL("./src", import.meta.url)),
      },
    },

    server: {
      host: true,
      proxy: {
        '/api': {
          target: env.VITE_BASE_REST_URL,
          changeOrigin: true,
        },

        '/ws': {
          target: env.VITE_BASE_WS_URL,
          ws: true,
          changeOrigin: true,
        },
      },
    },

    build: {
      outDir: "../dist",
    },
  };
});
