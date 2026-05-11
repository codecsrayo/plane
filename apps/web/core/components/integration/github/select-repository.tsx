/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import { useParams } from "react-router";
import useSWRInfinite from "swr/infinite";
import type { IGithubRepo, IGithubRepositoriesResponse, IWorkspaceIntegration } from "@plane/types";
import { CustomSearchSelect, Spinner } from "@plane/ui";
import { truncateText } from "@plane/utils";
import { GithubIntegrationService } from "@/services/integrations/github.service";

// Tuple key for useSWRInfinite — captures every input that should invalidate
// the cache when it changes (workspace, integration, page, cache tag).
type TGithubRepoKey = [workspaceSlug: string, workspaceIntegrationId: string, page: number, cacheTag: string];

type Props = {
  integration: IWorkspaceIntegration;
  /** `full_name` of the currently synced repo, or null when none is linked. */
  value: string | null | undefined;
  label: string | React.ReactNode;
  onChange: (repo: IGithubRepo | undefined) => void;
  characterLimit?: number;
};

const githubService = new GithubIntegrationService();

export function SelectRepository(props: Props) {
  const { integration, value, label, onChange, characterLimit = 25 } = props;
  const { workspaceSlug } = useParams();

  // Return a structured tuple key — relying on `APIService.baseURL` to
  // resolve the absolute URL keeps the service as the single source of
  // truth for endpoint layout and avoids leaking `process.env` into the
  // component (which Vite only substitutes at build time).
  const getKey = (pageIndex: number): TGithubRepoKey | null => {
    if (!workspaceSlug || !integration) return null;
    return [workspaceSlug.toString(), integration.id, pageIndex + 1, "github-repositories"];
  };

  const fetchGithubRepos = async (key: TGithubRepoKey): Promise<IGithubRepositoriesResponse> => {
    const [slug, integrationId, page] = key;
    return githubService.listAllRepositories(slug, integrationId, page);
  };

  const {
    data: paginatedData,
    setSize,
    isValidating,
    error,
  } = useSWRInfinite<IGithubRepositoriesResponse>(getKey, fetchGithubRepos);

  const isLoading = !paginatedData && !error;

  // Flatten pages and defensively filter out entries without a stable id —
  // the backend used to emit rows with only `full_name`, but the UI key
  // contract requires `id`.
  const userRepositories = (paginatedData ?? []).flatMap((data) => data.repositories ?? []).filter((data) => data?.id);

  const totalCount = paginatedData && paginatedData.length > 0 ? paginatedData[0].total_count : 0;
  const isInstallationToken = paginatedData?.[0]?.is_installation_token ?? false;
  const manageInstallationUrl = paginatedData?.[0]?.manage_installation_url ?? null;

  const options = userRepositories.map((repo) => ({
    value: repo.id,
    query: repo.full_name,
    content: <p className="truncate">{truncateText(repo.full_name, characterLimit)}</p>,
  }));

  if (isLoading) {
    return (
      <div className="text-sm text-custom-text-300 flex items-center gap-2">
        <Spinner height="16px" width="16px" />
        <span>Loading repositories…</span>
      </div>
    );
  }

  if (error) {
    return <p className="text-sm text-red-500">Failed to load repositories. Check your GitHub integration settings.</p>;
  }

  return (
    <div className="flex flex-col gap-1.5">
      <CustomSearchSelect
        value={value}
        options={options}
        onChange={(val: string) => {
          const repo = userRepositories.find((r) => r.id === val);
          onChange(repo);
        }}
        label={label}
        footerOption={
          <>
            {options.length < totalCount && (
              <button
                type="button"
                className="text-xs text-custom-text-300 hover:bg-custom-background-80 w-full p-1 text-center"
                onClick={() => setSize((prev) => prev + 1)}
                disabled={isValidating}
              >
                {isValidating ? "Loading…" : "Load more repositories"}
              </button>
            )}
            {isInstallationToken && manageInstallationUrl && (
              <div className="border-custom-border-200 border-t px-2 py-1.5">
                <p className="text-xs text-custom-text-400">
                  {"Can't find a repo? "}
                  <a
                    href={manageInstallationUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-custom-primary-100 hover:text-custom-primary-200 underline underline-offset-2"
                  >
                    Manage GitHub App access
                  </a>
                  {" to grant access to more repositories."}
                </p>
              </div>
            )}
          </>
        }
        optionsClassName="w-56"
      />
    </div>
  );
}
