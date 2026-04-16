/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IInstanceConfig } from "@plane/types";

export type TIntegrationProvider = "github" | "gitlab" | "slack";

export const getGithubReposSwrKey = (workspaceSlug: string) => `GITHUB_REPOS_${workspaceSlug}`;

export const isIntegrationEnabled = (provider: string, config?: IInstanceConfig | null): boolean => {
  if (provider === "github") return config?.is_github_integration_enabled ?? true;
  if (provider === "gitlab") return config?.is_gitlab_integration_enabled ?? true;
  if (provider === "slack") return config?.is_slack_enabled ?? true;

  return true;
};
