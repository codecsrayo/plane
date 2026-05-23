# Django → Rust API Migration Gap Analysis

**Generated:** 2026-05-22  
**Issue:** pl-k38.1  
**Scope:** `apps/api` (Django) vs `apps/api_rust/src/routes` (Rust)

---

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented in Rust |
| ❌ | Missing from Rust — gap |
| ➕ | Rust-only (no Django equivalent; Rust extended beyond Django) |
| ⚠️ | Partial — present but missing some HTTP methods |

**Priority scale:** P0 = blocks migration/users; P1 = API consumers affected; P2 = low-impact / legacy

---

## Executive Summary

The Rust server covers **~95% of the `api/` (session-auth) surface** of `plane.app`. The largest remaining gaps are:

1. **P0 — Space/Deploy Board public API** (`api/public/` → `anchor/*`): entirely absent from Rust.
2. **P1 — Auth OAuth space variants** (`auth/spaces/google`, `auth/spaces/github`, etc.): Rust has credential/magic/space-email-check but no space OAuth flows.
3. **P1 — v1 API gaps**: invitations, stickies, and estimate full-CRUD missing from `/api/v1/`.
4. **P2 — Partial method coverage** on individual cycle/module issue items.
5. **P2 — Legacy V1 file-asset routes** not ported (pre-v2 endpoints).

---

## 1. Authentication (`auth/`)

Django prefix: `auth/` → `plane.authentication.urls`  
Rust prefix: `/auth/`

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `sign-in` | POST | `SignInAuthEndpoint` | `email_auth::sign_in` | ✅ |
| `sign-up` | POST | `SignUpAuthEndpoint` | `email_auth::sign_up` | ✅ |
| `sign-out` | POST | `SignOutAuthEndpoint` | `logout::logout` | ✅ |
| `get-csrf-token` | GET | `CSRFTokenEndpoint` | `csrf::get_csrf_token` | ✅ |
| `email-check` | POST | `EmailCheckEndpoint` | `email_check::email_check` | ✅ |
| `magic-generate` | POST | `MagicGenerateEndpoint` | `magic_auth::magic_generate` | ✅ |
| `magic-sign-in` | POST | `MagicSignInEndpoint` | `magic_auth::magic_sign_in` | ✅ |
| `magic-sign-up` | POST | `MagicSignUpEndpoint` | `magic_auth::magic_sign_up` | ✅ |
| `forgot-password` | POST | `ForgotPasswordEndpoint` | `forgot_reset_password::forgot_password` | ✅ |
| `reset-password/{uidb64}/{token}` | POST | `ResetPasswordEndpoint` | `forgot_reset_password::reset_password` | ✅ |
| `change-password` | POST | `ChangePasswordEndpoint` | `password_management::change_password` | ✅ |
| `set-password` | POST | `SetUserPasswordEndpoint` | `password_management::set_password` | ✅ |
| `google` | GET | `GoogleOauthInitiateEndpoint` | `oauth::google_initiate` | ✅ |
| `google/callback` | GET | `GoogleCallbackEndpoint` | `oauth::google_callback` | ✅ |
| `github` | GET | `GitHubOauthInitiateEndpoint` | `oauth::github_initiate` | ✅ |
| `github/callback` | GET | `GitHubCallbackEndpoint` | `oauth::github_auth_callback` | ✅ |
| `gitlab` | GET | `GitLabOauthInitiateEndpoint` | `oauth::gitlab_initiate` | ✅ |
| `gitlab/callback` | GET | `GitLabCallbackEndpoint` | `oauth::gitlab_callback` | ✅ |
| `gitea` | GET | `GiteaOauthInitiateEndpoint` | `oauth::gitea_initiate` | ✅ |
| `gitea/callback` | GET | `GiteaCallbackEndpoint` | `oauth::gitea_callback` | ✅ |
| `github/user-callback` | GET/POST | `UserGithubConnectionView` | `integrations::github_user_callback` | ✅ |
| **`spaces/sign-in`** | POST | `SignInAuthSpaceEndpoint` | `email_auth::sign_in_space` | ✅ |
| **`spaces/sign-up`** | POST | `SignUpAuthSpaceEndpoint` | `email_auth::sign_up_space` | ✅ |
| **`spaces/sign-out`** | POST | `SignOutAuthSpaceEndpoint` | `logout::logout_space` | ✅ |
| **`spaces/email-check`** | POST | `EmailCheckSpaceEndpoint` | `email_check::email_check_space` | ✅ |
| **`spaces/magic-generate`** | POST | `MagicGenerateSpaceEndpoint` | `magic_auth::magic_generate_space` | ✅ |
| **`spaces/magic-sign-in`** | POST | `MagicSignInSpaceEndpoint` | `magic_auth::magic_sign_in_space` | ✅ |
| **`spaces/magic-sign-up`** | POST | `MagicSignUpSpaceEndpoint` | `magic_auth::magic_sign_up_space` | ✅ |
| **`spaces/forgot-password`** | POST | `ForgotPasswordSpaceEndpoint` | `forgot_reset_password::forgot_password_space` | ✅ |
| **`spaces/reset-password/{uidb64}/{token}`** | POST | `ResetPasswordSpaceEndpoint` | `forgot_reset_password::reset_password_space` | ✅ |
| **`spaces/google`** | GET | `GoogleOauthInitiateSpaceEndpoint` | — | ❌ P1 |
| **`spaces/google/callback`** | GET | `GoogleCallbackSpaceEndpoint` | — | ❌ P1 |
| **`spaces/github`** | GET | `GitHubOauthInitiateSpaceEndpoint` | — | ❌ P1 |
| **`spaces/github/callback`** | GET | `GitHubCallbackSpaceEndpoint` | — | ❌ P1 |
| **`spaces/gitlab`** | GET | `GitLabOauthInitiateSpaceEndpoint` | — | ❌ P1 |
| **`spaces/gitlab/callback`** | GET | `GitLabCallbackSpaceEndpoint` | — | ❌ P1 |
| **`spaces/gitea`** | GET | `GiteaOauthInitiateSpaceEndpoint` | — | ❌ P1 |
| **`spaces/gitea/callback`** | GET | `GiteaCallbackSpaceEndpoint` | — | ❌ P1 |

---

## 2. Instance / License (`api/instances/`)

