use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "email_notification_logs")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: String,
    pub data: Option<Json>,
    pub processed_at: Option<DateTimeWithTimeZone>,
    pub sent_at: Option<DateTimeWithTimeZone>,
    pub entity: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub created_by_id: Option<Uuid>,
    pub receiver_id: Uuid,
    pub triggered_by_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
