/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitLab OAuth callback page.
 *
 * GitLab redirects here after the user authorizes the OAuth app:
 *   /auth/gitlab/callback?code=XXXXX&state=<nonce>:<workspaceSlug>
 *
 * This page:
 *   1. Reads code and state from the query string
 *   2. Parses state as `<nonce>:<workspaceSlug>` — the nonce is sent back in
 *      the postMessage payload so the parent window can validate the CSRF token.
 *   3. POSTs to the backend to create the WorkspaceIntegration record
 *   4. Communicates the result back to the parent window via postMessage, then
 *      closes itself. The auto-close timer is cleared on unmount to avoid the
 *      memory-leak that occurs when the component is torn down before the delay
 *      fires.
 *
 * UI delegated to the shared OAuthCallbackPage component (Fix #4).
 */

import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import type { TGitlabInstallPayload } from "@plane/types";
// services
import { AppInstallationService } from "@/services/app_installation.service";
// components
import { OAuthCallbackPage, type TOAuthCallbackStatus } from "@/components/integration/oauth-callback-page";

const appInstallationService = new AppInstallationService();

export default function GitlabIntegrationCallbackPage() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<TOAuthCallbackStatus>("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const called = useRef(false); // guard against React double-invoke in dev
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const code = searchParams.get("code");

    // state is `<nonce>:<workspaceSlug>` — split on first colon only so
    // workspaceSlugs containing colons remain intact.
    const rawState = searchParams.get("state") ?? "";
    const colonIndex = rawState.indexOf(":");
    const csrfNonce = colonIndex !== -1 ? rawState.slice(0, colonIndex) : "";
    const workspaceSlug = colonIndex !== -1 ? rawState.slice(colonIndex + 1) : rawState;

    if (!code || !workspaceSlug) {
      setErrorMessage("Missing authorization code or workspace context. Please close this window and try again.");
      setStatus("error");
      window.opener?.postMessage({ type: "gitlab-integration", success: false, csrfNonce }, window.location.origin);
      return;
    }

    const installPayload: TGitlabInstallPayload = { code };

    appInstallationService
      .addInstallationApp(workspaceSlug, "gitlab", installPayload)
      .then((result) => {
        setStatus("success");
        window.opener?.postMessage({ type: "gitlab-integration", success: true, csrfNonce }, window.location.origin);
        closeTimer.current = setTimeout(() => window.close(), 1500);
        return result;
      })
      .catch((err) => {
        const msg =
          err?.data?.error ?? err?.error ?? err?.statusText ?? "Failed to complete GitLab integration. Please try again.";
        setErrorMessage(msg);
        setStatus("error");
        window.opener?.postMessage(
          { type: "gitlab-integration", success: false, error: msg, csrfNonce },
          window.location.origin
        );
      });

    return () => {
      if (closeTimer.current !== null) clearTimeout(closeTimer.current);
    };
  }, [searchParams]);

  return (
    <OAuthCallbackPage
      status={status}
      errorMessage={errorMessage}
      processingText="Completing GitLab integration…"
      successText="GitLab integrated successfully! Closing…"
    />
  );
}
