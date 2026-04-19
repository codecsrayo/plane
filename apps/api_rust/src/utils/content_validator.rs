// src/utils/content_validator.rs
//! Validación y sanitización de contenido para Pages.
//!
//! Espeja `apps/api/plane/utils/content_validator.py` — incluye:
//! * `validate_binary_data`: tamaño máximo (10 MB), longitud mínima y detección
//!   de patrones HTML/JS sospechosos (marcadores de ataques incrustados en lo
//!   que debería ser un documento Y.js binario).
//! * `validate_html_content`: tamaño máximo y sanitización con `ammonia`,
//!   preservando los tags y atributos personalizados del editor de Plane
//!   (`mention-component`, `image-component`, `input`, etc.) para no perder
//!   datos al guardar HTML generado por el editor colaborativo.
//!
//! Las listas de tags, atributos y protocolos replican exactamente las
//! definidas en Python para garantizar compatibilidad bidireccional cuando el
//! backend Rust corra en paralelo con el Django.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

/// Tamaño máximo permitido para contenido (binario o HTML): 10 MB.
/// Mirror de `MAX_SIZE` en `content_validator.py`.
pub const MAX_CONTENT_SIZE: usize = 10 * 1024 * 1024;

/// Longitud mínima razonable para un documento Y.js binario — valores más
/// cortos son sospechosos (un doc vacío legítimo ya ocupa >=4 bytes de header).
const MIN_BINARY_LEN: usize = 4;

/// Patrones que no deberían aparecer en un Y.js binario — su presencia sugiere
/// que alguien está intentando pasar HTML/JS disfrazado.
const SUSPICIOUS_BINARY_PATTERNS: &[&str] = &[
    "<html",
    "<!doctype",
    "<script",
    "javascript:",
    "data:",
    "<iframe",
];

/// Tags personalizados del editor de Plane (además de los defaults de ammonia).
/// Deben coincidir con `CUSTOM_TAGS` en Python.
const CUSTOM_TAGS: &[&str] = &["mention-component", "label", "input", "image-component"];

/// Atributos "genéricos" permitidos en cualquier tag.
/// Mirror de `ATTRIBUTES["*"]` en Python.
const GENERIC_ATTRS: &[&str] = &[
    "class",
    "id",
    "title",
    "role",
    "aria-label",
    "aria-hidden",
    "style",
    "start",
    "type",
    "xmlns",
    // data-* conocidos del editor
    "data-tight",
    "data-node-type",
    "data-type",
    "data-checked",
    "data-background-color",
    "data-text-color",
    "data-name",
    "data-id",
    // callouts
    "data-icon-name",
    "data-icon-color",
    "data-background",
    "data-emoji-unicode",
    "data-emoji-url",
    "data-logo-in-use",
    "data-block-type",
];

/// Protocolos permitidos en URLs.
/// Mirror de `SAFE_PROTOCOLS` en Python.
const SAFE_PROTOCOLS: &[&str] = &["http", "https", "mailto", "tel"];

/// Prefijos de atributos permitidos (genéricos) — equivalente al manejo
/// dinámico de `data-*` en Python, que inspecciona el HTML de entrada.
/// Con `ammonia` usamos `generic_attribute_prefixes` para aceptar cualquier
/// `data-*` sin tener que enumerarlos uno por uno.
const GENERIC_ATTR_PREFIXES: &[&str] = &["data-"];

/// Construye (una sola vez) el `ammonia::Builder` reutilizable con toda la
/// configuración del editor. Cachear es barato y evita reconstruir la
/// estructura por cada request — que se repite con alta frecuencia cuando el
/// cliente colaborativo persiste HTML en cada pulsación.
fn builder() -> &'static ammonia::Builder<'static> {
    static BUILDER: OnceLock<ammonia::Builder<'static>> = OnceLock::new();
    BUILDER.get_or_init(|| {
        let mut b = ammonia::Builder::default();

        // Tags: defaults + custom del editor
        b.add_tags(CUSTOM_TAGS.iter().copied());

        // Atributos genéricos (aplican a "*")
        b.add_generic_attributes(GENERIC_ATTRS.iter().copied());

        // data-* dinámico sin enumerar
        b.add_generic_attribute_prefixes(GENERIC_ATTR_PREFIXES.iter().copied());

        // Atributos específicos por tag — mirror de `ATTRIBUTES` por clave
        let mut per_tag: HashMap<&'static str, HashSet<&'static str>> = HashMap::new();
        per_tag.insert("a", ["href", "target"].into_iter().collect());
        per_tag.insert(
            "image-component",
            [
                "id",
                "width",
                "height",
                "aspectRatio",
                "aspectratio",
                "src",
                "alignment",
                "status",
            ]
            .into_iter()
            .collect(),
        );
        per_tag.insert(
            "img",
            [
                "width",
                "height",
                "aspectRatio",
                "aspectratio",
                "alignment",
                "src",
                "alt",
                "title",
            ]
            .into_iter()
            .collect(),
        );
        per_tag.insert(
            "mention-component",
            ["id", "entity_identifier", "entity_name"].into_iter().collect(),
        );
        per_tag.insert(
            "th",
            ["colspan", "rowspan", "colwidth", "background", "style"]
                .into_iter()
                .collect(),
        );
        per_tag.insert(
            "td",
            [
                "colspan",
                "rowspan",
                "colwidth",
                "background",
                "textColor",
                "textcolor",
                "style",
            ]
            .into_iter()
            .collect(),
        );
        per_tag.insert(
            "tr",
            ["background", "textColor", "textcolor", "style"]
                .into_iter()
                .collect(),
        );
        per_tag.insert("pre", ["language"].into_iter().collect());
        per_tag.insert("code", ["language", "spellcheck"].into_iter().collect());
        per_tag.insert("input", ["type", "checked"].into_iter().collect());
        for (tag, attrs) in per_tag {
            b.add_tag_attributes(tag, attrs);
        }

        // URL schemes permitidos
        b.url_schemes(SAFE_PROTOCOLS.iter().copied().collect());

        b
    })
}

