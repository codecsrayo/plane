/** Variables de entorno compartidas entre todas las apps E2E. */
export const Env = {
  API_BASE: process.env.API_BASE_URL ?? "http://localhost:8000",
  TEST_EMAIL: process.env.E2E_EMAIL ?? `e2e-${Date.now()}@plane-test.local`,
  TEST_PASSWORD: process.env.E2E_PASSWORD ?? "PlaneE2E!2024",
  WORKSPACE_SLUG: process.env.E2E_WORKSPACE_SLUG ?? "",
  PROJECT_ID: process.env.E2E_PROJECT_ID ?? "",
  STATE_IDS: process.env.E2E_STATE_IDS ? JSON.parse(process.env.E2E_STATE_IDS) as string[] : [],
  API_TOKEN: process.env.E2E_API_TOKEN ?? "",
} as const;

/** Asegura que las variables críticas estén seteadas (llamar al inicio de specs). */
export function assertEnv(...keys: (keyof typeof Env)[]): void {
  for (const k of keys) {
    if (!Env[k]) throw new Error(`Missing required env: ${k}`);
  }
}
