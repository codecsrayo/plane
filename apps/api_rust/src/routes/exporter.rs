// src/routes/exporter.rs
//! Endpoints de export de issues — paridad con
//! `apps/api/plane/app/views/exporter/base.py::ExportIssuesEndpoint`.
//!
//! - `GET  /api/workspaces/{slug}/export-issues/` — lista paginada de
//!   `ExporterHistory` filtrado por `type="issue_exports"` (requiere
//!   `per_page` + `cursor`).
//! - `POST /api/workspaces/{slug}/export-issues/` — encola un job de
//!   export y retorna el token para polling.
//! - `GET  /api/workspaces/{slug}/export-issues/{token}/` — consulta el
//!   estado del job.

use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    auth::{extractors::WorkspaceMemberGuard, permissions::ROLE_MEMBER},
    entities::{exporters, project_members, projects, users},
    error::AppError,
    routes::workspaces::{user_to_lite, UserLiteDto},
    utils::{pagination, soft_delete::SoftDeleteExt},
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ExportIssuesRequest {
    /// IDs de proyectos a exportar. Paridad Django: el campo del body es
    /// `project` (no `project_ids`) — ver
    /// apps/api/plane/app/views/exporter/base.py:29. Si está ausente o vacío,
    /// se usan todos los proyectos del workspace donde el user es miembro
    /// activo y el proyecto no está archivado.
    pub project: Option<Vec<Uuid>>,
    /// Formato de exportación: "csv", "xlsx" o "json".
    pub provider: Option<String>,
    /// Aceptado por paridad con Django (apps/api/plane/app/views/exporter/
    /// base.py:28) aunque no se usa en este punto del flujo — lo consume el
    /// worker vía la fila persistida.
    #[serde(default)]
    pub multiple: Option<bool>,
}

/// Response shape de Django (apps/api/plane/app/views/exporter/base.py:57-60):
/// `200 {"message": "Once the export is ready you will be able to download it"}`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExportIssuesEnqueuedResponse {
    pub message: String,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExportIssuesResponse {
    pub token: String,
    pub status: String,
    pub url: Option<String>,
}

// ── GET /export-issues/ ──────────────────────────────────────────────────────

/// Query params de `GET /workspaces/{slug}/export-issues/`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct ListExportIssuesQuery {
    /// Cursor Django: `"{per_page}:{offset}:{is_prev}"` (e.g. `"10:0:0"`).
    pub cursor: Option<String>,
    /// Override del per_page. Django lo toma antes que el valor del cursor.
    pub per_page: Option<u64>,
    /// Campo de orden, formato Django: `"-created_at"` (default) o `"created_at"`.
    /// Sólo se soporta `created_at` por paridad con el uso real del frontend.
    pub order_by: Option<String>,
}

/// Mirror de `ExporterHistorySerializer`
/// (apps/api/plane/app/serializers/exporter.py:11-30).
///
/// Campos emitidos: id, created_at, updated_at, project, provider, status, url,
/// initiated_by, initiated_by_detail (UserLiteSerializer no-admin), token,
/// created_by, updated_by.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ExporterHistoryResponse {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub project: Option<Vec<Uuid>>,
    pub provider: String,
    pub status: String,
    pub url: Option<String>,
    pub initiated_by: Uuid,
    pub initiated_by_detail: Option<UserLiteDto>,
    pub token: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
}

