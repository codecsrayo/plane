use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "exporters")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub project: Option<Vec<Uuid>>,
    pub provider: String,
    pub status: String,
    pub reason: String,
    pub key: String,
    pub url: Option<String>,
    pub token: String,
    pub created_by_id: Option<Uuid>,
    pub initiated_by_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub filters: Option<Json>,
    pub name: Option<String>,
    #[sea_orm(column_name = "type")]
    pub r#type: String,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub rich_filters: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
