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
