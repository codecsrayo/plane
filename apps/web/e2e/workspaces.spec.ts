/**
 * Suite: workspaces.spec.ts
 * Cubre sección 4 del todo_test.md
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

test.describe("Workspaces — CRUD", () => {
  test("GET /api/workspaces lista workspaces del usuario", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces`);
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(body.length).toBeGreaterThan(0);
  });

  test("GET /api/workspaces/{slug} devuelve workspace de seed", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ slug: slug() });
  });

  test("PATCH /api/workspaces/{slug} actualiza nombre", async ({ request, csrf }) => {
    const newName = `E2E WS ${Date.now()}`;
    const res = await request.patch(`${BASE}/api/workspaces/${slug()}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: newName },
    });
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ name: newName });
  });

  test("GET /api/workspace-slug-check verifica disponibilidad de slug", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspace-slug-check?slug=doesnotexist-${Date.now()}`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Workspaces — miembros", () => {
  test("GET /api/workspaces/{slug}/members devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/members`);
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(body.length).toBeGreaterThan(0);
  });

  test("GET /api/workspaces/{slug}/workspace-members/me devuelve rol propio", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/workspace-members/me`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ role: expect.any(Number) });
  });
});

test.describe("Workspaces — themes", () => {
  test("POST + GET + DELETE /api/workspaces/{slug}/workspace-themes", async ({ request, csrf }) => {
    // Create
    const create = await request.post(`${BASE}/api/workspaces/${slug()}/workspace-themes`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Theme ${Date.now()}`, actor: "#ffffff", background: "#000000" },
    });
    expect(create.status()).toBe(201);
    const theme = await create.json() as { id: string };

    // Read
    const get = await request.get(`${BASE}/api/workspaces/${slug()}/workspace-themes/${theme.id}`);
    expect(get.status()).toBe(200);

    // Delete
    const del = await request.delete(`${BASE}/api/workspaces/${slug()}/workspace-themes/${theme.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("Workspaces — aggregate (read-only)", () => {
  for (const resource of ["cycles", "modules", "estimates", "labels", "states", "issues"]) {
    test(`GET /api/workspaces/{slug}/${resource} devuelve 200`, async ({ request }) => {
      const res = await request.get(`${BASE}/api/workspaces/${slug()}/${resource}`);
      expect(res.status()).toBe(200);
    });
  }
});

test.describe("Workspaces — invitaciones", () => {
  test("GET /api/workspaces/{slug}/invitations devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/invitations`);
    expect(res.status()).toBe(200);
  });
});
