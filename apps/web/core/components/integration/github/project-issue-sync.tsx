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
import { IntegrationService } from "@/services/integrations";

const integrationService = new IntegrationService();

type Props = {
  workspaceSlug: string;
};

export const getRepoSyncSwrKey = (workspaceSlug: string) => `GITHUB_REPO_SYNCS_${workspaceSlug}`;

export const GithubProjectIssueSync = observer(function GithubProjectIssueSync({ workspaceSlug }: Props) {
  const { getProjectById } = useProject();

  const SYNCS_KEY = getRepoSyncSwrKey(workspaceSlug);

  const { data: syncs, isLoading } = useSWR(SYNCS_KEY, () =>
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
    <div className="flex flex-col divide-y divide-custom-border-200 rounded-md border border-custom-border-200">
      {syncs.map((sync: any) => {
        const project = getProjectById(sync.project_id);
        return (
          <div key={sync.id} className="flex items-center justify-between px-4 py-3">
            <div className="flex items-center gap-3 text-sm">
              <span className="font-medium text-custom-text-200">{sync.repo_full_name}</span>
              <span className="text-custom-text-300">↔</span>
              <span className="font-medium text-custom-text-100">{project?.identifier ?? sync.project_id}</span>
            </div>
            <button
              onClick={() => handleRemove(sync.id)}
              className="text-red-500 hover:text-red-400 transition-colors"
            >
              <Trash2 className="h-4 w-4" />
            </button>
          </div>
        );
      })}
    </div>
  );
});
