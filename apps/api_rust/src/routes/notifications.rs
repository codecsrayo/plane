// src/routes/notifications.rs
//! Endpoints for user in-app notifications.
//!
//!   GET    /api/workspaces/{slug}/users/notifications/
//!   GET    /api/workspaces/{slug}/users/notifications/{pk}/
//!   PATCH  /api/workspaces/{slug}/users/notifications/{pk}/
//!   DELETE /api/workspaces/{slug}/users/notifications/{pk}/
//!   POST   /api/workspaces/{slug}/users/notifications/{pk}/read/
//!   DELETE /api/workspaces/{slug}/users/notifications/{pk}/read/
//!   POST   /api/workspaces/{slug}/users/notifications/{pk}/archive/
//!   DELETE /api/workspaces/{slug}/users/notifications/{pk}/archive/
//!   GET    /api/workspaces/{slug}/users/notifications/unread/
//!   POST   /api/workspaces/{slug}/users/notifications/mark-all-read/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{any_auth::AnyAuth, extractors::WorkspaceMemberGuard},
    entities::{
        intake_issues, issue_assignees, issue_subscribers, issues, notifications, users,
        user_notification_preferences,
    },
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

/// Minimum IUserLite shape for triggered_by_details.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserLite {
    pub id: Uuid,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub data: Option<serde_json::Value>,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: String,
    pub message_html: String,
    pub message: Option<serde_json::Value>,
    pub message_stripped: Option<String>,
    pub sender: String,
    // serde renames to match TNotification field names
    #[serde(rename = "receiver")]
    pub receiver_id: Uuid,
    #[serde(rename = "triggered_by")]
    pub triggered_by_id: Option<Uuid>,
    pub triggered_by_details: Option<UserLite>,
    pub read_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub archived_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    #[serde(rename = "project")]
    pub project_id: Option<Uuid>,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    // computed annotations (mirror of Django)
    pub is_inbox_issue: bool,
    pub is_mentioned_notification: bool,
}

impl NotificationResponse {
    fn from_model(
        m: notifications::Model,
        triggered_by_details: Option<UserLite>,
        is_inbox_issue: bool,
    ) -> Self {
        let is_mentioned_notification = m.sender.to_lowercase().contains("mentioned");
        Self {
            id: m.id,
            title: m.title,
            data: m.data,
            entity_identifier: m.entity_identifier,
            entity_name: m.entity_name,
            message_html: m.message_html,
            message: m.message,
            message_stripped: m.message_stripped,
            sender: m.sender,
            receiver_id: m.receiver_id,
            triggered_by_id: m.triggered_by_id,
            triggered_by_details,
            read_at: m.read_at,
            archived_at: m.archived_at,
            snoozed_till: m.snoozed_till,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            created_by_id: m.created_by_id,
            updated_by_id: m.updated_by_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            is_inbox_issue,
            is_mentioned_notification,
        }
    }
}

