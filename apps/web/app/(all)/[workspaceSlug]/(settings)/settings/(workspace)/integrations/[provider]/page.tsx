/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState, useEffect, useRef, useCallback } from "react";
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
import { GithubPRStateMapping, getPRStateMappingSwrKey } from "@/components/integration/github/pr-state-mapping";
import { GithubPRStateMappingModal } from "@/components/integration/github/pr-state-mapping-modal";
import { GithubProjectIssueSync, getRepoSyncSwrKey } from "@/components/integration/github/project-issue-sync";
import { GithubProjectIssueSyncModal } from "@/components/integration/github/project-issue-sync-modal";
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
import { IntegrationService } from "@/services/integrations";
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

const integrationService = new IntegrationService();

// ---------------------------------------------------------------------------
// Helper: render the metadata card for the connected account
// ---------------------------------------------------------------------------

function ConnectedAccountDetails({ metadata }: { metadata: Record<string, unknown> | null }) {
  if (!metadata || Object.keys(metadata).length === 0) {
    return <p className="text-sm text-custom-text-300">No account details available.</p>;
  }

  const rows: { label: string; value: string }[] = [];

  if (typeof metadata.installation_id !== "undefined")
    rows.push({ label: "Installation ID", value: String(metadata.installation_id) });
  if (typeof metadata.team_name !== "undefined")
    rows.push({ label: "Team name", value: String(metadata.team_name) });
  if (typeof metadata.team_id !== "undefined")
    rows.push({ label: "Team ID", value: String(metadata.team_id) });
  if (typeof metadata.account !== "undefined")
    rows.push({ label: "Account", value: String(metadata.account) });
  if (typeof metadata.login !== "undefined")
    rows.push({ label: "Login", value: String(metadata.login) });

  if (rows.length === 0) {
    for (const [k, v] of Object.entries(metadata)) {
      rows.push({ label: k, value: String(v) });
    }
  }

  return (
    <dl className="space-y-2">
      {rows.map(({ label, value }) => (
        <div key={label} className="flex items-center gap-3">
          <dt className="w-36 shrink-0 text-xs font-medium text-custom-text-200">{label}</dt>
          <dd className="text-xs text-custom-text-100">{value}</dd>
        </div>
      ))}
    </dl>
  );
}

// ---------------------------------------------------------------------------
// Helper: GitHub personal account connection section
// ---------------------------------------------------------------------------

interface GithubPersonalConnectProps {
  workspaceSlug: string;
  githubClientId: string;
}

