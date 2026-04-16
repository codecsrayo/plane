/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import useSWR, { mutate } from "swr";
import { Button } from "@plane/propel/button";
import { TOAST_TYPE, setToast } from "@plane/propel/toast";
import type { IGithubRepository } from "@plane/types";
import { EModalWidth, ModalCore } from "@plane/ui";
// hooks
import { useProject } from "@/hooks/store/use-project";
import { useProjectState } from "@/hooks/store/use-project-state";
// services
import { getGithubReposSwrKey } from "@/components/integration/utils";
import { integrationService } from "@/services/integrations";

type SyncDirection = "bidirectional" | "unidirectional";
type GithubRepositoryOption = IGithubRepository & { name?: string };
type IntegrationError = {
  error?: string;
};

type Props = {
  isOpen: boolean;
  onClose: () => void;
  workspaceSlug: string;
  swrKey: string;
};

export const GithubProjectIssueSyncModal = observer(function GithubProjectIssueSyncModal({
  isOpen,
  onClose,
  workspaceSlug,
  swrKey,
}: Props) {
  const { workspaceProjectIds, getProjectById } = useProject();
  const { getProjectStates } = useProjectState();

  const [selectedProject, setSelectedProject] = useState("");
  const [selectedRepo, setSelectedRepo] = useState("");
  const [issueOpenStateId, setIssueOpenStateId] = useState("");
  const [issueClosedStateId, setIssueClosedStateId] = useState("");
  const [syncDirection, setSyncDirection] = useState<SyncDirection>("bidirectional");
  const [isSyncing, setIsSyncing] = useState(false);

  const REPOS_KEY = getGithubReposSwrKey(workspaceSlug);
  const { data: repos, isLoading: reposLoading } = useSWR<GithubRepositoryOption[]>(isOpen ? REPOS_KEY : null, () =>
    integrationService.getGithubRepositories(workspaceSlug)
  );

  const projectStates = selectedProject ? (getProjectStates(selectedProject) ?? []) : [];

  const handleProjectChange = (projectId: string) => {
    setSelectedProject(projectId);
    setIssueOpenStateId("");
    setIssueClosedStateId("");
  };

  const handleClose = () => {
    setSelectedProject("");
    setSelectedRepo("");
    setIssueOpenStateId("");
    setIssueClosedStateId("");
    setSyncDirection("bidirectional");
    onClose();
  };

  const handleStartSync = async () => {
    if (!selectedRepo || !selectedProject) return;
    const repo = repos?.find((r) => r.full_name === selectedRepo || String(r.id) === selectedRepo);
    setIsSyncing(true);
    try {
      await integrationService.createRepoSync(workspaceSlug, {
        repo_id: repo?.id ?? selectedRepo,
        repo_full_name: repo?.full_name ?? selectedRepo,
        project_id: selectedProject,
        issue_open_state: issueOpenStateId || undefined,
        issue_closed_state: issueClosedStateId || undefined,
        sync_direction: syncDirection,
      });
      mutate(swrKey);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Repository synced" });
      handleClose();
    } catch (err: unknown) {
      const msg =
        (typeof err === "object" && err !== null && "error" in err ? (err as IntegrationError).error : undefined) ??
        "Failed to start sync";
      setToast({ type: TOAST_TYPE.ERROR, title: msg });
    } finally {
      setIsSyncing(false);
    }
  };

  return (
    <ModalCore isOpen={isOpen} handleClose={handleClose} width={EModalWidth.XL}>
      <div className="flex flex-col gap-5 p-5">
        {/* Header */}
        <h2 className="text-base font-semibold text-primary">Link GitHub Repository to a Plane Project</h2>

        {/* Plane Project selector */}
        <div className="flex flex-col gap-1.5">
          <label htmlFor="github-project-issue-sync-project" className="text-xs font-medium text-secondary">
            Plane Project
          </label>
          <select
            id="github-project-issue-sync-project"
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

        {/* GitHub Repository selector */}
        <div className="flex flex-col gap-1.5">
          <label htmlFor="github-project-issue-sync-repository" className="text-xs font-medium text-secondary">
            Github Repository
          </label>
          {reposLoading ? (
            <div className="bg-custom-background-80 h-9 w-full animate-pulse rounded-md" />
          ) : (
            <select
              id="github-project-issue-sync-repository"
              className="border-custom-border-200 bg-custom-background-100 text-sm focus:border-custom-primary-100 w-full rounded-md border px-3 py-2 text-primary outline-none"
              value={selectedRepo}
              onChange={(e) => setSelectedRepo(e.target.value)}
            >
              <option value="">Choose Repository...</option>
              {(repos ?? []).map((r) => (
                <option key={r.id ?? r.full_name} value={r.full_name ?? String(r.id)}>
                  {r.full_name ?? r.name}
                </option>
              ))}
            </select>
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
              <select
                className="border-custom-border-200 bg-custom-background-100 text-sm focus:border-custom-primary-100 flex-1 rounded-md border px-3 py-1.5 text-primary outline-none disabled:cursor-not-allowed disabled:opacity-60"
                value={issueOpenStateId}
                onChange={(e) => setIssueOpenStateId(e.target.value)}
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

            {/* Issue Closed */}
            <div className="flex items-center justify-between gap-4">
              <span className="text-sm w-32 shrink-0 text-primary">Issue Closed</span>
              <select
                className="border-custom-border-200 bg-custom-background-100 text-sm focus:border-custom-primary-100 flex-1 rounded-md border px-3 py-1.5 text-primary outline-none disabled:cursor-not-allowed disabled:opacity-60"
                value={issueClosedStateId}
                onChange={(e) => setIssueClosedStateId(e.target.value)}
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
              onChange={() => setSyncDirection("bidirectional")}
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
              onChange={() => setSyncDirection("unidirectional")}
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