/// Loads triggered_by_details and is_inbox_issue for a set of notifications
/// in batch (no N+1). Returns enriched NotificationResponse.
async fn enrich_notifications(
    db: &sea_orm::DatabaseConnection,
    rows: Vec<notifications::Model>,
    workspace_id: Uuid,
) -> Result<Vec<NotificationResponse>, AppError> {
    use sea_orm::{ColumnTrait as _, EntityTrait as _, QueryFilter as _, QuerySelect as _};

    // Batch-load triggered_by users
    let triggered_ids: Vec<Uuid> = rows
        .iter()
        .filter_map(|n| n.triggered_by_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let user_map: std::collections::HashMap<Uuid, UserLite> = if triggered_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(triggered_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| {
                let avatar_url = if let Some(asset_id) = u.avatar_asset_id {
                    Some(format!("/api/assets/v2/static/{}/", asset_id))
                } else if !u.avatar.is_empty() {
                    Some(u.avatar)
                } else {
                    None
                };
                (u.id, UserLite {
                    id: u.id,
                    display_name: u.display_name,
                    avatar_url,
                })
            })
            .collect()
    };

    // Batch-load is_inbox_issue: entity_identifier IN (intake_issues.issue_id)
    // status in [0, 2, -2] (pending=0, accepted=2, rejected=-2 like Django)
    let entity_ids: Vec<Uuid> = rows
        .iter()
        .filter(|n| n.entity_name == "issue")
        .filter_map(|n| n.entity_identifier)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let inbox_issue_ids: std::collections::HashSet<Uuid> = if entity_ids.is_empty() {
        std::collections::HashSet::new()
    } else {
        intake_issues::Entity::find()
            .select_only()
            .column(intake_issues::Column::IssueId)
            .filter(intake_issues::Column::IssueId.is_in(entity_ids))
            .filter(
                intake_issues::Column::Status
                    .is_in(vec![0i32, 2i32, -2i32]),
            )
            .filter(intake_issues::Column::WorkspaceId.eq(workspace_id))
            .into_tuple::<Uuid>()
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .collect()
    };

    Ok(rows
        .into_iter()
        .map(|n| {
            let details = n.triggered_by_id.and_then(|id| {
                user_map.get(&id).map(|u| UserLite {
                    id: u.id,
                    display_name: u.display_name.clone(),
                    avatar_url: u.avatar_url.clone(),
                })
            });
            let is_inbox = n
                .entity_identifier
                .map(|id| inbox_issue_ids.contains(&id))
                .unwrap_or(false);
            NotificationResponse::from_model(n, details, is_inbox)
        })
        .collect())
}


#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UnreadCountResponse {
    pub total_unread_notifications_count: u64,
    pub mention_unread_notifications_count: u64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct NotificationFilter {
    pub read: Option<bool>,
    pub archived: Option<bool>,
    pub snoozed: Option<bool>,
    pub type_filter: Option<String>,
}