/// GET /api/workspaces/{slug}/export-issues/ — lista paginada de exports.
///
/// Paridad con `ExportIssuesEndpoint.get`
/// (apps/api/plane/app/views/exporter/base.py:67-84):
///  - `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")` → role >= MEMBER.
///  - Filtra por `workspace.slug == slug` y `type == "issue_exports"`.
///  - Requiere `per_page` Y `cursor` en la query — si falta alguno, 400.
///  - `order_by` default = `"-created_at"`.
#[utoipa::path(
    get,
    path = "/api/workspaces/{slug}/export-issues/",
    tag = "Exporter",
    params(
        ("slug" = String, Path, description = "Workspace slug"),
        ("cursor" = Option<String>, Query, description = "Cursor Django: per_page:offset:is_prev"),
        ("per_page" = Option<u64>, Query, description = "Override per_page"),
        ("order_by" = Option<String>, Query, description = "Orden, default '-created_at'"),
    ),
    responses(
        (status = 200, description = "Lista paginada de ExporterHistory"),
        (status = 400, description = "Falta per_page o cursor"),
        (status = 403, description = "Role insuficiente (GUEST no permitido)"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn list_export_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Query(q): Query<ListExportIssuesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Paridad Django: `@allow_permission([ADMIN, MEMBER], level="WORKSPACE")`.
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // Paridad Django (apps/api/plane/app/views/exporter/base.py:73-84): este
    // endpoint requiere **ambos** `per_page` y `cursor` presentes. Sin alguno
    // responde 400 con `{"error": "per_page and cursor are required"}`. No
    // fallback a defaults — replicamos el shape de error.
    if q.per_page.is_none() || q.cursor.as_deref().map(str::trim).is_none_or(str::is_empty) {
        return Err(AppError::BadRequest(
            "per_page and cursor are required".into(),
        ));
    }

    const DEFAULT_PER_PAGE: u64 = 10;
    const MAX_PER_PAGE: u64 = pagination::DEFAULT_MAX_LIMIT;

    let cursor = pagination::parse_cursor_or_default(q.cursor.as_deref(), DEFAULT_PER_PAGE)?;
    let limit = pagination::resolve_per_page(
        Some(cursor.per_page),
        q.per_page,
        DEFAULT_PER_PAGE,
        MAX_PER_PAGE,
    );

    // Paridad Django: `order_by=request.GET.get("order_by", "-created_at")`.
    // Sólo soportamos created_at (asc/desc) porque ninguna otra columna se usa
    // en los callsites del frontend. Cualquier otro valor cae al default para
    // evitar expandir la superficie de SQL injection via dynamic column.
    let order_desc = match q.order_by.as_deref().unwrap_or("-created_at") {
        "created_at" => false,
        _ => true, // "-created_at" o cualquier otro → desc
    };

    let base = exporters::Entity::find()
        .filter(exporters::Column::WorkspaceId.eq(guard.workspace.id))
        .filter(exporters::Column::Type.eq("issue_exports"))
        .filter(exporters::Column::DeletedAt.is_null());

    let total_count = base
        .clone()
        .count(&state.db)
        .await
        .map_err(AppError::Database)?;

    let ordered = if order_desc {
        base.order_by_desc(exporters::Column::CreatedAt)
    } else {
        base.order_by_asc(exporters::Column::CreatedAt)
    };

    let rows = ordered
        .paginate(&state.db, limit)
        .fetch_page(cursor.offset)
        .await
        .map_err(AppError::Database)?;

    // Batch-load de `initiated_by` para evitar N+1 (paridad con
    // `select_related("initiated_by")` en Django, apps/api/plane/app/views/
    // exporter/base.py:69).
    let initiator_ids: Vec<Uuid> = {
        let mut seen = std::collections::HashSet::new();
        rows.iter()
            .map(|r| r.initiated_by_id)
            .filter(|id| seen.insert(*id))
            .collect()
    };

    let users_by_id: HashMap<Uuid, users::Model> = if initiator_ids.is_empty() {
        HashMap::new()
    } else {
        users::Entity::find()
            .filter(users::Column::Id.is_in(initiator_ids))
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|u| (u.id, u))
            .collect()
    };

    let results: Vec<ExporterHistoryResponse> = rows
        .into_iter()
        .map(|row| ExporterHistoryResponse {
            // `initiated_by_detail` usa `UserLiteSerializer` (no-admin), así
            // que `is_admin=false` → sin email ni last_login_medium.
            initiated_by_detail: users_by_id.get(&row.initiated_by_id).map(|u| user_to_lite(u, false)),
            id: row.id,
            created_at: row.created_at.into(),
            updated_at: row.updated_at.into(),
            project: row.project,
            provider: row.provider,
            status: row.status,
            url: row.url,
            initiated_by: row.initiated_by_id,
            token: row.token,
            created_by: row.created_by_id,
            updated_by: row.updated_by_id,
        })
        .collect();

    let body = pagination::build_response(results, total_count, limit, cursor.offset);
    Ok((StatusCode::OK, Json(body)))
}

