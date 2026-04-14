// src/utils/oauth_popup.rs
//! Helper para generar HTML de cierre de popup OAuth con postMessage al opener.

use axum::response::Html;

/// Tipos de mensaje permitidos para postMessage OAuth.
///
/// [Fix #16] Allowlist tipada — evita XSS por interpolación de message_type
/// no sanitizado (p.ej. si un atacante controla el parámetro `state=` del
/// callback de GitHub y el tipo era un `&str` libre).
#[derive(Debug, Clone, Copy)]
pub enum OAuthMessageType {
    GithubIntegration,
    GitlabIntegration,
    SlackIntegration,
    GithubUserConnection,
}

impl OAuthMessageType {
    /// Retorna el string literal exacto enviado al frontend.
    /// ⚠️ Los valores deben coincidir con los que espera el frontend Next.js.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GithubIntegration => "github-integration",
            Self::GitlabIntegration => "gitlab-integration",
            Self::SlackIntegration => "slack-integration",
            Self::GithubUserConnection => "github-user-connection",
        }
    }
}

/// Genera el HTML de cierre de popup OAuth con postMessage al opener.
///
/// [Fix #16] `message_type` es un enum tipado — nunca interpolado desde input externo.
/// [Fix #17] El payload se serializa con `serde_json` para garantizar escaping
/// correcto de todos los caracteres especiales (\n, ', ", backtick, etc.).
pub fn postmessage_html(
    success: bool,
    message_type: OAuthMessageType,
    error: Option<&str>,
) -> Html<String> {
    // Serializar a JSON con escaping completo vía serde_json — nunca concatenar strings
    let payload = serde_json::json!({
        "type":    message_type.as_str(),
        "success": success,
        "error":   error,
    });
    let payload_json = payload.to_string();

    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Connecting…</title></head>
<body>
<script>
(function(){{
  try{{
    window.opener && window.opener.postMessage(
      {payload_json},
      window.location.origin
    );
  }}catch(e){{}}
  window.close();
}})();
</script>
<p style="font-family:sans-serif;text-align:center;margin-top:4rem;">
  {msg}
</p>
</body></html>"#,
        msg = if success {
            "Integration connected successfully. You may close this window."
        } else {
            "An error occurred. You may close this window."
        }
    ))
}
