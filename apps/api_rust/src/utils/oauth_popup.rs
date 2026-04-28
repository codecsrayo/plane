// src/utils/oauth_popup.rs
//! Helper to generate OAuth popup closure HTML with postMessage to opener.

use axum::response::Html;

/// Allowed message types for OAuth postMessage.
///
/// [Fix #16] Typed allowlist — prevents XSS via unsanitized `message_type`
/// interpolation (e.g. if an attacker controls the `state=` parameter of
/// a GitHub callback and the type was a free `&str`).
#[derive(Debug, Clone, Copy)]
pub enum OAuthMessageType {
    GithubIntegration,
    GithubAuth,
    GitlabIntegration,
    SlackIntegration,
    GithubUserConnection,
    GoogleAuth,
    GiteaAuth,
}

impl OAuthMessageType {
    /// Returns the exact literal string sent to the frontend.
    /// ⚠️ Values must match what the Next.js frontend expects.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GithubIntegration => "github-integration",
            Self::GithubAuth => "github-auth",
            Self::GitlabIntegration => "gitlab-integration",
            Self::SlackIntegration => "slack-integration",
            Self::GithubUserConnection => "github-user-connection",
            Self::GoogleAuth => "google-auth",
            Self::GiteaAuth => "gitea-auth",
        }
    }
}

/// Generates the OAuth popup closure HTML with postMessage to the opener.
///
/// [Fix #16] `message_type` is a typed enum — never interpolated from external input.
/// [Fix #17] The payload is serialized with `serde_json` to ensure correct
/// escaping of all special characters (\n, ', ", backtick, etc.).
pub fn postmessage_html(
    success: bool,
    message_type: OAuthMessageType,
    error: Option<&str>,
    target_origin: Option<&str>,
) -> Html<String> {
    // Serialize to JSON with full escaping via serde_json — never concatenate strings
    let payload = serde_json::json!({
        "type":    message_type.as_str(),
        "success": success,
        "error":   error,
    });
    let payload_json = payload.to_string();
    let origin_expr = match target_origin {
        Some(o) => serde_json::to_string(o).unwrap(),
        None => "window.location.origin".to_owned(),
    };

    Html(format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Connecting…</title></head>
<body>
<script>
(function(){{
  try{{
    var targetOrigin = {origin_expr};
    window.opener && window.opener.postMessage(
      {payload_json},
      targetOrigin
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
