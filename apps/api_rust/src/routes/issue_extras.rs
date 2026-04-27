// src/routes/issue_extras.rs
//! Sub-endpoints de issues: comments, reactions, relations, links, subscribers, activities.
//!
//! Equivalente a `plane/app/views/issue/` (múltiples archivos) en Django.
//!
//! Rutas implementadas:
//!   GET/POST   issues/{id}/comments/
//!   GET/PATCH/DELETE issues/{id}/comments/{pk}/
//!   GET/POST   issues/{id}/reactions/        (reaction code en body)
//!   DELETE     issues/{id}/reactions/{code}/
//!   GET/POST   comments/{id}/reactions/      (reaction code en body)
//!   DELETE     comments/{id}/reactions/{code}/
//!   GET/POST   issues/{id}/issue-links/
//!   PATCH/DELETE issues/{id}/issue-links/{pk}/
//!   GET/POST   issues/{id}/issue-relation/
//!   POST       issues/{id}/remove-relation/
//!   GET        issues/{id}/history/
//!   GET/POST   issues/{id}/issue-subscribers/
//!   DELETE     issues/{id}/issue-subscribers/{subscriber_id}/
//!   POST/DELETE issues/{id}/subscribe/
//!   GET/POST   issues/{id}/sub-issues/

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{
        extractors::ProjectMemberGuard,
        permissions::{require_role, ROLE_GUEST, ROLE_MEMBER},
    },
    entities::{
        comment_reactions, issue_activities, issue_comments, issue_links,
        issue_reactions, issue_relations, issue_subscribers, issues, projects, states,
        users, workspaces,
    },
    error::AppError,
    routes::workspaces::{user_to_lite, UserLiteDto},
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentResponse {
    pub id: Uuid,
    pub comment_html: String,
    pub comment_stripped: String,
    pub access: String,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub created_by_id: Option<Uuid>,
    pub updated_by_id: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edited_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateCommentRequest {
    pub comment_html: String,
    pub access: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateCommentRequest {
    pub comment_html: Option<String>,
    pub access: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    /// Mirror de `IssueReactionSerializer.actor_detail` (UserLiteSerializer).
    /// `apps/api/plane/app/serializers/issue.py:648-654`.
    pub actor_detail: UserLiteDto,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct CommentReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub comment_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    /// Mirror de `CommentReactionSerializer.display_name` + actor lite.
    /// `apps/api/plane/app/serializers/issue.py:665-686`.
    pub actor_detail: UserLiteDto,
}

/// Body de POST `/issues/{id}/reactions/` y `/comments/{id}/reactions/`.
///
/// Django recibe `{"reaction": "<code>"}` vía `IssueReactionSerializer` /
/// `CommentReactionSerializer` (`apps/api/plane/app/views/issue/reaction.py:47`
/// y `apps/api/plane/app/views/issue/comment.py:186`). El `reaction_code` NO va
/// en la URL para create — solo para destroy.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateReactionRequest {
    pub reaction: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueLinkResponse {
    pub id: Uuid,
    pub title: Option<String>,
    pub url: String,
    pub metadata: serde_json::Value,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub created_by_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UpdateLinkRequest {
    pub url: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct IssueRelationResponse {
    pub id: Uuid,
    pub relation_type: String,
    pub issue_id: Uuid,
    pub related_issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct CreateRelationRequest {
    /// Tipo de relación. Django no valida estrictamente el valor — lo pasa
    /// por `get_actual_relation` (apps/api/plane/utils/issue_relation_mapper.py)
    /// que mapea tipos "inversos" al canónico almacenado en BD. La columna
    /// es `varchar(20)` (`migration/src/sql/baseline.sql:1420`) sin enum
    /// constraint.
    pub relation_type: String,

    /// Lista de UUIDs de issues a relacionar. Shape único, alineado con:
    ///   - Django: `apps/api/plane/app/views/issue/relation.py:217`
    ///     (`request.data.get("issues", [])`).
    ///   - Frontend canónico: `apps/web/core/services/issue/issue_relation.service.ts:33`
    ///     (`data: { relation_type, issues: string[] }`).
    ///
    /// Nota histórica: existió un `related_list: Vec<Uuid>` en un servicio
    /// frontend obsoleto (`issue.service.ts:177`) que nadie consumía. Se
    /// estandarizó en este shape para evitar parsers polimórficos en el
    /// handler.
    pub issues: Vec<Uuid>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RemoveRelationRequest {
    /// UUID del issue contraparte de la relación a eliminar.
    ///
    /// Paridad Django (`apps/api/plane/app/views/issue/relation.py:263`):
    /// `related_issue = request.data.get("related_issue", None)`. Antes el
    /// campo se llamaba `related_issue_id`, lo que provocaba un 422
    /// (`missing field 'related_issue_id'`) cuando el frontend enviaba el
    /// payload con paridad Django.
    pub related_issue: Uuid,

    /// Django no usa `relation_type` en `remove_relation` — la relación se
    /// localiza únicamente por el par `(issue_id, related_issue)`. Lo
    /// aceptamos opcional por compatibilidad con clientes que lo envíen,
    /// pero NO se usa para filtrar la query (mantendría paridad estricta
    /// con `apps/api/plane/app/views/issue/relation.py:265-269`).
    #[serde(default)]
    pub relation_type: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ActivityResponse {
    pub id: Uuid,
    pub verb: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub old_identifier: Option<String>,
    pub new_identifier: Option<String>,
    pub comment: String,
    // Frontend IIssueActivity uses "actor" not "actor_id"
    #[serde(rename = "actor")]
    pub actor_id: Option<Uuid>,
    pub actor_detail: Option<UserLiteDto>,
    // Frontend uses "issue" not "issue_id"
    #[serde(rename = "issue")]
    pub issue_id: Option<Uuid>,
    pub issue_comment: Option<String>,
    // Frontend uses "project" not "project_id"
    #[serde(rename = "project")]
    pub project_id: Uuid,
    pub project_detail: Option<serde_json::Value>,
    pub workspace_detail: Option<serde_json::Value>,
    pub attachments: Vec<serde_json::Value>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub access: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SubscriberResponse {
    pub id: Uuid,
    pub subscriber_id: Uuid,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn comment_to_response(c: &issue_comments::Model) -> CommentResponse {
    CommentResponse {
        id: c.id,
        comment_html: c.comment_html.clone(),
        comment_stripped: c.comment_stripped.clone(),
        access: c.access.clone(),
        issue_id: c.issue_id,
        project_id: c.project_id,
        workspace_id: c.workspace_id,
        actor_id: c.actor_id,
        created_by_id: c.created_by_id,
        updated_by_id: c.updated_by_id,
        parent_id: c.parent_id,
        created_at: c.created_at,
        updated_at: c.updated_at,
        edited_at: c.edited_at,
    }
}

fn strip_html(html: &str) -> String {
    // Remove HTML tags for comment_stripped
    let mut result = String::new();
    let mut inside = false;
    for ch in html.chars() {
        if ch == '<' { inside = true; continue; }
        if ch == '>' { inside = false; continue; }
        if !inside { result.push(ch); }
    }
    result.trim().to_string()
}

// ── Comments ──────────────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Comments list"))
)]
pub async fn list_comments(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let comments = issue_comments::Entity::find()
        .active()
        .filter(issue_comments::Column::IssueId.eq(issue_id))
        .filter(issue_comments::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(issue_comments::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(comments.iter().map(comment_to_response).collect::<Vec<_>>()))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Comment created"))
)]
pub async fn create_comment(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.comment_html.trim().is_empty() {
        return Err(AppError::BadRequest("comment_html is required".into()));
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let comment_html = crate::utils::content_validator::sanitize_description(&body.comment_html)
        .map_err(AppError::BadRequest)?;
    let stripped = strip_html(&comment_html);
    let new_comment = issue_comments::ActiveModel {
        id: Set(Uuid::new_v4()),
        comment_html: Set(comment_html),
        comment_stripped: Set(stripped),
        comment_json: Set(serde_json::json!({})),
        access: Set(body.access.unwrap_or_else(|| "INTERNAL".to_string())),
        attachments: Set(vec![]),
        issue_id: Set(issue_id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        actor_id: Set(Some(guard.user.id)),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        parent_id: Set(body.parent_id),
        external_id: Set(None),
        external_source: Set(None),
        description_id: Set(None),
        edited_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_comment.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(comment_to_response(&created))))
}

/// PATCH /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/
#[utoipa::path(
    patch, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Updated"))
)]
pub async fn update_comment(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::IssueId.eq(issue_id))
        .filter(issue_comments::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Only actor/creator can update
    let is_owner = comment.actor_id == Some(guard.user.id)
        || comment.created_by_id == Some(guard.user.id);
    if !is_owner {
        return Err(AppError::Forbidden);
    }

    let mut am: issue_comments::ActiveModel = comment.into();
    am.updated_at = Set(Utc::now().into());
    am.edited_at = Set(Some(Utc::now().into()));
    am.updated_by_id = Set(Some(guard.user.id));

    if let Some(html) = body.comment_html {
        let clean = crate::utils::content_validator::sanitize_description(&html)
            .map_err(AppError::BadRequest)?;
        am.comment_stripped = Set(strip_html(&clean));
        am.comment_html = Set(clean);
    }
    if let Some(access) = body.access {
        am.access = Set(access);
    }

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(comment_to_response(&updated)))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Deleted"))
)]
pub async fn delete_comment(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::IssueId.eq(issue_id))
        .filter(issue_comments::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let is_owner = comment.actor_id == Some(guard.user.id)
        || comment.created_by_id == Some(guard.user.id);
    let is_admin = guard.project_member.role >= 20 || guard.workspace_member.role >= 20;
    if !is_owner && !is_admin {
        return Err(AppError::Forbidden);
    }

    let mut am: issue_comments::ActiveModel = comment.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Reactions ───────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/
///
/// Mirror de `IssueReactionViewSet.list` (`apps/api/plane/app/views/issue/reaction.py:25`).
/// Orden: `-created_at` (Django `Meta.ordering`).
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Reactions"))
)]
pub async fn list_issue_reactions(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let reactions = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::ProjectId.eq(guard.project.id))
        .order_by_desc(issue_reactions::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch-fetch distinct actors (evita N+1). Django usa ORM joins implícitos;
    // aquí hacemos una sola query agrupada por `actor_id`.
    let actor_ids: Vec<Uuid> = reactions
        .iter()
        .map(|r| r.actor_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let actors: std::collections::HashMap<Uuid, users::Model> = if actor_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(actor_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let resp: Vec<ReactionResponse> = reactions
        .iter()
        .filter_map(|r| {
            let actor = actors.get(&r.actor_id)?;
            Some(ReactionResponse {
                id: r.id,
                reaction: r.reaction.clone(),
                issue_id: r.issue_id,
                actor_id: r.actor_id,
                project_id: r.project_id,
                workspace_id: r.workspace_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                // `is_admin=false` → mirror de `UserLiteSerializer` (sin email /
                // last_login_medium). Django usa la variante lite en el nested
                // `actor_detail` de `IssueReactionSerializer`.
                actor_detail: user_to_lite(actor, false),
            })
        })
        .collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/
///
/// Mirror de `IssueReactionViewSet.create` (`apps/api/plane/app/views/issue/reaction.py:45-62`).
/// El `reaction` code viaja en el body JSON (`{"reaction": "👍"}`), NO en la URL.
/// Idempotente: si ya existe una reacción activa del mismo actor con el mismo
/// code, la devuelve con 201 en vez de disparar IntegrityError (mejora de UX
/// sobre Django — `unique_together = ["issue", "actor", "reaction", "deleted_at"]`).
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/",
    tag = "Issues", security(("TokenAuth" = [])),
    request_body = CreateReactionRequest,
    responses((status = 201, description = "Reaction added"))
)]
pub async fn add_issue_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let reaction_code = body.reaction.trim().to_string();
    if reaction_code.is_empty() {
        return Err(AppError::BadRequest("reaction is required".into()));
    }

    // Idempotencia — evita IntegrityError del unique constraint de Django.
    let existing = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::ActorId.eq(guard.user.id))
        .filter(issue_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(r) = existing {
        return Ok((StatusCode::CREATED, Json(ReactionResponse {
            id: r.id,
            reaction: r.reaction,
            issue_id: r.issue_id,
            actor_id: r.actor_id,
            project_id: r.project_id,
            workspace_id: r.workspace_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            actor_detail: user_to_lite(&guard.user, false),
        })));
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_reaction = issue_reactions::ActiveModel {
        id: Set(Uuid::new_v4()),
        reaction: Set(reaction_code),
        issue_id: Set(issue_id),
        actor_id: Set(guard.user.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_reaction.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(ReactionResponse {
        id: created.id,
        reaction: created.reaction,
        issue_id: created.issue_id,
        actor_id: created.actor_id,
        project_id: created.project_id,
        workspace_id: created.workspace_id,
        created_at: created.created_at,
        updated_at: created.updated_at,
        actor_detail: user_to_lite(&guard.user, false),
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/
///
/// Mirror de `IssueReactionViewSet.destroy`
/// (`apps/api/plane/app/views/issue/reaction.py:64-85`). Soft delete.
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Removed"))
)]
pub async fn remove_issue_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, reaction_code)): Path<(String, Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let reaction = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::ActorId.eq(guard.user.id))
        .filter(issue_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Comment Reactions ─────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/
///
/// Mirror de `CommentReactionViewSet.list` (`apps/api/plane/app/views/issue/comment.py:163`).
/// Orden: `-created_at` (Django `Meta.ordering` de `CommentReaction`).
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Comment reactions"))
)]
pub async fn list_comment_reactions(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, comment_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let reactions = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::ProjectId.eq(guard.project.id))
        .order_by_desc(comment_reactions::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let actor_ids: Vec<Uuid> = reactions
        .iter()
        .map(|r| r.actor_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let actors: std::collections::HashMap<Uuid, users::Model> = if actor_ids.is_empty() {
        std::collections::HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(actor_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let resp: Vec<CommentReactionResponse> = reactions
        .iter()
        .filter_map(|r| {
            let actor = actors.get(&r.actor_id)?;
            Some(CommentReactionResponse {
                id: r.id,
                reaction: r.reaction.clone(),
                comment_id: r.comment_id,
                actor_id: r.actor_id,
                project_id: r.project_id,
                workspace_id: r.workspace_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
                actor_detail: user_to_lite(actor, false),
            })
        })
        .collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/
///
/// Mirror de `CommentReactionViewSet.create`
/// (`apps/api/plane/app/views/issue/comment.py:183-210`). El `reaction` code
/// viaja en el body JSON, no en la URL. Idempotente.
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/",
    tag = "Issues", security(("TokenAuth" = [])),
    request_body = CreateReactionRequest,
    responses((status = 201, description = "Reaction added"))
)]
pub async fn add_comment_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, comment_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    let reaction_code = body.reaction.trim().to_string();
    if reaction_code.is_empty() {
        return Err(AppError::BadRequest("reaction is required".into()));
    }

    let existing = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::ActorId.eq(guard.user.id))
        .filter(comment_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if let Some(r) = existing {
        return Ok((StatusCode::CREATED, Json(CommentReactionResponse {
            id: r.id,
            reaction: r.reaction,
            comment_id: r.comment_id,
            actor_id: r.actor_id,
            project_id: r.project_id,
            workspace_id: r.workspace_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
            actor_detail: user_to_lite(&guard.user, false),
        })));
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_reaction = comment_reactions::ActiveModel {
        id: Set(Uuid::new_v4()),
        reaction: Set(reaction_code),
        comment_id: Set(comment_id),
        actor_id: Set(guard.user.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_reaction.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(CommentReactionResponse {
        id: created.id,
        reaction: created.reaction,
        comment_id: created.comment_id,
        actor_id: created.actor_id,
        project_id: created.project_id,
        workspace_id: created.workspace_id,
        created_at: created.created_at,
        updated_at: created.updated_at,
        actor_detail: user_to_lite(&guard.user, false),
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/
///
/// Mirror de `CommentReactionViewSet.destroy`
/// (`apps/api/plane/app/views/issue/comment.py:212-240`). Soft delete.
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Removed"))
)]
pub async fn remove_comment_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, comment_id, reaction_code)): Path<(String, Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    let reaction = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::ActorId.eq(guard.user.id))
        .filter(comment_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: comment_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Links ───────────────────────────────────────────────────────────────

// `normalize_and_validate_url` se movió a `crate::utils::url` (paridad con
// Django) para reutilizarse desde issue-links, module-links y otros endpoints
// con shape similar. Re-export local para no romper call-sites internos.
use crate::utils::url::normalize_and_validate_url;


/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Links"))
)]
pub async fn list_issue_links(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let links = issue_links::Entity::find()
        .active()
        .filter(issue_links::Column::IssueId.eq(issue_id))
        .filter(issue_links::Column::ProjectId.eq(guard.project.id))
        .order_by_desc(issue_links::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<IssueLinkResponse> = links.iter().map(|l| IssueLinkResponse {
        id: l.id, title: l.title.clone(), url: l.url.clone(), metadata: l.metadata.clone(),
        issue_id: l.issue_id, project_id: l.project_id,
        created_by_id: l.created_by_id, created_at: l.created_at, updated_at: l.updated_at,
    }).collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Link created"))
)]
pub async fn create_issue_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateLinkRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.url.trim().is_empty() {
        return Err(AppError::BadRequest("url is required".into()));
    }

    // Paridad Django (apps/api/plane/app/serializers/issue.py:565-579):
    //   1. to_internal_value: si la URL no empieza por http(s)://, antepone
    //      "http://" antes de validar.
    //   2. validate_url: usa Django URLValidator (RFC 3986 con host válido y
    //      TLD o IP). "not-a-url" → "http://not-a-url" → rechazado por falta
    //      de TLD.
    // Sin esta validación el handler aceptaba cualquier string como URL
    // (ej. "not-a-url" devolvía 201) — divergencia con Django y con la
    // expectativa del frontend que confía en que `url` sea fetcheable.
    let url = normalize_and_validate_url(body.url.trim())?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_link = issue_links::ActiveModel {
        id: Set(Uuid::new_v4()),
        url: Set(url),
        title: Set(body.title),
        metadata: Set(body.metadata.unwrap_or(serde_json::json!({}))),
        issue_id: Set(issue_id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_link.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(IssueLinkResponse {
        id: created.id, title: created.title, url: created.url, metadata: created.metadata,
        issue_id: created.issue_id, project_id: created.project_id,
        created_by_id: created.created_by_id, created_at: created.created_at, updated_at: created.updated_at,
    })))
}

/// PATCH /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/
#[utoipa::path(
    patch, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Updated"))
)]
pub async fn update_issue_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
    Json(body): Json<UpdateLinkRequest>,
) -> Result<impl IntoResponse, AppError> {
    let link = issue_links::Entity::find_by_id(pk)
        .active()
        .filter(issue_links::Column::IssueId.eq(issue_id))
        .filter(issue_links::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_links::ActiveModel = link.into();
    am.updated_at = Set(Utc::now().into());
    am.updated_by_id = Set(Some(guard.user.id));

    if let Some(v) = body.url { am.url = Set(v); }
    if let Some(v) = body.title { am.title = Set(Some(v)); }
    if let Some(v) = body.metadata { am.metadata = Set(v); }

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;
    Ok(Json(IssueLinkResponse {
        id: updated.id, title: updated.title, url: updated.url, metadata: updated.metadata,
        issue_id: updated.issue_id, project_id: updated.project_id,
        created_by_id: updated.created_by_id, created_at: updated.created_at, updated_at: updated.updated_at,
    }))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-links/{pk}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Deleted"))
)]
pub async fn delete_issue_link(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let link = issue_links::Entity::find_by_id(pk)
        .active()
        .filter(issue_links::Column::IssueId.eq(issue_id))
        .filter(issue_links::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_links::ActiveModel = link.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Relations ───────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Relations"))
)]
pub async fn list_issue_relations(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let relations = issue_relations::Entity::find()
        .active()
        .filter(issue_relations::Column::IssueId.eq(issue_id))
        .filter(issue_relations::Column::ProjectId.eq(guard.project.id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<IssueRelationResponse> = relations.iter().map(|r| IssueRelationResponse {
        id: r.id, relation_type: r.relation_type.clone(),
        issue_id: r.issue_id, related_issue_id: r.related_issue_id,
        project_id: r.project_id, workspace_id: r.workspace_id, created_at: r.created_at,
    }).collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-relation/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Relation created"))
)]
pub async fn create_issue_relation(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<CreateRelationRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    if body.issues.is_empty() {
        return Err(AppError::BadRequest("issues no puede estar vacío".into()));
    }

    // Paridad Django (`apps/api/plane/utils/issue_relation_mapper.py:19-31`,
    // `apps/api/plane/app/views/issue/relation.py:220-237`):
    //
    // `get_actual_relation` mapea los tipos "inversos" al canónico almacenado
    // en BD, y la creación invierte la orientación de la relación cuando el
    // tipo es uno de los inversos. La idea: una relación A→B "blocking"
    // se almacena como B→A "blocked_by", de modo que toda la BD habla en
    // términos canónicos y los queries simétricos del frontend son simples.
    //
    // Sin este mapping, la BD acumula tipos duplicados/inconsistentes
    // ("blocking" y "blocked_by" coexistiendo) y los reads del frontend
    // se rompen.
    let is_inverse = matches!(
        body.relation_type.as_str(),
        "blocking" | "start_after" | "finish_after" | "implements"
    );
    let actual_type: String = match body.relation_type.as_str() {
        "blocking" => "blocked_by".into(),
        "start_after" => "start_before".into(),
        "finish_after" => "finish_before".into(),
        "implements" => "implemented_by".into(),
        // Pass-through. Truncamos a 20 chars (límite de columna en BD) para
        // evitar 500 desde sea-orm si llega un valor absurdamente largo.
        other => other.chars().take(20).collect(),
    };

    let now: DateTime<FixedOffset> = Utc::now().into();
    let workspace_id = guard.workspace.id;
    let project_id = guard.project.id;
    let user_id = guard.user.id;

    // Bulk-create con paridad `IssueRelation.objects.bulk_create([...],
    // ignore_conflicts=True)`. Sea-ORM no expone `ignore_conflicts` directo;
    // usamos `on_conflict().do_nothing()` vía sea-query para el mismo efecto:
    // si ya existe la relación (por unique constraint), no falla.
    use sea_orm::sea_query::OnConflict;

    let models: Vec<issue_relations::ActiveModel> = body
        .issues
        .iter()
        .map(|other_id| {
            // Inversión de orientación cuando el tipo es inverso:
            //   - "blocking":     A→B(blocking) se guarda como B→A(blocked_by)
            //   - "start_after":  A→B(start_after) se guarda como B→A(start_before)
            //   - etc.
            // El `issue_id` del path es A; los `body.issues` son los B.
            let (src, dst) = if is_inverse {
                (*other_id, issue_id)
            } else {
                (issue_id, *other_id)
            };
            issue_relations::ActiveModel {
                id: Set(Uuid::new_v4()),
                relation_type: Set(actual_type.clone()),
                issue_id: Set(src),
                related_issue_id: Set(dst),
                project_id: Set(project_id),
                workspace_id: Set(workspace_id),
                created_by_id: Set(Some(user_id)),
                updated_by_id: Set(Some(user_id)),
                created_at: Set(now),
                updated_at: Set(now),
                deleted_at: Set(None),
            }
        })
        .collect();

    // Insert con tolerancia a duplicados — paridad `ignore_conflicts=True`.
    // La unique constraint en BD es `(issue_id, related_issue_id, deleted_at)`
    // (`migration/src/sql/baseline.sql:3504`), NO incluye `relation_type`.
    //
    // Decisión de diseño de Plane: un par (A, B) admite una sola relación
    // viva (deleted_at IS NULL) sin importar el tipo. Si el cliente intenta
    // crear A→B "blocked_by" cuando ya existe A→B "duplicate", el segundo
    // INSERT colisiona y se ignora — paridad con
    // `IssueRelation.objects.bulk_create([...], ignore_conflicts=True)`.
    //
    // Patrón sea-orm: `.on_conflict(...).do_nothing()` requiere el
    // `.do_nothing()` final para devolver `TryInsertResult` y manejar el
    // caso "0 rows inserted" sin propagar error (ver
    // `src/routes/projects.rs:1936-1947` para precedente en este codebase).
    issue_relations::Entity::insert_many(models)
        .on_conflict(
            OnConflict::columns([
                issue_relations::Column::IssueId,
                issue_relations::Column::RelatedIssueId,
                issue_relations::Column::DeletedAt,
            ])
            .do_nothing()
            .to_owned(),
        )
        .do_nothing()
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Releemos las relaciones recién creadas/existentes para devolver el
    // shape esperado por el cliente. No usamos los `models` locales porque
    // `on_conflict do_nothing` no garantiza que se hayan persistido (puede
    // haber colisión); la lectura es la fuente de verdad.
    let created_pairs: Vec<(Uuid, Uuid)> = body
        .issues
        .iter()
        .map(|other_id| {
            if is_inverse {
                (*other_id, issue_id)
            } else {
                (issue_id, *other_id)
            }
        })
        .collect();

    let mut resp: Vec<IssueRelationResponse> = Vec::with_capacity(created_pairs.len());
    for (src, dst) in created_pairs {
        // No filtramos por relation_type: la unique constraint en BD
        // permite una sola relación viva por par (src, dst) sin importar
        // el tipo. Si ya existía con un tipo distinto, esa es la fuente
        // de verdad y la devolvemos al cliente.
        if let Some(r) = issue_relations::Entity::find()
            .active()
            .filter(issue_relations::Column::IssueId.eq(src))
            .filter(issue_relations::Column::RelatedIssueId.eq(dst))
            .filter(issue_relations::Column::ProjectId.eq(project_id))
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
        {
            resp.push(IssueRelationResponse {
                id: r.id,
                relation_type: r.relation_type,
                issue_id: r.issue_id,
                related_issue_id: r.related_issue_id,
                project_id: r.project_id,
                workspace_id: r.workspace_id,
                created_at: r.created_at,
            });
        }
    }

    Ok((StatusCode::CREATED, Json(resp)))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/
///
/// Paridad Django (apps/api/plane/app/urls/issue.py:241-242): el endpoint usa
/// **POST**, no DELETE. La acción es semánticamente "remove" pero Django la
/// despacha vía POST porque el cliente envía un body con `relation_type` y
/// `related_issue` — DELETE con body no es universalmente soportado.
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Removed"))
)]
pub async fn remove_issue_relation(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<RemoveRelationRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // Paridad Django (`apps/api/plane/app/views/issue/relation.py:265-269`):
    //   IssueRelation.objects.filter(workspace__slug=slug).filter(
    //     Q(issue_id=related_issue, related_issue_id=issue_id) |
    //     Q(issue_id=issue_id, related_issue_id=related_issue)
    //   )
    //
    // La búsqueda es bidireccional porque la relación se modela como dirigida
    // pero el cliente puede invocar `remove-relation` desde cualquiera de los
    // dos extremos. Antes filtrábamos solo `(issue_id, related_issue_id)` →
    // devolvía 404 cuando la relación existía con orientación inversa.
    //
    // Filtramos además por `project_id` del guard (no por workspace slug
    // como Django) para mantener el aislamiento por proyecto que ya impone
    // `ProjectMemberGuard` en el resto de handlers — evita filtrar relaciones
    // cruzadas entre proyectos del mismo workspace.
    //
    // No filtramos por `relation_type`: Django tampoco lo hace, y mantener la
    // restricción aquí provocaría 404s falsos cuando el cliente envía un
    // `relation_type` que no coincide con el almacenado tras `get_actual_relation`
    // (ej. cliente manda "blocking", BD guarda "blocked_by" en el extremo opuesto).
    let relation = issue_relations::Entity::find()
        .active()
        .filter(issue_relations::Column::ProjectId.eq(guard.project.id))
        .filter(
            Condition::any()
                .add(
                    Condition::all()
                        .add(issue_relations::Column::IssueId.eq(issue_id))
                        .add(issue_relations::Column::RelatedIssueId.eq(body.related_issue)),
                )
                .add(
                    Condition::all()
                        .add(issue_relations::Column::IssueId.eq(body.related_issue))
                        .add(issue_relations::Column::RelatedIssueId.eq(issue_id)),
                ),
        )
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Idempotencia DELETE-like: si la relación ya no existe, devolvemos 204
    // sin error. Es semánticamente correcto (la postcondición "no existe la
    // relación" se cumple) y evita filtrar a clientes la existencia/ausencia
    // de un par de UUIDs específico (mitigación enumeración). Django no lo
    // maneja explícitamente — `.first().delete()` crashearía con AttributeError
    // sobre None — pero aquí preferimos robustez.
    if let Some(relation) = relation {
        let mut am: issue_relations::ActiveModel = relation.into();
        am.deleted_at = Set(Some(Utc::now().into()));
        am.update(&state.db).await.map_err(AppError::Database)?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Activity / History ──────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/history/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Activity log"))
)]
pub async fn list_issue_activities(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let activities = issue_activities::Entity::find()
        .active()
        .filter(issue_activities::Column::IssueId.eq(issue_id))
        .filter(issue_activities::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(issue_activities::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Batch-load actors
    let actor_ids: Vec<Uuid> = activities.iter()
        .filter_map(|a| a.actor_id).collect::<std::collections::HashSet<_>>()
        .into_iter().collect();
    let actor_map: std::collections::HashMap<Uuid, users::Model> =
        users::Entity::find()
            .filter(users::Column::Id.is_in(actor_ids))
            .all(&state.db).await.map_err(AppError::Database)?
            .into_iter().map(|u| (u.id, u)).collect();

    let project_detail = serde_json::json!({
        "id": guard.project.id,
        "name": guard.project.name,
        "identifier": guard.project.identifier,
        "logo_props": guard.project.logo_props,
    });
    let workspace_detail = serde_json::json!({
        "id": guard.workspace.id,
        "name": guard.workspace.name,
        "slug": guard.workspace.slug,
    });

    let resp: Vec<ActivityResponse> = activities.iter().map(|a| {
        let actor_detail = a.actor_id.and_then(|id| actor_map.get(&id)).map(|u| user_to_lite(u, false));
        ActivityResponse {
            id: a.id, verb: a.verb.clone(), field: a.field.clone(),
            old_value: a.old_value.clone(), new_value: a.new_value.clone(),
            old_identifier: a.old_identifier.clone(), new_identifier: a.new_identifier.clone(),
            comment: a.comment.clone(), actor_id: a.actor_id, actor_detail,
            issue_id: a.issue_id, issue_comment: a.issue_comment.clone(),
            project_id: a.project_id,
            project_detail: Some(project_detail.clone()),
            workspace_detail: Some(workspace_detail.clone()),
            attachments: vec![],
            created_at: a.created_at, updated_at: a.updated_at,
            created_by: a.created_by_id, updated_by: a.updated_by_id,
            access: "INTERNAL".to_owned(),
        }
    }).collect();

    Ok(Json(resp))
}

/// GET /workspaces/{slug}/projects/{project_id}/{issues|work-items}/{issue_id}/activities/{pk}
///
/// Devuelve una actividad concreta del issue. 404 si no existe.
/// Espejo del path-by-pk usado por el frontend para deep-linking a una
/// entrada específica del activity log.
#[utoipa::path(
    get,
    path = "/workspaces/{slug}/projects/{project_id}/work-items/{issue_id}/activities/{pk}/",
    tag = "Issues",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "Actividad encontrada"),
        (status = 401, description = "Unauthenticated"),
        (status = 404, description = "Actividad inexistente"),
    )
)]
pub async fn get_issue_activity(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, pk)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let act = issue_activities::Entity::find_by_id(pk)
        .active()
        .filter(issue_activities::Column::IssueId.eq(issue_id))
        .filter(issue_activities::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let actor_detail = if let Some(uid) = act.actor_id {
        users::Entity::find_by_id(uid).one(&state.db).await.map_err(AppError::Database)?
            .map(|u| user_to_lite(&u, false))
    } else { None };

    let project_detail = serde_json::json!({
        "id": guard.project.id, "name": guard.project.name,
        "identifier": guard.project.identifier, "logo_props": guard.project.logo_props,
    });
    let workspace_detail = serde_json::json!({
        "id": guard.workspace.id, "name": guard.workspace.name, "slug": guard.workspace.slug,
    });

    Ok(Json(ActivityResponse {
        id: act.id, verb: act.verb, field: act.field,
        old_value: act.old_value, new_value: act.new_value,
        old_identifier: act.old_identifier, new_identifier: act.new_identifier,
        comment: act.comment, actor_id: act.actor_id, actor_detail,
        issue_id: act.issue_id, issue_comment: act.issue_comment,
        project_id: act.project_id,
        project_detail: Some(project_detail),
        workspace_detail: Some(workspace_detail),
        attachments: vec![],
        created_at: act.created_at, updated_at: act.updated_at,
        created_by: act.created_by_id, updated_by: act.updated_by_id,
        access: "INTERNAL".to_owned(),
    }))
}

