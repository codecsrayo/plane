// src/utils/github_app.rs
//! Utilities for GitHub App authentication (JWT RS256 + installation access token).

use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, utils::instance_config::get_instance_config, AppState};

#[derive(Debug, Serialize, Deserialize)]
struct AppClaims {
    /// Issued at — 60 s in the past to compensate for clock skew
    iat: i64,
    /// Expiry — maximum 10 min, we use 9 min for margin
    exp: i64,
    /// Issuer — GitHub App ID (numeric string)
    iss: String,
}

/// Generates a JWT RS256 signed with the GitHub App's private key.
///
/// Returns `Ok(None)` if the app is not configured (GITHUB_APP_ID /
/// GITHUB_APP_PRIVATE_KEY absent) to allow soft degradation without fatal error.
async fn build_app_jwt(state: &AppState) -> Result<Option<String>, AppError> {
    let app_id = get_instance_config(state, "GITHUB_APP_ID").await?;
    let key_b64 = get_instance_config(state, "GITHUB_APP_PRIVATE_KEY").await?;

    let (Some(app_id), Some(key_b64)) = (app_id, key_b64) else {
        return Ok(None); // GitHub App not configured
    };

    let pem = general_purpose::STANDARD
        .decode(key_b64.trim())
        .context("GITHUB_APP_PRIVATE_KEY: invalid base64")
        .map_err(AppError::Internal)?;

    let encoding_key = EncodingKey::from_rsa_pem(&pem)
        .context("GITHUB_APP_PRIVATE_KEY: invalid RSA PEM")
        .map_err(AppError::Internal)?;

    let now = chrono::Utc::now().timestamp();
    let claims = AppClaims {
        iat: now - 60,
        exp: now + 540, // 9 min (max 10 min allowed by GitHub)
        iss: app_id,
    };

    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .context("Error signing JWT RS256")
        .map_err(AppError::Internal)?;

    Ok(Some(token))
}

/// Gets an installation access token for the GitHub App.
///
/// The token is valid for 1 hour. This function generates it on demand —
/// it is not cached because most operations are not high frequency.
///
/// [Fix #18] Receives `http: &reqwest::Client` from AppState — never create
/// `Client::new()` per request (new TCP pool per call, exhausts descriptors).
pub async fn get_installation_access_token(
    state: &AppState,
    installation_id: &str,
) -> Result<Option<String>, AppError> {
    let Some(app_jwt) = build_app_jwt(state).await? else {
        tracing::warn!("GitHub App not configured — GITHUB_APP_ID or GITHUB_APP_PRIVATE_KEY absent");
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
        .context("Error contacting GitHub API")
        .map_err(AppError::Internal)?;

    if resp.status().is_success() {
        let body: serde_json::Value = resp
            .json()
            .await
            .context("GitHub API: response is not valid JSON")
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
