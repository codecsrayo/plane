use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "workspace_user_properties")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub filters: Json,
    pub display_filters: Json,
    pub display_properties: Json,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub rich_filters: Json,
    pub navigation_control_preference: String,
    pub navigation_project_limit: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
