// src/routes/timezones.rs
//! Timezones endpoint — exact parity with
//! `apps/api/plane/app/views/timezone/base.py` (`TimezoneEndpoint`).
//!
//! Responds to `GET /api/timezones/` with the structure the frontend expects
//! (type `TTimezones` in `packages/types/src/timezone.ts`):
//!
//! ```json
//! {
//!   "timezones": [
//!     {
//!       "utc_offset": "UTC-05:00",
//!       "gmt_offset": "GMT-05:00",
//!       "label":      "Bogota",
//!       "value":      "America/Bogota"
//!     },
//!     ...
//!   ]
//! }
//! ```
//!
//! Parity details:
//! - The `(label, value)` list verbatim reproduces
//!   `timezone_locations` from the Django view (≈115 entries with friendly names).
//! - `utc_offset`/`gmt_offset` are calculated at runtime with `chrono-tz`, the
//!   IANA equivalent of `pytz`. This respects DST — important in zones like
//!   `America/Santiago` or `Pacific/Auckland` where it changes twice a year.
//! - Entries with unknown timezones are silently omitted (same
//!   behavior as the `except pytz.exceptions.UnknownTimeZoneError` in the
//!   original view).
//! - The result is sorted by `(offset_minutes, label)` before removing
//!   the internal `offset` key — identical to Django.
//!
//! Authentication: `AllowAny` in Django. In this router the route doesn't use
//! an `AnyAuth` extractor, so it is effectively public; the
//! `rate_limit_headers_middleware` applied to the `api_router` only injects
//! headers and does not require a session.
//!
//! Cache: Django uses `@cache_page(60 * 60 * 2)` (2h). Here it's computed per
//! request — it's ~115 lookups in a static table compiled into the binary
//! (microseconds) and avoids serving obsolete offsets during DST transitions.

use axum::{response::IntoResponse, Json};
use chrono::{Offset, Utc};
use chrono_tz::Tz;
use serde::Serialize;

/// A timezone entry as consumed by the frontend.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TimezoneEntry {
    /// Current UTC offset in `UTC±HH:MM` format (respects DST).
    pub utc_offset: String,
    /// Current GMT offset in `GMT±HH:MM` format (respects DST).
    pub gmt_offset: String,
    /// Friendly name to show to the user (e.g., `"Bogota"`).
    pub label: String,
    /// IANA identifier (e.g., `"America/Bogota"`).
    pub value: String,
}

/// Response wrapper — parity with DRF's `Response({"timezones": ...})`.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TimezonesResponse {
    pub timezones: Vec<TimezoneEntry>,
}

