use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "stickies")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: Option<String>,
    pub description: Json,
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub description_binary: Option<Vec<u8>>,
    pub logo_props: Json,
    pub color: Option<String>,
    pub background_color: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub sort_order: f64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
