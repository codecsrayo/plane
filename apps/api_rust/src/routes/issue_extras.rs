// src/routes/issue_extras.rs
//! Sub-endpoints de issues: comments, reactions, relations, links, subscribers, activities.
//!
//! Equivalente a `plane/app/views/issue/` (múltiples archivos) en Django.
//!
//! Rutas implementadas:
//!   GET/POST   issues/{id}/comments/
//!   GET/PATCH/DELETE issues/{id}/comments/{pk}/
//!   POST/DELETE issues/{id}/reactions/{code}/
//!   POST/DELETE comments/{id}/reactions/{code}/
//!   GET/POST   issues/{id}/issue-links/
//!   PATCH/DELETE issues/{id}/issue-links/{pk}/
//!   GET/POST   issues/{id}/issue-relation/
//!   DELETE     issues/{id}/remove-relation/
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
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
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
        issue_reactions, issue_relations, issue_subscribers, issues,
    },
    error::AppError,
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct CreateCommentRequest {
    pub comment_html: String,
    pub access: Option<String>,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentRequest {
    pub comment_html: Option<String>,
    pub access: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
pub struct CommentReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub comment_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Deserialize)]
pub struct CreateLinkRequest {
    pub url: String,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLinkRequest {
    pub url: Option<String>,
    pub title: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct IssueRelationResponse {
    pub id: Uuid,
    pub relation_type: String,
    pub issue_id: Uuid,
    pub related_issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationRequest {
    pub relation_type: String,
    pub related_issue_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct RemoveRelationRequest {
    pub relation_type: String,
    pub related_issue_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct ActivityResponse {
    pub id: Uuid,
    pub verb: String,
    pub field: Option<String>,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub comment: String,
    pub actor_id: Option<Uuid>,
    pub issue_id: Option<Uuid>,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize)]
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
    let stripped = strip_html(&body.comment_html);
    let new_comment = issue_comments::ActiveModel {
        id: Set(Uuid::new_v4()),
        comment_html: Set(body.comment_html),
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
        am.comment_stripped = Set(strip_html(&html));
        am.comment_html = Set(html);
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
        .order_by_asc(issue_reactions::Column::CreatedAt)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let resp: Vec<ReactionResponse> = reactions.iter().map(|r| ReactionResponse {
        id: r.id,
        reaction: r.reaction.clone(),
        issue_id: r.issue_id,
        actor_id: r.actor_id,
        project_id: r.project_id,
        created_at: r.created_at,
    }).collect();

    Ok(Json(resp))
}

/// POST /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Reaction added"))
)]
pub async fn add_issue_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id, reaction_code)): Path<(String, Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

    // Idempotent — evitar duplicados
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
            created_at: r.created_at,
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
        created_at: created.created_at,
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/reactions/{reaction_code}/
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

/// POST /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/
#[utoipa::path(
    post, path = "/workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 201, description = "Reaction added"))
)]
pub async fn add_comment_reaction(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, comment_id, reaction_code)): Path<(String, Uuid, Uuid, String)>,
) -> Result<impl IntoResponse, AppError> {
    require_role(guard.project_member.role, guard.workspace_member.role, ROLE_MEMBER)?;

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
            id: r.id, reaction: r.reaction, comment_id: r.comment_id,
            actor_id: r.actor_id, project_id: r.project_id, created_at: r.created_at,
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
        id: created.id, reaction: created.reaction, comment_id: created.comment_id,
        actor_id: created.actor_id, project_id: created.project_id, created_at: created.created_at,
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/comments/{comment_id}/reactions/{reaction_code}/
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

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_link = issue_links::ActiveModel {
        id: Set(Uuid::new_v4()),
        url: Set(body.url),
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

    let valid_types = ["duplicate", "relates_to", "blocked_by", "blocking"];
    if !valid_types.contains(&body.relation_type.as_str()) {
        return Err(AppError::BadRequest(format!(
            "relation_type must be one of: {}", valid_types.join(", ")
        )));
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let new_relation = issue_relations::ActiveModel {
        id: Set(Uuid::new_v4()),
        relation_type: Set(body.relation_type),
        issue_id: Set(issue_id),
        related_issue_id: Set(body.related_issue_id),
        project_id: Set(guard.project.id),
        workspace_id: Set(guard.workspace.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_relation.insert(&state.db).await.map_err(AppError::Database)?;
    Ok((StatusCode::CREATED, Json(IssueRelationResponse {
        id: created.id, relation_type: created.relation_type,
        issue_id: created.issue_id, related_issue_id: created.related_issue_id,
        project_id: created.project_id, workspace_id: created.workspace_id,
        created_at: created.created_at,
    })))
}

/// DELETE /workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/
#[utoipa::path(
    delete, path = "/workspaces/{slug}/projects/{project_id}/issues/{issue_id}/remove-relation/",
    tag = "Issues", security(("TokenAuth" = [])),
    responses((status = 204, description = "Removed"))
)]
pub async fn remove_issue_relation(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Path((_slug, _project_id, issue_id)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<RemoveRelationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let relation = issue_relations::Entity::find()
        .active()
        .filter(issue_relations::Column::IssueId.eq(issue_id))
        .filter(issue_relations::Column::RelatedIssueId.eq(body.related_issue_id))
        .filter(issue_relations::Column::RelationType.eq(&body.relation_type))
        .filter(issue_relations::Column::ProjectId.eq(guard.project.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_relations::ActiveModel = relation.into();
    am.deleted_at = Set(Some(Utc::now().into()));
    am.update(&state.db).await.map_err(AppError::Database)?;

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

    let resp: Vec<ActivityResponse> = activities.iter().map(|a| ActivityResponse {
        id: a.id, verb: a.verb.clone(), field: a.field.clone(),
        old_value: a.old_value.clone(), new_value: a.new_value.clone(),
        comment: a.comment.clone(), actor_id: a.actor_id, issue_id: a.issue_id,
        project_id: a.project_id, workspace_id: a.workspace_id, created_at: a.created_at,
    }).collect();

    Ok(Json(resp))
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
