/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitHub App installation callback page.
 *
 * GitHub redirects here after the user installs the App:
 *   /auth/github/callback?installation_id=XXXXX&state={workspaceSlug}
 *
 * This page:
 *   1. Reads installation_id and state (workspaceSlug) from the query string
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

export default function GithubIntegrationCallbackPage() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<TStatus>("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const called = useRef(false); // guard against React double-invoke in dev

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const installation_id = searchParams.get("installation_id");
    const setup_action = searchParams.get("setup_action"); // "install" or "update"
    // state param carries workspaceSlug
    const workspaceSlug = searchParams.get("state");
    // api_installed=true means the API callback already saved the WorkspaceIntegration.
    // Just notify the parent and close — no need to POST again.
    const apiInstalled = searchParams.get("api_installed") === "true";

    if (!installation_id || !workspaceSlug) {
      setErrorMessage("Missing installation_id or workspace context. Please close this window and try again.");
      setStatus("error");
      window.opener?.postMessage({ type: "github-integration", success: false }, window.location.origin);
      return;
    }

    if (apiInstalled) {
      // Backend already handled the install via /api/github/callback/ — just notify and close.
      setStatus("success");
      window.opener?.postMessage({ type: "github-integration", success: true }, window.location.origin);
      setTimeout(() => window.close(), 1500);
      return;
    }

    appInstallationService
      .addInstallationApp(workspaceSlug, "github", {
        installation_id,
        setup_action: setup_action ?? "install",
      })
      .then((result) => {
        setStatus("success");
        window.opener?.postMessage({ type: "github-integration", success: true }, window.location.origin);
        setTimeout(() => window.close(), 1500);
        return result;
      })
      .catch((err) => {
        const msg = err?.data?.error ?? err?.error ?? err?.statusText ?? "Failed to complete GitHub integration. Please try again.";
        setErrorMessage(msg);
        setStatus("error");
        window.opener?.postMessage({ type: "github-integration", success: false, error: msg }, window.location.origin);
      });
  }, [searchParams]);

  return (
    <div className="flex h-screen w-full items-center justify-center bg-surface-1">
      <div className="shadow-sm flex flex-col items-center gap-4 rounded-lg border border-subtle bg-surface-2 p-10">
        {status === "processing" && (
          <>
            <div className="border-primary-400 h-8 w-8 animate-spin rounded-full border-4 border-t-transparent" />
            <p className="text-body-sm-medium text-secondary">Completing GitHub integration…</p>
          </>
        )}

        {status === "success" && (
          <>
            <svg className="h-12 w-12 text-success-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <p className="text-body-sm-medium">GitHub integrated successfully! Closing…</p>
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
