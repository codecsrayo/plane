/**
 * globalSetup — ejecutado UNA VEZ antes de toda la suite.
 * Orden: health → CSRF → sign-up/sign-in → workspace → project → states → API token
 * Persiste storageState + variables de entorno vía process.env para uso en specs.
 */
import { request as playwrightRequest } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import {
  waitForHealth,
  fetchCsrfToken,
  signIn,
  signUp,
  ensureInstanceConfigured,
  createWorkspace,
  createProject,
  getStates,
} from "../helpers/api.js";

const API_BASE = process.env.API_BASE_URL ?? "http://localhost:8000";
const EMAIL = process.env.E2E_EMAIL ?? `e2e-${Date.now()}@plane-test.local`;
const PASSWORD = process.env.E2E_PASSWORD ?? "PlaneE2E!2024";
const STORAGE_PATH = process.env.E2E_STORAGE_PATH ?? "state.json";
const ADMIN_EMAIL = process.env.E2E_ADMIN_EMAIL ?? EMAIL;
const ADMIN_PASSWORD = process.env.E2E_ADMIN_PASSWORD ?? PASSWORD;
const ADMIN_FIRST_NAME = process.env.E2E_ADMIN_FIRST_NAME ?? "E2E";

export default async function globalSetup(): Promise<void> {
  const ctx = await playwrightRequest.newContext({ baseURL: API_BASE });

  try {
    // 1. Esperar backend
    await waitForHealth(ctx);

    // 2. Asegurar que la instancia este configurada (idempotente).
    //    Usa context propio para no dejar cookies de admin sobre `ctx`.
    const adminCtx = await playwrightRequest.newContext({ baseURL: API_BASE });
    try {
      await ensureInstanceConfigured(adminCtx, ADMIN_EMAIL, ADMIN_PASSWORD, ADMIN_FIRST_NAME);
    } finally {
      await adminCtx.dispose();
    }

    // 3. CSRF
    const csrfToken = await fetchCsrfToken(ctx);

    // 4. Intentar sign-in; si falla, sign-up
    let user: Record<string, unknown>;
    try {
      user = await signIn(ctx, EMAIL, PASSWORD, csrfToken);
    } catch {
      user = await signUp(ctx, EMAIL, PASSWORD, csrfToken);
    }

    // 5. Persistir storage state (cookies de sesión)
    await ctx.storageState({ path: STORAGE_PATH });

    // 6. Crear workspace con slug único
    const slug = `e2e-${Date.now()}`;
    const workspace = await createWorkspace(ctx, csrfToken, slug);
    process.env["E2E_WORKSPACE_SLUG"] = workspace.slug;

    // 7. Crear proyecto
    const project = await createProject(
      ctx,
      csrfToken,
      workspace.slug,
      "E2E Project",
      `EP${Date.now().toString().slice(-4)}`
    );
    process.env["E2E_PROJECT_ID"] = project.id;

    // 8. Recuperar estados por defecto (Backlog, Todo, In Progress, Done)
    const states = await getStates(ctx, workspace.slug, project.id);
    process.env["E2E_STATE_IDS"] = JSON.stringify(states.map((s) => s.id));

    // 9. Crear API token para tests de v1
    const tokenRes = await ctx.post(`${API_BASE}/api/api-tokens`, {
      headers: { "X-CSRFToken": csrfToken },
      data: { label: "e2e-test-token" },
    });
    if (tokenRes.ok()) {
      const token = (await tokenRes.json()) as { token?: string };
      process.env["E2E_API_TOKEN"] = token.token ?? "";
      process.env["E2E_API_TOKEN_ID"] = ((token as Record<string, unknown>)["id"] as string) ?? "";
    }

    // 10. Persistir context de seed para teardown
    const seedMeta = {
      slug: workspace.slug,
      email: EMAIL,
      userId: (user as Record<string, unknown>)["id"],
      apiTokenId: process.env["E2E_API_TOKEN_ID"],
    };
    fs.writeFileSync(path.resolve(path.dirname(STORAGE_PATH), "e2e-seed.json"), JSON.stringify(seedMeta, null, 2));

    console.log(`✅ globalSetup: workspace=${workspace.slug} project=${project.id}`);
  } finally {
    await ctx.dispose();
  }
}
