use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000004_projects_and_states"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // issue_types (workspace-level, no project FK yet)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("issue_types"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("logo_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_default"))
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("level")).double().not_null())
                    .col(
                        ColumnDef::new(Alias::new("external_id"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("external_source"))
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_epic")).boolean().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_types_created_by_id_48764f53_fk_users_id")
                            .from(Alias::new("issue_types"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_types_updated_by_id_4919203b_fk_users_id")
                            .from(Alias::new("issue_types"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_types_workspace_id_591c6f3b_fk_workspaces_id")
                            .from(Alias::new("issue_types"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("issue_types_workspace_id_591c6f3b")
                    .table(Alias::new("issue_types"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // projects - depends on workspaces, users, file_assets, states (states need projects first - circular)
        // Create projects WITHOUT state/estimate FKs, add them after
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("projects"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("description_text"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_html"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("network"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("identifier"))
                            .string_len(12)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("default_assignee_id"))
                            .uuid()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_lead_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("emoji")).string_len(255).null())
                    .col(
                        ColumnDef::new(Alias::new("cycle_view"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("module_view"))
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("cover_image")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("issue_views_view"))
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("page_view")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("estimate_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("icon_prop")).json_binary().null())
                    .col(
                        ColumnDef::new(Alias::new("intake_view"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("archive_in"))
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("close_in")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("default_state_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("logo_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("archived_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_time_tracking_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_issue_type_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("guest_view_all_features"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("timezone"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("cover_image_asset_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("external_id"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("external_source"))
                            .string_len(255)
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_created_by_id_6cc13408_fk_user_id")
                            .from(Alias::new("projects"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_default_assignee_id_6ba45f90_fk_user_id")
                            .from(Alias::new("projects"), Alias::new("default_assignee_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_project_lead_id_caf8e353_fk_user_id")
                            .from(Alias::new("projects"), Alias::new("project_lead_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_updated_by_id_fe290525_fk_user_id")
                            .from(Alias::new("projects"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_workspace_id_01764ff9_fk_workspace_id")
                            .from(Alias::new("projects"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("projects_cover_image_asset_id_e6636b92_fk_file_assets_id")
                            .from(Alias::new("projects"), Alias::new("cover_image_asset_id"))
                            .to(Alias::new("file_assets"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_unique_name_workspace_when_deleted_at_null
                    ON projects (name, workspace_id) WHERE deleted_at IS NULL;
                CREATE UNIQUE INDEX project_unique_identifier_workspace_when_deleted_at_null
                    ON projects (identifier, workspace_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_workspace_id_01764ff9")
                    .table(Alias::new("projects"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("projects_identifier_3267ade8")
                    .table(Alias::new("projects"))
                    .col(Alias::new("identifier"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("projects_cover_image_asset_id_e6636b92")
                    .table(Alias::new("projects"))
                    .col(Alias::new("cover_image_asset_id"))
                    .to_owned(),
            )
            .await?;

        // states (depends on projects, users, workspaces)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("states"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("color"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("slug"))
                            .string_len(100)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("sequence")).double().not_null())
                    .col(
                        ColumnDef::new(Alias::new("group"))
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("default")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("external_id"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("external_source"))
                            .string_len(255)
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_triage")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("state_created_by_id_ff51a50d_fk_user_id")
                            .from(Alias::new("states"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("state_project_id_23a65fd6_fk_project_id")
                            .from(Alias::new("states"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("state_updated_by_id_be298453_fk_user_id")
                            .from(Alias::new("states"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("state_workspace_id_2293282d_fk_workspace_id")
                            .from(Alias::new("states"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX state_unique_name_project_when_deleted_at_null
                    ON states (name, project_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("state_project_id_23a65fd6")
                    .table(Alias::new("states"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("state_slug_bab0af35")
                    .table(Alias::new("states"))
                    .col(Alias::new("slug"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("state_workspace_id_2293282d")
                    .table(Alias::new("states"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // Now add deferred FKs from projects -> states and projects -> estimates
        // (estimates table will be added below)
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("projects_default_state_id_f13e8b95_fk_states_id")
                    .from(Alias::new("projects"), Alias::new("default_state_id"))
                    .to(Alias::new("states"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("projects_default_state_id_f13e8b95")
                    .table(Alias::new("projects"))
                    .col(Alias::new("default_state_id"))
                    .to_owned(),
            )
            .await?;

        // labels (depends on projects, users, workspaces)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("labels"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("parent_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("color"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(
                        ColumnDef::new(Alias::new("external_id"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("external_source"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("label_created_by_id_aa6ffcfa_fk_user_id")
                            .from(Alias::new("labels"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("labels_project_id_cf57a802_fk_projects_id")
                            .from(Alias::new("labels"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("label_updated_by_id_894a5464_fk_user_id")
                            .from(Alias::new("labels"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("label_workspace_id_c4c9ae5a_fk_workspace_id")
                            .from(Alias::new("labels"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("label_parent_id_7a853296_fk_label_id")
                            .from(Alias::new("labels"), Alias::new("parent_id"))
                            .to(Alias::new("labels"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX unique_name_when_project_null_and_not_deleted
                    ON labels (name) WHERE deleted_at IS NULL AND project_id IS NULL;
                CREATE UNIQUE INDEX unique_project_name_when_not_deleted
                    ON labels (project_id, name) WHERE deleted_at IS NULL AND project_id IS NOT NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("label_project_id_90e0f1a2")
                    .table(Alias::new("labels"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("label_parent_id_7a853296")
                    .table(Alias::new("labels"))
                    .col(Alias::new("parent_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("label_workspace_id_c4c9ae5a")
                    .table(Alias::new("labels"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // estimates (depends on projects, users, workspaces)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("estimates"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("type"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("last_used")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimates_created_by_id_7e401493_fk_users_id")
                            .from(Alias::new("estimates"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimates_project_id_7f195a41_fk_projects_id")
                            .from(Alias::new("estimates"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimates_updated_by_id_b3fcfb1d_fk_users_id")
                            .from(Alias::new("estimates"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimates_workspace_id_718811eb_fk_workspaces_id")
                            .from(Alias::new("estimates"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX estimate_unique_name_project_when_deleted_at_null
                    ON estimates (name, project_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("estimates_project_id_7f195a41")
                    .table(Alias::new("estimates"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        // estimate_points
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("estimate_points"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("key")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("value"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("estimate_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimate_points_created_by_id_d1b04bd9_fk_users_id")
                            .from(Alias::new("estimate_points"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimate_points_estimate_id_4b4cb706_fk_estimates_id")
                            .from(Alias::new("estimate_points"), Alias::new("estimate_id"))
                            .to(Alias::new("estimates"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimate_points_project_id_ba9bcb2c_fk_projects_id")
                            .from(Alias::new("estimate_points"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimate_points_updated_by_id_a1da94e1_fk_users_id")
                            .from(Alias::new("estimate_points"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("estimate_points_workspace_id_96fc4f92_fk_workspaces_id")
                            .from(Alias::new("estimate_points"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("estimate_points_estimate_id_4b4cb706")
                    .table(Alias::new("estimate_points"))
                    .col(Alias::new("estimate_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("estimate_points_project_id_ba9bcb2c")
                    .table(Alias::new("estimate_points"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        // Now add estimate FK to projects
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("projects_estimate_id_85c7b2ac_fk_estimates_id")
                    .from(Alias::new("projects"), Alias::new("estimate_id"))
                    .to(Alias::new("estimates"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("projects_estimate_id_85c7b2ac")
                    .table(Alias::new("projects"))
                    .col(Alias::new("estimate_id"))
                    .to_owned(),
            )
            .await?;

        // project_identifiers
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_identifiers"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("name")).string_len(12).not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("project_id"))
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_identifier_created_by_id_2b6f273a_fk_user_id")
                            .from(
                                Alias::new("project_identifiers"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_identifier_project_id_13de58a9_fk_project_id")
                            .from(Alias::new("project_identifiers"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_identifier_updated_by_id_1a00e2a0_fk_user_id")
                            .from(
                                Alias::new("project_identifiers"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_identifier_workspace_id_6024b517_fk_workspace_id")
                            .from(
                                Alias::new("project_identifiers"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX unique_name_workspace_when_deleted_at_null
                    ON project_identifiers (name, workspace_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_identifiers_name_6ca8a4b0")
                    .table(Alias::new("project_identifiers"))
                    .col(Alias::new("name"))
                    .to_owned(),
            )
            .await?;

        // project_members
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_members"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("comment")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("role"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("member_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("view_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("default_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(
                        ColumnDef::new(Alias::new("preferences"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_created_by_id_8b363306_fk_user_id")
                            .from(Alias::new("project_members"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_member_id_9d6b126b_fk_user_id")
                            .from(Alias::new("project_members"), Alias::new("member_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_project_id_11ea1a9e_fk_project_id")
                            .from(Alias::new("project_members"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_updated_by_id_cf6aaac4_fk_user_id")
                            .from(Alias::new("project_members"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_workspace_id_88bb9a97_fk_workspace_id")
                            .from(Alias::new("project_members"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_member_unique_project_member_when_deleted_at_null
                    ON project_members (project_id, member_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_member_member_id_9d6b126b")
                    .table(Alias::new("project_members"))
                    .col(Alias::new("member_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_member_project_id_11ea1a9e")
                    .table(Alias::new("project_members"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        // project_member_invites
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_member_invites"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("email"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("accepted")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("token"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("message")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("responded_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("role"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_invite_created_by_id_a87df45c_fk_user_id")
                            .from(
                                Alias::new("project_member_invites"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_invite_project_id_8fb7750e_fk_project_id")
                            .from(
                                Alias::new("project_member_invites"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_invite_updated_by_id_5aa55c96_fk_user_id")
                            .from(
                                Alias::new("project_member_invites"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_member_invite_workspace_id_64e2dc4c_fk_workspace_id")
                            .from(
                                Alias::new("project_member_invites"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_member_invite_project_id_8fb7750e")
                    .table(Alias::new("project_member_invites"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        // project_public_members
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_public_members"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("member_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_public_members_created_by_id_c4c7c776_fk_users_id")
                            .from(
                                Alias::new("project_public_members"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_public_members_member_id_52f257f9_fk_users_id")
                            .from(
                                Alias::new("project_public_members"),
                                Alias::new("member_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_public_members_project_id_2dfd893d_fk_projects_id")
                            .from(
                                Alias::new("project_public_members"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_public_members_updated_by_id_c3e4d675_fk_users_id")
                            .from(
                                Alias::new("project_public_members"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_public_members_workspace_id_ebfce110_fk_workspaces_id")
                            .from(
                                Alias::new("project_public_members"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_public_member_unique_project_member_when_deleted_at_nul
                    ON project_public_members (project_id, member_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        // project_issue_types
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_issue_types"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("level")).integer().not_null())
                    .col(
                        ColumnDef::new(Alias::new("is_default"))
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("issue_type_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_issue_types_created_by_id_049cecfd_fk_users_id")
                            .from(
                                Alias::new("project_issue_types"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_issue_types_issue_type_id_9494de9f_fk_issue_types_id")
                            .from(
                                Alias::new("project_issue_types"),
                                Alias::new("issue_type_id"),
                            )
                            .to(Alias::new("issue_types"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_issue_types_project_id_ef6e52e4_fk_projects_id")
                            .from(Alias::new("project_issue_types"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_issue_types_updated_by_id_b5998397_fk_users_id")
                            .from(
                                Alias::new("project_issue_types"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_issue_types_workspace_id_ace3c5b5_fk_workspaces_id")
                            .from(
                                Alias::new("project_issue_types"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_issue_type_unique_project_issue_type_when_deleted_at_nu
                    ON project_issue_types (project_id, issue_type_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_issue_types_project_id_ef6e52e4")
                    .table(Alias::new("project_issue_types"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_issue_types_issue_type_id_9494de9f")
                    .table(Alias::new("project_issue_types"))
                    .col(Alias::new("issue_type_id"))
                    .to_owned(),
            )
            .await?;

        // project_user_properties
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_user_properties"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("display_properties"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("display_filters"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("filters"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("rich_filters"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("preferences"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_property_created_by_id_8e92131c_fk_user_id")
                            .from(
                                Alias::new("project_user_properties"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_property_project_id_30e7de7b_fk_project_id")
                            .from(
                                Alias::new("project_user_properties"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_property_updated_by_id_ff158d4d_fk_user_id")
                            .from(
                                Alias::new("project_user_properties"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_property_user_id_0b1d1c8f_fk_user_id")
                            .from(Alias::new("project_user_properties"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("issue_property_workspace_id_17860d65_fk_workspace_id")
                            .from(
                                Alias::new("project_user_properties"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_user_property_unique_user_project_when_deleted_at_null
                    ON project_user_properties (user_id, project_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("issue_property_project_id_30e7de7b")
                    .table(Alias::new("project_user_properties"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("issue_property_user_id_0b1d1c8f")
                    .table(Alias::new("project_user_properties"))
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        // project_webhooks
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_webhooks"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("created_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("updated_at"))
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("webhook_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_webhooks_created_by_id_c3e4bfa3_fk_users_id")
                            .from(Alias::new("project_webhooks"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_webhooks_project_id_bec3cf8c_fk_projects_id")
                            .from(Alias::new("project_webhooks"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_webhooks_updated_by_id_a0183aeb_fk_users_id")
                            .from(Alias::new("project_webhooks"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_webhooks_webhook_id_da27c6a7_fk_webhooks_id")
                            .from(Alias::new("project_webhooks"), Alias::new("webhook_id"))
                            .to(Alias::new("webhooks"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("project_webhooks_workspace_id_429ebf05_fk_workspaces_id")
                            .from(Alias::new("project_webhooks"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX project_webhook_unique_project_webhook_when_deleted_at_null
                    ON project_webhooks (project_id, webhook_id) WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_webhooks_project_id_bec3cf8c")
                    .table(Alias::new("project_webhooks"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("project_webhooks_webhook_id_da27c6a7")
                    .table(Alias::new("project_webhooks"))
                    .col(Alias::new("webhook_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "project_webhooks",
            "project_user_properties",
            "project_issue_types",
            "project_public_members",
            "project_member_invites",
            "project_members",
            "project_identifiers",
            "estimate_points",
            "estimates",
            "labels",
            "states",
            "projects",
            "issue_types",
        ] {
            manager
                .drop_table(
                    Table::drop()
                        .table(Alias::new(table))
                        .if_exists()
                        .cascade()
                        .to_owned(),
                )
                .await?;
        }
        Ok(())
    }
}
