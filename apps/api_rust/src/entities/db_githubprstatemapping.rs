use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "db_githubprstatemapping")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub github_pr_state: String,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_integration_id: Uuid,
    pub project_id: Uuid,
    pub state_id: Uuid,
    pub prevent_regression: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
