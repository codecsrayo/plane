import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const path = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/labels`;

test.describe("Labels — CRUD (canónico)", () => {
  test("GET labels devuelve lista", async ({ request }) => {
    const res = await request.get(path());
    expect(res.status()).toBe(200);
    expect(await res.json()).toBeInstanceOf(Array);
  });

  test("POST + PATCH + DELETE label", async ({ request, csrf }) => {
    const create = await request.post(path(), {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Label ${Date.now()}`, color: "#123456" },
    });
    expect(create.status()).toBe(201);
    const label = await create.json() as { id: string };

    const patch = await request.patch(`${path()}/${label.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { color: "#654321" },
    });
    expect(patch.status()).toBe(200);

    const del = await request.delete(`${path()}/${label.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });

  test("POST bulk-create-labels crea múltiples labels", async ({ request, csrf }) => {
    const names = [`BL1-${Date.now()}`, `BL2-${Date.now()}`];
    // Paridad: handler espera body { label_data: [...] } (no array directo)
    const res = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/bulk-create-labels`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { label_data: names.map((name) => ({ name, color: "#aaaaaa" })) },
      },
    );
    expect([200, 201]).toContain(res.status());
  });
});

test.describe("Labels — alias issue-labels", () => {
  test("GET /issue-labels devuelve misma lista que /labels", async ({ request }) => {
    const [r1, r2] = await Promise.all([
      request.get(path()),
      request.get(`${BASE}/api/workspaces/${slug()}/projects/${pid()}/issue-labels`),
    ]);
    expect(r1.status()).toBe(200);
    expect(r2.status()).toBe(200);
  });
});
