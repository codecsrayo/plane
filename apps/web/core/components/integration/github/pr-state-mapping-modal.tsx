/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useEffect, useState } from "react";
import { observer } from "mobx-react";
import { mutate } from "swr";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import { EModalWidth, ModalCore } from "@plane/ui";
// hooks
import { useProject } from "@/hooks/store/use-project";
import { useProjectState } from "@/hooks/store/use-project-state";
// services
import { integrationService } from "@/services/integrations";

const GITHUB_PR_STATES: { value: string; label: string }[] = [
  { value: "draft_open", label: "Draft Open" },
  { value: "open", label: "Open" },
  { value: "review_requested", label: "Review Requested" },
  { value: "ready_for_merge", label: "Ready for Merge" },
  { value: "merged", label: "Merged" },
  { value: "closed", label: "Closed" },
];

type StateMappingRow = Record<string, string>; // github_pr_state -> plane state id

type Props = {
  isOpen: boolean;
  onClose: () => void;
  workspaceSlug: string;
  workspaceIntegrationId: string;
  swrKey: string;
};

export const GithubPRStateMappingModal = observer(function GithubPRStateMappingModal({
  isOpen,
  onClose,
  workspaceSlug,
  workspaceIntegrationId,
  swrKey,
}: Props) {
  const { workspaceProjectIds, getProjectById } = useProject();
  const { getProjectStates, fetchProjectStates } = useProjectState();

  const [selectedProject, setSelectedProject] = useState("");
  const [stateMapping, setStateMapping] = useState<StateMappingRow>({});
  const [preventRegression, setPreventRegression] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const projectStates = selectedProject ? (getProjectStates(selectedProject) ?? []) : [];

  useEffect(() => {
    if (!selectedProject) return;

    fetchProjectStates(workspaceSlug, selectedProject).catch(() => {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to load project states" });
    });
  }, [fetchProjectStates, selectedProject, workspaceSlug]);

  const handleProjectChange = (projectId: string) => {
    setSelectedProject(projectId);
    setStateMapping({});
    // Reset preventRegression when the project changes — keeps state
    // consistent with handleClose which always resets it.
    setPreventRegression(false);
  };

  const handleStateChange = (prState: string, planeStateId: string) => {
    setStateMapping((prev) => ({ ...prev, [prState]: planeStateId }));
  };

  const handleClose = () => {
    setSelectedProject("");
    setStateMapping({});
    setPreventRegression(false);
    onClose();
  };

  const handleSave = async () => {
    if (!selectedProject) return;

    const entries = Object.entries(stateMapping).filter(([, stateId]) => !!stateId);
    if (entries.length === 0) {
      setToast({ type: TOAST_TYPE.ERROR, title: "Select at least one state mapping" });
      return;
    }

    setIsSaving(true);

    // Sequential creates (not Promise.all) for two reasons:
    //
    //   1. The DB has a UNIQUE constraint on
    //      (workspace_integration, project, github_pr_state). Parallel
    //      POSTs with the same triple race and one gets a 500/400 —
    //      harder to reason about than a serialised failure.
    //   2. If the Nth POST fails, rows 1..N-1 are already persisted.
    //      Promise.all reported "all failed" on any partial error,
    //      which was a UX lie. Here we record exactly what succeeded
    //      and surface that to the user.
    //
    // We intentionally do NOT compensate-delete successful creates on
    // failure: the delete is itself a network call that can fail, and
    // silently discarding the user's work is worse than persisting a
    // partial state they can see in the list.
    const failures: { githubPrState: string; error: unknown }[] = [];
    let successCount = 0;
    for (const [githubPrState, planeStateId] of entries) {
      try {
        // oxlint-disable-next-line no-await-in-loop -- serial on purpose: see comment above.
        await integrationService.createPRStateMapping(workspaceSlug, workspaceIntegrationId, {
          project: selectedProject,
          state: planeStateId,
          github_pr_state: githubPrState,
          prevent_regression: preventRegression,
        });
        successCount += 1;
      } catch (error) {
        failures.push({ githubPrState, error });
      }
    }

    // Always revalidate — even a partial save mutates the server state
    // and the list underneath the modal should reflect reality.
    mutate(swrKey);
    setIsSaving(false);

    if (failures.length === 0) {
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Mappings saved" });
      handleClose();
      return;
    }

    const labelFor = (prState: string) => GITHUB_PR_STATES.find((s) => s.value === prState)?.label ?? prState;
    const failedLabels = failures.map((f) => labelFor(f.githubPrState)).join(", ");

    if (successCount === 0) {
      setToast({
        type: TOAST_TYPE.ERROR,
        title: "Failed to save mappings",
        message: `None of the selected mappings could be saved (${failedLabels}).`,
      });
      return;
    }

    setToast({
      type: TOAST_TYPE.ERROR,
      title: `Saved ${successCount} of ${entries.length} mappings`,
      message: `These failed and were not saved: ${failedLabels}.`,
    });
  };

  return (
    <ModalCore isOpen={isOpen} handleClose={handleClose} width={EModalWidth.XL}>
      <div className="flex flex-col gap-5 p-5">
        {/* Header */}
        <h2 className="text-base font-semibold text-primary">Add Pull Request State Mapping for Plane project</h2>

        {/* Plane Project selector */}
        <div className="flex flex-col gap-1.5">
          <label htmlFor="github-pr-state-mapping-project" className="text-xs font-medium text-secondary">
            Plane Project
          </label>
          <select
            id="github-pr-state-mapping-project"
            className="border-custom-border-200 bg-custom-background-100 text-sm focus:border-custom-primary-100 w-full rounded-md border px-3 py-2 text-primary outline-none"
            value={selectedProject}
            onChange={(e) => handleProjectChange(e.target.value)}
          >
            <option value="">Choose Project...</option>
            {(workspaceProjectIds ?? []).map((id) => {
              const p = getProjectById(id);
              return p ? (
                <option key={id} value={id}>
                  {p.name}
                </option>
              ) : null;
            })}
          </select>
        </div>

        {/* PR Automation section */}
        <div className="border-custom-border-200 flex flex-col gap-3 rounded-lg border p-4">
          <div>
            <p className="text-sm font-semibold text-primary">Pull Request Automation</p>
            <p className="text-xs mt-0.5 text-secondary">
              Configure pull request state mapping from GitHub to your Plane project
            </p>
          </div>

          <div className="flex flex-col gap-2">
            {GITHUB_PR_STATES.map(({ value, label }) => (
              <div key={value} className="flex items-center justify-between gap-4">
                <span className="text-sm w-40 shrink-0 text-primary">{label}</span>
                <select
                  className="border-custom-border-200 bg-custom-background-100 text-sm focus:border-custom-primary-100 flex-1 rounded-md border px-3 py-1.5 text-primary outline-none disabled:cursor-not-allowed disabled:opacity-60"
                  value={stateMapping[value] ?? ""}
                  onChange={(e) => handleStateChange(value, e.target.value)}
                  disabled={!selectedProject}
                >
                  <option value="">Set State</option>
                  {projectStates.map((s) => (
                    <option key={s.id} value={s.id}>
                      {s.name}
                    </option>
                  ))}
                </select>
              </div>
            ))}
          </div>
        </div>

        {/* Prevent regression */}
        <label className="flex cursor-pointer items-center gap-2.5 select-none">
          <input
            type="checkbox"
            checked={preventRegression}
            onChange={(e) => setPreventRegression(e.target.checked)}
            className="border-custom-border-200 accent-custom-primary-100 h-3.5 w-3.5 rounded"
          />
          <span className="text-sm text-secondary">
            Prevent issues from moving to an earlier state due to PR updates
          </span>
        </label>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3">
          <Button variant="secondary" size="sm" onClick={handleClose}>
            Cancel
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={handleSave}
            loading={isSaving}
            disabled={!selectedProject || isSaving}
          >
            Save
          </Button>
        </div>
      </div>
    </ModalCore>
  );
});
