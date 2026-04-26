/**
 * Suite: intake.spec.ts — Sección 11
 * Cubre aliases: /intakes/ ↔ /inboxes/ y /intake-issues/ ↔ /inbox-issues/
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const base = () => `${BASE}/api/workspaces/${slug()}/projects/${pid()}`;

async function getOrCreateIntake(
  request: import("@playwright/test").APIRequestContext,
  csrf: string,
) {
  // Intentar listar; si no existe crear
  let res = await request.get(`${base()}/intakes`);
  const list = await res.json() as unknown[];
  if (list.length > 0) return (list[0] as { id: string }).id;

  const create = await request.post(`${base()}/intakes`, {
    headers: { "X-CSRFToken": csrf },
    data: { name: `E2E Intake ${Date.now()}` },
  });
  const intake = await create.json() as { id: string };
  return intake.id;
}

test.describe("Intake — aliases intakes / inboxes", () => {
  test("GET /intakes y GET /inboxes devuelven el mismo resultado", async ({ request }) => {
    const [r1, r2] = await Promise.all([
      request.get(`${base()}/intakes`),
      request.get(`${base()}/inboxes`),
    ]);
    expect(r1.status()).toBe(200);
    expect(r2.status()).toBe(200);
  });

  test("POST /intakes crea intake", async ({ request, csrf }) => {
    const res = await request.post(`${base()}/intakes`, {
      headers: { "X-CSRFToken": csrf },
      data: { name: `E2E Intake ${Date.now()}` },
    });
    expect([200, 201]).toContain(res.status());
    const intake = await res.json() as { id: string };

    // Cleanup
    await request.delete(`${base()}/intakes/${intake.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
  });
});

test.describe("Intake — intake-issues / inbox-issues aliases", () => {
  test("GET /intake-issues y GET /inbox-issues devuelven 200", async ({ request, csrf }) => {
    const intakeId = await getOrCreateIntake(request, csrf);

    const [r1, r2] = await Promise.all([
      request.get(`${base()}/intake-issues?intake_id=${intakeId}`),
      request.get(`${base()}/inbox-issues?intake_id=${intakeId}`),
    ]);
    expect(r1.status()).toBe(200);
    expect(r2.status()).toBe(200);
  });

  test("POST /intake-issues crea issue en intake", async ({ request, csrf }) => {
    const intakeId = await getOrCreateIntake(request, csrf);

    const res = await request.post(`${base()}/intake-issues`, {
      headers: { "X-CSRFToken": csrf },
      data: {
        intake_id: intakeId,
        issue: { name: `E2E Intake Issue ${Date.now()}`, state_id: Env.STATE_IDS[0] },
      },
    });
    expect([200, 201]).toContain(res.status());
    const intakeIssue = await res.json() as { id: string };

    // Cleanup
    await request.delete(`${base()}/intake-issues/${intakeIssue.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
  });
});
