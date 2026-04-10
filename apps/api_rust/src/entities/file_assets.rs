use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "file_assets")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub attributes: Json,
    pub asset: String,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Option<Uuid>,
    pub is_deleted: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub is_archived: bool,
    pub comment_id: Option<Uuid>,
    pub entity_type: Option<String>,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub is_uploaded: bool,
    pub issue_id: Option<Uuid>,
    pub page_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub size: f64,
    pub storage_metadata: Option<Json>,
    pub user_id: Option<Uuid>,
    pub draft_issue_id: Option<Uuid>,
    pub entity_identifier: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
