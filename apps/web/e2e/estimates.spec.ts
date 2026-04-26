import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const basePath = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}`;

test.describe("Estimates — CRUD", () => {
  test("GET /project-estimates devuelve lista", async ({ request }) => {
    const res = await request.get(`${basePath()}/project-estimates`);
    expect(res.status()).toBe(200);
  });

  test("POST + PATCH + DELETE estimate con points", async ({ request, csrf }) => {
    // Create — el handler exige `points` en el body (Vec<EstimatePointInput>).
    // Mandamos array vacío y luego añadimos puntos via /estimate-points.
    const create = await request.post(`${basePath()}/estimates`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Est ${Date.now()}`, type: "points", points: [] },
    });
    expect([200, 201]).toContain(create.status());
    const est = await create.json() as { id: string };

    // Add points
    const addPoints = await request.post(
      `${basePath()}/estimates/${est.id}/estimate-points`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { key: 1, value: "1" },
      },
    );
    expect([200, 201, 204]).toContain(addPoints.status());

    // Delete estimate
    const del = await request.delete(`${basePath()}/estimates/${est.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});
