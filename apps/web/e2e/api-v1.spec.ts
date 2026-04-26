/**
 * Suite: api-v1.spec.ts
 * Auth: header X-Api-Key (no CSRF, no session cookie)
 * Cubre sección 21 del todo_test.md
 */
import { test, expect } from "@playwright/test";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

const apiKey = () => ({
  "X-Api-Key": Env.API_TOKEN,
  "Content-Type": "application/json",
});

test.describe("API v1 — usuarios y workspace", () => {
  test("GET /api/v1/users/me devuelve usuario", async ({ request }) => {
    const res = await request.get(`${BASE}/api/v1/users/me`, { headers: apiKey() });
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ id: expect.any(String) });
  });

  test("GET /api/v1/workspaces/{slug}/members devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/members`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("API v1 — projects CRUD", () => {
  test("GET /api/v1/workspaces/{slug}/projects lista proyectos", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });

  test("GET /api/v1/workspaces/{slug}/projects/{pk} devuelve proyecto", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
    expect(await res.json()).toMatchObject({ id: pid() });
  });
});

test.describe("API v1 — work-items CRUD", () => {
  test("POST + GET + PATCH + DELETE work-item", async ({ request }) => {
    // Create
    const create = await request.post(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/work-items`,
      {
        headers: apiKey(),
        data: JSON.stringify({ name: "V1 Work Item E2E", state_id: Env.STATE_IDS[0] }),
      },
    );
    expect(create.status()).toBe(201);
    const item = await create.json() as { id: string };

    // Read
    const get = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/work-items/${item.id}`,
      { headers: apiKey() },
    );
    expect(get.status()).toBe(200);

    // Patch
    const patch = await request.patch(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/work-items/${item.id}`,
      {
        headers: apiKey(),
        data: JSON.stringify({ priority: "medium" }),
      },
    );
    expect(patch.status()).toBe(200);

    // Delete
    const del = await request.delete(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/work-items/${item.id}`,
      { headers: apiKey() },
    );
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("API v1 — states y labels", () => {
  test("GET states devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/states`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });

  test("GET labels devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/labels`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("API v1 — cycles y modules", () => {
  test("GET cycles devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/cycles`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });

  test("GET modules devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/v1/workspaces/${slug()}/projects/${pid()}/modules`,
      { headers: apiKey() },
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("API v1 — sin API key devuelve 401", () => {
  test("GET /api/v1/users/me sin key", async ({ request }) => {
    const res = await request.get(`${BASE}/api/v1/users/me`);
    expect(res.status()).toBe(401);
  });
});