Django prefix: `api/instances/` → `plane.license.urls`  
Rust prefix: `/api/instances/`

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `` (root) | GET/PATCH | `InstanceEndpoint` | `instances::get_instance` / `patch_instance` | ✅ |
| `admins` | GET/POST | `InstanceAdminEndpoint` | `instances::list_instance_admins` / `create_instance_admin` | ✅ |
| `admins/me` | GET | `InstanceAdminUserMeEndpoint` | `instances::get_instance_admin_me` | ✅ |
| `admins/session` | GET | `InstanceAdminUserSessionEndpoint` | `instances::get_instance_admin_session` | ✅ |
| `admins/sign-in` | POST | `InstanceAdminSignInEndpoint` | `auth::god_mode::admin_sign_in` | ✅ |
| `admins/sign-up` | POST | `InstanceAdminSignUpEndpoint` | `auth::god_mode::admin_sign_up` | ✅ |
| `admins/sign-out` | POST | `InstanceAdminSignOutEndpoint` | `auth::god_mode::admin_sign_out` | ✅ |
| `admins/sign-up-screen-visited` | POST | `SignUpScreenVisitedEndpoint` | `instances::signup_screen_visited` | ✅ |
| `admins/{pk}` | DELETE | `InstanceAdminEndpoint` | `instances::delete_instance_admin` | ✅ |
| `configurations` | GET/PATCH | `InstanceConfigurationEndpoint` | `instances::list_configurations` / `update_configurations` | ✅ |
| `configurations/disable-email-feature` | DELETE | `DisableEmailFeatureEndpoint` | `instances::disable_email_feature` | ✅ |
| `email-credentials-check` | POST | `EmailCredentialCheckEndpoint` | `instances::email_credentials_check` | ✅ |
| `workspace-slug-check` | GET | `InstanceWorkSpaceAvailabilityCheckEndpoint` | `instances::instance_workspace_slug_check` | ✅ |
| `workspaces` | GET | `InstanceWorkSpaceEndpoint` | `instances::list_instance_workspaces` | ✅ |

---

## 3. Main App API (`api/`) — `plane.app`

Django prefix: `api/` → all `plane.app.urls.*`  
Rust prefix: `/api/`

### 3.1 Analytics

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/analytics` | GET | `AnalyticsEndpoint` | `analytics::workspace_analytics` | ✅ |
| `workspaces/{slug}/default-analytics` | GET | `DefaultAnalyticsEndpoint` | `analytics::default_analytics` | ✅ |
| `workspaces/{slug}/project-stats` | GET | `ProjectStatsEndpoint` | `analytics::project_stats` | ✅ |
| `workspaces/{slug}/export-analytics` | POST | `ExportAnalyticsEndpoint` | `analytics::export_analytics` | ✅ |
| `workspaces/{slug}/analytic-view` | GET/POST | `AnalyticViewViewset` | `analytics::list_analytic_views` / `create_analytic_view` | ✅ |
| `workspaces/{slug}/analytic-view/{pk}` | GET/PATCH/DELETE | `AnalyticViewViewset` | `analytics::get/update/delete_analytic_view` | ✅ |
| `workspaces/{slug}/saved-analytic-view/{pk}` | GET | `SavedAnalyticEndpoint` | `analytics::get_saved_analytic_view` | ✅ |
| `workspaces/{slug}/advance-analytics` | GET | `AdvanceAnalyticsEndpoint` | `analytics::advance_analytics` | ✅ |
| `workspaces/{slug}/advance-analytics-stats` | GET | `AdvanceAnalyticsStatsEndpoint` | `analytics::advance_analytics_stats` | ✅ |
| `workspaces/{slug}/advance-analytics-charts` | GET | `AdvanceAnalyticsChartEndpoint` | `analytics::advance_analytics_charts` | ✅ |
| `workspaces/{slug}/projects/{project_id}/advance-analytics` | GET | `ProjectAdvanceAnalyticsEndpoint` | `analytics::project_advance_analytics` | ✅ |
| `workspaces/{slug}/projects/{project_id}/advance-analytics-stats` | GET | `ProjectAdvanceAnalyticsStatsEndpoint` | `analytics::project_advance_analytics_stats` | ✅ |
| `workspaces/{slug}/projects/{project_id}/advance-analytics-charts` | GET | `ProjectAdvanceAnalyticsChartEndpoint` | `analytics::project_advance_analytics_charts` | ✅ |

### 3.2 API Tokens

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `users/api-tokens` | GET/POST | `ApiTokenEndpoint` | `api_tokens::list_api_tokens` / `create_api_token` | ✅ |
| `users/api-tokens/{pk}` | GET/PATCH/DELETE | `ApiTokenEndpoint` | `api_tokens::get/update/delete_api_token` | ✅ |
| `api-tokens` | GET/POST | — (Rust alias) | `api_tokens::list_api_tokens` / `create_api_token` | ➕ |
| `api-tokens/{pk}` | GET/PATCH/DELETE | — (Rust alias) | `api_tokens::get/update/delete_api_token` | ➕ |

### 3.3 Assets

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/file-assets` | GET | `FileAssetEndpoint` (legacy) | — | ❌ P2 |
| `workspaces/file-assets/{workspace_id}/{asset_key}` | DELETE | `FileAssetEndpoint` | `assets::delete_legacy_workspace_file_asset` | ✅ |
| `workspaces/file-assets/{workspace_id}/{asset_key}/restore` | POST | `FileAssetViewSet.restore` | `assets::restore_legacy_workspace_file_asset` | ✅ |
| `users/file-assets` | GET/POST | `UserAssetsEndpoint` (legacy) | — | ❌ P2 |
| `users/file-assets/{asset_key}` | DELETE | `UserAssetsEndpoint` | `assets::delete_legacy_user_file_asset` | ✅ |
| `assets/v2/user-assets` | POST | `UserAssetsV2Endpoint` | `assets::initiate_user_asset_upload` | ✅ |
| `assets/v2/user-assets/{asset_id}` | PATCH/DELETE | `UserAssetsV2Endpoint` | `assets::complete_user_asset_upload` / `delete_user_asset` | ✅ |
| `assets/v2/workspaces/{slug}` | POST | `WorkspaceFileAssetEndpoint` | `assets::initiate_workspace_asset_upload` | ✅ |
| `assets/v2/workspaces/{slug}/{asset_id}` | GET/PATCH/DELETE | `WorkspaceFileAssetEndpoint` | `assets::get/complete/delete_workspace_asset` | ✅ |
| `assets/v2/static/{asset_id}` | GET | `StaticFileAssetEndpoint` | `assets::get_static_asset` | ✅ |
| `assets/v2/workspaces/{slug}/restore/{asset_id}` | POST | `AssetRestoreEndpoint` | `assets::restore_workspace_asset` | ✅ |
| `assets/v2/workspaces/{slug}/check/{asset_id}` | GET | `AssetCheckEndpoint` | `assets::check_workspace_asset` | ✅ |
| `assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}` | POST | `DuplicateAssetEndpoint` | `assets::duplicate_workspace_asset` | ✅ |
| `assets/v2/workspaces/{slug}/download/{asset_id}` | GET | `WorkspaceAssetDownloadEndpoint` | `assets::download_workspace_asset` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}` | POST | `ProjectAssetEndpoint` | `assets::initiate_project_asset_upload` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}/{pk}` | GET/PATCH/DELETE | `ProjectAssetEndpoint` | `assets::get/complete/delete_project_asset` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk` | POST | `ProjectBulkAssetEndpoint` | `assets::bulk_project_assets` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}` | GET | `ProjectAssetDownloadEndpoint` | `assets::download_project_asset` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments` | GET/POST | `IssueAttachmentV2Endpoint` | `assets::list/initiate_issue_attachment_upload_v2` | ✅ |
| `assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}` | PATCH/DELETE | `IssueAttachmentV2Endpoint` | `assets::complete/delete_issue_attachment_v2` | ✅ |
| `assets/v2/workspaces/{slug}/{entity_id}/bulk` | POST | — | `assets::bulk_workspace_assets` | ➕ |

### 3.4 Cycles

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/cycles` | GET/POST | `CycleViewSet` | `cycles::list_cycles` / `create_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{pk}` | GET/PUT/PATCH/DELETE | `CycleViewSet` | `cycles::get/update/delete_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues` | GET/POST | `CycleIssueViewSet` | `cycles::list_cycle_issues` / `add_issues_to_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}` | GET/PUT/PATCH/DELETE | `CycleIssueViewSet` | DELETE only | ⚠️ P2 |
| `workspaces/{slug}/projects/{project_id}/cycles/date-check` | POST | `CycleDateCheckEndpoint` | `cycles::cycle_date_check` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-cycles` | GET/POST | `CycleFavoriteViewSet` | `cycles::list/create_favorite_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}` | DELETE | `CycleFavoriteViewSet` | `cycles::delete_favorite_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues` | POST | `TransferCycleIssueEndpoint` | `cycles::transfer_cycle_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties` | GET/PATCH | `CycleUserPropertiesEndpoint` | `cycles::get/update_cycle_user_properties` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive` | POST/DELETE | `CycleArchiveUnarchiveEndpoint` | `cycles::archive_cycle` / `unarchive_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-cycles` | GET | `CycleArchiveUnarchiveEndpoint` | `cycles::list_archived_cycles` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}` | GET/DELETE | `CycleArchiveUnarchiveEndpoint` | `cycles::get_archived_cycle` / `unarchive_cycle` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress` | GET | `CycleProgressEndpoint` | `cycles::cycle_progress` | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics` | GET | `CycleAnalyticsEndpoint` | `cycles::cycle_analytics` | ✅ |

### 3.5 Estimates

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/project-estimates` | GET | `ProjectEstimatePointEndpoint` | `estimates::list_project_estimates` | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates` | GET/POST | `BulkEstimatePointEndpoint` | `estimates::list/create_estimate` | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}` | GET/PATCH/DELETE | `BulkEstimatePointEndpoint` | `estimates::get/update/delete_estimate` | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points` | POST | `EstimatePointEndpoint` | `estimates::create_estimate_point` | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}` | PATCH/DELETE | `EstimatePointEndpoint` | `estimates::update/delete_estimate_point` | ✅ |

