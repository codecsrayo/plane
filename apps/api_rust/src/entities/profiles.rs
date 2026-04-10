use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "profiles")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub theme: Json,
    pub is_tour_completed: bool,
    pub onboarding_step: Json,
    pub use_case: Option<String>,
    pub role: Option<String>,
    pub is_onboarded: bool,
    pub last_workspace_id: Option<Uuid>,
    pub billing_address_country: String,
    pub billing_address: Option<Json>,
    pub has_billing_address: bool,
    pub company_name: String,
    pub user_id: Uuid,
    pub is_mobile_onboarded: bool,
    pub mobile_onboarding_step: Json,
    pub mobile_timezone_auto_set: bool,
    pub language: String,
    pub is_smooth_cursor_enabled: bool,
    pub start_of_the_week: i16,
    pub is_app_rail_docked: bool,
    pub background_color: String,
    pub goals: Json,
    pub has_marketing_email_consent: bool,
    pub is_navigation_tour_completed: bool,
    pub is_subscribed_to_changelog: bool,
    pub notification_view_mode: String,
    pub product_tour: Json,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
