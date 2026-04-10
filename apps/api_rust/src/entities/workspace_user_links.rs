use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "workspace_user_links")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: Option<String>,
    pub url: String,
    pub metadata: Json,
    pub created_by_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
