use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "webhooks")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub url: String,
    pub is_active: bool,
    pub secret_key: String,
    pub project: bool,
    pub issue: bool,
    pub module: bool,
    pub cycle: bool,
    pub issue_comment: bool,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub is_internal: bool,
    pub version: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
