/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

// types
import { API_BASE_URL } from "@plane/constants";
import type { IModule, ILinkDetails, ModuleLink, TIssuesResponse } from "@plane/types";
// services
import { APIService, ApiError } from "@/services/api.service";

export class ModuleService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async getWorkspaceModules(workspaceSlug: string): Promise<IModule[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/modules/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getModules(workspaceSlug: string, projectId: string): Promise<IModule[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async createModule(workspaceSlug: string, projectId: string, data: any): Promise<IModule> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async updateModule(workspaceSlug: string, projectId: string, moduleId: string, data: any): Promise<any> {
    return this.put(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getModuleDetails(workspaceSlug: string, projectId: string, moduleId: string): Promise<IModule> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async patchModule(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    data: Partial<IModule>
  ): Promise<IModule> {
    return this.patch(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async deleteModule(workspaceSlug: string, projectId: string, moduleId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getModuleIssues(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    queries?: any,
    config = {}
  ): Promise<TIssuesResponse> {
    return this.get(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/issues/`,
      {
        params: queries,
      },
      config
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addIssuesToModule(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    data: { issues: string[] }
  ): Promise<void> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/issues/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addModulesToIssue(
    workspaceSlug: string,
    projectId: string,
    issueId: string,
    data: { modules: string[]; removed_modules?: string[] }
  ): Promise<void> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/issues/${issueId}/modules/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeIssuesFromModuleBulk(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    issueIds: string[]
  ): Promise<void> {
    const promiseDataUrls: any = [];
    issueIds.forEach((issueId) => {
      promiseDataUrls.push(
        this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/issues/${issueId}/`)
      );
    });
    await Promise.all(promiseDataUrls)
      .then((response) => response)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeModulesFromIssueBulk(
    workspaceSlug: string,
    projectId: string,
    issueId: string,
    moduleIds: string[]
  ): Promise<void> {
    const promiseDataUrls: any = [];
    moduleIds.forEach((moduleId) => {
      promiseDataUrls.push(
        this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/issues/${issueId}/`)
      );
    });
    await Promise.all(promiseDataUrls)
      .then((response) => response)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async createModuleLink(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    data: Partial<ModuleLink>
  ): Promise<ILinkDetails> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/module-links/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async updateModuleLink(
    workspaceSlug: string,
    projectId: string,
    moduleId: string,
    linkId: string,
    data: Partial<ModuleLink>
  ): Promise<ILinkDetails> {
    return this.patch(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/module-links/${linkId}/`,
      data
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async deleteModuleLink(workspaceSlug: string, projectId: string, moduleId: string, linkId: string): Promise<any> {
    return this.delete(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/modules/${moduleId}/module-links/${linkId}/`
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addModuleToFavorites(
    workspaceSlug: string,
    projectId: string,
    data: {
      module: string;
    }
  ): Promise<any> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-favorite-modules/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeModuleFromFavorites(workspaceSlug: string, projectId: string, moduleId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-favorite-modules/${moduleId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
