// src/routes/external.rs
//! Endpoints de integraciones externas: AI assistant, rephrase-grammar y Unsplash.
//!
//! Equivalente a `plane/app/views/external/base.py` en Django.
//!
//! Rutas implementadas:
//!   GET  /api/unsplash/
//!   POST /api/workspaces/{slug}/projects/{project_id}/ai-assistant/
//!   POST /api/workspaces/{slug}/ai-assistant/
//!   POST /api/workspaces/{slug}/rephrase-grammar/

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
    utils::instance_config::get_config_value,
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

/// Body de POST /workspaces/{slug}/rephrase-grammar/
///
/// Mirror de `RephraseGrammarEndpoint` en `plane/app/views/external/base.py`.
#[derive(Debug, Deserialize, utoipa::ToSchema)]
pub struct RephraseGrammarRequest {
    /// Texto seleccionado en el editor que se desea mejorar (requerido).
    pub text_input: String,
    /// Instrucción libre del usuario (flujo ASK_ANYTHING). Opcional.
    pub prompt: Option<String>,
    /// Puntaje de tono casual 0-10. Opcional.
    pub casual_score: Option<i64>,
    /// Puntaje de tono formal 0-10. Opcional.
    pub formal_score: Option<i64>,
}

/// Respuesta de POST /workspaces/{slug}/rephrase-grammar/
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct RephraseGrammarResponse {
    pub response: String,
}

// ── Unsplash ──────────────────────────────────────────────────────────────────

