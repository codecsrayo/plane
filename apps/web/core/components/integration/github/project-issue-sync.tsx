/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { useState } from "react";
import { observer } from "mobx-react";
import useSWR, { mutate } from "swr";
import { Trash2 } from "lucide-react";
import { Button } from "@plane/propel/button";
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

export const GithubProjectIssueSync = observer(function GithubProjectIssueSync({ workspaceSlug }: Props) {
  const { workspaceProjectIds, getProjectById } = useProject();

  const [selectedRepo, setSelectedRepo] = useState("");
  const [selectedProject, setSelectedProject] = useState("");
  const [isConnecting, setIsConnecting] = useState(false);

  const SYNCS_KEY = `GITHUB_REPO_SYNCS_${workspaceSlug}`;
  const REPOS_KEY = `GITHUB_REPOS_${workspaceSlug}`;

  const { data: syncs, isLoading: syncsLoading } = useSWR(SYNCS_KEY, () =>
    integrationService.getRepoSyncs(workspaceSlug)
  );

  const { data: repos, isLoading: reposLoading } = useSWR(REPOS_KEY, () =>
    integrationService.getGithubRepositories(workspaceSlug)
  );

  const handleConnect = async () => {
    if (!selectedRepo || !selectedProject) return;
    const repo = repos?.find((r: any) => r.id === selectedRepo || r.full_name === selectedRepo);
    setIsConnecting(true);
    try {
      await integrationService.createRepoSync(workspaceSlug, {
        repo_id: repo?.id ?? selectedRepo,
        repo_full_name: repo?.full_name ?? selectedRepo,
        project_id: selectedProject,
      });
      mutate(SYNCS_KEY);
      setSelectedRepo("");
      setSelectedProject("");
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Repository connected" });
    } catch (err: any) {
      const msg = err?.error ?? "Failed to connect repository";
      setToast({ type: TOAST_TYPE.ERROR, title: msg });
    } finally {
      setIsConnecting(false);
    }
  };

  const handleRemove = async (syncId: string) => {
    try {
      await integrationService.deleteRepoSync(workspaceSlug, syncId);
      mutate(SYNCS_KEY);
      setToast({ type: TOAST_TYPE.SUCCESS, title: "Sync removed" });
    } catch {
      setToast({ type: TOAST_TYPE.ERROR, title: "Failed to remove sync" });
    }
  };

  return (
    <div className="flex flex-col gap-4">
      {/* Active syncs */}
      {syncsLoading ? (
        <Loader className="space-y-2">
          <Loader.Item height="36px" />
          <Loader.Item height="36px" />
        </Loader>
      ) : syncs && syncs.length > 0 ? (
        <div className="flex flex-col divide-y divide-subtle rounded-md border border-subtle">
          {syncs.map((sync: any) => {
            const project = getProjectById(sync.project_id);
            return (
              <div key={sync.id} className="flex items-center justify-between px-4 py-3">
                <div className="flex items-center gap-3 text-sm">
                  <span className="font-medium text-secondary">{sync.repo_full_name}</span>
                  <span className="text-secondary">↔</span>
                  <span className="font-medium">{project?.identifier ?? sync.project_id}</span>
                </div>
                <button
                  onClick={() => handleRemove(sync.id)}
                  className="text-error-primary hover:text-error-primary/80 transition-colors"
                >
                  <Trash2 className="h-4 w-4" />
                </button>
              </div>
            );
          })}
        </div>
      ) : (
        <p className="text-sm text-secondary">No repositories connected yet.</p>
      )}

      {/* Add sync form */}
      <div className="flex flex-wrap items-end gap-3 rounded-md border border-subtle bg-surface-2 p-4">
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-secondary">GitHub Repository</label>
          {reposLoading ? (
            <div className="h-8 w-48 animate-pulse rounded bg-surface-3" />
          ) : (
            <select
              className="rounded border border-subtle bg-surface-1 px-3 py-1.5 text-sm"
              value={selectedRepo}
              onChange={(e) => setSelectedRepo(e.target.value)}
            >
              <option value="">Select repository</option>
              {(repos ?? []).map((r: any) => (
                <option key={r.id ?? r.full_name} value={r.full_name ?? r.id}>
                  {r.full_name ?? r.name}
                </option>
              ))}
            </select>
          )}
        </div>
        <div className="flex flex-col gap-1">
          <label className="text-xs font-medium text-secondary">Plane Project</label>
          <select
            className="rounded border border-subtle bg-surface-1 px-3 py-1.5 text-sm"
            value={selectedProject}
            onChange={(e) => setSelectedProject(e.target.value)}
          >
            <option value="">Select project</option>
            {(workspaceProjectIds ?? []).map((id) => {
              const p = getProjectById(id);
              return p ? <option key={id} value={id}>{p.name}</option> : null;
            })}
          </select>
        </div>
        <Button
          variant="primary"
          size="sm"
          onClick={handleConnect}
          loading={isConnecting}
          disabled={!selectedRepo || !selectedProject}
        >
          Connect
        </Button>
      </div>
    </div>
  );
});
