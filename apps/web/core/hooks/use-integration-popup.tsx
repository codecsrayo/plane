/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useRef, useState } from "react";
import { useParams } from "react-router";
import { mutate } from "swr";
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
   * Build the provider OAuth URL at call-time so the CSRF nonce is always
   * fresh. The nonce is embedded as the first segment of the `state` param
   * using the format `<nonce>:<workspaceSlug>[,<extras>]`.
   * Callbacks parse this format to recover workspaceSlug and return the nonce
   * in the postMessage payload for validation.
   */
  const buildProviderUrl = (nonce: string): Record<string, string> => {
    const baseState = `${nonce}:${workspaceSlug?.toString()}`;
    return {
      github: `https://github.com/apps/${github_app_name}/installations/new?state=${baseState}`,
      gitlab: `${gitlab_host}/oauth/authorize?client_id=${gitlab_client_id}&redirect_uri=${window.location.origin}/auth/gitlab/callback&response_type=code&scope=api+read_user+read_repository+write_repository&state=${baseState}`,
      slack: `https://slack.com/oauth/v2/authorize?scope=chat:write,im:history,im:write,links:read,links:write,users:read,users:read.email&user_scope=&client_id=${slack_client_id}&state=${baseState}`,
      slackChannel: `https://slack.com/oauth/v2/authorize?scope=incoming-webhook&client_id=${slack_client_id}&state=${baseState},${projectId?.toString()}${
        stateParams ? "," + stateParams : ""
      }`,
    };
  };

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

  const checkPopup = () => {
    const check = setInterval(() => {
      // Fix #2: Guard against null before accessing `.closed`.
      // `popup.current` is null when the browser blocks the popup window,
      // which would otherwise cause a TypeError crash.
      if (!popup.current || popup.current.closed || popup.current.closed === undefined) {
        clearInterval(check);
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
    const url = buildProviderUrl(nonce)[provider];

    return window.open(url, "", `width=${width}, height=${height}, top=${top}, left=${left}`) ?? null;
  };

  const startAuth = () => {
    // Generate a cryptographically strong CSRF nonce (Fix #1).
    // UUID v4 provides 122 bits of entropy — sufficient for a CSRF token.
    const nonce = crypto.randomUUID();
    sessionStorage.setItem(OAUTH_CSRF_NONCE_KEY, nonce);

    popup.current = openPopup(nonce);
    checkPopup();
    setAuthLoader(true);
  };

  return {
    startAuth,
    isConnecting: authLoader,
  };
};

export default useIntegrationPopup;
