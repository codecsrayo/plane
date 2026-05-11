/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import useSWR, { mutate } from "swr";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
// plane constants
import { WEB_URL } from "@plane/constants";
import { UserService } from "@/services/user.service";

type Props = {
  githubClientId: string;
};

type TUserAccount = {
  id: string;
  provider: string;
  metadata?: {
    github_username?: string;
    github_avatar_url?: string;
  } | null;
};

type TGithubPersonalConnection = {
  github_username: string;
  github_avatar_url: string;
};

const userService = new UserService();

const CURRENT_USER_ACCOUNTS_KEY = "CURRENT_USER_ACCOUNTS";

/**
 * sessionStorage key for the one-shot CSRF nonce that guards the personal
 * GitHub OAuth round-trip. Distinct from the workspace-integration key
 * (`useIntegrationPopup::OAUTH_CSRF_NONCE_KEY`) so the two flows cannot
 * collide if a user opens both at once.
 */
const PERSONAL_OAUTH_CSRF_NONCE_KEY = "github_personal_oauth_csrf_nonce";

const getGithubPersonalConnection = (accounts: TUserAccount[] | undefined): TGithubPersonalConnection | null => {
  const githubAccount = accounts?.find((account) => account.provider === "github");
  const githubUsername = githubAccount?.metadata?.github_username;

  if (!githubUsername) return null;

  return {
    github_username: githubUsername,
    github_avatar_url: githubAccount?.metadata?.github_avatar_url ?? "",
  };
};

export function GithubPersonalConnectCard({ githubClientId }: Props) {
  const [isConnecting, setIsConnecting] = useState(false);
  const popup = useRef<Window | null>(null);
  const pollInterval = useRef<ReturnType<typeof setInterval> | null>(null);

  const { data: accounts } = useSWR<TUserAccount[]>(CURRENT_USER_ACCOUNTS_KEY, () =>
    userService.getCurrentUserAccounts()
  );
  const personalConnection = getGithubPersonalConnection(accounts);

  // The OAuth URL only changes when the clientId does; memoise to avoid
  // reconstructing the string on every parent re-render. Redirect URI is
  // URL-encoded so provider hosts that reject unescaped path characters
  // (colons, slashes in edge cases) accept the round-trip.
  const oauthUrlTemplate = useMemo(() => {
    if (!githubClientId) return null;
    const redirectUri = encodeURIComponent(`${WEB_URL}auth/github/user-callback`);
    const scope = encodeURIComponent("read:user user:email");
    return `https://github.com/login/oauth/authorize?client_id=${githubClientId}&scope=${scope}&redirect_uri=${redirectUri}`;
  }, [githubClientId]);

  const stopPolling = useCallback(() => {
    if (pollInterval.current !== null) {
      clearInterval(pollInterval.current);
      pollInterval.current = null;
    }
  }, []);

  const handleMessage = useCallback(
    (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return;
      if (event.data?.type !== "github-user-connection") return;

      // CSRF validation: the callback page echoes back the `state` value
      // that GitHub returned, which must match the nonce we minted in
      // openPersonalOAuth. A missing, stale, or mismatched nonce means
      // this message is not ours — drop it.
      const storedNonce = sessionStorage.getItem(PERSONAL_OAUTH_CSRF_NONCE_KEY);
      if (!storedNonce || event.data?.state !== storedNonce) return;
      // One-shot: consume the nonce as soon as it validates.
      sessionStorage.removeItem(PERSONAL_OAUTH_CSRF_NONCE_KEY);

      stopPolling();
      setIsConnecting(false);

      if (event.data?.success) {
        mutate(CURRENT_USER_ACCOUNTS_KEY);
        return;
      }

      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Failed to connect GitHub account",
        message: event.data?.error ?? "Please try again.",
      });
    },
    [stopPolling]
  );

  useEffect(() => {
    window.addEventListener("message", handleMessage);
    return () => {
      window.removeEventListener("message", handleMessage);
      // On unmount: clear any in-flight poll + discard a stale nonce so
      // the next mount does not accept a message from a popup that was
      // opened before the component was replaced.
      stopPolling();
      sessionStorage.removeItem(PERSONAL_OAUTH_CSRF_NONCE_KEY);
    };
  }, [handleMessage, stopPolling]);

  const openPersonalOAuth = () => {
    if (!oauthUrlTemplate) return;

    // UUID v4 — 122 bits of entropy, same bar as the workspace popup.
    const nonce = crypto.randomUUID();
    sessionStorage.setItem(PERSONAL_OAUTH_CSRF_NONCE_KEY, nonce);
    const authUrl = `${oauthUrlTemplate}&state=${encodeURIComponent(nonce)}`;

    const width = 600;
    const height = 600;
    const left = window.innerWidth / 2 - width / 2;
    const top = window.innerHeight / 2 - height / 2;

    popup.current = window.open(authUrl, "", `width=${width},height=${height},top=${top},left=${left}`);
    setIsConnecting(true);

    // Guard against the "user dismisses the popup without completing
    // OAuth" path. Without this, `isConnecting` used to stay `true`
    // forever and the only way to recover was a full page reload.
    stopPolling();
    pollInterval.current = setInterval(() => {
      if (!popup.current || popup.current.closed) {
        stopPolling();
        setIsConnecting(false);
        // Popup gone without a validated postMessage — drop the nonce
        // so it cannot be replayed later.
        sessionStorage.removeItem(PERSONAL_OAUTH_CSRF_NONCE_KEY);
      }
    }, 1000);
  };

  if (!githubClientId) return null;

  return (
    <div className="border-custom-border-200 bg-custom-background-100 gap-y-4 rounded-lg border p-5">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-sm text-custom-text-100 font-semibold">Your GitHub Account</h2>
          <p className="text-xs text-custom-text-300 mt-1">
            Connect your personal GitHub account to enable user-level actions.
          </p>
        </div>
        {personalConnection ? (
          <div className="flex items-center gap-2">
            {personalConnection.github_avatar_url && (
              <img
                src={personalConnection.github_avatar_url}
                alt={personalConnection.github_username}
                className="size-6 rounded-full"
              />
            )}
            <span className="text-xs text-custom-text-100 font-medium">@{personalConnection.github_username}</span>
          </div>
        ) : (
          <Button
            variant="primary"
            size="sm"
            onClick={openPersonalOAuth}
            loading={isConnecting}
            disabled={isConnecting}
          >
            {isConnecting ? "Connecting..." : "Connect account"}
          </Button>
        )}
      </div>
    </div>
  );
}
