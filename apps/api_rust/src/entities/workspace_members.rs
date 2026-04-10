use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "workspace_members")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub role: i16,
    pub created_by_id: Option<Uuid>,
    pub member_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub company_role: Option<String>,
    pub view_props: Json,
    pub default_props: Json,
    pub issue_props: Json,
    pub is_active: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub explored_features: Json,
    pub getting_started_checklist: Json,
    pub tips: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
