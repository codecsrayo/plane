use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "accounts")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub provider_account_id: String,
    pub provider: String,
    pub access_token: String,
    pub access_token_expired_at: Option<DateTimeWithTimeZone>,
    pub refresh_token: Option<String>,
    pub refresh_token_expired_at: Option<DateTimeWithTimeZone>,
    pub last_connected_at: DateTimeWithTimeZone,
    pub metadata: Json,
    pub user_id: Uuid,
    pub id_token: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
