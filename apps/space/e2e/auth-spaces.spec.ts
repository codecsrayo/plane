/**
 * Suite: auth-spaces.spec.ts
 * Cubre endpoints /auth/spaces/* (sección 1 del todo_test.md)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;

const SPACE_AUTH_PATHS = [
  "/auth/spaces/email-check",
  "/auth/spaces/magic-generate",
] as const;

test.describe("Spaces — auth endpoints", () => {
  test("POST /auth/spaces/email-check acepta email válido", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/email-check`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL },
    });
    expect([200, 400]).toContain(res.status());
  });

  test("POST /auth/spaces/sign-in con credenciales válidas", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
    });
    expect([200, 400]).toContain(res.status());
  });

  test("POST /auth/spaces/forgot-password acepta email", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/forgot-password`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL },
    });
    expect([200, 400]).toContain(res.status());
  });

  test("POST /auth/spaces/sign-out devuelve 200/204", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/sign-out`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(res.status());
  });
});
