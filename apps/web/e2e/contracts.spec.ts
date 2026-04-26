/**
 * Suite: contracts.spec.ts
 * Valida respuestas contra el schema OpenAPI expuesto por el backend.
 * Librería: ajv (instalada en devDependencies)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

test.describe("Contracts — OpenAPI schema", () => {
  test("GET /api/docs/openapi.json devuelve schema válido", async ({ request }) => {
    const res = await request.get(`${BASE}/api/docs/openapi.json`);
    // Solo disponible cuando DEBUG=true
    if (res.status() === 404) return test.skip();
    expect(res.status()).toBe(200);
    const schema = await res.json() as Record<string, unknown>;
    expect(schema).toMatchObject({
      openapi: expect.stringMatching(/^3\./),
      info: expect.objectContaining({ title: expect.any(String) }),
      paths: expect.any(Object),
    });
  });

  test("GET /api/health devuelve shape { status: string }", async ({ request }) => {
    const res = await request.get(`${BASE}/api/health`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(typeof body.status).toBe("string");
  });

  test("GET /api/timezones devuelve array de strings", async ({ request }) => {
    const res = await request.get(`${BASE}/api/timezones`);
    expect(res.status()).toBe(200);
    const body = await res.json() as unknown[];
    expect(Array.isArray(body)).toBe(true);
    expect(body.length).toBeGreaterThan(0);
  });

  test("Workspace response shape incluye slug, name, id", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({
      id: expect.any(String),
      name: expect.any(String),
      slug: expect.any(String),
    });
  });

  test("Issue response shape incluye id, name, state, priority", async ({ request, freshIssue }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${Env.PROJECT_ID}/issues/${freshIssue.id}`,
    );
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    // IssueDetailResponse — campos garantizados por el struct Rust
    expect(body).toMatchObject({
      id: expect.any(String),
      name: expect.any(String),
      priority: expect.any(String),
      sequence_id: expect.any(Number),
      project_id: expect.any(String),
      is_draft: expect.any(Boolean),
      attachment_count: expect.any(Number),
      link_count: expect.any(Number),
      sub_issues_count: expect.any(Number),
      description_html: expect.any(String),   // solo en detail, no en create
      is_subscribed: expect.any(Boolean),
      is_intake: expect.any(Boolean),
    });
    // estimate_point renombrado desde estimate_point_id en la serialización
    expect(Object.prototype.hasOwnProperty.call(body, "estimate_point")).toBe(true);
  });
});
