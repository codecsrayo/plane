/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { IGitlabRepoInfo } from "@plane/types";
import { ApiError, APIService } from "@/services/api.service";

export class GitlabIntegrationService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  /**
   * Paginated list of GitLab repositories reachable with the supplied
   * OAuth token. The shape mirrors Django
   * (\`views/integration/base.py::GitlabRepositoriesView\`) and is kept
   * inline here because it is the only response type this service
   * currently produces.
   */
  async listAllRepositories(
    workspaceSlug: string,
    token: string,
    page: number = 1
  ): Promise<{ repositories: IGitlabRepoInfo[]; total_count: number; page: number }> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/gitlab/repositories/`, {
      params: { token, page },
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
