// src/routes/exporter.rs
//! POST /api/workspaces/{slug}/export-issues/ — inicia un job de exportación
//! y retorna el token del ExporterHistory para que el cliente haga polling.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::ROLE_MEMBER},
    entities::exporters,
    error::AppError,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExportIssuesRequest {
    /// IDs de proyectos a exportar (vacío = todos los proyectos del workspace)
    pub project_ids: Option<Vec<Uuid>>,
    /// Formato de exportación: "csv" (único soportado actualmente)
    pub provider: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExportIssuesResponse {
    pub token: String,
    pub status: String,
    pub url: Option<String>,
}

// ── POST /export-issues/ ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/export-issues/",
    tag = "Exporter",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 201, description = "Exportación iniciada — polling por token"),
        (status = 400, description = "Error de validación"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn export_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<ExportIssuesRequest>,
) -> Result<(StatusCode, Json<ExportIssuesResponse>), AppError> {
    // Paridad Django: `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")`
    // (apps/api/plane/app/views/exporter/base.py:22). GUEST (role=5) no puede
    // encolar exports. `WorkspaceMemberGuard` sólo valida membresía, así que
    // hay que forzar el piso de rol acá.
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // Paridad Django: sólo csv/xlsx/json son providers válidos
    // (apps/api/plane/app/views/exporter/base.py:31,62-65). Cualquier otro
    // valor → 400 con el mismo shape de error.
    let provider = body.provider.as_deref().unwrap_or("csv").to_owned();
    if !matches!(provider.as_str(), "csv" | "xlsx" | "json") {
        return Err(AppError::BadRequest(format!(
            "Provider '{provider}' not found."
        )));
    }

    // Generar token único para este job
    let token = format!("{}", Uuid::new_v4().as_simple());

    let exporter = exporters::ActiveModel {
        id: Set(Uuid::new_v4()),
        token: Set(token.clone()),
        provider: Set(provider),
        status: Set("queued".to_owned()),
        reason: Set(String::new()),
        key: Set(String::new()),
        url: Set(None),
        project: Set(body.project_ids.clone()),
        initiated_by_id: Set(guard.user.id),
        created_by_id: Set(Some(guard.user.id)),
        updated_by_id: Set(Some(guard.user.id)),
        workspace_id: Set(guard.workspace.id),
        // Paridad Django: `type="issue_exports"` es el default del modelo
        // (apps/api/plane/db/models/exporter.py:26-33) y es el filtro usado
        // por el GET de listado. Si escribimos `"issues"`, los registros
        // creados por Rust quedan invisibles al listar.
        r#type: Set("issue_exports".to_owned()),
        name: Set(None),
        filters: Set(None),
        rich_filters: Set(None),
        ..Default::default()
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Encolar el job de exportación en apalis
    use crate::jobs::export::ExportIssuesJob;
    use apalis::prelude::Storage;
    use apalis_sql::postgres::PostgresStorage;

    // Crear storage y encolar — best-effort (si falla, el cliente puede reintentar)
    let pg_pool = sqlx::PgPool::connect(&state.config.database_url)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("PgPool error: {e}")))?;

    let mut storage: PostgresStorage<ExportIssuesJob> = PostgresStorage::new(pg_pool);
    storage
        .push(ExportIssuesJob {
            exporter_token: token.clone(),
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Job queue error: {e}")))?;

    Ok((
        StatusCode::CREATED,
        Json(ExportIssuesResponse {
            token,
            status: exporter.status,
            url: exporter.url,
        }),
    ))
}

/// GET /api/workspaces/{slug}/export-issues/{token}/ — polling del estado
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/export-issues/{token}/",
    tag = "Exporter",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("token" = String, Path, description = "Export token"),
    ),
    responses(
        (status = 200, description = "Estado del export"),
        (status = 404, description = "No encontrado"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn get_export_status(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Path((_slug, token)): Path<(String, String)>,
) -> Result<Json<ExportIssuesResponse>, AppError> {
    let exporter = exporters::Entity::find()
        .filter(exporters::Column::Token.eq(&token))
        .filter(exporters::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(exporters::Column::DeletedAt.is_null())
        .one(&state.db)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound)?;

    Ok(Json(ExportIssuesResponse {
        token: exporter.token,
        status: exporter.status,
        url: exporter.url,
    }))
}
