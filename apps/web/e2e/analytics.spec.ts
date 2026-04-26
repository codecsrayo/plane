import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

test.describe("Analytics — workspace legacy", () => {
  // analytics requiere x_axis/y_axis. default-analytics y project-stats no.
  for (const path of ["default-analytics", "project-stats"]) {
    test(`GET /api/workspaces/{slug}/${path} devuelve 200`, async ({ request }) => {
      const res = await request.get(`${BASE}/api/workspaces/${slug()}/${path}`);
      expect(res.status()).toBe(200);
    });
  }

  test(`GET /api/workspaces/{slug}/analytics con x_axis/y_axis devuelve 200`, async ({ request }) => {
    // Paridad Django: x_axis y y_axis son requeridos. Valores válidos del whitelist.
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/analytics?x_axis=priority&y_axis=issue_count`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET/POST analytic-view CRUD", async ({ request, csrf }) => {
    const create = await request.post(`${BASE}/api/workspaces/${slug()}/analytic-view`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `E2E AnalyticView ${Date.now()}`, query: {}, query_dict: {} },
    });
    expect([200, 201]).toContain(create.status());
    const view = await create.json() as { id: string };

    const get = await request.get(`${BASE}/api/workspaces/${slug()}/analytic-view/${view.id}`);
    expect(get.status()).toBe(200);

    const del = await request.delete(
      `${BASE}/api/workspaces/${slug()}/analytic-view/${view.id}`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("Analytics — advance analytics", () => {
  for (const suffix of ["advance-analytics", "advance-analytics-stats", "advance-analytics-charts"]) {
    test(`GET /workspaces/{slug}/${suffix} devuelve 200`, async ({ request }) => {
      const res = await request.get(`${BASE}/api/workspaces/${slug()}/${suffix}`);
      expect(res.status()).toBe(200);
    });

    test(`GET /projects/{project_id}/${suffix} devuelve 200`, async ({ request }) => {
      // El handler project-level de "advance-analytics-charts" valida `type` y
      // defaults a "projects" que solo es válido a nivel workspace. A nivel
      // proyecto solo acepta "work-items" | "custom-work-items".
      const qs = suffix === "advance-analytics-charts" ? "?type=work-items" : "";
      const res = await request.get(
        `${BASE}/api/workspaces/${slug()}/projects/${pid()}/${suffix}${qs}`,
      );
      expect(res.status()).toBe(200);
    });
  }
});
