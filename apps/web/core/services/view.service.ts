/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { IProjectView } from "@plane/types";
import { APIService, ApiError } from "@/services/api.service";
// types
// helpers

export class ViewService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async createView(workspaceSlug: string, projectId: string, data: Partial<IProjectView>): Promise<any> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async patchView(workspaceSlug: string, projectId: string, viewId: string, data: Partial<IProjectView>): Promise<any> {
    return this.patch(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/${viewId}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async deleteView(workspaceSlug: string, projectId: string, viewId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/${viewId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getViews(workspaceSlug: string, projectId: string): Promise<IProjectView[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getViewDetails(workspaceSlug: string, projectId: string, viewId: string): Promise<IProjectView> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/${viewId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getViewIssues(workspaceSlug: string, projectId: string, viewId: string): Promise<any> {
    return this.get(`/api/workspaces/${workspaceSlug}/projects/${projectId}/views/${viewId}/issues/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addViewToFavorites(
    workspaceSlug: string,
    projectId: string,
    data: {
      view: string;
    }
  ): Promise<any> {
    return this.post(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-favorite-views/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeViewFromFavorites(workspaceSlug: string, projectId: string, viewId: string): Promise<any> {
    return this.delete(`/api/workspaces/${workspaceSlug}/projects/${projectId}/user-favorite-views/${viewId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
