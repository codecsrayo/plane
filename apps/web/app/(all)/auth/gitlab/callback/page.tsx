/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitLab OAuth callback page.
 *
 * GitLab redirects here after the user authorizes the OAuth app:
 *   /auth/gitlab/callback?code=XXXXX&state={workspaceSlug}
 *
 * This page:
 *   1. Reads code and state (workspaceSlug) from the query string
 *   2. POSTs to the backend to create the WorkspaceIntegration record
 *   3. Communicates the result back to the parent window (the integrations panel)
 *      via postMessage, then closes itself.
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

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const code = searchParams.get("code");
    // state param carries workspaceSlug (set in use-integration-popup.tsx)
    const workspaceSlug = searchParams.get("state");

    if (!code || !workspaceSlug) {
      setErrorMessage("Missing authorization code or workspace context. Please close this window and try again.");
      setStatus("error");
      // Notify parent of failure
      window.opener?.postMessage({ type: "gitlab-integration", success: false }, window.location.origin);
      return;
    }

    appInstallationService
      .addInstallationApp(workspaceSlug, "gitlab", { code })
      .then((result) => {
        setStatus("success");
        // Notify the parent window (integrations panel popup) so it can refresh
        window.opener?.postMessage({ type: "gitlab-integration", success: true }, window.location.origin);
        // Auto-close after a short delay so the user sees the success state
        setTimeout(() => window.close(), 1500);
        return result;
      })
      .catch((err) => {
        const msg = err?.data?.error ?? err?.error ?? err?.statusText ?? "Failed to complete GitLab integration. Please try again.";
        setErrorMessage(msg);
        setStatus("error");
        window.opener?.postMessage({ type: "gitlab-integration", success: false, error: msg }, window.location.origin);
      });
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