### 3.6 Exporter

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/export-issues` | GET/POST | `ExportIssuesEndpoint` | `exporter::list_export_issues` / `export_issues` | ✅ |
| `workspaces/{slug}/export-issues/{token}` | GET | — | `exporter::get_export_status` | ➕ |

### 3.7 External (AI, Unsplash, Webhooks)

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `unsplash` | GET | `UnsplashEndpoint` | `external::unsplash` | ✅ |
| `github-webhook` | POST | `GitHubWebhookEndpoint` | `external::github_webhook` | ✅ |
| `gitlab-webhook` | POST | `GitLabWebhookEndpoint` | `external::gitlab_webhook` | ✅ |
| `workspaces/{slug}/projects/{project_id}/ai-assistant` | POST | `GPTIntegrationEndpoint` | `external::project_ai_assistant` | ✅ |
| `workspaces/{slug}/ai-assistant` | POST | `WorkspaceGPTIntegrationEndpoint` | `external::workspace_ai_assistant` | ✅ |
| `workspaces/{slug}/rephrase-grammar` | POST | `RephraseGrammarEndpoint` | `external::rephrase_grammar` | ✅ |

### 3.8 Importer

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/importers/github/repositories` | GET | `GithubRepositoriesEndpoint` | `importer::list_github_import_repositories` | ✅ |
| `workspaces/{slug}/importers/github` | GET/POST | `GithubImporterEndpoint` | `importer::list_github_importers` / `create_github_importer` | ✅ |
| `workspaces/{slug}/importers/github/{importer_id}` | DELETE | `GithubImporterEndpoint` | `importer::delete_github_importer` | ✅ |
| `workspaces/{slug}/importers/gitlab/repositories` | GET | `GitlabRepositoriesEndpoint` | `importer::list_gitlab_import_repositories` | ✅ |
| `workspaces/{slug}/importers/gitlab` | GET/POST | `GitlabImporterEndpoint` | `importer::list_gitlab_importers` / `create_gitlab_importer` | ✅ |
| `workspaces/{slug}/importers/gitlab/{importer_id}` | DELETE | `GitlabImporterEndpoint` | `importer::delete_gitlab_importer` | ✅ |
| `workspaces/{slug}/importers` | GET | — | `importer::list_all_importers` | ➕ |

