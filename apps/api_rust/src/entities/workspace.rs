// src/entities/workspace.rs
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "workspaces")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub logo: Option<String>,
    pub slug: String,
    pub created_by_id: Option<Uuid>,
    pub owner_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub organization_size: Option<String>,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub logo_asset_id: Option<Uuid>,
    pub timezone: String,
    pub background_color: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::project::Entity")]
    Projects,
    #[sea_orm(has_many = "super::state::Entity")]
    States,
    #[sea_orm(has_many = "super::issue::Entity")]
    Issues,
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Projects.def()
    }
}

impl Related<super::state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::States.def()
    }
}

impl Related<super::issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Issues.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
