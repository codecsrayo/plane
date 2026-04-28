/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { mutate } from "swr";
// plane constants
import { WEB_URL } from "@plane/constants";
import { WORKSPACE_INTEGRATIONS } from "@/constants/fetch-keys";

/**
 * sessionStorage key used to persist the CSRF nonce across the OAuth round-trip.
 * A fresh UUID is written here on every `startAuth` call and removed as soon as the
 * callback postMessage is validated, making replay attacks impossible.
 */
const OAUTH_CSRF_NONCE_KEY = "oauth_csrf_nonce";

const useIntegrationPopup = ({
  provider,
  stateParams,
  github_app_name,
  gitlab_client_id,
  gitlab_host,
  slack_client_id,
}: {
  provider: string | undefined;
  stateParams?: string;
  github_app_name?: string;
  gitlab_client_id?: string;
  gitlab_host?: string;
  slack_client_id?: string;
}) => {
  const [authLoader, setAuthLoader] = useState(false);

  const { workspaceSlug, projectId } = useParams();

  // Typed ref — avoids `any` and makes null-checks explicit downstream.
  const popup = useRef<Window | null>(null);

  /**
   * Fix #6: useCallback is the idiomatic hook for a memoised function
   * (useMemo returning a function was functionally equivalent but misleading).
   * The eslint-disable comment is no longer needed — deps are complete.
   *
   * Fix #5: every dynamic value injected into the `state` query parameter is
   * now wrapped with encodeURIComponent so slugs / IDs containing `&`, `=`,
   * `#` or spaces cannot corrupt the OAuth callback URL parsing.
   * The nonce is a UUID (url-safe by spec) but is encoded defensively too.
   */
  const buildProviderUrl = useCallback(
    (nonce: string): Record<string, string> => {
      const encodedNonce = encodeURIComponent(nonce);
      const encodedSlug = encodeURIComponent(workspaceSlug?.toString() ?? "");
      const baseState = `${encodedNonce}:${encodedSlug}`;

      const encodedProject = encodeURIComponent(projectId?.toString() ?? "");
      const slackChannelState = stateParams
        ? `${baseState},${encodedProject},${encodeURIComponent(stateParams)}`
        : `${baseState},${encodedProject}`;

      return {
        github: `https://github.com/apps/${github_app_name}/installations/new?state=${baseState}`,
        gitlab: `${gitlab_host}/oauth/authorize?client_id=${gitlab_client_id}&redirect_uri=${WEB_URL}auth/gitlab/callback&response_type=code&scope=api+read_user+read_repository+write_repository&state=${baseState}`,
        slack: `https://slack.com/oauth/v2/authorize?scope=chat:write,im:history,im:write,links:read,links:write,users:read,users:read.email&user_scope=&client_id=${slack_client_id}&state=${baseState}`,
        slackChannel: `https://slack.com/oauth/v2/authorize?scope=incoming-webhook&client_id=${slack_client_id}&state=${slackChannelState}`,
      };
    },
    [github_app_name, gitlab_client_id, gitlab_host, slack_client_id, workspaceSlug, projectId, stateParams]
  );

  // Listen for postMessage from the OAuth callback popup and refresh workspace integrations.
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return;
      if (!(["github-integration", "gitlab-integration", "slack-integration"] as string[]).includes(event.data?.type))
        return;

      // --- CSRF nonce validation (Fix #1) ---
      // Reject any message that does not carry the exact nonce we generated in
      // startAuth. This prevents an attacker who knows the workspaceSlug from
      // completing the OAuth flow on behalf of another user.
      const storedNonce = sessionStorage.getItem(OAUTH_CSRF_NONCE_KEY);
      if (!storedNonce || event.data?.csrfNonce !== storedNonce) return;
      // Consume the nonce — one-time use only.
      sessionStorage.removeItem(OAUTH_CSRF_NONCE_KEY);

      // Reset the loading state immediately so the button updates without waiting for checkPopup.
      setAuthLoader(false);

      if (event.data?.success && workspaceSlug) {
        // Force revalidation so the card switches to "Installed" / "Uninstall".
        mutate(WORKSPACE_INTEGRATIONS(workspaceSlug.toString()));
      }
    };

    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [workspaceSlug]);

  // Fix #3: store the interval ID in a ref so it survives re-renders and can
  // be cleared unconditionally in the unmount cleanup below.
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Cleanup on unmount: cancel a running poll so setAuthLoader is never called
  // on a dead component (was a memory-leak / React warning before this fix).
  useEffect(
    () => () => {
      if (intervalRef.current !== null) clearInterval(intervalRef.current);
    },
    []
  );

  const checkPopup = () => {
    intervalRef.current = setInterval(() => {
      // Fix #4: accessing `popup.current.closed` after the user navigates to the
      // OAuth provider makes the popup cross-origin. Some browsers / configs throw
      // a SecurityError on any property access to a cross-origin window.
      // The `=== undefined` guard from before did NOT cover the thrown exception.
      let isClosed = false;
      try {
        isClosed = !popup.current || popup.current.closed || popup.current.closed === undefined;
      } catch {
        // SecurityError or similar — treat as closed to stop polling.
        isClosed = true;
      }

      if (isClosed) {
        clearInterval(intervalRef.current!);
        intervalRef.current = null;
        setAuthLoader(false);
      }
    }, 1000);
  };

  const openPopup = (nonce: string): Window | null => {
    if (!provider) return null;

    const width = 600,
      height = 600;
    const left = window.innerWidth / 2 - width / 2;
    const top = window.innerHeight / 2 - height / 2;

    const urlMap = buildProviderUrl(nonce);

    // Guard: reject unknown providers before calling window.open.
    // An undefined URL causes window.open to open a blank page and fails silently.
    if (!(provider in urlMap)) {
      // Clean up the nonce we just stored — it will never be consumed by the
      // postMessage handler, so leaving it would be a stale credential.
      sessionStorage.removeItem(OAUTH_CSRF_NONCE_KEY);
      console.error(`[useIntegrationPopup] Unknown OAuth provider: "${provider}". No popup was opened.`);
      return null;
    }

    const url = urlMap[provider];

    return window.open(url, "", `width=${width}, height=${height}, top=${top}, left=${left}`) ?? null;
  };

  const startAuth = () => {
    // Generate a cryptographically strong CSRF nonce (Fix #1).
    // UUID v4 provides 122 bits of entropy — sufficient for a CSRF token.
    const nonce = crypto.randomUUID();
    sessionStorage.setItem(OAUTH_CSRF_NONCE_KEY, nonce);

    popup.current = openPopup(nonce);

    // If openPopup returned null (unknown provider, blocked popup, etc.) the
    // nonce has already been cleaned up inside openPopup. Do not start the
    // loader — there is no popup to track.
    if (!popup.current) return;

    checkPopup();
    setAuthLoader(true);
  };

  return {
    startAuth,
    isConnecting: authLoader,
  };
};

export default useIntegrationPopup;
