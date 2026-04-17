/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { IGithubRepoInfo, IGithubRepositoriesResponse, IGithubServiceImportFormData } from "@plane/types";
import { APIService } from "@/services/api.service";
// helpers
// types

const integrationServiceType: string = "github";

export class GithubIntegrationService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  /**
   * Paginated list of repositories reachable by a given workspace integration.
   *
   * This hits the per-integration endpoint which respects the installed GitHub
   * App's access scope (as opposed to `/importers/github/repositories/` which
   * is used by the importer flow).
   *
   * Callers must provide the `workspaceIntegrationId` (UUID of the
   * `IWorkspaceIntegration` row), not the integration slug — keeping the
   * parameter name consistent with the rest of the codebase.
   */
  async listAllRepositories(
    workspaceSlug: string,
    workspaceIntegrationId: string,
    page: number = 1
  ): Promise<IGithubRepositoriesResponse> {
    return this.get(
      `/api/workspaces/${workspaceSlug}/workspace-integrations/${workspaceIntegrationId}/github-repositories/`,
      { params: { page } }
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getGithubRepoInfo(workspaceSlug: string, params: { owner: string; repo: string }): Promise<IGithubRepoInfo> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/${integrationServiceType}/`, {
      params,
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async createGithubServiceImport(
    workspaceSlug: string,
    data: IGithubServiceImportFormData
  ): Promise<IGithubServiceImportFormData> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/importers/${integrationServiceType}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }
}
