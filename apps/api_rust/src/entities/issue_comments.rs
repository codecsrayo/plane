use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "issue_comments")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub comment_stripped: String,
    pub attachments: String,
    pub created_by_id: Option<Uuid>,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub comment_html: String,
    pub comment_json: Json,
    pub access: String,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub edited_at: Option<DateTimeWithTimeZone>,
    pub description_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
