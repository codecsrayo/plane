/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type {
  IProjectUserPropertiesResponse,
  ISearchIssueResponse,
  TProjectAnalyticsCount,
  TProjectAnalyticsCountParams,
  TProjectIssuesSearchParams,
} from "@plane/types";
// helpers
// plane web types
import type { TProject, TPartialProject } from "@/plane-web/types";
// services
import { APIService, ApiError } from "@/services/api.service";

export class ProjectService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async createProject(workspaceSlug: string, data: Partial<TProject>): Promise<TProject> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async checkProjectIdentifierAvailability(workspaceSlug: string, data: string): Promise<any> {
    return this.get(`/api/workspaces/${workspaceSlug}/project-identifiers`, {
      params: {
        name: data,
      },
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getProjectsLite(workspaceSlug: string): Promise<TPartialProject[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getProjects(workspaceSlug: string): Promise<TProject[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/details/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getProject(workspaceSlug: string, projectId: string): Promise<TProject> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getProjectAnalyticsCount(
    workspaceSlug: string,
    params?: TProjectAnalyticsCountParams
  ): Promise<TProjectAnalyticsCount[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/project-stats/`, {
      params,
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async updateProject(workspaceSlug: string, projectId: string, data: Partial<TProject>): Promise<TProject> {
    return this.patch(`/api/workspaces/${workspaceSlug}/projects/${projectId}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async deleteProject(workspaceSlug: string, projectId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  // User Properties
  async getProjectUserProperties(workspaceSlug: string, projectId: string): Promise<IProjectUserPropertiesResponse> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-properties/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async updateProjectUserProperties(
    workspaceSlug: string,
    projectId: string,
    data: Partial<IProjectUserPropertiesResponse>
  ): Promise<IProjectUserPropertiesResponse> {
    return this.patch(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-properties/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getUserProjectFavorites(workspaceSlug: string): Promise<any[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/user-favorite-projects/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addProjectToFavorites(workspaceSlug: string, project: string): Promise<any> {
    return this.post(`/api/workspaces/${workspaceSlug}/user-favorite-projects/`, { project })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeProjectFromFavorites(workspaceSlug: string, projectId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/user-favorite-projects/${projectId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async projectIssuesSearch(
    workspaceSlug: string,
    projectId: string,
    params: TProjectIssuesSearchParams
  ): Promise<ISearchIssueResponse[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/search-issues/`, {
      params,
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
