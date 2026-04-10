use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "integrations")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub title: String,
    pub provider: String,
    pub network: i32,
    pub description: Json,
    pub author: String,
    pub webhook_url: String,
    pub webhook_secret: String,
    pub redirect_url: String,
    pub metadata: Json,
    pub verified: bool,
    pub avatar_url: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
