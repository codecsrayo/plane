use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000005_issues_and_modules"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── analytic_views ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("analytic_views"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("query")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("query_dict")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("analytic_views_created_by_id_1b3ca0a9_fk_users_id").from(Alias::new("analytic_views"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("analytic_views_updated_by_id_b6d827e1_fk_users_id").from(Alias::new("analytic_views"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("analytic_views_workspace_id_ca6e5c0b_fk_workspaces_id").from(Alias::new("analytic_views"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issues ────────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issues"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("priority")).string_len(30).not_null())
                    .col(ColumnDef::new(Alias::new("start_date")).date().null())
                    .col(ColumnDef::new(Alias::new("target_date")).date().null())
                    .col(ColumnDef::new(Alias::new("sequence_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("state_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("completed_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("point")).integer().null())
                    .col(ColumnDef::new(Alias::new("archived_at")).date().null())
                    .col(ColumnDef::new(Alias::new("is_draft")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("estimate_point_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("type_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_created_by_id_8f0ae62b_fk_user_id").from(Alias::new("issues"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_updated_by_id_f1261863_fk_user_id").from(Alias::new("issues"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_parent_id_ce8d76ba_fk_issue_id").from(Alias::new("issues"), Alias::new("parent_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_project_id_fea0fc80_fk_project_id").from(Alias::new("issues"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_state_id_1a65560d_fk_state_id").from(Alias::new("issues"), Alias::new("state_id")).to(Alias::new("states"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_workspace_id_c84878c1_fk_workspace_id").from(Alias::new("issues"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issues_estimate_point_id_a6822abe_fk_estimate_points_id").from(Alias::new("issues"), Alias::new("estimate_point_id")).to(Alias::new("estimate_points"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issues_type_id_a4710b19_fk_issue_types_id").from(Alias::new("issues"), Alias::new("type_id")).to(Alias::new("issue_types"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        for (name, table, col) in [
            ("issue_created_by_id_8f0ae62b", "issues", "created_by_id"),
            ("issue_updated_by_id_f1261863", "issues", "updated_by_id"),
            ("issue_parent_id_ce8d76ba", "issues", "parent_id"),
            ("issue_project_id_fea0fc80", "issues", "project_id"),
            ("issue_state_id_1a65560d", "issues", "state_id"),
            ("issue_workspace_id_c84878c1", "issues", "workspace_id"),
            ("issues_estimate_point_id_a6822abe", "issues", "estimate_point_id"),
            ("issues_type_id_a4710b19", "issues", "type_id"),
            ("analytic_views_created_by_id_1b3ca0a9", "analytic_views", "created_by_id"),
            ("analytic_views_updated_by_id_b6d827e1", "analytic_views", "updated_by_id"),
            ("analytic_views_workspace_id_ca6e5c0b", "analytic_views", "workspace_id"),
        ] {
            manager.create_index(Index::create().name(name).table(Alias::new(table)).col(Alias::new(col)).to_owned()).await?;
        }

        // ── issue_activities ──────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_activities"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("verb")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("field")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("old_value")).text().null())
                    .col(ColumnDef::new(Alias::new("new_value")).text().null())
                    .col(ColumnDef::new(Alias::new("comment")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_comment_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("new_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("old_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("epoch")).double().null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_activity_created_by_id_49516e3d_fk_user_id").from(Alias::new("issue_activities"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_activity_updated_by_id_0075f9bd_fk_user_id").from(Alias::new("issue_activities"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_activity_actor_id_52fdd42d_fk_user_id").from(Alias::new("issue_activities"), Alias::new("actor_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_activities_issue_id_180e5662_fk_issues_id").from(Alias::new("issue_activities"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_activity_project_id_d0ac2ccf_fk_project_id").from(Alias::new("issue_activities"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_activity_workspace_id_65acaf73_fk_workspace_id").from(Alias::new("issue_activities"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        for (name, table, col) in [
            ("issue_activity_actor_id_52fdd42d", "issue_activities", "actor_id"),
            ("issue_activity_created_by_id_49516e3d", "issue_activities", "created_by_id"),
            ("issue_activity_issue_id_807fbde4", "issue_activities", "issue_id"),
            ("issue_activity_project_id_d0ac2ccf", "issue_activities", "project_id"),
            ("issue_activity_updated_by_id_0075f9bd", "issue_activities", "updated_by_id"),
            ("issue_activity_workspace_id_65acaf73", "issue_activities", "workspace_id"),
        ] {
            manager.create_index(Index::create().name(name).table(Alias::new(table)).col(Alias::new(col)).to_owned()).await?;
        }

        // ── issue_comments ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_comments"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("comment_stripped")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("comment_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("comment_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("access")).string_len(100).not_null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("edited_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("description_id")).uuid().null().unique_key())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .foreign_key(ForeignKey::create().name("issue_comment_created_by_id_0765f239_fk_user_id").from(Alias::new("issue_comments"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comment_updated_by_id_96cfb86e_fk_user_id").from(Alias::new("issue_comments"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comment_actor_id_d312315b_fk_user_id").from(Alias::new("issue_comments"), Alias::new("actor_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comment_issue_id_d0195e35_fk_issue_id").from(Alias::new("issue_comments"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comment_project_id_db37c105_fk_project_id").from(Alias::new("issue_comments"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comment_workspace_id_3f7969ec_fk_workspace_id").from(Alias::new("issue_comments"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_comments_parent_id_d8db10b1_fk_issue_comments_id").from(Alias::new("issue_comments"), Alias::new("parent_id")).to(Alias::new("issue_comments"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // Now update issue_activities to FK issue_comment_id -> issue_comments
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("issue_activity_issue_comment_id_701f3c3c_fk_issue_comment_id")
                    .from(Alias::new("issue_activities"), Alias::new("issue_comment_id"))
                    .to(Alias::new("issue_comments"), Alias::new("id"))
                    .to_owned(),
            )
            .await?;

        for (name, table, col) in [
            ("issue_comment_actor_id_d312315b", "issue_comments", "actor_id"),
            ("issue_comment_created_by_id_0765f239", "issue_comments", "created_by_id"),
            ("issue_comment_issue_id_d0195e35", "issue_comments", "issue_id"),
            ("issue_comment_project_id_db37c105", "issue_comments", "project_id"),
            ("issue_comment_updated_by_id_96cfb86e", "issue_comments", "updated_by_id"),
            ("issue_comment_workspace_id_3f7969ec", "issue_comments", "workspace_id"),
            ("issue_comments_parent_id_d8db10b1", "issue_comments", "parent_id"),
            ("issue_activity_issue_comment_id_701f3c3c", "issue_activities", "issue_comment_id"),
        ] {
            manager.create_index(Index::create().name(name).table(Alias::new(table)).col(Alias::new(col)).to_owned()).await?;
        }

        // ── issue_assignees ───────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_assignees"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("assignee_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_assignee_assignee_id_50f5c04e_fk_user_id").from(Alias::new("issue_assignees"), Alias::new("assignee_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_assignee_created_by_id_f693d43b_fk_user_id").from(Alias::new("issue_assignees"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_assignee_updated_by_id_c54088aa_fk_user_id").from(Alias::new("issue_assignees"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_assignee_issue_id_72da08db_fk_issue_id").from(Alias::new("issue_assignees"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_assignee_project_id_61c18bf2_fk_project_id").from(Alias::new("issue_assignees"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_assignee_workspace_id_9aad55b7_fk_workspace_id").from(Alias::new("issue_assignees"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("issue_assignees_issue_id_assignee_id_deleted_at_b2623a0e_uniq").table(Alias::new("issue_assignees")).col(Alias::new("issue_id")).col(Alias::new("assignee_id")).col(Alias::new("deleted_at")).to_owned()).await?;
        manager.create_index(Index::create().unique().name("issue_assignee_unique_issue_assignee_when_deleted_at_null").table(Alias::new("issue_assignees")).col(Alias::new("issue_id")).col(Alias::new("assignee_id")).to_owned()).await?;

        for (name, table, col) in [
            ("issue_assignee_assignee_id_50f5c04e", "issue_assignees", "assignee_id"),
            ("issue_assignee_created_by_id_f693d43b", "issue_assignees", "created_by_id"),
            ("issue_assignee_issue_id_72da08db", "issue_assignees", "issue_id"),
            ("issue_assignee_project_id_61c18bf2", "issue_assignees", "project_id"),
            ("issue_assignee_updated_by_id_c54088aa", "issue_assignees", "updated_by_id"),
            ("issue_assignee_workspace_id_9aad55b7", "issue_assignees", "workspace_id"),
        ] {
            manager.create_index(Index::create().name(name).table(Alias::new(table)).col(Alias::new(col)).to_owned()).await?;
        }

        // ── issue_attachments ─────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_attachments"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("attributes")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("asset")).string_len(100).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_attachments_created_by_id_87be05bb_fk_users_id").from(Alias::new("issue_attachments"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_attachments_updated_by_id_47dceec1_fk_users_id").from(Alias::new("issue_attachments"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_attachments_issue_id_0faf88bf_fk_issues_id").from(Alias::new("issue_attachments"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_attachments_project_id_a95fe706_fk_projects_id").from(Alias::new("issue_attachments"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_attachments_workspace_id_c456a532_fk_workspaces_id").from(Alias::new("issue_attachments"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_blockers ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_blockers"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("block_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("blocked_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_blocker_block_id_5d15a701_fk_issue_id").from(Alias::new("issue_blockers"), Alias::new("block_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_blocker_blocked_by_id_a138af71_fk_issue_id").from(Alias::new("issue_blockers"), Alias::new("blocked_by_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_blocker_created_by_id_0d19f6ea_fk_user_id").from(Alias::new("issue_blockers"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_blocker_updated_by_id_4af87d63_fk_user_id").from(Alias::new("issue_blockers"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_blocker_project_id_380bd100_fk_project_id").from(Alias::new("issue_blockers"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_blocker_workspace_id_419a1c71_fk_workspace_id").from(Alias::new("issue_blockers"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_labels ──────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_labels"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("label_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_label_created_by_id_94075315_fk_user_id").from(Alias::new("issue_labels"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_label_updated_by_id_a97a6733_fk_user_id").from(Alias::new("issue_labels"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_label_issue_id_0f252e52_fk_issue_id").from(Alias::new("issue_labels"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_label_label_id_5f22777f_fk_label_id").from(Alias::new("issue_labels"), Alias::new("label_id")).to(Alias::new("labels"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_label_project_id_eaa2ba39_fk_project_id").from(Alias::new("issue_labels"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_label_workspace_id_b5b1faac_fk_workspace_id").from(Alias::new("issue_labels"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_links ───────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_links"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("title")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("url")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("metadata")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_links_created_by_id_5e4aa092_fk_users_id").from(Alias::new("issue_links"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_links_updated_by_id_a771cce4_fk_users_id").from(Alias::new("issue_links"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_links_issue_id_7032881f_fk_issues_id").from(Alias::new("issue_links"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_links_project_id_63d6e9ce_fk_projects_id").from(Alias::new("issue_links"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_links_workspace_id_ff9038e7_fk_workspaces_id").from(Alias::new("issue_links"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_mentions ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_mentions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("mention_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_mentions_created_by_id_eb44759e_fk_users_id").from(Alias::new("issue_mentions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_mentions_updated_by_id_c62106d3_fk_users_id").from(Alias::new("issue_mentions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_mentions_mention_id_cf1b9346_fk_users_id").from(Alias::new("issue_mentions"), Alias::new("mention_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_mentions_issue_id_d8821107_fk_issues_id").from(Alias::new("issue_mentions"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_mentions_project_id_d0cccdf5_fk_projects_id").from(Alias::new("issue_mentions"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_mentions_workspace_id_4ca59d05_fk_workspaces_id").from(Alias::new("issue_mentions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_reactions ───────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_reactions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("reaction")).text().not_null())
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_reactions_actor_id_5f5b8303_fk_users_id").from(Alias::new("issue_reactions"), Alias::new("actor_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_reactions_created_by_id_3953b7de_fk_users_id").from(Alias::new("issue_reactions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_reactions_updated_by_id_4069af90_fk_users_id").from(Alias::new("issue_reactions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_reactions_issue_id_2c324bae_fk_issues_id").from(Alias::new("issue_reactions"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_reactions_project_id_8708ecaf_fk_projects_id").from(Alias::new("issue_reactions"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_reactions_workspace_id_bd8d7550_fk_workspaces_id").from(Alias::new("issue_reactions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("issue_reactions_issue_id_actor_id_reacti_7da73ced_uniq").table(Alias::new("issue_reactions")).col(Alias::new("issue_id")).col(Alias::new("actor_id")).col(Alias::new("reaction")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── issue_relations ───────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_relations"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("relation_type")).string_len(20).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("related_issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_relations_created_by_id_854d07e7_fk_users_id").from(Alias::new("issue_relations"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_relations_updated_by_id_3dfa850f_fk_users_id").from(Alias::new("issue_relations"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_relations_issue_id_e1db6f72_fk_issues_id").from(Alias::new("issue_relations"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_relations_related_issue_id_e1ea44a7_fk_issues_id").from(Alias::new("issue_relations"), Alias::new("related_issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_relations_project_id_15350161_fk_projects_id").from(Alias::new("issue_relations"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_relations_workspace_id_00b50e90_fk_workspaces_id").from(Alias::new("issue_relations"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("issue_relations_issue_id_related_issue_i_cc724584_uniq").table(Alias::new("issue_relations")).col(Alias::new("issue_id")).col(Alias::new("related_issue_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── issue_sequences ───────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_sequences"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("sequence")).big_integer().not_null())
                    .col(ColumnDef::new(Alias::new("deleted")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_sequence_created_by_id_59270506_fk_user_id").from(Alias::new("issue_sequences"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_sequence_updated_by_id_310c8dd3_fk_user_id").from(Alias::new("issue_sequences"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_sequence_issue_id_16e9f00f_fk_issue_id").from(Alias::new("issue_sequences"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_sequence_project_id_ce882e85_fk_project_id").from(Alias::new("issue_sequences"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_sequence_workspace_id_0d3f0fd4_fk_workspace_id").from(Alias::new("issue_sequences"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_subscribers ─────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_subscribers"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("subscriber_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_subscribers_created_by_id_b6ea0157_fk_users_id").from(Alias::new("issue_subscribers"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_subscribers_updated_by_id_1bfc2f55_fk_users_id").from(Alias::new("issue_subscribers"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_subscribers_subscriber_id_2d89c988_fk_users_id").from(Alias::new("issue_subscribers"), Alias::new("subscriber_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_subscribers_issue_id_85cf2093_fk_issues_id").from(Alias::new("issue_subscribers"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_subscribers_project_id_cf48d75f_fk_projects_id").from(Alias::new("issue_subscribers"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_subscribers_workspace_id_96afa91f_fk_workspaces_id").from(Alias::new("issue_subscribers"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("issue_subscribers_issue_id_subscriber_id_d_587dec1a_uniq").table(Alias::new("issue_subscribers")).col(Alias::new("issue_id")).col(Alias::new("subscriber_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── issue_votes ───────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_votes"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("vote")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_votes_actor_id_525cab61_fk_users_id").from(Alias::new("issue_votes"), Alias::new("actor_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_votes_created_by_id_86adcf5c_fk_users_id").from(Alias::new("issue_votes"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_votes_updated_by_id_9e2a6cdc_fk_users_id").from(Alias::new("issue_votes"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_votes_issue_id_07a61ecb_fk_issues_id").from(Alias::new("issue_votes"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_votes_project_id_b649f55b_fk_projects_id").from(Alias::new("issue_votes"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_votes_workspace_id_a3e91a6b_fk_workspaces_id").from(Alias::new("issue_votes"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("issue_votes_issue_id_actor_id_deleted_at_886f34e8_uniq").table(Alias::new("issue_votes")).col(Alias::new("issue_id")).col(Alias::new("actor_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── issue_views ───────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_views"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("query")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("access")).small_integer().not_null())
                    .col(ColumnDef::new(Alias::new("filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("display_filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("display_properties")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("is_locked")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("rich_filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("archived_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("issue_views_created_by_id_0d2e456b_fk_users_id").from(Alias::new("issue_views"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_views_updated_by_id_28cd9870_fk_users_id").from(Alias::new("issue_views"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_views_owned_by_id_5e261e5d_fk_users_id").from(Alias::new("issue_views"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_views_project_id_55ee009f_fk_projects_id").from(Alias::new("issue_views"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_views_workspace_id_8785e03d_fk_workspaces_id").from(Alias::new("issue_views"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_description_versions ────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_description_versions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("description_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("last_saved_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(ForeignKey::create().name("issue_description_versions_created_by_id_3f7e62a1_fk_users_id").from(Alias::new("issue_description_versions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_description_versions_updated_by_id_6530365d_fk_users_id").from(Alias::new("issue_description_versions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_description_versions_owned_by_id_0effe4d0_fk_users_id").from(Alias::new("issue_description_versions"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_description_versions_issue_id_c8baa13e_fk_issues_id").from(Alias::new("issue_description_versions"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_description_versions_project_id_536b23ef_fk_projects_id").from(Alias::new("issue_description_versions"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_description_ve_workspace_id_88e930f9_fk_workspace").from(Alias::new("issue_description_versions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── issue_versions ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_versions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("parent")).uuid().null())
                    .col(ColumnDef::new(Alias::new("state")).uuid().null())
                    .col(ColumnDef::new(Alias::new("estimate_point")).uuid().null())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("priority")).string_len(30).not_null())
                    .col(ColumnDef::new(Alias::new("start_date")).date().null())
                    .col(ColumnDef::new(Alias::new("target_date")).date().null())
                    .col(ColumnDef::new(Alias::new("sequence_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("completed_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("archived_at")).date().null())
                    .col(ColumnDef::new(Alias::new("is_draft")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("type")).uuid().null())
                    .col(ColumnDef::new(Alias::new("last_saved_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("properties")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("meta")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("activity_id")).uuid().null())
                    .foreign_key(ForeignKey::create().name("issue_versions_created_by_id_a782830a_fk_users_id").from(Alias::new("issue_versions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_updated_by_id_dcae6dd2_fk_users_id").from(Alias::new("issue_versions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_owned_by_id_7586378d_fk_users_id").from(Alias::new("issue_versions"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_issue_id_25cf001c_fk_issues_id").from(Alias::new("issue_versions"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_project_id_a069ad03_fk_projects_id").from(Alias::new("issue_versions"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_workspace_id_b8c48b7c_fk_workspaces_id").from(Alias::new("issue_versions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("issue_versions_activity_id_b1872ffc_fk_issue_activities_id").from(Alias::new("issue_versions"), Alias::new("activity_id")).to(Alias::new("issue_activities"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── comment_reactions ─────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("comment_reactions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("reaction")).text().not_null())
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("comment_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("comment_reactions_actor_id_21219e9c_fk_users_id").from(Alias::new("comment_reactions"), Alias::new("actor_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("comment_reactions_created_by_id_9aeb43c4_fk_users_id").from(Alias::new("comment_reactions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("comment_reactions_updated_by_id_c74c9bbd_fk_users_id").from(Alias::new("comment_reactions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("comment_reactions_comment_id_87c59446_fk_issue_comments_id").from(Alias::new("comment_reactions"), Alias::new("comment_id")).to(Alias::new("issue_comments"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("comment_reactions_project_id_ab9114b4_fk_projects_id").from(Alias::new("comment_reactions"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("comment_reactions_workspace_id_b614ca4f_fk_workspaces_id").from(Alias::new("comment_reactions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("comment_reactions_comment_id_actor_id_reac_24dc2de6_uniq").table(Alias::new("comment_reactions")).col(Alias::new("comment_id")).col(Alias::new("actor_id")).col(Alias::new("reaction")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── cycles ────────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cycles"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("start_date")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("end_date")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("view_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("progress_snapshot")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("archived_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("timezone")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .foreign_key(ForeignKey::create().name("cycle_created_by_id_78e43b79_fk_user_id").from(Alias::new("cycles"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_updated_by_id_93baee43_fk_user_id").from(Alias::new("cycles"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_owned_by_id_5456a4d1_fk_user_id").from(Alias::new("cycles"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_project_id_0b590349_fk_project_id").from(Alias::new("cycles"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_workspace_id_a199e8e1_fk_workspace_id").from(Alias::new("cycles"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── cycle_issues ──────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cycle_issues"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("cycle_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("cycle_issue_created_by_id_30b27539_fk_user_id").from(Alias::new("cycle_issues"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_issue_updated_by_id_cb4516f2_fk_user_id").from(Alias::new("cycle_issues"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_issue_cycle_id_ec681215_fk_cycle_id").from(Alias::new("cycle_issues"), Alias::new("cycle_id")).to(Alias::new("cycles"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_issues_issue_id_2d5ac97f_fk_issues_id").from(Alias::new("cycle_issues"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_issue_project_id_6ad3257a_fk_project_id").from(Alias::new("cycle_issues"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_issue_workspace_id_1d77330e_fk_workspace_id").from(Alias::new("cycle_issues"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("cycle_issues_issue_id_cycle_id_deleted_at_93e8fecd_uniq").table(Alias::new("cycle_issues")).col(Alias::new("issue_id")).col(Alias::new("cycle_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── cycle_user_properties ─────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("cycle_user_properties"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("display_filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("display_properties")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("cycle_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("rich_filters")).json_binary().not_null())
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_created_by_id_501f371c_fk_users_id").from(Alias::new("cycle_user_properties"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_updated_by_id_1b5ac27b_fk_users_id").from(Alias::new("cycle_user_properties"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_user_id_9e9ef97d_fk_users_id").from(Alias::new("cycle_user_properties"), Alias::new("user_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_cycle_id_1f8bdf35_fk_cycles_id").from(Alias::new("cycle_user_properties"), Alias::new("cycle_id")).to(Alias::new("cycles"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_project_id_4efc0f07_fk_projects_id").from(Alias::new("cycle_user_properties"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("cycle_user_properties_workspace_id_62d65d71_fk_workspaces_id").from(Alias::new("cycle_user_properties"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("cycle_user_properties_cycle_id_user_id_deleted_at_fbe00cf4_uniq").table(Alias::new("cycle_user_properties")).col(Alias::new("cycle_id")).col(Alias::new("user_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── modules ───────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("modules"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_text")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("description_html")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("start_date")).date().null())
                    .col(ColumnDef::new(Alias::new("target_date")).date().null())
                    .col(ColumnDef::new(Alias::new("status")).string_len(20).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("lead_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("view_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("archived_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("module_created_by_id_ff7a5866_fk_user_id").from(Alias::new("modules"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_updated_by_id_72ab6d5c_fk_user_id").from(Alias::new("modules"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_lead_id_04966630_fk_user_id").from(Alias::new("modules"), Alias::new("lead_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_project_id_da84b04f_fk_project_id").from(Alias::new("modules"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_workspace_id_0a826fef_fk_workspace_id").from(Alias::new("modules"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── module_issues ─────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("module_issues"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("module_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("module_issues_created_by_id_de0b995a_fk_user_id").from(Alias::new("module_issues"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_issues_updated_by_id_46dbf724_fk_user_id").from(Alias::new("module_issues"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_issues_issue_id_7caa908b_fk_issues_id").from(Alias::new("module_issues"), Alias::new("issue_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_issues_module_id_74e0ed5a_fk_module_id").from(Alias::new("module_issues"), Alias::new("module_id")).to(Alias::new("modules"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_issues_project_id_59836d1e_fk_project_id").from(Alias::new("module_issues"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_issues_workspace_id_6bf85201_fk_workspace_id").from(Alias::new("module_issues"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("module_issues_issue_id_module_id_deleted_at_f944f7c9_uniq").table(Alias::new("module_issues")).col(Alias::new("issue_id")).col(Alias::new("module_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── module_links ──────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("module_links"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("title")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("url")).string_len(200).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("module_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("metadata")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("module_links_created_by_id_eaf6492f_fk_users_id").from(Alias::new("module_links"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_links_updated_by_id_4da419e7_fk_users_id").from(Alias::new("module_links"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_links_module_id_0fda3f8a_fk_modules_id").from(Alias::new("module_links"), Alias::new("module_id")).to(Alias::new("modules"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_links_project_id_f720bb79_fk_projects_id").from(Alias::new("module_links"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_links_workspace_id_0521c11c_fk_workspaces_id").from(Alias::new("module_links"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── module_members ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("module_members"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("member_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("module_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("module_member_created_by_id_2ed84a65_fk_user_id").from(Alias::new("module_members"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_member_updated_by_id_a9046438_fk_user_id").from(Alias::new("module_members"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_member_member_id_928f473e_fk_user_id").from(Alias::new("module_members"), Alias::new("member_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_member_module_id_f00be7ef_fk_module_id").from(Alias::new("module_members"), Alias::new("module_id")).to(Alias::new("modules"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_member_project_id_ec8d2376_fk_project_id").from(Alias::new("module_members"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_member_workspace_id_f2f23c73_fk_workspace_id").from(Alias::new("module_members"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("module_members_module_id_member_id_deleted_at_bb7a6f00_uniq").table(Alias::new("module_members")).col(Alias::new("module_id")).col(Alias::new("member_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── module_user_properties ────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("module_user_properties"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("display_filters")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("display_properties")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("module_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("rich_filters")).json_binary().not_null())
                    .foreign_key(ForeignKey::create().name("module_user_properties_created_by_id_bdd98440_fk_users_id").from(Alias::new("module_user_properties"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_user_properties_updated_by_id_b7dafc77_fk_users_id").from(Alias::new("module_user_properties"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_user_properties_user_id_e83a1c2c_fk_users_id").from(Alias::new("module_user_properties"), Alias::new("user_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_user_properties_module_id_e95b158a_fk_modules_id").from(Alias::new("module_user_properties"), Alias::new("module_id")).to(Alias::new("modules"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_user_properties_project_id_3c5a4972_fk_projects_id").from(Alias::new("module_user_properties"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("module_user_properties_workspace_id_ddaf807c_fk_workspaces_id").from(Alias::new("module_user_properties"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("module_user_properties_module_id_user_id_delete_3269582d_uniq").table(Alias::new("module_user_properties")).col(Alias::new("module_id")).col(Alias::new("user_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── pages ─────────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("pages"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("access")).small_integer().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("color")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("archived_at")).date().null())
                    .col(ColumnDef::new(Alias::new("is_locked")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("view_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("is_global")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("moved_to_page")).uuid().null())
                    .col(ColumnDef::new(Alias::new("moved_to_project")).uuid().null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .foreign_key(ForeignKey::create().name("pages_created_by_id_d109a675_fk_users_id").from(Alias::new("pages"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("pages_updated_by_id_6c42de3e_fk_users_id").from(Alias::new("pages"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("pages_owned_by_id_bf50485f_fk_users_id").from(Alias::new("pages"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("pages_parent_id_8b823409_fk_pages_id").from(Alias::new("pages"), Alias::new("parent_id")).to(Alias::new("pages"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("pages_workspace_id_c6c51010_fk_workspaces_id").from(Alias::new("pages"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── page_labels / page_logs / page_versions / project_pages ───────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("page_labels"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("label_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("page_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("page_labels_created_by_id_fbd942c0_fk_users_id").from(Alias::new("page_labels"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_labels_updated_by_id_d9fddbff_fk_users_id").from(Alias::new("page_labels"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_labels_label_id_05958e53_fk_labels_id").from(Alias::new("page_labels"), Alias::new("label_id")).to(Alias::new("labels"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_labels_page_id_0e6cdb3d_fk_pages_id").from(Alias::new("page_labels"), Alias::new("page_id")).to(Alias::new("pages"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_labels_workspace_id_078bb01c_fk_workspaces_id").from(Alias::new("page_labels"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("page_logs"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("transaction")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("entity_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("entity_name")).string_len(30).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("page_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("entity_type")).string_len(30).null())
                    .foreign_key(ForeignKey::create().name("page_logs_created_by_id_4a295aec_fk_users_id").from(Alias::new("page_logs"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_logs_updated_by_id_1995190b_fk_users_id").from(Alias::new("page_logs"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_logs_page_id_0e0d747d_fk_pages_id").from(Alias::new("page_logs"), Alias::new("page_id")).to(Alias::new("pages"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_logs_workspace_id_be7bde64_fk_workspaces_id").from(Alias::new("page_logs"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("page_logs_page_id_transaction_9ab05334_uniq").table(Alias::new("page_logs")).col(Alias::new("page_id")).col(Alias::new("transaction")).to_owned()).await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("page_versions"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("last_saved_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("description_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owned_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("page_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("sub_pages_data")).json_binary().not_null())
                    .foreign_key(ForeignKey::create().name("page_versions_created_by_id_d660b13b_fk_users_id").from(Alias::new("page_versions"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_versions_updated_by_id_72d5e579_fk_users_id").from(Alias::new("page_versions"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_versions_owned_by_id_6d9143db_fk_users_id").from(Alias::new("page_versions"), Alias::new("owned_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_versions_page_id_c46471da_fk_pages_id").from(Alias::new("page_versions"), Alias::new("page_id")).to(Alias::new("pages"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("page_versions_workspace_id_8330a200_fk_workspaces_id").from(Alias::new("page_versions"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_pages"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("page_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("project_pages_created_by_id_b9d02062_fk_users_id").from(Alias::new("project_pages"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("project_pages_updated_by_id_b80bf0f4_fk_users_id").from(Alias::new("project_pages"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("project_pages_page_id_a0f54439_fk_pages_id").from(Alias::new("project_pages"), Alias::new("page_id")).to(Alias::new("pages"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("project_pages_project_id_376ba35a_fk_projects_id").from(Alias::new("project_pages"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("project_pages_workspace_id_13ed9e73_fk_workspaces_id").from(Alias::new("project_pages"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("project_pages_project_id_page_id_deleted_at_7c80a40c_uniq").table(Alias::new("project_pages")).col(Alias::new("project_id")).col(Alias::new("page_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── notifications ─────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("notifications"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("data")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("entity_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("entity_name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("title")).text().not_null())
                    .col(ColumnDef::new(Alias::new("message")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("message_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("message_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("sender")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("read_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("snoozed_till")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("archived_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("receiver_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("triggered_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("notifications_created_by_id_b9c3f81b_fk_users_id").from(Alias::new("notifications"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("notifications_updated_by_id_8a651e96_fk_users_id").from(Alias::new("notifications"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("notifications_receiver_id_b708b2b0_fk_users_id").from(Alias::new("notifications"), Alias::new("receiver_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("notifications_triggered_by_id_31cdec21_fk_users_id").from(Alias::new("notifications"), Alias::new("triggered_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("notifications_project_id_e4d4f192_fk_projects_id").from(Alias::new("notifications"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("notifications_workspace_id_b2f09ef7_fk_workspaces_id").from(Alias::new("notifications"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── user_notification_preferences ─────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("user_notification_preferences"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("property_change")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("state_change")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("comment")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("mention")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("issue_completed")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("user_notification_pr_created_by_id_54dc743a_fk_users_id").from(Alias::new("user_notification_preferences"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_notification_pr_updated_by_id_eb70a86d_fk_users_id").from(Alias::new("user_notification_preferences"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_notification_preferences_user_id_9dccc056_fk_users_id").from(Alias::new("user_notification_preferences"), Alias::new("user_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_notification_pr_project_id_e0ca17f8_fk_projects_").from(Alias::new("user_notification_preferences"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_notification_pr_workspace_id_a2321c58_fk_workspace").from(Alias::new("user_notification_preferences"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── user_favorites ────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("user_favorites"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("entity_type")).string_len(100).not_null())
                    .col(ColumnDef::new(Alias::new("entity_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("is_folder")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("sequence")).double().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("user_favorites_created_by_id_dc025309_fk_users_id").from(Alias::new("user_favorites"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_favorites_updated_by_id_a1a5ac4a_fk_users_id").from(Alias::new("user_favorites"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_favorites_user_id_cea7e2d2_fk_users_id").from(Alias::new("user_favorites"), Alias::new("user_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_favorites_parent_id_550512e4_fk_user_favorites_id").from(Alias::new("user_favorites"), Alias::new("parent_id")).to(Alias::new("user_favorites"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_favorites_project_id_359b527f_fk_projects_id").from(Alias::new("user_favorites"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_favorites_workspace_id_aa90f680_fk_workspaces_id").from(Alias::new("user_favorites"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("user_favorites_entity_type_user_id_enti_22b103ff_uniq").table(Alias::new("user_favorites")).col(Alias::new("entity_type")).col(Alias::new("user_id")).col(Alias::new("entity_identifier")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── user_recent_visits ────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("user_recent_visits"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("entity_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("entity_name")).string_len(30).not_null())
                    .col(ColumnDef::new(Alias::new("visited_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("user_recent_visits_created_by_id_a655b75f_fk_users_id").from(Alias::new("user_recent_visits"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_recent_visits_updated_by_id_42b12ef2_fk_users_id").from(Alias::new("user_recent_visits"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_recent_visits_user_id_f5153288_fk_users_id").from(Alias::new("user_recent_visits"), Alias::new("user_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_recent_visits_project_id_e5eecf27_fk_projects_id").from(Alias::new("user_recent_visits"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("user_recent_visits_workspace_id_362a4e80_fk_workspaces_id").from(Alias::new("user_recent_visits"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── stickies ──────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("stickies"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).text().null())
                    .col(ColumnDef::new(Alias::new("description")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("color")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("background_color")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owner_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .foreign_key(ForeignKey::create().name("stickies_created_by_id_f72e05c4_fk_users_id").from(Alias::new("stickies"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("stickies_updated_by_id_d660f1fb_fk_users_id").from(Alias::new("stickies"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("stickies_owner_id_6ee3be2b_fk_users_id").from(Alias::new("stickies"), Alias::new("owner_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("stickies_workspace_id_0094496a_fk_workspaces_id").from(Alias::new("stickies"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── teams ─────────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("teams"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("logo_props")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("team_created_by_id_725a9101_fk_user_id").from(Alias::new("teams"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("team_updated_by_id_79bb36f2_fk_user_id").from(Alias::new("teams"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("team_workspace_id_1d56407f_fk_workspace_id").from(Alias::new("teams"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        manager.create_index(Index::create().unique().name("teams_name_workspace_id_deleted_at_4b131aa2_uniq").table(Alias::new("teams")).col(Alias::new("name")).col(Alias::new("workspace_id")).col(Alias::new("deleted_at")).to_owned()).await?;

        // ── email_notification_logs ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("email_notification_logs"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("entity_identifier")).uuid().null())
                    .col(ColumnDef::new(Alias::new("entity_name")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("data")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("processed_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("sent_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("entity")).string_len(200).not_null())
                    .col(ColumnDef::new(Alias::new("old_value")).string_len(300).null())
                    .col(ColumnDef::new(Alias::new("new_value")).string_len(300).null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("receiver_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("triggered_by_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .foreign_key(ForeignKey::create().name("email_notification_logs_created_by_id_6faff587_fk_users_id").from(Alias::new("email_notification_logs"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("email_notification_logs_updated_by_id_5d99c798_fk_users_id").from(Alias::new("email_notification_logs"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("email_notification_logs_receiver_id_7c7d2e13_fk_users_id").from(Alias::new("email_notification_logs"), Alias::new("receiver_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("email_notification_logs_triggered_by_id_b551e727_fk_users_id").from(Alias::new("email_notification_logs"), Alias::new("triggered_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        // ── draft_issues and related ──────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("draft_issues"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("description_json")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("description_html")).text().not_null())
                    .col(ColumnDef::new(Alias::new("description_stripped")).text().null())
                    .col(ColumnDef::new(Alias::new("description_binary")).binary().null())
                    .col(ColumnDef::new(Alias::new("priority")).string_len(30).not_null())
                    .col(ColumnDef::new(Alias::new("start_date")).date().null())
                    .col(ColumnDef::new(Alias::new("target_date")).date().null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("completed_at")).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(Alias::new("external_source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("external_id")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("estimate_point_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("state_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("type_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(ForeignKey::create().name("draft_issues_created_by_id_aedba72a_fk_users_id").from(Alias::new("draft_issues"), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_updated_by_id_1ca3cd4e_fk_users_id").from(Alias::new("draft_issues"), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_estimate_point_id_9e333189_fk_estimate_points_id").from(Alias::new("draft_issues"), Alias::new("estimate_point_id")).to(Alias::new("estimate_points"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_parent_id_eee6ec32_fk_issues_id").from(Alias::new("draft_issues"), Alias::new("parent_id")).to(Alias::new("issues"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_project_id_784a560c_fk_projects_id").from(Alias::new("draft_issues"), Alias::new("project_id")).to(Alias::new("projects"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_state_id_94f28f5a_fk_states_id").from(Alias::new("draft_issues"), Alias::new("state_id")).to(Alias::new("states"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_type_id_7a62fe34_fk_issue_types_id").from(Alias::new("draft_issues"), Alias::new("type_id")).to(Alias::new("issue_types"), Alias::new("id")))
                    .foreign_key(ForeignKey::create().name("draft_issues_workspace_id_9d8512c8_fk_workspaces_id").from(Alias::new("draft_issues"), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                    .to_owned(),
            )
            .await?;

        for tbl in ["draft_issue_assignees", "draft_issue_cycles", "draft_issue_labels", "draft_issue_modules"] {
            let is_assignees = tbl == "draft_issue_assignees";
            let is_cycles = tbl == "draft_issue_cycles";
            let is_labels = tbl == "draft_issue_labels";

            let rel_col = if is_assignees { "assignee_id" } else if is_cycles { "cycle_id" } else if is_labels { "label_id" } else { "module_id" };
            let rel_tbl = if is_assignees { "users" } else if is_cycles { "cycles" } else if is_labels { "labels" } else { "modules" };
            let fk_suffix = if is_assignees { "assignee_id_9cc52f9d" } else if is_cycles { "cycle_id_b214e11f" } else if is_labels { "label_id_b9b001a5" } else { "module_id_4d3f477a" };

            manager
                .create_table(
                    Table::create()
                        .table(Alias::new(tbl))
                        .if_not_exists()
                        .col(ColumnDef::new(Alias::new("created_at")).timestamp_with_time_zone().not_null())
                        .col(ColumnDef::new(Alias::new("updated_at")).timestamp_with_time_zone().not_null())
                        .col(ColumnDef::new(Alias::new("deleted_at")).timestamp_with_time_zone().null())
                        .col(ColumnDef::new(Alias::new("id")).uuid().not_null().primary_key())
                        .col(ColumnDef::new(Alias::new(rel_col)).uuid().not_null())
                        .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                        .col(ColumnDef::new(Alias::new("draft_issue_id")).uuid().not_null())
                        .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                        .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                        .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                        .foreign_key(ForeignKey::create().name(format!("{}_created_by", tbl)).from(Alias::new(tbl), Alias::new("created_by_id")).to(Alias::new("users"), Alias::new("id")))
                        .foreign_key(ForeignKey::create().name(format!("{}_updated_by", tbl)).from(Alias::new(tbl), Alias::new("updated_by_id")).to(Alias::new("users"), Alias::new("id")))
                        .foreign_key(ForeignKey::create().name(format!("{}_{}", tbl, fk_suffix)).from(Alias::new(tbl), Alias::new(rel_col)).to(Alias::new(rel_tbl), Alias::new("id")))
                        .foreign_key(ForeignKey::create().name(format!("{}_draft_issue_id", tbl)).from(Alias::new(tbl), Alias::new("draft_issue_id")).to(Alias::new("draft_issues"), Alias::new("id")))
                        .foreign_key(ForeignKey::create().name(format!("{}_workspace_id", tbl)).from(Alias::new(tbl), Alias::new("workspace_id")).to(Alias::new("workspaces"), Alias::new("id")))
                        .to_owned(),
                )
                .await?;

            // Indexes
            for col in ["created_by_id", "updated_by_id", "draft_issue_id", "project_id", "workspace_id", rel_col] {
                manager
                    .create_index(
                        Index::create()
                            .name(format!("{}_{}", tbl, col))
                            .table(Alias::new(tbl))
                            .col(Alias::new(col))
                            .to_owned(),
                    )
                    .await?;
            }

            // Partial unique constraints (WHERE deleted_at IS NULL)
            let uniq_name = if is_assignees {
                "draft_issue_assignee_unique_issue_assignee_when_deleted_at_null"
            } else if is_cycles {
                "draft_issue_cycle_when_deleted_at_null"
            } else if is_labels {
                "" // no unique constraint for labels in baseline
            } else {
                "module_draft_issue_unique_issue_module_when_deleted_at_null"
            };

            if !uniq_name.is_empty() {
                manager
                    .get_connection()
                    .execute_unprepared(&format!(
                        "CREATE UNIQUE INDEX IF NOT EXISTS {} ON {} (draft_issue_id, {}) WHERE deleted_at IS NULL",
                        uniq_name, tbl, rel_col
                    ))
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "draft_issue_modules", "draft_issue_labels", "draft_issue_cycles", "draft_issue_assignees",
            "draft_issues", "email_notification_logs", "teams", "stickies",
            "user_recent_visits", "user_favorites", "user_notification_preferences",
            "notifications", "project_pages", "page_versions", "page_logs", "page_labels", "pages",
            "module_user_properties", "module_members", "module_links", "module_issues", "modules",
            "cycle_user_properties", "cycle_issues", "cycles",
            "comment_reactions", "issue_versions", "issue_description_versions",
            "issue_views", "issue_votes", "issue_subscribers", "issue_sequences",
            "issue_relations", "issue_reactions", "issue_mentions", "issue_links",
            "issue_labels", "issue_blockers", "issue_attachments",
            "issue_comments", "issue_activities", "issue_assignees", "issues", "analytic_views",
        ] {
            manager.drop_table(Table::drop().table(Alias::new(table)).if_exists().cascade().to_owned()).await?;
        }
        Ok(())
    }
}