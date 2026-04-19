import type { Config } from "@react-router/dev/config";

// VITE_WEB_BASE_PATH es inyectado por Docker Compose como ARG/ENV.
// En local sin Docker queda como "" → se usa "/" como fallback.
const basePath = process.env.VITE_WEB_BASE_PATH || "/";

export default {
  appDirectory: "app",
  // Web runs as a client-side app; build a static client bundle only
  ssr: false,
  // Cuando home sirve en "/" el web app vive bajo "/app"
  basename: basePath || "/",
} satisfies Config;
