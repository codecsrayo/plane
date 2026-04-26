import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;

test.describe("Search", () => {
  test("GET /workspaces/{slug}/search devuelve resultados", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/search?query=e2e`);
    expect(res.status()).toBe(200);
  });

  test("GET /work-items/search alias devuelve 200", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/work-items/search?query=e2e`);
    expect(res.status()).toBe(200);
  });

  test("GET /issues/search (alias legacy) devuelve 200", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/issues/search?query=e2e`);
    expect(res.status()).toBe(200);
  });

  test("GET /projects/{project_id}/search-issues devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/search-issues?query=e2e`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /entity-search con distintos entity_name", async ({ request }) => {
    for (const entity of ["issue", "page", "cycle", "module", "view", "project"]) {
      const res = await request.get(
        `${BASE}/api/workspaces/${slug()}/entity-search?query=e2e&entity_name=${entity}`,
      );
      expect(res.status()).toBe(200);
    }
  });
});
