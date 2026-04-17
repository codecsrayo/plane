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
 */

import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
// services
import { AppInstallationService } from "@/services/app_installation.service";

const appInstallationService = new AppInstallationService();

type TStatus = "processing" | "success" | "error";

export default function GitlabIntegrationCallbackPage() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<TStatus>("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const called = useRef(false); // guard against React double-invoke in dev
  // Fix #3: keep a reference to the auto-close timer so it can be cancelled on unmount.
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const code = searchParams.get("code");

    // Fix #1: state is now `<nonce>:<workspaceSlug>`.
    // Split on the first colon only so workspaceSlugs containing colons are safe.
    const rawState = searchParams.get("state") ?? "";
    const colonIndex = rawState.indexOf(":");
    const csrfNonce = colonIndex !== -1 ? rawState.slice(0, colonIndex) : "";
    const workspaceSlug = colonIndex !== -1 ? rawState.slice(colonIndex + 1) : rawState;

    if (!code || !workspaceSlug) {
      setErrorMessage("Missing authorization code or workspace context. Please close this window and try again.");
      setStatus("error");
      // Notify parent of failure; include nonce so parent can validate even error paths.
      window.opener?.postMessage({ type: "gitlab-integration", success: false, csrfNonce }, window.location.origin);
      return;
    }

    appInstallationService
      .addInstallationApp(workspaceSlug, "gitlab", { code })
      .then((result) => {
        setStatus("success");
        // Notify the parent window (integrations panel) so it can refresh.
        // Return the nonce so the parent can validate the CSRF token.
        window.opener?.postMessage({ type: "gitlab-integration", success: true, csrfNonce }, window.location.origin);
        // Fix #3: store the timer ID so it can be cleared if this component unmounts early.
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

    // Fix #3: cleanup — cancel the auto-close timer if the component unmounts before it fires.
    return () => {
      if (closeTimer.current !== null) clearTimeout(closeTimer.current);
    };
  }, [searchParams]);

  return (
    <div className="flex h-screen w-full items-center justify-center bg-surface-1">
      <div className="shadow-sm flex flex-col items-center gap-4 rounded-lg border border-subtle bg-surface-2 p-10">
        {status === "processing" && (
          <>
            <div className="border-primary-400 h-8 w-8 animate-spin rounded-full border-4 border-t-transparent" />
            <p className="text-body-sm-medium text-secondary">Completing GitLab integration…</p>
          </>
        )}

        {status === "success" && (
          <>
            <svg className="h-12 w-12 text-success-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <p className="text-body-sm-medium">GitLab integrated successfully! Closing…</p>
          </>
        )}

        {status === "error" && (
          <>
            <svg className="text-red-500 h-12 w-12" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
            <p className="text-red-500 text-body-sm-medium">{errorMessage}</p>
            <button className="text-sm mt-2 text-secondary underline" onClick={() => window.close()}>
              Close this window
            </button>
          </>
        )}
      </div>
    </div>
  );
}
