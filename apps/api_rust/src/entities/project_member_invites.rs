use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "project_member_invites")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub email: String,
    pub accepted: bool,
    pub token: String,
    pub message: Option<String>,
    pub responded_at: Option<DateTimeWithTimeZone>,
    pub role: i16,
    pub created_by_id: Option<Uuid>,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
