/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import { useParams } from "next/navigation";
import useSWR, { mutate } from "swr";
import { Trash2 } from "lucide-react";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { Loader } from "@plane/ui";
// hooks
import { useProject } from "@/hooks/store/use-project";
import { useProjectState } from "@/hooks/store/use-project-state";
// services
import { IntegrationService } from "@/services/integrations";

const integrationService = new IntegrationService();

const GITHUB_PR_STATES = [
  { value: "open", label: "Open" },
  { value: "merged", label: "Merged" },
  { value: "closed", label: "Closed" },
];

type Props = {
  workspaceIntegrationId: string;
};

export const GithubPRStateMapping = observer(function GithubPRStateMapping({ workspaceIntegrationId }: Props) {
  const { workspaceSlug } = useParams();
  const { workspaceProjectIds, getProjectById } = useProject();
  const { getProjectStates } = useProjectState();

  const [selectedProject, setSelectedProject] = useState("");
  const [selectedPrState, setSelectedPrState] = useState("open");
  const [selectedState, setSelectedState] = useState("");
  const [isAdding, setIsAdding] = useState(false);

  const SWR_KEY = workspaceSlug ? `PR_STATE_MAPPINGS_${workspaceIntegrationId}` : null;

  const { data: mappings, isLoading } = useSWR(SWR_KEY, () =>
    integrationService.getPRStateMappings(workspaceSlug as string, workspaceIntegrationId)
  );

  const projectStates = selectedProject ? getProjectStates(selectedProject) : [];

  const handleAdd = async () => {
    if (!selectedProject || !selectedState || !selectedPrState || !workspaceSlug) return;
    setIsAdding(true);
    try {
      await integrationService.createPRStateMapping(workspaceSlug as string, workspaceIntegrationId, {
        project: selectedProject,
        state: selectedState,
        github_pr_state: selectedPrState,
      });
      mutate(SWR_KEY);
      setSelectedProject("");
      setSelectedState("");
      setSelectedPrState("open");
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Mapping added" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to add mapping" });
    } finally {
      setIsAdding(false);
    }
  };

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

  return (
    <div className="flex flex-col gap-4">
      {/* Existing mappings */}
      {isLoading ? (
        <Loader className="space-y-2">
          <Loader.Item height="36px" />
          <Loader.Item height="36px" />
        </Loader>
      ) : mappings && mappings.length > 0 ? (
        <div className="flex flex-col divide-y divide-subtle rounded-md border border-subtle">
          {mappings.map((mapping: any) => {
            const project = getProjectById(mapping.project);
            return (
              <div key={mapping.id} className="flex items-center justify-between px-4 py-3">
                <div className="flex items-center gap-3 text-sm">
                  <span className="font-medium">{project?.name ?? mapping.project}</span>
                  <span className="text-secondary">→</span>
                  <span className="capitalize rounded bg-surface-2 px-2 py-0.5 text-xs font-medium">
                    {mapping.github_pr_state}
                  </span>
                </div>
                <button
                  onClick={() => handleDelete(mapping.id)}
                  className="text-error-primary hover:text-error-primary/80 transition-colors"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            );
          })}
        </div>
      ) : (
        <p className="text-sm text-secondary">No mappings yet. Add one below.</p>
      )}

      {/* Add mapping form */}
      <div className="flex flex-wrap items-end gap-3 rounded-md border border-subtle bg-surface-2 p-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-secondary">Project</label>
          <select
            className="rounded border border-subtle bg-surface-1 px-3 py-1.5 text-sm"
            value={selectedProject}
            onChange={(e) => { setSelectedProject(e.target.value); setSelectedState(""); }}
          >
            <option value="">Select project</option>
            {(workspaceProjectIds ?? []).map((id) => {
              const p = getProjectById(id);
              return p ? <option key={id} value={id}>{p.name}</option> : null;
            })}
          </select>
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-secondary">Plane State</label>
          <select
            className="rounded border border-subtle bg-surface-1 px-3 py-1.5 text-sm"
            value={selectedState}
            onChange={(e) => setSelectedState(e.target.value)}
            disabled={!selectedProject}
          >
            <option value="">Select state</option>
            {(projectStates ?? []).map((s) => (
              <option key={s.id} value={s.id}>{s.name}</option>
            ))}
          </select>
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-secondary">GitHub PR State</label>
          <select
            className="rounded border border-subtle bg-surface-1 px-3 py-1.5 text-sm"
            value={selectedPrState}
            onChange={(e) => setSelectedPrState(e.target.value)}
          >
            {GITHUB_PR_STATES.map((s) => (
              <option key={s.value} value={s.value}>{s.label}</option>
            ))}
          </select>
        </div>
        <Button
          variant="primary"
          size="sm"
          onClick={handleAdd}
          loading={isAdding}
          disabled={!selectedProject || !selectedState}
        >
          Add mapping
        </Button>
      </div>
    </div>
  );
});