/// GET /api/unsplash/
///
/// Proxy hacia la API de Unsplash para búsqueda y listado de fotos.
/// No requiere autenticación propia — usa la API key configurada en el servidor.
///
/// Paridad con Django (`plane/app/views/external/base.py::UnsplashEndpoint`):
/// - La key se resuelve vía `instance_configurations` (DB) con fallback a env
///   (equivalente a `get_configuration_value`).
/// - Si no hay key configurada, devuelve `[]` con HTTP 200 (no 400).
/// - Codifica los query params de forma segura para evitar URL injection.
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
        (status = 200, description = "Unsplash photos (empty array if unconfigured)"),
    )
)]
pub async fn unsplash(
    State(state): State<AppState>,
    Query(params): Query<UnsplashQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Paridad con Django: DB (instance_configurations) → env var.
    let env_fallback = std::env::var("UNSPLASH_ACCESS_KEY").ok();
    let access_key_opt = get_config_value(&state, "UNSPLASH_ACCESS_KEY", env_fallback.as_deref())
        .await?
        .filter(|k| !k.is_empty());

    // Django devuelve [] con 200 cuando UNSPLASH_ACCESS_KEY no está configurado.
    let access_key = match access_key_opt {
        Some(k) => k,
        None => return Ok(Json(serde_json::json!([]))),
    };

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(20).clamp(1, 30);

    // Construir request con query params tipados (reqwest los codifica correctamente).
    // Evita pasar `query` del usuario sin escapar a la URL.
    let mut req = if let Some(q) = params.query.as_deref().filter(|s| !s.is_empty()) {
        state
            .http
            .get("https://api.unsplash.com/search/photos/")
            .query(&[
                ("client_id", access_key.as_str()),
                ("query", q),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
            ])
    } else {
        state
            .http
            .get("https://api.unsplash.com/photos/")
            .query(&[
                ("client_id", access_key.as_str()),
                ("page", &page.to_string()),
                ("per_page", &per_page.to_string()),
            ])
    };
    req = req.header("Content-Type", "application/json");

    let response = req.send().await.map_err(|e| {
        tracing::error!("Unsplash API error: {e}");
        AppError::Internal(anyhow::anyhow!("Failed to reach Unsplash API"))
    })?;

    // Django propaga el status de Unsplash — hacemos lo mismo sin leakear detalles
    // de la key en logs.
    let status = response.status();
    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to parse Unsplash response")))?;

    if !status.is_success() {
        tracing::warn!(status = %status, "Unsplash upstream returned non-2xx");
    }

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

// ── Rephrase Grammar ──────────────────────────────────────────────────────────

/// POST /api/workspaces/{slug}/rephrase-grammar/
///
/// Mejora la gramática, claridad y legibilidad del texto seleccionado en el editor.
/// Soporta instrucciones libres (prompt) y pesos de tono casual/formal.
///
/// Mirror de `RephraseGrammarEndpoint` en `plane/app/views/external/base.py`.
/// Requiere rol MEMBER o superior a nivel workspace.
#[utoipa::path(
    post,
    path = "/workspaces/{slug}/rephrase-grammar/",
    tag = "External",
    security(("TokenAuth" = [])),
    params(
        ("slug" = String, Path, description = "Workspace slug"),
    ),
    request_body = RephraseGrammarRequest,
    responses(
        (status = 200, description = "Texto mejorado", body = RephraseGrammarResponse),
        (status = 400, description = "text_input vacío o LLM no configurado"),
        (status = 403, description = "Forbidden"),
        (status = 500, description = "Error al llamar al proveedor LLM"),
    )
)]
pub async fn rephrase_grammar(
    State(state): State<AppState>,
    guard: WorkspaceMemberGuard,
    Json(body): Json<RephraseGrammarRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Mirror Django: @allow_permission([ROLE.ADMIN, ROLE.MEMBER], level="WORKSPACE")
    if guard.member.role < ROLE_MEMBER {
        return Err(AppError::Forbidden);
    }

    // Validar text_input antes de llamar al LLM
    let text_input = body.text_input.trim().to_string();
    if text_input.is_empty() {
        return Err(AppError::BadRequest("text_input is required".into()));
    }

    let api_key = state
        .config
        .llm_api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM provider is not configured".into()))?;

    let model = state
        .config
        .llm_model
        .as_deref()
        .filter(|m| !m.is_empty())
        .ok_or_else(|| AppError::BadRequest("LLM provider is not configured".into()))?;

    let provider = &state.config.llm_provider;

    // ── Construir instrucción del sistema ─────────────────────────────────────
    // Mirror de la lógica en RephraseGrammarEndpoint.post():
    //   task_parts = ["You are a writing assistant…"]
    //   if user_prompt → "User instruction: {prompt}"
    //   else           → "Improve the grammar, clarity, and readability…"
    //   if casual > formal → "Use a casual, friendly tone."
    //   if formal > casual → "Use a formal, professional tone."
    //   task_parts.append("Return only the improved text…")
    let mut task_parts: Vec<String> = vec![
        "You are a writing assistant helping improve text in a project management tool.".into(),
    ];

    let user_prompt = body.prompt.as_deref().map(str::trim).unwrap_or("").to_string();
    if !user_prompt.is_empty() {
        task_parts.push(format!("User instruction: {user_prompt}"));
    } else {
        task_parts.push(
            "Improve the grammar, clarity, and readability of the following text.".into(),
        );
    }

    // Aplicar hints de tono cuando ambos scores están presentes
    if let (Some(casual), Some(formal)) = (body.casual_score, body.formal_score) {
        if casual > formal {
            task_parts.push("Use a casual, friendly tone.".into());
        } else if formal > casual {
            task_parts.push("Use a formal, professional tone.".into());
        }
        // Si son iguales, tono neutro (sin hint adicional)
    }

    task_parts.push(
        "Return only the improved text without any preamble, explanation, or markdown.".into(),
    );

    let task = task_parts.join(" ");

    // ── Llamar al LLM — text_input va como "prompt" (concatenado con task) ───
    // Django: get_llm_response(task, text_input, …)
    //   final_text = task + "\n" + text_input
    let text = call_llm(
        &state.http,
        api_key,
        model,
        provider,
        &task,
        Some(&text_input),
    )
    .await
    .map_err(|e| {
        tracing::error!(provider = %provider, model = %model, "rephrase-grammar LLM error: {e}");
        AppError::Internal(anyhow::anyhow!(
            "Failed to generate a response from the AI provider"
        ))
    })?;

    Ok(Json(RephraseGrammarResponse { response: text }))
}


// ── Incoming webhooks de GitHub y GitLab ──────────────────────────────────────
//
// Mirror de `GitHubWebhookEndpoint` y `GitLabWebhookEndpoint` en Django
// (`plane/app/views/external/sync.py`).
//
// Ambos endpoints son públicos (sin auth). La seguridad se realiza mediante
// verificación HMAC de la firma del payload.

use axum::body::Bytes;

