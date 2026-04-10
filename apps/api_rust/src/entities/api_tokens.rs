use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "api_tokens")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub token: String,
    pub label: String,
    pub user_type: i16,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub user_id: Uuid,
    pub workspace_id: Option<Uuid>,
    pub description: String,
    pub expired_at: Option<DateTimeWithTimeZone>,
    pub is_active: bool,
    pub last_used: Option<DateTimeWithTimeZone>,
    pub is_service: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub allowed_rate_limit: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
