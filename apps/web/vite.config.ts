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

// Nota: NO configuramos `base` aquÃ­.
// Traefik aplica StripPrefix("/app") antes de hacer forward al dev server,
// por lo que Vite siempre recibe paths que empiezan en "/".
// El basename "/app" vive solo en react-router.config.ts (routing del browser).

const webBasePath = process.env.VITE_WEB_BASE_PATH || "/";

export default defineConfig(() => ({
  // base es el prefijo de todas las URLs de assets generados por Vite.
  // En dev (sin strip): Vite recibe /app/... y sirve assets en /app/assets/...
  // En prod (con strip nginx): assets en /app/assets/... → strip → /assets/...
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
