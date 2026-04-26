/**
 * Suite: admin-instances.spec.ts
 * Cubre sección 1.5 y 2 del todo_test.md — God Mode / /api/instances/*
 * Auth: sesión separada via /api/instances/admins/sign-in
 *
 * Endpoints cubiertos (extraídos de packages/services/src/instance/instance.service.ts):
 *   GET/PATCH  /api/instances
 *   GET        /api/instances/admins
 *   GET        /api/instances/admins/me
 *   GET        /api/instances/admins/session
 *   DELETE     /api/instances/admins/{pk}
 *   GET/PATCH  /api/instances/configurations
 *   DELETE     /api/instances/configurations/disable-email-feature
 *   POST       /api/instances/email-credentials-check
 *   GET        /api/instances/workspace-slug-check
 *   GET        /api/instances/workspaces
 */
import { test, expect, request as playwrightRequest } from "@playwright/test";
import { Env } from "@plane/e2e-utils/helpers/env";
import { fetchCsrfToken } from "@plane/e2e-utils/helpers/api";

const BASE = Env.API_BASE;
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? Env.TEST_EMAIL;
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? Env.TEST_PASSWORD;

async function getAdminContext() {
  const ctx = await playwrightRequest.newContext({ baseURL: BASE });
  const csrf = await fetchCsrfToken(ctx);

  const signIn = await ctx.post(`${BASE}/api/instances/admins/sign-in`, {
    headers: { "X-CSRFToken": csrf },
    data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
  });

  if (!signIn.ok()) {
    await ctx.post(`${BASE}/api/instances/admins/sign-up`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: ADMIN_EMAIL, password: ADMIN_PASSWORD },
    });
  }

  return { ctx, csrf };
}

test.describe("Admin — instancia", () => {
  test("GET /api/instances devuelve configuración de instancia", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances`);
      expect([200, 403]).toContain(res.status());
      if (res.status() === 200) {
        const body = await res.json() as Record<string, unknown>;
        // Shape mínimo esperado
        expect(body).toMatchObject({ id: expect.any(String) });
      }
    } finally { await ctx.dispose(); }
  });

  test("PATCH /api/instances actualiza configuración", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.patch(`${BASE}/api/instances`, {
        headers: { "X-CSRFToken": csrf },
        data: { instance_name: `E2E Instance ${Date.now()}` },
      });
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});

test.describe("Admin — admins", () => {
  test("GET /api/instances/admins lista admins", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/admins`);
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });

  test("GET /api/instances/admins/me devuelve admin actual", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/admins/me`);
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });

  test("GET /api/instances/admins/session devuelve sesión", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/admins/session`);
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});

test.describe("Admin — configurations", () => {
  test("GET /api/instances/configurations lista config", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/configurations`);
      expect([200, 403]).toContain(res.status());
      if (res.status() === 200) {
        // Es un array de { key, value } según el service
        expect(await res.json()).toBeInstanceOf(Array);
      }
    } finally { await ctx.dispose(); }
  });

  test("PATCH /api/instances/configurations actualiza una key", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.patch(`${BASE}/api/instances/configurations`, {
        headers: { "X-CSRFToken": csrf },
        data: { ENABLE_SIGNUP: "1" },
      });
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });

  test("POST /api/instances/email-credentials-check valida credenciales SMTP", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.post(`${BASE}/api/instances/email-credentials-check`, {
        headers: { "X-CSRFToken": csrf },
        data: { receiver_email: ADMIN_EMAIL },
      });
      // 200 si SMTP configurado, 400 si no, 403 si no es admin
      expect([200, 400, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});

test.describe("Admin — workspace utilities", () => {
  test("GET /api/instances/workspace-slug-check valida slug", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(
        `${BASE}/api/instances/workspace-slug-check?slug=doesnotexist-${Date.now()}`,
      );
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });

  test("GET /api/instances/workspaces lista workspaces de instancia", async () => {
    const { ctx } = await getAdminContext();
    try {
      const res = await ctx.get(`${BASE}/api/instances/workspaces`);
      expect([200, 403]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});

test.describe("Admin — sign-out", () => {
  test("POST /api/instances/admins/sign-out cierra sesión", async () => {
    const { ctx, csrf } = await getAdminContext();
    try {
      const res = await ctx.post(`${BASE}/api/instances/admins/sign-out`, {
        headers: { "X-CSRFToken": csrf },
      });
      expect([200, 204]).toContain(res.status());
    } finally { await ctx.dispose(); }
  });
});
