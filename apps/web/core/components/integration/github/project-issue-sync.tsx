/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { observer } from "mobx-react";
import useSWR, { mutate } from "swr";
import { Trash2 } from "lucide-react";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { Loader } from "@plane/ui";
// hooks
import { useProject } from "@/hooks/store/use-project";
// services
import { IntegrationService, type IGithubRepoSync } from "@/services/integrations";

const integrationService = new IntegrationService();

type Props = {
  workspaceSlug: string;
};

export const getRepoSyncSwrKey = (workspaceSlug: string) => `GITHUB_REPO_SYNCS_${workspaceSlug}`;

export const GithubProjectIssueSync = observer(function GithubProjectIssueSync({ workspaceSlug }: Props) {
  const { getProjectById } = useProject();

  const SYNCS_KEY = getRepoSyncSwrKey(workspaceSlug);

  const { data: syncs, isLoading } = useSWR<IGithubRepoSync[]>(SYNCS_KEY, () =>
    integrationService.getRepoSyncs(workspaceSlug)
  );

  const handleRemove = async (syncId: string) => {
    try {
      await integrationService.deleteRepoSync(workspaceSlug, syncId);
      mutate(SYNCS_KEY);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Sync removed" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to remove sync" });
    }
  };

  if (isLoading) {
    return (
      <Loader className="space-y-2">
        <Loader.Item height="36px" />
        <Loader.Item height="36px" />
      </Loader>
    );
  }

  if (!syncs || syncs.length === 0) {
    return <p className="text-sm text-custom-text-300">No repositories connected yet. Click + to add one.</p>;
  }

  return (
    <div className="divide-custom-border-200 border-custom-border-200 flex flex-col divide-y rounded-md border">
      {syncs.map((sync) => {
        const project = getProjectById(sync.project_id);
        const projectLabel = project?.name ?? sync.project_name ?? sync.project_identifier ?? sync.project_id;
        const isBidirectional = (sync.sync_direction ?? "bidirectional") === "bidirectional";
        return (
          <div key={sync.id} className="flex items-center justify-between px-4 py-3">
            <div className="text-sm flex items-center gap-3">
              <span className="text-custom-text-200 font-medium">{sync.repo_full_name}</span>
              <span className="text-custom-text-300">{isBidirectional ? "↔" : "→"}</span>
              <span className="text-custom-text-100 font-medium">{projectLabel}</span>
              {sync.sync_direction && (
                <span className="bg-custom-background-80 text-custom-text-300 rounded px-1.5 py-0.5 text-[11px] font-medium capitalize">
                  {sync.sync_direction}
                </span>
              )}
            </div>
            <button onClick={() => handleRemove(sync.id)} className="text-red-500 hover:text-red-400 transition-colors">
              <Trash2 className="h-4 w-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
});