### 3.9 Intake

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/intakes` | GET/POST | `IntakeViewSet` | `intake::list/create_intake` | ✅ |
| `workspaces/{slug}/projects/{project_id}/intakes/{pk}` | GET/PATCH/DELETE | `IntakeViewSet` | `intake::get/update/delete_intake` | ✅ |
| `workspaces/{slug}/projects/{project_id}/inboxes` | GET/POST | `IntakeViewSet` (alias) | `intake::list/create_intake` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/inboxes/{pk}` | GET/PATCH/DELETE | `IntakeViewSet` (alias) | `intake::get/update/delete_intake` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-issues` | GET/POST | `IntakeIssueViewSet` | `intake::list/create_intake_issue` | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-issues/{pk}` | GET/PATCH/DELETE | `IntakeIssueViewSet` | `intake::get/update/delete_intake_issue` | ✅ |
| `workspaces/{slug}/projects/{project_id}/inbox-issues` | GET/POST | `IntakeIssueViewSet` (alias) | `intake::list/create_intake_issue` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/inbox-issues/{pk}` | GET/PATCH/DELETE | `IntakeIssueViewSet` (alias) | `intake::get/update/delete_intake_issue` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions` | GET | `IntakeWorkItemDescriptionVersionEndpoint` | `issue_description_versions::list_description_versions` | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions/{pk}` | GET | `IntakeWorkItemDescriptionVersionEndpoint` | `issue_description_versions::get_description_version` | ✅ |

### 3.10 Integrations

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `github/callback` | GET | `GithubAppCallbackEndpoint` | `integrations::github_app_callback` | ✅ |
| `auth/github/user-callback` | GET/POST | `UserGithubConnectionView` | `integrations::github_user_callback` | ✅ |
| `integrations` | GET | `IntegrationViewSet` | `integrations::list_integrations` | ✅ |
| `workspaces/{slug}/workspace-integrations` | GET/POST | `WorkspaceIntegrationViewSet` | `integrations::list/create_workspace_integration` | ✅ |
| `workspaces/{slug}/workspace-integrations/{pk}` | GET/PATCH/DELETE | `WorkspaceIntegrationViewSet` | `integrations::get/update/delete_workspace_integration` | ✅ |
| `workspaces/{slug}/workspace-integrations/{provider}/provider` | DELETE | `WorkspaceIntegrationViewSet.provider_destroy` | `integrations::delete_workspace_integration_by_provider` | ✅ |
| `workspaces/{slug}/workspace-integrations/{provider}/install` | POST | `WorkspaceIntegrationViewSet.provider_install` | `integrations::provider_install` | ✅ |
| `workspaces/{slug}/workspace-integrations/github/repo-syncs` | GET/POST | `GithubRepoSyncViewSet` | `integrations::list/create_github_repo_sync` | ✅ |
| `workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}` | DELETE | `GithubRepoSyncViewSet` | `integrations::delete_github_repo_sync` | ✅ |
| `workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories` | GET | `GithubRepositoriesEndpoint` | `integrations::list_github_repositories` | ✅ |
| `workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories` | GET | — | `integrations::list_gitlab_repositories` | ➕ |
| `workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings` | GET/POST | `GithubPRStateMappingViewSet` | `integrations::list/create_pr_state_mapping` | ✅ |
| `workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}` | DELETE | `GithubPRStateMappingViewSet` | `integrations::delete_pr_state_mapping` | ✅ |

### 3.11 Issues

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/issues` | GET/POST | `IssueViewSet` | `issues::list_issues` / `create_issue` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/list` | GET | `IssueListEndpoint` | `issues::list_issues_by_ids` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues-detail` | GET | `IssueDetailEndpoint` | `issues::list_issues_detail` | ✅ |
| `workspaces/{slug}/projects/{project_id}/v2/issues` | GET | `IssuePaginatedViewSet` | `issues::list_issues_v2` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{pk}` | GET/PUT/PATCH/DELETE | `IssueViewSet` | `issues::get/update/delete_issue` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issue-labels` | GET/POST | `LabelViewSet` | `labels::list/create_label` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/issue-labels/{pk}` | GET/PUT/PATCH/DELETE | `LabelViewSet` | `labels::get/update/delete_label` (alias) | ✅ |
| `workspaces/{slug}/projects/{project_id}/bulk-create-labels` | POST | `BulkCreateIssueLabelsEndpoint` | `labels::bulk_create_labels` | ✅ |
| `workspaces/{slug}/projects/{project_id}/bulk-delete-issues` | DELETE/POST | `BulkDeleteIssuesEndpoint` | `issue_extras2::bulk_delete_issues` (DELETE+POST) | ✅ |
| `workspaces/{slug}/projects/{project_id}/bulk-archive-issues` | POST | `BulkArchiveIssuesEndpoint` | `issue_extras2::bulk_archive_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/bulk-operation-issues` | POST | — | `issue_extras2::bulk_operation_issues` | ➕ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues` | GET/POST | `SubIssuesEndpoint` | `issue_extras::list/assign_sub_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links` | GET/POST | `IssueLinkViewSet` | `issue_extras::list/create_issue_link` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}` | GET/PUT/PATCH/DELETE | `IssueLinkViewSet` | `issue_extras::update/delete_issue_link` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments` | GET/POST | `IssueAttachmentEndpoint` | `issue_extras2::list/initiate_issue_attachment_upload` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}` | PATCH/DELETE | `IssueAttachmentEndpoint` | `issue_extras2::complete/delete_issue_attachment` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history` | GET | `IssueActivityEndpoint` | `issue_extras::list_issue_activities` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities` | GET | — (Rust alias) | `issue_extras::list_issue_activities` | ➕ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments` | GET/POST | `IssueCommentViewSet` | `issue_extras::list/create_comment` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}` | GET/PUT/PATCH/DELETE | `IssueCommentViewSet` | `issue_extras::get/update/delete_comment` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers` | GET | `IssueSubscriberViewSet` | `issue_extras::list_issue_subscribers` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}` | DELETE | `IssueSubscriberViewSet` | `issue_extras::delete_issue_subscriber` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe` | GET/POST/DELETE | `IssueSubscriberViewSet` | `issue_extras::get_subscription_status/subscribe/unsubscribe` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions` | GET/POST | `IssueReactionViewSet` | `issue_extras::list/add_issue_reaction` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}` | DELETE | `IssueReactionViewSet` | `issue_extras::remove_issue_reaction` | ✅ |
| `workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions` | GET/POST | `CommentReactionViewSet` | `issue_extras::list/add_comment_reaction` | ✅ |
| `workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}` | DELETE | `CommentReactionViewSet` | `issue_extras::remove_comment_reaction` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-properties` | GET/PATCH | `ProjectUserDisplayPropertyEndpoint` | `project_user_properties::get/update_project_user_properties` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-issues` | GET | `IssueArchiveViewSet` | `issue_extras2::list_archived_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{pk}/archive` | GET/POST/DELETE | `IssueArchiveViewSet` | `issue_extras2::get_archived/archive/unarchive_issue` | ✅ |
| `workspaces/{slug}/projects/{project_id}/deleted-issues` | GET | `DeletedIssuesListViewSet` | `issue_extras2::list_deleted_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation` | GET/POST | `IssueRelationViewSet` | `issue_extras::list/create_issue_relation` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation` | POST | `IssueRelationViewSet.remove_relation` | `issue_extras::remove_issue_relation` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issue-dates` | POST | `IssueBulkUpdateDateEndpoint` | `issue_extras2::bulk_update_issue_dates` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions` | GET | `IssueVersionEndpoint` | `issue_extras2::list_issue_versions` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}` | GET | `IssueVersionEndpoint` | `issue_extras2::get_issue_version` | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions` | GET | `WorkItemDescriptionVersionEndpoint` | `issue_description_versions::list_description_versions` | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}` | GET | `WorkItemDescriptionVersionEndpoint` | `issue_description_versions::get_description_version` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta` | GET | `IssueMetaEndpoint` | `issue_extras2::get_issue_meta` | ✅ |
| `workspaces/{slug}/work-items/{project_identifier}-{issue_identifier}` | GET | `IssueDetailIdentifierEndpoint` | `issue_extras2::get_issue_by_identifier` | ✅ |
| `workspaces/{slug}/issues/{combined}` | GET | — | `issue_extras2::get_issue_by_identifier` (alias) | ➕ |

### 3.12 Labels

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/labels` | GET/POST | `LabelViewSet` | `labels::list/create_label` | ✅ |
| `workspaces/{slug}/projects/{project_id}/labels/{pk}` | GET/PATCH/DELETE | `LabelViewSet` | `labels::get/update/delete_label` | ✅ |

### 3.13 Modules

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/modules` | GET/POST | `ModuleViewSet` | `modules::list/create_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{pk}` | GET/PUT/PATCH/DELETE | `ModuleViewSet` | `modules::get/update/delete_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules` | POST | `ModuleIssueViewSet.create_issue_modules` | `modules::set_issue_modules` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues` | GET/POST | `ModuleIssueViewSet` | `modules::list/add_issues_to_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}` | GET/PUT/PATCH/DELETE | `ModuleIssueViewSet` | DELETE only | ⚠️ P2 |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links` | GET/POST | `ModuleLinkViewSet` | `modules::list/create_module_link` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}` | GET/PUT/PATCH/DELETE | `ModuleLinkViewSet` | `modules::get/update/delete_module_link` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-modules` | GET/POST | `ModuleFavoriteViewSet` | `modules::list/create_favorite_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}` | DELETE | `ModuleFavoriteViewSet` | `modules::delete_favorite_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties` | GET/PATCH | `ModuleUserPropertiesEndpoint` | `modules::get/update_module_user_properties` | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive` | POST/DELETE | `ModuleArchiveUnarchiveEndpoint` | `modules::archive/unarchive_module` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-modules` | GET | `ModuleArchiveUnarchiveEndpoint` | `modules::list_archived_modules` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-modules/{pk}` | GET/DELETE | `ModuleArchiveUnarchiveEndpoint` | `modules::get_archived_module` / `unarchive_module` | ✅ |