// ── Issue Subscribers ─────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Subscribers"))
)]
pub async fn list_issue_subscribers(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let subs = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .filter(issue_subscribers::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(issue_subscribers::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<SubscriberResponse> = subs.iter().map(|s| SubscriberResponse {
        id: s.id, subscriber_id: s.subscriber_id, issue_id: s.issue_id,
        project_id: s.project_id, created_at: s.created_at,
    }).collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/
/// Suscribe al usuario autenticado al issue.
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Subscribed"))
)]
pub async fn subscribe_to_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    // Idempotent
    let existing = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    if existing.is_some() {
        return Ok(StatusCode::CREATED);
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_sub = issue_subscribers::ActiveModel {
        id: Set(Uuid::new_v4()),
        issue_id: Set(issue_id),
        subscriber_id: Set(guard.user.id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };
    new_sub.insert(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::CREATED)
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/
/// Desuscribe al usuario autenticado del issue.
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Unsubscribed"))
)]
pub async fn unsubscribe_from_issue(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let sub = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_subscribers::ActiveModel = sub.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Sub-Issues ────────────────────────────────────────────────────────────────

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 200, description = "Sub-issues"))
)]
pub async fn list_sub_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_GUEST)?;

    let sub_issues = issues::Entity::find()
        .active()
        .filter(issues::Column::ParentId.eq(issue_id))
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .order_by_asc(issues::Column::SequenceId)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Devuelve los IDs y secuencias — el cliente carga el detalle si necesita
    let resp: Vec<serde_json::Value> = sub_issues.iter().map(|i| serde_json::json!({
        "id": i.id,
        "sequence_id": i.sequence_id,
        "name": i.name,
        "state_id": i.state_id,
        "priority": i.priority,
        "project_id": i.project_id,
    })).collect();

    Ok(Json(resp))
}

