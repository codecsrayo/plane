/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useRef, useState } from "react";
import { useParams } from "next/navigation";
import { mutate } from "swr";
import { WORKSPACE_INTEGRATIONS } from "@/constants/fetch-keys";

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

  const providerUrls: { [key: string]: string } = {
    github: `https://github.com/apps/${github_app_name}/installations/new?state=${workspaceSlug?.toString()}`,
    gitlab: `${gitlab_host}/oauth/authorize?client_id=${gitlab_client_id}&redirect_uri=${window.location.origin}/auth/gitlab/callback&response_type=code&scope=api+read_user+read_repository+write_repository&state=${workspaceSlug?.toString()}`,
    slack: `https://slack.com/oauth/v2/authorize?scope=chat:write,im:history,im:write,links:read,links:write,users:read,users:read.email&user_scope=&client_id=${slack_client_id}&state=${workspaceSlug?.toString()}`,
    slackChannel: `https://slack.com/oauth/v2/authorize?scope=incoming-webhook&client_id=${slack_client_id}&state=${workspaceSlug?.toString()},${projectId?.toString()}${
      stateParams ? "," + stateParams : ""
    }`,
  };

  const popup = useRef<any>();

  // Listen for postMessage from the OAuth callback popup and refresh workspace integrations
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return;
      if (!(["github-integration", "gitlab-integration", "slack-integration"] as string[]).includes(event.data?.type)) return;

      // Reset the loading state immediately so the button updates without waiting for checkPopup
      setAuthLoader(false);

      if (event.data?.success && workspaceSlug) {
        // Force revalidation so the card switches to "Installed" / "Uninstall"
        mutate(WORKSPACE_INTEGRATIONS(workspaceSlug.toString()));
      }
    };

    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [workspaceSlug]);

  const checkPopup = () => {
    const check = setInterval(() => {
      if (!popup || popup.current.closed || popup.current.closed === undefined) {
        clearInterval(check);
        setAuthLoader(false);
      }
    }, 1000);
  };

  const openPopup = () => {
    if (!provider) return;

    const width = 600,
      height = 600;
    const left = window.innerWidth / 2 - width / 2;
    const top = window.innerHeight / 2 - height / 2;
    const url = providerUrls[provider];

    return window.open(url, "", `width=${width}, height=${height}, top=${top}, left=${left}`);
  };

  const startAuth = () => {
    popup.current = openPopup();
    checkPopup();
    setAuthLoader(true);
  };

  return {
    startAuth,
    isConnecting: authLoader,
  };
};

export default useIntegrationPopup;
