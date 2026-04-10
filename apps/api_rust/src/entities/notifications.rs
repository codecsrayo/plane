use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "notifications")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub data: Option<Json>,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: String,
    pub title: String,
    pub message: Option<Json>,
    pub message_html: String,
    pub message_stripped: Option<String>,
    pub sender: String,
    pub read_at: Option<DateTimeWithTimeZone>,
    pub snoozed_till: Option<DateTimeWithTimeZone>,
    pub archived_at: Option<DateTimeWithTimeZone>,
    pub created_by_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub receiver_id: Uuid,
    pub triggered_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
