/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 *
 * Per-project card for a workspace-level integration (currently GitHub or Slack).
 *
 * Historical note: earlier versions of this component called
 *   projectService.syncGithubRepository  (POST .../projects/.../github-repository-sync/)
 *   projectService.getProjectGithubRepository  (GET same URL)
 * neither of which existed in Django nor in Rust. The real contract is
 * workspace-scoped at .../workspace-integrations/github/repo-syncs/, exposed
 * through `integrationService.getRepoSyncs` / `createRepoSync`. Those phantom
 * routes were removed from `project.service.ts`; this component now goes
 * through the real ones and filters client-side by project_id.
 */

import { useMemo } from "react";
import { useParams } from "next/navigation";
import useSWR, { mutate } from "swr";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IGithubRepo, IGithubRepoSync, IWorkspaceIntegration } from "@plane/types";
// assets
import GithubLogo from "@/app/assets/logos/github-square.png?url";
import SlackLogo from "@/app/assets/services/slack.png?url";
// components
import { SelectChannel } from "@/components/integration/slack/select-channel";
import { SelectRepository } from "@/components/integration/github/select-repository";
import { getRepoSyncSwrKey } from "@/components/integration/github/project-issue-sync";
// services
import { integrationService } from "@/services/integrations";

type IntegrationProviderMeta = {
  logo: string;
  description: string;
};

const integrationDetails: Record<string, IntegrationProviderMeta> = {
  github: {
    logo: GithubLogo,
    description: "Select GitHub repository to enable sync.",
  },
  slack: {
    logo: SlackLogo,
    description: "Get regular updates and control which notification you want to receive.",
  },
};

type Props = {
  integration: IWorkspaceIntegration;
};

/**
 * Derive a human-readable "owner/name" label from a repo sync record.
 *
 * `repo_full_name` is the only field present on BOTH the Django and Rust
 * responses for the list and create endpoints, so we prefer it. Falls back
 * to the split `repo_owner`/`repo_name` pair for defensive readability.
 */
const getSyncedRepoLabel = (sync: IGithubRepoSync): string => {
  if (sync.repo_full_name) return sync.repo_full_name;
  if (sync.repo_owner && sync.repo_name) return `${sync.repo_owner}/${sync.repo_name}`;
  return "";
};

export function IntegrationCard({ integration }: Props) {
  const { workspaceSlug, projectId } = useParams<{ workspaceSlug?: string; projectId?: string }>();

  // The repo-syncs endpoint is workspace-scoped, so we SWR-cache the whole list
  // under a workspace key and filter per project in memory. All `IntegrationCard`
  // instances on a workspace settings page share this cache entry.
  const syncsSwrKey = workspaceSlug ? getRepoSyncSwrKey(workspaceSlug) : null;

  const { data: workspaceRepoSyncs } = useSWR<IGithubRepoSync[]>(
    syncsSwrKey,
    workspaceSlug ? () => integrationService.getRepoSyncs(workspaceSlug) : null
  );

  const syncedGithubRepository = useMemo<IGithubRepoSync | undefined>(() => {
    if (!workspaceRepoSyncs || !projectId) return undefined;
    return workspaceRepoSyncs.find((s) => s.project_id === projectId);
  }, [workspaceRepoSyncs, projectId]);

  const handleChange = async (repo: IGithubRepo | undefined) => {
    if (!workspaceSlug || !projectId || !integration || !repo) return;

    try {
      await integrationService.createRepoSync(workspaceSlug, {
        repo_id: repo.id,
        repo_full_name: `${repo.owner}/${repo.name}`,
        project_id: projectId,
      });

      if (syncsSwrKey) mutate(syncsSwrKey);

      setToast({
        type: TOAST_TYPE.SUCCESS,
        title: "Success!",
        message: `${repo.owner}/${repo.name} repository synced with the project successfully.`,
      });
    } catch (err) {
      console.error(err);
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Error!",
        message: "Repository could not be synced with the project. Please try again.",
      });
    }
  };

  const providerMeta = integrationDetails[integration.integration_detail.provider];

  if (!integration || !providerMeta) return null;

  const syncedLabel = syncedGithubRepository ? getSyncedRepoLabel(syncedGithubRepository) : "";

  return (
    <div className="flex items-center justify-between gap-2 border-b border-subtle bg-surface-1 px-4 py-6">
      <div className="flex items-start gap-4">
        <div className="size-10 flex-shrink-0">
          <img
            src={providerMeta.logo}
            className="h-full w-full object-cover"
            alt={`${integration.integration_detail.title} Logo`}
          />
        </div>
        <div>
          <h3 className="flex items-center gap-4 text-13 font-medium">{integration.integration_detail.title}</h3>
          <p className="text-13 tracking-tight text-secondary">{providerMeta.description}</p>
        </div>
      </div>
      {integration.integration_detail.provider === "github" && (
        <SelectRepository
          integration={integration}
          value={syncedLabel || null}
          label={syncedLabel || "Select Repository"}
          onChange={handleChange}
        />
      )}
      {integration.integration_detail.provider === "slack" && <SelectChannel integration={integration} />}
    </div>
  );
}
