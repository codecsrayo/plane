use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000003_workspaces_and_tokens"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // workspaces
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspaces"))
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
                    .col(ColumnDef::new(Alias::new("name")).string_len(80).not_null())
                    .col(ColumnDef::new(Alias::new("logo")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("slug"))
                            .string_len(48)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owner_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("organization_size"))
                            .string_len(20)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("logo_asset_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("timezone"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("background_color"))
                            .string_len(255)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_created_by_id_10ad894e_fk_user_id")
                            .from(Alias::new("workspaces"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_owner_id_60a8bafc_fk_user_id")
                            .from(Alias::new("workspaces"), Alias::new("owner_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_updated_by_id_09d249ed_fk_user_id")
                            .from(Alias::new("workspaces"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspaces_logo_asset_id_a784bb00_fk_file_assets_id")
                            .from(Alias::new("workspaces"), Alias::new("logo_asset_id"))
                            .to(Alias::new("file_assets"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_owner_id_60a8bafc")
                    .table(Alias::new("workspaces"))
                    .col(Alias::new("owner_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspaces_logo_asset_id_a784bb00")
                    .table(Alias::new("workspaces"))
                    .col(Alias::new("logo_asset_id"))
                    .to_owned(),
            )
            .await?;

        // Now add workspace_id FK to file_assets
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("file_assets_workspace_id_fa50b9c5_fk_workspaces_id")
                    .from(Alias::new("file_assets"), Alias::new("workspace_id"))
                    .to(Alias::new("workspaces"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::NoAction)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("file_assets_workspace_id_fa50b9c5")
                    .table(Alias::new("file_assets"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // api_tokens
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("api_tokens"))
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
                        ColumnDef::new(Alias::new("token"))
                            .string_len(255)
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("label"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("user_type"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("description")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("expired_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("last_used"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_service"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("allowed_rate_limit"))
                            .string_len(255)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_tokens_created_by_id_441e3d24_fk_users_id")
                            .from(Alias::new("api_tokens"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_tokens_updated_by_id_bcd544cf_fk_users_id")
                            .from(Alias::new("api_tokens"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_tokens_user_id_2db24e1c_fk_users_id")
                            .from(Alias::new("api_tokens"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_tokens_workspace_id_6791c7bd_fk_workspaces_id")
                            .from(Alias::new("api_tokens"), Alias::new("workspace_id"))
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
                    .name("api_tokens_user_id_2db24e1c")
                    .table(Alias::new("api_tokens"))
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("api_tokens_workspace_id_6791c7bd")
                    .table(Alias::new("api_tokens"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // api_activity_logs
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("api_activity_logs"))
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
                        ColumnDef::new(Alias::new("token_identifier"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("path"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("method"))
                            .string_len(10)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("query_params")).text().null())
                    .col(ColumnDef::new(Alias::new("headers")).text().null())
                    .col(ColumnDef::new(Alias::new("body")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("response_code"))
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("response_body")).text().null())
                    .col(ColumnDef::new(Alias::new("ip_address")).string().null())
                    .col(
                        ColumnDef::new(Alias::new("user_agent"))
                            .string_len(512)
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_activity_logs_created_by_id_7f5c4ca8_fk_users_id")
                            .from(Alias::new("api_activity_logs"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("api_activity_logs_updated_by_id_9ba0d417_fk_users_id")
                            .from(Alias::new("api_activity_logs"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // profiles
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("profiles"))
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
                    .col(ColumnDef::new(Alias::new("theme")).json_binary().not_null())
                    .col(
                        ColumnDef::new(Alias::new("is_tour_completed"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("onboarding_step"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("use_case")).text().null())
                    .col(ColumnDef::new(Alias::new("role")).string_len(300).null())
                    .col(
                        ColumnDef::new(Alias::new("is_onboarded"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("last_workspace_id"))
                            .uuid()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("billing_address_country"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("billing_address"))
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("has_billing_address"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("company_name"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("user_id"))
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_mobile_onboarded"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("mobile_onboarding_step"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("mobile_timezone_auto_set"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("language"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_smooth_cursor_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("start_of_the_week"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_app_rail_docked"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("background_color"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("goals")).json_binary().not_null())
                    .col(
                        ColumnDef::new(Alias::new("has_marketing_email_consent"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_navigation_tour_completed"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_subscribed_to_changelog"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("notification_view_mode"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("product_tour"))
                            .json_binary()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("profiles_user_id_36580373_fk_users_id")
                            .from(Alias::new("profiles"), Alias::new("user_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // workspace_integrations (needs workspaces, users, api_tokens, integrations)
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_integrations"))
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
                        ColumnDef::new(Alias::new("metadata"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("api_token_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(
                        ColumnDef::new(Alias::new("integration_id"))
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
                            .name("workspace_integrations_actor_id_21619aa1_fk_users_id")
                            .from(Alias::new("workspace_integrations"), Alias::new("actor_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_integrations_api_token_id_bdb1759b_fk_api_tokens_id")
                            .from(
                                Alias::new("workspace_integrations"),
                                Alias::new("api_token_id"),
                            )
                            .to(Alias::new("api_tokens"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_integrations_created_by_id_37639c73_fk_users_id")
                            .from(
                                Alias::new("workspace_integrations"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_integratio_integration_id_6cb0aace_fk_integrati")
                            .from(
                                Alias::new("workspace_integrations"),
                                Alias::new("integration_id"),
                            )
                            .to(Alias::new("integrations"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_integrations_updated_by_id_fce01dcb_fk_users_id")
                            .from(
                                Alias::new("workspace_integrations"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_integrations_workspace_id_27ebeb6b_fk_workspaces_id")
                            .from(
                                Alias::new("workspace_integrations"),
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
                    .unique()
                    .name("workspace_integrations_workspace_id_integration_fa041c22_uniq")
                    .table(Alias::new("workspace_integrations"))
                    .col(Alias::new("workspace_id"))
                    .col(Alias::new("integration_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_integrations_actor_id_21619aa1")
                    .table(Alias::new("workspace_integrations"))
                    .col(Alias::new("actor_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_integrations_api_token_id_bdb1759b")
                    .table(Alias::new("workspace_integrations"))
                    .col(Alias::new("api_token_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_integrations_integration_id_6cb0aace")
                    .table(Alias::new("workspace_integrations"))
                    .col(Alias::new("integration_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_integrations_workspace_id_27ebeb6b")
                    .table(Alias::new("workspace_integrations"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // workspace_member_invites
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_member_invites"))
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
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_invite_created_by_id_082f21d3_fk_user_id")
                            .from(
                                Alias::new("workspace_member_invites"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_invite_updated_by_id_d31a9c7f_fk_user_id")
                            .from(
                                Alias::new("workspace_member_invites"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_invite_workspace_id_d935b364_fk_workspace_id")
                            .from(
                                Alias::new("workspace_member_invites"),
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
                    .unique()
                    .name("workspace_member_invites_email_workspace_id_delet_2f03573e_uniq")
                    .table(Alias::new("workspace_member_invites"))
                    .col(Alias::new("email"))
                    .col(Alias::new("workspace_id"))
                    .col(Alias::new("deleted_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX workspace_member_invite_unique_email_workspace_when_deleted_at_
                    ON workspace_member_invites (email, workspace_id)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        // workspace_members
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_members"))
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
                        ColumnDef::new(Alias::new("role"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("member_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("company_role")).text().null())
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
                    .col(
                        ColumnDef::new(Alias::new("issue_props"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("explored_features"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("getting_started_checklist"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("tips")).json_binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_created_by_id_8dc8b040_fk_user_id")
                            .from(Alias::new("workspace_members"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_member_id_824f5497_fk_user_id")
                            .from(Alias::new("workspace_members"), Alias::new("member_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_updated_by_id_1cec0062_fk_user_id")
                            .from(Alias::new("workspace_members"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_member_workspace_id_33f66d4b_fk_workspace_id")
                            .from(Alias::new("workspace_members"), Alias::new("workspace_id"))
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
                    .unique()
                    .name("workspace_members_workspace_id_member_id_d_d7bfa872_uniq")
                    .table(Alias::new("workspace_members"))
                    .col(Alias::new("workspace_id"))
                    .col(Alias::new("member_id"))
                    .col(Alias::new("deleted_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX workspace_member_unique_workspace_member_when_deleted_at_null
                    ON workspace_members (workspace_id, member_id)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_member_member_id_824f5497")
                    .table(Alias::new("workspace_members"))
                    .col(Alias::new("member_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("workspace_member_workspace_id_33f66d4b")
                    .table(Alias::new("workspace_members"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // workspace_themes
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_themes"))
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
                            .string_len(300)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("colors"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("actor_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_themes_actor_id_0e94172e_fk_users_id")
                            .from(Alias::new("workspace_themes"), Alias::new("actor_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_themes_created_by_id_676e2655_fk_users_id")
                            .from(Alias::new("workspace_themes"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_themes_updated_by_id_bba863fe_fk_users_id")
                            .from(Alias::new("workspace_themes"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_themes_workspace_id_d1bffad8_fk_workspaces_id")
                            .from(Alias::new("workspace_themes"), Alias::new("workspace_id"))
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
                    .unique()
                    .name("workspace_themes_workspace_id_name_deleted_at_b536ffd3_uniq")
                    .table(Alias::new("workspace_themes"))
                    .col(Alias::new("workspace_id"))
                    .col(Alias::new("name"))
                    .col(Alias::new("deleted_at"))
                    .to_owned(),
            )
            .await?;

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE UNIQUE INDEX workspace_theme_unique_workspace_name_when_deleted_at_null
                    ON workspace_themes (workspace_id, name)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        // workspace_user_properties
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_user_properties"))
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
                        ColumnDef::new(Alias::new("filters"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("display_filters"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("display_properties"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
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
                        ColumnDef::new(Alias::new("navigation_control_preference"))
                            .string_len(25)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("navigation_project_limit"))
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_properties_created_by_id_6d8d1c4e_fk_users_id")
                            .from(
                                Alias::new("workspace_user_properties"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_properties_updated_by_id_910a2cc5_fk_users_id")
                            .from(
                                Alias::new("workspace_user_properties"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_properties_user_id_b1079e07_fk_users_id")
                            .from(
                                Alias::new("workspace_user_properties"),
                                Alias::new("user_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_prope_workspace_id_1dc3e2a6_fk_workspace")
                            .from(
                                Alias::new("workspace_user_properties"),
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
                r#"CREATE UNIQUE INDEX workspace_user_properties_unique_workspace_user_when_deleted_at
                    ON workspace_user_properties (workspace_id, user_id)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        // workspace_user_preferences
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_user_preferences"))
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
                    .col(ColumnDef::new(Alias::new("key")).string_len(255).not_null())
                    .col(ColumnDef::new(Alias::new("is_pinned")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_preferences_created_by_id_2d566570_fk_users_id")
                            .from(
                                Alias::new("workspace_user_preferences"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_preferences_updated_by_id_65fed266_fk_users_id")
                            .from(
                                Alias::new("workspace_user_preferences"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_preferences_user_id_0ba5007a_fk_users_id")
                            .from(
                                Alias::new("workspace_user_preferences"),
                                Alias::new("user_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_prefe_workspace_id_a345adde_fk_workspace")
                            .from(
                                Alias::new("workspace_user_preferences"),
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
                r#"CREATE UNIQUE INDEX workspace_user_preferences_unique_workspace_user_key_when_delet
                    ON workspace_user_preferences (workspace_id, user_id, key)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        // workspace_home_preferences
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_home_preferences"))
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
                    .col(ColumnDef::new(Alias::new("key")).string_len(255).not_null())
                    .col(
                        ColumnDef::new(Alias::new("is_enabled"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("config"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("user_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("sort_order")).double().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_home_preferences_created_by_id_f31fc163_fk_users_id")
                            .from(
                                Alias::new("workspace_home_preferences"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_home_preferences_updated_by_id_14ed118a_fk_users_id")
                            .from(
                                Alias::new("workspace_home_preferences"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_home_preferences_user_id_4087938d_fk_users_id")
                            .from(
                                Alias::new("workspace_home_preferences"),
                                Alias::new("user_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_home_prefe_workspace_id_b49f76e0_fk_workspace")
                            .from(
                                Alias::new("workspace_home_preferences"),
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
                r#"CREATE UNIQUE INDEX workspace_user_home_preferences_unique_workspace_user_key_when_
                    ON workspace_home_preferences (workspace_id, user_id, key)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        // webhooks
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("webhooks"))
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
                        ColumnDef::new(Alias::new("url"))
                            .string_len(1024)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("is_active")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("secret_key"))
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("project")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("issue")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("module")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("cycle")).boolean().not_null())
                    .col(
                        ColumnDef::new(Alias::new("issue_comment"))
                            .boolean()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("is_internal"))
                            .boolean()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("version"))
                            .string_len(50)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhooks_created_by_id_25aca1b0_fk_users_id")
                            .from(Alias::new("webhooks"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhooks_updated_by_id_ea35154e_fk_users_id")
                            .from(Alias::new("webhooks"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhooks_workspace_id_da5865d7_fk_workspaces_id")
                            .from(Alias::new("webhooks"), Alias::new("workspace_id"))
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
                r#"CREATE UNIQUE INDEX webhook_url_unique_url_when_deleted_at_null
                    ON webhooks (workspace_id, url)
                    WHERE deleted_at IS NULL"#,
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("webhooks_workspace_id_da5865d7")
                    .table(Alias::new("webhooks"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // webhook_logs
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("webhook_logs"))
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
                        ColumnDef::new(Alias::new("event_type"))
                            .string_len(255)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("request_method"))
                            .string_len(10)
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("request_headers")).text().null())
                    .col(ColumnDef::new(Alias::new("request_body")).text().null())
                    .col(ColumnDef::new(Alias::new("response_status")).text().null())
                    .col(ColumnDef::new(Alias::new("response_headers")).text().null())
                    .col(ColumnDef::new(Alias::new("response_body")).text().null())
                    .col(
                        ColumnDef::new(Alias::new("retry_count"))
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("webhook")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .col(
                        ColumnDef::new(Alias::new("deleted_at"))
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhook_logs_created_by_id_71e7bc38_fk_users_id")
                            .from(Alias::new("webhook_logs"), Alias::new("created_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhook_logs_updated_by_id_3d9bad04_fk_users_id")
                            .from(Alias::new("webhook_logs"), Alias::new("updated_by_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("webhook_logs_workspace_id_ffcd0e31_fk_workspaces_id")
                            .from(Alias::new("webhook_logs"), Alias::new("workspace_id"))
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
                    .name("webhook_logs_workspace_id_ffcd0e31")
                    .table(Alias::new("webhook_logs"))
                    .col(Alias::new("workspace_id"))
                    .to_owned(),
            )
            .await?;

        // workspace_user_links
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("workspace_user_links"))
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
                    .col(ColumnDef::new(Alias::new("title")).string_len(255).null())
                    .col(ColumnDef::new(Alias::new("url")).text().not_null())
                    .col(
                        ColumnDef::new(Alias::new("metadata"))
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Alias::new("created_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("owner_id")).uuid().not_null())
                    .col(ColumnDef::new(Alias::new("project_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("updated_by_id")).uuid().null())
                    .col(ColumnDef::new(Alias::new("workspace_id")).uuid().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_links_created_by_id_b9ce7a5d_fk_users_id")
                            .from(
                                Alias::new("workspace_user_links"),
                                Alias::new("created_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_links_owner_id_37d99444_fk_users_id")
                            .from(Alias::new("workspace_user_links"), Alias::new("owner_id"))
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_links_updated_by_id_bd0b017f_fk_users_id")
                            .from(
                                Alias::new("workspace_user_links"),
                                Alias::new("updated_by_id"),
                            )
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("workspace_user_links_workspace_id_1b0a8e22_fk_workspaces_id")
                            .from(
                                Alias::new("workspace_user_links"),
                                Alias::new("workspace_id"),
                            )
                            .to(Alias::new("workspaces"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::NoAction)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        for table in [
            "workspace_user_links",
            "webhook_logs",
            "webhooks",
            "workspace_home_preferences",
            "workspace_user_preferences",
            "workspace_user_properties",
            "workspace_themes",
            "workspace_members",
            "workspace_member_invites",
            "workspace_integrations",
            "profiles",
            "api_activity_logs",
            "api_tokens",
            "workspaces",
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
