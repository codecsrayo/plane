// src/utils/serde_date.rs
//! Deserializers de fecha/datetime y conversión tz-aware.
//!
//! Dos responsabilidades:
//!
//! 1. **`deserialize_date_or_datetime`** — replica la tolerancia de DRF
//!    `serializers.DateTimeField` (ISO 8601). Acepta `"YYYY-MM-DD"` (lo
//!    que envía el frontend via `renderFormattedPayloadDate`), RFC3339,
//!    `null`, `""`, o campo ausente. Sin esto, chrono rechaza la mayoría
//!    de payloads del frontend y axum devuelve 400 antes del handler.
//!
//! 2. **`project_tz_to_utc`** — réplica de
//!    `apps/api/plane/utils/timezone_converter.py::convert_to_utc`.
//!    Convierte una `NaiveDate` a UTC usando la zona del proyecto como
//!    referencia local, con las mismas reglas que Django:
//!      - `start_date` → 00:00:01 local (o `now()` si el date es hoy).
//!      - `end_date`   → 23:59:00 local.
//!
//! El deserializer por sí solo arregla el 400 para proyectos en UTC;
//! `project_tz_to_utc` es necesario para paridad con Django en proyectos
//! con zona horaria distinta.
//!
//! Uso típico en un handler:
//!
//! ```ignore
//! #[derive(Deserialize)]
//! struct Dto {
//!     #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
//!     start_date: Option<chrono::DateTime<chrono::FixedOffset>>,
//!     #[serde(default, deserialize_with = "crate::utils::serde_date::deserialize_date_or_datetime")]
//!     end_date: Option<chrono::DateTime<chrono::FixedOffset>>,
//! }
//!
//! // En el handler, si ambos dates están presentes:
//! if let (Some(s), Some(e)) = (body.start_date, body.end_date) {
//!     let start = project_tz_to_utc(s.date_naive(), &project.timezone, true,  Utc::now())?;
//!     let end   = project_tz_to_utc(e.date_naive(), &project.timezone, false, Utc::now())?;
//!     // ... persistir start/end.
//! }
//! ```
//!
//! Ref: `apps/api/plane/app/serializers/cycle.py::CycleWriteSerializer::validate`.

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

// ── Project-tz aware UTC conversion ──────────────────────────────────────────
//
// Réplica de `apps/api/plane/utils/timezone_converter.py::convert_to_utc`.

/// Errores de conversión tz-aware. El caller es responsable de mapearlos a
/// `AppError::BadRequest` (4xx) — son fallos de input del cliente, no del
/// servidor.
#[derive(Debug, thiserror::Error)]
pub enum TzConvertError {
    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),
    /// DST spring-forward / skipped-day: la medianoche local no existe
    /// (p. ej. Pacific/Apia saltó el 2011-12-30 entero). pytz.localize
    /// lanzaría NonExistentTimeError; aquí devolvemos 400.
    #[error("local midnight does not exist for this date in the project timezone (DST/skip)")]
    AmbiguousLocalTime,
}