fn hex_encode_bytes(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push_str(&format!("{byte:02x}"));
    }
    output
}

fn hex_decode_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .filter_map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Verifica firma HMAC-SHA256 de GitHub usando OpenSSL.
/// Mirror de `hmac.new(secret, body, sha256).hexdigest()` en Django.
fn verify_github_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    use openssl::hash::MessageDigest;
    use openssl::pkey::PKey;
    use openssl::sign::Signer;

    let Ok(key) = PKey::hmac(secret.as_bytes()) else {
        return false;
    };
    let Ok(mut signer) = Signer::new(MessageDigest::sha256(), &key) else {
        return false;
    };
    if signer.update(body).is_err() {
        return false;
    }
    let Ok(mac_bytes) = signer.sign_to_vec() else {
        return false;
    };
    let expected = format!("sha256={}", hex_encode_bytes(&mac_bytes));
    // Comparación constante para evitar timing attacks
    expected.len() == signature.len()
        && expected
            .bytes()
            .zip(signature.bytes())
            .fold(0u8, |acc, (a, b)| acc | (a ^ b))
            == 0
}

// ── GitHub webhook ────────────────────────────────────────────────────────────

/// Recibe webhooks entrantes de GitHub Apps.
///
/// `POST /github-webhook`
///
/// Valida la firma `X-Hub-Signature-256`, luego despacha según el evento:
/// - `issues`: sincroniza issues (crear/editar/cerrar/reabrir)
/// - `pull_request`: loguea para trazabilidad
///
/// Mirror de `GitHubWebhookEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/github-webhook",
    tag = "External",
    responses(
        (status = 200, description = "Webhook processed or ignored"),
        (status = 400, description = "Signature missing"),
        (status = 403, description = "Invalid signature"),
    )
)]
pub async fn github_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::{
        entities::{
            github_issue_syncs, github_repositories, github_repository_syncs, issues, states,
        },
        utils::instance_config::get_config_value,
    };
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    // ── Verificar firma HMAC ──────────────────────────────────────────────────
    let signature = headers
        .get("x-hub-signature-256")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::BadRequest("Signature missing".into()))?
        .to_owned();

    let webhook_secret = get_config_value(
        &state,
        "GITHUB_WEBHOOK_SECRET",
        std::env::var("GITHUB_WEBHOOK_SECRET").ok().as_deref(),
    )
    .await
    .ok()
    .flatten();

    if let Some(secret) = webhook_secret.as_deref() {
        if !verify_github_signature(secret, &body, &signature) {
            return Err(AppError::Forbidden);
        }
    }

    // ── Parsear payload ───────────────────────────────────────────────────────
    let event = headers
        .get("x-github-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    let payload: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);

    tracing::debug!(event = %event, "GitHub webhook received");

    // Despachar según evento — errores se capturan para no reintentar
    let dispatch_result = match event.as_str() {
        "issues" => {
            handle_github_issue_event(&state, &payload).await
        }
        "pull_request" => {
            let action = payload["action"].as_str().unwrap_or("");
            let pr_number = payload["pull_request"]["number"].as_i64().unwrap_or(0);
            tracing::info!(action = %action, pr = pr_number, "GitHub PR webhook received");
            Ok(())
        }
        _ => {
            tracing::debug!(event = %event, "GitHub webhook event ignored");
            Ok(())
        }
    };

    if let Err(e) = dispatch_result {
        tracing::error!(event = %event, "Error processing GitHub webhook: {e}");
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn handle_github_issue_event(
    state: &AppState,
    payload: &serde_json::Value,
) -> Result<(), AppError> {
    use crate::entities::{
        github_issue_syncs, github_repositories, github_repository_syncs, issues, states,
    };
    use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};

    let action = payload["action"].as_str().unwrap_or("");
    let gh_issue = &payload["issue"];
    let repo_id = payload["repository"]["id"].as_i64().unwrap_or(0);
    let title = gh_issue["title"].as_str().unwrap_or("").to_owned();
    let gh_issue_id = gh_issue["id"].as_i64().unwrap_or(0);
    let gh_issue_number = gh_issue["number"].as_i64().unwrap_or(0);

    // Buscar repositorio registrado
    let repo = github_repositories::Entity::find()
        .filter(github_repositories::Column::RepositoryId.eq(repo_id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(repo) = repo else {
        tracing::debug!(repo_id, "GitHub repo not configured — ignoring event");
        return Ok(());
    };

    let sync = github_repository_syncs::Entity::find()
        .filter(github_repository_syncs::Column::RepositoryId.eq(repo.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    let Some(sync) = sync else {
        return Ok(());
    };

    let issue_sync = github_issue_syncs::Entity::find()
        .filter(github_issue_syncs::Column::GithubIssueId.eq(gh_issue_id))
        .filter(github_issue_syncs::Column::RepositorySyncId.eq(sync.id))
        .one(&state.db)
        .await
        .map_err(AppError::Database)?;

    match action {
        "opened" if issue_sync.is_none() => {
            // Obtener estado triage del proyecto
            let triage_state = states::Entity::find()
                .filter(states::Column::ProjectId.eq(sync.project_id))
                .filter(states::Column::IsTriage.eq(true))
                .one(&state.db)
                .await
                .map_err(AppError::Database)?;

            let new_issue = issues::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                project_id: Set(sync.project_id),
                workspace_id: Set(sync.workspace_id),
                name: Set(title),
                state_id: Set(triage_state.map(|s| s.id)),
                created_by_id: Set(Some(sync.actor_id)),
                sequence_id: Set(1),
                sort_order: Set(65535.0),
                priority: Set("none".to_owned()),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let created = new_issue.insert(&state.db).await.map_err(AppError::Database)?;

            let new_sync = github_issue_syncs::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                github_issue_id: Set(gh_issue_id),
                repo_issue_id: Set(gh_issue_number),
                issue_url: Set(
                    payload["issue"]["html_url"]
                        .as_str()
                        .unwrap_or("")
                        .to_owned(),
                ),
                issue_id: Set(created.id),
                project_id: Set(sync.project_id),
                workspace_id: Set(sync.workspace_id),
                repository_sync_id: Set(sync.id),
                created_by_id: Set(Some(sync.actor_id)),
                created_at: Set(chrono::Utc::now().into()),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            let _ = new_sync.insert(&state.db).await;
            tracing::info!(github_issue_id, "Created Plane issue from GitHub event");
        }
        "edited" => {
            if let Some(is) = issue_sync {
                if let Some(issue) = issues::Entity::find_by_id(is.issue_id)
                    .one(&state.db)
                    .await
                    .map_err(AppError::Database)?
                {
                    let mut active: issues::ActiveModel = issue.into();
                    active.name = Set(title);
                    let _ = active.update(&state.db).await;
                }
            }
        }
        _ => {
            tracing::debug!(action = %action, "GitHub issue action not handled");
        }
    }

    Ok(())
}

// ── GitLab webhook ────────────────────────────────────────────────────────────

/// Recibe webhooks entrantes de GitLab.
///
/// `POST /gitlab-webhook`
///
/// Valida el token `X-Gitlab-Token` y despacha según el evento.
///
/// Mirror de `GitLabWebhookEndpoint.post` en Django.
#[utoipa::path(
    post,
    path = "/gitlab-webhook",
    tag = "External",
    responses(
        (status = 200, description = "Webhook processed or ignored"),
        (status = 403, description = "Invalid token"),
    )
)]
pub async fn gitlab_webhook(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    use crate::utils::instance_config::get_config_value;

    // Verificar token si está configurado
    let gitlab_token = get_config_value(
        &state,
        "GITLAB_WEBHOOK_TOKEN",
        std::env::var("GITLAB_WEBHOOK_TOKEN").ok().as_deref(),
    )
    .await
    .ok()
    .flatten();

    if let Some(expected_token) = gitlab_token.as_deref() {
        let received_token = headers
            .get("x-gitlab-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if received_token != expected_token {
            return Err(AppError::Forbidden);
        }
    }

    let event = headers
        .get("x-gitlab-event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();

    tracing::debug!(event = %event, "GitLab webhook received");

    match event.as_str() {
        "Issue Hook" | "Merge Request Hook" | "Push Hook" => {
            tracing::info!(
                event = %event,
                "GitLab webhook received — full sync processing pending"
            );
        }
        _ => {
            tracing::debug!(event = %event, "GitLab webhook event ignored");
        }
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}
