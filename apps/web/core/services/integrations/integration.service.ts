/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type {
  IAppIntegration,
  IImporterService,
  IWorkspaceIntegration,
  IExportServiceResponse,
  IGithubRepoSync,
  IGithubRepository,
  TGithubRepoSyncCreatePayload,
} from "@plane/types";
import { APIService } from "@/services/api.service";
// types
// helper

// Re-export so existing `import { IGithubRepoSync } from "@/services/integrations"`
// call sites keep working. The canonical definition now lives in @plane/types.
export type { IGithubRepoSync } from "@plane/types";

export interface IGithubPRStateMapping {
  id: string;
  project: string;
  state: string;
  github_pr_state: string;
}

export type TIntegrationDeleteResponse = void;

export class IntegrationService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async getAppIntegrationsList(): Promise<IAppIntegration[]> {
    return this.get(`/api/integrations/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getWorkspaceIntegrationsList(workspaceSlug: string): Promise<IWorkspaceIntegration[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/workspace-integrations/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async deleteWorkspaceIntegration(workspaceSlug: string, integrationId: string): Promise<TIntegrationDeleteResponse> {
    return this.delete(`/api/workspaces/${workspaceSlug}/workspace-integrations/${integrationId}/`)
      .then((res) => res?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getImporterServicesList(workspaceSlug: string): Promise<IImporterService[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }
  async getExportsServicesList(
    workspaceSlug: string,
    cursor: string,
    per_page: number
  ): Promise<IExportServiceResponse> {
    return this.get(`/api/workspaces/${workspaceSlug}/export-issues`, {
      params: {
        per_page,
        cursor,
      },
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async deleteImporterService(
    workspaceSlug: string,
    service: string,
    importerId: string
  ): Promise<TIntegrationDeleteResponse> {
    return this.delete(`/api/workspaces/${workspaceSlug}/importers/${service}/${importerId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getRepoSyncs(workspaceSlug: string): Promise<IGithubRepoSync[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/workspace-integrations/github/repo-syncs/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async createRepoSync(workspaceSlug: string, data: TGithubRepoSyncCreatePayload): Promise<IGithubRepoSync> {
    return this.post(`/api/workspaces/${workspaceSlug}/workspace-integrations/github/repo-syncs/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async deleteRepoSync(workspaceSlug: string, syncId: string): Promise<void> {
    return this.delete(`/api/workspaces/${workspaceSlug}/workspace-integrations/github/repo-syncs/${syncId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getGithubRepositories(workspaceSlug: string): Promise<IGithubRepository[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/importers/github/repositories/`)
      .then((response) => {
        const data = response?.data;
        // Backend wraps repos in { repositories: [...], total_count, page }
        return Array.isArray(data) ? data : (data?.repositories ?? []);
      })
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async getPRStateMappings(workspaceSlug: string, workspaceIntegrationId: string): Promise<IGithubPRStateMapping[]> {
    return this.get(
      `/api/workspaces/${workspaceSlug}/workspace-integrations/${workspaceIntegrationId}/pr-state-mappings/`
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async createPRStateMapping(
    workspaceSlug: string,
    workspaceIntegrationId: string,
    data: { project: string; state: string; github_pr_state: string; prevent_regression?: boolean }
  ): Promise<IGithubPRStateMapping> {
    return this.post(
      `/api/workspaces/${workspaceSlug}/workspace-integrations/${workspaceIntegrationId}/pr-state-mappings/`,
      data
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }

  async deletePRStateMapping(workspaceSlug: string, workspaceIntegrationId: string, mappingId: string): Promise<void> {
    return this.delete(
      `/api/workspaces/${workspaceSlug}/workspace-integrations/${workspaceIntegrationId}/pr-state-mappings/${mappingId}/`
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw error?.response?.data;
      });
  }
}

export const integrationService = new IntegrationService();
