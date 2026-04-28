// src/utils/csv_sanitize.rs
//! Value sanitization for CSV export — CSV injection prevention.
//!
//! Exact parity with `apps/api/plane/utils/csv_utils.py::sanitize_csv_value`.
//! If the first character of the value belongs to the set of formula triggers
//! recognized by Excel/LibreOffice/Google Sheets, it is prefixed with a
//! single quote (`'`) to force its interpretation as plain text.
//!
//! Triggers (mirror of `_CSV_FORMULA_TRIGGERS` in Django):
//!   `=`, `+`, `-`, `@`, `\t` (tab), `\r` (CR), `\n` (LF).
//!
//! See: <https://owasp.org/www-community/attacks/CSV_Injection>
//!
//! # Context
//!
//! This module consolidates the logic that originally lived duplicated in
//! `src/jobs/export.rs::sanitize_csv_cell`. The previous implementation omitted
//! `\n` in the set of triggers — this file fixes that gap and exposes a
//! single helper reusable by all Rust API CSV exports.
//!
//! # Example
//!
//! ```ignore
//! use crate::utils::csv_sanitize::sanitize_csv_cell;
//!
//! assert_eq!(sanitize_csv_cell("=SUM(A1:A9)"), "'=SUM(A1:A9)");
//! assert_eq!(sanitize_csv_cell("hello"), "hello");
//! assert_eq!(sanitize_csv_cell(""), "");
//! ```

/// Sanitizes an individual value for safe CSV export.
///
/// If `value` starts with a formula trigger character, returns a
/// new `String` with `'` prefixed; otherwise, returns the
/// original value without unnecessarily copying bytes except for the final
/// `to_owned` (required because the signature must be uniform for the CSV writer).
///
/// 1:1 parity with `sanitize_csv_value` in Django.
#[inline]
pub fn sanitize_csv_cell(value: &str) -> String {
    if let Some(first) = value.chars().next() {
        if matches!(first, '=' | '+' | '-' | '@' | '\t' | '\r' | '\n') {
            let mut out = String::with_capacity(value.len() + 1);
            out.push('\'');
            out.push_str(value);
            return out;
        }
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::sanitize_csv_cell;

    #[test]
    fn pass_through_when_safe() {
        assert_eq!(sanitize_csv_cell("hello"), "hello");
        assert_eq!(sanitize_csv_cell("123"), "123");
        assert_eq!(sanitize_csv_cell("a=b"), "a=b"); // trigger not in position 0
    }

    #[test]
    fn empty_string_passes_through() {
        assert_eq!(sanitize_csv_cell(""), "");
    }

    #[test]
    fn prefixes_formula_triggers() {
        assert_eq!(sanitize_csv_cell("=SUM(1,2)"), "'=SUM(1,2)");
        assert_eq!(sanitize_csv_cell("+1+1"), "'+1+1");
        assert_eq!(sanitize_csv_cell("-cmd"), "'-cmd");
        assert_eq!(sanitize_csv_cell("@import"), "'@import");
    }

    #[test]
    fn prefixes_whitespace_triggers() {
        // Django parity: \t, \r, \n in position 0 are also triggers.
        assert_eq!(sanitize_csv_cell("\tvalue"), "'\tvalue");
        assert_eq!(sanitize_csv_cell("\rvalue"), "'\rvalue");
        assert_eq!(sanitize_csv_cell("\nvalue"), "'\nvalue");
    }

    #[test]
    fn unicode_first_char_is_safe() {
        // Non-ASCII characters are not triggers — same Django behavior.
        assert_eq!(sanitize_csv_cell("ñandú"), "ñandú");
        assert_eq!(sanitize_csv_cell("中文"), "中文");
    }
}
