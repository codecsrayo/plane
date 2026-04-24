# Django API – Árbol de archivos (`apps/api`)

```tree
apps/api/
├── bin/
│   ├── docker-entrypoint-api-local.sh
│   ├── docker-entrypoint-api.sh
│   ├── docker-entrypoint-beat.sh
│   ├── docker-entrypoint-migrator.sh
│   └── docker-entrypoint-worker.sh
├── plane/
│   ├── analytics/
│   │   ├── __init__.py
│   │   └── apps.py
│   ├── api/
│   │   ├── middleware/
│   │   │   ├── __init__.py
│   │   │   └── api_authentication.py
│   │   ├── serializers/
│   │   │   ├── __init__.py
│   │   │   ├── asset.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── estimate.py
│   │   │   ├── intake.py
│   │   │   ├── invite.py
│   │   │   ├── issue.py
│   │   │   ├── member.py
│   │   │   ├── module.py
│   │   │   ├── project.py
│   │   │   ├── state.py
│   │   │   ├── sticky.py
│   │   │   ├── user.py
│   │   │   └── workspace.py
│   │   ├── urls/
│   │   │   ├── __init__.py
│   │   │   ├── asset.py
│   │   │   ├── cycle.py
│   │   │   ├── estimate.py
│   │   │   ├── intake.py
│   │   │   ├── invite.py
│   │   │   ├── label.py
│   │   │   ├── member.py
│   │   │   ├── module.py
│   │   │   ├── project.py
│   │   │   ├── schema.py
│   │   │   ├── state.py
│   │   │   ├── sticky.py
│   │   │   ├── user.py
│   │   │   └── work_item.py
│   │   ├── views/
│   │   │   ├── __init__.py
│   │   │   ├── asset.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── estimate.py
│   │   │   ├── intake.py
│   │   │   ├── invite.py
│   │   │   ├── issue.py
│   │   │   ├── member.py
│   │   │   ├── module.py
│   │   │   ├── project.py
│   │   │   ├── state.py
│   │   │   ├── sticky.py
│   │   │   └── user.py
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── middleware
│   │   ├── rate_limit.py
│   │   ├── serializers
│   │   ├── urls
│   │   └── views
│   ├── app/
│   │   ├── middleware/
│   │   │   ├── __init__.py
│   │   │   └── api_authentication.py
│   │   ├── permissions/
│   │   │   ├── __init__.py
│   │   │   ├── base.py
│   │   │   ├── page.py
│   │   │   ├── project.py
│   │   │   └── workspace.py
│   │   ├── serializers/
│   │   │   ├── __init__.py
│   │   │   ├── analytic.py
│   │   │   ├── api.py
│   │   │   ├── asset.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── draft.py
│   │   │   ├── estimate.py
│   │   │   ├── exporter.py
│   │   │   ├── favorite.py
│   │   │   ├── importer.py
│   │   │   ├── intake.py
│   │   │   ├── integration.py
│   │   │   ├── issue.py
│   │   │   ├── module.py
│   │   │   ├── notification.py
│   │   │   ├── page.py
│   │   │   ├── project.py
│   │   │   ├── state.py
│   │   │   ├── user.py
│   │   │   ├── view.py
│   │   │   ├── webhook.py
│   │   │   └── workspace.py
│   │   ├── urls/
│   │   │   ├── __init__.py
│   │   │   ├── analytic.py
│   │   │   ├── api.py
│   │   │   ├── asset.py
│   │   │   ├── cycle.py
│   │   │   ├── estimate.py
│   │   │   ├── exporter.py
│   │   │   ├── external.py
│   │   │   ├── importer.py
│   │   │   ├── intake.py
│   │   │   ├── integration.py
│   │   │   ├── issue.py
│   │   │   ├── module.py
│   │   │   ├── notification.py
│   │   │   ├── page.py
│   │   │   ├── project.py
│   │   │   ├── search.py
│   │   │   ├── state.py
│   │   │   ├── timezone.py
│   │   │   ├── user.py
│   │   │   ├── views.py
│   │   │   ├── webhook.py
│   │   │   └── workspace.py
│   │   ├── views/
│   │   │   ├── analytic/
│   │   │   │   ├── advance.py
│   │   │   │   ├── base.py
│   │   │   │   └── project_analytics.py
│   │   │   ├── asset/
│   │   │   │   ├── base.py
│   │   │   │   └── v2.py
│   │   │   ├── cycle/
│   │   │   │   ├── archive.py
│   │   │   │   ├── base.py
│   │   │   │   └── issue.py
│   │   │   ├── estimate/
│   │   │   │   └── base.py
│   │   │   ├── exporter/
│   │   │   │   └── base.py
│   │   │   ├── external/
│   │   │   │   ├── base.py
│   │   │   │   └── sync.py
│   │   │   ├── importer/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── github.py
│   │   │   │   └── gitlab.py
│   │   │   ├── intake/
│   │   │   │   └── base.py
│   │   │   ├── integration/
│   │   │   │   └── base.py
│   │   │   ├── issue/
│   │   │   │   ├── activity.py
│   │   │   │   ├── archive.py
│   │   │   │   ├── attachment.py
│   │   │   │   ├── base.py
│   │   │   │   ├── comment.py
│   │   │   │   ├── label.py
│   │   │   │   ├── link.py
│   │   │   │   ├── reaction.py
│   │   │   │   ├── relation.py
│   │   │   │   ├── sub_issue.py
│   │   │   │   ├── subscriber.py
│   │   │   │   └── version.py
│   │   │   ├── module/
│   │   │   │   ├── archive.py
│   │   │   │   ├── base.py
│   │   │   │   └── issue.py
│   │   │   ├── notification/
│   │   │   │   └── base.py
│   │   │   ├── page/
│   │   │   │   ├── base.py
│   │   │   │   └── version.py
│   │   │   ├── project/
│   │   │   │   ├── base.py
│   │   │   │   ├── invite.py
│   │   │   │   └── member.py
│   │   │   ├── search/
│   │   │   │   ├── base.py
│   │   │   │   └── issue.py
│   │   │   ├── state/
│   │   │   │   └── base.py
│   │   │   ├── timezone/
│   │   │   │   └── base.py
│   │   │   ├── user/
│   │   │   │   └── base.py
│   │   │   ├── view/
│   │   │   │   └── base.py
│   │   │   ├── webhook/
│   │   │   │   └── base.py
│   │   │   ├── workspace/
│   │   │   │   ├── base.py
│   │   │   │   ├── cycle.py
│   │   │   │   ├── draft.py
│   │   │   │   ├── estimate.py
│   │   │   │   ├── favorite.py
│   │   │   │   ├── home.py
│   │   │   │   ├── invite.py
│   │   │   │   ├── label.py
│   │   │   │   ├── member.py
│   │   │   │   ├── module.py
│   │   │   │   ├── quick_link.py
│   │   │   │   ├── recent_visit.py
│   │   │   │   ├── state.py
│   │   │   │   ├── sticky.py
│   │   │   │   ├── user.py
│   │   │   │   └── user_preference.py
│   │   │   ├── __init__.py
│   │   │   ├── analytic
│   │   │   ├── api.py
│   │   │   ├── asset
│   │   │   ├── base.py
│   │   │   ├── cycle
│   │   │   ├── error_404.py
│   │   │   ├── estimate
│   │   │   ├── exporter
│   │   │   ├── external
│   │   │   ├── importer
│   │   │   ├── intake
│   │   │   ├── integration
│   │   │   ├── issue
│   │   │   ├── module
│   │   │   ├── notification
│   │   │   ├── page
│   │   │   ├── project
│   │   │   ├── search
│   │   │   ├── state
│   │   │   ├── timezone
│   │   │   ├── user
│   │   │   ├── view
│   │   │   ├── webhook
│   │   │   └── workspace
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── middleware
│   │   ├── permissions
│   │   ├── serializers
│   │   ├── urls
│   │   └── views
│   ├── authentication/
│   │   ├── adapter/
│   │   │   ├── __init__.py
│   │   │   ├── base.py
│   │   │   ├── credential.py
│   │   │   ├── error.py
│   │   │   ├── exception.py
│   │   │   └── oauth.py
│   │   ├── middleware/
│   │   │   ├── __init__.py
│   │   │   └── session.py
│   │   ├── provider/
│   │   │   ├── credentials/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── email.py
│   │   │   │   └── magic_code.py
│   │   │   ├── oauth/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── gitea.py
│   │   │   │   ├── github.py
│   │   │   │   ├── gitlab.py
│   │   │   │   └── google.py
│   │   │   ├── __init__.py
│   │   │   ├── credentials
│   │   │   └── oauth
│   │   ├── utils/
│   │   │   ├── host.py
│   │   │   ├── login.py
│   │   │   ├── redirection_path.py
│   │   │   ├── user_auth_workflow.py
│   │   │   └── workspace_project_join.py
│   │   ├── views/
│   │   │   ├── app/
│   │   │   │   ├── check.py
│   │   │   │   ├── email.py
│   │   │   │   ├── gitea.py
│   │   │   │   ├── github.py
│   │   │   │   ├── gitlab.py
│   │   │   │   ├── google.py
│   │   │   │   ├── magic.py
│   │   │   │   ├── password_management.py
│   │   │   │   └── signout.py
│   │   │   ├── space/
│   │   │   │   ├── check.py
│   │   │   │   ├── email.py
│   │   │   │   ├── gitea.py
│   │   │   │   ├── github.py
│   │   │   │   ├── gitlab.py
│   │   │   │   ├── google.py
│   │   │   │   ├── magic.py
│   │   │   │   ├── password_management.py
│   │   │   │   └── signout.py
│   │   │   ├── __init__.py
│   │   │   ├── app
│   │   │   ├── common.py
│   │   │   └── space
│   │   ├── __init__.py
│   │   ├── adapter
│   │   ├── apps.py
│   │   ├── middleware
│   │   ├── provider
│   │   ├── rate_limit.py
│   │   ├── session.py
│   │   ├── urls.py
│   │   ├── utils
│   │   └── views
│   ├── bgtasks/
│   │   ├── __init__.py
│   │   ├── analytic_plot_export.py
│   │   ├── apps.py
│   │   ├── cleanup_task.py
│   │   ├── copy_s3_object.py
│   │   ├── deletion_task.py
│   │   ├── dummy_data_task.py
│   │   ├── email_notification_task.py
│   │   ├── event_tracking_task.py
│   │   ├── export_task.py
│   │   ├── exporter_expired_task.py
│   │   ├── file_asset_task.py
│   │   ├── forgot_password_task.py
│   │   ├── github_sync_task.py
│   │   ├── importer_task.py
│   │   ├── issue_activities_task.py
│   │   ├── issue_automation_task.py
│   │   ├── issue_description_version_sync.py
│   │   ├── issue_description_version_task.py
│   │   ├── issue_version_sync.py
│   │   ├── logger_task.py
│   │   ├── magic_link_code_task.py
│   │   ├── notification_task.py
│   │   ├── page_transaction_task.py
│   │   ├── page_version_task.py
│   │   ├── project_add_user_email_task.py
│   │   ├── project_invitation_task.py
│   │   ├── recent_visited_task.py
│   │   ├── storage_metadata_task.py
│   │   ├── sync_task.py
│   │   ├── user_activation_email_task.py
│   │   ├── user_deactivation_email_task.py
│   │   ├── user_email_update_task.py
│   │   ├── webhook_task.py
│   │   ├── work_item_link_task.py
│   │   ├── workspace_invitation_task.py
│   │   └── workspace_seed_task.py
│   ├── db/
│   │   ├── management/
│   │   │   ├── commands/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── activate_user.py
│   │   │   │   ├── clear_cache.py
│   │   │   │   ├── copy_issue_comment_to_description.py
│   │   │   │   ├── create_bucket.py
│   │   │   │   ├── create_dummy_data.py
│   │   │   │   ├── create_instance_admin.py
│   │   │   │   ├── create_project_member.py
│   │   │   │   ├── fix_duplicate_sequences.py
│   │   │   │   ├── reset_password.py
│   │   │   │   ├── sync_issue_description_version.py
│   │   │   │   ├── sync_issue_version.py
│   │   │   │   ├── test_email.py
│   │   │   │   ├── update_bucket.py
│   │   │   │   ├── update_deleted_workspace_slug.py
│   │   │   │   ├── wait_for_db.py
│   │   │   │   └── wait_for_migrations.py
│   │   │   ├── __init__.py
│   │   │   └── commands
│   │   ├── migrations/
│   │   │   ├── 0001_initial.py
│   │   │   ├── 0002_auto_20221104_2239.py
│   │   │   ├── 0003_auto_20221109_2320.py
│   │   │   ├── 0004_alter_state_sequence.py
│   │   │   ├── 0005_auto_20221114_2127.py
│   │   │   ├── 0006_alter_cycle_status.py
│   │   │   ├── 0007_label_parent.py
│   │   │   ├── 0008_label_colour.py
│   │   │   ├── 0009_auto_20221208_0310.py
│   │   │   ├── 0010_auto_20221213_0037.py
│   │   │   ├── 0011_auto_20221222_2357.py
│   │   │   ├── 0012_auto_20230104_0117.py
│   │   │   ├── 0013_auto_20230107_0041.py
│   │   │   ├── 0014_alter_workspacememberinvite_unique_together.py
│   │   │   ├── 0015_auto_20230107_1636.py
│   │   │   ├── 0016_auto_20230107_1735.py
│   │   │   ├── 0017_alter_workspace_unique_together.py
│   │   │   ├── 0018_auto_20230130_0119.py
│   │   │   ├── 0019_auto_20230131_0049.py
│   │   │   ├── 0020_auto_20230214_0118.py
│   │   │   ├── 0021_auto_20230223_0104.py
│   │   │   ├── 0022_auto_20230307_0304.py
│   │   │   ├── 0023_auto_20230316_0040.py
│   │   │   ├── 0024_auto_20230322_0138.py
│   │   │   ├── 0025_auto_20230331_0203.py
│   │   │   ├── 0026_alter_projectmember_view_props.py
│   │   │   ├── 0027_auto_20230409_0312.py
│   │   │   ├── 0028_auto_20230414_1703.py
│   │   │   ├── 0029_auto_20230502_0126.py
│   │   │   ├── 0030_alter_estimatepoint_unique_together.py
│   │   │   ├── 0031_analyticview.py
│   │   │   ├── 0032_auto_20230520_2015.py
│   │   │   ├── 0033_auto_20230618_2125.py
│   │   │   ├── 0034_auto_20230628_1046.py
│   │   │   ├── 0035_auto_20230704_2225.py
│   │   │   ├── 0036_alter_workspace_organization_size.py
│   │   │   ├── 0037_issue_archived_at_project_archive_in_and_more.py
│   │   │   ├── 0038_auto_20230720_1505.py
│   │   │   ├── 0039_auto_20230723_2203.py
│   │   │   ├── 0040_projectmember_preferences_user_cover_image_and_more.py
│   │   │   ├── 0041_cycle_sort_order_issuecomment_access_and_more.py
│   │   │   ├── 0042_alter_analyticview_created_by_and_more.py
│   │   │   ├── 0043_alter_analyticview_created_by_and_more.py
│   │   │   ├── 0044_auto_20230913_0709.py
│   │   │   ├── 0045_issueactivity_epoch_workspacemember_issue_props_and_more.py
│   │   │   ├── 0046_label_sort_order_alter_analyticview_created_by_and_more.py
│   │   │   ├── 0047_webhook_apitoken_description_apitoken_expired_at_and_more.py
│   │   │   ├── 0048_auto_20231116_0713.py
│   │   │   ├── 0049_auto_20231116_0713.py
│   │   │   ├── 0050_user_use_case_alter_workspace_organization_size.py
│   │   │   ├── 0051_cycle_external_id_cycle_external_source_and_more.py
│   │   │   ├── 0052_auto_20231220_1141.py
│   │   │   ├── 0053_auto_20240102_1315.py
│   │   │   ├── 0054_dashboard_widget_dashboardwidget.py
│   │   │   ├── 0055_auto_20240108_0648.py
│   │   │   ├── 0056_usernotificationpreference_emailnotificationlog.py
│   │   │   ├── 0057_auto_20240122_0901.py
│   │   │   ├── 0058_alter_moduleissue_issue_and_more.py
│   │   │   ├── 0059_auto_20240208_0957.py
│   │   │   ├── 0060_cycle_progress_snapshot.py
│   │   │   ├── 0061_project_logo_props.py
│   │   │   ├── 0062_cycle_archived_at_module_archived_at_and_more.py
│   │   │   ├── 0063_state_is_triage_alter_state_group.py
│   │   │   ├── 0064_auto_20240409_1134.py
│   │   │   ├── 0065_auto_20240415_0937.py
│   │   │   ├── 0066_account_id_token_cycle_logo_props_module_logo_props.py
│   │   │   ├── 0067_issue_estimate.py
│   │   │   ├── 0068_remove_pagelabel_project_remove_pagelog_project_and_more.py
│   │   │   ├── 0069_alter_account_provider_and_more.py
│   │   │   ├── 0070_apitoken_is_service_exporterhistory_filters_and_more.py
│   │   │   ├── 0071_rename_issueproperty_issueuserproperty_and_more.py
│   │   │   ├── 0072_issueattachment_external_id_and_more.py
│   │   │   ├── 0073_alter_commentreaction_unique_together_and_more.py
│   │   │   ├── 0074_deploy_board_and_project_issues.py
│   │   │   ├── 0075_alter_fileasset_asset.py
│   │   │   ├── 0076_alter_projectmember_role_and_more.py
│   │   │   ├── 0077_draftissue_cycle_user_timezone_project_user_timezone_and_more.py
│   │   │   ├── 0078_fileasset_comment_fileasset_entity_type_and_more.py
│   │   │   ├── 0079_auto_20241009_0619.py
│   │   │   ├── 0080_fileasset_draft_issue_alter_fileasset_entity_type.py
│   │   │   ├── 0081_remove_globalview_created_by_and_more.py
│   │   │   ├── 0082_alter_issue_managers_alter_cycleissue_issue_and_more.py
│   │   │   ├── 0083_device_workspace_timezone_and_more.py
│   │   │   ├── 0084_remove_label_label_unique_name_project_when_deleted_at_null_and_more.py
│   │   │   ├── 0085_intake_intakeissue_remove_inboxissue_created_by_and_more.py
│   │   │   ├── 0086_issueversion_alter_teampage_unique_together_and_more.py
│   │   │   ├── 0087_remove_issueversion_description_and_more.py
│   │   │   ├── 0088_sticky_sort_order_workspaceuserlink.py
│   │   │   ├── 0089_workspacehomepreference_and_more.py
│   │   │   ├── 0090_rename_dashboard_deprecateddashboard_and_more.py
│   │   │   ├── 0091_issuecomment_edited_at_and_more.py
│   │   │   ├── 0092_alter_deprecateddashboardwidget_unique_together_and_more.py
│   │   │   ├── 0093_page_moved_to_page_page_moved_to_project_and_more.py
│   │   │   ├── 0094_auto_20250425_0902.py
│   │   │   ├── 0095_page_external_id_page_external_source.py
│   │   │   ├── 0096_user_is_email_valid_user_masked_at.py
│   │   │   ├── 0097_project_external_id_project_external_source.py
│   │   │   ├── 0098_profile_is_app_rail_docked_and_more.py
│   │   │   ├── 0099_profile_background_color_profile_goals_and_more.py
│   │   │   ├── 0100_profile_has_marketing_email_consent_and_more.py
│   │   │   ├── 0101_description_descriptionversion.py
│   │   │   ├── 0102_page_sort_order_pagelog_entity_type_and_more.py
│   │   │   ├── 0103_fileasset_asset_entity_type_idx_and_more.py
│   │   │   ├── 0104_cycleuserproperties_rich_filters_and_more.py
│   │   │   ├── 0105_alter_project_cycle_view_and_more.py
│   │   │   ├── 0106_auto_20250912_0845.py
│   │   │   ├── 0107_migrate_filters_to_rich_filters.py
│   │   │   ├── 0108_alter_issueactivity_issue_comment.py
│   │   │   ├── 0109_issuecomment_description_and_parent_id.py
│   │   │   ├── 0110_workspaceuserproperties_navigation_control_preference_and_more.py
│   │   │   ├── 0111_notification_notif_receiver_status_idx_and_more.py
│   │   │   ├── 0112_auto_20251124_0603.py
│   │   │   ├── 0113_webhook_version.py
│   │   │   ├── 0114_projectuserproperty_delete_issueuserproperty_and_more.py
│   │   │   ├── 0115_auto_20260105_1406.py
│   │   │   ├── 0116_workspacemember_explored_features_and_more.py
│   │   │   ├── 0117_rename_description_draftissue_description_json_and_more.py
│   │   │   ├── 0118_remove_workspaceuserproperties_product_tour_and_more.py
│   │   │   ├── 0119_alter_estimatepoint_key.py
│   │   │   ├── 0120_issueview_archived_at.py
│   │   │   ├── 0121_alter_estimate_type.py
│   │   │   ├── 0122_add_github_gitlab_integrations.py
│   │   │   ├── 0123_add_slack_integration.py
│   │   │   ├── 0124_githubprstatemapping_usergithubconnection.py
│   │   │   ├── 0125_githubprstatemapping_extend_states.py
│   │   │   ├── 0126_gitlab_sync_models.py
│   │   │   └── __init__.py
│   │   ├── models/
│   │   │   ├── integration/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── base.py
│   │   │   │   ├── github.py
│   │   │   │   ├── github_pr_state.py
│   │   │   │   ├── gitlab.py
│   │   │   │   ├── signals.py
│   │   │   │   ├── slack.py
│   │   │   │   └── user_github_connection.py
│   │   │   ├── __init__.py
│   │   │   ├── analytic.py
│   │   │   ├── api.py
│   │   │   ├── asset.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── deploy_board.py
│   │   │   ├── description.py
│   │   │   ├── device.py
│   │   │   ├── draft.py
│   │   │   ├── estimate.py
│   │   │   ├── exporter.py
│   │   │   ├── favorite.py
│   │   │   ├── importer.py
│   │   │   ├── intake.py
│   │   │   ├── integration
│   │   │   ├── issue.py
│   │   │   ├── issue_type.py
│   │   │   ├── label.py
│   │   │   ├── module.py
│   │   │   ├── notification.py
│   │   │   ├── page.py
│   │   │   ├── project.py
│   │   │   ├── recent_visit.py
│   │   │   ├── session.py
│   │   │   ├── social_connection.py
│   │   │   ├── state.py
│   │   │   ├── sticky.py
│   │   │   ├── user.py
│   │   │   ├── view.py
│   │   │   ├── webhook.py
│   │   │   └── workspace.py
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── management
│   │   ├── migrations
│   │   ├── mixins.py
│   │   └── models
│   ├── license/
│   │   ├── api/
│   │   │   ├── permissions/
│   │   │   │   ├── __init__.py
│   │   │   │   └── instance.py
│   │   │   ├── serializers/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── admin.py
│   │   │   │   ├── base.py
│   │   │   │   ├── configuration.py
│   │   │   │   ├── instance.py
│   │   │   │   ├── user.py
│   │   │   │   └── workspace.py
│   │   │   ├── views/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── admin.py
│   │   │   │   ├── base.py
│   │   │   │   ├── configuration.py
│   │   │   │   ├── instance.py
│   │   │   │   └── workspace.py
│   │   │   ├── __init__.py
│   │   │   ├── permissions
│   │   │   ├── serializers
│   │   │   └── views
│   │   ├── bgtasks/
│   │   │   ├── __init__.py
│   │   │   └── tracer.py
│   │   ├── management/
│   │   │   ├── commands/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── configure_instance.py
│   │   │   │   └── register_instance.py
│   │   │   ├── __init__.py
│   │   │   └── commands
│   │   ├── migrations/
│   │   │   ├── 0001_initial.py
│   │   │   ├── 0002_rename_version_instance_current_version_and_more.py
│   │   │   ├── 0003_alter_changelog_title_alter_changelog_version_and_more.py
│   │   │   ├── 0004_changelog_deleted_at_instance_deleted_at_and_more.py
│   │   │   ├── 0005_rename_product_instance_edition_and_more.py
│   │   │   ├── 0006_instance_is_current_version_deprecated.py
│   │   │   └── __init__.py
│   │   ├── models/
│   │   │   ├── __init__.py
│   │   │   └── instance.py
│   │   ├── utils/
│   │   │   ├── __init__.py
│   │   │   ├── encryption.py
│   │   │   └── instance_value.py
│   │   ├── __init__.py
│   │   ├── api
│   │   ├── apps.py
│   │   ├── bgtasks
│   │   ├── management
│   │   ├── migrations
│   │   ├── models
│   │   ├── urls.py
│   │   └── utils
│   ├── middleware/
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── db_routing.py
│   │   ├── logger.py
│   │   └── request_body_size.py
│   ├── seeds/
│   │   ├── data/
│   │   │   ├── cycles.json
│   │   │   ├── issues.json
│   │   │   ├── labels.json
│   │   │   ├── modules.json
│   │   │   ├── pages.json
│   │   │   ├── projects.json
│   │   │   ├── states.json
│   │   │   └── views.json
│   │   └── data
│   ├── settings/
│   │   ├── __init__.py
│   │   ├── common.py
│   │   ├── local.py
│   │   ├── mongo.py
│   │   ├── openapi.py
│   │   ├── production.py
│   │   ├── redis.py
│   │   ├── storage.py
│   │   └── test.py
│   ├── space/
│   │   ├── serializer/
│   │   │   ├── __init__.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── intake.py
│   │   │   ├── issue.py
│   │   │   ├── module.py
│   │   │   ├── project.py
│   │   │   ├── state.py
│   │   │   ├── user.py
│   │   │   └── workspace.py
│   │   ├── urls/
│   │   │   ├── __init__.py
│   │   │   ├── asset.py
│   │   │   ├── intake.py
│   │   │   ├── issue.py
│   │   │   └── project.py
│   │   ├── utils/
│   │   │   └── grouper.py
│   │   ├── views/
│   │   │   ├── __init__.py
│   │   │   ├── asset.py
│   │   │   ├── base.py
│   │   │   ├── cycle.py
│   │   │   ├── intake.py
│   │   │   ├── issue.py
│   │   │   ├── label.py
│   │   │   ├── meta.py
│   │   │   ├── module.py
│   │   │   ├── project.py
│   │   │   └── state.py
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── serializer
│   │   ├── urls
│   │   ├── utils
│   │   └── views
│   ├── static/
│   │   ├── css/
│   │   │   └── style.css
│   │   ├── js/
│   │   │   └── script.js
│   │   ├── logos/
│   │   │   ├── Logo.png
│   │   │   ├── github_32px.png
│   │   │   ├── linkedin_32px.png
│   │   │   ├── twitter_32px.png
│   │   │   └── website_32px.png
│   │   ├── css
│   │   ├── humans.txt
│   │   ├── js
│   │   └── logos
│   ├── tests/
│   │   ├── contract/
│   │   │   ├── api/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_cycles.py
│   │   │   │   └── test_labels.py
│   │   │   ├── app/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_api_token.py
│   │   │   │   ├── test_authentication.py
│   │   │   │   ├── test_github_app_callback.py
│   │   │   │   ├── test_github_user_connection.py
│   │   │   │   ├── test_page_app.py
│   │   │   │   ├── test_project_app.py
│   │   │   │   └── test_workspace_app.py
│   │   │   ├── __init__.py
│   │   │   ├── api
│   │   │   └── app
│   │   ├── smoke/
│   │   │   ├── __init__.py
│   │   │   └── test_auth_smoke.py
│   │   ├── unit/
│   │   │   ├── bg_tasks/
│   │   │   │   └── test_copy_s3_objects.py
│   │   │   ├── license/
│   │   │   │   └── test_instance_endpoint.py
│   │   │   ├── middleware/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_db_routing.py
│   │   │   │   └── test_session_middleware.py
│   │   │   ├── models/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_integration_signals.py
│   │   │   │   ├── test_issue_comment_modal.py
│   │   │   │   └── test_workspace_model.py
│   │   │   ├── serializers/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_issue_recent_visit.py
│   │   │   │   ├── test_label.py
│   │   │   │   └── test_workspace.py
│   │   │   ├── settings/
│   │   │   │   ├── __init__.py
│   │   │   │   └── test_storage.py
│   │   │   ├── utils/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── test_url.py
│   │   │   │   └── test_uuid.py
│   │   │   ├── __init__.py
│   │   │   ├── bg_tasks
│   │   │   ├── license
│   │   │   ├── middleware
│   │   │   ├── models
│   │   │   ├── serializers
│   │   │   ├── settings
│   │   │   ├── test_importer_api.py
│   │   │   └── utils
│   │   ├── README.md
│   │   ├── TESTING_GUIDE.md
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── conftest.py
│   │   ├── conftest_external.py
│   │   ├── contract
│   │   ├── factories.py
│   │   ├── smoke
│   │   └── unit
│   ├── throttles/
│   │   └── asset.py
│   ├── utils/
│   │   ├── core/
│   │   │   ├── mixins/
│   │   │   │   ├── __init__.py
│   │   │   │   └── view.py
│   │   │   ├── __init__.py
│   │   │   ├── dbrouters.py
│   │   │   ├── mixins
│   │   │   └── request_scope.py
│   │   ├── exporters/
│   │   │   ├── schemas/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── base.py
│   │   │   │   └── issue.py
│   │   │   ├── README.md
│   │   │   ├── __init__.py
│   │   │   ├── exporter.py
│   │   │   ├── formatters.py
│   │   │   └── schemas
│   │   ├── filters/
│   │   │   ├── __init__.py
│   │   │   ├── converters.py
│   │   │   ├── filter_backend.py
│   │   │   ├── filter_migrations.py
│   │   │   └── filterset.py
│   │   ├── instance_config_variables/
│   │   │   ├── __init__.py
│   │   │   ├── core.py
│   │   │   └── extended.py
│   │   ├── openapi/
│   │   │   ├── README.md
│   │   │   ├── __init__.py
│   │   │   ├── auth.py
│   │   │   ├── decorators.py
│   │   │   ├── examples.py
│   │   │   ├── hooks.py
│   │   │   ├── parameters.py
│   │   │   └── responses.py
│   │   ├── permissions/
│   │   │   ├── __init__.py
│   │   │   ├── base.py
│   │   │   ├── page.py
│   │   │   ├── project.py
│   │   │   └── workspace.py
│   │   ├── porters/
│   │   │   ├── serializers/
│   │   │   │   ├── __init__.py
│   │   │   │   └── issue.py
│   │   │   ├── __init__.py
│   │   │   ├── exporter.py
│   │   │   ├── formatters.py
│   │   │   └── serializers
│   │   ├── __init__.py
│   │   ├── analytics_events.py
│   │   ├── analytics_plot.py
│   │   ├── build_chart.py
│   │   ├── cache.py
│   │   ├── color.py
│   │   ├── constants.py
│   │   ├── content_validator.py
│   │   ├── core
│   │   ├── csv_utils.py
│   │   ├── cycle_transfer_issues.py
│   │   ├── date_utils.py
│   │   ├── email.py
│   │   ├── error_codes.py
│   │   ├── exception_logger.py
│   │   ├── exporters
│   │   ├── filters
│   │   ├── github_app.py
│   │   ├── global_paginator.py
│   │   ├── grouper.py
│   │   ├── host.py
│   │   ├── html_processor.py
│   │   ├── imports.py
│   │   ├── instance_config_variables
│   │   ├── ip_address.py
│   │   ├── issue_filters.py
│   │   ├── issue_relation_mapper.py
│   │   ├── issue_search.py
│   │   ├── logging.py
│   │   ├── markdown.py
│   │   ├── openapi
│   │   ├── order_queryset.py
│   │   ├── paginator.py
│   │   ├── path_validator.py
│   │   ├── permissions
│   │   ├── porters
│   │   ├── telemetry.py
│   │   ├── timezone_converter.py
│   │   ├── url.py
│   │   └── uuid.py
│   ├── web/
│   │   ├── __init__.py
│   │   ├── apps.py
│   │   ├── urls.py
│   │   └── views.py
│   ├── __init__.py
│   ├── analytics
│   ├── api
│   ├── app
│   ├── asgi.py
│   ├── authentication
│   ├── bgtasks
│   ├── celery.py
│   ├── db
│   ├── license
│   ├── middleware
│   ├── seeds
│   ├── settings
│   ├── space
│   ├── static
│   ├── tests
│   ├── throttles
│   ├── urls.py
│   ├── utils
│   ├── web
│   └── wsgi.py
├── requirements/
│   ├── base.txt
│   ├── local.txt
│   ├── production.txt
│   └── test.txt
├── templates/
│   ├── admin/
│   │   └── base_site.html
│   ├── emails/
│   │   ├── auth/
│   │   │   ├── forgot_password.html
│   │   │   └── magic_signin.html
│   │   ├── exports/
│   │   │   └── analytics.html
│   │   ├── invitations/
│   │   │   ├── project_invitation.html
│   │   │   └── workspace_invitation.html
│   │   ├── notifications/
│   │   │   ├── issue-updates.html
│   │   │   ├── project_addition.html
│   │   │   └── webhook-deactivate.html
│   │   ├── user/
│   │   │   ├── email_updated.html
│   │   │   ├── user_activation.html
│   │   │   └── user_deactivation.html
│   │   ├── auth
│   │   ├── exports
│   │   ├── invitations
│   │   ├── notifications
│   │   ├── test_email.html
│   │   └── user
│   ├── admin
│   ├── base.html
│   ├── csrf_failure.html
│   └── emails
├── .coveragerc
├── .env.example
├── .prettierignore
├── Dockerfile.api
├── Dockerfile.dev
├── bin
├── manage.py
├── package.json
├── plane
├── pyproject.toml
├── pytest.ini
├── requirements
├── requirements.txt
├── run_tests.py
├── run_tests.sh
└── templates
```

