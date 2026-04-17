/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitHub personal OAuth callback page.
 *
 * GitHub redirects here after the user authorizes the personal OAuth app:
 *   /auth/github/user-callback?code=XXXXX&state=YYYYY
 *
 * Responsibilities:
 *   1. Read `code` and `state` from the query string.
 *   2. POST `code` to the backend to exchange it for a user access token
 *      and persist the connection.
 *   3. postMessage the outcome — including the opaque `state` — back to
 *      the opener so it can validate the CSRF nonce before trusting the
 *      success/error signal, then close itself.
 *
 * The backend does not consume `state`; CSRF is enforced end-to-end by
 * the opener comparing `state` to the nonce it minted when opening the
 * popup. The state value must therefore be forwarded untouched on every
 * postMessage, including the pre-POST "missing code" error case.
 */

import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
// services
import { GithubUserConnectionService } from "@/services/integrations";
// components
import { OAuthCallbackPage, type TOAuthCallbackStatus } from "@/components/integration/oauth-callback-page";

const service = new GithubUserConnectionService();

export default function GithubUserCallbackPage() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<TOAuthCallbackStatus>("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const called = useRef(false);
  const closeTimer = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const code = searchParams.get("code");
    // GitHub echoes the `state` we sent in the authorize URL. Forward it
    // verbatim — an empty string is still meaningful (it means the
    // opener will reject the message, which is correct when `state` is
    // absent or tampered with).
    const state = searchParams.get("state") ?? "";
    const origin = window.location.origin;

    if (!code) {
      const msg = "Missing authorization code. Please close this window and try again.";
      setErrorMessage(msg);
      setStatus("error");
      window.opener?.postMessage({ type: "github-user-connection", success: false, state, error: msg }, origin);
      return;
    }

    service
      .connectPersonalAccount(code)
      .then(() => {
        setStatus("success");
        window.opener?.postMessage({ type: "github-user-connection", success: true, state }, origin);
        closeTimer.current = setTimeout(() => window.close(), 1500);
        return undefined;
      })
      .catch((err) => {
        const msg = err?.data?.error ?? err?.statusText ?? "Failed to connect GitHub account. Please try again.";
        setErrorMessage(msg);
        setStatus("error");
        window.opener?.postMessage({ type: "github-user-connection", success: false, state, error: msg }, origin);
      });

    return () => {
      if (closeTimer.current !== null) clearTimeout(closeTimer.current);
    };
  }, [searchParams]);

  return (
    <OAuthCallbackPage
      status={status}
      errorMessage={errorMessage}
      processingText="Connecting your GitHub account…"
      successText="GitHub account connected! Closing…"
    />
  );
}