### 3.14 Notifications

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/users/notifications` | GET | `NotificationViewSet` | `notifications::list_notifications` | ✅ |
| `workspaces/{slug}/users/notifications/{pk}` | GET/PATCH/DELETE | `NotificationViewSet` | `notifications::get/update/delete_notification` | ✅ |
| `workspaces/{slug}/users/notifications/{pk}/read` | POST/DELETE | `NotificationViewSet` | `notifications::mark_read` / `mark_unread` | ✅ |
| `workspaces/{slug}/users/notifications/{pk}/archive` | POST/DELETE | `NotificationViewSet` | `notifications::archive/unarchive_notification` | ✅ |
| `workspaces/{slug}/users/notifications/unread` | GET | `UnreadNotificationEndpoint` | `notifications::unread_count` | ✅ |
| `workspaces/{slug}/users/notifications/mark-all-read` | POST | `MarkAllReadNotificationViewSet` | `notifications::mark_all_read` | ✅ |
| `users/me/notification-preferences` | GET/PATCH | `UserNotificationPreferenceEndpoint` | `notifications::get/update_user_notification_preferences` | ✅ |

### 3.15 Pages

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/pages-summary` | GET | `PageViewSet.summary` | `pages::pages_summary` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages` | GET/POST | `PageViewSet` | `pages::list_pages` / `create_page` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}` | GET/PATCH/DELETE | `PageViewSet` | `pages::get/update/delete_page` | ✅ |
| `workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}` | POST/DELETE | `PageFavoriteViewSet` | `pages::add/remove_page_favorite` | ✅ |
| `workspaces/{slug}/projects/{project_id}/favorite-pages` | GET | — | `pages::list_favorite_pages` | ➕ |
| `workspaces/{slug}/projects/{project_id}/archived-pages` | GET | — | `pages::list_archived_pages` | ➕ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive` | POST/DELETE | `PageViewSet` | `pages::archive/unarchive_page` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock` | POST/DELETE | `PageViewSet` | `pages::lock/unlock_page` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/access` | POST | `PageViewSet.access` | `pages::update_page_access` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/description` | GET/PATCH | `PagesDescriptionViewSet` | `pages::get/update_page_description` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions` | GET | `PageVersionEndpoint` | `pages::list_page_versions` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}` | GET | `PageVersionEndpoint` | `pages::get_page_version` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate` | POST | `PageDuplicateEndpoint` | `pages::duplicate_page` | ✅ |
| `workspaces/{slug}/projects/{project_id}/pages/{page_id}/move` | POST | — | `pages::move_page` | ➕ |

### 3.16 Projects

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects` | GET/POST | `ProjectViewSet` | `projects::list/create_project` | ✅ |
| `workspaces/{slug}/projects/details` | GET | `ProjectViewSet.list_detail` | `projects::list_projects_detail` | ✅ |
| `workspaces/{slug}/projects/{pk}` | GET/PUT/PATCH/DELETE | `ProjectViewSet` | `projects::get/update/delete_project` | ✅ |
| `workspaces/{slug}/project-identifiers` | GET/DELETE | `ProjectIdentifierEndpoint` | `projects::check/delete_project_identifier` | ✅ |
| `workspaces/{slug}/projects/{project_id}/invitations` | GET/POST | `ProjectInvitationsViewset` | `projects::list/create_project_invitations` | ✅ |
| `workspaces/{slug}/projects/{project_id}/invitations/{pk}` | GET/DELETE | `ProjectInvitationsViewset` | `projects::get/delete_project_invitation` | ✅ |
| `users/me/workspaces/{slug}/projects/invitations` | GET/POST | `UserProjectInvitationsViewset` | `projects::list/join_user_project_invitations` | ✅ |
| `users/me/workspaces/{slug}/project-roles` | GET | `UserProjectRolesEndpoint` | `users::get_user_project_roles` | ✅ |
| `workspaces/{slug}/projects/{project_id}/join/{pk}` | POST | `ProjectJoinEndpoint` | `projects::join_project_invitation` | ✅ |
| `workspaces/{slug}/projects/{project_id}/members` | GET/POST | `ProjectMemberViewSet` | `projects::list/create_project_members` | ✅ |
| `workspaces/{slug}/projects/{project_id}/members/{pk}` | GET/PATCH/DELETE | `ProjectMemberViewSet` | `projects::get/update/remove_project_member` | ✅ |
| `workspaces/{slug}/projects/{project_id}/members/leave` | POST | `ProjectMemberViewSet.leave` | `projects::leave_project` | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-views` | GET/POST | `ProjectUserViewsEndpoint` | `projects::get/update_project_views` | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-members/me` | GET | `ProjectMemberUserEndpoint` | `projects::get_project_member_me` | ✅ |
| `workspaces/{slug}/user-favorite-projects` | GET/POST | `ProjectFavoritesViewSet` | `projects::list/create_project_favorite` | ✅ |
| `workspaces/{slug}/user-favorite-projects/{project_id}` | DELETE | `ProjectFavoritesViewSet` | `projects::delete_project_favorite` | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-deploy-boards` | GET/POST | `DeployBoardViewSet` | `projects::get/upsert_project_deploy_board` | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-deploy-boards/{pk}` | GET/PATCH/DELETE | `DeployBoardViewSet` | `projects::get/update/delete_project_deploy_board` | ✅ |
| `workspaces/{slug}/projects/{project_id}/archive` | POST/DELETE | `ProjectArchiveUnarchiveEndpoint` | `projects::archive/unarchive_project` | ✅ |
| `workspaces/{slug}/projects/{project_id}/preferences/member/{member_id}` | GET/PATCH | `ProjectMemberPreferenceEndpoint` | `projects::get/update_project_member_preferences` | ✅ |
| `workspaces/{slug}/projects/{project_id}/summary` | GET | — | `projects::get_project_summary` | ➕ |

