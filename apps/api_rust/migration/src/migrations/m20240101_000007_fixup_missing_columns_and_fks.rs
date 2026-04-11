use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240101_000007_fixup_missing_columns_and_fks"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // ── 1. exporters.project (uuid[] nullable) ────────────────────────────
        db.execute_unprepared(
            "ALTER TABLE exporters ADD COLUMN IF NOT EXISTS project uuid[] NULL",
        )
        .await?;

        // ── 2. issue_activities.attachments (varchar(200)[] NOT NULL) ─────────
        db.execute_unprepared(
            "ALTER TABLE issue_activities ADD COLUMN IF NOT EXISTS attachments character varying(200)[] NOT NULL DEFAULT '{}'",
        )
        .await?;

        // ── 3. issue_comments.attachments (varchar(200)[] NOT NULL) ───────────
        db.execute_unprepared(
            "ALTER TABLE issue_comments ADD COLUMN IF NOT EXISTS attachments character varying(200)[] NOT NULL DEFAULT '{}'",
        )
        .await?;

        // ── 4. issue_versions array columns ───────────────────────────────────
        db.execute_unprepared(
            "ALTER TABLE issue_versions
               ADD COLUMN IF NOT EXISTS assignees uuid[] NOT NULL DEFAULT '{}',
               ADD COLUMN IF NOT EXISTS cycle    uuid          NULL,
               ADD COLUMN IF NOT EXISTS labels   uuid[] NOT NULL DEFAULT '{}',
               ADD COLUMN IF NOT EXISTS modules  uuid[] NOT NULL DEFAULT '{}'",
        )
        .await?;

        // ── 5. draft_issue_* missing project_id FK ────────────────────────────
        for (tbl, fk_name) in [
            ("draft_issue_assignees", "draft_issue_assignees_project_id_c87dd571_fk_projects_id"),
            ("draft_issue_cycles",    "draft_issue_cycles_project_id_dc5d1ff6_fk_projects_id"),
            ("draft_issue_labels",    "draft_issue_labels_project_id_16f9ba0a_fk_projects_id"),
            ("draft_issue_modules",   "draft_issue_modules_project_id_c32eadab_fk_projects_id"),
        ] {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name(fk_name)
                        .from(Alias::new(tbl), Alias::new("project_id"))
                        .to(Alias::new("projects"), Alias::new("id"))
                        .to_owned(),
                )
                .await?;
        }

        // ── 6. file_assets deferred FKs ───────────────────────────────────────
        // These reference tables created after file_assets (m005 tables)
        for (fk_name, col, ref_tbl) in [
            ("file_assets_comment_id_35d4ecaf_fk_issue_comments_id", "comment_id",     "issue_comments"),
            ("file_assets_draft_issue_id_52633145_fk_draft_issues_id", "draft_issue_id", "draft_issues"),
            ("file_assets_issue_id_cfe87d6c_fk_issues_id",            "issue_id",       "issues"),
            ("file_assets_page_id_64c753d1_fk_pages_id",              "page_id",        "pages"),
            ("file_assets_project_id_ebd5c0d8_fk_projects_id",        "project_id",     "projects"),
        ] {
            manager
                .create_foreign_key(
                    ForeignKey::create()
                        .name(fk_name)
                        .from(Alias::new("file_assets"), Alias::new(col))
                        .to(Alias::new(ref_tbl), Alias::new("id"))
                        .to_owned(),
                )
                .await?;
        }

        // ── 7. workspace_user_links.project_id FK ─────────────────────────────
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("workspace_user_links_project_id_045e0d53_fk_projects_id")
                    .from(Alias::new("workspace_user_links"), Alias::new("project_id"))
                    .to(Alias::new("projects"), Alias::new("id"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Remove FK workspace_user_links.project_id
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("workspace_user_links_project_id_045e0d53_fk_projects_id")
                    .table(Alias::new("workspace_user_links"))
                    .to_owned(),
            )
            .await?;

        // Remove file_assets FKs
        for (fk_name, tbl) in [
            ("file_assets_comment_id_35d4ecaf_fk_issue_comments_id",   "file_assets"),
            ("file_assets_draft_issue_id_52633145_fk_draft_issues_id", "file_assets"),
            ("file_assets_issue_id_cfe87d6c_fk_issues_id",             "file_assets"),
            ("file_assets_page_id_64c753d1_fk_pages_id",               "file_assets"),
            ("file_assets_project_id_ebd5c0d8_fk_projects_id",         "file_assets"),
        ] {
            manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .name(fk_name)
                        .table(Alias::new(tbl))
                        .to_owned(),
                )
                .await?;
        }

        // Remove draft_issue_* project_id FKs
        for (tbl, fk_name) in [
            ("draft_issue_assignees", "draft_issue_assignees_project_id_c87dd571_fk_projects_id"),
            ("draft_issue_cycles",    "draft_issue_cycles_project_id_dc5d1ff6_fk_projects_id"),
            ("draft_issue_labels",    "draft_issue_labels_project_id_16f9ba0a_fk_projects_id"),
            ("draft_issue_modules",   "draft_issue_modules_project_id_c32eadab_fk_projects_id"),
        ] {
            manager
                .drop_foreign_key(
                    ForeignKey::drop()
                        .name(fk_name)
                        .table(Alias::new(tbl))
                        .to_owned(),
                )
                .await?;
        }

        // Drop added columns
        db.execute_unprepared(
            "ALTER TABLE issue_versions
               DROP COLUMN IF EXISTS assignees,
               DROP COLUMN IF EXISTS cycle,
               DROP COLUMN IF EXISTS labels,
               DROP COLUMN IF EXISTS modules",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE issue_comments DROP COLUMN IF EXISTS attachments",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE issue_activities DROP COLUMN IF EXISTS attachments",
        )
        .await?;

        db.execute_unprepared(
            "ALTER TABLE exporters DROP COLUMN IF EXISTS project",
        )
        .await?;

        Ok(())
    }
}
