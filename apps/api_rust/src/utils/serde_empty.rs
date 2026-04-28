// src/utils/serde_empty.rs
//! Deserializers that treat empty strings (`""`) as `None`.
//!
//! The Plane frontend by default sends fields like `state_id: ""` when
//! the user has not selected a value (cf. `packages/constants/src/issue/
//! modal.ts`). DRF/Django tolerates that case by converting it to `None`, but serde
//! rejects `""` when deserializing `Option<Uuid>` / `Option<NaiveDate>` and fails
//! the request with 422 Unprocessable Entity.
//!
//! Uso:
//!
//! ```ignore
//! #[derive(Deserialize)]
//! struct Dto {
//!     #[serde(default, deserialize_with = "deserialize_empty_as_none_uuid")]
//!     state_id: Option<Uuid>,
//!     #[serde(default, deserialize_with = "deserialize_empty_as_none_date")]
//!     start_date: Option<chrono::NaiveDate>,
//! }
//! ```
//!
//! Note: we keep one helper per type so that serde infers correctly.
//! A generic helper with `T: FromStr` would struggle with the absence of
//! `Deserialize` impls by default in some types (e.g., `NaiveDate` accepts ISO-8601
//! but not arbitrary paths).

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

/// Accepts `null`, missing field, `""`, or a valid UUID.
///
/// - `null` / missing / `""`  → `None`
/// - Valid UUID              → `Some(Uuid)`
/// - Invalid UUID            → deserialization error (400/422)
pub fn deserialize_empty_as_none_uuid<'de, D>(deserializer: D) -> Result<Option<Uuid>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => Uuid::parse_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Accepts `null`, missing field, `""`, or an ISO-8601 date (`YYYY-MM-DD`).
pub fn deserialize_empty_as_none_date<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => NaiveDate::parse_from_str(&s, "%Y-%m-%d")
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

/// Accepts `null`, missing field, `""`, or any non-empty string.
///
/// Useful for optional text fields (e.g., `priority: ""`).
pub fn deserialize_empty_as_none_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
}

