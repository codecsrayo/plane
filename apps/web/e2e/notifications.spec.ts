import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

// Paridad Django: notificaciones viven bajo /workspaces/{slug}/users/notifications/
// (apps/api/plane/app/urls/notification.py).
test.describe("Notifications", () => {
  test("GET /workspaces/{slug}/users/notifications devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/users/notifications`);
    expect(res.status()).toBe(200);
  });

  test("GET /workspaces/{slug}/users/notifications/unread devuelve count", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/users/notifications/unread`);
    expect(res.status()).toBe(200);
  });

  test("POST /workspaces/{slug}/users/notifications/mark-all-read devuelve 200", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/api/workspaces/${slug()}/users/notifications/mark-all-read`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(res.status());
  });
});
