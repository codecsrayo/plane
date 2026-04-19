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

// NO se configura `base` aqui intencionalmente.
//
// Problema: fijar base: '/app' en Vite choca con appDirectory: "app" en
// react-router.config.ts. El plugin @react-router/dev/vite v7 evalua
// virtual:react-router/server-build en el dev server (incluso con ssr:false)
// importando "/app/root.tsx". Con base='/app', Vite hace strip del prefijo
// '/app' y busca "/root.tsx" -> no existe -> pantalla de error.
//
// Solucion adoptada (ver docker-compose-dev.yml):
//   1. Traefik aplica StripPrefix("/app") antes de hacer forward al dev server
//      -> Vite recibe "/" y sirve assets con rutas raiz (/styles/globals.css…)
//   2. Un segundo router de baja prioridad enruta los paths de assets de Vite
//      (/@, /__, /styles, /core, /node_modules, etc.) al mismo contenedor web.
//
// El basename "/app" vive SOLO en react-router.config.ts (routing del browser).

export default defineConfig(() => ({
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
