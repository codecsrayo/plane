// src/routes/space.rs
//! Public Space/Deploy Board API — mirrors `plane.space` Django app.
//!
//! All endpoints are served under `/api/public/` and correspond to
//! Django's `api/public/` URL prefix (`plane/urls.py:19`).
//!
//! GET endpoints are unauthenticated (AllowAny).
//! Write operations (POST/PATCH/DELETE for comments, reactions, votes)
//! require session or API-key authentication (IsAuthenticated).
//!
//! Endpoints implemented:
//!   GET    anchor/{anchor}/settings
//!   GET    anchor/{anchor}/meta
//!   GET    workspaces/{slug}/projects/{project_id}/anchor
//!   GET    anchor/{anchor}/issues
//!   GET    anchor/{anchor}/issues/{issue_id}
//!   GET    anchor/{anchor}/cycles
//!   GET    anchor/{anchor}/modules
//!   GET    anchor/{anchor}/states
//!   GET    anchor/{anchor}/labels
//!   GET    anchor/{anchor}/members
//!   GET/POST   anchor/{anchor}/issues/{issue_id}/comments
//!   PATCH/DELETE anchor/{anchor}/issues/{issue_id}/comments/{pk}
//!   GET/POST   anchor/{anchor}/issues/{issue_id}/reactions
//!   DELETE     anchor/{anchor}/issues/{issue_id}/reactions/{reaction_code}
//!   GET/POST   anchor/{anchor}/comments/{comment_id}/reactions
//!   DELETE     anchor/{anchor}/comments/{comment_id}/reactions/{reaction_code}
//!   GET/POST/DELETE anchor/{anchor}/issues/{issue_id}/votes
//!   GET/POST   anchor/{anchor}/intakes/{intake_id}/intake-issues
//!   GET/PATCH/DELETE anchor/{anchor}/intakes/{intake_id}/intake-issues/{pk}
//!   GET    workspaces/{slug}/project-boards

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, Order, QueryFilter, QueryOrder,
    QuerySelect,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{
        comment_reactions, cycles, deploy_boards, issue_comments, issue_reactions, issue_votes,
        issues, labels, modules, project_members, project_public_members, projects, states, users,
        workspaces, issue_assignees, issue_labels, module_issues, cycle_issues, intake_issues,
        intakes,
    },
    error::AppError,
    routes::{
        helpers::workspace_by_slug,
        workspaces::user_to_lite,
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Helper: lookup a deploy board by anchor ───────────────────────────────────

/// Looks up a non-deleted `deploy_boards` row by anchor string.
/// Returns `AppError::NotFound` if absent.
async fn deploy_board_by_anchor(
    db: &sea_orm::DatabaseConnection,
    anchor: &str,
) -> Result<deploy_boards::Model, AppError> {
    deploy_boards::Entity::find()
        .filter(deploy_boards::Column::Anchor.eq(anchor))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

/// Same as `deploy_board_by_anchor` but also filters to `entity_name = "project"`.
async fn project_deploy_board_by_anchor(
    db: &sea_orm::DatabaseConnection,
    anchor: &str,
) -> Result<deploy_boards::Model, AppError> {
    deploy_boards::Entity::find()
        .filter(deploy_boards::Column::Anchor.eq(anchor))
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

// ── DTOs ──────────────────────────────────────────────────────────────────────

/// Public settings response — mirrors `DeployBoardSerializer`.
/// Same shape as `projects::DeployBoardResponse` but re-declared here so
/// this module is self-contained and doesn't create a cross-module dependency
/// on the private `projects` struct.
#[derive(Debug, Serialize)]
pub struct PublicBoardSettingsResponse {
    pub id: Uuid,
    pub anchor: String,
    pub is_comments_enabled: bool,
    pub is_reactions_enabled: bool,
    pub is_votes_enabled: bool,
    pub is_activity_enabled: bool,
    pub is_disabled: bool,
    pub view_props: serde_json::Value,
    pub intake_id: Option<Uuid>,
    #[serde(rename = "project")]
    pub project_id: Option<Uuid>,
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    pub entity_name: Option<String>,
    pub entity_identifier: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub inbox: Option<serde_json::Value>,
    pub project_details: Option<serde_json::Value>,
    pub workspace_detail: Option<serde_json::Value>,
}

impl From<deploy_boards::Model> for PublicBoardSettingsResponse {
    fn from(m: deploy_boards::Model) -> Self {
        Self {
            id: m.id,
            anchor: m.anchor,
            is_comments_enabled: m.is_comments_enabled,
            is_reactions_enabled: m.is_reactions_enabled,
            is_votes_enabled: m.is_votes_enabled,
            is_activity_enabled: m.is_activity_enabled,
            is_disabled: m.is_disabled,
            view_props: m.view_props,
            intake_id: m.intake_id,
            project_id: m.project_id,
            workspace_id: m.workspace_id,
            entity_name: m.entity_name,
            entity_identifier: m.entity_identifier,
            created_at: m.created_at.into(),
            updated_at: m.updated_at.into(),
            created_by: m.created_by_id,
            updated_by: m.updated_by_id,
            inbox: None,
            project_details: None,
            workspace_detail: None,
        }
    }
}

/// Lite project metadata — mirrors `ProjectLiteSerializer`.
#[derive(Debug, Serialize)]
pub struct ProjectMetaResponse {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub cover_image: Option<String>,
    pub icon_prop: Option<serde_json::Value>,
    pub emoji: Option<String>,
    pub description: String,
}

impl From<projects::Model> for ProjectMetaResponse {
    fn from(p: projects::Model) -> Self {
        Self {
            id: p.id,
            identifier: p.identifier,
            name: p.name,
            cover_image: p.cover_image,
            icon_prop: p.icon_prop,
            emoji: p.emoji,
            description: p.description,
        }
    }
}

/// Public issue shape — mirrors `IssuePublicSerializer`.
#[derive(Debug, Serialize)]
pub struct PublicIssueResponse {
    pub id: Uuid,
    pub name: String,
    pub sequence_id: i32,
    /// state FK as UUID (Django field name "state")
    #[serde(rename = "state")]
    pub state_id: Option<Uuid>,
    /// project FK as UUID
    pub project: Uuid,
    /// workspace FK as UUID
    pub workspace: Uuid,
    pub priority: String,
    pub target_date: Option<chrono::NaiveDate>,
    pub label_ids: Vec<Uuid>,
    pub assignee_ids: Vec<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub cycle_id: Option<Uuid>,
    /// created_by FK as UUID
    pub created_by: Option<Uuid>,
    pub reactions: Vec<serde_json::Value>,
    pub votes: Vec<serde_json::Value>,
}

/// Public comment response.
#[derive(Debug, Serialize)]
pub struct PublicCommentResponse {
    pub id: Uuid,
    pub comment_html: String,
    pub comment_stripped: String,
    pub access: String,
    pub issue_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

/// Public reaction response.
#[derive(Debug, Serialize)]
pub struct PublicReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

/// Public comment reaction response.
#[derive(Debug, Serialize)]
pub struct PublicCommentReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub comment_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

/// Public vote response — mirrors `IssueVoteSerializer`.
#[derive(Debug, Serialize)]
pub struct PublicVoteResponse {
    pub id: Uuid,
    pub vote: i32,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

/// Request body for creating a public comment.
#[derive(Debug, Deserialize)]
pub struct CreatePublicCommentRequest {
    pub comment_html: String,
}

/// Request body for updating a public comment.
#[derive(Debug, Deserialize)]
pub struct UpdatePublicCommentRequest {
    pub comment_html: Option<String>,
}

/// Request body for creating a reaction.
#[derive(Debug, Deserialize)]
pub struct CreateReactionRequest {
    pub reaction: String,
}

/// Request body for creating a vote.
#[derive(Debug, Deserialize)]
pub struct CreateVoteRequest {
    pub vote: Option<i32>,
}

/// Pagination cursor query parameter shared by list endpoints.
#[derive(Debug, Deserialize)]
pub struct CursorQuery {
    pub cursor: Option<String>,
    pub per_page: Option<u64>,
}

// ── GET /api/public/anchor/{anchor}/settings ─────────────────────────────────

/// Mirror of `ProjectDeployBoardPublicSettingsEndpoint.get`.
/// Returns the deploy board settings for the published project.
pub async fn get_project_public_settings(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<PublicBoardSettingsResponse>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;
    Ok(Json(PublicBoardSettingsResponse::from(board)))
}

// ── GET /api/public/anchor/{anchor}/meta ─────────────────────────────────────

/// Mirror of `ProjectMetaDataEndpoint.get`.
/// Returns lite project metadata for the published project.
pub async fn get_project_meta(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<ProjectMetaResponse>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    let project_id = board.entity_identifier.ok_or(AppError::NotFound)?;
    let project = projects::Entity::find_by_id(project_id)
        .filter(projects::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ProjectMetaResponse::from(project)))
}

// ── GET /api/public/workspaces/{slug}/projects/{project_id}/anchor ───────────

/// Mirror of `WorkspaceProjectAnchorEndpoint.get`.
/// Returns the deploy board for the given workspace/project pair.
pub async fn get_project_anchor(
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<PublicBoardSettingsResponse>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    let board = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PublicBoardSettingsResponse::from(board)))
}

// ── GET /api/public/workspaces/{slug}/project-boards ─────────────────────────

/// Mirror of `WorkspaceProjectDeployBoardEndpoint.get`.
/// Returns all published projects in a workspace.
#[derive(Debug, Serialize)]
pub struct WorkspaceProjectBoardItem {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub description: String,
    pub emoji: Option<String>,
    pub icon_prop: Option<serde_json::Value>,
    pub cover_image: Option<String>,
    pub anchor: String,
}

pub async fn list_workspace_project_boards(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<WorkspaceProjectBoardItem>>, AppError> {
    let ws = workspace_by_slug(&state.db, &slug).await?;

    // Load all non-deleted project deploy boards for this workspace
    let boards = deploy_boards::Entity::find()
        .filter(deploy_boards::Column::WorkspaceId.eq(ws.id))
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .filter(deploy_boards::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let project_ids: Vec<Uuid> = boards
        .iter()
        .filter_map(|b| b.entity_identifier)
        .collect();

    let projects_list = projects::Entity::find()
        .filter(projects::Column::WorkspaceId.eq(ws.id))
        .filter(projects::Column::Id.is_in(project_ids))
        .filter(projects::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let board_map: std::collections::HashMap<Uuid, &deploy_boards::Model> = boards
        .iter()
        .filter_map(|b| b.entity_identifier.map(|eid| (eid, b)))
        .collect();

    let result = projects_list
        .into_iter()
        .filter_map(|p| {
            board_map.get(&p.id).map(|b| WorkspaceProjectBoardItem {
                id: p.id,
                identifier: p.identifier,
                name: p.name,
                description: p.description,
                emoji: p.emoji,
                icon_prop: p.icon_prop,
                cover_image: p.cover_image,
                anchor: b.anchor.clone(),
            })
        })
        .collect();

    Ok(Json(result))
}

// ── GET /api/public/anchor/{anchor}/cycles ────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicCycleItem {
    pub id: Uuid,
    pub name: String,
}

/// Mirror of `ProjectCyclesEndpoint.get`.
pub async fn list_public_cycles(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<PublicCycleItem>>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let items = cycles::Entity::find()
        .filter(cycles::Column::WorkspaceId.eq(board.workspace_id))
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        items
            .into_iter()
            .map(|c| PublicCycleItem { id: c.id, name: c.name })
            .collect(),
    ))
}

// ── GET /api/public/anchor/{anchor}/modules ───────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicModuleItem {
    pub id: Uuid,
    pub name: String,
}

/// Mirror of `ProjectModulesEndpoint.get`.
pub async fn list_public_modules(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<PublicModuleItem>>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let items = modules::Entity::find()
        .filter(modules::Column::WorkspaceId.eq(board.workspace_id))
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        items
            .into_iter()
            .map(|m| PublicModuleItem { id: m.id, name: m.name })
            .collect(),
    ))
}

