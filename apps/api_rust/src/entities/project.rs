use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "projects")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub description_text: Option<Json>,
    pub description_html: Option<Json>,
    pub network: i16,
    pub identifier: String,
    pub created_by_id: Option<Uuid>,
    pub default_assignee_id: Option<Uuid>,
    pub project_lead_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub emoji: Option<String>,
    pub cycle_view: bool,
    pub module_view: bool,
    pub cover_image: Option<String>,
    pub issue_views_view: bool,
    pub page_view: bool,
    pub estimate_id: Option<Uuid>,
    pub icon_prop: Option<Json>,
    pub intake_view: bool,
    pub archive_in: i32,
    pub close_in: i32,
    pub default_state_id: Option<Uuid>,
    pub logo_props: Json,
    pub archived_at: Option<DateTimeWithTimeZone>,
    pub is_time_tracking_enabled: bool,
    pub is_issue_type_enabled: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
    pub guest_view_all_features: bool,
    pub timezone: String,
    pub cover_image_asset_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub external_source: Option<String>,
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
        belongs_to = "super::state::Entity",
        from = "Column::DefaultStateId",
        to = "super::state::Column::Id"
    )]
    DefaultState,
    #[sea_orm(has_many = "super::state::Entity")]
    States,
    #[sea_orm(has_many = "super::issue::Entity")]
    Issues,
}

impl Related<super::workspace::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Workspace.def()
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
