import path from "node:path";
import * as dotenv from "dotenv";
import { reactRouter } from "@react-router/dev/vite";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

dotenv.config({ path: path.resolve(__dirname, ".env") });

// Expose only vars starting with VITE_
const viteEnv = Object.keys(process.env)
  .filter((k) => k.startsWith("VITE_"))
  .reduce<Record<string, string>>((a, k) => {
    a[k] = process.env[k] ?? "";
    return a;
  }, {});

// `base` debe coincidir con VITE_WEB_BASE_PATH ("/app") para que todos los
// assets de Vite (CSS, JS, /node_modules/.vite/deps/…) se sirvan bajo /app/*
// y coincidan con la regla PathPrefix(`/app`) de Traefik.
//
// Sin este `base`, los assets se generan con rutas raíz (/styles/globals.css,
// /node_modules/.vite/…) que Traefik no enruta al contenedor web → 404 →
// pantalla negra tras login.
//
// Con base: '/app':
//   · Vite sirve /app/styles/globals.css, /app/node_modules/.vite/deps/…
//   · Traefik matchea PathPrefix(`/app`) → forward correcto al dev server ✓
//   · Vite internamente stripea el prefijo /app y sirve el archivo ✓
//
// Nota: el StripPrefix que mencionaba el comentario anterior NUNCA se definió
// en los labels de Traefik del docker-compose-dev.yml, por eso fallaba.
const webBasePath = process.env.VITE_WEB_BASE_PATH || "/app";

export default defineConfig(() => ({
  base: webBasePath,
  define: {
    "process.env": JSON.stringify(viteEnv),
  },
  build: {
    assetsInlineLimit: 0,
  },
  plugins: [reactRouter(), tsconfigPaths({ projects: [path.resolve(__dirname, "tsconfig.json")] })],
  resolve: {
    alias: {
      // Next.js compatibility shims used within web
      "next/link": path.resolve(__dirname, "app/compat/next/link.tsx"),
      "next/navigation": path.resolve(__dirname, "app/compat/next/navigation.ts"),
      "next/script": path.resolve(__dirname, "app/compat/next/script.tsx"),
    },
    dedupe: ["react", "react-dom", "@headlessui/react"],
  },
  server: {
    allowedHosts: true as const,
    host: "0.0.0.0",
  },
  // No SSR-specific overrides needed; alias resolves to ESM build
}));
