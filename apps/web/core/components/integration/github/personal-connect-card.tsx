/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useCallback, useEffect, useState } from "react";
import useSWR, { mutate } from "swr";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
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

  const { data: accounts } = useSWR<TUserAccount[]>(CURRENT_USER_ACCOUNTS_KEY, () =>
    userService.getCurrentUserAccounts()
  );
  const personalConnection = getGithubPersonalConnection(accounts);

  const oauthUrl = githubClientId
    ? `https://github.com/login/oauth/authorize?client_id=${githubClientId}&scope=read:user,user:email&redirect_uri=${window.location.origin}/auth/github/user-callback`
    : null;

  const handleMessage = useCallback((event: MessageEvent) => {
    if (event.origin !== window.location.origin) return;
    if (event.data?.type !== "github-user-connection") return;

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
  }, []);

  useEffect(() => {
    window.addEventListener("message", handleMessage);

    return () => window.removeEventListener("message", handleMessage);
  }, [handleMessage]);

  const openPersonalOAuth = () => {
    if (!oauthUrl) return;

    const width = 600;
    const height = 600;
    const left = window.innerWidth / 2 - width / 2;
    const top = window.innerHeight / 2 - height / 2;

    window.open(oauthUrl, "", `width=${width},height=${height},top=${top},left=${left}`);
    setIsConnecting(true);
  };

  if (!githubClientId) return null;

  return (
    <div className="border-custom-border-200 bg-custom-background-100 space-y-4 rounded-lg border p-5">
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
                className="h-6 w-6 rounded-full"
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
