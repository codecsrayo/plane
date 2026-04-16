// src/routes/notifications.rs
//! Endpoints de Notificaciones in-app del usuario.
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
    auth::extractors::WorkspaceMemberGuard,
    entities::{issue_assignees, issue_subscribers, issues, notifications},
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub title: String,
    pub message_html: String,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: String,
    pub sender: String,
    pub read_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub archived_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub snoozed_till: Option<chrono::DateTime<chrono::FixedOffset>>,
    pub project_id: Option<Uuid>,
    pub workspace_id: Uuid,
    pub triggered_by_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

impl NotificationResponse {
    fn from_model(m: notifications::Model) -> Self {
        Self {
            id: m.id,
            title: m.title,
            message_html: m.message_html,
            entity_identifier: m.entity_identifier,
            entity_name: m.entity_name,
            sender: m.sender,
            read_at: m.read_at,
            archived_at: m.archived_at,
            snoozed_till: m.snoozed_till,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            triggered_by_id: m.triggered_by_id,
            created_at: m.created_at,
        }
    }
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
        ("read" = Option<bool>, Query, description = "Filtrar por leídas"),
        ("archived" = Option<bool>, Query, description = "Incluir archivadas"),
    ),
    responses((status = 200, description = "Lista de notificaciones")),
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

    // Filtro snoozed (default: false — no pospuestas)
    let show_snoozed = filter.snoozed.unwrap_or(false);
    if show_snoozed {
        query = query.filter(notifications::Column::SnoozedTill.is_not_null());
    } else {
        query = query.filter(notifications::Column::SnoozedTill.is_null());
    }

    // Por defecto no mostrar archivadas
    let include_archived = filter.archived.unwrap_or(false);
    if !include_archived {
        query = query.filter(notifications::Column::ArchivedAt.is_null());
    }

    // Filtrar por estado leído/no leído
    if let Some(read) = filter.read {
        if read {
            query = query.filter(notifications::Column::ReadAt.is_not_null());
        } else {
            query = query.filter(notifications::Column::ReadAt.is_null());
        }
    }

    // Filtrar por tipo: subscribed | assigned | created (comma-separated, default "all")
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

    Ok(Json(rows.into_iter().map(NotificationResponse::from_model).collect()))
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
        (status = 200, description = "Detalle de la notificación"),
        (status = 404, description = "No encontrada"),
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

    Ok(Json(NotificationResponse::from_model(n)))
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
    responses((status = 200, description = "Notificación actualizada")),
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
    Ok(Json(NotificationResponse::from_model(updated)))
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
    responses((status = 204, description = "Eliminada")),
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
    responses((status = 200, description = "Marcada como leída")),
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
    Ok(Json(NotificationResponse::from_model(updated)))
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
    responses((status = 200, description = "Marcada como no leída")),
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
    Ok(Json(NotificationResponse::from_model(updated)))
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
    responses((status = 200, description = "Archivada")),
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
    Ok(Json(NotificationResponse::from_model(updated)))
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
    responses((status = 200, description = "Desarchivada")),
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
    Ok(Json(NotificationResponse::from_model(updated)))
}

// ── GET /notifications/unread/ ────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/users/notifications/unread/",
    tag = "Notifications",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses((status = 200, description = "Conteo de no leídas")),
    security(("TokenAuth" = []))
)]
pub async fn unread_count(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
) -> Result<Json<UnreadCountResponse>, AppError> {
    use sea_orm::PaginatorTrait;

    // Total de no leídas, sin archivadas, sin pospuestas, excluyendo menciones
    // (espeja el QuerySet de Django: .exclude(sender__icontains="mentioned"))
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

    // Solo menciones no leídas
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
    responses((status = 200, description = "Todas marcadas como leídas")),
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
