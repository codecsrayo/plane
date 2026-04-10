use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "device_sessions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub is_active: bool,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    pub start_time: DateTimeWithTimeZone,
    pub end_time: Option<DateTimeWithTimeZone>,
    pub created_by_id: Option<Uuid>,
    pub device_id: Uuid,
    pub session_id: String,
    pub updated_by_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
