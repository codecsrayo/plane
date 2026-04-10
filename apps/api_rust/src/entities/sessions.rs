use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "sessions")]
pub struct Model {
    pub session_data: String,
    pub expire_date: DateTimeWithTimeZone,
    pub device_info: Option<Json>,
    #[sea_orm(primary_key)]
    pub session_key: String,
    pub user_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
