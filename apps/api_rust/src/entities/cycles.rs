use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cycles")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub start_date: Option<DateTimeWithTimeZone>,
    pub end_date: Option<DateTimeWithTimeZone>,
    pub created_by_id: Option<Uuid>,
    pub owned_by_id: Uuid,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub view_props: Json,
    pub sort_order: f64,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub progress_snapshot: Json,
    pub archived_at: Option<DateTimeWithTimeZone>,
    pub logo_props: Json,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub timezone: String,
    pub version: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
