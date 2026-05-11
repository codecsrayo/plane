/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useState } from "react";
import { observer } from "mobx-react";
import { useNavigate } from "react-router";
import useSWR, { mutate } from "swr";
import { ArrowLeft, Plus } from "lucide-react";
import { EUserPermissions, EUserPermissionsLevel } from "@plane/constants";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IWorkspaceIntegration } from "@plane/types";
import { Loader } from "@plane/ui";
// assets
import GithubLogo from "@/app/assets/services/github.png?url";
import GitlabLogo from "@/app/assets/services/gitlab.png?url";
import SlackLogo from "@/app/assets/services/slack.png?url";
// integration components
import { ConnectedAccountDetails } from "@/components/integration/connected-account-details";
import { GithubPRStateMapping, getPRStateMappingSwrKey } from "@/components/integration/github/pr-state-mapping";
import { GithubPRStateMappingModal } from "@/components/integration/github/pr-state-mapping-modal";
import { GithubPersonalConnectCard } from "@/components/integration/github/personal-connect-card";
import { GithubProjectIssueSync, getRepoSyncSwrKey } from "@/components/integration/github/project-issue-sync";
import { GithubProjectIssueSyncModal } from "@/components/integration/github/project-issue-sync-modal";
import { IntegrationConfirmActionModal } from "@/components/integration/confirm-action-modal";
// components
import { NotAuthorizedView } from "@/components/auth-screens/not-authorized-view";
import { PageHead } from "@/components/core/page-title";
// constants
import { WORKSPACE_INTEGRATIONS } from "@/constants/fetch-keys";
// hooks
import { useWorkspace } from "@/hooks/store/use-workspace";
import { useUserPermissions } from "@/hooks/store/user";
import { useInstance } from "@/hooks/store/use-instance";
// services
import { integrationService } from "@/services/integrations";
// local imports
import type { Route } from "./+types/page";

// ---------------------------------------------------------------------------
// Static data
// ---------------------------------------------------------------------------

const integrationMeta: Record<string, { logo: string; title: string }> = {
  github: { logo: GithubLogo, title: "GitHub" },
  gitlab: { logo: GitlabLogo, title: "GitLab" },
  slack: { logo: SlackLogo, title: "Slack" },
};

