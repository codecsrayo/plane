/**
 * Suite: users.spec.ts
 * Cubre sección 3 del todo_test.md: /api/users/me/*
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;

test.describe("Users — me", () => {
  test("GET /api/users/me devuelve usuario autenticado", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({
      id: expect.any(String),
      email: expect.any(String),
    });
  });

  test("GET /api/users/session devuelve sesión activa", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/session`);
    expect(res.status()).toBe(200);
  });

  test("GET /api/users/me/settings devuelve configuración", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/settings`);
    expect(res.status()).toBe(200);
  });

  test("PATCH /api/users/me actualiza display_name", async ({ request, csrf }) => {
    const newName = `E2E User ${Date.now()}`;
    const res = await request.patch(`${BASE}/api/users/me`, {
      headers: { "X-CSRFToken": csrf },
      data: { display_name: newName },
    });
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ display_name: newName });
  });

  test("GET /api/users/me/profile devuelve perfil", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/profile`);
    expect(res.status()).toBe(200);
  });

  test("GET /api/users/me/accounts devuelve lista de cuentas", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/accounts`);
    expect(res.status()).toBe(200);
    expect(await res.json()).toBeInstanceOf(Array);
  });

  test("GET /api/users/me/notification-preferences — get_or_create nunca 404", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/notification-preferences`);
    expect(res.status()).toBe(200);
  });

  test("GET /api/users/me/workspaces devuelve lista de workspaces", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/workspaces`);
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(body.length).toBeGreaterThan(0);
  });

  test("GET /api/users/last-visited-workspace devuelve workspace", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/last-visited-workspace`);
    expect([200, 404]).toContain(res.status());
  });

  test("GET /api/users/me/activities paginación por cursor", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/activities`);
    expect(res.status()).toBe(200);
  });

  test("GET /api/users/me/workspaces/invitations lista invitaciones", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/me/workspaces/invitations`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Users — sin autenticación", () => {
  test("GET /api/users/me sin sesión devuelve 401", async () => {
    // Contexto limpio sin cookies
    const { request: playwrightRequest } = await import("@playwright/test");
    const ctx = await playwrightRequest.newContext();
    const res = await ctx.get(`${BASE}/api/users/me`);
    expect(res.status()).toBe(401);
    await ctx.dispose();
  });
});
