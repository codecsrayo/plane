use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "gitlab_issue_syncs")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub repo_issue_id: i64,
    pub gitlab_issue_id: i64,
    pub issue_url: String,
    pub created_by_id: Option<Uuid>,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub repository_sync_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
