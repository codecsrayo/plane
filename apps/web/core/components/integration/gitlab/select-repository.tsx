/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import React from "react";
import { useParams } from "react-router";
import useSWRInfinite from "swr/infinite";
import type { IGitlabRepoInfo, IWorkspaceIntegration } from "@plane/types";
import { CustomSearchSelect } from "@plane/ui";
import { truncateText } from "@plane/utils";
import { GitlabIntegrationService } from "@/services/integrations/gitlab.service";

// Tuple that matches exactly what getKey() returns for useSWRInfinite
type TGitlabRepoKey = [workspaceSlug: string, token: string, page: number, cacheTag: string];

type Props = {
  integration: IWorkspaceIntegration;
  value: string | undefined;
  label: string | React.ReactNode;
  onChange: (repo: IGitlabRepoInfo | undefined) => void;
  characterLimit?: number;
  token: string;
};

const gitlabService = new GitlabIntegrationService();

export function SelectGitlabRepository(props: Props) {
  const { value, label, onChange, characterLimit = 25, token } = props;
  const { workspaceSlug } = useParams();

  const getKey = (pageIndex: number) => {
    if (!workspaceSlug || !token) return null;
    return [workspaceSlug, token, pageIndex + 1, "gitlab-repositories"];
  };

  const fetchGitlabRepos = async (key: TGitlabRepoKey) => {
    const [slug, gitToken, page] = key;
    const data = await gitlabService.listAllRepositories(slug, gitToken, page);
    return data;
  };

  const { data: paginatedData, size, setSize, isValidating } = useSWRInfinite(getKey, fetchGitlabRepos);

  const userRepositories = (paginatedData ?? []).flatMap((data) => data.repositories);
  const totalCount = paginatedData && paginatedData.length > 0 ? paginatedData[0].total_count : 0;

  const options =
    userRepositories.map((repo) => ({
      value: repo.id,
      query: repo.full_name,
      content: <p>{truncateText(repo.full_name, characterLimit)}</p>,
    })) ?? [];

  return (
    <CustomSearchSelect
      value={value}
      options={options}
      onChange={(val: string) => {
        const selected = userRepositories.find((r) => r.id === val);
        onChange(selected);
      }}
      label={label}
      footerOption={
        <>
          {userRepositories && options.length < totalCount && (
            <button
              type="button"
              className="w-full p-1 text-center text-10 text-secondary hover:bg-layer-1"
              onClick={() => setSize(size + 1)}
              disabled={isValidating}
            >
              {isValidating ? "Loading..." : "Click to load more..."}
            </button>
          )}
        </>
      }
      optionsClassName="w-48"
    />
  );
}
