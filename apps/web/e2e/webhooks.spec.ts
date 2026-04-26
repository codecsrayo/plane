import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

test.describe("Webhooks — CRUD", () => {
  test("GET webhooks devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/webhooks`);
    expect(res.status()).toBe(200);
  });

  test("GET + PATCH webhook (freshWebhook)", async ({ request, csrf, freshWebhook }) => {
    const get = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}`,
    );
    expect(get.status()).toBe(200);

    const patch = await request.patch(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { is_active: false },
      },
    );
    expect(patch.status()).toBe(200);
  });

  test("POST regenerate webhook secret", async ({ request, csrf, freshWebhook }) => {
    const res = await request.post(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}/regenerate`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 201]).toContain(res.status());
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ secret_key: expect.any(String) });
  });

  test("GET webhook-logs devuelve lista", async ({ request, freshWebhook }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhook-logs/${freshWebhook.id}`,
    );
    expect(res.status()).toBe(200);
  });
});
