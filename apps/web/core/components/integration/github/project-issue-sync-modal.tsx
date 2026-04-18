/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useReducer } from "react";
import { observer } from "mobx-react";
import useSWR from "swr";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IGithubRepository } from "@plane/types";
import { CustomSearchSelect, EModalWidth, ModalCore } from "@plane/ui";
// hooks
import { useProject } from "@/hooks/store/use-project";
import { useProjectState } from "@/hooks/store/use-project-state";
// services
import { getGithubReposSwrKey } from "@/components/integration/utils";
import { integrationService } from "@/services/integrations";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

type SyncDirection = "bidirectional" | "unidirectional";

type ModalState = {
  selectedProject: string;
  selectedRepo: string;
  issueOpenStateId: string;
  issueClosedStateId: string;
  syncDirection: SyncDirection;
  isSyncing: boolean;
};

type ModalAction =
  | { type: "SET_PROJECT"; projectId: string }
  | { type: "SET_REPO"; repoId: string }
  | { type: "SET_OPEN_STATE"; stateId: string }
  | { type: "SET_CLOSED_STATE"; stateId: string }
  | { type: "SET_SYNC_DIRECTION"; direction: SyncDirection }
  | { type: "SET_SYNCING"; syncing: boolean }
  | { type: "RESET" };

type IntegrationError = { error?: string };

type Props = {
  isOpen: boolean;
  onClose: () => void;
  workspaceSlug: string;
  /** Called after a successful sync creation so the parent can revalidate its own SWR key. */
  onSuccess: () => void;
};

// ---------------------------------------------------------------------------
// Reducer
// ---------------------------------------------------------------------------

const INITIAL_STATE: ModalState = {
  selectedProject: "",
  selectedRepo: "",
  issueOpenStateId: "",
  issueClosedStateId: "",
  syncDirection: "bidirectional",
  isSyncing: false,
};