### 3.17 Search

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/search` | GET | `GlobalSearchEndpoint` | `search::global_search` | ✅ |
| `workspaces/{slug}/projects/{project_id}/search-issues` | GET | `IssueSearchEndpoint` | `search::search_issues` | ✅ |
| `workspaces/{slug}/entity-search` | GET | `SearchEndpoint` | `search::entity_search` | ✅ |
| `workspaces/{slug}/work-items/search` | GET | — | `search::global_search` (alias) | ➕ |
| `workspaces/{slug}/issues/search` | GET | — | `search::global_search` (alias) | ➕ |

### 3.18 States

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/states` | GET/POST | `StateViewSet` | `states::list/create_state` | ✅ |
| `workspaces/{slug}/projects/{project_id}/states/{pk}` | GET/PATCH/DELETE | `StateViewSet` | `states::get/update/delete_state` | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-state` | GET | `IntakeStateEndpoint` | `states::intake_state` | ✅ |
| `workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default` | POST | `StateViewSet.mark_as_default` | `states::mark_default` | ✅ |

### 3.19 Timezones

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `timezones` | GET | `TimezoneEndpoint` | `timezones::list_timezones` | ✅ |

### 3.20 Users

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `users/me` | GET/PATCH/DELETE | `UserEndpoint` | `users::get/update_me` / `deactivate_me` | ✅ |
| `users/session` | GET | `UserSessionEndpoint` | `users::get_session` | ✅ |
| `users/me/settings` | GET | `UserEndpoint.retrieve_user_settings` | `users::get_settings` | ✅ |
| `users/me/instance-admin` | GET | `UserEndpoint.retrieve_instance_admin` | `users::get_instance_admin` | ✅ |
| `users/me/email/generate-code` | POST | `UserEndpoint.generate_email_verification_code` | `users::generate_email_code` | ✅ |
| `users/me/email` | PATCH | `UserEndpoint.update_email` | `users::update_user_email` | ✅ |
| `users/me/profile` | GET/PATCH | `ProfileEndpoint` | `users::get/update_profile` | ✅ |
| `users/me/accounts` | GET | `AccountEndpoint` | `users::list_accounts` | ✅ |
| `users/me/accounts/{pk}` | GET/DELETE | `AccountEndpoint` | `users::get/delete_account` | ✅ |
| `users/me/onboard` | PATCH | `UpdateUserOnBoardedEndpoint` | `users::update_onboard` | ✅ |
| `users/me/tour-completed` | PATCH | `UpdateUserTourCompletedEndpoint` | `users::update_tour_completed` | ✅ |
| `users/me/activities` | GET | `UserActivityEndpoint` | `users::get_my_activities` | ✅ |
| `users/me/workspaces` | GET | `UserWorkSpacesEndpoint` | `users::list_user_workspaces` | ✅ |
| `users/me/workspaces/invitations` | GET/POST | `UserWorkspaceInvitationsViewSet` | `users::list/join_user_workspace_invitations` | ✅ |
| `users/me/workspaces/{slug}/activity-graph` | GET | `UserActivityGraphEndpoint` | `users::get_activity_graph` | ✅ |
| `users/me/workspaces/{slug}/issues-completed-graph` | GET | `UserIssueCompletedGraphEndpoint` | `users::get_issues_completed_graph` | ✅ |
| `users/me/workspaces/{slug}/dashboard` | GET | `UserWorkspaceDashboardEndpoint` | `users::get_workspace_dashboard` | ✅ |
| `users/last-visited-workspace` | GET | `UserLastProjectWithWorkspaceEndpoint` | `users::get_last_workspace` | ✅ |

### 3.21 Views (Issue Views)

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/projects/{project_id}/views` | GET/POST | `IssueViewViewSet` | `views::list/create_project_view` | ✅ |
| `workspaces/{slug}/projects/{project_id}/views/{pk}` | GET/PUT/PATCH/DELETE | `IssueViewViewSet` | `views::get/update/delete_project_view` | ✅ |
| `workspaces/{slug}/views` | GET/POST | `WorkspaceViewViewSet` | `views::list/create_workspace_view` | ✅ |
| `workspaces/{slug}/views/{pk}` | GET/PUT/PATCH/DELETE | `WorkspaceViewViewSet` | `views::get/update/delete_workspace_view` | ✅ |
| `workspaces/{slug}/issues` | GET | `WorkspaceViewIssuesViewSet` | `workspace_view_issues::list_workspace_view_issues` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-views` | GET/POST | `IssueViewFavoriteViewSet` | `views::list/add_favorite_view` | ✅ |
| `workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}` | DELETE | `IssueViewFavoriteViewSet` | `views::remove_favorite_view` | ✅ |

### 3.22 Webhooks

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspaces/{slug}/webhooks` | GET/POST | `WebhookEndpoint` | `webhooks::list/create_webhook` | ✅ |
| `workspaces/{slug}/webhooks/{pk}` | GET/PATCH/DELETE | `WebhookEndpoint` | `webhooks::get/update/delete_webhook` | ✅ |
| `workspaces/{slug}/webhooks/{pk}/regenerate` | POST | `WebhookSecretRegenerateEndpoint` | `webhooks::regenerate_secret` | ✅ |
| `workspaces/{slug}/webhook-logs/{webhook_id}` | GET | `WebhookLogsEndpoint` | `webhooks::list_webhook_logs` | ✅ |

### 3.23 Workspaces