/// `(label, value)` catalog — exact replica of `timezone_locations` in
/// `apps/api/plane/app/views/timezone/base.py`.
///
/// DO NOT modify the order without synchronizing with the Django view: the frontend
/// groups by `value` (`use-timezone.tsx::groupTimezones`) assuming that
/// duplicate labels arrive consecutively after the stable sort by offset.
const TIMEZONE_LOCATIONS: &[(&str, &str)] = &[
    ("Midway Island", "Pacific/Midway"),
    ("American Samoa", "Pacific/Pago_Pago"),
    ("Hawaii", "Pacific/Honolulu"),
    ("Aleutian Islands", "America/Adak"),
    ("Marquesas Islands", "Pacific/Marquesas"),
    ("Alaska", "America/Anchorage"),
    ("Gambier Islands", "Pacific/Gambier"),
    ("Pacific Time (US and Canada)", "America/Los_Angeles"),
    ("Baja California", "America/Tijuana"),
    ("Mountain Time (US and Canada)", "America/Denver"),
    ("Arizona", "America/Phoenix"),
    ("Chihuahua, Mazatlan", "America/Chihuahua"),
    ("Central Time (US and Canada)", "America/Chicago"),
    ("Saskatchewan", "America/Regina"),
    ("Guadalajara, Mexico City, Monterrey", "America/Mexico_City"),
    ("Tegucigalpa, Honduras", "America/Tegucigalpa"),
    ("Costa Rica", "America/Costa_Rica"),
    ("Eastern Time (US and Canada)", "America/New_York"),
    ("Lima", "America/Lima"),
    ("Bogota", "America/Bogota"),
    ("Quito", "America/Guayaquil"),
    ("Chetumal", "America/Cancun"),
    ("Caracas (Old Venezuela Time)", "America/Caracas"),
    ("Atlantic Time (Canada)", "America/Halifax"),
    ("Caracas", "America/Caracas"),
    ("Santiago", "America/Santiago"),
    ("La Paz", "America/La_Paz"),
    ("Manaus", "America/Manaus"),
    ("Georgetown", "America/Guyana"),
    ("Bermuda", "Atlantic/Bermuda"),
    ("Newfoundland Time (Canada)", "America/St_Johns"),
    ("Buenos Aires", "America/Argentina/Buenos_Aires"),
    ("Brasilia", "America/Sao_Paulo"),
    ("Greenland", "America/Godthab"),
    ("Montevideo", "America/Montevideo"),
    ("Falkland Islands", "Atlantic/Stanley"),
    (
        "South Georgia and the South Sandwich Islands",
        "Atlantic/South_Georgia",
    ),
    ("Azores", "Atlantic/Azores"),
    ("Cape Verde Islands", "Atlantic/Cape_Verde"),
    ("Dublin", "Europe/Dublin"),
    ("Reykjavik", "Atlantic/Reykjavik"),
    ("Lisbon", "Europe/Lisbon"),
    ("Monrovia", "Africa/Monrovia"),
    ("Casablanca", "Africa/Casablanca"),
    ("Central European Time (Berlin, Rome, Paris)", "Europe/Paris"),
    ("West Central Africa", "Africa/Lagos"),
    ("Algiers", "Africa/Algiers"),
    ("Lagos", "Africa/Lagos"),
    ("Tunis", "Africa/Tunis"),
    ("Eastern European Time (Cairo, Helsinki, Kyiv)", "Europe/Kyiv"),
    ("Athens", "Europe/Athens"),
    ("Jerusalem", "Asia/Jerusalem"),
    ("Johannesburg", "Africa/Johannesburg"),
    ("Harare, Pretoria", "Africa/Harare"),
    ("Moscow Time", "Europe/Moscow"),
    ("Baghdad", "Asia/Baghdad"),
    ("Nairobi", "Africa/Nairobi"),
    ("Kuwait, Riyadh", "Asia/Riyadh"),
    ("Tehran", "Asia/Tehran"),
    ("Abu Dhabi", "Asia/Dubai"),
    ("Baku", "Asia/Baku"),
    ("Yerevan", "Asia/Yerevan"),
    ("Astrakhan", "Europe/Astrakhan"),
    ("Tbilisi", "Asia/Tbilisi"),
    ("Mauritius", "Indian/Mauritius"),
    ("Kabul", "Asia/Kabul"),
    ("Islamabad", "Asia/Karachi"),
    ("Karachi", "Asia/Karachi"),
    ("Tashkent", "Asia/Tashkent"),
    ("Yekaterinburg", "Asia/Yekaterinburg"),
    ("Maldives", "Indian/Maldives"),
    ("Chagos", "Indian/Chagos"),
    ("Chennai", "Asia/Kolkata"),
    ("Kolkata", "Asia/Kolkata"),
    ("Mumbai", "Asia/Kolkata"),
    ("New Delhi", "Asia/Kolkata"),
    ("Sri Jayawardenepura", "Asia/Colombo"),
    ("Kathmandu", "Asia/Kathmandu"),
    ("Dhaka", "Asia/Dhaka"),
    ("Almaty", "Asia/Almaty"),
    ("Bishkek", "Asia/Bishkek"),
    ("Thimphu", "Asia/Thimphu"),
    ("Yangon (Rangoon)", "Asia/Yangon"),
    ("Cocos Islands", "Indian/Cocos"),
    ("Bangkok", "Asia/Bangkok"),
    ("Hanoi", "Asia/Ho_Chi_Minh"),
    ("Jakarta", "Asia/Jakarta"),
    ("Novosibirsk", "Asia/Novosibirsk"),
    ("Krasnoyarsk", "Asia/Krasnoyarsk"),
    ("Beijing", "Asia/Shanghai"),
    ("Singapore", "Asia/Singapore"),
    ("Perth", "Australia/Perth"),
    ("Hong Kong", "Asia/Hong_Kong"),
    ("Ulaanbaatar", "Asia/Ulaanbaatar"),
    ("Palau", "Pacific/Palau"),
    ("Eucla", "Australia/Eucla"),
    ("Tokyo", "Asia/Tokyo"),
    ("Seoul", "Asia/Seoul"),
    ("Yakutsk", "Asia/Yakutsk"),
    ("Adelaide", "Australia/Adelaide"),
    ("Darwin", "Australia/Darwin"),
    ("Sydney", "Australia/Sydney"),
    ("Brisbane", "Australia/Brisbane"),
    ("Guam", "Pacific/Guam"),
    ("Vladivostok", "Asia/Vladivostok"),
    ("Tahiti", "Pacific/Tahiti"),
    ("Lord Howe Island", "Australia/Lord_Howe"),
    ("Solomon Islands", "Pacific/Guadalcanal"),
    ("Magadan", "Asia/Magadan"),
    ("Norfolk Island", "Pacific/Norfolk"),
    ("Bougainville Island", "Pacific/Bougainville"),
    ("Chokurdakh", "Asia/Srednekolymsk"),
    ("Auckland", "Pacific/Auckland"),
    ("Wellington", "Pacific/Auckland"),
    ("Fiji Islands", "Pacific/Fiji"),
    ("Anadyr", "Asia/Anadyr"),
    ("Chatham Islands", "Pacific/Chatham"),
    ("Nuku'alofa", "Pacific/Tongatapu"),
    ("Samoa", "Pacific/Apia"),
    ("Kiritimati Island", "Pacific/Kiritimati"),
];

