import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const modulesBase = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/modules`;

test.describe("Modules — CRUD", () => {
  test("GET modules devuelve lista", async ({ request }) => {
    const res = await request.get(modulesBase());
    expect(res.status()).toBe(200);
  });

  test("GET + PATCH module (freshModule)", async ({ request, csrf, freshModule }) => {
    const get = await request.get(`${modulesBase()}/${freshModule.id}`);
    expect(get.status()).toBe(200);

    const patch = await request.patch(`${modulesBase()}/${freshModule.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Updated ${freshModule.name}` },
    });
    expect(patch.status()).toBe(200);
  });
});

test.describe("Modules — module-issues", () => {
  test("GET/POST/DELETE module-issues", async ({ request, csrf, freshModule, freshIssue }) => {
    const path = `${modulesBase()}/${freshModule.id}/issues`;

    const add = await request.post(path, {
      headers: { "X-CSRFToken": csrf },
      data: { issues: [freshIssue.id] },
    });
    expect([200, 201]).toContain(add.status());

    const list = await request.get(path);
    expect(list.status()).toBe(200);

    const remove = await request.delete(`${path}/${freshIssue.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(remove.status());
  });
});

test.describe("Modules — module-links", () => {
  test("POST + DELETE module-link", async ({ request, csrf, freshModule }) => {
    const linksPath = `${modulesBase()}/${freshModule.id}/module-links`;

    const create = await request.post(linksPath, {
      headers: { "X-CSRFToken": csrf },
      data: { url: "https://example.com", title: "E2E Module Link" },
    });
    expect(create.status()).toBe(201);
    const link = await create.json() as { id: string };

    const del = await request.delete(`${linksPath}/${link.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("Modules — archive y favorites", () => {
  test("POST archive / DELETE unarchive", async ({ request, csrf, freshModule }) => {
    const archivePath = `${modulesBase()}/${freshModule.id}/archive`;

    await request.post(archivePath, { headers: { "X-CSRFToken": csrf } });
    await request.delete(archivePath, { headers: { "X-CSRFToken": csrf } });

    const list = await request.get(`${BASE}/api/workspaces/${slug()}/projects/${pid()}/archived-modules`);
    expect(list.status()).toBe(200);
  });

  test("POST + DELETE user-favorite-modules", async ({ request, csrf, freshModule }) => {
    const favPath = `${BASE}/api/workspaces/${slug()}/projects/${pid()}/user-favorite-modules`;
    await request.post(favPath, {
      headers: { "X-CSRFToken": csrf },
      data: { module: freshModule.id },
    });
    const del = await request.delete(`${favPath}/${freshModule.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});
