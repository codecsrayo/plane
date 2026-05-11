---
type: community
cohesion: 0.03
members: 129
---

# Returns Sign

**Cohesion:** 0.03 - loosely connected
**Members:** 129 nodes

## Members

- [[.drop()]] - code - api_rust/src/jobs/email_notification.rs
- [[.spawn()]] - code - api_rust/tests/common/mod.rs
- [[RedisLockGuard]] - code - api_rust/src/jobs/email_notification.rs
- [[admin_sign_in_missing_fields_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_in_nonexistent_user_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_in_wrong_password_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_out_without_csrf_rejected()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_up_duplicate_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_up_invalid_email_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_up_missing_fields_redirects()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_up_valid_first_time_redirects_success()]] - code - api_rust/tests/instances_admins.rs
- [[admin_sign_up_weak_password_redirects_error()]] - code - api_rust/tests/instances_admins.rs
- [[api_tokens.rs]] - code - api_rust/tests/api_tokens.rs
- [[auth_csrf_and_email_check.rs]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[auth_sign_in_up_out.rs]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[bulk_create_labels_returns_201()]] - code - api_rust/tests/labels.rs
- [[create_api_token_no_label_returns_201()]] - code - api_rust/tests/api_tokens.rs
- [[create_api_token_with_label_returns_201()]] - code - api_rust/tests/api_tokens.rs
- [[create_instance_admin_regular_user_returns_403()]] - code - api_rust/tests/instances_admins.rs
- [[create_instance_admin_unauthenticated_returns_401()]] - code - api_rust/tests/instances_admins.rs
- [[create_label_default_color()]] - code - api_rust/tests/labels.rs
- [[create_label_empty_name_returns_400()]] - code - api_rust/tests/labels.rs
- [[create_label_success()]] - code - api_rust/tests/labels.rs
- [[create_project_duplicate_identifier_returns_422()]] - code - api_rust/tests/projects.rs
- [[create_project_empty_name_returns_422()]] - code - api_rust/tests/projects.rs
- [[create_project_identifier_forbidden_chars_returns_422()]] - code - api_rust/tests/projects.rs
- [[create_project_identifier_too_long_returns_422()]] - code - api_rust/tests/projects.rs
- [[create_project_proptest_forbidden_identifier_chars()]] - code - api_rust/tests/projects.rs
- [[create_project_success()]] - code - api_rust/tests/projects.rs
- [[create_state_duplicate_name_returns_400()]] - code - api_rust/tests/states.rs
- [[create_state_invalid_group_returns_400()]] - code - api_rust/tests/states.rs
- [[create_state_proptest_invalid_groups()]] - code - api_rust/tests/states.rs
- [[create_state_success()]] - code - api_rust/tests/states.rs
- [[create_workspace_duplicate_slug_returns_409()]] - code - api_rust/tests/workspaces.rs
- [[create_workspace_empty_name_returns_400()]] - code - api_rust/tests/workspaces.rs
- [[create_workspace_invalid_slugs_always_400()]] - code - api_rust/tests/workspaces.rs
- [[create_workspace_name_with_url_returns_400()]] - code - api_rust/tests/workspaces.rs
- [[create_workspace_restricted_slug_returns_400()]] - code - api_rust/tests/workspaces.rs
- [[create_workspace_success()]] - code - api_rust/tests/workspaces.rs
- [[csrf_token_endpoint_returns_uuid_and_cookie()]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[csrf_tokens_are_unique_per_request()]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[delete_account_unauthenticated_returns_401()]] - code - api_rust/tests/users_settings.rs
- [[delete_api_token_returns_204()]] - code - api_rust/tests/api_tokens.rs
- [[delete_instance_admin_regular_user_returns_403()]] - code - api_rust/tests/instances_admins.rs
- [[delete_instance_admin_unauthenticated_returns_401()]] - code - api_rust/tests/instances_admins.rs
- [[delete_label_returns_204()]] - code - api_rust/tests/labels.rs
- [[delete_nonexistent_account_returns_4xx()]] - code - api_rust/tests/users_settings.rs
- [[delete_state_returns_204()]] - code - api_rust/tests/states.rs
- [[disable_email_feature_regular_user_returns_403()]] - code - api_rust/tests/instances_admins.rs
- [[disable_email_feature_unauthenticated_returns_401()]] - code - api_rust/tests/instances_admins.rs
- [[email_check_accepts_valid_email_not_registered()]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[email_check_rejects_empty_email()]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[email_check_rejects_malformed_emails()]] - code - api_rust/tests/auth_csrf_and_email_check.rs
- [[generate_email_code_authenticated_returns_2xx_or_400()]] - code - api_rust/tests/users_settings.rs
- [[generate_email_code_unauthenticated_returns_401()]] - code - api_rust/tests/users_settings.rs
- [[get_api_token_by_id_returns_200()]] - code - api_rust/tests/api_tokens.rs
- [[get_api_token_not_found_returns_404()]] - code - api_rust/tests/api_tokens.rs
- [[get_instance_admin_status_regular_user_returns_200()]] - code - api_rust/tests/users_settings.rs
- [[get_instance_admin_status_unauthenticated_returns_401()]] - code - api_rust/tests/users_settings.rs
- [[get_intake_state_returns_200()]] - code - api_rust/tests/states.rs
- [[get_label_by_id_returns_200()]] - code - api_rust/tests/labels.rs
- [[get_label_not_found_returns_404()]] - code - api_rust/tests/labels.rs
- [[get_me_authenticated_returns_user()]] - code - api_rust/tests/users_me.rs
- [[get_me_unauthenticated_returns_401()]] - code - api_rust/tests/users_me.rs
- [[get_project_member_returns_200()]] - code - api_rust/tests/projects.rs
- [[get_project_non_member_returns_404()]] - code - api_rust/tests/projects.rs
- [[get_session_authenticated_returns_user_data()]] - code - api_rust/tests/users_me.rs
- [[get_session_unauthenticated_returns_not_authenticated()]] - code - api_rust/tests/users_me.rs
- [[get_settings_authenticated_returns_200()]] - code - api_rust/tests/users_me.rs
- [[get_settings_authenticated_returns_200()_1]] - code - api_rust/tests/users_settings.rs
- [[get_settings_unauthenticated_returns_401()]] - code - api_rust/tests/users_me.rs
- [[get_settings_unauthenticated_returns_401()_1]] - code - api_rust/tests/users_settings.rs
- [[get_state_by_id_returns_200()]] - code - api_rust/tests/states.rs
- [[get_workspace_member_returns_200()_1]] - code - api_rust/tests/workspaces.rs
- [[get_workspace_non_member_returns_404()]] - code - api_rust/tests/workspaces.rs
- [[instances_admins.rs]] - code - api_rust/tests/instances_admins.rs
- [[labels.rs]] - code - api_rust/tests/labels.rs
- [[list_api_tokens_returns_own_tokens()]] - code - api_rust/tests/api_tokens.rs
- [[list_api_tokens_unauthenticated_returns_401()]] - code - api_rust/tests/api_tokens.rs
- [[list_instance_admins_regular_user_returns_403()]] - code - api_rust/tests/instances_admins.rs
- [[list_instance_admins_unauthenticated_returns_401()]] - code - api_rust/tests/instances_admins.rs
- [[list_labels_empty_returns_200()]] - code - api_rust/tests/labels.rs
- [[list_labels_unauthenticated_returns_401()]] - code - api_rust/tests/labels.rs
- [[list_project_members_returns_list()]] - code - api_rust/tests/projects.rs
- [[list_projects_details_returns_member_projects()]] - code - api_rust/tests/projects.rs
- [[list_projects_empty_workspace()]] - code - api_rust/tests/projects.rs
- [[list_projects_returns_existing_project()]] - code - api_rust/tests/projects.rs
- [[list_projects_unauthenticated_returns_401()]] - code - api_rust/tests/projects.rs
- [[list_states_member_returns_200()]] - code - api_rust/tests/states.rs
- [[list_states_unauthenticated_returns_401()]] - code - api_rust/tests/states.rs
- [[list_timezones_returns_200_with_data()]] - code - api_rust/tests/api_tokens.rs
- [[list_workspace_members_returns_member_list()]] - code - api_rust/tests/workspaces.rs
- [[list_workspaces_empty_for_new_user()]] - code - api_rust/tests/workspaces.rs
- [[list_workspaces_returns_member_workspaces()]] - code - api_rust/tests/workspaces.rs
- [[list_workspaces_unauthenticated_returns_401()]] - code - api_rust/tests/workspaces.rs
- [[location_has_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[patch_me_display_name_too_long_returns_400()]] - code - api_rust/tests/users_me.rs
- [[patch_me_empty_display_name_returns_400()]] - code - api_rust/tests/users_me.rs
- [[patch_me_updates_display_name()]] - code - api_rust/tests/users_me.rs
- [[patch_me_valid_display_name_always_200()]] - code - api_rust/tests/users_me.rs
- [[patch_project_admin_updates_name()]] - code - api_rust/tests/projects.rs
- [[patch_workspace_admin_can_update_name()]] - code - api_rust/tests/workspaces.rs
- [[posthog.rs]] - code - api_rust/src/utils/posthog.rs
- [[projects.rs]] - code - api_rust/tests/projects.rs
- [[sign_in_invalid_email_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_in_missing_credentials_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_in_unknown_user_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_in_valid_credentials_redirects_to_success()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_in_wrong_password_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_out_without_session_returns_401()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_up_duplicate_email_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_up_rejects_invalid_emails()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_up_with_valid_credentials_redirects_to_success()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_up_without_email_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[sign_up_without_password_redirects_with_error()]] - code - api_rust/tests/auth_sign_in_up_out.rs
- [[slug_check_available_returns_true()]] - code - api_rust/tests/workspaces.rs
- [[slug_check_restricted_returns_false()]] - code - api_rust/tests/workspaces.rs
- [[slug_check_taken_returns_false()]] - code - api_rust/tests/workspaces.rs
- [[states.rs]] - code - api_rust/tests/states.rs
- [[track_event()_1]] - code - api_rust/src/utils/posthog.rs
- [[update_api_token_returns_200()]] - code - api_rust/tests/api_tokens.rs
- [[update_label_returns_200()]] - code - api_rust/tests/labels.rs
- [[update_state_returns_200()]] - code - api_rust/tests/states.rs
- [[update_user_email_unauthenticated_returns_401()]] - code - api_rust/tests/users_settings.rs
- [[update_user_email_with_invalid_code_returns_4xx()]] - code - api_rust/tests/users_settings.rs
- [[users_api_tokens_alias_returns_200()]] - code - api_rust/tests/api_tokens.rs
- [[users_me.rs]] - code - api_rust/tests/users_me.rs
- [[users_settings.rs]] - code - api_rust/tests/users_settings.rs
- [[workspaces.rs]] - code - api_rust/tests/workspaces.rs

## Live Query (requires Dataview plugin)

```dataview
TABLE source_file, type FROM #community/Returns_Sign
SORT file.name ASC
```

## Connections to other communities

- 14 edges to [[_COMMUNITY_Asset Issue]]
- 13 edges to [[_COMMUNITY_Returns Without]]
- 12 edges to [[_COMMUNITY_Password Email]]
- 11 edges to [[_COMMUNITY_Returns Graph]]
- 11 edges to [[_COMMUNITY_Magic Sign]]
- 11 edges to [[_COMMUNITY_Space Sign]]
- 10 edges to [[_COMMUNITY_Instance Returns]]
- 7 edges to [[_COMMUNITY_Returns Notification]]
- 7 edges to [[_COMMUNITY_Returns Analytics]]
- 7 edges to [[_COMMUNITY_Password Returns]]
- 6 edges to [[_COMMUNITY_Returns Unauthenticated]]
- 3 edges to [[_COMMUNITY_Returns State]]
- 3 edges to [[_COMMUNITY_Returns Invitation]]
- 3 edges to [[_COMMUNITY_User Returns]]
- 3 edges to [[_COMMUNITY_Test Binary]]
- 2 edges to [[_COMMUNITY_Search Returns]]
- 1 edge to [[_COMMUNITY_Redis Manager]]
- 1 edge to [[_COMMUNITY_Estimate Point]]
- 1 edge to [[_COMMUNITY_Returns Project]]
- 1 edge to [[_COMMUNITY_Returns Asset]]
- 1 edge to [[_COMMUNITY_Cycle Returns]]
- 1 edge to [[_COMMUNITY_Returns Issue]]
- 1 edge to [[_COMMUNITY_Returns Project]]
- 1 edge to [[_COMMUNITY_Returns Asset]]
- 1 edge to [[_COMMUNITY_Returns Link]]
- 1 edge to [[_COMMUNITY_Returns Modules]]
- 1 edge to [[_COMMUNITY_Workspace Returns]]
- 1 edge to [[_COMMUNITY_Returns Issue]]
- 1 edge to [[_COMMUNITY_Project Returns]]
- 1 edge to [[_COMMUNITY_Returns Issue]]
- 1 edge to [[_COMMUNITY_Returns Issue]]
- 1 edge to [[_COMMUNITY_Returns Legacy]]
- 1 edge to [[_COMMUNITY_Returns Cycle]]
- 1 edge to [[_COMMUNITY_Returns Intake]]
- 1 edge to [[_COMMUNITY_Returns Unauthenticated]]
- 1 edge to [[_COMMUNITY_Returns Nonexistent]]
- 1 edge to [[_COMMUNITY_Work Returns]]
- 1 edge to [[_COMMUNITY_Returns Workspace]]
- 1 edge to [[_COMMUNITY_Workspace Returns]]
- 1 edge to [[_COMMUNITY_Returns Issues]]
- 1 edge to [[_COMMUNITY_Importer Returns]]
- 1 edge to [[_COMMUNITY_Returns Nonexistent]]
- 1 edge to [[_COMMUNITY_Project Summary]]
- 1 edge to [[_COMMUNITY_Returns Estimate]]
- 1 edge to [[_COMMUNITY_Returns Webhook]]
- 1 edge to [[_COMMUNITY_Returns Workspace]]
- 1 edge to [[_COMMUNITY_Returns Pages]]
- 1 edge to [[_COMMUNITY_Magic Sign]]

## Top bridge nodes

- [[.spawn()]] - degree 262, connects to 47 communities
- [[email_check_rejects_malformed_emails()]] - degree 3, connects to 1 community
- [[sign_up_rejects_invalid_emails()]] - degree 3, connects to 1 community
- [[create_project_proptest_forbidden_identifier_chars()]] - degree 3, connects to 1 community
- [[create_state_proptest_invalid_groups()]] - degree 3, connects to 1 community