| Route | Method | Django impl | Rust impl | Status |
|-------|--------|------------|-----------|--------|
| `workspace-slug-check` | GET | `WorkSpaceAvailabilityCheckEndpoint` | `workspaces::slug_check` | ✅ |
| `workspaces` | GET/POST | `WorkSpaceViewSet` | `workspaces::list/create_workspace` | ✅ |
| `workspaces/{slug}` | GET/PUT/PATCH/DELETE | `WorkSpaceViewSet` | `workspaces::get/update/delete_workspace` | ✅ |
| `workspaces/{slug}/invitations` | GET/POST | `WorkspaceInvitationsViewset` | `workspaces::list/create_invitations` | ✅ |
| `workspaces/{slug}/invitations/{pk}` | GET/PATCH/DELETE | `WorkspaceInvitationsViewset` | `workspaces::get/update/delete_invitation` | ✅ |
| `workspaces/{slug}/invitations/{pk}/join` | GET/POST | `WorkspaceJoinEndpoint` | `workspaces::get/join_workspace_invitation` | ✅ |
| `users/me/workspaces/invitations` | GET/POST | `UserWorkspaceInvitationsViewSet` | `users::list/join_user_workspace_invitations` | ✅ |
| `workspaces/{slug}/members` | GET | `WorkSpaceMemberViewSet` | `workspaces::list_members` | ✅ |
| `workspaces/{slug}/members/{pk}` | GET/PATCH/DELETE | `WorkSpaceMemberViewSet` | `workspaces::get/update/remove_member` | ✅ |
| `workspaces/{slug}/members/leave` | POST | `WorkSpaceMemberViewSet.leave` | `workspaces::leave_workspace` | ✅ |
| `workspaces/{slug}/project-members` | GET | `WorkspaceProjectMemberEndpoint` | `workspaces::get_project_members` | ✅ |
| `users/last-visited-workspace` | GET | `UserLastProjectWithWorkspaceEndpoint` | `users::get_last_workspace` | ✅ |
| `workspaces/{slug}/workspace-members/me` | GET | `WorkspaceMemberUserEndpoint` | `workspaces::get_workspace_member_me` | ✅ |
| `workspaces/{slug}/workspace-views` | GET/POST | `WorkspaceMemberUserViewsEndpoint` | `workspaces::get/update_workspace_views` | ✅ |
| `workspaces/{slug}/workspace-themes` | GET/POST | `WorkspaceThemeViewSet` | `workspaces::list/create_workspace_theme` | ✅ |
| `workspaces/{slug}/workspace-themes/{pk}` | GET/PATCH/DELETE | `WorkspaceThemeViewSet` | `workspaces::get/update/delete_workspace_theme` | ✅ |
| `workspaces/{slug}/user-stats/{user_id}` | GET | `WorkspaceUserProfileStatsEndpoint` | `workspaces::get_user_stats` | ✅ |
| `workspaces/{slug}/user-activity/{user_id}` | GET | `WorkspaceUserActivityEndpoint` | `workspaces::get_workspace_user_activity` | ✅ |
| `workspaces/{slug}/user-activity/{user_id}/export` | GET/POST | `ExportWorkspaceUserActivityEndpoint` | `workspaces::export_workspace_user_activity` | ✅ |
| `workspaces/{slug}/user-profile/{user_id}` | GET | `WorkspaceUserProfileEndpoint` | `workspaces::get_user_profile` | ✅ |
| `workspaces/{slug}/user-issues/{user_id}` | GET | `WorkspaceUserProfileIssuesEndpoint` | `user_profile_issues::list_user_profile_issues` | ✅ |
| `workspaces/{slug}/labels` | GET | `WorkspaceLabelsEndpoint` | `workspace_extras::list_workspace_labels` | ✅ |
| `workspaces/{slug}/user-properties` | GET/PATCH | `WorkspaceUserPropertiesEndpoint` | `workspace_extras::get/update_workspace_user_properties` | ✅ |
| `workspaces/{slug}/states` | GET | `WorkspaceStatesEndpoint` | `workspace_extras::list_workspace_states` | ✅ |
| `workspaces/{slug}/estimates` | GET | `WorkspaceEstimatesEndpoint` | `workspace_extras::list_workspace_estimates` | ✅ |
| `workspaces/{slug}/modules` | GET | `WorkspaceModulesEndpoint` | `workspace_extras::list_workspace_modules` | ✅ |
| `workspaces/{slug}/cycles` | GET | `WorkspaceCyclesEndpoint` | `workspace_extras::list_workspace_cycles` | ✅ |
| `workspaces/{slug}/active-cycles` | GET | — | `workspace_extras::list_workspace_active_cycles` | ➕ |
| `workspaces/{slug}/user-favorites` | GET/POST | `WorkspaceFavoriteEndpoint` | `workspace_extras::list/create_favorite` | ✅ |
| `workspaces/{slug}/user-favorites/{favorite_id}` | PATCH/DELETE | `WorkspaceFavoriteEndpoint` | `workspace_extras::update/delete_favorite` | ✅ |
| `workspaces/{slug}/user-favorites/{favorite_id}/group` | GET | `WorkspaceFavoriteGroupEndpoint` | `workspace_extras::list_favorite_children` (alias) | ✅ |
| `workspaces/{slug}/user-favorites/{favorite_id}/children` | GET | — | `workspace_extras::list_favorite_children` | ➕ |
| `workspaces/{slug}/draft-issues` | GET/POST | `WorkspaceDraftIssueViewSet` | `workspace_extras::list/create_draft_issue` | ✅ |
| `workspaces/{slug}/draft-issues/{pk}` | GET/PATCH/DELETE | `WorkspaceDraftIssueViewSet` | `workspace_extras::get/update/delete_draft_issue` | ✅ |
| `workspaces/{slug}/draft-to-issue/{draft_id}` | POST | `WorkspaceDraftIssueViewSet.create_draft_to_issue` | `workspace_extras::draft_to_issue` | ✅ |
| `workspaces/{slug}/quick-links` | GET/POST | `QuickLinkViewSet` | `workspace_extras::list/create_quick_link` | ✅ |
| `workspaces/{slug}/quick-links/{pk}` | PATCH/DELETE | `QuickLinkViewSet` | `workspace_extras::update/delete_quick_link` | ✅ |
| `workspaces/{slug}/home-preferences` | GET | `WorkspaceHomePreferenceViewSet` | `workspace_extras::get_home_preferences` | ✅ |
| `workspaces/{slug}/home-preferences/{key}` | GET/PATCH | `WorkspaceHomePreferenceViewSet` | `workspace_extras::get/update_home_preference` | ✅ |
| `workspaces/{slug}/recent-visits` | GET | `UserRecentVisitViewSet` | `workspace_extras::list_recent_visits` | ✅ |
| `workspaces/{slug}/stickies` | GET/POST | `WorkspaceStickyViewSet` | `workspace_extras::list/create_sticky` | ✅ |
| `workspaces/{slug}/stickies/{pk}` | PATCH/DELETE | `WorkspaceStickyViewSet` | `workspace_extras::update/delete_sticky` | ✅ |
| `workspaces/{slug}/sidebar-preferences` | GET/PATCH | `WorkspaceUserPreferenceViewSet` | `workspace_extras::get/update_user_preferences` | ✅ |

---

## 4. Public Space API (`api/public/`) — `plane.space` — **ENTIRELY MISSING**

Django prefix: `api/public/` → `plane.space.urls`  
Rust prefix: none

**These endpoints serve the public deploy-board (Spaces) feature. None exist in Rust.**

| Route | Method | Django impl | Rust impl | Priority |
|-------|--------|------------|-----------|----------|
| `anchor/{anchor}/meta` | GET | `ProjectMetaDataEndpoint` | ❌ | P0 |
| `anchor/{anchor}/settings` | GET | `ProjectDeployBoardPublicSettingsEndpoint` | ❌ | P0 |
| `anchor/{anchor}/issues` | GET | `ProjectIssuesPublicEndpoint` | ❌ | P0 |
| `workspaces/{slug}/projects/{project_id}/anchor` | GET | `WorkspaceProjectAnchorEndpoint` | ❌ | P0 |
| `anchor/{anchor}/cycles` | GET | `ProjectCyclesEndpoint` | ❌ | P0 |
| `anchor/{anchor}/modules` | GET | `ProjectModulesEndpoint` | ❌ | P0 |
| `anchor/{anchor}/states` | GET | `ProjectStatesEndpoint` | ❌ | P0 |
| `anchor/{anchor}/labels` | GET | `ProjectLabelsEndpoint` | ❌ | P0 |
| `anchor/{anchor}/members` | GET | `ProjectMembersEndpoint` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}` | GET | `IssueRetrievePublicEndpoint` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}/comments` | GET/POST | `IssueCommentPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}/comments/{pk}` | GET/PATCH/DELETE | `IssueCommentPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}/reactions` | GET/POST | `IssueReactionPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}/reactions/{reaction_code}` | DELETE | `IssueReactionPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/comments/{comment_id}/reactions` | GET/POST | `CommentReactionPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/comments/{comment_id}/reactions/{reaction_code}` | DELETE | `CommentReactionPublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/issues/{issue_id}/votes` | GET/POST/DELETE | `IssueVotePublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/intakes/{intake_id}/intake-issues` | GET/POST | `IntakeIssuePublicViewSet` | ❌ | P0 |
| `anchor/{anchor}/intakes/{intake_id}/inbox-issues` | GET/POST | `IntakeIssuePublicViewSet` (alias) | ❌ | P0 |
| `anchor/{anchor}/intakes/{intake_id}/intake-issues/{pk}` | GET/PATCH/DELETE | `IntakeIssuePublicViewSet` | ❌ | P0 |
| `workspaces/{slug}/project-boards` | GET | `WorkspaceProjectDeployBoardEndpoint` | ❌ | P0 |
| `assets/v2/anchor/{anchor}` | POST/GET | `EntityAssetEndpoint` | ❌ | P0 |
| `assets/v2/anchor/{anchor}/{pk}` | GET/PATCH/DELETE | `EntityAssetEndpoint` | ❌ | P0 |
| `assets/v2/anchor/{anchor}/restore/{pk}` | POST | `AssetRestoreEndpoint` | ❌ | P0 |
| `assets/v2/anchor/{anchor}/{entity_id}/bulk` | POST | `EntityBulkAssetEndpoint` | ❌ | P0 |

