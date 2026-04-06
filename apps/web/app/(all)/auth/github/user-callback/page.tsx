/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * GitHub personal OAuth callback page.
 *
 * GitHub redirects here after the user authorizes the personal OAuth app:
 *   /auth/github/user-callback?code=XXXXX
 *
 * This page:
 *   1. Reads code from the query string
 *   2. POSTs to the backend to exchange code for a user access token and save the connection
 *   3. Communicates the result back to the parent window via postMessage, then closes itself.
 */

import { useEffect, useRef, useState } from "react";
import { useSearchParams } from "react-router";
import { APIService } from "@/services/api.service";
import { API_BASE_URL } from "@plane/constants";

type TStatus = "processing" | "success" | "error";

class GithubUserConnectionService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async connectPersonalAccount(code: string): Promise<any> {
    return this.post("/api/auth/github/user-callback/", { code })
      .then((res) => res?.data)
      .catch((err) => { throw err?.response; });
  }
}

const service = new GithubUserConnectionService();

export default function GithubUserCallbackPage() {
  const [searchParams] = useSearchParams();
  const [status, setStatus] = useState<TStatus>("processing");
  const [errorMessage, setErrorMessage] = useState<string>("");
  const called = useRef(false);

  useEffect(() => {
    if (called.current) return;
    called.current = true;

    const code = searchParams.get("code");

    if (!code) {
      setErrorMessage("Missing authorization code. Please close this window and try again.");
      setStatus("error");
      window.opener?.postMessage({ type: "github-user-connection", success: false }, window.location.origin);
      return;
    }

    service
      .connectPersonalAccount(code)
      .then((result) => {
        setStatus("success");
        window.opener?.postMessage({ type: "github-user-connection", success: true }, window.location.origin);
        setTimeout(() => window.close(), 1500);
        return result;
      })
      .catch((err) => {
        const msg = err?.data?.error ?? err?.statusText ?? "Failed to connect GitHub account. Please try again.";
        setErrorMessage(msg);
        setStatus("error");
        window.opener?.postMessage({ type: "github-user-connection", success: false, error: msg }, window.location.origin);
      });
  }, [searchParams]);

  return (
    <div className="flex h-screen w-full items-center justify-center bg-surface-1">
      <div className="shadow-sm flex flex-col items-center gap-4 rounded-lg border border-subtle bg-surface-2 p-10">
        {status === "processing" && (
          <>
            <div className="border-primary-400 h-8 w-8 animate-spin rounded-full border-4 border-t-transparent" />
            <p className="text-body-sm-medium text-secondary">Connecting your GitHub account…</p>
          </>
        )}
        {status === "success" && (
          <>
            <svg className="h-12 w-12 text-success-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
            <p className="text-body-sm-medium">GitHub account connected! Closing…</p>
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