---

# Estado de implementación Rust vs Django

> Verificado contra `apps/api_rust/src/routes/mod.rs`
> ✅ = implementado en Rust | ❌ = pendiente

## Auth (`authentication/`)

- ✅ `GET  /auth/get-csrf-token`
- ✅ `POST /auth/sign-in`
- ✅ `POST /auth/sign-up`
- ✅ `POST /auth/sign-out`
- ✅ `POST /auth/email-check`
- ✅ `POST /auth/magic-sign-in`
- ✅ `POST /auth/magic-sign-up`
- ✅ `POST /auth/magic-generate`
- ✅ `POST /auth/forgot-password`
- ✅ `POST /auth/reset-password/{uidb64}/{token}`
- ✅ `POST /auth/change-password`
- ✅ `POST /auth/set-password`
- ✅ `GET  /auth/github/callback`
- ✅ `GET  /auth/github/user-callback`
- ✅ `GET  /auth/gitlab`
- ✅ `GET  /auth/gitlab/callback`
- ✅ `GET  /auth/google`
- ✅ `GET  /auth/google/callback`
- ✅ `GET  /auth/gitea`
- ✅ `GET  /auth/gitea/callback`
- ✅ `POST /spaces/sign-in`
- ✅ `POST /spaces/sign-out`
- ✅ `POST /spaces/magic-sign-in`
- ✅ `POST /spaces/magic-sign-up`
- ✅ `POST /spaces/magic-generate`
- ✅ `POST /spaces/email-check`
- ✅ `POST /spaces/forgot-password`
- ✅ `POST /spaces/reset-password/{uidb64}/{token}`

