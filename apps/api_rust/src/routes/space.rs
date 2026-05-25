// src/routes/space.rs
//! Public Space / Publish-board endpoints — no authentication required for reads.
//!
//! Equivalent to `plane.space.urls` (Django CE) mounted at `/api/public/`.
//!
//! ## Routes implemented
//!   GET  /api/public/anchor/{anchor}/settings/
//!   GET  /api/public/anchor/{anchor}/meta/
//!   GET  /api/public/workspaces/{slug}/projects/{project_id}/anchor/
//!   GET  /api/public/anchor/{anchor}/states/
//!   GET  /api/public/anchor/{anchor}/labels/
//!   GET  /api/public/anchor/{anchor}/members/
//!   GET  /api/public/anchor/{anchor}/cycles/
//!   GET  /api/public/anchor/{anchor}/modules/
//!   GET  /api/public/anchor/{anchor}/issues/
//!   GET  /api/public/anchor/{anchor}/issues/{issue_id}/
//!   GET/POST   /api/public/anchor/{anchor}/issues/{issue_id}/comments/
//!   GET/PATCH/DELETE  /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}/
//!   GET/POST   /api/public/anchor/{anchor}/issues/{issue_id}/reactions/
//!   DELETE     /api/public/anchor/{anchor}/issues/{issue_id}/reactions/{code}/
//!   GET/POST   /api/public/anchor/{anchor}/comments/{comment_id}/reactions/
//!   DELETE     /api/public/anchor/{anchor}/comments/{comment_id}/reactions/{code}/
//!   GET/POST/DELETE  /api/public/anchor/{anchor}/issues/{issue_id}/votes/

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::DateTime;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Order, PaginatorTrait,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::any_auth::AnyAuth,
    entities::{
        comment_reactions, cycles, deploy_boards, issue_comments, issue_reactions, issue_votes,
        labels, module_issues, cycle_issues, issue_assignees, issue_labels,
        issue_links, file_assets, issues, modules, project_members, projects, states,
        users, workspaces,
    },
    error::AppError,
    routes::issue_pagination::{
        parse_cursor, paginated_response, empty_paginated_response,
        DEFAULT_PER_PAGE, ENTITY_TYPE_ISSUE_ATTACHMENT,
    },
    utils::soft_delete::SoftDeleteExt,
    AppState,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Look up a project deploy board by anchor. Returns 404 if not found.
async fn board_by_anchor(
    db: &sea_orm::DatabaseConnection,
    anchor: &str,
) -> Result<deploy_boards::Model, AppError> {
    deploy_boards::Entity::find()
        .active()
        .filter(deploy_boards::Column::Anchor.eq(anchor))
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)
}

fn user_avatar_url(user: &users::Model) -> Option<String> {
    if let Some(asset_id) = user.avatar_asset_id {
        Some(format!("/api/assets/v2/static/{}/", asset_id))
    } else if !user.avatar.is_empty() {
        Some(user.avatar.clone())
    } else {
        None
    }
}

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct WorkspaceDetailDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectDetailsDto {
    pub id: Uuid,
    pub identifier: String,
    pub name: String,
    pub cover_image: Option<String>,
    pub icon_prop: Option<serde_json::Value>,
    pub emoji: Option<String>,
    pub description: String,
}

/// Response for `/anchor/{anchor}/settings/` and `/workspaces/{slug}/projects/{id}/anchor/`.
/// Mirrors `DeployBoardSerializer` in `plane.app.serializers.project`.
#[derive(Debug, Serialize)]
pub struct DeployBoardSettingsResponse {
    pub id: Uuid,
    pub anchor: String,
    pub entity_identifier: Option<Uuid>,
    pub entity_name: Option<String>,
    pub is_comments_enabled: bool,
    pub is_reactions_enabled: bool,
    pub is_votes_enabled: bool,
    pub is_activity_enabled: bool,
    pub is_disabled: bool,
    pub view_props: serde_json::Value,
    /// Project UUID (Django calls it `project`, not `project_id`).
    #[serde(rename = "project")]
    pub project_id: Option<Uuid>,
    /// Workspace UUID (Django calls it `workspace`, not `workspace_id`).
    #[serde(rename = "workspace")]
    pub workspace_id: Uuid,
    pub intake: Option<serde_json::Value>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    #[serde(rename = "updated_by")]
    pub updated_by_id: Option<Uuid>,
    pub created_at: DateTime<chrono::Utc>,
    pub updated_at: DateTime<chrono::Utc>,
    pub project_details: Option<ProjectDetailsDto>,
    pub workspace_detail: Option<WorkspaceDetailDto>,
}

/// Response for `/anchor/{anchor}/meta/`.
/// Mirrors `ProjectLiteSerializer` in `plane.space.serializer.project`.
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

#[derive(Debug, Serialize)]
pub struct StateItemResponse {
    pub id: Uuid,
    pub name: String,
    pub group: String,
    pub color: String,
    pub sequence: f64,
}

#[derive(Debug, Serialize)]
pub struct LabelItemResponse {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub parent: Option<Uuid>,
}

