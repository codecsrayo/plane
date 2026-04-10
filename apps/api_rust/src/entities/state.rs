use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "states")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub color: String,
    pub slug: String,
    pub created_by_id: Option<Uuid>,
    pub project_id: Uuid,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub sequence: f64,
    #[sea_orm(column_name = "group")]
    pub group_name: String,
    #[sea_orm(column_name = "default")]
    pub is_default: bool,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
    pub is_triage: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::workspace::Entity",
        from = "Column::WorkspaceId",
        to = "super::workspace::Column::Id"
    )]
    Workspace,
    #[sea_orm(
        belongs_to = "super::project::Entity",
        from = "Column::ProjectId",
        to = "super::project::Column::Id"
    )]
    Project,
    #[sea_orm(has_many = "super::issue::Entity")]
    Issues,
}

impl Related<super::workspace::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Workspace.def()
    }
}

impl Related<super::project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl Related<super::issue::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Issues.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
