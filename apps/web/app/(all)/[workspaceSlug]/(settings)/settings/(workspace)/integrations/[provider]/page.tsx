/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import { useNavigate } from "react-router";
import useSWR, { mutate } from "swr";
import { ArrowLeft } from "lucide-react";
import { EUserPermissions, EUserPermissionsLevel } from "@plane/constants";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IWorkspaceIntegration } from "@plane/types";
import { Loader } from "@plane/ui";
// assets
import GithubLogo from "@/app/assets/services/github.png?url";
import GitlabLogo from "@/app/assets/services/gitlab.png?url";
import SlackLogo from "@/app/assets/services/slack.png?url";
// components
import { NotAuthorizedView } from "@/components/auth-screens/not-authorized-view";
import { PageHead } from "@/components/core/page-title";
// constants
import { WORKSPACE_INTEGRATIONS } from "@/constants/fetch-keys";
// hooks
import { useWorkspace } from "@/hooks/store/use-workspace";
import { useUserPermissions } from "@/hooks/store/user";
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
    return <p className="text-sm text-tertiary">No account details available.</p>;
  }

  // Pick the most meaningful fields to display
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

  // Fallback: show all keys we didn't already handle
  if (rows.length === 0) {
    for (const [k, v] of Object.entries(metadata)) {
      rows.push({ label: k, value: String(v) });
    }
  }

  return (
    <dl className="space-y-2">
      {rows.map(({ label, value }) => (
        <div key={label} className="flex items-center gap-3">
          <dt className="w-36 shrink-0 text-xs font-medium text-secondary">{label}</dt>
          <dd className="text-xs text-primary">{value}</dd>
        </div>
      ))}
    </dl>
  );
}

// ---------------------------------------------------------------------------
// Page component
// ---------------------------------------------------------------------------

function IntegrationDetailPage({ params }: Route.ComponentProps) {
  const { workspaceSlug, provider } = params;

  // states
  const [isDisconnecting, setIsDisconnecting] = useState(false);

  // navigation
  const navigate = useNavigate();

  // store hooks
  const { currentWorkspace } = useWorkspace();
  const { allowPermissions } = useUserPermissions();

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

  // Not authorised
  if (!isAdmin) return <NotAuthorizedView section="settings" className="h-auto" />;

  // Unknown provider
  if (!meta) {
    navigate(`/${workspaceSlug}/settings/integrations`, { replace: true });
    return null;
  }

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

  // Find the matching workspace integration for this provider
  const workspaceIntegration = workspaceIntegrations.find(
    (i) => i.integration_detail?.provider === provider
  );

  // Not installed → redirect
  if (!workspaceIntegration) {
    navigate(`/${workspaceSlug}/settings/integrations`, { replace: true });
    return null;
  }

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
            className="flex items-center gap-1.5 text-xs text-secondary hover:text-primary transition-colors w-fit"
            onClick={() => navigate(`/${workspaceSlug}/settings/integrations`)}
          >
            <ArrowLeft className="h-3.5 w-3.5" />
            Back to integrations
          </button>

          {/* Logo + name */}
          <div className="flex items-center gap-4">
            <div className="h-12 w-12 flex-shrink-0 rounded-xl border border-subtle p-2">
              <img src={meta.logo} className="h-full w-full object-contain" alt={`${meta.title} logo`} />
            </div>
            <div>
              <h1 className="text-xl font-semibold text-primary">{meta.title}</h1>
              <p className="text-sm text-secondary">Integration settings</p>
            </div>
          </div>
        </div>

        {/* ----------------------------------------------------------------
            Connected account card
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-subtle bg-surface-1 p-5 space-y-4">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold text-primary">Connected account</h2>
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
            Pull Request State Mapping — placeholder
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-subtle bg-surface-1 p-5 space-y-3">
          <h2 className="text-sm font-semibold text-primary">Pull Request State Mapping</h2>
          <p className="text-xs text-tertiary">
            Map pull-request states to Plane issue states.{" "}
            <span className="rounded bg-custom-background-80 px-1.5 py-0.5 text-[11px] font-medium text-secondary">
              Coming soon
            </span>
          </p>
        </div>

        {/* ----------------------------------------------------------------
            Project Issue Sync — placeholder
        ---------------------------------------------------------------- */}
        <div className="rounded-lg border border-subtle bg-surface-1 p-5 space-y-3">
          <h2 className="text-sm font-semibold text-primary">Project Issue Sync</h2>
          <p className="text-xs text-tertiary">
            Sync issues between Plane projects and your {meta.title} repository.{" "}
            <span className="rounded bg-custom-background-80 px-1.5 py-0.5 text-[11px] font-medium text-secondary">
              Coming soon
            </span>
          </p>
        </div>
      </section>
    </>
  );
}

export default observer(IntegrationDetailPage);
