import type { APIRequestContext } from "@playwright/test";

export const API_BASE = process.env.API_BASE_URL ?? "http://localhost:8000";

/** Obtiene CSRF token y lo inyecta en el contexto. Devuelve el token. */
export async function fetchCsrfToken(request: APIRequestContext): Promise<string> {
  const res = await request.get(`${API_BASE}/auth/get-csrf-token`);
  if (!res.ok()) throw new Error(`CSRF fetch failed: ${res.status()}`);
  const body = await res.json();
  return (body.csrf_token ?? body.csrfToken) as string;
}

/** Sign-in con email/password. Devuelve el body del usuario. */
export async function signIn(
  request: APIRequestContext,
  email: string,
  password: string,
  csrfToken: string,
): Promise<Record<string, unknown>> {
  const res = await request.post(`${API_BASE}/auth/sign-in`, {
    headers: { "X-CSRFToken": csrfToken },
    data: { email, password },
  });
  if (!res.ok()) throw new Error(`sign-in failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Sign-up. Devuelve el body del usuario creado. */
export async function signUp(
  request: APIRequestContext,
  email: string,
  password: string,
  csrfToken: string,
): Promise<Record<string, unknown>> {
  const res = await request.post(`${API_BASE}/auth/sign-up`, {
    headers: { "X-CSRFToken": csrfToken },
    data: { email, password },
  });
  if (!res.ok()) throw new Error(`sign-up failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Espera a que el backend responda /api/health con 200. */
export async function waitForHealth(
  request: APIRequestContext,
  retries = 10,
  delayMs = 2000,
): Promise<void> {
  for (let i = 0; i < retries; i++) {
    try {
      const res = await request.get(`${API_BASE}/api/health`);
      if (res.ok()) return;
    } catch {
      // backend aún no disponible
    }
    await new Promise((r) => setTimeout(r, delayMs));
  }
  throw new Error("Backend health check failed after retries");
}

/** Crea workspace. Devuelve slug + id. */
export async function createWorkspace(
  request: APIRequestContext,
  csrfToken: string,
  slug: string,
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
  identifier: string,
): Promise<{ id: string; identifier: string }> {
  const res = await request.post(
    `${API_BASE}/api/workspaces/${slug}/projects`,
    {
      headers: { "X-CSRFToken": csrfToken },
      data: { name, identifier, network: 2 },
    },
  );
  if (!res.ok()) throw new Error(`createProject failed: ${res.status()} ${await res.text()}`);
  return res.json();
}

/** Obtiene estados del proyecto. */
export async function getStates(
  request: APIRequestContext,
  slug: string,
  projectId: string,
): Promise<Array<{ id: string; name: string; group: string }>> {
  const res = await request.get(
    `${API_BASE}/api/workspaces/${slug}/projects/${projectId}/states`,
  );
  if (!res.ok()) throw new Error(`getStates failed: ${res.status()}`);
  return res.json();
}
