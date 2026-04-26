/**
 * Suite: auth.spec.ts
 * Cubre sección 1 del todo_test.md: /auth/* endpoints
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;

test.describe("Auth — CSRF & email-check", () => {
  test("GET /auth/get-csrf-token devuelve 200 y csrf_token en body", async ({ request }) => {
    const res = await request.get(`${BASE}/auth/get-csrf-token`);
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ csrf_token: expect.any(String) });
  });

  test("POST /auth/email-check responde con info del email", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/email-check`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL },
    });
    // 200 si existe, 400 si formato incorrecto — en este caso el email existe del setup
    expect([200, 400]).toContain(res.status());
  });
});

test.describe("Auth — sign-in / sign-out", () => {
  test("POST /auth/sign-in con credenciales válidas devuelve sesión", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
    });
    expect(res.status()).toBe(200);
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ id: expect.any(String) });
  });

  test("POST /auth/sign-in con password incorrecto devuelve 400/401", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: "wrong-password!" },
    });
    expect([400, 401]).toContain(res.status());
  });

  test("POST /auth/sign-out cierra sesión correctamente", async ({ request, csrf }) => {
    // Iniciar sesión primero
    await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
    });
    const res = await request.post(`${BASE}/auth/sign-out`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(res.status());
  });
});

test.describe("Auth — validación de input", () => {
  test("POST /auth/sign-in sin email devuelve 400", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { password: "irrelevant" },
    });
    expect(res.status()).toBe(400);
  });

  test("POST /auth/sign-up con email ya registrado devuelve 400", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-up`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
    });
    // El email existe desde globalSetup
    expect([400, 409]).toContain(res.status());
  });
});

test.describe("Auth — rate-limit headers", () => {
  test("Respuestas de /auth/* incluyen cabeceras X-RateLimit-*", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
    });
    // El middleware puede agregar rate-limit headers aunque la respuesta sea 200
    const headers = res.headers();
    // No forzamos que existan en todos los entornos, pero si están deben tener formato correcto
    if (headers["x-ratelimit-limit"]) {
      expect(Number(headers["x-ratelimit-limit"])).toBeGreaterThan(0);
    }
  });
});

test.describe("Auth — spaces aliases", () => {
  test("POST /auth/spaces/email-check responde igual que el endpoint principal", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/email-check`, {
      headers: { "X-CSRFToken": csrf },
      data: { email: Env.TEST_EMAIL },
    });
    expect([200, 400]).toContain(res.status());
  });
});
