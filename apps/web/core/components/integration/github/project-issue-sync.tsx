/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import useSWR, { mutate } from "swr";
import { Trash2 } from "lucide-react";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { Loader } from "@plane/ui";
import { IntegrationConfirmActionModal } from "@/components/integration/confirm-action-modal";
// hooks
import { useProject } from "@/hooks/store/use-project";
// services
import { integrationService, type IGithubRepoSync } from "@/services/integrations";

type Props = {
  workspaceSlug: string;
};

export const getRepoSyncSwrKey = (workspaceSlug: string) => `GITHUB_REPO_SYNCS_${workspaceSlug}`;

export const GithubProjectIssueSync = observer(function GithubProjectIssueSync({ workspaceSlug }: Props) {
  const { getProjectById } = useProject();
  const [syncToDelete, setSyncToDelete] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const SYNCS_KEY = getRepoSyncSwrKey(workspaceSlug);

  const { data: syncs, isLoading } = useSWR<IGithubRepoSync[]>(SYNCS_KEY, () =>
    integrationService.getRepoSyncs(workspaceSlug)
  );

  const handleRemove = async () => {
    if (!syncToDelete) return;

    setIsDeleting(true);

    try {
      await integrationService.deleteRepoSync(workspaceSlug, syncToDelete);
      mutate(SYNCS_KEY);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Sync removed" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to remove sync" });
    } finally {
      setIsDeleting(false);
      setSyncToDelete(null);
    }
  };

  if (isLoading) {
    return (
      <Loader className="gap-y-2">
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
            <button
              type="button"
              onClick={() => setSyncToDelete(sync.id)}
              className="text-red-500 hover:text-red-400 transition-colors"
              aria-label={`Remove sync for ${projectLabel}`}
            >
              <Trash2 className="size-4" />
            </button>
          </div>
        );
      })}

      <IntegrationConfirmActionModal
        isOpen={Boolean(syncToDelete)}
        onClose={() => setSyncToDelete(null)}
        onConfirm={handleRemove}
        isSubmitting={isDeleting}
        title="Remove project issue sync"
        content="Are you sure you want to remove this repository sync? This action cannot be undone."
      />
    </div>
  );
});