// ── GET /notifications/ ───────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/users/notifications/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("read" = Option<bool>, Query, description = "Filter by read"),
        ("archived" = Option<bool>, Query, description = "Include archived"),
    ),
    responses((status = 200, description = "Notification list")),
    security(("TokenAuth" = []))
)]
pub async fn list_notifications(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(filter): Query<NotificationFilter>,
) -> Result<Json<Vec<NotificationResponse>>, AppError> {
    let mut query = notifications::Entity::find()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .filter(notifications::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(notifications::Column::DeletedAt.is_null());

    // Snoozed filter (default: false — not snoozed)
    let show_snoozed = filter.snoozed.unwrap_or(false);
    if show_snoozed {
        query = query.filter(notifications::Column::SnoozedTill.is_not_null());
    } else {
        query = query.filter(notifications::Column::SnoozedTill.is_null());
    }

    // By default, do not show archived
    let include_archived = filter.archived.unwrap_or(false);
    if !include_archived {
        query = query.filter(notifications::Column::ArchivedAt.is_null());
    }

    // Filter by read/unread status
    if let Some(read) = filter.read {
        if read {
            query = query.filter(notifications::Column::ReadAt.is_not_null());
        } else {
            query = query.filter(notifications::Column::ReadAt.is_null());
        }
    }

    // Filter by type: subscribed | assigned | created (comma-separated, default "all")
    if let Some(ref type_str) = filter.type_filter {
        let types: Vec<&str> = type_str.split(',').map(str::trim).collect();
        if !types.contains(&"all") {
            let mut issue_ids: Vec<Uuid> = Vec::new();

            if types.contains(&"assigned") {
                let ids = issue_assignees::Entity::find()
                    .select_only()
                    .column(issue_assignees::Column::IssueId)
                    .filter(issue_assignees::Column::AssigneeId.eq(guard.user.id))
                    .into_tuple::<Uuid>()
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?;
                issue_ids.extend(ids);
            }

            if types.contains(&"created") {
                let ids = issues::Entity::find()
                    .select_only()
                    .column(issues::Column::Id)
                    .filter(issues::Column::CreatedById.eq(guard.user.id))
                    .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
                    .into_tuple::<Uuid>()
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?;
                issue_ids.extend(ids);
            }

            if types.contains(&"subscribed") {
                let ids = issue_subscribers::Entity::find()
                    .select_only()
                    .column(issue_subscribers::Column::IssueId)
                    .filter(issue_subscribers::Column::SubscriberId.eq(guard.user.id))
                    .filter(issue_subscribers::Column::WorkspaceId.eq(guard.workspace.id))
                    .into_tuple::<Uuid>()
                    .all(&state.db)
                    .await
                    .map_err(AppError::Database)?;
                issue_ids.extend(ids);
            }

            issue_ids.dedup();
            query = query.filter(notifications::Column::EntityIdentifier.is_in(issue_ids));
        }
    }

    let rows = query
        .order_by_desc(notifications::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let result = enrich_notifications(&state.db, rows, guard.workspace.id).await?;
    Ok(Json(result))
}

// ── GET /notifications/{pk}/ ──────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses(
        (status = 200, description = "Notification detail"),
        (status = 404, description = "Not found"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_notification(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![n], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── PATCH /notifications/{pk}/ ────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateNotificationRequest {
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
}

#[utoipa::path(
    patch,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 200, description = "Notification updated")),
    security(("TokenAuth" = []))
)]
pub async fn update_notification(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
    Json(body): Json<UpdateNotificationRequest>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.snoozed_till = Set(body.snoozed_till);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![updated], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── DELETE /notifications/{pk}/ ───────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 204, description = "Deleted")),
    security(("TokenAuth" = []))
)]
pub async fn delete_notification(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.deleted_at = Set(Some(chrono::Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(StatusCode::NO_CONTENT)
}

// ── POST /notifications/{pk}/read/ ────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/read/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 200, description = "Marked as read")),
    security(("TokenAuth" = []))
)]
pub async fn mark_read(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.read_at = Set(Some(chrono::Utc::now().into()));
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![updated], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── DELETE /notifications/{pk}/read/ ─────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/read/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 200, description = "Marked as unread")),
    security(("TokenAuth" = []))
)]
pub async fn mark_unread(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.read_at = Set(None);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![updated], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── POST /notifications/{pk}/archive/ ────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/archive/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 200, description = "Archived")),
    security(("TokenAuth" = []))
)]
pub async fn archive_notification(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.archived_at = Set(Some(chrono::Utc::now().into()));
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![updated], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── DELETE /notifications/{pk}/archive/ ──────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/users/notifications/{pk}/archive/",
    tag = "Notifications",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("pk" = Uuid, Path, description = "Notification ID"),
    ),
    responses((status = 200, description = "Unarchived")),
    security(("TokenAuth" = []))
)]
pub async fn unarchive_notification(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, pk)): Path<(String, Uuid)>,
) -> Result<Json<NotificationResponse>, AppError> {
    let n = notifications::Entity::find_by_id(pk)
        .active()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: notifications::ActiveModel = n.into();
    am.archived_at = Set(None);
    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    {
        let ws_id = guard.workspace.id;
        let mut enriched = enrich_notifications(&state.db, vec![updated], ws_id).await?;
        Ok(Json(enriched.remove(0)))
    }
}

