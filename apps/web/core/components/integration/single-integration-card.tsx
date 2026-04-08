/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { observer } from "mobx-react";
import { useParams } from "next/navigation";
import { useNavigate } from "react-router";
import useSWR from "swr";
import { CheckCircle } from "lucide-react";
import { EUserPermissions, EUserPermissionsLevel } from "@plane/constants";
import { Button } from "@plane/propel/button";
import { Tooltip } from "@plane/propel/tooltip";
import type { IAppIntegration } from "@plane/types";
// ui
import { Loader } from "@plane/ui";
// assets
import GithubLogo from "@/app/assets/services/github.png?url";
import GitlabLogo from "@/app/assets/services/gitlab.png?url";
import SlackLogo from "@/app/assets/services/slack.png?url";
// constants
import { WORKSPACE_INTEGRATIONS } from "@/constants/fetch-keys";
// hooks
import { useInstance } from "@/hooks/store/use-instance";
import { useUserPermissions } from "@/hooks/store/user";
import useIntegrationPopup from "@/hooks/use-integration-popup";
import { usePlatformOS } from "@/hooks/use-platform-os";
// services
import { IntegrationService } from "@/services/integrations";

type Props = {
  integration: IAppIntegration;
};

const integrationDetails: { [key: string]: any } = {
  github: {
    logo: GithubLogo,
    installed: "Activate GitHub on individual projects to sync with specific repositories.",
    notInstalled: "Connect with GitHub with your Plane workspace to sync project work items.",
  },
  gitlab: {
    logo: GitlabLogo,
    installed: "Activate GitLab on individual projects to sync with specific repositories.",
    notInstalled: "Connect with GitLab with your Plane workspace to sync project work items.",
  },
  slack: {
    logo: SlackLogo,
    installed: "Activate Slack on individual projects to sync with specific channels.",
    notInstalled: "Connect with Slack with your Plane workspace to sync project work items.",
  },
};

// services
const integrationService = new IntegrationService();

export const SingleIntegrationCard = observer(function SingleIntegrationCard({ integration }: Props) {
  // router
  const { workspaceSlug } = useParams();
  const navigate = useNavigate();
  // store hooks
  const { config } = useInstance();
  const { allowPermissions } = useUserPermissions();

  const isUserAdmin = allowPermissions([EUserPermissions.ADMIN], EUserPermissionsLevel.WORKSPACE);
  const { isMobile } = usePlatformOS();
  const { startAuth, isConnecting: isInstalling } = useIntegrationPopup({
    provider: integration.provider,
    github_app_name: config?.github_app_name || "",
    gitlab_client_id: config?.gitlab_client_id || "",
    gitlab_host: config?.gitlab_host || "https://gitlab.com",
    slack_client_id: config?.slack_client_id || "",
  });

  const { data: workspaceIntegrations } = useSWR(workspaceSlug ? WORKSPACE_INTEGRATIONS(workspaceSlug) : null, () =>
    workspaceSlug ? integrationService.getWorkspaceIntegrationsList(workspaceSlug) : null
  );

  const isInstalled = workspaceIntegrations?.find((i: any) => i.integration_detail?.id === integration.id);

  // Check if the integration is properly configured in God Mode (has required credentials)
  const isConfigured =
    integration.provider === "github"
      ? !!config?.github_app_name
      : integration.provider === "gitlab"
        ? !!config?.gitlab_client_id
        : integration.provider === "slack"
          ? !!config?.slack_client_id
          : true;

  // Respect God Mode enable/disable flags per integration provider
  const isEnabled =
    integration.provider === "github"
      ? (config?.is_github_enabled ?? true)
      : integration.provider === "gitlab"
        ? (config?.is_gitlab_enabled ?? true)
        : integration.provider === "slack"
          ? (config?.is_slack_enabled ?? true)
          : true;

  // Guard: if the provider is not locally known, skip rendering to avoid runtime errors
  const providerDetails = integrationDetails[integration.provider];
  if (!providerDetails) return null;

  return (
    <div className="flex flex-col gap-4 rounded-lg border border-subtle bg-surface-1 p-5">
      {/* Logo + title */}
      <div className="flex items-center gap-3">
        <div className="h-10 w-10 flex-shrink-0 rounded-lg border border-subtle p-1.5">
          <img src={providerDetails.logo} className="h-full w-full object-contain" alt={`${integration.title} Logo`} />
        </div>
        <h3 className="text-sm flex items-center gap-2 font-medium">
          {integration.title}
          {workspaceIntegrations
            ? isInstalled && <CheckCircle className="h-3.5 w-3.5 fill-transparent text-success-primary" />
            : null}
        </h3>
      </div>

      {/* Description */}
      <p className="text-xs flex-1 text-secondary">
        {workspaceIntegrations
          ? isInstalled
            ? providerDetails.installed
            : providerDetails.notInstalled
          : "Loading..."}
      </p>

      {/* Action */}
      {workspaceIntegrations ? (
        isInstalled ? (
          // When installed, show "Configure" to navigate to the detail page.
          <Tooltip
            isMobile={isMobile}
            disabled={isUserAdmin}
            tooltipContent={!isUserAdmin ? "You don't have permission to perform this" : null}
          >
            <Button
              className={`${!isUserAdmin ? "hover:cursor-not-allowed" : ""} w-fit`}
              variant="primary"
              onClick={() => {
                if (!isUserAdmin) return;
                navigate(`/${workspaceSlug}/settings/integrations/${integration.provider}`);
              }}
              disabled={!isUserAdmin}
            >
              Configure →
            </Button>
          </Tooltip>
        ) : !isEnabled ? (
          <span className="text-xs text-secondary">Disabled by admin</span>
        ) : (
          <Tooltip
            isMobile={isMobile}
            tooltipContent={
              !isUserAdmin
                ? "You don't have permission to perform this"
                : !isConfigured
                  ? "Configure this integration in God Mode before installing"
                  : null
            }
            disabled={isUserAdmin && isConfigured}
          >
            <Button
              className={`${!isUserAdmin || !isConfigured ? "hover:cursor-not-allowed" : ""} w-fit`}
              variant="primary"
              onClick={() => {
                if (!isUserAdmin || !isConfigured) return;
                startAuth();
              }}
              disabled={!isUserAdmin || !isConfigured}
              loading={isInstalling}
            >
              {isInstalling ? "Installing..." : "Install"}
            </Button>
          </Tooltip>
        )
      ) : (
        <Loader>
          <Loader.Item height="32px" width="64px" />
        </Loader>
      )}
    </div>
  );
});
