/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import { API_BASE_URL } from "@plane/constants";
import type { IFavorite } from "@plane/types";
// helpers
// services
import { APIService, ApiError } from "@/services/api.service";
// types

export class FavoriteService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  async addFavorite(workspaceSlug: string, data: Partial<IFavorite>): Promise<IFavorite> {
    return this.post(`/api/workspaces/${workspaceSlug}/user-favorites/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async updateFavorite(workspaceSlug: string, favoriteId: string, data: Partial<IFavorite>): Promise<IFavorite> {
    return this.patch(`/api/workspaces/${workspaceSlug}/user-favorites/${favoriteId}/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async deleteFavorite(workspaceSlug: string, favoriteId: string): Promise<void> {
    return this.delete(`/api/workspaces/${workspaceSlug}/user-favorites/${favoriteId}/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getFavorites(workspaceSlug: string): Promise<IFavorite[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/user-favorites/`, {
      params: {
        all: true,
      },
    })
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getGroupedFavorites(workspaceSlug: string, favoriteId: string): Promise<IFavorite[]> {
    return this.get(`/api/workspaces/${workspaceSlug}/user-favorites/${favoriteId}/group/`)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
