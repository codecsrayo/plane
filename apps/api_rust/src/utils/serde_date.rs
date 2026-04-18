// src/utils/serde_date.rs
//! Deserializers de fecha/datetime que replican la tolerancia de DRF.
//!
//! DRF `serializers.DateTimeField` (ISO 8601) acepta tanto `"YYYY-MM-DD"` como
//! `"YYYY-MM-DDThh:mm:ss[±hh:mm|Z]"` sin que el cliente tenga que declarar
//! cuál está enviando. En Rust, `Option<DateTime<FixedOffset>>` con la
//! implementación `Deserialize` de chrono **rechaza** `"YYYY-MM-DD"` y
//! `""`, lo que devuelve 400 antes de entrar al handler.
//!
//! El frontend de Plane (`packages/utils/src/datetime.ts::renderFormattedPayloadDate`)
//! envía exclusivamente `"YYYY-MM-DD"` para `start_date`/`end_date` de cycles,
//! modules, inbox issues, etc. Sin este wrapper, esas requests fallan.
//!
//! Mapa de entradas → salidas:
//!
//! | Entrada JSON             | Resultado                           |
//! |--------------------------|-------------------------------------|
//! | `null` / ausente         | `None`                              |
//! | `""` / `"   "`           | `None` (paridad con DRF allow_blank)|
//! | `"YYYY-MM-DD"`           | `Some(midnight UTC)` *              |
//! | RFC3339 válido           | `Some(valor)`                       |
//! | Otro                     | Error de deserialización            |
//!
//! \* Cuando el input es date-only, el valor queda como medianoche UTC.
//! El handler debe recalcular con la zona horaria del proyecto (cf.
//! `CycleWriteSerializer.validate` en `apps/api/plane/app/serializers/cycle.py`
//! y `convert_to_utc` en `apps/api/plane/utils/timezone_converter.py`).
//!
//! Uso:
//!
//! ```ignore
//! use crate::utils::serde_date::deserialize_date_or_datetime;
//!
//! #[derive(Deserialize)]
//! struct Dto {
//!     #[serde(default, deserialize_with = "deserialize_date_or_datetime")]
//!     start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
//! }
//! ```

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

/// Acepta `null` / ausente / `""` / `"YYYY-MM-DD"` / RFC3339.
///
/// Para un input date-only (`"YYYY-MM-DD"`) devuelve medianoche UTC; la
/// conversión a la zona del proyecto se hace en el handler.
pub fn deserialize_date_or_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    let Some(raw) = opt else { return Ok(None) };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    // 1) RFC3339 completo (lo que envían webhooks y algunas integraciones).
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(Some(dt));
    }

    // 2) Date-only (lo que envía el frontend via renderFormattedPayloadDate).
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let naive = date
            .and_hms_opt(0, 0, 0)
            .expect("00:00:00 siempre es válido");
        return Ok(Some(Utc.from_utc_datetime(&naive).fixed_offset()));
    }

    Err(serde::de::Error::custom(format!(
        "invalid date/datetime: {trimmed:?} (expected YYYY-MM-DD or RFC3339)"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TestDto {
        #[serde(default, deserialize_with = "deserialize_date_or_datetime")]
        date: Option<DateTime<FixedOffset>>,
    }

    fn parse(json: &str) -> Result<TestDto, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn missing_field_is_none() {
        assert!(parse(r#"{}"#).unwrap().date.is_none());
    }

    #[test]
    fn null_is_none() {
        assert!(parse(r#"{"date": null}"#).unwrap().date.is_none());
    }

    #[test]
    fn empty_string_is_none() {
        assert!(parse(r#"{"date": ""}"#).unwrap().date.is_none());
    }

    #[test]
    fn whitespace_is_none() {
        assert!(parse(r#"{"date": "   "}"#).unwrap().date.is_none());
    }

    #[test]
    fn date_only_is_midnight_utc() {
        let dto = parse(r#"{"date": "2026-04-17"}"#).unwrap();
        let dt = dto.date.unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-04-17T00:00:00+00:00");
    }

    #[test]
    fn rfc3339_z_parses() {
        let dto = parse(r#"{"date": "2026-04-17T14:30:00Z"}"#).unwrap();
        let dt = dto.date.unwrap();
        assert_eq!(dt.with_timezone(&Utc).to_rfc3339(), "2026-04-17T14:30:00+00:00");
    }

    #[test]
    fn rfc3339_with_offset_preserves_offset() {
        let dto = parse(r#"{"date": "2026-04-17T14:30:00+05:30"}"#).unwrap();
        let dt = dto.date.unwrap();
        // Preserva offset tal cual
        assert_eq!(dt.offset().local_minus_utc(), 5 * 3600 + 30 * 60);
    }

    #[test]
    fn invalid_string_errors() {
        let res = parse(r#"{"date": "not a date"}"#);
        assert!(res.is_err());
        let msg = res.unwrap_err().to_string();
        assert!(msg.contains("invalid date/datetime"));
    }

    #[test]
    fn bad_month_errors() {
        // "2026-13-01" — mes 13 inválido, no coincide ni con RFC3339 ni con %Y-%m-%d.
        let res = parse(r#"{"date": "2026-13-01"}"#);
        assert!(res.is_err());
    }
}
