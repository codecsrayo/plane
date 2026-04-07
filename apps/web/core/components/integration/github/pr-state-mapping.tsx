/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { observer } from "mobx-react";
import { useParams } from "next/navigation";
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
  workspaceIntegrationId: string;
};

export const getPRStateMappingSwrKey = (workspaceIntegrationId: string) =>
  `PR_STATE_MAPPINGS_${workspaceIntegrationId}`;

export const GithubPRStateMapping = observer(function GithubPRStateMapping({ workspaceIntegrationId }: Props) {
  const { workspaceSlug } = useParams();
  const { getProjectById } = useProject();

  const SWR_KEY = workspaceSlug ? getPRStateMappingSwrKey(workspaceIntegrationId) : null;

  const { data: mappings, isLoading } = useSWR(SWR_KEY, () =>
    integrationService.getPRStateMappings(workspaceSlug as string, workspaceIntegrationId)
  );

  const handleDelete = async (mappingId: string) => {
    if (!workspaceSlug) return;
    try {
      await integrationService.deletePRStateMapping(workspaceSlug as string, workspaceIntegrationId, mappingId);
      mutate(SWR_KEY);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Mapping removed" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to remove mapping" });
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
    <div className="flex flex-col divide-y divide-custom-border-200 rounded-md border border-custom-border-200">
      {mappings.map((mapping: any) => {
        const project = getProjectById(mapping.project);
        return (
          <div key={mapping.id} className="flex items-center justify-between px-4 py-3">
            <div className="flex items-center gap-3 text-sm">
              <span className="font-medium text-custom-text-100">{project?.name ?? mapping.project}</span>
              <span className="text-custom-text-300">→</span>
              <span className="capitalize rounded bg-custom-background-80 px-2 py-0.5 text-xs font-medium">
                {mapping.github_pr_state?.replace(/_/g, " ")}
              </span>
            </div>
            <button
              onClick={() => handleDelete(mapping.id)}
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