## License / Instance (`license/`)

- ✅ `GET/POST   /instances`
- ✅ `GET        /instances/email-credentials-check`
- ✅ `GET/PATCH  /instances/configurations`
- ✅ `POST       /instances/configurations/disable-email-feature`
- ✅ `GET/POST   /instances/admins`
- ✅ `GET/PATCH/DELETE /instances/admins/{pk}`
- ✅ `POST       /instances/admins/sign-in`
- ✅ `POST       /instances/admins/sign-up`
- ✅ `POST       /instances/admins/sign-out`
- ✅ `GET        /instances/admins/me`
- ✅ `GET/PATCH  /instances/admins/session`
- ✅ `GET        /instances/workspaces`
- ✅ `GET        /instances/workspace-slug-check`
- ✅ `POST       /instances/admins/sign-up-screen-visited`

## Users (`app/urls/user.py`)

- ✅ `GET/PATCH  /users/me`
- ✅ `GET/DELETE /users/session`
- ✅ `PATCH      /users/me/settings`
- ✅ `GET/PATCH  /users/me/profile`
- ✅ `GET        /users/me/accounts`
- ✅ `DELETE     /users/me/accounts/{pk}`
- ✅ `GET/POST   /users/me/instance-admin`
- ✅ `POST       /users/me/onboard`
- ✅ `POST       /users/me/tour-completed`
- ✅ `GET        /users/me/activities`
- ✅ `GET        /users/me/workspaces`
- ✅ `GET        /users/me/workspaces/{slug}/activity-graph`
- ✅ `GET        /users/me/workspaces/{slug}/issues-completed-graph`
- ✅ `GET        /users/me/workspaces/{slug}/dashboard`
- ✅ `GET/PATCH  /users/me/notification-preferences`
- ✅ `GET        /users/last-visited-workspace`
- ✅ `POST       /users/me/email/generate-code`
- ✅ `POST       /users/me/email`

