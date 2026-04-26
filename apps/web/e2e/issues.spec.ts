/**
 * Suite: issues.spec.ts
 * Cubre sección 7 del todo_test.md — CRUD básico + bulk ops
 * Cubre ambos paths: /issues/ (legacy) y /work-items/ (nuevo)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const issuesPath = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues`;

test.describe("Issues — CRUD básico", () => {
  test("POST + GET + PATCH + DELETE issue (ciclo completo)", async ({ request, csrf }) => {
    const stateId = Env.STATE_IDS[0];

    // CREATE
    const create = await request.post(issuesPath(), {
      headers: { "X-CSRFToken": csrf },
      data: { name: "E2E Issue full cycle", state_id: stateId },
    });
    expect(create.status()).toBe(201);
    const issue = await create.json() as { id: string; name: string };
    expect(issue).toMatchObject({ name: "E2E Issue full cycle" });

    // READ
    const get = await request.get(`${issuesPath()}/${issue.id}`);
    expect(get.status()).toBe(200);
    expect(await get.json()).toMatchObject({ id: issue.id });

    // UPDATE
    const patch = await request.patch(`${issuesPath()}/${issue.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: "E2E Issue updated", priority: "high" },
    });
    expect(patch.status()).toBe(200);
    const updated = await patch.json() as Record<string, unknown>;
    expect(updated).toMatchObject({ name: "E2E Issue updated", priority: "high" });

    // READ-BACK verificar persistencia
    const readBack = await request.get(`${issuesPath()}/${issue.id}`);
    expect(await readBack.json()).toMatchObject({ name: "E2E Issue updated" });

    // DELETE
    const del = await request.delete(`${issuesPath()}/${issue.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());

    // Confirmar eliminación
    const gone = await request.get(`${issuesPath()}/${issue.id}`);
    expect([404]).toContain(gone.status());
  });

  test("GET /issues/list devuelve 200", async ({ request }) => {
    const res = await request.get(`${issuesPath()}/list`);
    expect(res.status()).toBe(200);
  });

  test("GET /issues-detail devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues-detail`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /v2/issues devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/v2/issues`,
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("Issues — validación de input", () => {
  test("POST issue sin name devuelve 400", async ({ request, csrf }) => {
    const res = await request.post(issuesPath(), {
      headers: { "X-CSRFToken": csrf },
      data: { priority: "high" }, // sin name
    });
    expect(res.status()).toBe(400);
  });
});

test.describe("Issues — path work-items (nuevo)", () => {
  test("freshIssue es accesible vía /work-items/{combined} (PROJ-N)", async ({ request, freshIssue }) => {
    // El combined es formato {identifier}-{sequence} — obtenemos el issue para saber su sequence
    const get = await request.get(`${issuesPath()}/${freshIssue.id}`);
    const body = await get.json() as Record<string, unknown>;
    const seqId = body.sequence_id as number;
    // Obtener identifier del proyecto
    const proj = await request.get(`${BASE}/api/workspaces/${slug()}/projects/${pid()}`);
    const projBody = await proj.json() as Record<string, unknown>;
    const combined = `${projBody.identifier}-${seqId}`;

    const workItem = await request.get(`${BASE}/api/workspaces/${slug()}/work-items/${combined}`);
    expect(workItem.status()).toBe(200);
    expect(await workItem.json()).toMatchObject({ id: freshIssue.id });
  });
});

test.describe("Issues — bulk ops", () => {
  test("POST /bulk-delete-issues acepta lista de ids", async ({ request, csrf, freshIssue }) => {
    const res = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/bulk-delete-issues`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { issue_ids: [] }, // lista vacía = no-op
      },
    );
    expect([200, 204]).toContain(res.status());
    void freshIssue; // fixture crea/elimina el issue independientemente
  });

  test("POST /bulk-archive-issues acepta lista de ids", async ({ request, csrf }) => {
    const res = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/bulk-archive-issues`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { issue_ids: [] },
      },
    );
    expect([200, 204]).toContain(res.status());
  });
});

test.describe("Issues — archive", () => {
  test("POST/DELETE /issues/{pk}/archive — archive y unarchive", async ({ request, csrf, freshIssue }) => {
    const archivePath = `${issuesPath()}/${freshIssue.id}/archive`;

    const archive = await request.post(archivePath, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(archive.status());

    const unarchive = await request.delete(archivePath, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(unarchive.status());
  });

  test("GET /archived-issues devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/archived-issues`,
    );
    expect(res.status()).toBe(200);
  });
});