/// Formats an offset (in minutes, signed) as `±HH:MM` — identical to the
/// format Django composes with `hours_offset` / `minutes_offset`.
///
/// The sign applies to the hours component; minutes are always absolute,
/// replicating `f"{'+' if hours_offset >= 0 else '-'}{abs(hours_offset):02}:{minutes_offset:02}"`.
fn format_offset_hhmm(total_minutes: i32) -> String {
    let sign = if total_minutes >= 0 { '+' } else { '-' };
    let abs_minutes = total_minutes.abs();
    let hours = abs_minutes / 60;
    let minutes = abs_minutes % 60;
    format!("{sign}{hours:02}:{minutes:02}")
}

/// Builds the sorted list of timezones with current offsets.
///
/// Separated from the handler to allow unit tests without starting axum.
fn build_timezone_list() -> Vec<TimezoneEntry> {
    // `Utc::now()` — equivalent to `datetime.now()` in Django but explicitly
    // in UTC to avoid dependency on the host's TZ (Django uses naive now()
    // with astimezone(tz), which produces the same result by being agnostic
    // of the local clock when only current offsets matter).
    let now = Utc::now();

    // `(offset_minutes, entry)` pairs to sort before stripping the offset.
    let mut entries: Vec<(i32, TimezoneEntry)> = Vec::with_capacity(TIMEZONE_LOCATIONS.len());

    for (label, value) in TIMEZONE_LOCATIONS {
        // `str::parse::<Tz>()` — equivalent to `pytz.timezone(tz_identifier)`.
        // An unknown value returns `Err`, which corresponds to
        // `pytz.exceptions.UnknownTimeZoneError` in the view's `continue`.
        let Ok(tz) = value.parse::<Tz>() else {
            tracing::warn!(
                timezone = %value,
                "Unknown timezone in TIMEZONE_LOCATIONS — entry omitted"
            );
            continue;
        };

        // Total offset in seconds respecting current DST rule.
        // `chrono-tz` compiles static IANA rules, no I/O.
        let offset_seconds = now.with_timezone(&tz).offset().fix().local_minus_utc();
        let offset_minutes = offset_seconds / 60;

        let hhmm = format_offset_hhmm(offset_minutes);

        entries.push((
            offset_minutes,
            TimezoneEntry {
                utc_offset: format!("UTC{hhmm}"),
                gmt_offset: format!("GMT{hhmm}"),
                label: (*label).to_string(),
                value: (*value).to_string(),
            },
        ));
    }

    // Order: first by numeric offset (west → east), then by label
    // alphabetically. Replicates `sort(key=lambda x: (x["offset"], x["label"]))`.
    //
    // `sort_by` in Rust is stable, same as `list.sort` in CPython, so
    // entries with same key maintain insertion order — required
    // for duplicate labels on the same `value` (Chennai/Kolkata/
    // Mumbai/New Delhi → Asia/Kolkata) to appear in catalog order.
    entries.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.label.cmp(&b.1.label))
    });

    entries.into_iter().map(|(_, entry)| entry).collect()
}

