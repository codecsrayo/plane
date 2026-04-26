/**
 * Suite: spaces-deploy-boards.spec.ts
 * Cubre endpoints project-deploy-boards (sección 5 del todo_test.md)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const boardsPath = () =>
  `${BASE}/api/workspaces/${slug()}/projects/${pid()}/project-deploy-boards`;

test.describe("Deploy Boards", () => {
  test("GET project-deploy-boards devuelve lista", async ({ request }) => {
    const res = await request.get(boardsPath());
    expect(res.status()).toBe(200);
  });

  test("POST project-deploy-boards (upsert) devuelve board", async ({ request, csrf }) => {
    const res = await request.post(boardsPath(), {
      headers: { "X-CSRFToken": csrf },
      data: { is_issues_enabled: true, view_props: {} },
    });
    expect([200, 201]).toContain(res.status());
  });
});
