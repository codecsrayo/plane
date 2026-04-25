// src/utils/url.rs
//! URL detection utilities — mirrors `plane/utils/url.py`.
//!
//! The primary export [`contains_url`] checks whether a string contains an
//! embedded URL.  Django uses this to reject workspace/user names that sneak
//! in links (phishing, spam).  The regex intentionally matches the same four
//! alternatives as the Python `URL_PATTERN`:
//!
//! 1. `https?://…`
//! 2. `www.…`
//! 3. bare domain (`foo.example.com`)
//! 4. IPv4 literal (`192.168.1.1`)
//!
//! ReDoS protection mirrors the Python version: input length cap + per-line
//! truncation.

use std::sync::OnceLock;
use regex::Regex;

/// Returns a reference to the compiled URL detection regex.
///
/// Equivalent to Django's `URL_PATTERN`.  Compiled once on first call.
fn url_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    PATTERN.get_or_init(|| {
        Regex::new(concat!(
            r"(?i)(?:",
            // 1) http:// or https:// followed by non-whitespace
            r"https?://\S+",
            r"|",
            // 2) www. followed by valid domain segments
            r"www\.[a-zA-Z0-9](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*",
            r"|",
            // 3) bare domain: one or more labels ending with a 2-6 char TLD
            r"(?:[a-zA-Z0-9](?:[a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,6}",
            r"|",
            // 4) IPv4 literal
            r"(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)",
            r")",
        ))
        .expect("URL_PATTERN regex must compile")
    })
}

/// Returns `true` if `value` contains an embedded URL.
///
/// Mirrors `plane.utils.url.contains_url` from the Django codebase, including
/// the same ReDoS protections (input length cap, per-line truncation).
pub fn contains_url(value: &str) -> bool {
    // Prevent ReDoS by limiting input length (same as Python: 1000 chars)
    if value.len() > 1000 {
        return false;
    }

    for line in value.lines() {
        // Truncate very long lines (same as Python: 500 chars per line)
        let check = if line.len() > 500 { &line[..500] } else { line };
        if url_pattern().is_match(check) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_url() {
        assert!(contains_url("check http://example.com please"));
        assert!(contains_url("https://foo.bar"));
    }

    #[test]
    fn detects_www_url() {
        assert!(contains_url("visit www.example.com"));
    }

    #[test]
    fn detects_bare_domain() {
        assert!(contains_url("go to example.com"));
    }

    #[test]
    fn detects_ipv4() {
        assert!(contains_url("server at 192.168.1.1"));
    }

    #[test]
    fn allows_normal_names() {
        assert!(!contains_url("My Workspace"));
        assert!(!contains_url("acme-corp"));
        assert!(!contains_url("test 123"));
        assert!(!contains_url(""));
    }

    #[test]
    fn long_input_returns_false() {
        let long = "a".repeat(1001);
        assert!(!contains_url(&long));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// URL validation for link payloads (issue-links / module-links / cycle-links)
// ─────────────────────────────────────────────────────────────────────────────

/// Replica `*LinkSerializer.to_internal_value` + `validate_url` de Django.
///
/// 1. Si la URL no empieza con `http://` o `https://`, antepone `http://`
///    (paridad apps/api/plane/app/serializers/issue.py:565-571 y
///    apps/api/plane/app/serializers/module.py:170-176).
/// 2. Valida con un regex equivalente al `URLValidator` de Django (host con
///    TLD válido o IP, scheme http/https/ftp, puerto opcional, path opcional).
///    Strings sin TLD como `"not-a-url"` → tras prepend `http://not-a-url` →
///    rechazado por carecer de dominio válido.
///
/// Devuelve `AppError::BadRequest` con mensaje idéntico al de Django si la
/// URL no es válida, lo que el frontend reconoce como error de validación.
pub fn normalize_and_validate_url(raw: &str) -> Result<String, crate::error::AppError> {
    use std::sync::OnceLock;

    let normalized = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_owned()
    } else {
        format!("http://{raw}")
    };

    // Regex pragmático: scheme + host (con TLD ≥ 2 chars o IPv4) + puerto/
    // path opcionales. No replica el `URLValidator` byte-a-byte (Django
    // soporta IPv6, IDN punycode, etc.) pero cubre los casos del frontend
    // y rechaza basura como "not-a-url".
    static URL_RE: OnceLock<Regex> = OnceLock::new();
    let re = URL_RE.get_or_init(|| {
        Regex::new(
            r"(?i)^(https?|ftp)://(?:[^\s/@:]+(?::[^\s/@:]*)?@)?(?:(?:[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}|localhost|(?:\d{1,3}\.){3}\d{1,3})(?::\d{1,5})?(?:[/?#]\S*)?$",
        )
        .expect("URL regex válido")
    });

    if !re.is_match(&normalized) {
        return Err(crate::error::AppError::BadRequest("Invalid URL format.".into()));
    }
    Ok(normalized)
}

#[cfg(test)]
mod url_validation_tests {
    use super::normalize_and_validate_url;

    #[test]
    fn accepts_full_https() {
        assert_eq!(
            normalize_and_validate_url("https://docs.plane.so").unwrap(),
            "https://docs.plane.so"
        );
    }

    #[test]
    fn prepends_http_when_missing_scheme() {
        assert_eq!(
            normalize_and_validate_url("example.com").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn rejects_garbage() {
        assert!(normalize_and_validate_url("not-a-url").is_err());
        assert!(normalize_and_validate_url("").is_err());
        assert!(normalize_and_validate_url("ftp://").is_err());
    }

    #[test]
    fn accepts_localhost_and_ipv4() {
        assert!(normalize_and_validate_url("http://localhost:3000").is_ok());
        assert!(normalize_and_validate_url("http://127.0.0.1:8080/path").is_ok());
    }
}