// ── GET /notifications/unread/ ────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/users/notifications/unread/",
    tag = "Notifications",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "Unread count")),
    security(("TokenAuth" = []))
)]
pub async fn unread_count(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<UnreadCountResponse>, AppError> {
    use sea_orm::PaginatorTrait;

    // Total unread, without archived, without snoozed, excluding mentions
    // (mirrors Django QuerySet: .exclude(sender__icontains="mentioned"))
    let total_unread = notifications::Entity::find()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .filter(notifications::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(notifications::Column::ReadAt.is_null())
        .filter(notifications::Column::ArchivedAt.is_null())
        .filter(notifications::Column::SnoozedTill.is_null())
        .filter(notifications::Column::DeletedAt.is_null())
        .filter(
            sea_orm::Condition::all().add(
                notifications::Column::Sender.not_like("%mentioned%"),
            ),
        )
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Only unread mentions
    let mention_unread = notifications::Entity::find()
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .filter(notifications::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(notifications::Column::ReadAt.is_null())
        .filter(notifications::Column::ArchivedAt.is_null())
        .filter(notifications::Column::SnoozedTill.is_null())
        .filter(notifications::Column::DeletedAt.is_null())
        .filter(notifications::Column::Sender.like("%mentioned%"))
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(UnreadCountResponse {
        total_unread_notifications_count: total_unread,
        mention_unread_notifications_count: mention_unread,
    }))
}

// ── POST /notifications/mark-all-read/ ───────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/users/notifications/mark-all-read/",
    tag = "Notifications",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "All marked as read")),
    security(("TokenAuth" = []))
)]
pub async fn mark_all_read(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<serde_json::Value>, AppError> {
    use sea_orm::sea_query::Expr;
    use sea_orm::EntityTrait as _;

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    notifications::Entity::update_many()
        .col_expr(notifications::Column::ReadAt, Expr::value(now))
        .filter(notifications::Column::ReceiverId.eq(guard.user.id))
        .filter(notifications::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(notifications::Column::ReadAt.is_null())
        .filter(notifications::Column::DeletedAt.is_null())
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(serde_json::json!({ "message": "All notifications marked as read" })))
}

// ═════════════════════════════════════════════════════════════════════════════
// User Notification Preferences — /users/me/notification-preferences
// ═════════════════════════════════════════════════════════════════════════════
//
// Equivalent to Django's `UserNotificationPreferenceEndpoint`
// (`plane/app/views/notification/base.py:291`).
//
// Django exposes `GET` and `PATCH` over the single row of the authenticated user.
// The queryset is resolved with `.get(user=request.user)`, but existence
// of the row depends on a `post_save` signal in the User model
// (`plane/db/models/user.py:305`). Users created via Rust do not
// trigger that signal, so the row might not exist and `.get()` would
// 500. Mirroring the *intent* (not the bug), here we use get_or_create.
//
// Security: although the Django serializer uses `fields = "__all__"` and
// would accept writing FKs (`user`, `workspace`, `project`), here we only
// allow modifying the 5 preference booleans — avoids the anti-pattern
// of mass-assignment over foreign keys.

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct UserNotificationPreferenceResponse {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub user: Uuid,
    pub workspace: Option<Uuid>,
    pub project: Option<Uuid>,
    pub property_change: bool,
    pub state_change: bool,
    pub comment: bool,
    pub mention: bool,
    pub issue_completed: bool,
    pub deleted_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}

impl UserNotificationPreferenceResponse {
    fn from_model(m: user_notification_preferences::Model) -> Self {
        // Field names (`user`, `workspace`, `project`, `created_by`,
        // `updated_by`) replicate DRF output with `fields = "__all__"`,
        // where FKs are serialized as their PK value, not as
        // `*_id`. This maintains frontend compatibility.
        Self {
            id: m.id,
            created_at: m.created_at,
            updated_at: m.updated_at,
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            user: m.user_id,
            workspace: m.workspace_id,
            project: m.project_id,
            property_change: m.property_change,
            state_change: m.state_change,
            comment: m.comment,
            mention: m.mention,
            issue_completed: m.issue_completed,
            deleted_at: m.deleted_at,
        }
    }
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateUserNotificationPreferenceRequest {
    pub property_change: Option<bool>,
    pub state_change: Option<bool>,
    pub comment: Option<bool>,
    pub mention: Option<bool>,
    pub issue_completed: Option<bool>,
}

/// Gets or creates the preference row for the authenticated user.
///
/// Django relies on a `post_save` signal to create the row when registering
/// the user; if the user registered via the Rust flow (which doesn't trigger
/// Django signals), the row might not exist. We create it with the default
/// values of the Django model (all `true`) to avoid a 500.
async fn get_or_create_preferences(
    db: &sea_orm::DatabaseConnection,
    user_id: Uuid,
) -> Result<user_notification_preferences::Model, AppError> {
    if let Some(existing) = user_notification_preferences::Entity::find()
        .filter(user_notification_preferences::Column::UserId.eq(user_id))
        .filter(user_notification_preferences::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
    {
        return Ok(existing);
    }

    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let new_row = user_notification_preferences::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_at: Set(now),
        updated_at: Set(now),
        user_id: Set(user_id),
        workspace_id: Set(None),
        project_id: Set(None),
        // Django Defaults (`plane/db/models/user.py:305-312`).
        property_change: Set(true),
        state_change: Set(true),
        comment: Set(true),
        mention: Set(true),
        issue_completed: Set(true),
        created_by_id: Set(Some(user_id)),
        updated_by_id: Set(Some(user_id)),
        deleted_at: Set(None),
    };

    // Race-safe: if another request created the row between SELECT and INSERT,
    // the INSERT fails due to (expected) logical uniqueness per user; we fall
    // back to re-SELECT instead of propagating the error.
    match new_row.insert(db).await {
        Ok(m) => Ok(m),
        Err(_) => user_notification_preferences::Entity::find()
            .filter(user_notification_preferences::Column::UserId.eq(user_id))
            .filter(user_notification_preferences::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .ok_or_else(|| AppError::BadRequest("could not create preferences".into())),
    }
}

/// GET /api/users/me/notification-preferences
///
/// Mirror Django: `UserNotificationPreferenceEndpoint.get`
/// (`plane/app/views/notification/base.py:296`).
#[utoipa::path(
    get,
    path = "/api/users/me/notification-preferences",
    tag = "Notifications",
    responses(
        (status = 200, description = "User notification preferences"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_user_notification_preferences(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
) -> Result<Json<UserNotificationPreferenceResponse>, AppError> {
    let prefs = get_or_create_preferences(&state.db, user.id).await?;
    Ok(Json(UserNotificationPreferenceResponse::from_model(prefs)))
}

/// PATCH /api/users/me/notification-preferences
///
/// Mirror Django: `UserNotificationPreferenceEndpoint.patch`
/// (`plane/app/views/notification/base.py:302`).
///
/// Unlike Django (which uses `fields = "__all__"` and would allow
/// rewriting `user`, `workspace`, `project` via JSON), here only
/// the five preference booleans are accepted. This blocks
/// mass-assignment over foreign keys without breaking the contract with the
/// frontend (which only sends those fields).
#[utoipa::path(
    patch,
    path = "/api/users/me/notification-preferences",
    tag = "Notifications",
    request_body = UpdateUserNotificationPreferenceRequest,
    responses(
        (status = 200, description = "Preferences updated"),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn update_user_notification_preferences(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<UpdateUserNotificationPreferenceRequest>,
) -> Result<Json<UserNotificationPreferenceResponse>, AppError> {
    let current = get_or_create_preferences(&state.db, user.id).await?;

    let mut active: user_notification_preferences::ActiveModel = current.into();

    if let Some(v) = body.property_change {
        active.property_change = Set(v);
    }
    if let Some(v) = body.state_change {
        active.state_change = Set(v);
    }
    if let Some(v) = body.comment {
        active.comment = Set(v);
    }
    if let Some(v) = body.mention {
        active.mention = Set(v);
    }
    if let Some(v) = body.issue_completed {
        active.issue_completed = Set(v);
    }

    // Audit: the authenticated user is the one modifying.
    active.updated_by_id = Set(Some(user.id));
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(UserNotificationPreferenceResponse::from_model(updated)))
}
