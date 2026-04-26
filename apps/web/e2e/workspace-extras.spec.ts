/**
 * Suite: workspace-extras.spec.ts — Sección 4.1
 * Cubre endpoints de workspace extras:
 *   favorites, home-preferences, quick-links, stickies,
 *   sidebar-preferences, recent-visits, draft-issues, user-properties
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const base = () => `${BASE}/api/workspaces/${slug()}`;

// ── Favorites ─────────────────────────────────────────────────────────────────

test.describe("Workspace extras — user-favorites", () => {
  test("GET user-favorites devuelve lista", async ({ request }) => {
    const res = await request.get(`${base()}/user-favorites`);
    expect(res.status()).toBe(200);
    expect(await res.json()).toBeInstanceOf(Array);
  });

  test("POST + GET children + DELETE user-favorite", async ({ request, csrf }) => {
    const create = await request.post(`${base()}/user-favorites`, {
      headers: { "X-CSRFToken": csrf },
      data: { entity_type: "project", entity_identifier: pid(), name: "E2E Fav" },
    });
    expect([200, 201]).toContain(create.status());
    const fav = await create.json() as { id: string };

    // Children
    const children = await request.get(`${base()}/user-favorites/${fav.id}/children`);
    expect(children.status()).toBe(200);

    // Group alias
    const group = await request.get(`${base()}/user-favorites/${fav.id}/group`);
    expect(group.status()).toBe(200);

    // Delete
    const del = await request.delete(`${base()}/user-favorites/${fav.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

// ── Home preferences ──────────────────────────────────────────────────────────

test.describe("Workspace extras — home-preferences", () => {
  test("GET home-preferences devuelve lista", async ({ request }) => {
    const res = await request.get(`${base()}/home-preferences`);
    expect(res.status()).toBe(200);
  });

  test("GET + PATCH home-preferences/{key}", async ({ request, csrf }) => {
    const get = await request.get(`${base()}/home-preferences/RECENT_ACTIVITY`);
    expect([200, 404]).toContain(get.status());

    const patch = await request.patch(`${base()}/home-preferences/RECENT_ACTIVITY`, {
      headers: { "X-CSRFToken": csrf },
      data: { is_enabled: true, sort_order: 1 },
    });
    expect([200, 201]).toContain(patch.status());
  });
});

// ── Quick-links ───────────────────────────────────────────────────────────────

test.describe("Workspace extras — quick-links", () => {
  test("GET + POST + PATCH + DELETE quick-link", async ({ request, csrf }) => {
    // Create
    const create = await request.post(`${base()}/quick-links`, {
      headers: { "X-CSRFToken": csrf },
      data: { title: `E2E Link ${Date.now()}`, url: "https://example.com" },
    });
    expect([200, 201]).toContain(create.status());
    const link = await create.json() as { id: string };

    // List
    const list = await request.get(`${base()}/quick-links`);
    expect(list.status()).toBe(200);

    // Patch
    const patch = await request.patch(`${base()}/quick-links/${link.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { title: "Updated Link" },
    });
    expect(patch.status()).toBe(200);

    // Delete
    const del = await request.delete(`${base()}/quick-links/${link.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

// ── Stickies ──────────────────────────────────────────────────────────────────

test.describe("Workspace extras — stickies", () => {
  test("GET + POST + PATCH + DELETE sticky", async ({ request, csrf }) => {
    const create = await request.post(`${base()}/stickies`, {
      headers: { "X-CSRFToken": csrf },
      data: { description_html: "<p>E2E sticky note</p>" },
    });
    expect([200, 201]).toContain(create.status());
    const sticky = await create.json() as { id: string };

    const list = await request.get(`${base()}/stickies`);
    expect(list.status()).toBe(200);

    const patch = await request.patch(`${base()}/stickies/${sticky.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { description_html: "<p>Updated sticky</p>" },
    });
    expect(patch.status()).toBe(200);

    const del = await request.delete(`${base()}/stickies/${sticky.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

// ── Sidebar preferences ───────────────────────────────────────────────────────

test.describe("Workspace extras — sidebar-preferences", () => {
  test("GET sidebar-preferences devuelve 200", async ({ request }) => {
    const res = await request.get(`${base()}/sidebar-preferences`);
    expect(res.status()).toBe(200);
  });

  test("PATCH sidebar-preferences actualiza preferencia", async ({ request, csrf }) => {
    // Paridad Django: PATCH espera un ARRAY de { key, is_pinned?, sort_order? }
    const res = await request.patch(`${base()}/sidebar-preferences`, {
      headers: { "X-CSRFToken": csrf },
      data: [{ key: "drafts", is_pinned: true }],
    });
    expect([200, 204]).toContain(res.status());
  });
});

// ── Recent visits ──────────────────────────────────────────────────────────────

test.describe("Workspace extras — recent-visits", () => {
  test("GET recent-visits devuelve lista", async ({ request }) => {
    const res = await request.get(`${base()}/recent-visits`);
    expect(res.status()).toBe(200);
    expect(await res.json()).toBeInstanceOf(Array);
  });
});

// ── User properties ───────────────────────────────────────────────────────────

test.describe("Workspace extras — user-properties", () => {
  test("GET + PATCH user-properties — get_or_create nunca 404", async ({ request, csrf }) => {
    const get = await request.get(`${base()}/user-properties`);
    expect(get.status()).toBe(200);

    const patch = await request.patch(`${base()}/user-properties`, {
      headers: { "X-CSRFToken": csrf },
      data: { filters: {} },
    });
    expect([200, 204]).toContain(patch.status());
  });
});

// ── Draft issues ──────────────────────────────────────────────────────────────

test.describe("Workspace extras — draft-issues", () => {
  test("GET draft-issues devuelve lista", async ({ request }) => {
    const res = await request.get(`${base()}/draft-issues`);
    expect(res.status()).toBe(200);
  });

  test("POST + GET + PATCH + DELETE draft issue", async ({ request, csrf }) => {
    const create = await request.post(`${base()}/draft-issues`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Draft ${Date.now()}`, project_id: pid() },
    });
    expect([200, 201]).toContain(create.status());
    const draft = await create.json() as { id: string };

    const get = await request.get(`${base()}/draft-issues/${draft.id}`);
    expect(get.status()).toBe(200);

    const patch = await request.patch(`${base()}/draft-issues/${draft.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Draft updated ${Date.now()}` },
    });
    expect(patch.status()).toBe(200);

    const del = await request.delete(`${base()}/draft-issues/${draft.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });

  test("POST draft-to-issue convierte draft en issue real", async ({ request, csrf }) => {
    // Crear draft
    const create = await request.post(`${base()}/draft-issues`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `Draft to promote ${Date.now()}`, project_id: pid() },
    });
    if (!create.ok()) return test.skip();
    const draft = await create.json() as { id: string };

    // Convertir
    const promote = await request.post(`${base()}/draft-to-issue/${draft.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { project_id: pid(), state_id: Env.STATE_IDS[0] },
    });
    expect([200, 201]).toContain(promote.status());

    // El issue creado debe limpiarse — obtenemos su id del response
    if (promote.ok()) {
      const issue = await promote.json() as { id: string };
      await request.delete(
        `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues/${issue.id}`,
        { headers: { "X-CSRFToken": csrf } },
      );
    }
  });
});
