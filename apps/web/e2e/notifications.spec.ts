import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;

test.describe("Notifications", () => {
  test("GET /api/users/notifications devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/notifications`);
    expect(res.status()).toBe(200);
  });

  test("GET /api/users/notifications/unread devuelve count", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/notifications/unread`);
    expect(res.status()).toBe(200);
  });

  test("POST /api/users/notifications/mark-all-read devuelve 200", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/api/users/notifications/mark-all-read`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(res.status());
  });
});