function IntegrationDetailPage({ params }: Route.ComponentProps) {
  const { workspaceSlug, provider } = params;

  // states
  const [isDisconnecting, setIsDisconnecting] = useState(false);
  const [isDisconnectModalOpen, setIsDisconnectModalOpen] = useState(false);
  const [isPRMappingModalOpen, setIsPRMappingModalOpen] = useState(false);
  const [isIssueSyncModalOpen, setIsIssueSyncModalOpen] = useState(false);

  // navigation
  const navigate = useNavigate();

  // store hooks
  const { currentWorkspace } = useWorkspace();
  const { allowPermissions } = useUserPermissions();
  const { config } = useInstance();

  // derived values
  const isAdmin = allowPermissions([EUserPermissions.ADMIN], EUserPermissionsLevel.WORKSPACE);
  const meta = integrationMeta[provider];
  const pageTitle = currentWorkspace?.name
    ? `${currentWorkspace.name} - ${meta?.title ?? provider} Integration`
    : undefined;

  // data fetching
  const { data: workspaceIntegrations, isLoading } = useSWR(
    isAdmin && workspaceSlug ? WORKSPACE_INTEGRATIONS(workspaceSlug) : null,
    () => (isAdmin && workspaceSlug ? integrationService.getWorkspaceIntegrationsList(workspaceSlug) : null)
  );

  // Derived: find the matching workspace integration after data loads
  const workspaceIntegration = workspaceIntegrations?.find((i) => i.integration_detail?.provider === provider);

  // Redirect side-effects: unknown provider or integration not installed
  // Must be in useEffect — calling navigate() during render causes hydration errors
  useEffect(() => {
    if (!meta) {
      navigate(`/${workspaceSlug}/settings/integrations`, { replace: true });
    }
  }, [meta, workspaceSlug, navigate]);

  useEffect(() => {
    if (!isLoading && workspaceIntegrations && !workspaceIntegration) {
      navigate(`/${workspaceSlug}/settings/integrations`, { replace: true });
    }
  }, [isLoading, workspaceIntegrations, workspaceIntegration, workspaceSlug, navigate]);

  // Not authorised
  if (!isAdmin) return <NotAuthorizedView section="settings" className="h-auto" />;

  // Unknown provider — render nothing while useEffect redirects
  if (!meta) return null;
  if (!workspaceSlug) return null;

  // Loading
  if (isLoading || !workspaceIntegrations) {
    return (
      <div className="flex h-full w-full items-center justify-center p-8">
        <Loader>
          <Loader.Item height="40px" width="300px" />
        </Loader>
      </div>
    );
  }

  // Integration not installed — render nothing while useEffect redirects
  if (!workspaceIntegration) return null;

  // Disconnect handler
  const handleDisconnect = async () => {
    if (!workspaceIntegration) return;

    setIsDisconnecting(true);
    try {
      await integrationService.deleteWorkspaceIntegration(workspaceSlug, workspaceIntegration.id);
      mutate<IWorkspaceIntegration[]>(
        WORKSPACE_INTEGRATIONS(workspaceSlug),
        (prev) => prev?.filter((i) => i.id !== workspaceIntegration.id),
        false
      );
      setToast({
        type: TOAST_TYPE.SUCCESS,
        title: "Disconnected",
        message: `${meta.title} integration removed successfully.`,
      });
      setIsDisconnectModalOpen(false);
      navigate(`/${workspaceSlug}/settings/integrations`);
    } catch {
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Error",
        message: `Could not disconnect ${meta.title}. Please try again.`,
      });
      setIsDisconnecting(false);
    }
  };

  const prMappingSwrKey = getPRStateMappingSwrKey(workspaceIntegration.id);
  const repoSyncSwrKey = getRepoSyncSwrKey(workspaceSlug);

  return (
    <>
      <PageHead title={pageTitle} />
      <section className="w-full max-w-3xl space-y-8 p-6">
        {/* ----------------------------------------------------------------
            Header
        ---------------------------------------------------------------- */}
        <div className="flex flex-col gap-4">
          {/* Back link */}
          <button
            type="button"
            className="text-xs text-custom-text-300 hover:text-custom-text-100 flex w-fit items-center gap-1.5 transition-colors"
            onClick={() => navigate(`/${workspaceSlug}/settings/integrations`)}
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            Back to integrations
          </button>

          {/* Logo + name */}
          <div className="flex items-center gap-4">
            <div className="border-custom-border-200 size-12 flex-shrink-0 rounded-xl border p-2">
              <img src={meta.logo} className="h-full w-full object-contain" alt={`${meta.title} logo`} />
            </div>
            <div>
              <h1 className="text-xl text-custom-text-100 font-semibold">{meta.title}</h1>
              <p className="text-sm text-custom-text-300">Integration settings</p>
            </div>
          </div>
        </div>

        {/* ----------------------------------------------------------------
            Connected account card
        ---------------------------------------------------------------- */}
        <div className="border-custom-border-200 bg-custom-background-100 space-y-4 rounded-lg border p-5">
          <div className="flex items-center justify-between">
            <h2 className="text-sm text-custom-text-100 font-semibold">Connected account</h2>
            <Button
              variant="error-fill"
              size="sm"
              onClick={() => setIsDisconnectModalOpen(true)}
              loading={isDisconnecting}
              disabled={isDisconnecting}
            >
              {isDisconnecting ? "Disconnecting..." : "Disconnect"}
            </Button>
          </div>
          <ConnectedAccountDetails metadata={workspaceIntegration.metadata} />
        </div>

        {/* ----------------------------------------------------------------
            Personal GitHub account — GitHub only
        ---------------------------------------------------------------- */}
        {provider === "github" && config?.github_client_id && (
          <GithubPersonalConnectCard githubClientId={config.github_client_id} />
        )}

        {/* ----------------------------------------------------------------
            Pull Request State Mapping — GitHub only
        ---------------------------------------------------------------- */}
        <div className="border-custom-border-200 bg-custom-background-100 space-y-4 rounded-lg border p-5">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm text-custom-text-100 font-semibold">Pull Request State Mapping</h2>
              <p className="text-xs text-custom-text-300 mt-1">
                Map GitHub pull request states to Plane issue states per project.
              </p>
            </div>
            {provider === "github" && (
              <button
                type="button"
                onClick={() => setIsPRMappingModalOpen(true)}
                className="border-custom-border-200 text-custom-text-300 hover:text-custom-text-100 hover:border-custom-border-100 flex size-7 items-center justify-center rounded-md border transition-colors"
                title="Add PR state mapping"
              >
                <Plus className="size-4" />
              </button>
            )}
          </div>
          {provider === "github" && workspaceIntegration ? (
            <GithubPRStateMapping workspaceIntegrationId={workspaceIntegration.id} />
          ) : (
            <span className="bg-custom-background-80 text-custom-text-300 rounded px-1.5 py-0.5 text-[11px] font-medium">
              Coming soon
            </span>
          )}
        </div>

        {/* ----------------------------------------------------------------
            Project Issue Sync — GitHub only
        ---------------------------------------------------------------- */}
        <div className="border-custom-border-200 bg-custom-background-100 space-y-4 rounded-lg border p-5">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm text-custom-text-100 font-semibold">Project Issue Sync</h2>
              <p className="text-xs text-custom-text-300 mt-1">
                Connect Plane projects to {meta.title} repositories for issue synchronization.
              </p>
            </div>
            {provider === "github" && (
              <button
                type="button"
                onClick={() => setIsIssueSyncModalOpen(true)}
                className="border-custom-border-200 text-custom-text-300 hover:text-custom-text-100 hover:border-custom-border-100 flex size-7 items-center justify-center rounded-md border transition-colors"
                title="Add project issue sync"
              >
                <Plus className="size-4" />
              </button>
            )}
          </div>
          {provider === "github" ? (
            <GithubProjectIssueSync workspaceSlug={workspaceSlug} />
          ) : (
            <span className="bg-custom-background-80 text-custom-text-300 rounded px-1.5 py-0.5 text-[11px] font-medium">
              Coming soon
            </span>
          )}
        </div>
      </section>

      {/* Modals */}
      {provider === "github" && (
        <>
          <GithubPRStateMappingModal
            isOpen={isPRMappingModalOpen}
            onClose={() => setIsPRMappingModalOpen(false)}
            workspaceSlug={workspaceSlug}
            workspaceIntegrationId={workspaceIntegration.id}
            swrKey={prMappingSwrKey}
          />
          <GithubProjectIssueSyncModal
            isOpen={isIssueSyncModalOpen}
            onClose={() => setIsIssueSyncModalOpen(false)}
            workspaceSlug={workspaceSlug}
            onSuccess={() => mutate(repoSyncSwrKey)}
          />
        </>
      )}

      <IntegrationConfirmActionModal
        isOpen={isDisconnectModalOpen}
        onClose={() => setIsDisconnectModalOpen(false)}
        onConfirm={handleDisconnect}
        isSubmitting={isDisconnecting}
        title={`Disconnect ${meta.title}`}
        content={
          <>
            Are you sure you want to disconnect <span className="font-medium text-primary">{meta.title}</span>? Any
            project-level mappings and repository sync setup may stop working until the integration is installed again.
          </>
        }
      />
    </>
  );
}

export default observer(IntegrationDetailPage);