// ── POST sub-issues: asignación masiva de parent ──────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AssignSubIssuesRequest {
    /// IDs de issues a re-parentar hacia `issue_id`. Debe tener al menos 1.
    pub sub_issue_ids: Vec<Uuid>,
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/
///
/// Asigna múltiples issues como sub-issues del issue `{issue_id}` (bulk set
/// de `parent_id`).
///
/// Mirror exacto de `SubIssuesEndpoint.post`
/// (`apps/api/plane/app/views/issue/sub_issue.py:173-217`):
/// - Lista vacía → 400.
/// - Bulk update de `parent` en todos los `sub_issue_ids`.
/// - Response: `{sub_issues: [...], state_distribution: {group: [id, ...]}}`.
///
/// Permisos: equivalente a `ProjectEntityPermission` Django (ADMIN / MEMBER).
/// Django no expone esta acción a GUEST porque modifica relaciones — aquí se
/// exige ROLE_MEMBER para mantener esa barrera.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/sub-issues/",
    tag = "Issues",
    security(("TokenAuth" = [])),
    request_body = AssignSubIssuesRequest,
    responses(
        (status = 200, description = "Sub-issues asignados"),
        (status = 400, description = "sub_issue_ids vacío"),
        (status = 403, description = "Sin permisos"),
    )
)]
pub async fn assign_sub_issues(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<AssignSubIssuesRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Django permite ADMIN/MEMBER (ProjectEntityPermission). Mantenemos esa
    // barrera — GUEST no puede re-parentar issues.
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    // Mirror: `if not len(sub_issue_ids): return 400`.
    if body.sub_issue_ids.is_empty() {
        return Err(AppError::BadRequest("Sub Issue IDs are required".into()));
    }

    // Validar que el parent existe en el mismo workspace y project.
    // Django solo hace `Issue.issue_objects.get(pk=issue_id)` pero eso puede
    // re-parentar un sub-issue bajo un parent de otro workspace — bug sutil que
    // prevenimos aquí. Filtro explícito por guard.workspace.id / project.id.
    let parent = issues::Entity::find_by_id(issue_id)
        .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Bulk update: `UPDATE issues SET parent_id = $1 WHERE id IN ($2...)`
    // con filtro de seguridad por workspace_id + project_id para evitar que
    // un sub_issue_id de otro workspace/proyecto se reparente cross-boundary.
    let now: DateTime<FixedOffset> = Utc::now().into();
    issues::Entity::update_many()
        .col_expr(issues::Column::ParentId, Expr::value(parent.id))
        .col_expr(issues::Column::UpdatedAt, Expr::value(now))
        .col_expr(issues::Column::UpdatedById, Expr::value(guard.user.id))
        .filter(issues::Column::Id.is_in(body.sub_issue_ids.clone()))
        .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::DeletedAt.is_null())
        .exec(&state.db)
        .await
        .map_err(AppError::Database)?;

    // Re-consulta los sub-issues actualizados para armar el response.
    // Aplicamos el mismo guardrail de workspace/project al leer.
    let updated = issues::Entity::find()
        .filter(issues::Column::Id.is_in(body.sub_issue_ids))
        .filter(issues::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(issues::Column::ProjectId.eq(guard.project.id))
        .filter(issues::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    // `state_distribution`: diccionario `{state.group: [issue_id, ...]}`.
    // Django lo construye con `F("state__group")` en un annotate; aquí lo
    // resolvemos con una query extra sobre los state_ids presentes.
    let state_ids: Vec<Uuid> = updated.iter().filter_map(|i| i.state_id).collect();
    let states_map: std::collections::HashMap<Uuid, String> = if state_ids.is_empty() {
        Default::default()
    } else {
        states::Entity::find()
            .filter(states::Column::Id.is_in(state_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|s| (s.id, s.group))
            .collect()
    };

    let mut state_distribution: std::collections::HashMap<String, Vec<Uuid>> =
        std::collections::HashMap::new();
    for issue in &updated {
        if let Some(sid) = issue.state_id {
            if let Some(group) = states_map.get(&sid) {
                state_distribution
                    .entry(group.clone())
                    .or_default()
                    .push(issue.id);
            }
        }
    }

    // Shape de cada sub_issue: mismo shape que `list_sub_issues` GET — así el
    // frontend consume ambos endpoints con el mismo tipo. Paridad funcional
    // con Django aunque IssueSerializer devuelva más campos; si más adelante
    // el cliente necesita campos extra, se extiende en el GET y el POST juntos.
    let sub_issues_payload: Vec<serde_json::Value> = updated
        .iter()
        .map(|i| {
            serde_json::json!({
                "id": i.id,
                "sequence_id": i.sequence_id,
                "name": i.name,
                "state_id": i.state_id,
                "priority": i.priority,
                "project_id": i.project_id,
                "parent_id": i.parent_id,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "sub_issues": sub_issues_payload,
        "state_distribution": state_distribution,
    })))
}


// ─── GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/ ──

/// Retorna el detalle de un comentario específico.
///
/// Espejo de `IssueCommentViewSet.retrieve`
/// (`apps/api/plane/app/views/issue/comment.py`).
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/comments/{pk}/",
    tag = "Issues",
    security(("TokenAuth" = []))
)]
pub async fn get_comment(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, _issue_id, pk)): Path<(String, uuid::Uuid, uuid::Uuid, uuid::Uuid)>,
) -> Result<axum::Json<serde_json::Value>, AppError> {
    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::ProjectId.eq(guard.project.id))
        .filter(issue_comments::Column::WorkspaceId.eq(guard.workspace.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let actor = if let Some(actor_id) = comment.actor_id {
        users::Entity::find_by_id(actor_id)
            .one(&state.db)
            .await
            .map_err(AppError::Database)?
            .map(|u| crate::routes::workspaces::user_to_lite(&u, true))
    } else {
        None
    };

    Ok(axum::Json(serde_json::json!({
        "id": comment.id,
        "comment_html": comment.comment_html,
        "comment_stripped": comment.comment_stripped,
        "actor_id": comment.actor_id,
        "actor": actor,
        "issue_id": comment.issue_id,
        "project_id": comment.project_id,
        "workspace_id": comment.workspace_id,
        "created_at": comment.created_at,
        "updated_at": comment.updated_at,
    })))
}

// ─── DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}/ ──

/// Elimina la suscripción de un usuario específico a un issue.
///
/// Espejo de `IssueSubscriberViewSet.destroy`
/// (`apps/api/plane/app/views/issue/subscriber.py`).
#[utoipa::path(
    delete,
    path = "/api/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/issue-subscribers/{subscriber_id}/",
    tag = "Issues",
    security(("TokenAuth" = []), ("SessionCookie" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("project_id" = Uuid, Path, description = "Project UUID"),
        ("issue_id" = Uuid, Path, description = "Issue UUID"),
        ("subscriber_id" = Uuid, Path, description = "Subscriber user UUID"),
    ),
    responses(
        (status = 204, description = "Unsubscribed"),
        (status = 404, description = "Subscription not found"),
    )
)]
pub async fn delete_issue_subscriber(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, subscriber_id)): Path<(String, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let sub = issue_subscribers::Entity::find()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(subscriber_id))
        .filter(issue_subscribers::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let am: issue_subscribers::ActiveModel = sub.into();
    am.delete(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/
/// Checks whether the current user is subscribed to the issue.
/// Returns { subscribed: bool } — mirror of IssueSubscriberEndpoint.get in Django.
#[utoipa::path(
    get, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/subscribe/",
    tag = "Issues", security((("sessionAuth" = []))),
    responses(
        (status = 200, description = "Subscription status"),
    )
)]
pub async fn get_issue_subscription_status(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
) -> Result<impl IntoResponse, AppError> {
    let subscribed = issue_subscribers::Entity::find()
        .active()
        .filter(issue_subscribers::Column::IssueId.eq(issue_id))
        .filter(issue_subscribers::Column::SubscriberId.eq(guard.user.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    Ok(Json(serde_json::json!({ "subscribed": subscribed })))
}
