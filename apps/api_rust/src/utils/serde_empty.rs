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

/// Deserializa un array tolerante de UUIDs — filtra `null` y `""`.
///
/// El frontend de Plane envía a veces `assignee_ids: [null]` o
/// `label_ids: ["", "<uuid>"]` cuando react-hook-form inicializa un select
/// controlado con valor por default vacío. Serde nativo falla con
/// 422 al intentar parsear `null` o `""` como `Uuid` dentro de `Vec<Uuid>`.
///
/// Django tolera este caso porque `ListField(child=PrimaryKeyRelatedField(...))`
/// corre la validación por item y el `PrimaryKeyRelatedField` trata `None`
/// como inválido pero el serializer en `apps/api/plane/app/serializers/
/// issue.py:149-155` aplica `ProjectMember.objects.filter(member_id__in=...)`
/// — Postgres simplemente descarta los NULLs del IN list.
///
/// Semántica:
/// - campo ausente        → `None`
/// - `null`               → `None`
/// - `[]`                 → `Some(vec![])`
/// - `[null, "", "uuid"]` → `Some(vec![uuid])` (null y "" filtrados)
/// - `["bad"]`            → error de deserialización (400/422)
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
    // Paso 1: deserializar como `Option<Vec<Option<String>>>` — la
    // representación más laxa posible. Acepta ausente, null, o array
    // con items null / string.
    let opt: Option<Vec<Option<String>>> = Option::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    None => continue,                           // filtrar null
                    Some(s) if s.trim().is_empty() => continue, // filtrar ""
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
        // Esta es la regresión específica de la 422 en PATCH issues:
        // frontend envía `assignee_ids: [null]` cuando RHF inicializa el
        // select de "unassigned" con placeholder null.
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
