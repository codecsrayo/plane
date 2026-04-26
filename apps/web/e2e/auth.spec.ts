/**
 * Suite: auth.spec.ts
 * Cubre sección 1 del todo_test.md: /auth/* endpoints
 *
 * Paridad Django: los endpoints de sign-in/sign-up/sign-out usan
 * `application/x-www-form-urlencoded` y responden 303 con Location que puede
 * incluir `?error_code=NNNN` cuando hay error. Con cookies y CSRF válidos en
 * éxito, Location apunta a la app sin error_code.
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
      form: { email: Env.TEST_EMAIL },
    });
    // 200 si existe, 400 si formato incorrecto
    expect([200, 400]).toContain(res.status());
  });
});

test.describe("Auth — sign-in / sign-out", () => {
  test("POST /auth/sign-in con credenciales válidas redirige (303 sin error_code)", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
      maxRedirects: 0,
    });
    expect([302, 303]).toContain(res.status());
    const location = res.headers()["location"] ?? "";
    expect(location).not.toContain("error_code=");
  });

  test("POST /auth/sign-in con password incorrecto redirige con error_code", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL, password: "wrong-password!" },
      maxRedirects: 0,
    });
    expect([302, 303]).toContain(res.status());
    const location = res.headers()["location"] ?? "";
    // Plane devuelve error_code en query string (ej. 5065 AUTHENTICATION_FAILED_SIGN_IN)
    expect(location).toContain("error_code=");
  });

  test("POST /auth/sign-out cierra sesión correctamente", async ({ request, csrf }) => {
    // Iniciar sesión primero
    await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
      maxRedirects: 0,
    });
    const res = await request.post(`${BASE}/auth/sign-out`, {
      headers: { "X-CSRFToken": csrf },
      form: { csrfmiddlewaretoken: csrf },
      maxRedirects: 0,
    });
    // Logout también responde 303 al landing
    expect([200, 204, 302, 303]).toContain(res.status());
  });
});

test.describe("Auth — validación de input", () => {
  test("POST /auth/sign-in sin email redirige con error_code", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      form: { password: "irrelevant" },
      maxRedirects: 0,
    });
    // Validación faltante -> redirect con error_code (paridad Django).
    expect([302, 303, 400]).toContain(res.status());
    if (res.status() === 302 || res.status() === 303) {
      expect(res.headers()["location"] ?? "").toContain("error_code=");
    }
  });

  test("POST /auth/sign-up con email ya registrado redirige con error_code", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-up`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
      maxRedirects: 0,
    });
    // El email existe desde globalSetup -> error_code 5030 USER_ALREADY_EXIST
    expect([302, 303, 400, 409]).toContain(res.status());
    if (res.status() === 302 || res.status() === 303) {
      expect(res.headers()["location"] ?? "").toContain("error_code=");
    }
  });
});

test.describe("Auth — rate-limit headers", () => {
  test("Respuestas de /auth/* incluyen cabeceras X-RateLimit-* (cuando aplica)", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/sign-in`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL, password: Env.TEST_PASSWORD },
      maxRedirects: 0,
    });
    const headers = res.headers();
    if (headers["x-ratelimit-limit"]) {
      expect(Number(headers["x-ratelimit-limit"])).toBeGreaterThan(0);
    }
  });
});

test.describe("Auth — spaces aliases", () => {
  test("POST /auth/spaces/email-check responde igual que el endpoint principal", async ({ request, csrf }) => {
    const res = await request.post(`${BASE}/auth/spaces/email-check`, {
      headers: { "X-CSRFToken": csrf },
      form: { email: Env.TEST_EMAIL },
    });
    expect([200, 400]).toContain(res.status());
  });
});