/// Valida un blob binario (Y.js document) antes de persistirlo.
///
/// Devuelve `Ok(())` si pasa, `Err(msg)` si alguna regla falla. Un buffer vacío
/// se considera válido (Django lo acepta como "sin contenido aún").
pub fn validate_binary_data(data: &[u8]) -> Result<(), String> {
    if data.is_empty() {
        return Ok(());
    }
    if data.len() > MAX_CONTENT_SIZE {
        return Err("Binary data exceeds maximum size limit (10MB)".to_string());
    }
    if data.len() < MIN_BINARY_LEN {
        return Err("Binary data too short to be valid document format".to_string());
    }

    // Chequear patrones sospechosos en los primeros 200 bytes tratados como
    // UTF-8 best-effort (bytes inválidos se reemplazan por U+FFFD).
    let prefix_len = data.len().min(200);
    let sniff = String::from_utf8_lossy(&data[..prefix_len]).to_lowercase();
    for pattern in SUSPICIOUS_BINARY_PATTERNS {
        if sniff.contains(pattern) {
            return Err("Binary data contains suspicious content patterns".to_string());
        }
    }

    Ok(())
}

/// Valida y sanitiza HTML. Devuelve el HTML limpio o un error descriptivo.
///
/// * HTML vacío → `Ok(String::new())` sin llamar al sanitizador.
/// * Tamaño > 10 MB → rechaza.
/// * Caso general → `ammonia::clean` con la config del editor.
pub fn sanitize_description(html: &str) -> Result<String, String> {
    if html.is_empty() {
        return Ok(String::new());
    }
    if html.len() > MAX_CONTENT_SIZE {
        return Err("HTML content exceeds maximum size limit (10MB)".to_string());
    }
    Ok(builder().clean(html).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_binary_is_valid() {
        assert!(validate_binary_data(&[]).is_ok());
    }

    #[test]
    fn tiny_binary_rejected() {
        assert!(validate_binary_data(&[1, 2]).is_err());
    }

    #[test]
    fn oversized_binary_rejected() {
        let big = vec![0u8; MAX_CONTENT_SIZE + 1];
        assert!(validate_binary_data(&big).is_err());
    }

    #[test]
    fn suspicious_binary_rejected() {
        let payload = b"<!DOCTYPE html><html><script>alert(1)</script>".to_vec();
        assert!(validate_binary_data(&payload).is_err());
    }

    #[test]
    fn normal_binary_accepted() {
        // 4+ bytes, sin patrones HTML
        let payload = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05];
        assert!(validate_binary_data(&payload).is_ok());
    }

    #[test]
    fn empty_html_returns_empty() {
        assert_eq!(sanitize_description("").unwrap(), "");
    }

    #[test]
    fn html_over_limit_rejected() {
        let big = "a".repeat(MAX_CONTENT_SIZE + 1);
        assert!(sanitize_description(&big).is_err());
    }

    #[test]
    fn script_tag_stripped() {
        let dirty = "<p>hi</p><script>evil()</script>";
        let clean = sanitize_description(dirty).unwrap();
        assert!(!clean.contains("<script"));
        assert!(clean.contains("<p>hi</p>"));
    }

    #[test]
    fn mention_component_preserved() {
        let dirty = r#"<p>hola <mention-component id="u1" entity_identifier="user-id" entity_name="user_mention"></mention-component></p>"#;
        let clean = sanitize_description(dirty).unwrap();
        assert!(
            clean.contains("<mention-component"),
            "mention-component debe preservarse, got: {clean}"
        );
        assert!(clean.contains("entity_identifier"));
    }

    #[test]
    fn data_attributes_preserved() {
        let dirty = r#"<div data-block-type="callout" data-icon-name="sparkles">x</div>"#;
        let clean = sanitize_description(dirty).unwrap();
        assert!(clean.contains("data-block-type"));
        assert!(clean.contains("data-icon-name"));
    }

    #[test]
    fn javascript_url_stripped() {
        let dirty = r#"<a href="javascript:alert(1)">x</a>"#;
        let clean = sanitize_description(dirty).unwrap();
        assert!(!clean.contains("javascript:"));
    }
}
