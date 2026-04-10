use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "deploy_boards")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: Option<String>,
    pub anchor: String,
    pub is_comments_enabled: bool,
    pub is_reactions_enabled: bool,
    pub is_votes_enabled: bool,
    pub view_props: Json,
    pub created_by_id: Option<Uuid>,
    pub intake_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub is_activity_enabled: bool,
    pub is_disabled: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
