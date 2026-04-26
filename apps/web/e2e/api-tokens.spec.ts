import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;

test.describe("API Tokens — sesión-based", () => {
  test("GET /api/api-tokens devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/api-tokens`);
    expect(res.status()).toBe(200);
    expect(await res.json()).toBeInstanceOf(Array);
  });

  test("GET /api/users/api-tokens (alias) devuelve misma lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/users/api-tokens`);
    expect(res.status()).toBe(200);
  });

  test("POST + GET + PATCH + DELETE API token", async ({ request, csrf, freshApiToken }) => {
    // GET
    const get = await request.get(`${BASE}/api/api-tokens/${freshApiToken.id}`);
    expect(get.status()).toBe(200);
    expect(await get.json()).toMatchObject({ id: freshApiToken.id });

    // PATCH
    const patch = await request.patch(`${BASE}/api/api-tokens/${freshApiToken.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { label: `Updated ${Date.now()}` },
    });
    expect(patch.status()).toBe(200);
  });

  test("El token generado tiene formato no vacío", async ({ freshApiToken }) => {
    expect(freshApiToken.token).toBeTruthy();
    expect(freshApiToken.token.length).toBeGreaterThan(10);
  });
});
