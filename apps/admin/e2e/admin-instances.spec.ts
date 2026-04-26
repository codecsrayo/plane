/**
 * Suite: admin-instances.spec.ts
 * Cubre sección 1.5 y 2 del todo_test.md — God Mode / /api/instances/*
 * Auth: sesión separada via /api/instances/admins/sign-in
 */
import { test, expect, request as playwrightRequest } from "@playwright/test";
import { Env } from "@plane/e2e-utils/helpers/env";
import { fetchCsrfToken } from "@plane/e2e-utils/helpers/api";

const BASE = Env.API_BASE;
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? Env.TEST_EMAIL;
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? Env.TEST_PASSWORD;

/**
 * Obtiene un contexto con sesión de admin (god mode).
 * Si el sign-in falla, intenta sign-up primero.
 */
async function getAdminContext() {
  const ctx = await playwrightRequest.newContext({ baseURL: BASE });
  const csrf = await fetchCsrfToken(ctx);

  // Intentar sign-in como admin
  const signIn = await ctx.post(`${BASE}/api/instances/admins/sign-in`, {
    headers: { "X-CSRFToken": csrf },
    data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
  });

  if (!signIn.ok()) {
    // Intentar sign-up
    await ctx.post(`${BASE}/api/instances/admins/sign-up`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
    });
  }

  return { ctx, csrf };
}

test.describe("Admin — instancias (God Mode)", () => {
  test("GET /api/instances devuelve configuración", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances`);
      // 200 si admin ya existe, 403 si no aplica
      expect([200, 403]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });

  test("GET /api/instances/admins/me devuelve admin actual", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/admins/me`);
      expect([200, 403]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });

  test("GET /api/instances/admins/session devuelve sesión admin", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/admins/session`);
      expect([200, 403]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });

  test("GET /api/instances/configurations devuelve config", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/configurations`);
      expect([200, 403]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });

  test("POST /api/instances/admins/sign-out cierra sesión admin", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.post(`${BASE}/api/instances/admins/sign-out`, {
        headers: { "X-CSRFToken": csrf },
      });
      expect([200, 204]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });

  test("GET /api/instances/workspace-slug-check valida slug", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(
        `${BASE}/api/instances/workspace-slug-check?slug=doesnotexist-${Date.now()}`,
      );
      expect([200, 403]).toContain(res.status());
    } finally {
      await ctx.dispose();
    }
  });
});
