/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import { useParams } from "react-router";
import useSWRInfinite from "swr/infinite";
import type { IGithubRepoInfo, IGithubRepositoriesResponse, IWorkspaceIntegration } from "@plane/types";
import { CustomSearchSelect, Loader } from "@plane/ui";
import { truncateText } from "@plane/utils";
import { ProjectService } from "@/services/project";

type Props = {
  integration: IWorkspaceIntegration;
  /** `full_name` of the currently synced repo, or null when none is linked. */
  value: string | null | undefined;
  label: string | React.ReactNode;
  onChange: (repo: IGithubRepoInfo | undefined) => void;
  characterLimit?: number;
};

const projectService = new ProjectService();

export function SelectRepository(props: Props) {
  const { integration, value, label, onChange, characterLimit = 25 } = props;
  const { workspaceSlug } = useParams();

  const getKey = (pageIndex: number) => {
    if (!workspaceSlug || !integration) return undefined;
    return `${process.env.VITE_API_BASE_URL}/api/workspaces/${workspaceSlug}/workspace-integrations/${
      integration.id
    }/github-repositories/?page=${pageIndex + 1}`;
  };

  const fetchGithubRepos = async (url: string): Promise<IGithubRepositoriesResponse> => {
    const data = await projectService.getGithubRepositories(url);
    return data;
  };

  const { data: paginatedData, size, setSize, isValidating, error } = useSWRInfinite<IGithubRepositoriesResponse>(
    getKey,
    fetchGithubRepos
  );

  const isLoading = !paginatedData && !error;

  let userRepositories = (paginatedData ?? []).flatMap((data) => data.repositories ?? []);
  userRepositories = userRepositories.filter((data) => data?.id);

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
      <div className="flex items-center gap-2 text-sm text-custom-text-300">
        <Loader className="h-4 w-4 animate-spin" />
        <span>Loading repositories…</span>
      </div>
    );
  }

  if (error) {
    return (
      <p className="text-sm text-red-500">
        Failed to load repositories. Check your GitHub integration settings.
      </p>
    );
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
                className="w-full p-1 text-center text-xs text-custom-text-300 hover:bg-custom-background-80"
                onClick={() => setSize(size + 1)}
                disabled={isValidating}
              >
                {isValidating ? "Loading…" : "Load more repositories"}
              </button>
            )}
            {isInstallationToken && manageInstallationUrl && (
              <div className="border-t border-custom-border-200 px-2 py-1.5">
                <p className="text-xs text-custom-text-400">
                  {"Can't find a repo? "}
                  <a
                    href={manageInstallationUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-custom-primary-100 underline underline-offset-2 hover:text-custom-primary-200"
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