## API Tokens (`app/urls/api.py`)

- ✅ `GET/POST   /api-tokens`
- ✅ `GET/PATCH/DELETE /api-tokens/{pk}`

## Timezones

- ✅ `GET /timezones`

## Workspaces (`app/urls/workspace.py`)

- ✅ `GET        /workspace-slug-check`
- ✅ `GET/POST   /workspaces`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}`
- ✅ `GET/POST   /workspaces/{slug}/invitations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/invitations/{pk}`
- ✅ `GET        /users/me/workspaces/invitations`
- ✅ `GET        /workspaces/{slug}/members`
- ✅ `GET        /workspaces/{slug}/project-members`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/members/{pk}`
- ✅ `POST       /workspaces/{slug}/members/leave`
- ✅ `GET        /workspaces/{slug}/workspace-members/me`
- ✅ `GET        /workspaces/{slug}/workspace-views`
- ✅ `GET        /workspaces/{slug}/user-stats/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-activity/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-activity/{user_id}/export`
- ✅ `GET        /workspaces/{slug}/user-profile/{user_id}`
- ✅ `GET        /workspaces/{slug}/user-issues/{user_id}`
- ✅ `GET        /workspaces/{slug}/labels`
- ✅ `GET/PATCH  /workspaces/{slug}/user-properties`
- ✅ `GET        /workspaces/{slug}/states`
- ✅ `GET        /workspaces/{slug}/estimates`
- ✅ `GET        /workspaces/{slug}/modules`
- ✅ `GET        /workspaces/{slug}/cycles`
- ✅ `GET/POST   /workspaces/{slug}/user-favorites`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/user-favorites/{favorite_id}`
- ✅ `GET/POST   /workspaces/{slug}/user-favorites/{favorite_id}/children`
- ✅ `GET/POST   /workspaces/{slug}/draft-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/draft-issues/{pk}`
- ✅ `POST       /workspaces/{slug}/draft-to-issue/{draft_id}`
- ✅ `GET/POST   /workspaces/{slug}/quick-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/quick-links/{pk}`
- ✅ `GET/PATCH  /workspaces/{slug}/home-preferences`
- ✅ `GET/PATCH  /workspaces/{slug}/home-preferences/{key}`
- ✅ `GET        /workspaces/{slug}/recent-visits`
- ✅ `GET/POST   /workspaces/{slug}/stickies`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/stickies/{pk}`
- ✅ `GET/PATCH  /workspaces/{slug}/sidebar-preferences`
- ✅ `POST       /workspaces/{slug}/invitations/{pk}/join`
- ✅ `GET/POST   /workspaces/{slug}/workspace-themes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-themes/{pk}`

