use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "states")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub color: String,
    pub slug: String,
    pub created_by_id: Option<Uuid>,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub sequence: f64,
    #[sea_orm(column_name = "group")]
    pub r#group: String,
    #[sea_orm(column_name = "default")]
    pub r#default: bool,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub is_triage: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
