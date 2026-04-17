/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

// services
import { API_BASE_URL } from "@plane/constants";
import type {
  ISlackIntegration,
  IWorkspaceIntegration,
  TProviderInstallPayload,
  TSlackChannelCreatePayload,
} from "@plane/types";
import { ApiError, APIService } from "@/services/api.service";

/**
 * Service wrapper around workspace-integration endpoints.
 *
 * Backend contract mirrors Django views in
 * `apps/api/plane/app/views/integration/base.py`:
 *   - POST   /workspaces/{slug}/workspace-integrations/{provider}/install/
 *   - POST   /workspaces/{slug}/projects/{pid}/workspace-integrations/{id}/project-slack-sync/
 *   - GET    /workspaces/{slug}/projects/{pid}/workspace-integrations/{id}/project-slack-sync/
 *   - DELETE /workspaces/{slug}/projects/{pid}/workspace-integrations/{id}/project-slack-sync/{sid}
 */
export class AppInstallationService extends APIService {
  constructor() {
    super(API_BASE_URL);
  }

  /**
   * Install a provider integration (github | gitlab | slack) for a workspace.
   * Payload shape is discriminated by `provider`; see `TProviderInstallPayload`.
   * Returns the created/updated `WorkspaceIntegration` as serialized by the backend.
   */
  async addInstallationApp(
    workspaceSlug: string,
    provider: string,
    data: TProviderInstallPayload
  ): Promise<IWorkspaceIntegration> {
    return this.post(`/api/workspaces/${workspaceSlug}/workspace-integrations/${provider}/install/`, data)
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async addSlackChannel(
    workspaceSlug: string,
    projectId: string,
    integrationId: string | null | undefined,
    data: TSlackChannelCreatePayload
  ): Promise<ISlackIntegration> {
    return this.post(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/workspace-integrations/${integrationId}/project-slack-sync/`,
      data
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async getSlackChannelDetail(
    workspaceSlug: string,
    projectId: string,
    integrationId: string | null | undefined
  ): Promise<ISlackIntegration[]> {
    return this.get(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/workspace-integrations/${integrationId}/project-slack-sync/`
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }

  async removeSlackChannel(
    workspaceSlug: string,
    projectId: string,
    integrationId: string | null | undefined,
    slackSyncId: string | undefined
  ): Promise<void> {
    return this.delete(
      `/api/workspaces/${workspaceSlug}/projects/${projectId}/workspace-integrations/${integrationId}/project-slack-sync/${slackSyncId}`
    )
      .then((response) => response?.data)
      .catch((error) => {
        throw new ApiError(error?.response);
      });
  }
}
