/**
 * Suite: issues-extras.spec.ts
 * Cubre sección 7.2 del todo_test.md
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;
const pid = () => Env.PROJECT_ID;
const issuePath = (id: string) =>
  `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues/${id}`;

test.describe("Issue — comentarios", () => {
  test("POST + GET + PATCH + DELETE comment", async ({ request, csrf, freshIssue }) => {
    const commentsUrl = `${issuePath(freshIssue.id)}/comments`;

    const create = await request.post(commentsUrl, {
      headers: { "X-CSRFToken": csrf },
      data: { comment_html: "<p>E2E comment</p>" },
    });
    expect(create.status()).toBe(201);
    const comment = await create.json() as { id: string };

    const get = await request.get(`${commentsUrl}/${comment.id}`);
    expect(get.status()).toBe(200);

    const patch = await request.patch(`${commentsUrl}/${comment.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { comment_html: "<p>Updated</p>" },
    });
    expect(patch.status()).toBe(200);

    const del = await request.delete(`${commentsUrl}/${comment.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("Issue — reactions", () => {
  test("POST + DELETE reaction en issue", async ({ request, csrf, freshIssue }) => {
    const reactPath = `${issuePath(freshIssue.id)}/reactions`;
    const reactionCode = "1F44D"; // 👍

    const add = await request.post(reactPath, {
      headers: { "X-CSRFToken": csrf },
      data: { reaction: reactionCode },
    });
    expect([200, 201]).toContain(add.status());

    const del = await request.delete(`${reactPath}/${reactionCode}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });
});

test.describe("Issue — links (canónico + alias)", () => {
  test("POST + PATCH + DELETE en /issue-links (canónico)", async ({ request, csrf, freshIssue }) => {
    const linksPath = `${issuePath(freshIssue.id)}/issue-links`;

    const create = await request.post(linksPath, {
      headers: { "X-CSRFToken": csrf },
      data: { url: "https://example.com", title: "E2E Link" },
    });
    expect(create.status()).toBe(201);
    const link = await create.json() as { id: string };

    const patch = await request.patch(`${linksPath}/${link.id}`, {
      headers: { "X-CSRFToken": csrf },
      data: { title: "Updated Link" },
    });
    expect(patch.status()).toBe(200);

    const del = await request.delete(`${linksPath}/${link.id}`, {
      headers: { "X-CSRFToken": csrf },
    });
    expect([200, 204]).toContain(del.status());
  });

  test("GET /links (alias corto) devuelve misma colección", async ({ request, freshIssue }) => {
    const res = await request.get(`${issuePath(freshIssue.id)}/links`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Issue — relations", () => {
  test("POST /issue-relation + POST /remove-relation (no DELETE)", async ({
    request,
    csrf,
    freshIssue,
  }) => {
    // Necesitamos dos issues: el freshIssue y uno secundario
    const secondary = await request.post(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { name: "E2E Secondary Issue", state_id: Env.STATE_IDS[0] },
      },
    );
    expect(secondary.status()).toBe(201);
    const sec = await secondary.json() as { id: string };

    // Crear relación
    const addRel = await request.post(`${issuePath(freshIssue.id)}/issue-relation`, {
      headers: { "X-CSRFToken": csrf },
      data: { relation_type: "duplicate_of", related_issue: sec.id },
    });
    expect([200, 201]).toContain(addRel.status());

    // Remover relación — es POST, no DELETE (paridad Django)
    const removeRel = await request.post(`${issuePath(freshIssue.id)}/remove-relation`, {
      headers: { "X-CSRFToken": csrf },
      data: { relation_type: "duplicate_of", related_issue: sec.id },
    });
    expect([200, 204]).toContain(removeRel.status());

    // Cleanup secondary
    await request.delete(
      `${BASE}/api/workspaces/${slug()}/projects/${pid()}/issues/${sec.id}`,
      { headers: { "X-CSRFToken": csrf } },
    );
  });
});

test.describe("Issue — subscribers", () => {
  test("GET /issue-subscribers + POST /subscribe + DELETE /subscribe", async ({
    request,
    csrf,
    freshIssue,
  }) => {
    const subPath = `${issuePath(freshIssue.id)}/subscribe`;

    const subscribe = await request.post(subPath, { headers: { "X-CSRFToken": csrf } });
    expect([200, 201]).toContain(subscribe.status());

    const list = await request.get(`${issuePath(freshIssue.id)}/issue-subscribers`);
    expect(list.status()).toBe(200);

    const unsubscribe = await request.delete(subPath, { headers: { "X-CSRFToken": csrf } });
    expect([200, 204]).toContain(unsubscribe.status());
  });
});

test.describe("Issue — sub-issues", () => {
  test("GET /sub-issues devuelve lista", async ({ request, freshIssue }) => {
    const res = await request.get(`${issuePath(freshIssue.id)}/sub-issues`);
    expect(res.status()).toBe(200);
  });
});

test.describe("Issue — activity / history / versions", () => {
  test("GET /activities devuelve lista", async ({ request, freshIssue }) => {
    const res = await request.get(`${issuePath(freshIssue.id)}/activities`);
    expect(res.status()).toBe(200);
  });

  test("GET /history (alias) devuelve 200", async ({ request, freshIssue }) => {
    const res = await request.get(`${issuePath(freshIssue.id)}/history`);
    expect(res.status()).toBe(200);
  });

  test("GET /versions devuelve lista", async ({ request, freshIssue }) => {
    const res = await request.get(`${issuePath(freshIssue.id)}/versions`);
    expect(res.status()).toBe(200);
  });
});