// ── GET /api/public/anchor/{anchor}/states ────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicStateItem {
    pub id: Uuid,
    pub name: String,
    pub group: String,
    pub color: String,
    pub sequence: f64,
}

/// Mirror of `ProjectStatesEndpoint.get`. Excludes triage state.
pub async fn list_public_states(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<PublicStateItem>>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let items = states::Entity::find()
        .filter(states::Column::WorkspaceId.eq(board.workspace_id))
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::DeletedAt.is_null())
        // Exclude triage state — mirrors Django `~Q(name="Triage")`
        .filter(states::Column::Name.ne("Triage"))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        items
            .into_iter()
            .map(|s| PublicStateItem {
                id: s.id,
                name: s.name,
                group: s.group,
                color: s.color,
                sequence: s.sequence,
            })
            .collect(),
    ))
}

// ── GET /api/public/anchor/{anchor}/labels ────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicLabelItem {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub parent: Option<Uuid>,
}

/// Mirror of `ProjectLabelsEndpoint.get`.
pub async fn list_public_labels(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<PublicLabelItem>>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let items = labels::Entity::find()
        .filter(labels::Column::WorkspaceId.eq(board.workspace_id))
        .filter(labels::Column::ProjectId.eq(project_id))
        .filter(labels::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        items
            .into_iter()
            .map(|l| PublicLabelItem {
                id: l.id,
                name: l.name,
                color: l.color,
                parent: l.parent_id,
            })
            .collect(),
    ))
}

