// src/routes/timezones.rs
//! Endpoint de timezones — paridad exacta con
//! `apps/api/plane/app/views/timezone/base.py` (`TimezoneEndpoint`).
//!
//! Responde a `GET /api/timezones/` con la estructura que el frontend espera
//! (tipo `TTimezones` en `packages/types/src/timezone.ts`):
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
//! Detalles de paridad:
//! - La lista de `(label, value)` reproduce textualmente
//!   `timezone_locations` del view Django (≈115 entradas con nombres amigables).
//! - `utc_offset`/`gmt_offset` se calculan en runtime con `chrono-tz`, el
//!   equivalente IANA de `pytz`. Esto respeta DST — importante en zonas como
//!   `America/Santiago` o `Pacific/Auckland` donde cambia dos veces al año.
//! - Las entradas con timezone desconocida se omiten silenciosamente (mismo
//!   comportamiento que el `except pytz.exceptions.UnknownTimeZoneError` del
//!   view original).
//! - El resultado se ordena por `(offset_minutes, label)` antes de eliminar
//!   la clave interna `offset` — idéntico a Django.
//!
//! Autenticación: `AllowAny` en Django. En este router la ruta no usa
//! extractor `AnyAuth`, por lo que es efectivamente pública; el
//! `rate_limit_headers_middleware` aplicado al `api_router` solo inyecta
//! headers y no exige sesión.
//!
//! Cache: Django usa `@cache_page(60 * 60 * 2)` (2h). Aquí se computa por
//! request — son ~115 lookups en una tabla estática compilada en el binario
//! (microsegundos) y evita servir offsets obsoletos durante transiciones DST.

use axum::{response::IntoResponse, Json};
use chrono::{Offset, Utc};
use chrono_tz::Tz;
use serde::Serialize;

/// Una entrada de timezone tal como la consume el frontend.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TimezoneEntry {
    /// Offset UTC actual en formato `UTC±HH:MM` (respeta DST).
    pub utc_offset: String,
    /// Offset GMT actual en formato `GMT±HH:MM` (respeta DST).
    pub gmt_offset: String,
    /// Nombre amigable para mostrar al usuario (p. ej. `"Bogota"`).
    pub label: String,
    /// Identificador IANA (p. ej. `"America/Bogota"`).
    pub value: String,
}

/// Envoltura de respuesta — paridad con `Response({"timezones": ...})` de DRF.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct TimezonesResponse {
    pub timezones: Vec<TimezoneEntry>,
}

/// Catálogo `(label, value)` — réplica exacta de `timezone_locations` en
/// `apps/api/plane/app/views/timezone/base.py`.
///
/// NO modificar el orden sin sincronizar con el Django view: el frontend
/// agrupa por `value` (`use-timezone.tsx::groupTimezones`) asumiendo que
/// labels duplicadas llegan consecutivas tras el sort estable por offset.
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

/// Formatea un offset (en minutos, con signo) como `±HH:MM` — idéntico al
/// formato que Django compone con `hours_offset` / `minutes_offset`.
///
/// El signo aplica al componente de horas; los minutos siempre son absolutos,
/// replicando `f"{'+' if hours_offset >= 0 else '-'}{abs(hours_offset):02}:{minutes_offset:02}"`.
fn format_offset_hhmm(total_minutes: i32) -> String {
    let sign = if total_minutes >= 0 { '+' } else { '-' };
    let abs_minutes = total_minutes.abs();
    let hours = abs_minutes / 60;
    let minutes = abs_minutes % 60;
    format!("{sign}{hours:02}:{minutes:02}")
}

/// Construye la lista ordenada de timezones con offsets actuales.
///
/// Separada del handler para permitir pruebas unitarias sin levantar axum.
fn build_timezone_list() -> Vec<TimezoneEntry> {
    // `Utc::now()` — equivalente a `datetime.now()` en Django pero explícitamente
    // en UTC para evitar dependencia del TZ del host (Django usa naive now()
    // con astimezone(tz), que produce el mismo resultado al ser agnóstico
    // del reloj local cuando solo interesan offsets actuales).
    let now = Utc::now();

    // Pares `(offset_minutes, entry)` para ordenar antes de stripear el offset.
    let mut entries: Vec<(i32, TimezoneEntry)> = Vec::with_capacity(TIMEZONE_LOCATIONS.len());

    for (label, value) in TIMEZONE_LOCATIONS {
        // `str::parse::<Tz>()` — equivalente a `pytz.timezone(tz_identifier)`.
        // Un valor desconocido devuelve `Err`, lo que se corresponde con
        // `pytz.exceptions.UnknownTimeZoneError` en el `continue` del view.
        let Ok(tz) = value.parse::<Tz>() else {
            tracing::warn!(
                timezone = %value,
                "Timezone desconocida en TIMEZONE_LOCATIONS — entrada omitida"
            );
            continue;
        };

        // Offset total en segundos respetando la regla DST vigente ahora.
        // `chrono-tz` compila reglas IANA estáticas, no hace I/O.
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

    // Orden: primero por offset numérico (oeste → este), luego por label
    // alfabéticamente. Replica `sort(key=lambda x: (x["offset"], x["label"]))`.
    //
    // `sort_by` en Rust es estable, igual que `list.sort` en CPython, por lo
    // que entradas con misma clave mantienen orden de inserción — requerido
    // para que labels duplicadas sobre un mismo `value` (Chennai/Kolkata/
    // Mumbai/New Delhi → Asia/Kolkata) aparezcan en el orden del catálogo.
    entries.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.label.cmp(&b.1.label))
    });

    entries.into_iter().map(|(_, entry)| entry).collect()
}

/// `GET /api/timezones/` — lista de timezones soportadas con offset actual.
///
/// Público, sin autenticación. Paridad con Django `TimezoneEndpoint`.
#[utoipa::path(
    get,
    path = "/timezones/",
    tag = "Timezones",
    responses(
        (status = 200, description = "Lista de timezones con offsets UTC/GMT actuales", body = TimezonesResponse),
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
        assert_eq!(format_offset_hhmm(-300), "-05:00"); // Bogotá / EST
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
        // Derivamos el offset numérico desde el string formateado.
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
                "orden violado entre {:?} y {:?}",
                pair[0],
                pair[1]
            );
        }
    }

    #[test]
    fn duplicated_india_entries_all_present() {
        // Chennai/Kolkata/Mumbai/New Delhi comparten `Asia/Kolkata` — los
        // cuatro labels deben sobrevivir al sort para que el agrupamiento
        // del frontend (`groupTimezones`) los consolide correctamente.
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