/// Convierte una `NaiveDate` a un `DateTime<FixedOffset>` en UTC, usando la
/// zona horaria del proyecto como referencia local.
///
/// Réplica de `convert_to_utc` en Django:
///
/// - `is_start = true`  → 00:00:01 en la zona del proyecto. Excepción: si
///   `date` coincide con "hoy" en esa zona, devuelve `now_utc` tal cual
///   (un ciclo que arranca "hoy" empieza ahora, no a medianoche pasada).
/// - `is_start = false` → 23:59:00 en la zona del proyecto (último minuto
///   inclusivo del día).
///
/// El valor retornado tiene offset +00:00 (UTC normalizado) para persistencia.
///
/// `now_utc` se inyecta por parámetro para que los handlers pasen
/// `chrono::Utc::now()` y los tests puedan pasar un valor fijo —
/// determinista y testeable.
pub fn project_tz_to_utc(
    date: NaiveDate,
    project_timezone: &str,
    is_start: bool,
    now_utc: DateTime<Utc>,
) -> Result<DateTime<FixedOffset>, TzConvertError> {
    let tz: chrono_tz::Tz = project_timezone
        .parse()
        .map_err(|_| TzConvertError::InvalidTimezone(project_timezone.to_owned()))?;

    let naive_midnight = date
        .and_hms_opt(0, 0, 0)
        .expect("00:00:00 siempre es válido");

    // `pytz.localize` sobre una hora inexistente lanza; `from_local_datetime`
    // devuelve `LocalResult::None`. `single()` aplana a `Option` — los casos
    // ambiguos (fall-back) o inexistentes (spring-forward) se tratan como error.
    let localized = tz
        .from_local_datetime(&naive_midnight)
        .single()
        .ok_or(TzConvertError::AmbiguousLocalTime)?;

    if is_start {
        // 00:00:01 local.
        let localized_plus_1s = localized + chrono::Duration::seconds(1);

        // Regla Django (CycleWriteSerializer): si el start_date es "hoy" en
        // la zona del proyecto, devolver `now()` en UTC. Esto permite que un
        // ciclo creado hoy empiece de inmediato en vez de a medianoche pasada.
        let now_in_project_tz = now_utc.with_timezone(&tz);
        if localized_plus_1s.date_naive() == now_in_project_tz.date_naive() {
            return Ok(now_utc.fixed_offset());
        }

        Ok(localized_plus_1s.with_timezone(&Utc).fixed_offset())
    } else {
        // 23:59:00 local (último minuto del día).
        let localized_eod = localized
            + chrono::Duration::hours(23)
            + chrono::Duration::minutes(59);
        Ok(localized_eod.with_timezone(&Utc).fixed_offset())
    }
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

    // ── project_tz_to_utc ────────────────────────────────────────────────────

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Un "now" arbitrario lejos de las fechas de test para evitar el
    /// short-circuit de "hoy" salvo donde explícitamente lo probamos.
    fn now_far() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap()
    }

    #[test]
    fn start_utc_is_one_second_past_midnight() {
        let r = project_tz_to_utc(ymd(2030, 6, 15), "UTC", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-15T00:00:01+00:00");
    }

    #[test]
    fn end_utc_is_last_minute_of_day() {
        let r = project_tz_to_utc(ymd(2030, 6, 15), "UTC", false, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-15T23:59:00+00:00");
    }

    #[test]
    fn bogota_start_shifts_five_hours_forward() {
        // America/Bogota = UTC-5 (sin DST). Medianoche local + 1s = 05:00:01 UTC.
        let r = project_tz_to_utc(ymd(2030, 6, 15), "America/Bogota", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-15T05:00:01+00:00");
    }

    #[test]
    fn bogota_end_rolls_into_next_day_utc() {
        // 23:59 Bogotá = 04:59 del día siguiente en UTC.
        let r = project_tz_to_utc(ymd(2030, 6, 15), "America/Bogota", false, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-16T04:59:00+00:00");
    }

    #[test]
    fn tokyo_start_rolls_back_to_previous_day_utc() {
        // Asia/Tokyo = UTC+9. Medianoche Tokio = 15:00 UTC del día anterior.
        let r = project_tz_to_utc(ymd(2030, 6, 15), "Asia/Tokyo", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-14T15:00:01+00:00");
    }

    #[test]
    fn tokyo_end_same_day_utc() {
        let r = project_tz_to_utc(ymd(2030, 6, 15), "Asia/Tokyo", false, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-15T14:59:00+00:00");
    }

    #[test]
    fn start_today_in_project_tz_returns_now() {
        // "now" = 2026-04-18 15:00 UTC = 2026-04-18 10:00 Bogotá.
        // Si el usuario pide start_date="2026-04-18" en Bogotá, Django
        // devuelve now (no medianoche pasada) — paridad exigida.
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 15, 0, 0).unwrap();
        let r = project_tz_to_utc(ymd(2026, 4, 18), "America/Bogota", true, now).unwrap();
        assert_eq!(r.to_rfc3339(), "2026-04-18T15:00:00+00:00");
    }

    #[test]
    fn start_today_rule_does_not_apply_to_end() {
        // El atajo "hoy" es solo para start. end_date de hoy devuelve 23:59 local.
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 15, 0, 0).unwrap();
        let r = project_tz_to_utc(ymd(2026, 4, 18), "America/Bogota", false, now).unwrap();
        // 23:59 Bogotá = 04:59 del día siguiente UTC.
        assert_eq!(r.to_rfc3339(), "2026-04-19T04:59:00+00:00");
    }

    #[test]
    fn start_tomorrow_uses_midnight_plus_1s_not_now() {
        // now = 2026-04-18 15:00 UTC. Usuario pide start=2026-04-19 en Bogotá.
        // No es "hoy" → devuelve 00:00:01 Bogotá del 2026-04-19.
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 15, 0, 0).unwrap();
        let r = project_tz_to_utc(ymd(2026, 4, 19), "America/Bogota", true, now).unwrap();
        assert_eq!(r.to_rfc3339(), "2026-04-19T05:00:01+00:00");
    }

    #[test]
    fn invalid_timezone_errors() {
        let r = project_tz_to_utc(ymd(2026, 1, 1), "Not/A_Real_Tz", true, now_far());
        assert!(matches!(r, Err(TzConvertError::InvalidTimezone(_))));
    }

    #[test]
    fn ny_spring_forward_midnight_is_unambiguous() {
        // DST spring-forward en NY ocurre a las 2am, no a medianoche. La
        // medianoche del día del cambio sigue siendo representable.
        let r = project_tz_to_utc(ymd(2026, 3, 8), "America/New_York", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2026-03-08T05:00:01+00:00");
    }
}