// ── GET /api/public/anchor/{anchor}/members ───────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicMemberItem {
    pub id: Uuid,
    pub member: Uuid,
    pub member__display_name: String,
    pub member__avatar: String,
}

/// Mirror of `ProjectMembersEndpoint.get`.
pub async fn list_public_members(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<PublicMemberItem>>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let members = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(board.workspace_id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let member_ids: Vec<Uuid> = members.iter().filter_map(|m| m.member_id).collect();
    let users_list = users::Entity::find()
        .filter(users::Column::Id.is_in(member_ids))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let users_map: std::collections::HashMap<Uuid, &users::Model> =
        users_list.iter().map(|u| (u.id, u)).collect();

    let result = members
        .iter()
        .filter_map(|m| {
            let member_id = m.member_id?;
            let user = *users_map.get(&member_id)?;
            Some(PublicMemberItem {
                id: m.id,
                member: member_id,
                member__display_name: user.display_name.clone(),
                member__avatar: user.avatar.clone(),
            })
        })
        .collect();

    Ok(Json(result))
}

// ── GET /api/public/anchor/{anchor}/issues ────────────────────────────────────

/// Mirror of `ProjectIssuesPublicEndpoint.get` (simplified flat list).
/// Returns issues in the published project with enrichment (label_ids,
/// assignee_ids, module_ids, cycle_id).
pub async fn list_public_issues(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
    Query(params): Query<CursorQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let per_page = params.per_page.unwrap_or(100).clamp(1, 1000);
    let page = params
        .cursor
        .as_deref()
        .and_then(|c| {
            let parts: Vec<&str> = c.splitn(3, ':').collect();
            if parts.len() == 3 {
                parts[1].parse::<u64>().ok()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let offset = page * per_page;

    let issue_list = issues::Entity::find()
        .filter(issues::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::DeletedAt.is_null())
        .filter(issues::Column::IsActive.is_not_null().or(issues::Column::IsActive.ne(false)))
        .order_by(issues::Column::CreatedAt, Order::Desc)
        .offset(offset)
        .limit(per_page + 1)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    let has_next = issue_list.len() as u64 > per_page;
    let page_issues: Vec<_> = issue_list.into_iter().take(per_page as usize).collect();

    let issue_ids: Vec<Uuid> = page_issues.iter().map(|i| i.id).collect();

    // Batch-load enrichment data
    let assignees_map = load_m2m_ids(
        &state.db,
        &issue_ids,
        issue_assignees::Entity,
        issue_assignees::Column::IssueId,
        issue_assignees::Column::AssigneeId,
    )
    .await?;

    let labels_map = load_m2m_ids(
        &state.db,
        &issue_ids,
        issue_labels::Entity,
        issue_labels::Column::IssueId,
        issue_labels::Column::LabelId,
    )
    .await?;

    let modules_map = load_m2m_ids_module(&state.db, &issue_ids).await?;

    let cycles_map = load_cycle_ids(&state.db, &issue_ids).await?;

    let issues_response: Vec<PublicIssueResponse> = page_issues
        .iter()
        .map(|i| PublicIssueResponse {
            id: i.id,
            name: i.name.clone(),
            sequence_id: i.sequence_id,
            state_id: i.state_id,
            project: i.project_id,
            workspace: i.workspace_id,
            priority: i.priority.clone(),
            target_date: i.target_date,
            label_ids: labels_map.get(&i.id).cloned().unwrap_or_default(),
            assignee_ids: assignees_map.get(&i.id).cloned().unwrap_or_default(),
            module_ids: modules_map.get(&i.id).cloned().unwrap_or_default(),
            cycle_id: cycles_map.get(&i.id).copied(),
            created_by: i.created_by_id,
            reactions: vec![],
            votes: vec![],
        })
        .collect();

    let total_count = issues_response.len() as u64 + offset;
    let next_cursor = if has_next {
        Some(format!("{}:{}:0", per_page, page + 1))
    } else {
        None
    };
    let prev_cursor = if page > 0 {
        Some(format!("{}:{}:1", per_page, page - 1))
    } else {
        None
    };

    Ok(Json(serde_json::json!({
        "count": total_count,
        "next_cursor": next_cursor,
        "prev_cursor": prev_cursor,
        "next_page_results": has_next,
        "results": issues_response,
    })))
}

// ── GET /api/public/anchor/{anchor}/issues/{issue_id} ────────────────────────

/// Mirror of `IssueRetrievePublicEndpoint.get`.
pub async fn get_public_issue(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<PublicIssueResponse>, AppError> {
    let board = deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let issue = issues::Entity::find_by_id(issue_id)
        .filter(issues::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let issue_ids = vec![issue.id];
    let assignees_map = load_m2m_ids(
        &state.db,
        &issue_ids,
        issue_assignees::Entity,
        issue_assignees::Column::IssueId,
        issue_assignees::Column::AssigneeId,
    )
    .await?;
    let labels_map = load_m2m_ids(
        &state.db,
        &issue_ids,
        issue_labels::Entity,
        issue_labels::Column::IssueId,
        issue_labels::Column::LabelId,
    )
    .await?;
    let modules_map = load_m2m_ids_module(&state.db, &issue_ids).await?;
    let cycles_map = load_cycle_ids(&state.db, &issue_ids).await?;

    Ok(Json(PublicIssueResponse {
        id: issue.id,
        name: issue.name,
        sequence_id: issue.sequence_id,
        state_id: issue.state_id,
        project: issue.project_id,
        workspace: issue.workspace_id,
        priority: issue.priority,
        target_date: issue.target_date,
        label_ids: labels_map.get(&issue.id).cloned().unwrap_or_default(),
        assignee_ids: assignees_map.get(&issue.id).cloned().unwrap_or_default(),
        module_ids: modules_map.get(&issue.id).cloned().unwrap_or_default(),
        cycle_id: cycles_map.get(&issue.id).copied(),
        created_by: issue.created_by_id,
        reactions: vec![],
        votes: vec![],
    }))
}

// ── Comments ─────────────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/comments
/// Mirror of `IssueCommentPublicViewSet.list`.
/// Returns only EXTERNAL comments when `is_comments_enabled`.
pub async fn list_public_issue_comments(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicCommentResponse>>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_comments_enabled {
        return Ok(Json(vec![]));
    }

    let comments = issue_comments::Entity::find()
        .active()
        .filter(issue_comments::Column::IssueId.eq(issue_id))
        .filter(issue_comments::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issue_comments::Column::Access.eq("EXTERNAL"))
        .order_by(issue_comments::Column::CreatedAt, Order::Asc)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        comments
            .into_iter()
            .map(|c| PublicCommentResponse {
                id: c.id,
                comment_html: c.comment_html,
                comment_stripped: c.comment_stripped,
                access: c.access,
                issue_id: c.issue_id,
                project_id: c.project_id,
                workspace_id: c.workspace_id,
                actor_id: c.actor_id,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })
            .collect(),
    ))
}

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/comments
/// Mirror of `IssueCommentPublicViewSet.create`. Requires auth.
pub async fn create_public_issue_comment(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    Json(body): Json<CreatePublicCommentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_comments_enabled {
        return Err(AppError::BadRequest(
            "Comments are not enabled for this project".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    if body.comment_html.trim().is_empty() {
        return Err(AppError::BadRequest("comment_html is required".into()));
    }

    let comment_html =
        crate::utils::content_validator::sanitize_description(&body.comment_html)
            .map_err(AppError::BadRequest)?;
    let stripped = strip_html_text(&comment_html);
    let now: DateTime<FixedOffset> = Utc::now().into();

    let new_comment = issue_comments::ActiveModel {
        id: Set(Uuid::new_v4()),
        comment_html: Set(comment_html),
        comment_stripped: Set(stripped),
        comment_json: Set(serde_json::json!({})),
        access: Set("EXTERNAL".to_string()),
        attachments: Set(vec![]),
        issue_id: Set(issue_id),
        project_id: Set(project_id),
        workspace_id: Set(board.workspace_id),
        actor_id: Set(Some(user.id)),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        parent_id: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        description_id: Set(None),
        edited_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
    };

    let created = new_comment.insert(&state.db).await.map_err(AppError::Database)?;

    // Track public member participation
    ensure_public_member(&state.db, project_id, board.workspace_id, user.id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublicCommentResponse {
            id: created.id,
            comment_html: created.comment_html,
            comment_stripped: created.comment_stripped,
            access: created.access,
            issue_id: created.issue_id,
            project_id: created.project_id,
            workspace_id: created.workspace_id,
            actor_id: created.actor_id,
            created_at: created.created_at,
            updated_at: created.updated_at,
        }),
    ))
}

/// PATCH /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}
/// Requires auth; only the comment author may edit.
pub async fn update_public_issue_comment(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, _issue_id, pk)): Path<(String, Uuid, Uuid)>,
    Json(body): Json<UpdatePublicCommentRequest>,
) -> Result<Json<PublicCommentResponse>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_comments_enabled {
        return Err(AppError::BadRequest(
            "Comments are not enabled for this project".into(),
        ));
    }

    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issue_comments::Column::Access.eq("EXTERNAL"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Only the original actor may edit
    if comment.actor_id != Some(user.id) {
        return Err(AppError::Forbidden);
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: issue_comments::ActiveModel = comment.into();

    if let Some(html) = body.comment_html {
        let clean = crate::utils::content_validator::sanitize_description(&html)
            .map_err(AppError::BadRequest)?;
        am.comment_stripped = Set(strip_html_text(&clean));
        am.comment_html = Set(clean);
        am.edited_at = Set(Some(now));
    }
    am.updated_at = Set(now);
    am.updated_by_id = Set(Some(user.id));

    let updated = am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(Json(PublicCommentResponse {
        id: updated.id,
        comment_html: updated.comment_html,
        comment_stripped: updated.comment_stripped,
        access: updated.access,
        issue_id: updated.issue_id,
        project_id: updated.project_id,
        workspace_id: updated.workspace_id,
        actor_id: updated.actor_id,
        created_at: updated.created_at,
        updated_at: updated.updated_at,
    }))
}

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}
/// Requires auth; only the comment author may delete.
pub async fn delete_public_issue_comment(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, _issue_id, pk)): Path<(String, Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issue_comments::Column::Access.eq("EXTERNAL"))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    if comment.actor_id != Some(user.id) {
        return Err(AppError::Forbidden);
    }

    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: issue_comments::ActiveModel = comment.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Reactions ───────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/reactions
pub async fn list_public_issue_reactions(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicReactionResponse>>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Ok(Json(vec![]));
    }

    let reactions = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(issue_reactions::Column::CreatedAt, Order::Desc)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        reactions
            .into_iter()
            .map(|r| PublicReactionResponse {
                id: r.id,
                reaction: r.reaction,
                issue_id: r.issue_id,
                actor_id: r.actor_id,
                project_id: r.project_id,
                workspace_id: r.workspace_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect(),
    ))
}

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/reactions
/// Requires auth.
pub async fn create_public_issue_reaction(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this project board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let reaction_code = body.reaction.trim().to_string();
    if reaction_code.is_empty() {
        return Err(AppError::BadRequest("reaction is required".into()));
    }

    // Idempotent
    let existing = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::ActorId.eq(user.id))
        .filter(issue_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let r = if let Some(r) = existing {
        r
    } else {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let new_reaction = issue_reactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            reaction: Set(reaction_code),
            issue_id: Set(issue_id),
            actor_id: Set(user.id),
            project_id: Set(project_id),
            workspace_id: Set(board.workspace_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_reaction.insert(&state.db).await.map_err(AppError::Database)?
    };

    ensure_public_member(&state.db, project_id, board.workspace_id, user.id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublicReactionResponse {
            id: r.id,
            reaction: r.reaction,
            issue_id: r.issue_id,
            actor_id: r.actor_id,
            project_id: r.project_id,
            workspace_id: r.workspace_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }),
    ))
}

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/reactions/{reaction_code}
/// Requires auth.
pub async fn delete_public_issue_reaction(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, issue_id, reaction_code)): Path<(String, Uuid, String)>,
) -> Result<StatusCode, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this project board".into(),
        ));
    }

    let reaction = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::ActorId.eq(user.id))
        .filter(issue_reactions::Column::Reaction.eq(&reaction_code))
        .filter(issue_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: issue_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Comment Reactions ─────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/comments/{comment_id}/reactions
pub async fn list_public_comment_reactions(
    State(state): State<AppState>,
    Path((anchor, comment_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicCommentReactionResponse>>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Ok(Json(vec![]));
    }

    let reactions = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(comment_reactions::Column::CreatedAt, Order::Desc)
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        reactions
            .into_iter()
            .map(|r| PublicCommentReactionResponse {
                id: r.id,
                reaction: r.reaction,
                comment_id: r.comment_id,
                actor_id: r.actor_id,
                project_id: r.project_id,
                workspace_id: r.workspace_id,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect(),
    ))
}

/// POST /api/public/anchor/{anchor}/comments/{comment_id}/reactions
/// Requires auth.
pub async fn create_public_comment_reaction(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, comment_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateReactionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let reaction_code = body.reaction.trim().to_string();
    if reaction_code.is_empty() {
        return Err(AppError::BadRequest("reaction is required".into()));
    }

    // Idempotent
    let existing = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::ActorId.eq(user.id))
        .filter(comment_reactions::Column::Reaction.eq(&reaction_code))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let r = if let Some(r) = existing {
        r
    } else {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let new_reaction = comment_reactions::ActiveModel {
            id: Set(Uuid::new_v4()),
            reaction: Set(reaction_code),
            comment_id: Set(comment_id),
            actor_id: Set(user.id),
            project_id: Set(project_id),
            workspace_id: Set(board.workspace_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_reaction.insert(&state.db).await.map_err(AppError::Database)?
    };

    ensure_public_member(&state.db, project_id, board.workspace_id, user.id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublicCommentReactionResponse {
            id: r.id,
            reaction: r.reaction,
            comment_id: r.comment_id,
            actor_id: r.actor_id,
            project_id: r.project_id,
            workspace_id: r.workspace_id,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }),
    ))
}

/// DELETE /api/public/anchor/{anchor}/comments/{comment_id}/reactions/{reaction_code}
/// Requires auth.
pub async fn delete_public_comment_reaction(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, comment_id, reaction_code)): Path<(String, Uuid, String)>,
) -> Result<StatusCode, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this board".into(),
        ));
    }

    let reaction = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::ActorId.eq(user.id))
        .filter(comment_reactions::Column::Reaction.eq(&reaction_code))
        .filter(comment_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: comment_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue Votes ───────────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/votes
pub async fn list_public_issue_votes(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicVoteResponse>>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if !board.is_votes_enabled {
        return Ok(Json(vec![]));
    }

    let votes = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::WorkspaceId.eq(board.workspace_id))
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        votes
            .into_iter()
            .map(|v| PublicVoteResponse {
                id: v.id,
                vote: v.vote,
                issue_id: v.issue_id,
                actor_id: v.actor_id,
                project_id: v.project_id,
                workspace_id: v.workspace_id,
                created_at: v.created_at,
                updated_at: v.updated_at,
            })
            .collect(),
    ))
}

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/votes
/// Mirror of `IssueVotePublicViewSet.create`. Requires auth.
/// Upserts a vote (get_or_create + update vote value).
pub async fn create_public_issue_vote(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    body: Option<Json<CreateVoteRequest>>,
) -> Result<impl IntoResponse, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let vote_value = body.as_ref().and_then(|b| b.vote).unwrap_or(1);

    let existing = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::ActorId.eq(user.id))
        .filter(issue_votes::Column::ProjectId.eq(project_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now: DateTime<FixedOffset> = Utc::now().into();

    let vote = if let Some(v) = existing {
        let mut am: issue_votes::ActiveModel = v.into();
        am.vote = Set(vote_value);
        am.updated_at = Set(now);
        am.update(&state.db).await.map_err(AppError::Database)?
    } else {
        let new_vote = issue_votes::ActiveModel {
            id: Set(Uuid::new_v4()),
            vote: Set(vote_value),
            issue_id: Set(issue_id),
            actor_id: Set(user.id),
            project_id: Set(project_id),
            workspace_id: Set(board.workspace_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_vote.insert(&state.db).await.map_err(AppError::Database)?
    };

    ensure_public_member(&state.db, project_id, board.workspace_id, user.id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublicVoteResponse {
            id: vote.id,
            vote: vote.vote,
            issue_id: vote.issue_id,
            actor_id: vote.actor_id,
            project_id: vote.project_id,
            workspace_id: vote.workspace_id,
            created_at: vote.created_at,
            updated_at: vote.updated_at,
        }),
    ))
}

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/votes
/// Requires auth.
pub async fn delete_public_issue_vote(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<StatusCode, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let vote = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::ActorId.eq(user.id))
        .filter(issue_votes::Column::ProjectId.eq(project_id))
        .filter(issue_votes::Column::WorkspaceId.eq(board.workspace_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let now: DateTime<FixedOffset> = Utc::now().into();
    let mut am: issue_votes::ActiveModel = vote.into();
    am.deleted_at = Set(Some(now));
    am.update(&state.db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Intake Issues (public) ────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicIntakeIssueResponse {
    pub id: Uuid,
    pub issue_id: Uuid,
    pub intake_id: Uuid,
    pub status: i32,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePublicIntakeIssueRequest {
    pub name: String,
    pub description_html: Option<String>,
    pub priority: Option<String>,
}

/// GET /api/public/anchor/{anchor}/intakes/{intake_id}/intake-issues
/// Mirror of `IntakeIssuePublicViewSet.list`. Requires `intake` enabled on board.
pub async fn list_public_intake_issues(
    State(state): State<AppState>,
    Path((anchor, intake_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicIntakeIssueResponse>>, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if board.intake_id.is_none() {
        return Err(AppError::BadRequest(
            "Intake is not enabled for this Project Board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let items = intake_issues::Entity::find()
        .filter(intake_issues::Column::IntakeId.eq(intake_id))
        .filter(intake_issues::Column::ProjectId.eq(project_id))
        .filter(intake_issues::Column::WorkspaceId.eq(board.workspace_id))
        .filter(intake_issues::Column::DeletedAt.is_null())
        .all(&state.db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        items
            .into_iter()
            .map(|i| PublicIntakeIssueResponse {
                id: i.id,
                issue_id: i.issue_id,
                intake_id: i.intake_id,
                status: i.status,
                project_id: i.project_id,
                workspace_id: i.workspace_id,
                created_at: i.created_at,
                updated_at: i.updated_at,
            })
            .collect(),
    ))
}

/// POST /api/public/anchor/{anchor}/intakes/{intake_id}/intake-issues
/// Requires auth.
pub async fn create_public_intake_issue(
    State(state): State<AppState>,
    AnyAuth(user): AnyAuth,
    Path((anchor, intake_id)): Path<(String, Uuid)>,
    Json(body): Json<CreatePublicIntakeIssueRequest>,
) -> Result<impl IntoResponse, AppError> {
    let board = project_deploy_board_by_anchor(&state.db, &anchor).await?;

    if board.intake_id.is_none() {
        return Err(AppError::BadRequest(
            "Intake is not enabled for this Project Board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    // Verify intake belongs to this project
    let intake = intakes::Entity::find_by_id(intake_id)
        .filter(intakes::Column::ProjectId.eq(project_id))
        .filter(intakes::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Get intake state (first triage state in the project)
    use crate::entities::states as states_ent;
    let triage_state = states_ent::Entity::find()
        .filter(states_ent::Column::ProjectId.eq(project_id))
        .filter(states_ent::Column::IsTriage.eq(true))
        .filter(states_ent::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let now: DateTime<FixedOffset> = Utc::now().into();

    // Create the issue
    let description_html = body.description_html.unwrap_or_default();
    let new_issue = issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(body.name),
        description_json: Set(serde_json::json!({})),
        description_html: Set(description_html.clone()),
        description_stripped: Set(Some(strip_html_text(&description_html))),
        priority: Set(body.priority.unwrap_or_else(|| "none".to_string())),
        start_date: Set(None),
        target_date: Set(None),
        sequence_id: Set(1), // will be overwritten by DB trigger in real system
        created_by_id: Set(Some(user.id)),
        parent_id: Set(None),
        project_id: Set(project_id),
        state_id: Set(triage_state.map(|s| s.id)),
        updated_by_id: Set(Some(user.id)),
        workspace_id: Set(board.workspace_id),
        sort_order: Set(0.0),
        is_draft: Set(false),
        archived_at: Set(None),
        external_id: Set(None),
        external_source: Set(None),
        description_binary: Set(None),
        estimate_point_id: Set(None),
        type_id: Set(None),
        deleted_at: Set(None),
        completed_at: Set(None),
        point: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let issue = new_issue.insert(&state.db).await.map_err(AppError::Database)?;

    // Create the intake issue bridge record
    let new_intake_issue = intake_issues::ActiveModel {
        id: Set(Uuid::new_v4()),
        issue_id: Set(issue.id),
        intake_id: Set(intake.id),
        status: Set(-2), // pending by default (matches Django default)
        project_id: Set(project_id),
        workspace_id: Set(board.workspace_id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now),
        updated_at: Set(now),
        deleted_at: Set(None),
        snoozed_till: Set(None),
        duplicate_to: Set(None),
        source: Set(Some("IN_APP".to_string())),
        source_email: Set(None),
        external_source: Set(None),
        external_id: Set(None),
    };

    let intake_issue = new_intake_issue
        .insert(&state.db)
        .await
        .map_err(AppError::Database)?;

    ensure_public_member(&state.db, project_id, board.workspace_id, user.id).await?;

    Ok((
        StatusCode::CREATED,
        Json(PublicIntakeIssueResponse {
            id: intake_issue.id,
            issue_id: intake_issue.issue_id,
            intake_id: intake_issue.intake_id,
            status: intake_issue.status,
            project_id: intake_issue.project_id,
            workspace_id: intake_issue.workspace_id,
            created_at: intake_issue.created_at,
            updated_at: intake_issue.updated_at,
        }),
    ))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Strips HTML tags from a string (same logic as issue_extras.rs).
fn strip_html_text(html: &str) -> String {
    let mut result = String::new();
    let mut inside = false;
    for ch in html.chars() {
        if ch == '<' {
            inside = true;
            continue;
        }
        if ch == '>' {
            inside = false;
            continue;
        }
        if !inside {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

/// Ensures a non-member user is tracked as a `ProjectPublicMember`.
/// Mirrors Django's `ProjectPublicMember.objects.get_or_create(...)` pattern
/// used in comment/reaction/vote write endpoints.
async fn ensure_public_member(
    db: &sea_orm::DatabaseConnection,
    project_id: Uuid,
    workspace_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    // Check if already a project member
    let is_member = project_members::Entity::find()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::MemberId.eq(user_id))
        .filter(project_members::Column::IsActive.eq(true))
        .filter(project_members::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if is_member {
        return Ok(());
    }

    // Check if already tracked
    let exists = project_public_members::Entity::find()
        .filter(project_public_members::Column::ProjectId.eq(project_id))
        .filter(project_public_members::Column::MemberId.eq(user_id))
        .filter(project_public_members::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .is_some();

    if !exists {
        let now: DateTime<FixedOffset> = Utc::now().into();
        let new_pm = project_public_members::ActiveModel {
            id: Set(Uuid::new_v4()),
            project_id: Set(project_id),
            member_id: Set(user_id),
            workspace_id: Set(workspace_id),
            created_by_id: Set(Some(user_id)),
            updated_by_id: Set(Some(user_id)),
            created_at: Set(now),
            updated_at: Set(now),
            deleted_at: Set(None),
        };
        new_pm.insert(db).await.map_err(AppError::Database)?;
    }

    Ok(())
}

// Generic M2M loader for issue_assignees and issue_labels
async fn load_m2m_ids<E, C1, C2>(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
    _entity: E,
    issue_col: C1,
    target_col: C2,
) -> Result<std::collections::HashMap<Uuid, Vec<Uuid>>, AppError>
where
    E: EntityTrait,
    C1: ColumnTrait,
    C2: ColumnTrait,
{
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = E::find()
        .filter(issue_col.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // We can't generically extract columns, so this function signature is a placeholder.
    // Actual implementations are specialized below.
    let _ = rows;
    let _ = target_col;
    Ok(std::collections::HashMap::new())
}

/// Batch-loads assignee_ids for a set of issues.
async fn load_assignees(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<Uuid>>, AppError> {
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut map: std::collections::HashMap<Uuid, Vec<Uuid>> = std::collections::HashMap::new();
    for r in rows {
        map.entry(r.issue_id).or_default().push(r.assignee_id);
    }
    Ok(map)
}

/// Batch-loads label_ids for a set of issues.
async fn load_labels(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<Uuid>>, AppError> {
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = issue_labels::Entity::find()
        .active()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut map: std::collections::HashMap<Uuid, Vec<Uuid>> = std::collections::HashMap::new();
    for r in rows {
        map.entry(r.issue_id).or_default().push(r.label_id);
    }
    Ok(map)
}

/// Batch-loads module_ids for a set of issues.
async fn load_m2m_ids_module(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<Uuid>>, AppError> {
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut map: std::collections::HashMap<Uuid, Vec<Uuid>> = std::collections::HashMap::new();
    for r in rows {
        map.entry(r.issue_id).or_default().push(r.module_id);
    }
    Ok(map)
}

/// Batch-loads the first active cycle_id for each issue.
async fn load_cycle_ids(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Uuid>, AppError> {
    if issue_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }
    let rows = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let mut map: std::collections::HashMap<Uuid, Uuid> = std::collections::HashMap::new();
    for r in rows {
        map.entry(r.issue_id).or_insert(r.cycle_id);
    }
    Ok(map)
}