---

## 5. Public v1 API (`api/v1/`) — `plane.api`

Django prefix: `api/v1/` → `plane.api.urls`  
Rust prefix: `/api/v1/`

### 5.1 Covered in Rust v1

| Route | Method | Status |
|-------|--------|--------|
| `users/me` | GET/PATCH | ✅ |
| `workspaces/{slug}/members` | GET | ✅ |
| `workspaces/{slug}/projects` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/archive` | POST/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/summary` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/members` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/members/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-members` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/project-members/{pk}` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/states` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/states/{state_id}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/labels` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/labels/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates` | GET | ✅ (GET only) |
| `workspaces/{slug}/projects/{project_id}/estimates/{id}/estimate-points` | POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/estimates/{id}/estimate-points/{pk}` | PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}` | DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues` | POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive` | POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-cycles` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}/unarchive` | DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-issues/{issue_id}` | DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/modules/{pk}/archive` | POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-modules` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-modules/{pk}` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/archived-modules/{pk}/unarchive` | DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/links/{pk}` | PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/activities/{pk}` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}` | PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-issues` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/intake-issues/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/links/{pk}` | PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/comments/{pk}` | GET/PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}` | GET | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments` | GET/POST | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/attachments/{pk}` | PATCH/DELETE | ✅ |
| `workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/relations` | GET/POST | ✅ |
| `assets/user-assets` | POST | ✅ |
| `assets/user-assets/{asset_id}` | PATCH/DELETE | ✅ |
| `assets/user-assets/server` | POST | ✅ |
| `assets/user-assets/{asset_id}/server` | POST | ✅ |
| `workspaces/{slug}/assets` | POST | ✅ |
| `workspaces/{slug}/assets/{asset_id}` | GET/PATCH/DELETE | ✅ |

### 5.2 Missing from Rust v1

| Route | Method | Django impl | Status | Priority |
|-------|--------|------------|--------|----------|
| `workspaces/{slug}/invitations` | GET/POST/PATCH/DELETE | `WorkspaceInvitationsViewset` (DRF Router) | ❌ | P1 |
| `workspaces/{slug}/invitations/{pk}` | GET/PATCH/DELETE | `WorkspaceInvitationsViewset` | ❌ | P1 |
| `workspaces/{slug}/stickies` | GET/POST/PATCH/DELETE | `StickyViewSet` (DRF Router) | ❌ | P1 |
| `workspaces/{slug}/stickies/{pk}` | GET/PATCH/DELETE | `StickyViewSet` | ❌ | P1 |
| `workspaces/{slug}/projects/{project_id}/estimates` | POST/PATCH/DELETE | `ProjectEstimateAPIEndpoint` (full CRUD) | ⚠️ GET only | P1 |

---

## 6. Gap Summary by Priority

### P0 — Blocks Migration (Space/public API entirely absent)

The entire `plane.space` application serving the public deploy-board feature (`api/public/`) is absent from Rust. This includes 25 routes covering:
- Anchor-based project browsing (settings, issues, cycles, modules, states, labels, members)
- Public issue viewing and commenting
- Issue reactions and voting
- Public intake/inbox creation
- Public asset uploads

**Estimated porting effort:** Large — requires new auth model (unauthenticated/anchor-based), new handlers for all 25 routes.

### P1 — API Key Consumers Affected

| Gap | Routes affected | Notes |
|-----|----------------|-------|
| Auth spaces OAuth | 8 | OAuth flows for public spaces login via social |
| v1 Invitations | 2 | Workspace invitation CRUD via API key |
| v1 Stickies | 2 | Workspace sticky notes via API key |
| v1 Estimate full CRUD | 1 | POST/PATCH/DELETE on estimates |

### P2 — Low Impact / Legacy

| Gap | Routes affected | Notes |
|-----|----------------|-------|
| Cycle issue item methods | 1 path | Missing GET/PUT/PATCH (only DELETE on individual cycle-issue) |
| Module issue item methods | 1 path | Missing GET/PUT/PATCH (only DELETE on individual module-issue) |
| Legacy workspace file-assets GET | 1 | Pre-v2 listing endpoint |
| Legacy user file-assets GET/POST | 1 | Pre-v2 user asset upload/list |

---

## 7. Rust Extras (in Rust but not in Django)

These endpoints exist in Rust but have no Django counterpart. They represent intentional improvements or Rust-side additions:

| Route | Handler | Notes |
|-------|---------|-------|
| `api/workspaces/{slug}/active-cycles` | `workspace_extras::list_workspace_active_cycles` | Explicit active-cycles list (Django uses filter on `/cycles/`) |
| `api/workspaces/{slug}/user-favorites/{id}/children` | `workspace_extras::list_favorite_children` | Alias of `/group/` |
| `api/workspaces/{slug}/projects/{project_id}/favorite-pages` | `pages::list_favorite_pages` | Favorite pages list (Django uses filters) |
| `api/workspaces/{slug}/projects/{project_id}/archived-pages` | `pages::list_archived_pages` | Archived pages list (Django uses filters) |
| `api/workspaces/{slug}/projects/{project_id}/pages/{page_id}/move` | `pages::move_page` | Move page across projects |
| `api/workspaces/{slug}/projects/{project_id}/bulk-operation-issues` | `issue_extras2::bulk_operation_issues` | Bulk issue operations endpoint |
| `api/workspaces/{slug}/export-issues/{token}` | `exporter::get_export_status` | Export job status (Django inline) |
| `api/workspaces/{slug}/importers` | `importer::list_all_importers` | Combined GitHub+GitLab importer list |
| `api/workspaces/{slug}/work-items/search` | `search::global_search` | Alias of `/search/` for work-items prefix |
| `api/workspaces/{slug}/issues/search` | `search::global_search` | Legacy search alias |
| `api/workspaces/{slug}/issues/{combined}` | `issue_extras2::get_issue_by_identifier` | Alias for identifier-based lookup |
| `api/api-tokens` | `api_tokens::*` | Extra alias (also at `/users/api-tokens/`) |
| `api/assets/v2/workspaces/{slug}/{entity_id}/bulk` | `assets::bulk_workspace_assets` | Workspace-level bulk asset operation |
| `api/workspaces/{slug}/projects/{project_id}/summary` | `projects::get_project_summary` | Public summary (without admin requirement) |
| `api/workspaces/{slug}/work-items/{issue_id}/activities/{pk}` | `issue_extras::get_issue_activity` | Activity detail in app router |
