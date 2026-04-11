use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000006_integrations_and_misc"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── descriptions ──────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("descriptions"))
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
                    .col(
                        ColumnDef::new(Alias::new("description_json"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_html"))
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_binary"))
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_stripped"))
                            .text()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("descriptions_created_by_id_b88ab399_fk_users_id")
                            .from(Alias::new("descriptions"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("descriptions_updated_by_id_af519c4d_fk_users_id")
                            .from(Alias::new("descriptions"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("descriptions_project_id_8f46180b_fk_projects_id")
                            .from(Alias::new("descriptions"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("descriptions_workspace_id_767279bf_fk_workspaces_id")
                            .from(Alias::new("descriptions"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── description_versions ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("description_versions"))
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
                    .col(
                        ColumnDef::new(Alias::new("description_json"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_html"))
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_binary"))
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("description_stripped"))
                            .text()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("description_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("description_versions_created_by_id_6633a3de_fk_users_id")
                            .from(
                                Alias::new("description_versions"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("description_versions_updated_by_id_8b5179ae_fk_users_id")
                            .from(
                                Alias::new("description_versions"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("description_versions_description_id_dc7f19b6_fk_descriptions_id")
                            .from(
                                Alias::new("description_versions"),
                                Alias::new("description_id"),
                            )
                            .to(Alias::new("descriptions"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("description_versions_project_id_1a6c9aa9_fk_projects_id")
                            .from(Alias::new("description_versions"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("description_versions_workspace_id_52857186_fk_workspaces_id")
                            .from(
                                Alias::new("description_versions"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // Now add FK from issue_comments.description_id -> descriptions
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("issue_comments_description_id_0cb72512_fk_descriptions_id")
                    .from(Alias::new("issue_comments"), Alias::new("description_id"))
                    .to(Alias::new("descriptions"), Alias::new("id"))
                    .to_owned(),
            )
            .await?;

        // ── intakes ───────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("intakes"))
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
                        ColumnDef::new(Alias::new("is_default"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("view_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("logo_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inboxes_created_by_id_9f1cf5ec_fk_users_id")
                            .from(Alias::new("intakes"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inboxes_updated_by_id_69b7b3ae_fk_users_id")
                            .from(Alias::new("intakes"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inboxes_project_id_a0135c66_fk_projects_id")
                            .from(Alias::new("intakes"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inboxes_workspace_id_d6178865_fk_workspaces_id")
                            .from(Alias::new("intakes"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("inboxes_name_project_id_deleted_at_95043f72_uniq")
                    .table(Alias::new("intakes"))
                    .col(Alias::new("name"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("deleted_at"))
                    .to_owned(),
            )
            .await?;

        // ── intake_issues ─────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("intake_issues"))
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
                    .col(ColumnDef::new(Alias::new("status")).integer().not_null())
                    .col(
                        ColumnDef::new(Alias::new("snoozed_till"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("source")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("duplicate_to_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("intake_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
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
                    .col(ColumnDef::new(Alias::new("extra")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("source_email")).text().null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_created_by_id_483bce13_fk_users_id")
                            .from(Alias::new("intake_issues"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_updated_by_id_d1b2b70f_fk_users_id")
                            .from(Alias::new("intake_issues"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_duplicate_to_id_6cb8d961_fk_issues_id")
                            .from(Alias::new("intake_issues"), Alias::new("duplicate_to_id"))
                            .to(Alias::new("issues"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_intake_id_a04a7455_fk_intakes_id")
                            .from(Alias::new("intake_issues"), Alias::new("intake_id"))
                            .to(Alias::new("intakes"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_issue_id_7d74b224_fk_issues_id")
                            .from(Alias::new("intake_issues"), Alias::new("issue_id"))
                            .to(Alias::new("issues"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_project_id_5117a70b_fk_projects_id")
                            .from(Alias::new("intake_issues"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("inbox_issues_workspace_id_4a61a7bd_fk_workspaces_id")
                            .from(Alias::new("intake_issues"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── deploy_boards ─────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("deploy_boards"))
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
                        ColumnDef::new(Alias::new("entity_identifier"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("entity_name"))
                            .string_len(30)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("anchor"))
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_comments_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_reactions_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_votes_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("view_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("intake_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_activity_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_disabled"))
                            .boolean()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("deploy_boards_created_by_id_149dff93_fk_users_id")
                            .from(Alias::new("deploy_boards"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("deploy_boards_updated_by_id_db7ae24f_fk_users_id")
                            .from(Alias::new("deploy_boards"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("deploy_boards_intake_id_76a6470a_fk_intakes_id")
                            .from(Alias::new("deploy_boards"), Alias::new("intake_id"))
                            .to(Alias::new("intakes"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("deploy_boards_project_id_cfc792a1_fk_projects_id")
                            .from(Alias::new("deploy_boards"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("deploy_boards_workspace_id_fcf03158_fk_workspaces_id")
                            .from(Alias::new("deploy_boards"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("deploy_boards_entity_name_entity_ident_800ce160_uniq")
                    .table(Alias::new("deploy_boards"))
                    .col(Alias::new("entity_name"))
                    .col(Alias::new("entity_identifier"))
                    .col(Alias::new("deleted_at"))
                    .to_owned(),
            )
            .await?;

        // ── project_deploy_boards ─────────────────────────────────────────────
        // (already created in migration 4, but FK to intakes is deferred here)
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("project_deploy_boards_intake_id_36aa612d_fk_intakes_id")
                    .from(Alias::new("project_deploy_boards"), Alias::new("intake_id"))
                    .to(Alias::new("intakes"), Alias::new("id"))
                    .to_owned(),
            )
            .await?;

        // ── exporters ─────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("exporters"))
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
                    .col(ColumnDef::new(Alias::new("project")).custom(Alias::new("uuid[]")).null())
                    .col(
                        ColumnDef::new(Alias::new("provider"))
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("reason")).text().not_null())
                    .col(ColumnDef::new(Alias::new("key")).text().not_null())
                    .col(ColumnDef::new(Alias::new("url")).string_len(800).null())
                    .col(
                        ColumnDef::new(Alias::new("token"))
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("initiated_by_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("filters")).json_binary().null())
                    .col(ColumnDef::new(Alias::new("name")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("type")).string_len(50).not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("rich_filters"))
                            .json_binary()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("exporters_created_by_id_44e1d9b3_fk_users_id")
                            .from(Alias::new("exporters"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("exporters_initiated_by_id_d51f7552_fk_users_id")
                            .from(Alias::new("exporters"), Alias::new("initiated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("exporters_updated_by_id_d2572861_fk_users_id")
                            .from(Alias::new("exporters"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("exporters_workspace_id_11a04317_fk_workspaces_id")
                            .from(Alias::new("exporters"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── importers ─────────────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("importers"))
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
                        ColumnDef::new(Alias::new("service"))
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("status"))
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("metadata"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("data")).json_binary().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("initiated_by_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("token_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("imported_data"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_created_by_id_7dd06433_fk_users_id")
                            .from(Alias::new("importers"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_initiated_by_id_3cddbd23_fk_users_id")
                            .from(Alias::new("importers"), Alias::new("initiated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_updated_by_id_3915139e_fk_users_id")
                            .from(Alias::new("importers"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_token_id_c951e89f_fk_api_tokens_id")
                            .from(Alias::new("importers"), Alias::new("token_id"))
                            .to(Alias::new("api_tokens"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_project_id_1f8b43ef_fk_projects_id")
                            .from(Alias::new("importers"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("importers_workspace_id_795b8985_fk_workspaces_id")
                            .from(Alias::new("importers"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── github_repositories ───────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("github_repositories"))
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
                            .string_len(500)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("url")).string_len(200).null())
                    .col(
                        ColumnDef::new(Alias::new("config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("repository_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("owner"))
                            .string_len(500)
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
                            .name("github_repositories_created_by_id_104fa685_fk_users_id")
                            .from(
                                Alias::new("github_repositories"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repositories_updated_by_id_8aa4d772_fk_users_id")
                            .from(
                                Alias::new("github_repositories"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repositories_project_id_65c546bb_fk_projects_id")
                            .from(Alias::new("github_repositories"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repositories_workspace_id_c4de7326_fk_workspaces_id")
                            .from(
                                Alias::new("github_repositories"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── github_repository_syncs ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("github_repository_syncs"))
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
                        ColumnDef::new(Alias::new("credentials"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("label_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("repository_id"))
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("workspace_integration_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_actor_id_1fa689fe_fk_users_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("actor_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_created_by_id_0df94495_fk_users_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_updated_by_id_07e9d065_fk_users_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_label_id_eb1e9bd7_fk_labels_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("label_id"),
                            )
                            .to(Alias::new("labels"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_sy_repository_id_ead52404_fk_github_re")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("repository_id"),
                            )
                            .to(Alias::new("github_repositories"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_project_id_e7e8291e_fk_projects_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_syncs_workspace_id_4a22a8b8_fk_workspaces_id")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_repository_sy_workspace_integratio_62858398_fk_workspace")
                            .from(
                                Alias::new("github_repository_syncs"),
                                Alias::new("workspace_integration_id"),
                            )
                            .to(Alias::new("workspace_integrations"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("github_repository_syncs_project_id_repository_id_0f3705e6_uniq")
                    .table(Alias::new("github_repository_syncs"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("repository_id"))
                    .to_owned(),
            )
            .await?;

        // ── github_issue_syncs ────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("github_issue_syncs"))
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
                        ColumnDef::new(Alias::new("repo_issue_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("github_issue_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("issue_url"))
                            .string_len(200)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("repository_sync_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_created_by_id_d02b7c56_fk_users_id")
                            .from(
                                Alias::new("github_issue_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_updated_by_id_e9cd6f86_fk_users_id")
                            .from(
                                Alias::new("github_issue_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_issue_id_450cb083_fk_issues_id")
                            .from(Alias::new("github_issue_syncs"), Alias::new("issue_id"))
                            .to(Alias::new("issues"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_repository_sync_id_ba0d4de4_fk_github_re")
                            .from(
                                Alias::new("github_issue_syncs"),
                                Alias::new("repository_sync_id"),
                            )
                            .to(Alias::new("github_repository_syncs"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_project_id_4609ad0c_fk_projects_id")
                            .from(Alias::new("github_issue_syncs"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_issue_syncs_workspace_id_eae020ad_fk_workspaces_id")
                            .from(Alias::new("github_issue_syncs"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("github_issue_syncs_repository_sync_id_issue_id_4b34427e_uniq")
                    .table(Alias::new("github_issue_syncs"))
                    .col(Alias::new("repository_sync_id"))
                    .col(Alias::new("issue_id"))
                    .to_owned(),
            )
            .await?;

        // ── github_comment_syncs ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("github_comment_syncs"))
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
                        ColumnDef::new(Alias::new("repo_comment_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("comment_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("issue_sync_id"))
                            .uuid()
                            .not_null(),
                    )
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
                            .name("github_comment_syncs_created_by_id_b1ef2517_fk_users_id")
                            .from(
                                Alias::new("github_comment_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_comment_syncs_updated_by_id_bb05c066_fk_users_id")
                            .from(
                                Alias::new("github_comment_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_comment_syncs_comment_id_6feec6d1_fk_issue_comments_id")
                            .from(Alias::new("github_comment_syncs"), Alias::new("comment_id"))
                            .to(Alias::new("issue_comments"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_comment_syncs_issue_sync_id_5e738eb5_fk_github_is")
                            .from(
                                Alias::new("github_comment_syncs"),
                                Alias::new("issue_sync_id"),
                            )
                            .to(Alias::new("github_issue_syncs"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_comment_syncs_project_id_6d199ace_fk_projects_id")
                            .from(Alias::new("github_comment_syncs"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("github_comment_syncs_workspace_id_b54528c8_fk_workspaces_id")
                            .from(
                                Alias::new("github_comment_syncs"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("github_comment_syncs_issue_sync_id_comment_id_38c82e7b_uniq")
                    .table(Alias::new("github_comment_syncs"))
                    .col(Alias::new("issue_sync_id"))
                    .col(Alias::new("comment_id"))
                    .to_owned(),
            )
            .await?;

        // ── db_githubprstatemapping ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("db_githubprstatemapping"))
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
                    .col(
                        ColumnDef::new(Alias::new("github_pr_state"))
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("workspace_integration_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("state_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("prevent_regression"))
                            .boolean()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("db_githubprstatemapping_created_by_id_381aa92b_fk_users_id")
                            .from(
                                Alias::new("db_githubprstatemapping"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("db_githubprstatemapping_updated_by_id_3330c9ff_fk_users_id")
                            .from(
                                Alias::new("db_githubprstatemapping"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("db_githubprstatemapp_workspace_integratio_2eab555c_fk_workspace")
                            .from(
                                Alias::new("db_githubprstatemapping"),
                                Alias::new("workspace_integration_id"),
                            )
                            .to(Alias::new("workspace_integrations"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("db_githubprstatemapping_project_id_361c0ac2_fk_projects_id")
                            .from(
                                Alias::new("db_githubprstatemapping"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("db_githubprstatemapping_state_id_d2959076_fk_states_id")
                            .from(
                                Alias::new("db_githubprstatemapping"),
                                Alias::new("state_id"),
                            )
                            .to(Alias::new("states"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("db_githubprstatemapping_workspace_integration_id_8a615667_uniq")
                    .table(Alias::new("db_githubprstatemapping"))
                    .col(Alias::new("workspace_integration_id"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("github_pr_state"))
                    .to_owned(),
            )
            .await?;

        // ── gitlab_repositories ───────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("gitlab_repositories"))
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
                            .string_len(500)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("url")).string_len(200).null())
                    .col(
                        ColumnDef::new(Alias::new("config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("repository_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("owner"))
                            .string_len(500)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repositories_created_by_id_1ea7b6cc_fk_users_id")
                            .from(
                                Alias::new("gitlab_repositories"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repositories_updated_by_id_5a30ec6c_fk_users_id")
                            .from(
                                Alias::new("gitlab_repositories"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repositories_project_id_9d20439a_fk_projects_id")
                            .from(Alias::new("gitlab_repositories"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repositories_workspace_id_89900ac2_fk_workspaces_id")
                            .from(
                                Alias::new("gitlab_repositories"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        // ── gitlab_repository_syncs ───────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("gitlab_repository_syncs"))
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
                        ColumnDef::new(Alias::new("credentials"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("label_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("repository_id"))
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("workspace_integration_id"))
                            .uuid()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_actor_id_0f7b2f41_fk_users_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("actor_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_created_by_id_51a64cc3_fk_users_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_updated_by_id_06371794_fk_users_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_label_id_a4aa3f90_fk_labels_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("label_id"),
                            )
                            .to(Alias::new("labels"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_sy_repository_id_0c59ee9a_fk_gitlab_re")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("repository_id"),
                            )
                            .to(Alias::new("gitlab_repositories"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_project_id_9d61576c_fk_projects_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("project_id"),
                            )
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_syncs_workspace_id_e72ee0b4_fk_workspaces_id")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_repository_sy_workspace_integratio_4b878644_fk_workspace")
                            .from(
                                Alias::new("gitlab_repository_syncs"),
                                Alias::new("workspace_integration_id"),
                            )
                            .to(Alias::new("workspace_integrations"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("gitlab_repository_syncs_project_id_repository_id_13a57d00_uniq")
                    .table(Alias::new("gitlab_repository_syncs"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("repository_id"))
                    .to_owned(),
            )
            .await?;

        // ── gitlab_issue_syncs ────────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("gitlab_issue_syncs"))
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
                        ColumnDef::new(Alias::new("repo_issue_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("gitlab_issue_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("issue_url"))
                            .string_len(200)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("issue_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("repository_sync_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_created_by_id_537fc725_fk_users_id")
                            .from(
                                Alias::new("gitlab_issue_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_updated_by_id_89b38a5c_fk_users_id")
                            .from(
                                Alias::new("gitlab_issue_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_issue_id_e97a3f41_fk_issues_id")
                            .from(Alias::new("gitlab_issue_syncs"), Alias::new("issue_id"))
                            .to(Alias::new("issues"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_repository_sync_id_5c48ce08_fk_gitlab_re")
                            .from(
                                Alias::new("gitlab_issue_syncs"),
                                Alias::new("repository_sync_id"),
                            )
                            .to(Alias::new("gitlab_repository_syncs"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_project_id_aff3dc2b_fk_projects_id")
                            .from(Alias::new("gitlab_issue_syncs"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_issue_syncs_workspace_id_b2d0d9da_fk_workspaces_id")
                            .from(Alias::new("gitlab_issue_syncs"), Alias::new("workspace_id"))
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("gitlab_issue_syncs_repository_sync_id_issue_id_14bdcc2c_uniq")
                    .table(Alias::new("gitlab_issue_syncs"))
                    .col(Alias::new("repository_sync_id"))
                    .col(Alias::new("issue_id"))
                    .to_owned(),
            )
            .await?;

        // ── gitlab_comment_syncs ──────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("gitlab_comment_syncs"))
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
                        ColumnDef::new(Alias::new("repo_comment_id"))
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("comment_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("issue_sync_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_created_by_id_b71e6a78_fk_users_id")
                            .from(
                                Alias::new("gitlab_comment_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_updated_by_id_b10af1ed_fk_users_id")
                            .from(
                                Alias::new("gitlab_comment_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_comment_id_ce3343de_fk_issue_comments_id")
                            .from(Alias::new("gitlab_comment_syncs"), Alias::new("comment_id"))
                            .to(Alias::new("issue_comments"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_issue_sync_id_933f43a8_fk_gitlab_is")
                            .from(
                                Alias::new("gitlab_comment_syncs"),
                                Alias::new("issue_sync_id"),
                            )
                            .to(Alias::new("gitlab_issue_syncs"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_project_id_aea53711_fk_projects_id")
                            .from(Alias::new("gitlab_comment_syncs"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("gitlab_comment_syncs_workspace_id_01365c77_fk_workspaces_id")
                            .from(
                                Alias::new("gitlab_comment_syncs"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("gitlab_comment_syncs_issue_sync_id_comment_id_61435f60_uniq")
                    .table(Alias::new("gitlab_comment_syncs"))
                    .col(Alias::new("issue_sync_id"))
                    .col(Alias::new("comment_id"))
                    .to_owned(),
            )
            .await?;

        // ── slack_project_syncs ───────────────────────────────────────────────
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("slack_project_syncs"))
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
                        ColumnDef::new(Alias::new("access_token"))
                            .string_len(300)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("scopes")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("bot_user_id"))
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("webhook_url"))
                            .string_len(1000)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("data")).json_binary().not_null())
                    .col(
                        ColumnDef::new(Alias::new("team_id"))
                            .string_len(30)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("team_name"))
                            .string_len(300)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("workspace_integration_id"))
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("slack_project_syncs_created_by_id_ec405a17_fk_users_id")
                            .from(
                                Alias::new("slack_project_syncs"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("slack_project_syncs_updated_by_id_152eb3b5_fk_users_id")
                            .from(
                                Alias::new("slack_project_syncs"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("slack_project_syncs_project_id_016dc792_fk_projects_id")
                            .from(Alias::new("slack_project_syncs"), Alias::new("project_id"))
                            .to(Alias::new("projects"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("slack_project_syncs_workspace_id_d1822b06_fk_workspaces_id")
                            .from(
                                Alias::new("slack_project_syncs"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("slack_project_syncs_workspace_integratio_d89c9b40_fk_workspace")
                            .from(
                                Alias::new("slack_project_syncs"),
                                Alias::new("workspace_integration_id"),
                            )
                            .to(Alias::new("workspace_integrations"), Alias::new("id")),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .unique()
                    .name("slack_project_syncs_team_id_project_id_50a144a7_uniq")
                    .table(Alias::new("slack_project_syncs"))
                    .col(Alias::new("team_id"))
                    .col(Alias::new("project_id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "slack_project_syncs",
            "gitlab_comment_syncs",
            "gitlab_issue_syncs",
            "gitlab_repository_syncs",
            "gitlab_repositories",
            "db_githubprstatemapping",
            "github_comment_syncs",
            "github_issue_syncs",
            "github_repository_syncs",
            "github_repositories",
            "importers",
            "exporters",
            "deploy_boards",
            "intake_issues",
            "intakes",
            "description_versions",
            "descriptions",
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

        // Remove the deferred FK we added to project_deploy_boards.intake_id
        // Use raw SQL because SeaORM's drop_foreign_key has no IF EXISTS support
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE project_deploy_boards \
                 DROP CONSTRAINT IF EXISTS project_deploy_boards_intake_id_36aa612d_fk_intakes_id",
            )
            .await?;

        // Remove the deferred FK we added to issue_comments.description_id
        manager
            .get_connection()
            .execute_unprepared(
                "ALTER TABLE issue_comments \
                 DROP CONSTRAINT IF EXISTS issue_comments_description_id_0cb72512_fk_descriptions_id",
            )
            .await?;

        Ok(())
    }
}
