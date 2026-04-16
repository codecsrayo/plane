import path from "node:path";
import * as dotenv from "dotenv";
import { reactRouter } from "@react-router/dev/vite";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";
import { joinUrlPath } from "@plane/utils";

dotenv.config({ path: path.resolve(__dirname, ".env") });

// Expose only vars starting with VITE_
const viteEnv = Object.keys(process.env)
  .filter((k) => k.startsWith("VITE_"))
  .reduce<Record<string, string>>((a, k) => {
    a[k] = process.env[k] ?? "";
    return a;
  }, {});

const basePath = joinUrlPath(process.env.VITE_ADMIN_BASE_PATH ?? "", "/") ?? "/";

// Dependencias descubiertas que Vite re-optimiza on-the-fly durante navegación,
// lo que dispara un full reload y deja pendientes requests al hash previo
// (reproducido como 504 Gateway Timeout cuando el admin corre detrás de un
// reverse proxy con timeouts agresivos). Pre-declararlas fuerza un único
// pre-bundle al arranque, evitando el reload y los hashes stale.
// Fuente de la lista: output de Vite "new dependencies optimized: ..." en
// docker-compose-dev.yml del servicio `admin`.
const optimizeDepsInclude = [
  "next-themes",
  "swr",
  "mobx-react",
  "mobx",
  "@bprogress/core",
  "lucide-react",
  "lodash-es",
  "uuid",
  "axios",
  "file-type",
  "react-popper",
  "@headlessui/react",
  "react-color",
  "@radix-ui/react-scroll-area",
  "@atlaskit/pragmatic-drag-and-drop/dist/cjs/entry-point/element/adapter.js",
  "@atlaskit/pragmatic-drag-and-drop/dist/cjs/entry-point/combine.js",
  "@atlaskit/pragmatic-drag-and-drop-hitbox/dist/cjs/closest-edge.js",
  "@blueprintjs/popover2",
  "date-fns",
  "date-fns/differenceInCalendarDays",
  "clsx",
  "tailwind-merge",
  "rehype-parse",
  "rehype-remark",
  "remark-gfm",
  "remark-stringify",
  "unified",
  "sanitize-html",
  "chroma-js",
  "class-variance-authority",
  "@base-ui-components/react/tooltip",
  "@base-ui-components/react/toast",
];

export default defineConfig(() => ({
  base: basePath,
  define: {
    "process.env": JSON.stringify(viteEnv),
  },
  build: {
    assetsInlineLimit: 0,
  },
  plugins: [reactRouter(), tsconfigPaths({ projects: [path.resolve(__dirname, "tsconfig.json")] })],
  resolve: {
    alias: {
      // Next.js compatibility shims used within admin
      "next/link": path.resolve(__dirname, "app/compat/next/link.tsx"),
      "next/navigation": path.resolve(__dirname, "app/compat/next/navigation.ts"),
    },
    dedupe: ["react", "react-dom"],
  },
  optimizeDeps: {
    include: optimizeDepsInclude,
  },
  server: {
    allowedHosts: true as const,
    host: "0.0.0.0",
  },
  // No SSR-specific overrides needed; alias resolves to ESM build
}));
