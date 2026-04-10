use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_recent_visits")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: String,
    pub visited_at: DateTimeWithTimeZone,
    pub created_by_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
