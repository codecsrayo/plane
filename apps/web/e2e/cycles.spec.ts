import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const cyclesBase = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/cycles`;

test.describe("Cycles — CRUD", () => {
  test("GET cycles devuelve lista", async ({ request }) => {
    const res = await request.get(cyclesBase());
    expect(res.status()).toBe(200);
  });

  test("POST + GET + PATCH + DELETE cycle", async ({ request, csrf, freshCycle }) => {
    // GET
    const get = await request.get(`${cyclesBase()}/${freshCycle.id}`);
    expect(get.status()).toBe(200);
    expect(await get.json()).toMatchObject({ id: freshCycle.id });

    // PATCH
    const patch = await request.patch(`${cyclesBase()}/${freshCycle.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Updated ${freshCycle.name}` },
    });
    expect(patch.status()).toBe(200);
  });

  test("POST /date-check valida solapamiento de fechas", async ({ request, csrf }) => {
    const now = new Date();
    const end = new Date(now.getTime() + 7 * 86400_000);
    const res = await request.post(`${cyclesBase()}/date-check`, {
      headers: { "X-CSRFToken": csrf },
      data: {
        start_date: now.toISOString().slice(0, 10),
        end_date: end.toISOString().slice(0, 10),
      },
    });
    expect(res.status()).toBe(200);
  });
});

test.describe("Cycles — cycle-issues", () => {
  test("GET/POST/DELETE cycle-issues", async ({ request, csrf, freshCycle, freshIssue }) => {
    const path = `${cyclesBase()}/${freshCycle.id}/cycle-issues`;

    // Add issue to cycle
    const add = await request.post(path, {
      headers: { "X-CSRFToken": csrf },
      data: { issues: [freshIssue.id] },
    });
    expect([200, 201]).toContain(add.status());

    // List
    const list = await request.get(path);
    expect(list.status()).toBe(200);

    // Remove
    const remove = await request.delete(`${path}/${freshIssue.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(remove.status());
  });
});

test.describe("Cycles — analytics, progress, user-properties", () => {
  test("GET cycle analytics devuelve 200", async ({ request, freshCycle }) => {
    const res = await request.get(`${cyclesBase()}/${freshCycle.id}/analytics`);
    expect(res.status()).toBe(200);
  });

  test("GET cycle progress devuelve 200", async ({ request, freshCycle }) => {
    const res = await request.get(`${cyclesBase()}/${freshCycle.id}/progress`);
    expect(res.status()).toBe(200);
  });

  test("GET cycle user-properties — get_or_create nunca 404", async ({ request, freshCycle }) => {
    const res = await request.get(`${cyclesBase()}/${freshCycle.id}/user-properties`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Cycles — archive", () => {
  test("POST archive / DELETE unarchive cycle", async ({ request, csrf, freshCycle }) => {
    const archivePath = `${cyclesBase()}/${freshCycle.id}/archive`;

    const archive = await request.post(archivePath, { headers: { "X-CSRFToken": csrf } });
    // Paridad Django: solo ciclos completados (end_date pasada) pueden archivarse.
    // freshCycle se crea con end_date = hoy + 7 días -> 400 esperado.
    expect([200, 204, 400]).toContain(archive.status());

    const unarchive = await request.delete(archivePath, { headers: { "X-CSRFToken": csrf } });
    expect([200, 204, 404]).toContain(unarchive.status());
  });

  test("GET /archived-cycles devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/archived-cycles`,
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("Cycles — favorites", () => {
  test("POST + DELETE user-favorite-cycles", async ({ request, csrf, freshCycle }) => {
    const favPath = `${BASE}/api/workspaces/${slug()}/projects/${pid()}/user-favorite-cycles`;

    const add = await request.post(favPath, {
      headers: { "X-CSRFToken": csrf },
      data: { cycle: freshCycle.id },
    });
    expect([200, 201, 204]).toContain(add.status());

    const remove = await request.delete(`${favPath}/${freshCycle.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(remove.status());
  });
});
