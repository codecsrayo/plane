/**
 * Suite: projects.spec.ts
 * Cubre sección 5 del todo_test.md
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

test.describe("Projects — CRUD", () => {
  test("GET /api/workspaces/{slug}/projects lista proyectos", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/projects`);
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(body.length).toBeGreaterThan(0);
  });

  test("GET /api/workspaces/{slug}/projects/details no capturado como UUID", async ({ request }) => {
    // Este path literal debe ir antes de /{project_id}
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/projects/details`);
    expect([200, 400]).toContain(res.status());
  });

  test("GET /api/workspaces/{slug}/projects/{project_id} devuelve proyecto de seed", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/projects/${pid()}`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ id: pid() });
  });

  test("PATCH /api/workspaces/{slug}/projects/{project_id} actualiza descripción", async ({ request, csrf }) => {
    const res = await request.patch(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { description: `Desc ${Date.now()}` },
      },
    );
    expect(res.status()).toBe(200);
  });

  test("POST/DELETE /api/workspaces/{slug}/projects/{project_id}/archive — archive y unarchive", async ({ request, csrf }) => {
    const archive = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/archive`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 204]).toContain(archive.status());

    // Unarchive
    const unarchive = await request.delete(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/archive`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 204]).toContain(unarchive.status());
  });
});

test.describe("Projects — miembros", () => {
  test("GET .../projects/{project_id}/members devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/members`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET .../projects/{project_id}/project-members/me devuelve rol", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/project-members/me`,
    );
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ role: expect.any(Number) });
  });
});

test.describe("Projects — summary y propiedades", () => {
  test("GET .../projects/{project_id}/summary devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/summary`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET .../projects/{project_id}/user-properties — get_or_create nunca 404", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/user-properties`,
    );
    expect(res.status()).toBe(200);
  });
});
