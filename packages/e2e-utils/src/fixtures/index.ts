import { test as base, expect } from "@playwright/test";
import { Env } from "../helpers/env.js";
import { fetchCsrfToken } from "../helpers/api.js";

// ──────────────────────────────────────────────
// Tipos de fixtures
// ──────────────────────────────────────────────
export type PlaneFixtures = {
  /** CSRF token renovado antes de cada test */
  csrf: string;
  /** Issue creado para el test y eliminado al terminar */
  freshIssue: { id: string; name: string };
  freshCycle: { id: string; name: string };
  freshLabel: { id: string; name: string };
  freshModule: { id: string; name: string };
  freshPage: { id: string; title: string };
  freshView: { id: string; name: string };
  freshWebhook: { id: string; url: string };
  freshApiToken: { id: string; token: string };
};

// ──────────────────────────────────────────────
// Helpers internos
// ──────────────────────────────────────────────
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const base = () => Env.API_BASE;

async function del(
  request: import("@playwright/test").APIRequestContext,
  path: string,
  csrf: string,
) {
  await request.delete(`${base()}${path}`, { headers: { "X-CSRFToken": csrf } });
}

// ──────────────────────────────────────────────
// Fixture extension
// ──────────────────────────────────────────────
export const test = base.extend<PlaneFixtures>({
  // CSRF renovado por test
  csrf: async ({ request }, use) => {
    const token = await fetchCsrfToken(request);
    await use(token);
  },

  // Issue
  freshIssue: async ({ request, csrf }, use) => {
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/projects/${pid()}/issues`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: `E2E Issue ${Date.now()}`, state_id: Env.STATE_IDS[0] },
      },
    );
    expect(res.status()).toBe(201);
    const issue = await res.json() as { id: string; name: string };
    await use(issue);
    await del(request, `/api/workspaces/${slug()}/projects/${pid()}/issues/${issue.id}`, csrf);
  },

  // Cycle
  freshCycle: async ({ request, csrf }, use) => {
    const now = new Date();
    const end = new Date(now.getTime() + 7 * 86400_000);
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/projects/${pid()}/cycles`,
      {
        headers: { "X-CSRFToken": csrf },
        data: {
          name: `E2E Cycle ${Date.now()}`,
          start_date: now.toISOString().slice(0, 10),
          end_date: end.toISOString().slice(0, 10),
        },
      },
    );
    expect(res.status()).toBe(201);
    const cycle = await res.json() as { id: string; name: string };
    await use(cycle);
    await del(request, `/api/workspaces/${slug()}/projects/${pid()}/cycles/${cycle.id}`, csrf);
  },

  // Label
  freshLabel: async ({ request, csrf }, use) => {
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/projects/${pid()}/labels`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: `E2E Label ${Date.now()}`, color: "#ff0000" },
      },
    );
    expect(res.status()).toBe(201);
    const label = await res.json() as { id: string; name: string };
    await use(label);
    await del(request, `/api/workspaces/${slug()}/projects/${pid()}/labels/${label.id}`, csrf);
  },

  // Module
  freshModule: async ({ request, csrf }, use) => {
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/projects/${pid()}/modules`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: `E2E Module ${Date.now()}`, status: "in-progress" },
      },
    );
    expect(res.status()).toBe(201);
    const mod = await res.json() as { id: string; name: string };
    await use(mod);
    await del(request, `/api/workspaces/${slug()}/projects/${pid()}/modules/${mod.id}`, csrf);
  },

  // Page
  freshPage: async ({ request, csrf }, use) => {
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/projects/${pid()}/pages`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { title: `E2E Page ${Date.now()}` },
      },
    );
    expect(res.status()).toBe(201);
    const page = await res.json() as { id: string; title: string };
    await use(page);
    await del(request, `/api/workspaces/${slug()}/projects/${pid()}/pages/${page.id}`, csrf);
  },

  // View (workspace-level)
  freshView: async ({ request, csrf }, use) => {
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/views`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: `E2E View ${Date.now()}`, filters: {} },
      },
    );
    expect(res.status()).toBe(201);
    const view = await res.json() as { id: string; name: string };
    await use(view);
    await del(request, `/api/workspaces/${slug()}/views/${view.id}`, csrf);
  },

  // Webhook
  freshWebhook: async ({ request, csrf }, use) => {
    const hookUrl = `https://webhook.site/e2e-${Date.now()}`;
    const res = await request.post(
      `${base()}/api/workspaces/${slug()}/webhooks`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { url: hookUrl, is_active: true },
      },
    );
    expect(res.status()).toBe(201);
    const hook = await res.json() as { id: string; url: string };
    await use(hook);
    await del(request, `/api/workspaces/${slug()}/webhooks/${hook.id}`, csrf);
  },

  // API Token
  freshApiToken: async ({ request, csrf }, use) => {
    const res = await request.post(`${base()}/api/api-tokens`, {
      headers: { "X-CSRFToken": csrf },
      data: { label: `e2e-token-${Date.now()}` },
    });
    expect(res.status()).toBe(201);
    const tok = await res.json() as { id: string; token: string };
    await use(tok);
    await del(request, `/api/api-tokens/${tok.id}`, csrf);
  },
});

export { expect };
