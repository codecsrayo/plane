import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

test.describe("Views — workspace level", () => {
  test("POST + GET + PATCH + DELETE workspace view", async ({ request, csrf, freshView }) => {
    const get = await request.get(`${BASE}/api/workspaces/${slug()}/views/${freshView.id}`);
    expect(get.status()).toBe(200);

    const patch = await request.patch(`${BASE}/api/workspaces/${slug()}/views/${freshView.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Updated ${freshView.name}` },
    });
    expect(patch.status()).toBe(200);
  });
});

test.describe("Views — project level", () => {
  test("GET project views devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/views`,
    );
    expect(res.status()).toBe(200);
  });

  test("POST + DELETE project view + favorite", async ({ request, csrf }) => {
    const create = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/views`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: `E2E Project View ${Date.now()}`, filters: {} },
      },
    );
    expect(create.status()).toBe(201);
    const view = await create.json() as { id: string };

    // Favorite
    const favPath = `${BASE}/api/workspaces/${slug()}/projects/${pid()}/user-favorite-views`;
    await request.post(favPath, {
      headers: { "X-CSRFToken": csrf },
      data: { view: view.id },
    });
    await request.delete(`${favPath}/${view.id}`, { headers: { "X-CSRFToken": csrf } });

    // Delete view
    const del = await request.delete(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/views/${view.id}`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 204]).toContain(del.status());
  });
});
