// src/routes/external.rs
//! Endpoints de integraciones externas: AI assistant y Unsplash.
//!
//! Equivalente a `plane/app/views/external/base.py` en Django.
//!
//! Rutas implementadas:
//!   GET  /api/unsplash/
//!   POST /api/workspaces/{slug}/projects/{project_id}/ai-assistant/
//!   POST /api/workspaces/{slug}/ai-assistant/

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    auth::{
        extractors::{ProjectMemberGuard, WorkspaceMemberGuard},
        permissions::{require_role, ROLE_MEMBER},
    },
    error::AppError,
    AppState,
};

// ── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct UnsplashQuery {
    pub query: Option<String>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct AiAssistantRequest {
    /// Tarea a realizar (requerido).
    pub task: String,
    /// Prompt adicional de contexto.
    pub prompt: Option<String>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AiAssistantResponse {
    pub response: String,
    pub response_html: String,
}

// ── Unsplash ──────────────────────────────────────────────────────────────────

/// GET /api/unsplash/
///
/// Proxy hacia la API de Unsplash para búsqueda y listado de fotos.
/// No requiere autenticación propia — usa la API key configurada en el servidor.
#[utoipa::path(
    get,
    path = "/unsplash/",
    tag = "External",
    params(
        ("query" = Option<String>, Query, description = "Search query"),
        ("page" = Option<u32>, Query, description = "Page number"),
        ("per_page" = Option<u32>, Query, description = "Results per page"),
    ),
    responses(
        (status = 200, description = "Unsplash photos"),
        (status = 400, description = "Unsplash not configured"),
    )
)]
pub async fn unsplash(
    State(state): State<AppState>,
    Query(params): Query<UnsplashQuery>,
) -> Result<impl IntoResponse, AppError> {
    let access_key = state
        .config
        .unsplash_access_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| AppError::BadRequest("Unsplash is not configured".into()))?;

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 30);

    let url = if let Some(ref q) = params.query {
        format!(
            "https://api.unsplash.com/search/photos/?client_id={access_key}&query={q}&page={page}&per_page={per_page}"
        )
    } else {
        format!(
            "https://api.unsplash.com/photos/?client_id={access_key}&page={page}&per_page={per_page}"
        )
    };

    let response = state
        .http
        .get(&url)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Unsplash API error: {e}");
            AppError::Internal(anyhow::anyhow!("Failed to reach Unsplash API"))
        })?;

    if !response.status().is_success() {
        return Err(AppError::Internal(anyhow::anyhow!(
            "Unsplash returned error status"
        )));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse Unsplash response")))?;

    Ok(Json(data))
}

// ── AI Assistant ──────────────────────────────────────────────────────────────

/// Llama al proveedor LLM configurado (OpenAI / Anthropic / Gemini).
async fn call_llm(
    http: &reqwest::Client,
    api_key: &str,
    model: &str,
    provider: &str,
    task: &str,
    prompt: Option<&str>,
) -> Result<String, AppError> {
    let full_prompt = match prompt {
        Some(p) if !p.is_empty() => format!("{task}\n{p}"),
        _ => task.to_string(),
    };

    match provider {
        "openai" => {
            let body = serde_json::json!({
                "model": model,
                "messages": [{"role": "user", "content": full_prompt}],
            });

            let resp = http
                .post("https://api.openai.com/v1/chat/completions")
                .bearer_auth(api_key)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("OpenAI request failed: {e}")))?;

            if !resp.status().is_success() {
                return Err(AppError::Internal(anyhow::anyhow!("OpenAI error response")));
            }

            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse OpenAI response")))?;

            let text = data["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            Ok(text)
        }
        "anthropic" => {
            let body = serde_json::json!({
                "model": model,
                "max_tokens": 1024,
                "messages": [{"role": "user", "content": full_prompt}],
            });

            let resp = http
                .post("https://api.anthropic.com/v1/messages")
                .header("x-api-key", api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&body)
                .send()
                .await
                .map_err(|e| {
                    AppError::Internal(anyhow::anyhow!("Anthropic request failed: {e}"))
                })?;

            if !resp.status().is_success() {
                return Err(AppError::Internal(anyhow::anyhow!(
                    "Anthropic error response"
                )));
            }

            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse Anthropic response")))?;

            let text = data["content"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();

            Ok(text)
        }
        "gemini" => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}"
            );
            let body = serde_json::json!({
                "contents": [{"parts": [{"text": full_prompt}]}]
            });

            let resp = http
                .post(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| AppError::Internal(anyhow::anyhow!("Gemini request failed: {e}")))?;

            if !resp.status().is_success() {
                return Err(AppError::Internal(anyhow::anyhow!("Gemini error response")));
            }

            let data: serde_json::Value = resp
                .json()
                .await
                .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse Gemini response")))?;

            let text = data["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .unwrap_or("")
                .to_string();

            Ok(text)
        }
        other => Err(AppError::Internal(anyhow::anyhow!(
            "Unsupported LLM provider: {other}"
        ))),
    }
}

/// POST /api/workspaces/{slug}/projects/{project_id}/ai-assistant/
///
/// Invoca el AI assistant en el contexto de un proyecto.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/projects/{project_id}/ai-assistant/",
    tag = "External",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "AI response"),
        (status = 400, description = "LLM not configured or task missing"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn project_ai_assistant(
    State(state): State<AppState>,
    guard: ProjectMemberGuard,
    Json(body): Json<AiAssistantRequest>,
) -> Result<impl IntoResponse, AppError> {
    require_role(
        guard.project_member.role,
        guard.workspace_member.role,
        ROLE_MEMBER,
    )?;

    let api_key = state
        .config
        .llm_api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM provider API key is not configured".into()))?;

    let model = state
        .config
        .llm_model
        .as_deref()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM model is not configured".into()))?;

    let provider = &state.config.llm_provider;

    if body.task.trim().is_empty() {
        return Err(AppError::BadRequest("task is required".into()));
    }

    let text = call_llm(
        &state.http,
        api_key,
        model,
        provider,
        &body.task,
        body.prompt.as_deref(),
    )
    .await?;

    let response_html = text.replace('\n', "<br/>");

    Ok(Json(AiAssistantResponse {
        response: text,
        response_html,
    }))
}

/// POST /api/workspaces/{slug}/ai-assistant/
///
/// Invoca el AI assistant en el contexto del workspace (sin proyecto).
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/ai-assistant/",
    tag = "External",
    security(("TokenAuth" = [])),
    responses(
        (status = 200, description = "AI response"),
        (status = 400, description = "LLM not configured or task missing"),
        (status = 403, description = "Forbidden"),
    )
)]
pub async fn workspace_ai_assistant(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<AiAssistantRequest>,
) -> Result<impl IntoResponse, AppError> {
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    let api_key = state
        .config
        .llm_api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM provider API key is not configured".into()))?;

    let model = state
        .config
        .llm_model
        .as_deref()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM model is not configured".into()))?;

    let provider = &state.config.llm_provider;

    if body.task.trim().is_empty() {
        return Err(AppError::BadRequest("task is required".into()));
    }

    let text = call_llm(
        &state.http,
        api_key,
        model,
        provider,
        &body.task,
        body.prompt.as_deref(),
    )
    .await?;

    let response_html = text.replace('\n', "<br/>");

    Ok(Json(AiAssistantResponse {
        response: text,
        response_html,
    }))
}
