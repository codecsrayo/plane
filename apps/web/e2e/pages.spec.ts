import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const pagesBase = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/pages`;
const pagePath = (id: string) => `${BASE}/api/workspaces/${slug()}/projects/${pid()}/pages/${id}`;

test.describe("Pages — CRUD", () => {
  test("GET pages-summary devuelve 200", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/projects/${pid()}/pages-summary`);
    expect(res.status()).toBe(200);
  });

  test("GET pages devuelve lista", async ({ request }) => {
    const res = await request.get(pagesBase());
    expect(res.status()).toBe(200);
  });

  test("GET + PATCH page (freshPage)", async ({ request, csrf, freshPage }) => {
    const get = await request.get(pagePath(freshPage.id));
    expect(get.status()).toBe(200);
    expect(await get.json()).toMatchObject({ id: freshPage.id });

    const patch = await request.patch(pagePath(freshPage.id), {
      headers: { "X-CSRFToken": csrf },
      data: { title: `Updated ${freshPage.title}` },
    });
    expect(patch.status()).toBe(200);
  });
});

test.describe("Pages — archive, lock, favorite, duplicate", () => {
  test("POST archive / DELETE unarchive page", async ({ request, csrf, freshPage }) => {
    // POST = archivar, DELETE = desarchivar
    const archive = await request.post(`${pagePath(freshPage.id)}/archive`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(archive.status());

    const unarchive = await request.delete(`${pagePath(freshPage.id)}/archive`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(unarchive.status());
  });

  test("POST lock / DELETE unlock page", async ({ request, csrf, freshPage }) => {
    const lock = await request.post(`${pagePath(freshPage.id)}/lock`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(lock.status());

    const unlock = await request.delete(`${pagePath(freshPage.id)}/lock`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(unlock.status());
  });

  test("POST + DELETE favorite-pages", async ({ request, csrf, freshPage }) => {
    const favPath = `${BASE}/api/workspaces/${slug()}/projects/${pid()}/favorite-pages/${freshPage.id}`;
    await request.post(favPath, { headers: { "X-CSRFToken": csrf } });
    const del = await request.delete(favPath, { headers: { "X-CSRFToken": csrf } });
    expect([200, 204]).toContain(del.status());
  });

  test("POST duplicate page devuelve nueva página", async ({ request, csrf, freshPage }) => {
    const dup = await request.post(`${pagePath(freshPage.id)}/duplicate`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 201]).toContain(dup.status());
    const dupPage = await dup.json() as { id: string };
    // Cleanup copia duplicada
    await request.delete(pagePath(dupPage.id), { headers: { "X-CSRFToken": csrf } });
  });
});

test.describe("Pages — versions", () => {
  test("GET /versions devuelve lista", async ({ request, freshPage }) => {
    const res = await request.get(`${pagePath(freshPage.id)}/versions`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Pages — description", () => {
  test("GET + PATCH /description", async ({ request, csrf, freshPage }) => {
    const get = await request.get(`${pagePath(freshPage.id)}/description`);
    expect([200, 404]).toContain(get.status()); // puede no existir aún

    const patch = await request.patch(`${pagePath(freshPage.id)}/description`, {
      headers: { "X-CSRFToken": csrf },
      data: { description_html: "<p>E2E content</p>", description_binary: "" },
    });
    expect([200, 204]).toContain(patch.status());
  });
});
