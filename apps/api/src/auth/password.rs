// Password hashing utilities
// Uses bcrypt for secure password hashing

use bcrypt::{hash, verify, DEFAULT_COST};

/// Hashes a password using bcrypt
///
/// # Arguments
/// * `password` - The plaintext password to hash
///
/// # Returns
/// * `Ok(String)` - The bcrypt hash
/// * `Err(String)` - If hashing fails
///
/// # Example
/// ```
/// use ghostpirates_api::auth::password::hash_password;
///
/// let hash = hash_password("my_password").expect("valid hash");
/// ```
#[allow(dead_code)]
pub fn hash_password(password: &str) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| e.to_string())
}

/// Verifies a password against a bcrypt hash
///
/// # Arguments
/// * `password` - The plaintext password to verify
/// * `hash` - The bcrypt hash to verify against
///
/// # Returns
/// * `Ok(bool)` - True if password matches, false otherwise
/// * `Err(String)` - If verification fails
///
/// # Example
/// ```
/// use ghostpirates_api::auth::password::{hash_password, verify_password};
///
/// let hash = hash_password("my_password").unwrap();
/// let valid = verify_password("my_password", &hash).unwrap();
/// assert!(valid);
/// ```
#[allow(dead_code)]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    verify(password, hash).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_password() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("valid hash");

        let valid = verify_password(password, &hash).expect("valid verification");
        assert!(valid);
    }

    #[test]
    fn verify_wrong_password() {
        let password = "test_password_123";
        let hash = hash_password(password).expect("valid hash");

        let valid = verify_password("wrong_password", &hash).expect("valid verification");
        assert!(!valid);
    }

    #[test]
    fn hash_different_outputs() {
        let password = "test_password_123";
        let hash1 = hash_password(password).expect("valid hash");
        let hash2 = hash_password(password).expect("valid hash");

        // Hashes should be different due to salt
        assert_ne!(hash1, hash2);

        // But both should verify the password
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