function GithubPersonalConnect({ workspaceSlug, githubClientId }: GithubPersonalConnectProps) {
  const [personalConnection, setPersonalConnection] = useState<{
    github_username: string;
    github_avatar_url: string;
  } | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);
  const popup = useRef<Window | null>(null);

  // Build personal OAuth URL (read:user scope, separate from GitHub App installation)
  const oauthUrl = githubClientId
    ? `https://github.com/login/oauth/authorize?client_id=${githubClientId}&scope=read:user,user:email&redirect_uri=${window.location.origin}/auth/github/user-callback`
    : null;

  // Listen for postMessage from the user-callback popup
  const handleMessage = useCallback(
    (event: MessageEvent) => {
      if (event.origin !== window.location.origin) return;
      if (event.data?.type !== "github-user-connection") return;
      setIsConnecting(false);
      if (event.data?.success) {
        // Re-fetch connection status (simple approach: reload personal connection)
        setPersonalConnection(event.data?.payload ?? null);
      }
    },
    []
  );

  useEffect(() => {
    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [handleMessage]);

  const openPersonalOAuth = () => {
    if (!oauthUrl) return;
    const width = 600, height = 600;
    const left = window.innerWidth / 2 - width / 2;
    const top = window.innerHeight / 2 - height / 2;
    popup.current = window.open(oauthUrl, "", `width=${width},height=${height},top=${top},left=${left}`);
    setIsConnecting(true);
  };

  if (!githubClientId) return null;

  return (
    <div className="rounded-lg border border-custom-border-200 bg-custom-background-100 p-5 space-y-4">
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-sm font-semibold text-custom-text-100">Your GitHub Account</h2>
          <p className="mt-1 text-xs text-custom-text-300">
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
            <span className="text-xs font-medium text-custom-text-100">
              @{personalConnection.github_username}
            </span>
          </div>
        ) : (
          <Button
            variant="neutral-primary"
            size="sm"
            onClick={openPersonalOAuth}
            loading={isConnecting}
            disabled={isConnecting}
          >
            {isConnecting ? "Connecting…" : "Connect account"}
          </Button>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------

function IntegrationDetailPage({ params }: Route.ComponentProps) {
  const { workspaceSlug, provider } = params;

  // states
  const [isDisconnecting, setIsDisconnecting] = useState(false);
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
  const workspaceIntegration = workspaceIntegrations?.find(
    (i) => i.integration_detail?.provider === provider
  );

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
    if (!workspaceSlug || !workspaceIntegration) return;

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
  const repoSyncSwrKey = getRepoSyncSwrKey(workspaceSlug as string);

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
            className="flex items-center gap-1.5 text-xs text-custom-text-300 hover:text-custom-text-100 transition-colors w-fit"
            onClick={() => navigate(`/${workspaceSlug}/settings/integrations`)}
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            Back to integrations
          </button>

          {/* Logo + name */}
          <div className="flex items-center gap-4">
            <div className="h-12 w-12 flex-shrink-0 rounded-xl border border-custom-border-200 p-2">
              <img src={meta.logo} className="h-full w-full object-contain" alt={`${meta.title} logo`} />
            </div>
            <div>
              <h1 className="text-xl font-semibold text-custom-text-100">{meta.title}</h1>
              <p className="text-sm text-custom-text-300">Integration settings</p>
            </div>
          </div>
        </div>

        {/* ----------------------------------------------------------------
            Connected account card
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-custom-border-200 bg-custom-background-100 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-custom-text-100">Connected account</h2>
            <Button
              variant="error-fill"
              size="sm"
              onClick={handleDisconnect}
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
          <GithubPersonalConnect
            workspaceSlug={workspaceSlug as string}
            githubClientId={config.github_client_id}
          />
        )}

        {/* ----------------------------------------------------------------
            Pull Request State Mapping — GitHub only
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-custom-border-200 bg-custom-background-100 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-custom-text-100">Pull Request State Mapping</h2>
              <p className="mt-1 text-xs text-custom-text-300">
                Map GitHub pull request states to Plane issue states per project.
              </p>
            </div>
            {provider === "github" && (
              <button
                type="button"
                onClick={() => setIsPRMappingModalOpen(true)}
                className="flex h-7 w-7 items-center justify-center rounded-md border border-custom-border-200 text-custom-text-300 hover:text-custom-text-100 hover:border-custom-border-100 transition-colors"
                title="Add PR state mapping"
              >
                <Plus className="h-4 w-4" />
              </button>
            )}
          </div>
          {provider === "github" && workspaceIntegration ? (
            <GithubPRStateMapping workspaceIntegrationId={workspaceIntegration.id} />
          ) : (
            <span className="rounded bg-custom-background-80 px-1.5 py-0.5 text-[11px] font-medium text-custom-text-300">
              Coming soon
            </span>
          )}
        </div>

        {/* ----------------------------------------------------------------
            Project Issue Sync — GitHub only
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-custom-border-200 bg-custom-background-100 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-sm font-semibold text-custom-text-100">Project Issue Sync</h2>
              <p className="mt-1 text-xs text-custom-text-300">
                Connect Plane projects to {meta.title} repositories for issue synchronization.
              </p>
            </div>
            {provider === "github" && (
              <button
                type="button"
                onClick={() => setIsIssueSyncModalOpen(true)}
                className="flex h-7 w-7 items-center justify-center rounded-md border border-custom-border-200 text-custom-text-300 hover:text-custom-text-100 hover:border-custom-border-100 transition-colors"
                title="Add project issue sync"
              >
                <Plus className="h-4 w-4" />
              </button>
            )}
          </div>
          {provider === "github" ? (
            <GithubProjectIssueSync workspaceSlug={workspaceSlug as string} />
          ) : (
            <span className="rounded bg-custom-background-80 px-1.5 py-0.5 text-[11px] font-medium text-custom-text-300">
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
            workspaceSlug={workspaceSlug as string}
            workspaceIntegrationId={workspaceIntegration.id}
            swrKey={prMappingSwrKey}
          />
          <GithubProjectIssueSyncModal
            isOpen={isIssueSyncModalOpen}
            onClose={() => setIsIssueSyncModalOpen(false)}
            workspaceSlug={workspaceSlug as string}
            swrKey={repoSyncSwrKey}
          />
        </>
      )}
    </>
  );
}

export default observer(IntegrationDetailPage);


