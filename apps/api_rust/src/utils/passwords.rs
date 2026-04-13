use base64::{engine::general_purpose::STANDARD, Engine as _};
use openssl::{hash::MessageDigest, pkcs5::pbkdf2_hmac};
use uuid::Uuid;

pub const DJANGO_PBKDF2_SHA256: &str = "pbkdf2_sha256";
pub const DJANGO_DEFAULT_ITERATIONS: usize = 600_000;
const DERIVED_KEY_LEN: usize = 32;

pub fn verify_password(password: &str, encoded: &str) -> bool {
    let Some(parsed) = ParsedHash::parse(encoded) else {
        return false;
    };

    if parsed.algorithm != DJANGO_PBKDF2_SHA256 {
        return false;
    }

    let mut derived = [0_u8; DERIVED_KEY_LEN];
    if pbkdf2_hmac(
        password.as_bytes(),
        parsed.salt.as_bytes(),
        parsed.iterations,
        MessageDigest::sha256(),
        &mut derived,
    )
    .is_err()
    {
        return false;
    }

    constant_time_eq(parsed.hash.as_bytes(), STANDARD.encode(derived).as_bytes())
}

pub fn make_password(password: &str) -> anyhow::Result<String> {
    make_password_with_iterations(password, DJANGO_DEFAULT_ITERATIONS)
}

pub fn make_password_with_iterations(password: &str, iterations: usize) -> anyhow::Result<String> {
    let salt = Uuid::new_v4().simple().to_string();
    let mut derived = [0_u8; DERIVED_KEY_LEN];

    pbkdf2_hmac(
        password.as_bytes(),
        salt.as_bytes(),
        iterations,
        MessageDigest::sha256(),
        &mut derived,
    )?;

    Ok(format!(
        "{DJANGO_PBKDF2_SHA256}${iterations}${salt}${}",
        STANDARD.encode(derived)
    ))
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right)
        .fold(0_u8, |acc, (lhs, rhs)| acc | (lhs ^ rhs))
        == 0
}

struct ParsedHash<'a> {
    algorithm: &'a str,
    iterations: usize,
    salt: &'a str,
    hash: &'a str,
}

impl<'a> ParsedHash<'a> {
    fn parse(value: &'a str) -> Option<Self> {
        let mut parts = value.split('$');
        let algorithm = parts.next()?;
        let iterations = parts.next()?.parse().ok()?;
        let salt = parts.next()?;
        let hash = parts.next()?;

        if parts.next().is_some() || salt.is_empty() || hash.is_empty() {
            return None;
        }

        Some(Self {
            algorithm,
            iterations,
            salt,
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{make_password_with_iterations, verify_password};

    #[test]
    fn generated_hash_verifies_with_same_password() {
        let encoded = make_password_with_iterations("CorrectHorseBatteryStaple", 12_000).unwrap();

        assert!(verify_password("CorrectHorseBatteryStaple", &encoded));
        assert!(!verify_password("wrong-password", &encoded));
    }

    #[test]
    fn verify_rejects_unknown_hash_format() {
        assert!(!verify_password("password", "sha256$123$salt$hash"));
        assert!(!verify_password("password", "pbkdf2_sha256$invalid"));
    }
}
