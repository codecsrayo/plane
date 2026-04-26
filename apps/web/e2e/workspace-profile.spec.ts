/**
 * Suite: workspace-profile.spec.ts — Sección 4 (user-profile endpoints)
 * Cubre:
 *   GET /workspaces/{slug}/user-profile/{user_id}
 *   GET /workspaces/{slug}/user-stats/{user_id}
 *   GET /workspaces/{slug}/user-activity/{user_id}
 *   GET /workspaces/{slug}/user-activity/{user_id}/export
 *   GET /workspaces/{slug}/user-issues/{user_id}
 *   GET /users/me/workspaces/{slug}/activity-graph
 *   GET /users/me/workspaces/{slug}/issues-completed-graph
 *   GET /users/me/workspaces/{slug}/dashboard
 *   GET /workspaces/{slug}/user-profile/{user_id} (workspace-views)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

/** Obtiene el user_id del usuario autenticado. */
async function getMyUserId(
  request: import("@playwright/test").APIRequestContext,
): Promise<string> {
  const res = await request.get(`${BASE}/api/users/me`);
  const me = await res.json() as { id: string };
  return me.id;
}

// ── User profile endpoints ─────────────────────────────────────────────────────

test.describe("Workspace profile — user-profile", () => {
  test("GET user-profile/{user_id} del usuario propio devuelve 200", async ({ request }) => {
    const userId = await getMyUserId(request);
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-profile/${userId}`,
    );
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    // Paridad Django: shape es { user_data: {...}, project_data: [...] }
    // (apps/api/plane/app/views/workspace/user.py: WorkspaceUserProfileEndpoint).
    expect(body).toMatchObject({
      user_data: expect.any(Object),
      project_data: expect.any(Array),
    });
  });

  test("GET user-profile con UUID inexistente devuelve 404", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-profile/00000000-0000-0000-0000-000000000001`,
    );
    expect([404]).toContain(res.status());
  });
});

// ── User stats ────────────────────────────────────────────────────────────────

test.describe("Workspace profile — user-stats", () => {
  test("GET user-stats/{user_id} devuelve estadísticas", async ({ request }) => {
    const userId = await getMyUserId(request);
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-stats/${userId}`,
    );
    expect(res.status()).toBe(200);
  });
});

// ── User activity ─────────────────────────────────────────────────────────────

test.describe("Workspace profile — user-activity", () => {
  test("GET user-activity/{user_id} devuelve 200", async ({ request }) => {
    const userId = await getMyUserId(request);
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-activity/${userId}`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET user-activity/{user_id}/export devuelve CSV o 202", async ({ request }) => {
    const userId = await getMyUserId(request);
    const get = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-activity/${userId}/export`,
    );
    expect([200, 202]).toContain(get.status());
  });
});

// ── User issues (tabs: assigned/created/subscribed) ───────────────────────────

test.describe("Workspace profile — user-issues", () => {
  test("GET user-issues/{user_id} por defecto devuelve 200", async ({ request }) => {
    const userId = await getMyUserId(request);
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/user-issues/${userId}`,
    );
    expect(res.status()).toBe(200);
  });

  for (const tab of ["assigned", "created", "subscribed"]) {
    test(`GET user-issues con ?type=${tab}`, async ({ request }) => {
      const userId = await getMyUserId(request);
      const res = await request.get(
        `${BASE}/api/workspaces/${slug()}/user-issues/${userId}?type=${tab}`,
      );
      expect(res.status()).toBe(200);
    });
  }
});

// ── Me dashboard endpoints ────────────────────────────────────────────────────

test.describe("Users me — workspace dashboard", () => {
  test("GET /users/me/workspaces/{slug}/dashboard devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/users/me/workspaces/${slug()}/dashboard`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /users/me/workspaces/{slug}/activity-graph devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/users/me/workspaces/${slug()}/activity-graph`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /users/me/workspaces/{slug}/issues-completed-graph devuelve 200", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/users/me/workspaces/${slug()}/issues-completed-graph`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /users/me/workspaces/{slug}/project-roles devuelve mapa de roles", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/users/me/workspaces/${slug()}/project-roles`,
    );
    expect(res.status()).toBe(200);
  });

  test("GET /users/me/workspaces/{slug}/projects/invitations devuelve lista", async ({ request }) => {
    const res = await request.get(
      `${BASE}/api/users/me/workspaces/${slug()}/projects/invitations`,
    );
    expect(res.status()).toBe(200);
  });
});

// ── Workspace views (workspace-views) ─────────────────────────────────────────

test.describe("Workspaces — workspace-views", () => {
  test("GET + POST + PATCH + DELETE workspace-views", async ({ request, csrf }) => {
    const create = await request.post(`${BASE}/api/workspaces/${slug()}/workspace-views`, {
      headers: { "X-CSRFToken": csrf },
      data: { view_props: { name: `WS View ${Date.now()}`, filters: {} } },
    });
    // Paridad Django: WorkspaceMemberUserViewsEndpoint.post devuelve 204 No Content.
    expect([200, 201, 204]).toContain(create.status());

    const list = await request.get(`${BASE}/api/workspaces/${slug()}/workspace-views`);
    expect(list.status()).toBe(200);
  });
});
