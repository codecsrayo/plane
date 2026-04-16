// src/utils/pagination.rs
//! Paginación cursor-estilo Django.
//!
//! Equivalente a `plane/utils/paginator.py::OffsetPaginator` + `Cursor`.
//!
//! Formato de cursor: `"{per_page}:{offset}:{is_prev}"` (e.g. `"20:0:0"`).
//!
//! El response se serializa con el mismo shape que Django produce en
//! `paginate()`:
//!
//! ```json
//! {
//!   "grouped_by": null,
//!   "sub_grouped_by": null,
//!   "total_count": N,
//!   "next_cursor": "20:1:0",
//!   "prev_cursor": "20:0:1",
//!   "next_page_results": true,
//!   "prev_page_results": false,
//!   "count": 20,
//!   "total_pages": M,
//!   "total_results": N,
//!   "extra_stats": null,
//!   "results": [...]
//! }
//! ```

use serde::Serialize;
use serde_json::{json, Value as JsonValue};

use crate::error::AppError;

pub const DEFAULT_MAX_LIMIT: u64 = 1000;

/// Cursor parseado desde `"per_page:offset:is_prev"`.
///
/// Django usa `value` como el per_page (Cursor.value). Aquí lo nombramos
/// `per_page` para mayor claridad.
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub per_page: u64,
    pub offset: u64,
    pub is_prev: bool,
}

impl Cursor {
    /// Parsea cursor desde string. Django acepta `"20:0:0"` y similar.
    ///
    /// Paridad con `Cursor.from_string`: 3 partes separadas por `:`.
    /// Si falla, se considera cursor inválido y devolvemos BadRequest
    /// (igual que Django lanza ParseError).
    pub fn from_string(s: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(AppError::BadRequest("Invalid cursor parameter.".into()));
        }
        // Django acepta int o float en value; aquí sólo usamos el valor como
        // per_page entero (es lo que se pasa en la práctica). Si viene float
        // lo truncamos de forma segura.
        let per_page: u64 = parts[0]
            .parse::<f64>()
            .map_err(|_| AppError::BadRequest("Invalid cursor parameter.".into()))?
            as u64;
        let offset: u64 = parts[1]
            .parse()
            .map_err(|_| AppError::BadRequest("Invalid cursor parameter.".into()))?;
        let is_prev_raw: u32 = parts[2]
            .parse()
            .map_err(|_| AppError::BadRequest("Invalid cursor parameter.".into()))?;
        Ok(Self { per_page, offset, is_prev: is_prev_raw != 0 })
    }

    /// Serialización que Django usa: `{value}:{offset}:{is_prev_int}`.
    pub fn to_string_repr(&self) -> String {
        format!("{}:{}:{}", self.per_page, self.offset, self.is_prev as u32)
    }
}

/// Resuelve per_page desde query params con el mismo fallback que Django:
/// si el cursor trae per_page, se usa; si no, `default_per_page`. Siempre
/// tope `max_per_page`.
pub fn resolve_per_page(
    cursor_per_page: Option<u64>,
    query_per_page: Option<u64>,
    default_per_page: u64,
    max_per_page: u64,
) -> u64 {
    let requested = query_per_page
        .or(cursor_per_page)
        .unwrap_or(default_per_page);
    requested.clamp(1, max_per_page.max(1))
}

/// Parsea el cursor desde la query — devuelve default si viene vacío/None.
///
/// El default es `"{default_per_page}:0:0"`, igual que Django.
pub fn parse_cursor_or_default(
    raw: Option<&str>,
    default_per_page: u64,
) -> Result<Cursor, AppError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => Cursor::from_string(s),
        None => Ok(Cursor { per_page: default_per_page, offset: 0, is_prev: false }),
    }
}

/// Construye el body paginado estilo Django a partir del slice `results`
/// ya serializado y los contadores calculados por el caller.
///
/// - `total_count`: total de filas que matchearon el filtro (sin paginar).
/// - `limit`: per_page efectivo usado.
/// - `offset_page`: la página 0-indexada que se pidió.
/// - `results`: los registros de la página actual.
pub fn build_response<T: Serialize>(
    results: Vec<T>,
    total_count: u64,
    limit: u64,
    offset_page: u64,
) -> JsonValue {
    let count = results.len() as u64;
    // Paridad con Django: has_next se determina trayendo limit+1 y chequeando.
    // Aquí lo aproximamos con total_count, evitando la query extra.
    let has_next = (offset_page + 1) * limit < total_count;
    let has_prev = offset_page > 0;

    let next_cursor = Cursor { per_page: limit, offset: offset_page + 1, is_prev: false };
    let prev_cursor = Cursor {
        per_page: limit,
        offset: offset_page.saturating_sub(1),
        is_prev: true,
    };

    // max_hits = ceil(total_count / limit). Django usa math.ceil(count/limit).
    let total_pages = if limit == 0 { 0 } else { total_count.div_ceil(limit) };

    json!({
        "grouped_by": JsonValue::Null,
        "sub_grouped_by": JsonValue::Null,
        "total_count": total_count,
        "next_cursor": next_cursor.to_string_repr(),
        "prev_cursor": prev_cursor.to_string_repr(),
        "next_page_results": has_next,
        "prev_page_results": has_prev,
        "count": count,
        "total_pages": total_pages,
        "total_results": total_count,
        "extra_stats": JsonValue::Null,
        "results": results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_cursor() {
        let c = Cursor::from_string("20:0:0").unwrap();
        assert_eq!(c.per_page, 20);
        assert_eq!(c.offset, 0);
        assert!(!c.is_prev);
    }

    #[test]
    fn parse_with_is_prev() {
        let c = Cursor::from_string("20:3:1").unwrap();
        assert!(c.is_prev);
        assert_eq!(c.offset, 3);
    }

    #[test]
    fn reject_malformed_cursor() {
        assert!(Cursor::from_string("abc").is_err());
        assert!(Cursor::from_string("20:0").is_err());
        assert!(Cursor::from_string("20:0:0:0").is_err());
    }

    #[test]
    fn default_when_empty() {
        let c = parse_cursor_or_default(None, 20).unwrap();
        assert_eq!(c.per_page, 20);
        let c = parse_cursor_or_default(Some(""), 20).unwrap();
        assert_eq!(c.per_page, 20);
    }

    #[test]
    fn response_shape() {
        let body = build_response::<u64>(vec![1, 2, 3], 10, 3, 0);
        assert_eq!(body["total_count"], 10);
        assert_eq!(body["total_pages"], 4); // ceil(10/3)
        assert_eq!(body["count"], 3);
        assert_eq!(body["next_page_results"], true);
        assert_eq!(body["prev_page_results"], false);
        assert_eq!(body["next_cursor"], "3:1:0");
        assert_eq!(body["prev_cursor"], "3:0:1");
    }

    #[test]
    fn response_shape_last_page() {
        let body = build_response::<u64>(vec![10], 10, 3, 3);
        assert_eq!(body["next_page_results"], false);
        assert_eq!(body["prev_page_results"], true);
    }
}
