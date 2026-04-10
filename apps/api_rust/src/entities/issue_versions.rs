use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_versions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub parent: Option<Uuid>,
    pub state: Option<Uuid>,
    pub estimate_point: Option<Uuid>,
    pub name: String,
    pub priority: String,
    pub start_date: Option<Date>,
    pub target_date: Option<Date>,
    pub sequence_id: i32,
    pub sort_order: f64,
    pub completed_at: Option<DateTimeWithTimeZone>,
    pub archived_at: Option<Date>,
    pub is_draft: bool,
    pub external_source: Option<String>,
    pub external_id: Option<String>,
    #[sea_orm(column_name = "type")]
    pub r#type: Option<Uuid>,
    pub last_saved_at: DateTimeWithTimeZone,
    pub owned_by_id: Uuid,
    pub assignees: Vec<String>,
    pub labels: Vec<String>,
    pub cycle: Option<Uuid>,
    pub modules: Vec<String>,
    pub properties: Json,
    pub meta: Json,
    pub created_by_id: Option<Uuid>,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub activity_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
