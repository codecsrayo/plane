use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "pages")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description_json: Json,
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub access: i16,
    pub created_by_id: Option<Uuid>,
    pub owned_by_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub color: String,
    pub archived_at: Option<Date>,
    pub is_locked: bool,
    pub parent_id: Option<Uuid>,
    pub view_props: Json,
    pub logo_props: Json,
    pub description_binary: Option<Vec<u8>>,
    pub is_global: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub moved_to_page: Option<Uuid>,
    pub moved_to_project: Option<Uuid>,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub sort_order: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
