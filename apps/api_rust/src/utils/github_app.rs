// src/utils/github_app.rs
//! Utilidades para autenticación con GitHub App (JWT RS256 + installation access token).

use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, utils::instance_config::get_instance_config, AppState};

#[derive(Debug, Serialize, Deserialize)]
struct AppClaims {
    /// Issued at — 60 s en el pasado para compensar skew de reloj
    iat: i64,
    /// Expiry — máximo 10 min, usamos 9 min para margen
    exp: i64,
    /// Issuer — GitHub App ID (string numérico)
    iss: String,
}

/// Genera un JWT RS256 firmado con la private key de la GitHub App.
///
/// Devuelve `Ok(None)` si la app no está configurada (GITHUB_APP_ID /
/// GITHUB_APP_PRIVATE_KEY ausentes) para permitir degradación suave sin error fatal.
async fn build_app_jwt(state: &AppState) -> Result<Option<String>, AppError> {
    let app_id = get_instance_config(state, "GITHUB_APP_ID").await?;
    let key_b64 = get_instance_config(state, "GITHUB_APP_PRIVATE_KEY").await?;

    let (Some(app_id), Some(key_b64)) = (app_id, key_b64) else {
        return Ok(None); // GitHub App no configurado
    };

    let pem = general_purpose::STANDARD
        .decode(key_b64.trim())
        .context("GITHUB_APP_PRIVATE_KEY: base64 inválido")
        .map_err(AppError::Internal)?;

    let encoding_key = EncodingKey::from_rsa_pem(&pem)
        .context("GITHUB_APP_PRIVATE_KEY: PEM RSA inválido")
        .map_err(AppError::Internal)?;

    let now = chrono::Utc::now().timestamp();
    let claims = AppClaims {
        iat: now - 60,
        exp: now + 540, // 9 min (máx 10 min permitido por GitHub)
        iss: app_id,
    };

    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .context("Error al firmar JWT RS256")
        .map_err(AppError::Internal)?;

    Ok(Some(token))
}

/// Obtiene un installation access token para la GitHub App.
///
/// El token es válido durante 1 hora. Esta función lo genera bajo demanda —
/// no se cachea porque la mayoría de operaciones no son de alta frecuencia.
///
/// [Fix #18] Recibe `http: &reqwest::Client` desde AppState — nunca crear
/// `Client::new()` por request (nuevo pool TCP por llamada, agota descriptores).
pub async fn get_installation_access_token(
    state: &AppState,
    installation_id: &str,
) -> Result<Option<String>, AppError> {
    let Some(app_jwt) = build_app_jwt(state).await? else {
        tracing::warn!("GitHub App no configurado — GITHUB_APP_ID o GITHUB_APP_PRIVATE_KEY ausentes");
        return Ok(None);
    };

    let resp = state
        .http
        .post(format!(
            "https://api.github.com/app/installations/{installation_id}/access_tokens"
        ))
        .header("Authorization", format!("Bearer {app_jwt}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "plane-api-rust/0.1")
        .send()
        .await
        .context("Error al contactar GitHub API")
        .map_err(AppError::Internal)?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp
            .json()
            .await
            .context("GitHub API: respuesta no es JSON válido")
            .map_err(AppError::Internal)?;

        Ok(body["token"].as_str().map(String::from))
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        tracing::warn!(
            installation_id,
            %status,
            body = %text,
            "GitHub App installation token request failed"
        );
        Ok(None)
    }
}
