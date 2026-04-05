/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { IGitlabRepoInfo, IGitlabServiceImportFormData } from "@plane/types";
import { APIService } from "@/services/api.service";

const integrationServiceType: string = "gitlab";

export class GitlabIntegrationService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async listAllRepositories(workspaceSlug: string, token: string, page: number = 1): Promise<{ repositories: IGitlabRepoInfo[], total_count: number, page: number }> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/${integrationServiceType}/repositories/`, {
      params: { token, page }
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async listImports(workspaceSlug: string): Promise<any[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/${integrationServiceType}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async createImport(workspaceSlug: string, data: IGitlabServiceImportFormData): Promise<any> {
    return this.post(`/api/workspaces/${workspaceSlug}/importers/${integrationServiceType}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async deleteImport(workspaceSlug: string, importerId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/importers/${integrationServiceType}/${importerId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }
}
