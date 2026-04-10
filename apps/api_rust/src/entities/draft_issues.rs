use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "draft_issues")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: Option<String>,
    pub description_json: Json,
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub description_binary: Option<Vec<u8>>,
    pub priority: String,
    pub start_date: Option<Date>,
    pub target_date: Option<Date>,
    pub sort_order: f64,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub estimate_point_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub state_id: Option<Uuid>,
    pub type_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
