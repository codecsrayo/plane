/**
 * globalTeardown — ejecutado UNA VEZ al finalizar toda la suite.
 * Elimina: API token → workspace (cascade) → usuario de prueba.
 */
import { request as playwrightRequest } from "@playwright/test";
import * as fs from "fs";
import * as path from "path";
import { fetchCsrfToken, signIn } from "../helpers/api.js";

const API_BASE = process.env.API_BASE_URL ?? "http://localhost:8000";
const STORAGE_PATH = process.env.E2E_STORAGE_PATH ?? "state.json";

export default async function globalTeardown(): Promise<void> {
  const seedPath = path.resolve(path.dirname(STORAGE_PATH), "e2e-seed.json");
  if (!fs.existsSync(seedPath)) {
    console.warn("globalTeardown: no seed metadata found, skipping cleanup.");
    return;
  }

  const seed = JSON.parse(fs.readFileSync(seedPath, "utf-8")) as {
    slug: string;
    email: string;
    apiTokenId?: string;
  };

  const ctx = await playwrightRequest.newContext({
    baseURL: API_BASE,
    storageState: fs.existsSync(STORAGE_PATH) ? STORAGE_PATH : undefined,
  });

  try {
    const csrfToken = await fetchCsrfToken(ctx);
    await signIn(ctx, seed.email, process.env.E2E_PASSWORD ?? "PlaneE2E!2024", csrfToken);

    // Eliminar API token si existe
    if (seed.apiTokenId) {
      await ctx.delete(`${API_BASE}/api/api-tokens/${seed.apiTokenId}`, {
        headers: { "X-CSRFToken": csrfToken },
      });
    }

    // Eliminar workspace (cascade: proyectos, issues, cycles, etc.)
    const delWs = await ctx.delete(`${API_BASE}/api/workspaces/${seed.slug}`, {
      headers: { "X-CSRFToken": csrfToken },
    });
    console.log(`🗑  workspace ${seed.slug} deleted: ${delWs.status()}`);

    // Eliminar usuario de prueba solo si fue creado por el setup
    if (process.env.E2E_DELETE_USER === "true") {
      const delUser = await ctx.delete(`${API_BASE}/api/users/me`, {
        headers: { "X-CSRFToken": csrfToken },
      });
      console.log(`🗑  test user deleted: ${delUser.status()}`);
    }

    // Limpiar artefactos de seed
    fs.rmSync(seedPath, { force: true });
    fs.rmSync(STORAGE_PATH, { force: true });
  } catch (err) {
    console.error("globalTeardown error:", err);
  } finally {
    await ctx.dispose();
  }
}