/// Deserializes a tolerant array of UUIDs — filters `null` and `""`.
///
/// The Plane frontend sometimes sends `assignee_ids: [null]` or
/// `label_ids: ["", "<uuid>"]` when react-hook-form initializes a controlled
/// select with an empty default value. Native Serde fails with
/// 422 when trying to parse `null` or `""` as `Uuid` inside `Vec<Uuid>`.
///
/// Django tolerates this case because `ListField(child=PrimaryKeyRelatedField(...))`
/// runs per-item validation and `PrimaryKeyRelatedField` treats `None`
/// as invalid but the serializer in `apps/api/plane/app/serializers/
/// issue.py:149-155` applies `ProjectMember.objects.filter(member_id__in=...)`
/// — Postgres simply discards the NULLs from the IN list.
///
/// Semantics:
/// - missing field        → `None`
/// - `null`               → `None`
/// - `[]`                 → `Some(vec![])`
/// - `[null, "", "uuid"]` → `Some(vec![uuid])` (null and "" filtered)
/// - `["bad"]`            → deserialization error (400/422)
///
/// # Uso
/// ```ignore
/// #[serde(default, deserialize_with = "deserialize_uuid_list_filter_nulls")]
/// pub assignee_ids: Option<Vec<Uuid>>,
/// ```
pub fn deserialize_uuid_list_filter_nulls<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<Uuid>>, D::Error>
where
    D: Deserializer<'de>,
{
    // Step 1: deserialize as `Option<Vec<Option<String>>>` — the
    // loosest possible representation. Accepts missing, null, or array
    // with null / string items.
    let opt: Option<Vec<Option<String>>> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    None => continue,                           // filter null
                    Some(s) if s.trim().is_empty() => continue, // filter ""
                    Some(s) => {
                        let u = Uuid::parse_str(&s).map_err(serde::de::Error::custom)?;
                        out.push(u);
                    }
                }
            }
            Ok(Some(out))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestDto {
        #[serde(default, deserialize_with = "deserialize_empty_as_none_uuid")]
        id: Option<Uuid>,
        #[serde(default, deserialize_with = "deserialize_empty_as_none_date")]
        date: Option<NaiveDate>,
        #[serde(default, deserialize_with = "deserialize_empty_as_none_string")]
        s: Option<String>,
    }

    #[test]
    fn empty_string_is_none_for_uuid() {
        let dto: TestDto = serde_json::from_str(r#"{"id": ""}"#).unwrap();
        assert!(dto.id.is_none());
    }

    #[test]
    fn null_is_none_for_uuid() {
        let dto: TestDto = serde_json::from_str(r#"{"id": null}"#).unwrap();
        assert!(dto.id.is_none());
    }

    #[test]
    fn missing_field_is_none_for_uuid() {
        let dto: TestDto = serde_json::from_str(r#"{}"#).unwrap();
        assert!(dto.id.is_none());
    }

    #[test]
    fn valid_uuid_round_trips() {
        let dto: TestDto =
            serde_json::from_str(r#"{"id": "00000000-0000-0000-0000-000000000001"}"#).unwrap();
        assert_eq!(
            dto.id.unwrap(),
            Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        );
    }

    #[test]
    fn invalid_uuid_fails() {
        let res: Result<TestDto, _> = serde_json::from_str(r#"{"id": "not-a-uuid"}"#);
        assert!(res.is_err());
    }

    #[test]
    fn empty_string_is_none_for_date() {
        let dto: TestDto = serde_json::from_str(r#"{"date": ""}"#).unwrap();
        assert!(dto.date.is_none());
    }

    #[test]
    fn valid_date_parses() {
        let dto: TestDto = serde_json::from_str(r#"{"date": "2025-01-15"}"#).unwrap();
        assert_eq!(dto.date.unwrap(), NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn empty_string_is_none_for_string() {
        let dto: TestDto = serde_json::from_str(r#"{"s": ""}"#).unwrap();
        assert!(dto.s.is_none());
        let dto: TestDto = serde_json::from_str(r#"{"s": "   "}"#).unwrap();
        assert!(dto.s.is_none());
        let dto: TestDto = serde_json::from_str(r#"{"s": "hi"}"#).unwrap();
        assert_eq!(dto.s.as_deref(), Some("hi"));
    }

    #[derive(Deserialize)]
    struct ListDto {
        #[serde(default, deserialize_with = "deserialize_uuid_list_filter_nulls")]
        ids: Option<Vec<Uuid>>,
    }

    #[test]
    fn uuid_list_missing_is_none() {
        let dto: ListDto = serde_json::from_str(r#"{}"#).unwrap();
        assert!(dto.ids.is_none());
    }

    #[test]
    fn uuid_list_null_is_none() {
        let dto: ListDto = serde_json::from_str(r#"{"ids": null}"#).unwrap();
        assert!(dto.ids.is_none());
    }

    #[test]
    fn uuid_list_empty_array_is_some_empty() {
        let dto: ListDto = serde_json::from_str(r#"{"ids": []}"#).unwrap();
        assert_eq!(dto.ids, Some(vec![]));
    }

    #[test]
    fn uuid_list_filters_null_items() {
        // This is the specific regression of the 422 in PATCH issues:
        // frontend sends `assignee_ids: [null]` when RHF initializes the
        // "unassigned" select with null placeholder.
        let dto: ListDto = serde_json::from_str(r#"{"ids": [null]}"#).unwrap();
        assert_eq!(dto.ids, Some(vec![]));
    }

    #[test]
    fn uuid_list_filters_empty_strings() {
        let dto: ListDto = serde_json::from_str(r#"{"ids": [""]}"#).unwrap();
        assert_eq!(dto.ids, Some(vec![]));
    }

    #[test]
    fn uuid_list_mixed_filters_and_keeps() {
        let dto: ListDto = serde_json::from_str(
            r#"{"ids": [null, "", "00000000-0000-0000-0000-000000000001", "   "]}"#,
        )
        .unwrap();
        assert_eq!(
            dto.ids,
            Some(vec![Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()])
        );
    }

    #[test]
    fn uuid_list_invalid_item_fails() {
        let res: Result<ListDto, _> = serde_json::from_str(r#"{"ids": ["not-a-uuid"]}"#);
        assert!(res.is_err());
    }
}
