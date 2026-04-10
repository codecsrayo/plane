use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_views")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub query: Json,
    pub access: i16,
    pub filters: Json,
    pub created_by_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub display_filters: Json,
    pub display_properties: Json,
    pub sort_order: f64,
    pub logo_props: Json,
    pub is_locked: bool,
    pub owned_by_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub rich_filters: Json,
    pub archived_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
