use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "instances")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub instance_name: String,
    pub whitelist_emails: Option<String>,
    pub instance_id: String,
    pub current_version: String,
    pub last_checked_at: DateTimeWithTimeZone,
    pub namespace: Option<String>,
    pub is_telemetry_enabled: bool,
    pub is_support_required: bool,
    pub is_setup_done: bool,
    pub is_signup_screen_visited: bool,
    pub is_verified: bool,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub domain: String,
    pub latest_version: Option<String>,
    pub edition: String,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub is_test: bool,
    pub is_current_version_deprecated: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