function modalReducer(state: ModalState, action: ModalAction): ModalState {
  switch (action.type) {
    case "SET_PROJECT":
      // Clearing state selections when the project changes avoids stale state IDs
      // that belong to the previous project reaching the backend.
      return { ...state, selectedProject: action.projectId, issueOpenStateId: "", issueClosedStateId: "" };
    case "SET_REPO":
      return { ...state, selectedRepo: action.repoId };
    case "SET_OPEN_STATE":
      return { ...state, issueOpenStateId: action.stateId };
    case "SET_CLOSED_STATE":
      return { ...state, issueClosedStateId: action.stateId };
    case "SET_SYNC_DIRECTION":
      return { ...state, syncDirection: action.direction };
    case "SET_SYNCING":
      return { ...state, isSyncing: action.syncing };
    case "RESET":
      return INITIAL_STATE;
  }
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

export const GithubProjectIssueSyncModal = observer(function GithubProjectIssueSyncModal({
  isOpen,
  onClose,
  workspaceSlug,
  onSuccess,
}: Props) {
  const { workspaceProjectIds, getProjectById } = useProject();
  const { getProjectStates } = useProjectState();

  const [modalState, dispatch] = useReducer(modalReducer, INITIAL_STATE);
  const { selectedProject, selectedRepo, issueOpenStateId, issueClosedStateId, syncDirection, isSyncing } = modalState;

  // Only fetch repos while the modal is open — avoids background requests when closed.
  const REPOS_KEY = getGithubReposSwrKey(workspaceSlug);
  const { data: repos, isLoading: reposLoading } = useSWR<IGithubRepository[]>(
    isOpen ? REPOS_KEY : null,
    () => integrationService.getGithubRepositories(workspaceSlug)
  );

  const projectStates = selectedProject ? (getProjectStates(selectedProject) ?? []) : [];

  // -------------------------------------------------------------------------
  // Options for CustomSearchSelect
  // -------------------------------------------------------------------------

  const projectOptions = (workspaceProjectIds ?? []).flatMap((id) => {
    const p = getProjectById(id);
    if (!p) return [];
    return [{ value: id, query: p.name, content: <span className="truncate">{p.name}</span> }];
  });

  // Use repo.id (string) as the canonical identifier — stable, unlike
  // full_name which can change on GitHub renames.
  const repoOptions = (repos ?? []).map((r) => ({
    value: r.id,
    query: r.full_name,
    content: <span className="truncate">{r.full_name}</span>,
  }));

  const stateOptions = projectStates.map((s) => ({
    value: s.id,
    query: s.name,
    content: <span className="truncate">{s.name}</span>,
  }));

  // -------------------------------------------------------------------------
  // Derived labels for CustomSearchSelect triggers
  // -------------------------------------------------------------------------

  const selectedProjectLabel = selectedProject
    ? (getProjectById(selectedProject)?.name ?? "Choose Project…")
    : "Choose Project…";

  const selectedRepoLabel = reposLoading
    ? "Loading repositories…"
    : selectedRepo
      ? (repos?.find((r) => r.id === selectedRepo)?.full_name ?? "Choose Repository…")
      : "Choose Repository…";

  const openStateLabel = issueOpenStateId
    ? (projectStates.find((s) => s.id === issueOpenStateId)?.name ?? "Set State")
    : "Set State";

  const closedStateLabel = issueClosedStateId
    ? (projectStates.find((s) => s.id === issueClosedStateId)?.name ?? "Set State")
    : "Set State";

  // -------------------------------------------------------------------------
  // Handlers
  // -------------------------------------------------------------------------

  const handleClose = () => {
    dispatch({ type: "RESET" });
    onClose();
  };

  const handleStartSync = async () => {
    if (!selectedRepo || !selectedProject) return;

    // Hard guard: if the repo is not in the loaded list, the ID is stale or
    // invalid. Fail loudly instead of sending an opaque string to the backend.
    const repo = repos?.find((r) => r.id === selectedRepo);
    if (!repo) {
      setToast({ type: TOAST_TYPE.ERROR, title: "Selected repository not found. Please reselect." });
      dispatch({ type: "SET_REPO", repoId: "" });
      return;
    }

    dispatch({ type: "SET_SYNCING", syncing: true });
    try {
      await integrationService.createRepoSync(workspaceSlug, {
        repo_id: repo.id,
        repo_full_name: repo.full_name,
        project_id: selectedProject,
        issue_open_state: issueOpenStateId || undefined,
        issue_closed_state: issueClosedStateId || undefined,
        sync_direction: syncDirection,
      });
      // Delegate cache invalidation to the parent — the modal has no knowledge
      // of which SWR key the parent uses for its sync list.
      onSuccess();
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Repository synced" });
      handleClose();
    } catch (err: unknown) {
      const msg =
        (typeof err === "object" && err !== null && "error" in err ? (err as IntegrationError).error : undefined) ??
        "Failed to start sync";
      setToast({ type: TOAST_TYPE.ERROR, title: msg });
    } finally {
      dispatch({ type: "SET_SYNCING", syncing: false });
    }
  };

  // -------------------------------------------------------------------------
  // Render
  // -------------------------------------------------------------------------

  return (
    <ModalCore isOpen={isOpen} handleClose={handleClose} width={EModalWidth.XL}>
      <div className="flex flex-col gap-5 p-5">
        {/* Header */}
        <h2 className="text-base font-semibold text-primary">Link GitHub Repository to a Plane Project</h2>

        {/* Plane Project selector */}
        <div className="flex flex-col gap-1.5">
          <span className="text-xs font-medium text-secondary">Plane Project</span>
          <CustomSearchSelect
            value={selectedProject}
            options={projectOptions}
            onChange={(val: string) => dispatch({ type: "SET_PROJECT", projectId: val })}
            label={selectedProjectLabel}
            optionsClassName="w-full"
          />
        </div>

        {/* GitHub Repository selector */}
        <div className="flex flex-col gap-1.5">
          <span className="text-xs font-medium text-secondary">Github Repository</span>
          {reposLoading ? (
            <div className="bg-custom-background-80 h-9 w-full animate-pulse rounded-md" />
          ) : (
            <CustomSearchSelect
              value={selectedRepo}
              options={repoOptions}
              onChange={(val: string) => dispatch({ type: "SET_REPO", repoId: val })}
              label={selectedRepoLabel}
              optionsClassName="w-full"
            />
          )}
        </div>

        {/* Issue Sync State section */}
        <div className="border-custom-border-200 flex flex-col gap-3 rounded-lg border p-4">
          <div>
            <p className="text-sm font-semibold text-primary">Project Issue Sync</p>
            <p className="text-xs mt-0.5 text-secondary">Configure Issue Sync State</p>
          </div>

          <div className="flex flex-col gap-2">
            {/* Issue Open */}
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm w-32 shrink-0 text-primary">Issue Open</span>
              <div className="flex-1">
                <CustomSearchSelect
                  value={issueOpenStateId}
                  options={stateOptions}
                  onChange={(val: string) => dispatch({ type: "SET_OPEN_STATE", stateId: val })}
                  label={openStateLabel}
                  disabled={!selectedProject}
                  optionsClassName="w-full"
                />
              </div>
            </div>

            {/* Issue Closed */}
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm w-32 shrink-0 text-primary">Issue Closed</span>
              <div className="flex-1">
                <CustomSearchSelect
                  value={issueClosedStateId}
                  options={stateOptions}
                  onChange={(val: string) => dispatch({ type: "SET_CLOSED_STATE", stateId: val })}
                  label={closedStateLabel}
                  disabled={!selectedProject}
                  optionsClassName="w-full"
                />
              </div>
            </div>
          </div>
        </div>

        {/* Sync direction */}
        <div className="flex flex-col gap-2">
          <p className="text-sm text-secondary">Select issue sync direction</p>
          <label className="flex cursor-pointer items-center gap-2.5">
            <input
              type="radio"
              name="sync_direction"
              value="bidirectional"
              checked={syncDirection === "bidirectional"}
              onChange={() => dispatch({ type: "SET_SYNC_DIRECTION", direction: "bidirectional" })}
              className="accent-custom-primary-100"
            />
            <span className="text-sm text-primary">
              Bidirectional - Sync issues and comments both ways between GitHub and Plane
            </span>
          </label>
          <label className="flex cursor-pointer items-center gap-2.5">
            <input
              type="radio"
              name="sync_direction"
              value="unidirectional"
              checked={syncDirection === "unidirectional"}
              onChange={() => dispatch({ type: "SET_SYNC_DIRECTION", direction: "unidirectional" })}
              className="accent-custom-primary-100"
            />
            <span className="text-sm text-primary">
              Unidirectional - Sync issues and comments from GitHub to Plane only
            </span>
          </label>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3">
          <Button variant="secondary" size="sm" onClick={handleClose}>
            Cancel
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={handleStartSync}
            loading={isSyncing}
            disabled={!selectedProject || !selectedRepo || isSyncing}
          >
            Start Sync
          </Button>
        </div>
      </div>
    </ModalCore>
  );
});
