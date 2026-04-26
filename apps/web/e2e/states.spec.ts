import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const path = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/states`;

test.describe("States — CRUD", () => {
  test("GET states devuelve los estados por defecto", async ({ request }) => {
    const res = await request.get(path());
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(body.length).toBeGreaterThan(0);
  });

  test("POST + PATCH + DELETE state", async ({ request, csrf }) => {
    // Create
    const create = await request.post(path(), {
      headers: { "X-CSRFToken": csrf },
      data: { name: `State ${Date.now()}`, color: "#aabbcc", group: "started" },
    });
    expect(create.status()).toBe(201);
    const state = await create.json() as { id: string };

    // Patch
    const patch = await request.patch(`${path()}/${state.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `State updated ${Date.now()}` },
    });
    expect(patch.status()).toBe(200);

    // Delete
    const del = await request.delete(`${path()}/${state.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });

  test("POST .../states/{pk}/mark-default cambia estado por defecto", async ({ request, csrf }) => {
    const stateId = Env.STATE_IDS[0];
    if (!stateId) return test.skip();
    const res = await request.post(`${path()}/${stateId}/mark-default`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(res.status());
  });
});
