use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "labels")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub created_by_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub color: String,
    pub sort_order: f64,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
