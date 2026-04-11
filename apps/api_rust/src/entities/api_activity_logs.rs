use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "api_activity_logs")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub token_identifier: String,
    pub path: String,
    pub method: String,
    pub query_params: Option<String>,
    pub headers: Option<String>,
    pub body: Option<String>,
    pub response_code: i32,
    pub response_body: Option<String>,
    // NOTE: baseline_old.sql defines this as `inet`; mapped to String since sea-orm
    // feature `with-ipnetwork` is not enabled. Enable it and use IpNetwork if needed.
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
