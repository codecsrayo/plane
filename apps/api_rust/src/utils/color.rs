// src/utils/color.rs
//! Color utilities — mirrors `plane/utils/color.py`.

/// Generate a random hex color string (e.g. `#a3f1b9`).
///
/// Equivalent to Django's `get_random_color()` used as the default value
/// for `Workspace.background_color`.
pub fn get_random_color() -> String {
    let value: u32 = rand::random();
    format!("#{:06x}", value & 0xFF_FF_FF)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_valid_hex_color() {
        let color = get_random_color();
        assert!(color.starts_with('#'));
        assert_eq!(color.len(), 7);
        // all chars after '#' are hex digits
        assert!(color[1..].chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn produces_different_values() {
        let a = get_random_color();
        let b = get_random_color();
        // Technically could collide, but 16M possibilities makes it unlikely
        assert_ne!(a, b);
    }
}
