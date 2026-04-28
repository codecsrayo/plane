// src/utils/serde_date.rs
//! Date/datetime deserializers and tz-aware conversion.
//!
//! Two responsibilities:
//!
//! 1. **`deserialize_date_or_datetime`** — replicates the tolerance of DRF
//!    `serializers.DateTimeField` (ISO 8601). Accepts `"YYYY-MM-DD"` (what the
//!    frontend sends via `renderFormattedPayloadDate`), RFC3339,
//!    `null`, `""`, or missing field. Without this, chrono rejects most
//!    frontend payloads and axum returns 400 before the handler.
//!
//! 2. **`project_tz_to_utc`** — replica of
//!    `apps/api/plane/utils/timezone_converter.py::convert_to_utc`.
//!    Converts a `NaiveDate` to UTC using the project zone as a
//!    local reference, with the same rules as Django:
//!      - `start_date` → 00:00:01 local (or `now()` if the date is today).
//!      - `end_date`   → 23:59:00 local.
//!
//! The deserializer alone fixes the 400 for projects in UTC;
//! `project_tz_to_utc` is necessary for parity with Django in projects
//! with a different time zone.
//!
//! Typical usage in a handler:
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
//! // In the handler, if both dates are present:
//! if let (Some(s), Some(e)) = (body.start_date, body.end_date) {
//!     let start = project_tz_to_utc(s.date_naive(), &project.timezone, true,  Utc::now())?;
//!     let end   = project_tz_to_utc(e.date_naive(), &project.timezone, false, Utc::now())?;
//!     // ... persist start/end.
//! }
//! ```
//!
//! Ref: `apps/api/plane/app/serializers/cycle.py::CycleWriteSerializer::validate`.

use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Deserializer};

/// Accepts `null` / missing / `""` / `"YYYY-MM-DD"` / RFC3339.
///
/// For a date-only input (`"YYYY-MM-DD"`) it returns midnight UTC; the
/// conversion to the project zone is done in the handler.
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

    // 1) Full RFC3339 (sent by webhooks and some integrations).
    if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
        return Ok(Some(dt));
    }

    // 2) Date-only (as sent by the frontend via renderFormattedPayloadDate).
    if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        let naive = date
            .and_hms_opt(0, 0, 0)
            .expect("00:00:00 is always valid");
        return Ok(Some(Utc.from_utc_datetime(&naive).fixed_offset()));
    }

    Err(serde::de::Error::custom(format!(
        "invalid date/datetime: {trimmed:?} (expected YYYY-MM-DD or RFC3339)"
    )))
}

// ── Project-tz aware UTC conversion ──────────────────────────────────────────
//
// Replica of `apps/api/plane/utils/timezone_converter.py::convert_to_utc`.

/// tz-aware conversion errors. The caller is responsible for mapping them to
/// `AppError::BadRequest` (4xx) — they are client input failures, not
/// server failures.
#[derive(Debug, thiserror::Error)]
pub enum TzConvertError {
    #[error("invalid timezone: {0}")]
    InvalidTimezone(String),
    /// DST spring-forward / skipped-day: local midnight does not exist
    /// (e.g. Pacific/Apia skipped 2011-12-30 entirely). pytz.localize
    /// would throw NonExistentTimeError; here we return 400.
    #[error("local midnight does not exist for this date in the project timezone (DST/skip)")]
    AmbiguousLocalTime,
}

