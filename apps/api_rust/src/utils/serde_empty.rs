// src/utils/serde_empty.rs
//! Deserializers que tratan strings vacíos (`""`) como `None`.
//!
//! El frontend de Plane envía por defecto campos como `state_id: ""` cuando
//! el usuario no ha seleccionado un valor (cf. `packages/constants/src/issue/
//! modal.ts`). DRF/Django tolera ese caso convirtiéndolo a `None`, pero serde
//! rechaza `""` al deserializar `Option<Uuid>` / `Option<NaiveDate>` y falla
//! el request con 422 Unprocessable Entity.
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
//! Nota: mantenemos un helper por tipo para que serde infiera correctamente.
//! Un helper genérico con `T: FromStr` pelearía con la ausencia de impls de
//! `Deserialize` por defecto en algunos tipos (e.g. `NaiveDate` acepta ISO-8601
//! pero no rutas arbitrarias).

use chrono::NaiveDate;
use serde::{Deserialize, Deserializer};
use uuid::Uuid;

/// Acepta `null`, campo ausente, `""`, o un UUID válido.
///
/// - `null` / ausente / `""`  → `None`
/// - UUID válido              → `Some(Uuid)`
/// - UUID inválido            → error de deserialización (400/422)
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

/// Acepta `null`, campo ausente, `""`, o una fecha ISO-8601 (`YYYY-MM-DD`).
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

/// Acepta `null`, campo ausente, `""`, o cualquier string no vacío.
///
/// Útil para campos de texto opcionales (p. ej. `priority: ""`).
pub fn deserialize_empty_as_none_string<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    Ok(opt.filter(|s| !s.trim().is_empty()))
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
}
