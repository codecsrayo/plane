// src/utils/csv_sanitize.rs
//! Sanitización de valores para exportación CSV — prevención de CSV injection.
//!
//! Paridad exacta con `apps/api/plane/utils/csv_utils.py::sanitize_csv_value`.
//! Si el primer carácter del valor pertenece al conjunto de disparadores de
//! fórmula reconocidos por Excel/LibreOffice/Google Sheets, se prefija con una
//! comilla simple (`'`) para forzar su interpretación como texto plano.
//!
//! Disparadores (espejo de `_CSV_FORMULA_TRIGGERS` en Django):
//!   `=`, `+`, `-`, `@`, `\t` (tab), `\r` (CR), `\n` (LF).
//!
//! Ver: <https://owasp.org/www-community/attacks/CSV_Injection>
//!
//! # Contexto
//!
//! Este módulo consolida la lógica que originalmente vivía duplicada en
//! `src/jobs/export.rs::sanitize_csv_cell`. La implementación previa omitía
//! `\n` en el set de disparadores — este archivo corrige ese gap y expone un
//! único helper reutilizable por todos los exports CSV del API Rust.
//!
//! # Ejemplo
//!
//! ```ignore
//! use crate::utils::csv_sanitize::sanitize_csv_cell;
//!
//! assert_eq!(sanitize_csv_cell("=SUM(A1:A9)"), "'=SUM(A1:A9)");
//! assert_eq!(sanitize_csv_cell("hola"), "hola");
//! assert_eq!(sanitize_csv_cell(""), "");
//! ```

/// Sanitiza un valor individual para exportación CSV segura.
///
/// Si `value` empieza por un carácter disparador de fórmula, devuelve una
/// nueva `String` con `'` prefijada; en caso contrario, devuelve el valor
/// original sin copiar innecesariamente los bytes salvo por el `to_owned`
/// final (requerido porque la firma debe ser uniforme para el writer CSV).
///
/// Paridad 1:1 con `sanitize_csv_value` en Django.
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
        assert_eq!(sanitize_csv_cell("hola"), "hola");
        assert_eq!(sanitize_csv_cell("123"), "123");
        assert_eq!(sanitize_csv_cell("a=b"), "a=b"); // disparador no en posición 0
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
        // Paridad Django: \t, \r, \n en posición 0 también son disparadores.
        assert_eq!(sanitize_csv_cell("\tvalue"), "'\tvalue");
        assert_eq!(sanitize_csv_cell("\rvalue"), "'\rvalue");
        assert_eq!(sanitize_csv_cell("\nvalue"), "'\nvalue");
    }

    #[test]
    fn unicode_first_char_is_safe() {
        // Caracteres no-ASCII no son disparadores — mismo comportamiento Django.
        assert_eq!(sanitize_csv_cell("ñandú"), "ñandú");
        assert_eq!(sanitize_csv_cell("中文"), "中文");
    }
}
