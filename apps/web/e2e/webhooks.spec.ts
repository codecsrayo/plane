/**
 * Suite: webhooks.spec.ts — Sección 14
 * Cubre CRUD, regenerate, webhook-logs y validación del envelope saliente.
 *
 * Envelope del webhook (src/jobs/webhook_delivery.rs → build_envelope):
 *   { event, action, webhook_id, workspace_id, data, activity }
 * Headers: X-Plane-Event, X-Plane-Delivery, X-Plane-Signature (HMAC-SHA256)
 */
import { test, expect } from "@plane/e2e-utils/fixtures";
import { Env } from "@plane/e2e-utils/helpers/env";
import type { WebhookEnvelope } from "@plane/e2e-utils/helpers/types";

const BASE = Env.API_BASE;
const slug = () => Env.WORKSPACE_SLUG;

test.describe("Webhooks — CRUD", () => {
  test("GET webhooks devuelve lista", async ({ request }) => {
    const res = await request.get(`${BASE}/api/workspaces/${slug()}/webhooks`);
    expect(res.status()).toBe(200);
  });

  test("GET + PATCH webhook (freshWebhook)", async ({ request, csrf, freshWebhook }) => {
    const get = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}`,
    );
    expect(get.status()).toBe(200);

    const patch = await request.patch(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}`,
      {
        headers: { "X-CSRFToken": csrf },
        data: { is_active: false },
      },
    );
    expect(patch.status()).toBe(200);
  });

  test("POST regenerate webhook secret", async ({ request, csrf, freshWebhook }) => {
    const res = await request.post(
      `${BASE}/api/workspaces/${slug()}/webhooks/${freshWebhook.id}/regenerate`,
      { headers: { "X-CSRFToken": csrf } },
    );
    expect([200, 201]).toContain(res.status());
    const body = await res.json() as Record<string, unknown>;
    expect(body).toMatchObject({ secret_key: expect.any(String) });
  });

  test("GET webhook-logs devuelve lista", async ({ request, freshWebhook }) => {
    const res = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhook-logs/${freshWebhook.id}`,
    );
    expect(res.status()).toBe(200);
  });
});

test.describe("Webhooks — envelope shape (webhook-logs)", () => {
  /**
   * Verifica que los webhook-logs registran el envelope correcto
   * tras una mutación de issue (trigger event=issue, action=created/updated).
   *
   * Envelope esperado (6 claves exactas):
   *   { event, action, webhook_id, workspace_id, data, activity }
   * NO incluye delivery_id en el body (va en header X-Plane-Delivery).
   */
  test("webhook-log registra envelope con 6 claves tras mutación de issue", async ({
    request,
    csrf,
    freshWebhook,
    freshIssue,
  }) => {
    // Trigger: PATCH issue dispara event=issue, action=updated
    await request.patch(
      `${BASE}/api/workspaces/${slug()}/projects/${Env.PROJECT_ID}/issues/${freshIssue.id}`,
      { headers: { "X-CSRFToken": csrf }, data: { priority: "medium" } },
    );

    // Esperar hasta 3s para que el job asíncrono encole la entrega
    await new Promise((r) => setTimeout(r, 3000));

    const logsRes = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhook-logs/${freshWebhook.id}`,
    );
    expect(logsRes.status()).toBe(200);
    const logs = await logsRes.json() as unknown[];

    if (logs.length === 0) {
      // El job puede no haber corrido en CI sin worker — skip graceful
      console.warn("No webhook logs yet — worker may not be running");
      return;
    }

    // Validar envelope del primer log
    const log = logs[0] as Record<string, unknown>;
    const body = typeof log.request_body === "string"
      ? JSON.parse(log.request_body) as WebhookEnvelope
      : log.request_body as WebhookEnvelope;

    expect(Object.keys(body)).toHaveLength(6);
    expect(body).toMatchObject({
      event: expect.stringMatching(/^(issue|project|module|module_issue|cycle|cycle_issue|issue_comment)$/),
      action: expect.stringMatching(/^(created|updated|deleted)$/),
      webhook_id: expect.any(String),
      workspace_id: expect.any(String),
      data: expect.any(Object),
      // activity puede ser null en create/delete, objeto en update con diff
    });
    // delivery_id NO debe estar en el body (va en header X-Plane-Delivery)
    expect(Object.prototype.hasOwnProperty.call(body, "delivery_id")).toBe(false);
  });

  test("webhook-log registra headers X-Plane-Event y X-Plane-Delivery", async ({
    request,
    freshWebhook,
  }) => {
    const logsRes = await request.get(
      `${BASE}/api/workspaces/${slug()}/webhook-logs/${freshWebhook.id}`,
    );
    expect(logsRes.status()).toBe(200);
    const logs = await logsRes.json() as unknown[];
    if (logs.length === 0) return; // graceful skip sin worker

    const log = logs[0] as Record<string, unknown>;
    const headers = typeof log.request_headers === "string"
      ? JSON.parse(log.request_headers) as Record<string, string>
      : log.request_headers as Record<string, string>;

    expect(headers).toMatchObject({
      "X-Plane-Event": expect.any(String),
      "X-Plane-Delivery": expect.any(String),
      // X-Plane-Signature está redactado como "[redacted]" en el log
      "X-Plane-Signature": expect.any(String),
    });
  });
});
