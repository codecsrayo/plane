/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

import type { IInstanceConfig } from "@plane/types";

export type TIntegrationProvider = "github" | "gitlab" | "slack";

const KNOWN_PROVIDERS: readonly TIntegrationProvider[] = ["github", "gitlab", "slack"] as const;

/**
 * Type predicate that narrows an arbitrary string to TIntegrationProvider at runtime.
 * Use this instead of `as TIntegrationProvider` to avoid unsafe casts when the backend
 * returns a provider value we haven't registered yet.
 */
export const isKnownIntegrationProvider = (value: string): value is TIntegrationProvider =>
  (KNOWN_PROVIDERS as readonly string[]).includes(value);

export const getGithubReposSwrKey = (workspaceSlug: string) => `GITHUB_REPOS_${workspaceSlug}`;

export const isIntegrationEnabled = (provider: string, config?: IInstanceConfig | null): boolean => {
  if (provider === "github") return config?.is_github_integration_enabled ?? true;
  if (provider === "gitlab") return config?.is_gitlab_integration_enabled ?? true;
  if (provider === "slack") return config?.is_slack_enabled ?? true;

  return true;
};

/**
 * Returns whether the given provider has been fully configured in God Mode.
 * Extracted from SingleIntegrationCard to keep the component free of provider-specific
 * config key knowledge and to make adding a 4th provider a one-liner here.
 */
export const getIsProviderConfigured = (
  provider: TIntegrationProvider,
  config?: IInstanceConfig | null
): boolean => {
  if (provider === "github") return !!config?.github_app_name;
  if (provider === "gitlab") return !!config?.gitlab_client_id;
  if (provider === "slack") return !!config?.slack_client_id;
  return true;
};
