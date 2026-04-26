/**
 * Suite: permissions.spec.ts
 * Matrix simplificada: endpoints protegidos devuelven 401/403 sin sesión.
 * Roles completos requieren múltiples usuarios — se valida el caso sin auth.
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";
import { request as playwrightRequest } from "@playwright/test";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

// Endpoints que requieren autenticación
const PROTECTED_ENDPOINTS = [
  `/api/users/me`,
  `/api/workspaces`,
  `/api/workspaces/${slug()}/projects`,
  `/api/workspaces/${slug()}/projects/${pid()}/issues`,
  `/api/workspaces/${slug()}/webhooks`,
  `/api/api-tokens`,
];

test.describe("Permissions — sin autenticación (401)", () => {
  test("Todos los endpoints protegidos devuelven 401 sin sesión", async () => {
    const ctx = await playwrightRequest.newContext(); // sin cookies
    for (const endpoint of PROTECTED_ENDPOINTS) {
      const res = await ctx.get(`${BASE}${endpoint}`);
      expect(res.status(), `Expected 401 for ${endpoint}`).toBe(401);
    }
    await ctx.dispose();
  });
});

test.describe("Permissions — CSRF requerido en mutaciones", () => {
  test("POST sin X-CSRFToken devuelve 403", async ({ request }) => {
    // Sin header CSRF
    const res = await request.post(`${BASE}/api/workspaces`, {
      data: { name: "test", slug: "test" },
      // Sin headers CSRF
    });
    expect([403, 400]).toContain(res.status());
  });
});

test.describe("Permissions — rol de workspace", () => {
  test("GET workspace-members/me devuelve rol numérico válido", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/workspace-members/me`,
    );
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    // Roles válidos: 5 (Guest), 10 (Viewer), 15 (Member), 20 (Admin)
    expect([5, 10, 15, 20]).toContain(body.role);
  });
});
