/**
 * Copyright (c) 2023-present Plane Software, Inc. and contributors
 * SPDX-License-Identifier: AGPL-3.0-only
 * See the LICENSE file for details.
 */

// All the app integrations that are available
export interface IAppIntegration {
  author: string;
  avatar_url: string | null;
  created_at: string;
  created_by: string | null;
  description: any;
  id: string;
  metadata: any;
  network: number;
  provider: string;
  redirect_url: string;
  title: string;
  updated_at: string;
  updated_by: string | null;
  verified: boolean;
  webhook_secret: string;
  webhook_url: string;
}

export interface IWorkspaceIntegration {
  actor: string;
  api_token: string;
  config: any;
  created_at: string;
  created_by: string;
  id: string;
  integration: string;
  integration_detail: IAppIntegration;
  metadata: any;
  updated_at: string;
  updated_by: string;
  workspace: string;
}

// slack integration
export interface ISlackIntegration {
  id: string;
  created_at: string;
  updated_at: string;
  access_token: string;
  scopes: string;
  bot_user_id: string;
  webhook_url: string;
  data: ISlackIntegrationData;
  team_id: string;
  team_name: string;
  created_by: string;
  updated_by: string;
  project: string;
  workspace: string;
  workspace_integration: string;
}

export interface ISlackIntegrationData {
  ok: boolean;
  team: {
    id: string;
    name: string;
  };
  scope: string;
  app_id: string;
  enterprise: any;
  token_type: string;
  authed_user: string;
  bot_user_id: string;
  access_token: string;
  incoming_webhook: {
    url: string;
    channel: string;
    channel_id: string;
    configuration_url: string;
  };
  is_enterprise_install: boolean;
}

/**
 * Provider-install payloads accepted by
 *   POST /api/workspaces/{slug}/workspace-integrations/{provider}/install/
 *
 * Shapes mirror the Django backend contract in
 * `apps/api/plane/app/views/integration/base.py::provider_install`:
 *   - github: `{ installation_id, setup_action? }`
 *   - gitlab: `{ code }`
 *   - slack:  `{ code }`
 */
export type TGithubInstallPayload = {
  installation_id: string;
  setup_action?: "install" | "update";
};

export type TGitlabInstallPayload = {
  code: string;
};

export type TSlackInstallPayload = {
  code: string;
};

export type TProviderInstallPayload = TGithubInstallPayload | TGitlabInstallPayload | TSlackInstallPayload;

/**
 * Payload for POST .../project-slack-sync/ — fields mirror `SlackProjectSync`
 * model (`apps/api/plane/db/models/integration/slack.py`).
 */
export type TSlackChannelCreatePayload = {
  access_token: string;
  scopes: string;
  bot_user_id: string;
  webhook_url: string;
  data: ISlackIntegrationData;
  team_id: string;
  team_name: string;
};

/**
 * Repo shape returned by
 *   GET /api/workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories/
 *
 * Mirror of backend mapping in
 * `apps/api/plane/app/views/importer/github.py::GithubRepositoriesEndpoint`
 * (and Rust port in `apps/api_rust/src/routes/integrations/github.rs::list_github_repositories`).
 *
 * Note: `owner` is a **string** (the GitHub login), not an object, and the
 * html URL is keyed as `url` — NOT `html_url`. Callers that assume otherwise
 * are bugs that `any` used to hide.
 *
 * Named `IGithubRepo` rather than `IGithubRepoInfo` to avoid collision with
 * the unrelated `IGithubRepoInfo` in `importer/github-importer.ts` (which
 * represents collaborators/issue-count metadata for the import flow).
 */
export interface IGithubRepo {
  id: string;
  full_name: string;
  name: string;
  owner: string;
  description: string;
  private: boolean;
  url: string;
  issues_count: number;
}

export interface IGithubRepositoriesResponse {
  repositories: IGithubRepo[];
  total_count: number;
  page: number;
  is_installation_token?: boolean;
  manage_installation_url?: string | null;
}

/**
 * Response shape for
 *   GET  /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
 *   POST /api/workspaces/{slug}/workspace-integrations/github/repo-syncs/
 *
 * Guaranteed fields are intersection of Django (`views/integration/base.py::GithubRepositorySyncViewSet`)
 * and Rust (`routes/integrations/github.rs::list_github_repo_syncs / create_github_repo_sync`).
 *
 * Divergence flagged for parity follow-up:
 *   - `project_name` / `project_identifier` / `created_at` → Django only (Rust DTO omits).
 *   - `repo_owner` / `repo_name` → Rust always; Django on LIST but not on CREATE response.
 */
export interface IGithubRepoSync {
  id: string;
  project_id: string;
  repo_id: string;
  repo_full_name: string;
  sync_direction?: "bidirectional" | "unidirectional";
  issue_open_state?: string | null;
  issue_closed_state?: string | null;
  repo_owner?: string;
  repo_name?: string;
  project_name?: string;
  project_identifier?: string;
  created_at?: string;
}

/**
 * Payload for POST .../workspace-integrations/github/repo-syncs/
 * Mirrors `GithubRepoSyncCreateRequest` (Rust) and the body parsed by
 * `GithubRepositorySyncViewSet.create` (Django).
 *
 * `repo_id` is the numeric GitHub repository id (the backend tolerates
 * both string-of-digits and JSON number; we standardize on string to
 * match what `IGithubRepo.id` already exposes).
 */
export type TGithubRepoSyncCreatePayload = {
  repo_id: string;
  repo_full_name: string;
  project_id: string;
  sync_direction?: "bidirectional" | "unidirectional";
  issue_open_state?: string;
  issue_closed_state?: string;
};