/// Converts a `NaiveDate` to a `DateTime<FixedOffset>` in UTC, using the
/// project time zone as a local reference.
///
/// Replica of `convert_to_utc` in Django:
///
/// - `is_start = true`  → 00:00:01 in the project zone. Exception: if
///   `date` matches "today" in that zone, it returns `now_utc` as is
///   (a cycle starting "today" starts now, not last midnight).
/// - `is_start = false` → 23:59:00 in the project zone (last inclusive minute
///   of the day).
///
/// The returned value has offset +00:00 (normalized UTC) for persistence.
///
/// `now_utc` is injected as a parameter so handlers can pass
/// `chrono::Utc::now()` and tests can pass a fixed value —
/// deterministic and testable.
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
        .expect("00:00:00 is always valid");

    // `pytz.localize` on a non-existent hour throws; `from_local_datetime`
    // returns `LocalResult::None`. `single()` flattens to `Option` —
    // ambiguous (fallback) or non-existent (spring-forward) cases are treated as errors.
    let localized = tz
        .from_local_datetime(&naive_midnight)
        .single()
        .ok_or(TzConvertError::AmbiguousLocalTime)?;

    if is_start {
        // 00:00:01 local.
        let localized_plus_1s = localized + chrono::Duration::seconds(1);

        // Django rule (CycleWriteSerializer): if start_date is "today" in
        // the project zone, return `now()` in UTC. This allows a cycle
        // created today to start immediately instead of last midnight.
        let now_in_project_tz = now_utc.with_timezone(&tz);
        if localized_plus_1s.date_naive() == now_in_project_tz.date_naive() {
            return Ok(now_utc.fixed_offset());
        }

        Ok(localized_plus_1s.with_timezone(&Utc).fixed_offset())
    } else {
        // 23:59:00 local (last minute of the day).
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
        // Preserves offset as is
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
        // "2026-13-01" — invalid month 13, matches neither RFC3339 nor %Y-%m-%d.
        let res = parse(r#"{"date": "2026-13-01"}"#);
        assert!(res.is_err());
    }

    // ── project_tz_to_utc ────────────────────────────────────────────────────

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// An arbitrary "now" far from test dates to avoid the
    /// "today" short-circuit except where we explicitly test it.
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
        // America/Bogota = UTC-5 (no DST). Local midnight + 1s = 05:00:01 UTC.
        let r = project_tz_to_utc(ymd(2030, 6, 15), "America/Bogota", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-15T05:00:01+00:00");
    }

    #[test]
    fn bogota_end_rolls_into_next_day_utc() {
        // 23:59 Bogota = 04:59 next day in UTC.
        let r = project_tz_to_utc(ymd(2030, 6, 15), "America/Bogota", false, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2030-06-16T04:59:00+00:00");
    }

    #[test]
    fn tokyo_start_rolls_back_to_previous_day_utc() {
        // Asia/Tokyo = UTC+9. Tokyo midnight = 15:00 UTC previous day.
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
        // "now" = 2026-04-18 15:00 UTC = 2026-04-18 10:00 Bogota.
        // If the user requests start_date="2026-04-18" in Bogota, Django
        // returns now (not last midnight) — parity required.
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 15, 0, 0).unwrap();
        let r = project_tz_to_utc(ymd(2026, 4, 18), "America/Bogota", true, now).unwrap();
        assert_eq!(r.to_rfc3339(), "2026-04-18T15:00:00+00:00");
    }

    #[test]
    fn start_today_rule_does_not_apply_to_end() {
        // The "today" shortcut is only for start. Today's end_date returns 23:59 local.
        let now = Utc.with_ymd_and_hms(2026, 4, 18, 15, 0, 0).unwrap();
        let r = project_tz_to_utc(ymd(2026, 4, 18), "America/Bogota", false, now).unwrap();
        // 23:59 Bogota = 04:59 next day UTC.
        assert_eq!(r.to_rfc3339(), "2026-04-19T04:59:00+00:00");
    }

    #[test]
    fn start_tomorrow_uses_midnight_plus_1s_not_now() {
        // now = 2026-04-18 15:00 UTC. User requests start=2026-04-19 in Bogota.
        // Not "today" → returns 00:00:01 Bogota of 2026-04-19.
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
        // DST spring-forward in NY occurs at 2am, not midnight. The
        // midnight of the change day is still representable.
        let r = project_tz_to_utc(ymd(2026, 3, 8), "America/New_York", true, now_far()).unwrap();
        assert_eq!(r.to_rfc3339(), "2026-03-08T05:00:01+00:00");
    }
}
