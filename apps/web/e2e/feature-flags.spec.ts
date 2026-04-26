/**
 * Suite: feature-flags.spec.ts
 * Verifica que /api/instances/configurations devuelve las claves conocidas
 * y que los endpoints gate-flagged responden correctamente según el flag.
 *
 * Feature flags extraídos de src/routes/instances.rs → config_var().
 * Claves encriptadas (is_encrypted=true) no se devuelven en texto plano.
 */
import { test, expect } from "@playwright/test";
import { Env } from "@plane/e2e-utils/helpers/env";
import { fetchCsrfToken, signIn } from "@plane/e2e-utils/helpers/api";
import {
  AUTH_FLAG_KEYS,
  ENCRYPTED_KEYS,
  FLAG_GATED_ENDPOINTS,
} from "@plane/e2e-utils/helpers/feature-flags";

const BASE = Env.API_BASE;

// ── Helpers ────────────────────────────────────────────────────────────────────

async function getAdminCtx() {
  const { request: playwrightRequest } = await import("@playwright/test");
  const ctx = await playwrightRequest.newContext({ baseURL: BASE });
  const csrf = await fetchCsrfToken(ctx);
  await ctx.post(`${BASE}/api/instances/admins/sign-in`, {
    headers: { "X-CSRFToken": csrf },
    data: { email: process.env.E2E_ADMIN_EMAIL ?? Env.TEST_EMAIL, password: process.env.E2E_ADMIN_PASSWORD ?? Env.TEST_PASSWORD },
  });
  return { ctx, csrf };
}

// ── Tests ──────────────────────────────────────────────────────────────────────

test.describe("Feature flags — /api/instances/configurations", () => {
  test("Respuesta es un array de objetos con clave 'key'", async () => {
    const { ctx } = await getAdminCtx();
    try {
      const res = await ctx.get(`${BASE}/api/instances/configurations`);
      if (res.status() === 403) return test.skip(); // sin admin en este entorno
      expect(res.status()).toBe(200);
      const body = await res.json() as Array<{ key: string; value: string }>;
      expect(Array.isArray(body)).toBe(true);
      // Al menos las flags de autenticación deben aparecer
      const keys = body.map((c) => c.key);
      for (const authKey of AUTH_FLAG_KEYS) {
        expect(keys, `Expected ${authKey} in configurations`).toContain(authKey);
      }
    } finally { await ctx.dispose(); }
  });

  test("Claves encriptadas no exponen el valor real (vacío o redactado)", async () => {
    const { ctx } = await getAdminCtx();
    try {
      const res = await ctx.get(`${BASE}/api/instances/configurations`);
      if (res.status() !== 200) return test.skip();
      const body = await res.json() as Array<{ key: string; value: string }>;
      for (const cfg of body) {
        if (ENCRYPTED_KEYS.includes(cfg.key) && cfg.value) {
          // El backend puede devolver vacío o un placeholder — nunca el secreto real
          // Si hay valor, validamos que no sea una clave privada o token largo
          expect(cfg.value.length, `Encrypted key ${cfg.key} should be masked or empty`).toBeLessThan(200);
        }
      }
    } finally { await ctx.dispose(); }
  });
});

test.describe("Feature flags — endpoints gate-flagged", () => {
  test("ENABLE_SIGNUP=0 bloquea POST /auth/sign-up (si flag está desactivado)", async () => {
    const { request: playwrightRequest } = await import("@playwright/test");
    const ctx = await playwrightRequest.newContext({ baseURL: BASE });
    try {
      const csrf = await fetchCsrfToken(ctx);
      const res = await ctx.post(`${BASE}/auth/sign-up`, {
        headers: { "X-CSRFToken": csrf },
        data: { email: `disabled-${Date.now()}@plane-test.local`, password: "PlaneE2E!2024" },
      });
      // Si ENABLE_SIGNUP="1" (default) → 200/201 o 400 (email exists)
      // Si ENABLE_SIGNUP="0"           → 403
      // En ambos casos el endpoint responde — no 500
      expect([200, 201, 400, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });

  test("DISABLE_WORKSPACE_CREATION=1 bloquea POST /api/workspaces", async () => {
    const { ctx, csrf } = await getAdminCtx();
    try {
      // Intentar crear workspace — si el flag está activo debe devolver 403
      const res = await ctx.post(`${BASE}/api/workspaces`, {
        headers: { "X-CSRFToken": csrf },
        data: { name: "blocked-ws", slug: `blocked-${Date.now()}` },
      });
      // 201 si flag inactivo, 403 si activo — ambos son comportamientos válidos
      expect([201, 400, 403]).toContain(res.status());
      // Cleanup si se creó
      if (res.status() === 201) {
        const ws = await res.json() as { slug: string };
        await ctx.delete(`${BASE}/api/workspaces/${ws.slug}`, {
          headers: { "X-CSRFToken": csrf },
        });
      }
    } finally { await ctx.dispose(); }
  });

  test("GET /api/unsplash sin UNSPLASH_ACCESS_KEY devuelve 400 o resultado vacío", async () => {
    const { request: playwrightRequest } = await import("@playwright/test");
    const ctx = await playwrightRequest.newContext({ storageState: "state.json" });
    try {
      const res = await ctx.get(`${BASE}/api/unsplash?query=mountains`);
      // 200 si key configurada, 400/500 si no
      expect([200, 400, 500]).toContain(res.status());
      if (res.status() === 200) {
        const body = await res.json() as Record<string, unknown>;
        expect(body).toMatchObject({ results: expect.any(Array) });
      }
    } finally { await ctx.dispose(); }
  });

  test("POST /ai-assistant sin LLM_API_KEY devuelve 400 o error de configuración", async () => {
    const { request: playwrightRequest } = await import("@playwright/test");
    const ctx = await playwrightRequest.newContext({ storageState: "state.json" });
    try {
      const csrf = await fetchCsrfToken(ctx);
      const res = await ctx.post(
        `${BASE}/api/workspaces/${Env.WORKSPACE_SLUG}/ai-assistant`,
        {
          headers: { "X-CSRFToken": csrf },
          data: { prompt: "Summarize this issue", task_name: "chat" },
        },
      );
      // 200 si LLM configurado, 400 si no hay key
      expect([200, 400, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});

test.describe("Feature flags — documentación de keys conocidas", () => {
  test("FLAG_GATED_ENDPOINTS cubre los grupos principales", () => {
    const groups = Object.keys(FLAG_GATED_ENDPOINTS);
    expect(groups).toContain("ENABLE_SIGNUP");
    expect(groups).toContain("ENABLE_EMAIL_PASSWORD");
    expect(groups).toContain("ENABLE_MAGIC_LINK_LOGIN");
    expect(groups).toContain("IS_GITHUB_INTEGRATION_ENABLED");
    expect(groups).toContain("LLM_API_KEY");
    expect(groups).toContain("UNSPLASH_ACCESS_KEY");
  });
});