// ── POST /export-issues/ ──────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/workspaces/{slug}/export-issues/",
    tag = "Exporter",
    params(("slug" = String, Path, description = "Workspace slug")),
    responses(
        (status = 200, description = "Exportación encolada"),
        (status = 400, description = "Provider inválido"),
        (status = 403, description = "Role insuficiente (GUEST no permitido)"),
    ),
    security(("TokenAuth" = []))
)]
pub async fn export_issues(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<ExportIssuesRequest>,
) -> Result<(StatusCode, Json<ExportIssuesEnqueuedResponse>), AppError> {
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

    // Paridad Django (apps/api/plane/app/views/exporter/base.py:29,32-39):
    //   project_ids = request.data.get("project", [])
    //   if not project_ids:
    //       project_ids = Project.objects.filter(
    //           workspace__slug=slug,
    //           project_projectmember__member=request.user,
    //           project_projectmember__is_active=True,
    //           archived_at__isnull=True,
    //       ).values_list("id", flat=True)
    //
    // El listado del frontend (column.tsx) hace `project.length`, y el worker
    // falla con "No hay proyectos en el exporter" si `project` es NULL, así
    // que este fallback es **requerido** — nunca persistir NULL en `project`.
    let project_ids: Vec<Uuid> = match body.project.as_ref() {
        Some(ids) if !ids.is_empty() => ids.clone(),
        _ => projects::Entity::find()
            .active()
            .inner_join(project_members::Entity)
            .filter(projects::Column::WorkspaceId.eq(guard.workspace.id))
            .filter(projects::Column::ArchivedAt.is_null())
            .filter(project_members::Column::MemberId.eq(guard.user.id))
            .filter(project_members::Column::IsActive.eq(true))
            .filter(project_members::Column::DeletedAt.is_null())
            .all(&state.db)
            .await
            .map_err(AppError::Database)?
            .into_iter()
            .map(|p| p.id)
            .collect(),
    };

    // Generar token único para este job
    let token = format!("{}", Uuid::new_v4().as_simple());

    // Paridad Django: `TimeAuditModel` (apps/api/plane/db/mixins.py:19-20)
    // rellena `created_at`/`updated_at` vía `auto_now_add`/`auto_now`. En el
    // entity gen de SeaORM (src/entities/exporters.rs:8-10) ambas columnas son
    // `NOT NULL` sin `ActiveModelBehavior::before_save`, así que el call site
    // debe setearlas explícitamente — omitirlas produce 500:
    //   `null value in column "created_at" of relation "exporters" violates
    //    not-null constraint`.
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

    let _exporter = exporters::ActiveModel {
        id: Set(Uuid::new_v4()),
        created_at: Set(now),
        updated_at: Set(now),
        token: Set(token.clone()),
        provider: Set(provider),
        status: Set("queued".to_owned()),
        reason: Set(String::new()),
        key: Set(String::new()),
        url: Set(None),
        // Nunca NULL: si el user no mandó `project` o mandó `[]`, lo
        // completamos con el fallback arriba. Django persiste exactamente el
        // mismo array resuelto (línea 43 del view).
        project: Set(Some(project_ids)),
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
        deleted_at: Set(None),
    }
    .insert(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Encolar el job de exportación en apalis
    use crate::jobs::export::ExportIssuesJob;
    use apalis::prelude::Storage;
    use apalis_sql::postgres::PostgresStorage;

    // Reusar el PgPool compartido de AppState en vez de abrir una conexión
    // nueva por request (antipatrón: pagar TCP+TLS+auth en cada export y
    // descartar el pool al salir de la función).
    let mut storage: PostgresStorage<ExportIssuesJob> =
        PostgresStorage::new(state.pg_pool.clone());
    storage
        .push(ExportIssuesJob {
            exporter_token: token.clone(),
            // Paridad Django (apps/api/plane/app/views/exporter/base.py:28,54):
            // default = false (export single consolidated file). El worker
            // todavía no respeta este flag — ver TODO en jobs/export.rs.
            multiple: body.multiple.unwrap_or(false),
        })
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Job queue error: {e}")))?;

    // Paridad exacta de shape con Django (apps/api/plane/app/views/exporter/
    // base.py:57-60): 200 OK con `{"message": "..."}`. El frontend
    // (export-modal.tsx:82, export-form.tsx:110) no lee el body de la
    // respuesta, sólo refresca via SWR el listado; devolver token/status/url
    // aquí es divergencia innecesaria.
    Ok((
        StatusCode::OK,
        Json(ExportIssuesEnqueuedResponse {
            message: "Once the export is ready you will be able to download it".to_owned(),
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