/// Response mirrors Django `.values("id","member","member__display_name","member__avatar")`.
#[derive(Debug, Serialize)]
pub struct MemberItemResponse {
    pub id: Uuid,
    /// Member user UUID.
    pub member: Option<Uuid>,
    /// Display name of the member user.
    #[serde(rename = "member__display_name")]
    pub member_display_name: String,
    /// Avatar URL of the member user.
    #[serde(rename = "member__avatar")]
    pub member_avatar: String,
}

#[derive(Debug, Serialize)]
pub struct CycleItemResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleItemResponse {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct ActorDetailDto {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub avatar: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
}

impl From<&users::Model> for ActorDetailDto {
    fn from(u: &users::Model) -> Self {
        Self {
            id: u.id,
            first_name: u.first_name.clone(),
            last_name: u.last_name.clone(),
            avatar: u.avatar.clone(),
            display_name: u.display_name.clone(),
            avatar_url: user_avatar_url(u),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct VoteItemDto {
    pub vote: i32,
    pub actor_details: ActorDetailDto,
}

#[derive(Debug, Serialize)]
pub struct ReactionItemDto {
    pub reaction: String,
    pub actor_details: ActorDetailDto,
}

/// Public issue item — mirrors `issue_on_results` + enrichment from
/// `plane.space.utils.grouper` (grouper.py:82-180).
#[derive(Debug, Serialize)]
pub struct PublicIssueItem {
    pub id: Uuid,
    pub name: String,
    pub state_id: Option<Uuid>,
    pub sort_order: f64,
    /// FK to estimate_point (serialized as `estimate_point` not `estimate_point_id`).
    #[serde(rename = "estimate_point")]
    pub estimate_point_id: Option<Uuid>,
    pub priority: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub target_date: Option<chrono::NaiveDate>,
    pub sequence_id: i32,
    pub project_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub cycle_id: Option<Uuid>,
    #[serde(rename = "created_by")]
    pub created_by_id: Option<Uuid>,
    /// State group for the associated state.
    #[serde(rename = "state__group")]
    pub state_group: Option<String>,
    pub assignee_ids: Vec<Uuid>,
    pub label_ids: Vec<Uuid>,
    pub module_ids: Vec<Uuid>,
    pub link_count: i64,
    pub attachment_count: i64,
    pub sub_issues_count: i64,
    pub vote_items: Vec<VoteItemDto>,
    pub reaction_items: Vec<ReactionItemDto>,
}

/// Query params for list_anchor_issues.
#[derive(Debug, Deserialize)]
pub struct ListAnchorIssuesQuery {
    pub cursor: Option<String>,
    pub per_page: Option<u64>,
    pub order_by: Option<String>,
}

// ── Enrichment ────────────────────────────────────────────────────────────────

struct IssueEnrichment {
    assignees: std::collections::HashMap<Uuid, Vec<Uuid>>,
    labels: std::collections::HashMap<Uuid, Vec<Uuid>>,
    modules: std::collections::HashMap<Uuid, Vec<Uuid>>,
    cycles: std::collections::HashMap<Uuid, Uuid>,
    sub_counts: std::collections::HashMap<Uuid, i64>,
    attachments: std::collections::HashMap<Uuid, i64>,
    links: std::collections::HashMap<Uuid, i64>,
    state_groups: std::collections::HashMap<Uuid, String>,
    vote_items: std::collections::HashMap<Uuid, Vec<VoteItemDto>>,
    reaction_items: std::collections::HashMap<Uuid, Vec<ReactionItemDto>>,
}

async fn load_public_issue_enrichment(
    db: &sea_orm::DatabaseConnection,
    issue_ids: &[Uuid],
    state_ids: &[Uuid],
    workspace_id: Uuid,
) -> Result<IssueEnrichment, AppError> {
    if issue_ids.is_empty() {
        return Ok(IssueEnrichment {
            assignees: Default::default(),
            labels: Default::default(),
            modules: Default::default(),
            cycles: Default::default(),
            sub_counts: Default::default(),
            attachments: Default::default(),
            links: Default::default(),
            state_groups: Default::default(),
            vote_items: Default::default(),
            reaction_items: Default::default(),
        });
    }

    // Assignees
    let raw_assignees = issue_assignees::Entity::find()
        .active()
        .filter(issue_assignees::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut assignees: std::collections::HashMap<Uuid, Vec<Uuid>> = Default::default();
    for a in raw_assignees {
        assignees.entry(a.issue_id).or_default().push(a.assignee_id);
    }

    // Labels
    let raw_labels = issue_labels::Entity::find()
        .active()
        .filter(issue_labels::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut labels: std::collections::HashMap<Uuid, Vec<Uuid>> = Default::default();
    for l in raw_labels {
        labels.entry(l.issue_id).or_default().push(l.label_id);
    }

    // Modules
    let raw_modules = module_issues::Entity::find()
        .active()
        .filter(module_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut modules: std::collections::HashMap<Uuid, Vec<Uuid>> = Default::default();
    for m in raw_modules {
        modules.entry(m.issue_id).or_default().push(m.module_id);
    }

    // Cycles
    let raw_cycles = cycle_issues::Entity::find()
        .active()
        .filter(cycle_issues::Column::IssueId.is_in(issue_ids.to_vec()))
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut cycle_map: std::collections::HashMap<Uuid, Uuid> = Default::default();
    for ci in raw_cycles {
        cycle_map.entry(ci.issue_id).or_insert(ci.cycle_id);
    }

    // Sub-issue counts
    let raw_sub: Vec<(Uuid, i64)> = issues::Entity::find()
        .select_only()
        .column(issues::Column::ParentId)
        .column_as(sea_orm::sea_query::Expr::col(issues::Column::Id).count(), "cnt")
        .filter(issues::Column::ParentId.is_in(issue_ids.to_vec()))
        .filter(issues::Column::DeletedAt.is_null())
        .group_by(issues::Column::ParentId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut sub_counts: std::collections::HashMap<Uuid, i64> = Default::default();
    for (parent_id, cnt) in raw_sub {
        sub_counts.insert(parent_id, cnt);
    }

    // Attachment counts
    let raw_attachments: Vec<(Option<Uuid>, i64)> = file_assets::Entity::find()
        .select_only()
        .column(file_assets::Column::IssueId)
        .column_as(sea_orm::sea_query::Expr::col(file_assets::Column::Id).count(), "cnt")
        .filter(file_assets::Column::IssueId.is_in(issue_ids.iter().cloned().map(Some).collect::<Vec<_>>()))
        .filter(file_assets::Column::EntityType.eq(ENTITY_TYPE_ISSUE_ATTACHMENT))
        .filter(file_assets::Column::DeletedAt.is_null())
        .group_by(file_assets::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut attachments: std::collections::HashMap<Uuid, i64> = Default::default();
    for (issue_id, cnt) in raw_attachments {
        if let Some(id) = issue_id {
            attachments.insert(id, cnt);
        }
    }

    // Link counts
    let raw_links: Vec<(Uuid, i64)> = issue_links::Entity::find()
        .select_only()
        .column(issue_links::Column::IssueId)
        .column_as(sea_orm::sea_query::Expr::col(issue_links::Column::Id).count(), "cnt")
        .filter(issue_links::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_links::Column::DeletedAt.is_null())
        .group_by(issue_links::Column::IssueId)
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;
    let mut links: std::collections::HashMap<Uuid, i64> = Default::default();
    for (issue_id, cnt) in raw_links {
        links.insert(issue_id, cnt);
    }

    // State groups
    let state_groups: std::collections::HashMap<Uuid, String> = if state_ids.is_empty() {
        Default::default()
    } else {
        let rows = states::Entity::find()
            .select_only()
            .column(states::Column::Id)
            .column(states::Column::Group)
            .filter(states::Column::Id.is_in(state_ids.to_vec()))
            .into_tuple::<(Uuid, String)>()
            .all(db)
            .await
            .map_err(AppError::Database)?;
        rows.into_iter().collect()
    };

    // Vote items — load all votes for issues in the page, then load actors
    let raw_votes = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_votes::Column::WorkspaceId.eq(workspace_id))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let vote_actor_ids: Vec<Uuid> = raw_votes.iter().map(|v| v.actor_id).collect();
    let vote_actors: std::collections::HashMap<Uuid, users::Model> = if vote_actor_ids.is_empty() {
        Default::default()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(vote_actor_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let mut vote_items: std::collections::HashMap<Uuid, Vec<VoteItemDto>> = Default::default();
    for v in raw_votes {
        if let Some(actor) = vote_actors.get(&v.actor_id) {
            vote_items.entry(v.issue_id).or_default().push(VoteItemDto {
                vote: v.vote,
                actor_details: ActorDetailDto::from(actor),
            });
        }
    }

    // Reaction items
    let raw_reactions = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.is_in(issue_ids.to_vec()))
        .filter(issue_reactions::Column::WorkspaceId.eq(workspace_id))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    let reaction_actor_ids: Vec<Uuid> = raw_reactions.iter().map(|r| r.actor_id).collect();
    let reaction_actors: std::collections::HashMap<Uuid, users::Model> =
        if reaction_actor_ids.is_empty() {
            Default::default()
        } else {
            users::Entity::find()
                .filter(users::Column::Id.is_in(reaction_actor_ids))
                .all(db)
                .await
                .map_err(AppError::Database)?
                .into_iter()
                .map(|u| (u.id, u))
                .collect()
        };

    let mut reaction_items: std::collections::HashMap<Uuid, Vec<ReactionItemDto>> =
        Default::default();
    for r in raw_reactions {
        if let Some(actor) = reaction_actors.get(&r.actor_id) {
            reaction_items
                .entry(r.issue_id)
                .or_default()
                .push(ReactionItemDto {
                    reaction: r.reaction.clone(),
                    actor_details: ActorDetailDto::from(actor),
                });
        }
    }

    Ok(IssueEnrichment {
        assignees,
        labels,
        modules,
        cycles: cycle_map,
        sub_counts,
        attachments,
        links,
        state_groups,
        vote_items,
        reaction_items,
    })
}

fn build_public_issue(issue: &issues::Model, enrichment: &IssueEnrichment) -> PublicIssueItem {
    let empty_uuids = vec![];
    let empty_votes = vec![];
    let empty_reactions = vec![];

    PublicIssueItem {
        id: issue.id,
        name: issue.name.clone(),
        state_id: issue.state_id,
        sort_order: issue.sort_order,
        estimate_point_id: issue.estimate_point_id,
        priority: issue.priority.clone(),
        start_date: issue.start_date,
        target_date: issue.target_date,
        sequence_id: issue.sequence_id,
        project_id: issue.project_id,
        parent_id: issue.parent_id,
        cycle_id: enrichment.cycles.get(&issue.id).copied(),
        created_by_id: issue.created_by_id,
        state_group: issue
            .state_id
            .and_then(|sid| enrichment.state_groups.get(&sid).cloned()),
        assignee_ids: enrichment
            .assignees
            .get(&issue.id)
            .unwrap_or(&empty_uuids)
            .clone(),
        label_ids: enrichment
            .labels
            .get(&issue.id)
            .unwrap_or(&empty_uuids)
            .clone(),
        module_ids: enrichment
            .modules
            .get(&issue.id)
            .unwrap_or(&empty_uuids)
            .clone(),
        link_count: enrichment.links.get(&issue.id).copied().unwrap_or(0),
        attachment_count: enrichment.attachments.get(&issue.id).copied().unwrap_or(0),
        sub_issues_count: enrichment.sub_counts.get(&issue.id).copied().unwrap_or(0),
        vote_items: enrichment
            .vote_items
            .get(&issue.id)
            .unwrap_or(&empty_votes)
            .iter()
            .map(|v| VoteItemDto {
                vote: v.vote,
                actor_details: ActorDetailDto {
                    id: v.actor_details.id,
                    first_name: v.actor_details.first_name.clone(),
                    last_name: v.actor_details.last_name.clone(),
                    avatar: v.actor_details.avatar.clone(),
                    display_name: v.actor_details.display_name.clone(),
                    avatar_url: v.actor_details.avatar_url.clone(),
                },
            })
            .collect(),
        reaction_items: enrichment
            .reaction_items
            .get(&issue.id)
            .unwrap_or(&empty_reactions)
            .iter()
            .map(|r| ReactionItemDto {
                reaction: r.reaction.clone(),
                actor_details: ActorDetailDto {
                    id: r.actor_details.id,
                    first_name: r.actor_details.first_name.clone(),
                    last_name: r.actor_details.last_name.clone(),
                    avatar: r.actor_details.avatar.clone(),
                    display_name: r.actor_details.display_name.clone(),
                    avatar_url: r.actor_details.avatar_url.clone(),
                },
            })
            .collect(),
    }
}

// ── Settings & Meta ───────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/settings/
///
/// Mirrors `ProjectDeployBoardPublicSettingsEndpoint` + `DeployBoardSerializer`.
pub async fn get_anchor_settings(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<DeployBoardSettingsResponse>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    // Load project details
    let project_details = if let Some(pid) = board.project_id {
        projects::Entity::find_by_id(pid)
            .filter(projects::Column::DeletedAt.is_null())
            .one(db)
            .await
            .map_err(AppError::Database)?
            .map(|p| ProjectDetailsDto {
                id: p.id,
                identifier: p.identifier.clone(),
                name: p.name.clone(),
                cover_image: p.cover_image.clone(),
                icon_prop: p.icon_prop.clone(),
                emoji: p.emoji.clone(),
                description: p.description.clone(),
            })
    } else {
        None
    };

    // Load workspace details
    let workspace_detail = workspaces::Entity::find_by_id(board.workspace_id)
        .filter(workspaces::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .map(|w| WorkspaceDetailDto {
            id: w.id,
            name: w.name.clone(),
            slug: w.slug.clone(),
        });

    Ok(Json(DeployBoardSettingsResponse {
        id: board.id,
        anchor: board.anchor,
        entity_identifier: board.entity_identifier,
        entity_name: board.entity_name,
        is_comments_enabled: board.is_comments_enabled,
        is_reactions_enabled: board.is_reactions_enabled,
        is_votes_enabled: board.is_votes_enabled,
        is_activity_enabled: board.is_activity_enabled,
        is_disabled: board.is_disabled,
        view_props: board.view_props,
        project_id: board.project_id,
        workspace_id: board.workspace_id,
        intake: None,
        created_by_id: board.created_by_id,
        updated_by_id: board.updated_by_id,
        created_at: board.created_at.into(),
        updated_at: board.updated_at.into(),
        project_details,
        workspace_detail,
    }))
}

/// GET /api/public/anchor/{anchor}/meta/
///
/// Mirrors `ProjectMetaDataEndpoint`.
pub async fn get_anchor_meta(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<ProjectMetaResponse>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let project = projects::Entity::find_by_id(project_id)
        .filter(projects::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ProjectMetaResponse {
        id: project.id,
        identifier: project.identifier,
        name: project.name,
        cover_image: project.cover_image,
        icon_prop: project.icon_prop,
        emoji: project.emoji,
        description: project.description,
    }))
}

/// GET /api/public/workspaces/{slug}/projects/{project_id}/anchor/
///
/// Mirrors `WorkspaceProjectAnchorEndpoint`.
pub async fn get_workspace_project_anchor(
    State(state): State<AppState>,
    Path((slug, project_id)): Path<(String, Uuid)>,
) -> Result<Json<DeployBoardSettingsResponse>, AppError> {
    let db = &state.db;

    let workspace = workspaces::Entity::find()
        .active()
        .filter(workspaces::Column::Slug.eq(&slug))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let board = deploy_boards::Entity::find()
        .active()
        .filter(deploy_boards::Column::WorkspaceId.eq(workspace.id))
        .filter(deploy_boards::Column::EntityIdentifier.eq(project_id))
        .filter(deploy_boards::Column::EntityName.eq("project"))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    // Load project details
    let project_details = projects::Entity::find_by_id(project_id)
        .filter(projects::Column::DeletedAt.is_null())
        .one(db)
        .await
        .map_err(AppError::Database)?
        .map(|p| ProjectDetailsDto {
            id: p.id,
            identifier: p.identifier.clone(),
            name: p.name.clone(),
            cover_image: p.cover_image.clone(),
            icon_prop: p.icon_prop.clone(),
            emoji: p.emoji.clone(),
            description: p.description.clone(),
        });

    let workspace_detail = Some(WorkspaceDetailDto {
        id: workspace.id,
        name: workspace.name.clone(),
        slug: workspace.slug.clone(),
    });

    Ok(Json(DeployBoardSettingsResponse {
        id: board.id,
        anchor: board.anchor,
        entity_identifier: board.entity_identifier,
        entity_name: board.entity_name,
        is_comments_enabled: board.is_comments_enabled,
        is_reactions_enabled: board.is_reactions_enabled,
        is_votes_enabled: board.is_votes_enabled,
        is_activity_enabled: board.is_activity_enabled,
        is_disabled: board.is_disabled,
        view_props: board.view_props,
        project_id: board.project_id,
        workspace_id: board.workspace_id,
        intake: None,
        created_by_id: board.created_by_id,
        updated_by_id: board.updated_by_id,
        created_at: board.created_at.into(),
        updated_at: board.updated_at.into(),
        project_details,
        workspace_detail,
    }))
}

// ── States / Labels / Members / Cycles / Modules ─────────────────────────────

/// GET /api/public/anchor/{anchor}/states/
pub async fn list_anchor_states(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<StateItemResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let rows = states::Entity::find()
        .active()
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::WorkspaceId.eq(board.workspace_id))
        .filter(states::Column::IsTriage.eq(false))
        .order_by(states::Column::Sequence, Order::Asc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|s| StateItemResponse {
                id: s.id,
                name: s.name,
                group: s.group,
                color: s.color,
                sequence: s.sequence,
            })
            .collect(),
    ))
}

/// GET /api/public/anchor/{anchor}/labels/
pub async fn list_anchor_labels(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<LabelItemResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let rows = labels::Entity::find()
        .active()
        .filter(labels::Column::ProjectId.eq(project_id))
        .filter(labels::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(labels::Column::Name, Order::Asc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|l| LabelItemResponse {
                id: l.id,
                name: l.name,
                color: l.color,
                parent: l.parent_id,
            })
            .collect(),
    ))
}

/// GET /api/public/anchor/{anchor}/members/
pub async fn list_anchor_members(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<MemberItemResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let members = project_members::Entity::find()
        .active()
        .filter(project_members::Column::ProjectId.eq(project_id))
        .filter(project_members::Column::WorkspaceId.eq(board.workspace_id))
        .filter(project_members::Column::IsActive.eq(true))
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Batch load user details
    let member_ids: Vec<Uuid> = members
        .iter()
        .filter_map(|m| m.member_id)
        .collect();

    let user_map: std::collections::HashMap<Uuid, users::Model> = if member_ids.is_empty() {
        Default::default()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(member_ids))
            .all(db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    Ok(Json(
        members
            .into_iter()
            .map(|m| {
                let (display_name, avatar) = m
                    .member_id
                    .and_then(|id| user_map.get(&id))
                    .map(|u| (u.display_name.clone(), u.avatar.clone()))
                    .unwrap_or_default();
                MemberItemResponse {
                    id: m.id,
                    member: m.member_id,
                    member_display_name: display_name,
                    member_avatar: avatar,
                }
            })
            .collect(),
    ))
}

/// GET /api/public/anchor/{anchor}/cycles/
pub async fn list_anchor_cycles(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<CycleItemResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let rows = cycles::Entity::find()
        .active()
        .filter(cycles::Column::ProjectId.eq(project_id))
        .filter(cycles::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(cycles::Column::Name, Order::Asc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|c| CycleItemResponse { id: c.id, name: c.name })
            .collect(),
    ))
}

/// GET /api/public/anchor/{anchor}/modules/
pub async fn list_anchor_modules(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
) -> Result<Json<Vec<ModuleItemResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let rows = modules::Entity::find()
        .active()
        .filter(modules::Column::ProjectId.eq(project_id))
        .filter(modules::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(modules::Column::Name, Order::Asc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
            .map(|m| ModuleItemResponse { id: m.id, name: m.name })
            .collect(),
    ))
}

// ── Issues ────────────────────────────────────────────────────────────────────

/// GET /api/public/anchor/{anchor}/issues/
///
/// Paginated public issue list. Mirrors `ProjectIssuesPublicEndpoint`.
/// Group-by is not supported; returns flat list.
pub async fn list_anchor_issues(
    State(state): State<AppState>,
    Path(anchor): Path<String>,
    Query(params): Query<ListAnchorIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let order_by = params.order_by.as_deref().unwrap_or("-created_at");
    let (page_size, current_page) = parse_cursor(params.cursor.as_deref(), DEFAULT_PER_PAGE);

    // Exclude triage states
    let triage_state_ids: Vec<Uuid> = states::Entity::find()
        .select_only()
        .column(states::Column::Id)
        .filter(states::Column::ProjectId.eq(project_id))
        .filter(states::Column::Group.eq("triage"))
        .filter(states::Column::DeletedAt.is_null())
        .into_tuple()
        .all(db)
        .await
        .map_err(AppError::Database)?;

    // Build base query
    use sea_orm::Order::{Asc, Desc};

    let base_query = issues::Entity::find()
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issues::Column::ArchivedAt.is_null())
        .filter(issues::Column::IsDraft.eq(false));

    let base_query = if !triage_state_ids.is_empty() {
        base_query.filter(issues::Column::StateId.is_not_in(triage_state_ids))
    } else {
        base_query
    };

    // Apply ordering
    let (order_col, order_dir): (issues::Column, _) = match order_by {
        "-created_at"  => (issues::Column::CreatedAt,  Desc),
        "created_at"   => (issues::Column::CreatedAt,  Asc),
        "-updated_at"  => (issues::Column::UpdatedAt,  Desc),
        "updated_at"   => (issues::Column::UpdatedAt,  Asc),
        "-priority"    => (issues::Column::Priority,   Desc),
        "priority"     => (issues::Column::Priority,   Asc),
        "-sort_order"  => (issues::Column::SortOrder,  Desc),
        "sort_order"   => (issues::Column::SortOrder,  Asc),
        "-sequence_id" => (issues::Column::SequenceId, Desc),
        "sequence_id"  => (issues::Column::SequenceId, Asc),
        "-target_date" => (issues::Column::TargetDate, Desc),
        "target_date"  => (issues::Column::TargetDate, Asc),
        "-start_date"  => (issues::Column::StartDate,  Desc),
        "start_date"   => (issues::Column::StartDate,  Asc),
        _              => (issues::Column::CreatedAt,  Desc),
    };

    let total_results = base_query
        .clone()
        .count(db)
        .await
        .map_err(AppError::Database)?;

    if total_results == 0 {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let page_issues = base_query
        .order_by(order_col, order_dir)
        .offset(current_page * page_size)
        .limit(page_size)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    if page_issues.is_empty() {
        return Ok(Json(empty_paginated_response(page_size)));
    }

    let issue_ids: Vec<Uuid> = page_issues.iter().map(|i| i.id).collect();
    let state_ids: Vec<Uuid> = page_issues.iter().filter_map(|i| i.state_id).collect();

    let enrichment = load_public_issue_enrichment(
        db,
        &issue_ids,
        &state_ids,
        board.workspace_id,
    )
    .await?;

    let results: Vec<PublicIssueItem> = page_issues
        .iter()
        .map(|i| build_public_issue(i, &enrichment))
        .collect();

    Ok(Json(paginated_response(results, page_size, current_page, total_results)))
}

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/
///
/// Mirrors `IssueRetrievePublicEndpoint`.
pub async fn get_anchor_issue(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<PublicIssueItem>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let issue = issues::Entity::find_by_id(issue_id)
        .active()
        .filter(issues::Column::ProjectId.eq(project_id))
        .filter(issues::Column::WorkspaceId.eq(board.workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let issue_ids = [issue.id];
    let state_ids: Vec<Uuid> = issue.state_id.into_iter().collect();

    let enrichment = load_public_issue_enrichment(
        db,
        &issue_ids,
        &state_ids,
        board.workspace_id,
    )
    .await?;

    Ok(Json(build_public_issue(&issue, &enrichment)))
}

// ── Comments ──────────────────────────────────────────────────────────────────

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
    pub created_at: DateTime<chrono::FixedOffset>,
    pub updated_at: DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCommentBody {
    pub comment_html: String,
    pub access: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentBody {
    pub comment_html: Option<String>,
    pub access: Option<String>,
}

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/comments/
pub async fn list_issue_comments(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicCommentResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_comments_enabled {
        return Ok(Json(vec![]));
    }

    let rows = issue_comments::Entity::find()
        .active()
        .filter(issue_comments::Column::IssueId.eq(issue_id))
        .filter(issue_comments::Column::WorkspaceId.eq(board.workspace_id))
        .filter(issue_comments::Column::Access.eq("EXTERNAL"))
        .order_by(issue_comments::Column::CreatedAt, Order::Asc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
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

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/comments/
pub async fn create_issue_comment(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateCommentBody>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_comments_enabled {
        return Err(AppError::BadRequest(
            "Comments are not enabled for this project".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let now = chrono::Utc::now().fixed_offset();

    let am = issue_comments::ActiveModel {
        id: Set(Uuid::new_v4()),
        comment_html: Set(body.comment_html.clone()),
        comment_stripped: Set(
            strip_html(&body.comment_html),
        ),
        access: Set(body.access.unwrap_or_else(|| "EXTERNAL".into())),
        issue_id: Set(issue_id),
        project_id: Set(project_id),
        workspace_id: Set(board.workspace_id),
        actor_id: Set(Some(user.id)),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        comment_json: Set(serde_json::json!({})),
        attachments: Set(vec![]),
        ..Default::default()
    };

    let comment = am.insert(db).await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(PublicCommentResponse {
            id: comment.id,
            comment_html: comment.comment_html,
            comment_stripped: comment.comment_stripped,
            access: comment.access,
            issue_id: comment.issue_id,
            project_id: comment.project_id,
            workspace_id: comment.workspace_id,
            actor_id: comment.actor_id,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
        }),
    ))
}

/// PATCH /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}/
pub async fn update_issue_comment(
    State(state): State<AppState>,
    Path((anchor, _issue_id, pk)): Path<(String, Uuid, Uuid)>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<UpdateCommentBody>,
) -> Result<Json<PublicCommentResponse>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_comments_enabled {
        return Err(AppError::BadRequest(
            "Comments are not enabled for this project".into(),
        ));
    }

    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::ActorId.eq(user.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_comments::ActiveModel = comment.into();
    if let Some(html) = body.comment_html {
        let stripped = strip_html(&html);
        am.comment_html = Set(html);
        am.comment_stripped = Set(stripped);
    }
    if let Some(access) = body.access {
        am.access = Set(access);
    }
    am.updated_at = Set(chrono::Utc::now().fixed_offset().into());

    let updated = am.update(db).await.map_err(AppError::Database)?;

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

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/comments/{pk}/
pub async fn delete_issue_comment(
    State(state): State<AppState>,
    Path((anchor, _issue_id, pk)): Path<(String, Uuid, Uuid)>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_comments_enabled {
        return Err(AppError::BadRequest(
            "Comments are not enabled for this project".into(),
        ));
    }

    let comment = issue_comments::Entity::find_by_id(pk)
        .active()
        .filter(issue_comments::Column::ActorId.eq(user.id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_comments::ActiveModel = comment.into();
    am.deleted_at = Set(Some(chrono::Utc::now().fixed_offset().into()));
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Issue reactions ───────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<chrono::FixedOffset>,
    pub updated_at: DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateReactionBody {
    pub reaction: String,
}

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/reactions/
pub async fn list_issue_reactions(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicReactionResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Ok(Json(vec![]));
    }

    let rows = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(issue_reactions::Column::CreatedAt, Order::Desc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
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

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/reactions/
pub async fn add_issue_reaction(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateReactionBody>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this project board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let now = chrono::Utc::now().fixed_offset();

    let am = issue_reactions::ActiveModel {
        id: Set(Uuid::new_v4()),
        reaction: Set(body.reaction),
        issue_id: Set(issue_id),
        actor_id: Set(user.id),
        project_id: Set(project_id),
        workspace_id: Set(board.workspace_id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let reaction = am.insert(db).await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(PublicReactionResponse {
            id: reaction.id,
            reaction: reaction.reaction,
            issue_id: reaction.issue_id,
            actor_id: reaction.actor_id,
            project_id: reaction.project_id,
            workspace_id: reaction.workspace_id,
            created_at: reaction.created_at,
            updated_at: reaction.updated_at,
        }),
    ))
}

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/reactions/{reaction_code}/
pub async fn remove_issue_reaction(
    State(state): State<AppState>,
    Path((anchor, issue_id, reaction_code)): Path<(String, Uuid, String)>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this project board".into(),
        ));
    }

    let reaction = issue_reactions::Entity::find()
        .active()
        .filter(issue_reactions::Column::IssueId.eq(issue_id))
        .filter(issue_reactions::Column::Reaction.eq(&reaction_code))
        .filter(issue_reactions::Column::ActorId.eq(user.id))
        .filter(issue_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(chrono::Utc::now().fixed_offset().into()));
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Comment reactions ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicCommentReactionResponse {
    pub id: Uuid,
    pub reaction: String,
    pub comment_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<chrono::FixedOffset>,
    pub updated_at: DateTime<chrono::FixedOffset>,
}

/// GET /api/public/anchor/{anchor}/comments/{comment_id}/reactions/
pub async fn list_comment_reactions(
    State(state): State<AppState>,
    Path((anchor, comment_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicCommentReactionResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Ok(Json(vec![]));
    }

    let rows = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(comment_reactions::Column::CreatedAt, Order::Desc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
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

/// POST /api/public/anchor/{anchor}/comments/{comment_id}/reactions/
pub async fn add_comment_reaction(
    State(state): State<AppState>,
    Path((anchor, comment_id)): Path<(String, Uuid)>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateReactionBody>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this board".into(),
        ));
    }

    let project_id = board.project_id.ok_or(AppError::NotFound)?;
    let now = chrono::Utc::now().fixed_offset();

    let am = comment_reactions::ActiveModel {
        id: Set(Uuid::new_v4()),
        reaction: Set(body.reaction),
        comment_id: Set(comment_id),
        actor_id: Set(user.id),
        project_id: Set(project_id),
        workspace_id: Set(board.workspace_id),
        created_by_id: Set(Some(user.id)),
        updated_by_id: Set(Some(user.id)),
        created_at: Set(now.into()),
        updated_at: Set(now.into()),
        ..Default::default()
    };

    let reaction = am.insert(db).await.map_err(AppError::Database)?;

    Ok((
        StatusCode::CREATED,
        Json(PublicCommentReactionResponse {
            id: reaction.id,
            reaction: reaction.reaction,
            comment_id: reaction.comment_id,
            actor_id: reaction.actor_id,
            project_id: reaction.project_id,
            workspace_id: reaction.workspace_id,
            created_at: reaction.created_at,
            updated_at: reaction.updated_at,
        }),
    ))
}

/// DELETE /api/public/anchor/{anchor}/comments/{comment_id}/reactions/{reaction_code}/
pub async fn remove_comment_reaction(
    State(state): State<AppState>,
    Path((anchor, comment_id, reaction_code)): Path<(String, Uuid, String)>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_reactions_enabled {
        return Err(AppError::BadRequest(
            "Reactions are not enabled for this board".into(),
        ));
    }

    let reaction = comment_reactions::Entity::find()
        .active()
        .filter(comment_reactions::Column::CommentId.eq(comment_id))
        .filter(comment_reactions::Column::Reaction.eq(&reaction_code))
        .filter(comment_reactions::Column::ActorId.eq(user.id))
        .filter(comment_reactions::Column::WorkspaceId.eq(board.workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: comment_reactions::ActiveModel = reaction.into();
    am.deleted_at = Set(Some(chrono::Utc::now().fixed_offset().into()));
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Votes ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct PublicVoteResponse {
    pub id: Uuid,
    pub vote: i32,
    pub issue_id: Uuid,
    pub actor_id: Uuid,
    pub project_id: Uuid,
    pub workspace_id: Uuid,
    pub created_at: DateTime<chrono::FixedOffset>,
    pub updated_at: DateTime<chrono::FixedOffset>,
}

#[derive(Debug, Deserialize)]
pub struct CreateVoteBody {
    #[serde(default = "default_vote")]
    pub vote: i32,
}

fn default_vote() -> i32 {
    1
}

/// GET /api/public/anchor/{anchor}/issues/{issue_id}/votes/
pub async fn list_issue_votes(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
) -> Result<Json<Vec<PublicVoteResponse>>, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;

    if !board.is_votes_enabled {
        return Ok(Json(vec![]));
    }

    let rows = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::WorkspaceId.eq(board.workspace_id))
        .order_by(issue_votes::Column::CreatedAt, Order::Desc)
        .all(db)
        .await
        .map_err(AppError::Database)?;

    Ok(Json(
        rows.into_iter()
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

/// POST /api/public/anchor/{anchor}/issues/{issue_id}/votes/
pub async fn add_issue_vote(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    AnyAuth(user): AnyAuth,
    Json(body): Json<CreateVoteBody>,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    // Check if votes are enabled
    if !board.is_votes_enabled {
        return Err(AppError::BadRequest(
            "Votes are not enabled for this project board".into(),
        ));
    }

    let now = chrono::Utc::now().fixed_offset();

    // get_or_create semantics — if vote exists, update it
    let existing = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::ActorId.eq(user.id))
        .filter(issue_votes::Column::ProjectId.eq(project_id))
        .one(db)
        .await
        .map_err(AppError::Database)?;

    let vote = if let Some(existing) = existing {
        let mut am: issue_votes::ActiveModel = existing.into();
        am.vote = Set(body.vote);
        am.updated_at = Set(now.into());
        am.update(db).await.map_err(AppError::Database)?
    } else {
        let am = issue_votes::ActiveModel {
            id: Set(Uuid::new_v4()),
            vote: Set(body.vote),
            issue_id: Set(issue_id),
            actor_id: Set(user.id),
            project_id: Set(project_id),
            workspace_id: Set(board.workspace_id),
            created_by_id: Set(Some(user.id)),
            updated_by_id: Set(Some(user.id)),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
            ..Default::default()
        };
        am.insert(db).await.map_err(AppError::Database)?
    };

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

/// DELETE /api/public/anchor/{anchor}/issues/{issue_id}/votes/
pub async fn remove_issue_vote(
    State(state): State<AppState>,
    Path((anchor, issue_id)): Path<(String, Uuid)>,
    AnyAuth(user): AnyAuth,
) -> Result<impl IntoResponse, AppError> {
    let db = &state.db;
    let board = board_by_anchor(db, &anchor).await?;
    let project_id = board.project_id.ok_or(AppError::NotFound)?;

    let vote = issue_votes::Entity::find()
        .active()
        .filter(issue_votes::Column::IssueId.eq(issue_id))
        .filter(issue_votes::Column::ActorId.eq(user.id))
        .filter(issue_votes::Column::ProjectId.eq(project_id))
        .filter(issue_votes::Column::WorkspaceId.eq(board.workspace_id))
        .one(db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    let mut am: issue_votes::ActiveModel = vote.into();
    am.deleted_at = Set(Some(chrono::Utc::now().fixed_offset().into()));
    am.update(db).await.map_err(AppError::Database)?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Utilities ─────────────────────────────────────────────────────────────────

fn strip_html(html: &str) -> String {
    // Minimal HTML tag stripper — replaces tags with spaces, collapses whitespace.
    let no_tags = regex_replace_all_tags(html);
    no_tags.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn regex_replace_all_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => { in_tag = true; result.push(' '); }
            '>' => { in_tag = false; }
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
}
