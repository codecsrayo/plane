// src/utils/pagination.rs
//! Django-style cursor pagination.
//!
//! Equivalent to `plane/utils/paginator.py::OffsetPaginator` + `Cursor`.
//!
//! Cursor format: `"{per_page}:{offset}:{is_prev}"` (e.g. `"20:0:0"`).
//!
//! The response is serialized with the same shape that Django produces in
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

/// Cursor parsed from `"per_page:offset:is_prev"`.
///
/// Django uses `value` as the per_page (Cursor.value). Here we name it
/// `per_page` for clarity.
#[derive(Debug, Clone, Copy)]
pub struct Cursor {
    pub per_page: u64,
    pub offset: u64,
    pub is_prev: bool,
}

impl Cursor {
    /// Parses cursor from string. Django accepts `"20:0:0"` and similar.
    ///
    /// Parity with `Cursor.from_string`: 3 parts separated by `:`.
    /// If it fails, it is considered an invalid cursor and we return BadRequest
    /// (same as Django throws ParseError).
    pub fn from_string(s: &str) -> Result<Self, AppError> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return Err(AppError::BadRequest("Invalid cursor parameter.".into()));
        }
        // Django accepts int or float in value; here we only use the value as
        // an integer per_page (it's what is passed in practice). If a float
        // comes in, we truncate it safely.
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

    /// Serialization that Django uses: `{value}:{offset}:{is_prev_int}`.
    pub fn to_string_repr(&self) -> String {
        format!("{}:{}:{}", self.per_page, self.offset, self.is_prev as u32)
    }
}

/// Resolves per_page from query params with the same fallback as Django:
/// if the cursor brings per_page, it's used; if not, `default_per_page`. Always
/// capped by `max_per_page`.
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

/// Parses the cursor from the query — returns default if empty/None.
///
/// The default is `"{default_per_page}:0:0"`, same as Django.
pub fn parse_cursor_or_default(
    raw: Option<&str>,
    default_per_page: u64,
) -> Result<Cursor, AppError> {
    match raw.map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => Cursor::from_string(s),
        None => Ok(Cursor { per_page: default_per_page, offset: 0, is_prev: false }),
    }
}

/// Builds the Django-style paginated body from the already serialized
/// `results` slice and the counters calculated by the caller.
///
/// - `total_count`: total rows that matched the filter (unpaginated).
/// - `limit`: effective per_page used.
/// - `offset_page`: the 0-indexed page requested.
/// - `results`: the records of the current page.
pub fn build_response<T: Serialize>(
    results: Vec<T>,
    total_count: u64,
    limit: u64,
    offset_page: u64,
) -> JsonValue {
    let count = results.len() as u64;
    // Parity with Django: has_next is determined by fetching limit+1 and checking.
    // Here we approximate it with total_count, avoiding the extra query.
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
