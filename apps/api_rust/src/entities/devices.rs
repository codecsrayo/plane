use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "devices")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub device_id: Option<String>,
    pub device_type: String,
    pub push_token: Option<String>,
    pub is_active: bool,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub user_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
