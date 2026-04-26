import type { APIRequestContext, APIResponse } from "@playwright/test";

export const API_BASE = process.env.API_BASE_URL ?? "http://localhost:8000";

const ERROR_CODE_RE = /[?&]error_code=(\d+)/;

function extractErrorCode(location: string | null | undefined): string | null {
  if (!location) return null;
  const m = ERROR_CODE_RE.exec(location);
  return m ? m[1] : null;
}

/** Obtiene CSRF token y lo inyecta en el contexto. Devuelve el token. */
export async function fetchCsrfToken(request: APIRequestContext): Promise<string> {
  const res = await request.get(`${API_BASE}/auth/get-csrf-token`);
  if (!res.ok()) throw new Error(`CSRF fetch failed: ${res.status()}`);
  const body = await res.json();
  return (body.csrf_token ?? body.csrfToken) as string;
}

/**
 * Sign-in form-urlencoded. El backend de Plane responde **303 See Other**:
 *   exito → Location apunta a la app (sin error_code).
 *   error → Location contiene `?error_code=NNNN&error_message=...`
 *           (5060 USER_DOES_NOT_EXIST, 5065 AUTHENTICATION_FAILED_SIGN_IN, etc.)
 * Devuelve el usuario via /api/users/me (sesion persistida en cookies del context).
 */
export async function signIn(
  request: APIRequestContext,
  email: string,
  password: string,
  csrfToken: string
): Promise<Record<string, unknown>> {
  const res = await request.post(`${API_BASE}/auth/sign-in`, {
    headers: { "X-CSRFToken": csrfToken },
    form: { email, password },
    maxRedirects: 0,
  });
  return handleAuthRedirect(request, res, "sign-in");
}

/** Sign-up. Misma semantica que signIn (5030 USER_ALREADY_EXIST, 5040, 5045). */
export async function signUp(
  request: APIRequestContext,
  email: string,
  password: string,
  csrfToken: string
): Promise<Record<string, unknown>> {
  const res = await request.post(`${API_BASE}/auth/sign-up`, {
    headers: { "X-CSRFToken": csrfToken },
    form: { email, password },
    maxRedirects: 0,
  });
  return handleAuthRedirect(request, res, "sign-up");
}

/** Procesa la respuesta 3xx de /auth/sign-{in,up}. */
async function handleAuthRedirect(
  request: APIRequestContext,
  res: APIResponse,
  op: "sign-in" | "sign-up"
): Promise<Record<string, unknown>> {
  const status = res.status();
  if (status < 300 || status >= 400) {
    throw new Error(`${op} failed: HTTP ${status} ${await res.text()}`);
  }
  const location = res.headers()["location"];
  const errorCode = extractErrorCode(location);
  if (errorCode) {
    throw new Error(`${op} failed: error_code=${errorCode} (Location=${location ?? ""})`);
  }
  return getMe(request);
}

/** Obtiene el usuario autenticado via cookie de sesion. */
export async function getMe(request: APIRequestContext): Promise<Record<string, unknown>> {
  const res = await request.get(`${API_BASE}/api/users/me/`);
  if (!res.ok()) throw new Error(`getMe failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Espera a que el backend responda /api/health con 200. */
export async function waitForHealth(request: APIRequestContext, retries = 10, delayMs = 2000): Promise<void> {
  // Polling secuencial intencional: cada intento espera al anterior y
  // el delay no debe paralelizarse. Promise.all no aplica aqui.
  /* eslint-disable no-await-in-loop */
  for (let i = 0; i < retries; i++) {
    try {
      const res = await request.get(`${API_BASE}/api/health`);
      if (res.ok()) return;
    } catch {
      // backend aún no disponible
    }
    await new Promise((r) => setTimeout(r, delayMs));
  }
  /* eslint-enable no-await-in-loop */
  throw new Error("Backend health check failed after retries");
}

/** Crea workspace. Devuelve slug + id. */
export async function createWorkspace(
  request: APIRequestContext,
  csrfToken: string,
  slug: string
): Promise<{ slug: string; id: string }> {
  const res = await request.post(`${API_BASE}/api/workspaces`, {
    headers: { "X-CSRFToken": csrfToken },
    data: { name: slug, slug },
  });
  if (!res.ok()) throw new Error(`createWorkspace failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Crea proyecto en workspace. Devuelve id + identifier. */
export async function createProject(
  request: APIRequestContext,
  csrfToken: string,
  slug: string,
  name: string,
  identifier: string
): Promise<{ id: string; identifier: string }> {
  const res = await request.post(`${API_BASE}/api/workspaces/${slug}/projects`, {
    headers: { "X-CSRFToken": csrfToken },
    data: { name, identifier, network: 2 },
  });
  if (!res.ok()) throw new Error(`createProject failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Obtiene estados del proyecto. */
export async function getStates(
  request: APIRequestContext,
  slug: string,
  projectId: string
): Promise<Array<{ id: string; name: string; group: string }>> {
  const res = await request.get(`${API_BASE}/api/workspaces/${slug}/projects/${projectId}/states`);
  if (!res.ok()) throw new Error(`getStates failed: ${res.status()}`);
  return res.json();
}
