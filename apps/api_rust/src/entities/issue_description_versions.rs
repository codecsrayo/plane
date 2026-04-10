use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_description_versions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub description_binary: Option<Vec<u8>>,
    pub description_html: String,
    pub description_stripped: Option<String>,
    pub description_json: Json,
    pub last_saved_at: DateTimeWithTimeZone,
    pub created_by_id: Option<Uuid>,
    pub issue_id: Uuid,
    pub owned_by_id: Uuid,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
