/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import { useParams } from "react-router";
import useSWR, { mutate } from "swr";
import { Trash2 } from "lucide-react";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { Loader } from "@plane/ui";
import { IntegrationConfirmActionModal } from "@/components/integration/confirm-action-modal";
// hooks
import { useProject } from "@/hooks/store/use-project";
import { useProjectState } from "@/hooks/store/use-project-state";
// services
import { integrationService, type IGithubPRStateMapping } from "@/services/integrations";

type Props = {
  workspaceIntegrationId: string;
};

export const getPRStateMappingSwrKey = (workspaceIntegrationId: string) =>
  `PR_STATE_MAPPINGS_${workspaceIntegrationId}`;

export const GithubPRStateMapping = observer(function GithubPRStateMapping({ workspaceIntegrationId }: Props) {
  const { workspaceSlug } = useParams();
  const { getProjectById } = useProject();
  const { getStateById } = useProjectState();
  const [mappingToDelete, setMappingToDelete] = useState<string | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

  const SWR_KEY = workspaceSlug ? getPRStateMappingSwrKey(workspaceIntegrationId) : null;

  const { data: mappings, isLoading } = useSWR<IGithubPRStateMapping[]>(
    SWR_KEY,
    workspaceSlug ? () => integrationService.getPRStateMappings(workspaceSlug, workspaceIntegrationId) : null
  );

  const handleDelete = async () => {
    if (!workspaceSlug || !mappingToDelete) return;

    setIsDeleting(true);

    try {
      await integrationService.deletePRStateMapping(workspaceSlug, workspaceIntegrationId, mappingToDelete);
      mutate(SWR_KEY);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Mapping removed" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to remove mapping" });
    } finally {
      setIsDeleting(false);
      setMappingToDelete(null);
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

  if (!mappings || mappings.length === 0) {
    return <p className="text-sm text-custom-text-300">No mappings yet. Click + to add one.</p>;
  }

  return (
    <div className="divide-custom-border-200 border-custom-border-200 flex flex-col divide-y rounded-md border">
      {mappings.map((mapping) => {
        const project = getProjectById(mapping.project);
        const state = getStateById(mapping.state);
        return (
          <div key={mapping.id} className="flex items-center justify-between px-4 py-3">
            <div className="text-sm flex items-center gap-3">
              <span className="text-custom-text-100 font-medium">{project?.name ?? mapping.project}</span>
              <span className="text-custom-text-300">→</span>
              <span className="bg-custom-background-80 text-xs rounded px-2 py-0.5 font-medium capitalize">
                {mapping.github_pr_state?.replace(/_/g, " ")}
              </span>
              <span className="text-custom-text-300">→</span>
              <span className="border-custom-border-200 text-xs text-custom-text-100 rounded border px-2 py-0.5 font-medium">
                {state?.name ?? mapping.state}
              </span>
            </div>
            <button
              type="button"
              onClick={() => setMappingToDelete(mapping.id)}
              className="text-red-500 hover:text-red-400 transition-colors"
              aria-label={`Delete mapping for ${project?.name ?? mapping.project}`}
            >
              <Trash2 className="h-4 w-4" />
            </button>
          </div>
        );
      })}

      <IntegrationConfirmActionModal
        isOpen={Boolean(mappingToDelete)}
        onClose={() => setMappingToDelete(null)}
        onConfirm={handleDelete}
        isSubmitting={isDeleting}
        title="Delete PR state mapping"
        content="Are you sure you want to remove this pull request state mapping? This action cannot be undone."
      />
    </div>
  );
});
