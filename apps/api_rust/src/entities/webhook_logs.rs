use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "webhook_logs")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub event_type: Option<String>,
    pub request_method: Option<String>,
    pub request_headers: Option<String>,
    pub request_body: Option<String>,
    pub response_status: Option<String>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub retry_count: i16,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub webhook: Uuid,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
