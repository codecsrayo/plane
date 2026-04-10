use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_favorites")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub entity_type: String,
    pub entity_identifier: Option<Uuid>,
    pub name: Option<String>,
    pub is_folder: bool,
    pub sequence: f64,
    pub created_by_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