/// `GET /api/timezones/` — list of supported timezones with current offset.
///
/// Public, no authentication. Parity with Django `TimezoneEndpoint`.
#[utoipa::path(
    get,
    path = "/timezones/",
    tag = "Timezones",
    responses(
        (status = 200, description = "List of timezones with current UTC/GMT offsets", body = TimezonesResponse),
    )
)]
pub async fn list_timezones() -> impl IntoResponse {
    Json(TimezonesResponse {
        timezones: build_timezone_list(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_offset_positive_zero_and_negative() {
        assert_eq!(format_offset_hhmm(0), "+00:00");
        assert_eq!(format_offset_hhmm(330), "+05:30"); // India
        assert_eq!(format_offset_hhmm(-300), "-05:00"); // Bogota / EST
        assert_eq!(format_offset_hhmm(-210), "-03:30"); // Newfoundland
        assert_eq!(format_offset_hhmm(825), "+13:45"); // Chatham DST
    }

    #[test]
    fn bogota_entry_has_expected_offset() {
        // `America/Bogota` no observa DST, por lo que su offset es constante.
        let list = build_timezone_list();
        let bogota = list
            .iter()
            .find(|e| e.value == "America/Bogota")
            .expect("Bogota debe estar en la lista");
        assert_eq!(bogota.utc_offset, "UTC-05:00");
        assert_eq!(bogota.gmt_offset, "GMT-05:00");
        assert_eq!(bogota.label, "Bogota");
    }

    #[test]
    fn list_sorted_by_offset_then_label() {
        let list = build_timezone_list();
        // We derive the numeric offset from the formatted string.
        let parse_offset = |e: &TimezoneEntry| -> i32 {
            let s = e.utc_offset.trim_start_matches("UTC");
            let (sign_char, rest) = s.split_at(1);
            let (h, m) = rest.split_once(':').unwrap();
            let h: i32 = h.parse().unwrap();
            let m: i32 = m.parse().unwrap();
            let mag = h * 60 + m;
            if sign_char == "-" { -mag } else { mag }
        };

        for pair in list.windows(2) {
            let a = parse_offset(&pair[0]);
            let b = parse_offset(&pair[1]);
            assert!(
                a < b || (a == b && pair[0].label <= pair[1].label),
                "order violated between {:?} and {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn duplicated_india_entries_all_present() {
        // Chennai/Kolkata/Mumbai/New Delhi share `Asia/Kolkata` — all
        // four labels must survive the sort so that frontend grouping
        // (`groupTimezones`) consolidates them correctly.
        let list = build_timezone_list();
        let india_labels: Vec<&str> = list
            .iter()
            .filter(|e| e.value == "Asia/Kolkata")
            .map(|e| e.label.as_str())
            .collect();
        assert!(india_labels.contains(&"Chennai"));
        assert!(india_labels.contains(&"Kolkata"));
        assert!(india_labels.contains(&"Mumbai"));
        assert!(india_labels.contains(&"New Delhi"));
    }
}