## Projects (`app/urls/project.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects`
- ✅ `GET        /workspaces/{slug}/projects/details`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{pk}`
- ✅ `GET/DELETE /workspaces/{slug}/project-identifiers`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/invitations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/invitations/{pk}`
- ✅ `GET        /users/me/workspaces/{slug}/project-roles`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/members`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/members/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/members/leave`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-members/me`
- ✅ `GET        /users/me/workspaces/{slug}/projects/invitations`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/join/{pk}`

## States (`app/urls/state.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/states`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/states/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intake-state`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/states/{pk}/mark-default`

## Issues (`app/urls/issue.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issue-labels`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issue-labels/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-create-labels`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-delete-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/bulk-archive-issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-attachments/{pk}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments`
- ✅ `DELETE     /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{pk}/archive`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/deleted-issues`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issue-dates`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/versions/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/work-items/{work_item_id}/description-versions/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/meta`
- ✅ `GET        /workspaces/{slug}/work-items/{combined}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/list`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues-detail`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/v2/issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}`

## Cycles (`app/urls/cycle.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/cycles/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/cycle-issues/{issue_id}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/progress`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/analytics`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/cycles/date-check`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-cycles`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-cycles/{cycle_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/transfer-issues`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/cycles/{cycle_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-cycles`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/archived-cycles/{pk}`

## Modules (`app/urls/module.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/modules/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/modules/{module_id}/issues/{issue_id}`
- ✅ `GET/PATCH  /workspaces/{slug}/projects/{project_id}/modules/{module_id}/user-properties`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/modules`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/modules/{module_id}/module-links/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-modules`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-modules/{module_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/modules/{module_id}/archive`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/archived-modules`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/archived-modules/{pk}`

## Estimates (`app/urls/estimate.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/estimates`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/estimates/{estimate_id}/estimate-points/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-estimates`

## Pages (`app/urls/page.py`)

- ✅ `GET        /workspaces/{slug}/projects/{project_id}/pages-summary`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/pages`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}`
- ✅ `POST/DELETE /workspaces/{slug}/projects/{project_id}/favorite-pages/{page_id}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/archive`
- ✅ `POST/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/lock`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/access`
- ✅ `GET/POST/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/description`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/pages/{page_id}/versions/{pk}`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/pages/{page_id}/duplicate`

## Views (`app/urls/views.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/views`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/views/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/views`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/views/{pk}`
- ✅ `GET        /workspaces/{slug}/issues`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/user-favorite-views`
- ✅ `DELETE     /workspaces/{slug}/projects/{project_id}/user-favorite-views/{view_id}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/project-views`

## Analytics (`app/urls/analytic.py`)

- ✅ `GET/POST   /workspaces/{slug}/analytics`
- ✅ `GET/POST   /workspaces/{slug}/analytic-view`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/analytic-view/{pk}`
- ✅ `GET/POST/PATCH/DELETE /workspaces/{slug}/saved-analytic-view/{analytic_id}`
- ✅ `POST       /workspaces/{slug}/export-analytics`
- ✅ `GET        /workspaces/{slug}/default-analytics`
- ✅ `GET        /workspaces/{slug}/project-stats`
- ✅ `GET        /workspaces/{slug}/advance-analytics`
- ✅ `GET        /workspaces/{slug}/advance-analytics-stats`
- ✅ `GET        /workspaces/{slug}/advance-analytics-charts`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics-stats`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/advance-analytics-charts`

## Notifications (`app/urls/notification.py`)

- ✅ `GET        /workspaces/{slug}/users/notifications`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/users/notifications/{pk}`
- ✅ `POST       /workspaces/{slug}/users/notifications/{pk}/read`
- ✅ `POST       /workspaces/{slug}/users/notifications/{pk}/archive`
- ✅ `GET        /workspaces/{slug}/users/notifications/unread`
- ✅ `POST       /workspaces/{slug}/users/notifications/mark-all-read`
- ✅ `GET/PATCH  /users/me/notification-preferences`

## Search (`app/urls/search.py`)

- ✅ `GET        /workspaces/{slug}/search`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/search-issues`
- ✅ `GET        /workspaces/{slug}/entity-search`

## Assets v2 (`app/urls/asset.py`)

- ✅ `GET/POST   /assets/v2/workspaces/{slug}`
- ✅ `GET/PATCH/DELETE /assets/v2/workspaces/{slug}/{asset_id}`
- ✅ `GET/POST   /assets/v2/user-assets`
- ✅ `GET/PATCH/DELETE /assets/v2/user-assets/{asset_id}`
- ✅ `GET        /assets/v2/static/{asset_id}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments`
- ✅ `DELETE     /assets/v2/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/attachments/{pk}`
- ✅ `POST       /assets/v2/workspaces/{slug}/restore/{asset_id}`
- ✅ `GET/POST   /assets/v2/workspaces/{slug}/projects/{project_id}`
- ✅ `GET/PATCH/DELETE /assets/v2/workspaces/{slug}/projects/{project_id}/{pk}`
- ✅ `POST       /assets/v2/workspaces/{slug}/projects/{project_id}/{entity_id}/bulk`
- ✅ `GET        /assets/v2/workspaces/{slug}/check/{asset_id}`
- ✅ `POST       /assets/v2/workspaces/{slug}/duplicate-assets/{asset_id}`
- ✅ `GET        /assets/v2/workspaces/{slug}/download/{asset_id}`
- ✅ `GET        /assets/v2/workspaces/{slug}/projects/{project_id}/download/{asset_id}`

## Exporter (`app/urls/exporter.py`)

- ✅ `GET/POST   /workspaces/{slug}/export-issues`
- ✅ `GET        /workspaces/{slug}/export-issues/{token}`

## Intake (`app/urls/intake.py`)

- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intakes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intakes/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/intake-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/intake-issues/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/inboxes`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/inboxes/{pk}`
- ✅ `GET/POST   /workspaces/{slug}/projects/{project_id}/inbox-issues`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/projects/{project_id}/inbox-issues/{pk}`
- ✅ `GET        /workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions`
- ✅ `GET/DELETE /workspaces/{slug}/projects/{project_id}/intake-work-items/{work_item_id}/description-versions/{pk}`

## Integrations (`app/urls/integration.py`)

- ✅ `GET        /github/callback`
- ✅ `GET        /github/user-callback`
- ✅ `GET        /integrations`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/{pk}`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{provider}/provider`
- ✅ `POST       /workspaces/{slug}/workspace-integrations/{provider}/install`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations/github/repo-syncs`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/github/repo-syncs/{pk}`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{wi_id}/github-repositories`
- ✅ `GET        /workspaces/{slug}/workspace-integrations/{wi_id}/gitlab-repositories`
- ✅ `GET/POST   /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/workspace-integrations/{wi_id}/pr-state-mappings/{pk}`

## Importer (`app/urls/importer.py`)

- ✅ `GET        /workspaces/{slug}/importers/github/repositories`
- ✅ `GET/POST   /workspaces/{slug}/importers/github`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/importers/github/{importer_id}`
- ✅ `GET        /workspaces/{slug}/importers/gitlab/repositories`
- ✅ `GET/POST   /workspaces/{slug}/importers/gitlab`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/importers/gitlab/{importer_id}`

## External / AI (`app/urls/external.py`)

- ✅ `GET        /unsplash`
- ✅ `POST       /workspaces/{slug}/ai-assistant`
- ✅ `POST       /workspaces/{slug}/projects/{project_id}/ai-assistant`
- ✅ `POST       /workspaces/{slug}/rephrase-grammar`
- ✅ `POST       /github-webhook`  ← webhook entrante de GitHub
- ✅ `POST       /gitlab-webhook`  ← webhook entrante de GitLab

## Webhooks (`app/urls/webhook.py`)

- ✅ `GET/POST   /workspaces/{slug}/webhooks`
- ✅ `GET/PATCH/DELETE /workspaces/{slug}/webhooks/{pk}`
- ✅ `POST       /workspaces/{slug}/webhooks/{pk}/regenerate`
- ✅ `GET        /workspaces/{slug}/webhook-logs/{webhook_id}`

---

## Resumen

| Módulo            | Implementado | Pendiente |
|-------------------|:------------:|:---------:|
| Auth              | 28           | 0         |
| License/Instance  | 14           | 0         |
| Users             | 18           | 0         |
| API Tokens        | 2            | 0         |
| Workspaces        | 30           | 3         |
| Projects          | 14           | 0         |
| States            | 4            | 0         |
| Issues            | 31           | 4         |
| Cycles            | 7            | 7         |
| Modules           | 5            | 8         |
| Estimates         | 4            | 1         |
| Pages             | 11           | 0         |
| Views             | 8            | 0         |
| Analytics         | 13           | 0         |
| Notifications     | 7            | 0         |
| Search            | 2            | 1         |
| Assets v2         | 15           | 0         |
| Exporter          | 2            | 0         |
| Intake            | 10           | 0         |
| Integrations      | 13           | 0         |
| Importer          | 6            | 0         |
| External/AI       | 6            | 0         |
| Webhooks          | 4            | 0         |
| **TOTAL**         | **258**      | **22**    |
