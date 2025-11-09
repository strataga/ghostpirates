// JWT token creation and verification
// Handles authentication tokens with 8-hour expiry

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// JWT claims structure
///
/// # Fields
/// * `sub` - Subject (user_id)
/// * `exp` - Expiry time (seconds since epoch)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Claims {
    /// User ID (subject)
    pub sub: Uuid,
    /// Expiry timestamp (seconds since epoch)
    pub exp: usize,
}

/// Creates a JWT token for a user
///
/// # Arguments
/// * `user_id` - The user's ID to include in the token
/// * `secret` - The secret key for signing (from environment)
///
/// # Returns
/// * `Ok(String)` - The JWT token
/// * `Err(String)` - If token creation fails
///
/// # Token Properties
/// - Expires after 8 hours
/// - Signed with HS256 algorithm
/// - Contains user_id in 'sub' claim
///
/// # Example
/// ```
/// use ghostpirates_api::auth::jwt::create_token;
/// use uuid::Uuid;
///
/// let user_id = Uuid::new_v4();
/// let secret = "your-secret-key";
/// let token = create_token(user_id, secret).expect("valid token");
/// ```
#[allow(dead_code)]
pub fn create_token(user_id: Uuid, secret: &str) -> Result<String, String> {
    let expiry = Utc::now() + Duration::hours(8);
    let claims = Claims {
        sub: user_id,
        exp: expiry.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|e| e.to_string())
}

/// Verifies and decodes a JWT token
///
/// # Arguments
/// * `token` - The JWT token string to verify
/// * `secret` - The secret key for verification (from environment)
///
/// # Returns
/// * `Ok(Claims)` - The decoded claims if token is valid
/// * `Err(String)` - If token is invalid or expired
///
/// # Example
/// ```
/// use ghostpirates_api::auth::jwt::{create_token, verify_token};
/// use uuid::Uuid;
///
/// let user_id = Uuid::new_v4();
/// let secret = "your-secret-key";
/// let token = create_token(user_id, secret).unwrap();
///
/// let claims = verify_token(&token, secret).expect("valid token");
/// assert_eq!(claims.sub, user_id);
/// ```
#[allow(dead_code)]
pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test-secret-key-for-unit-tests";

    #[test]
    fn create_and_verify_token() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("valid token");

        let claims = verify_token(&token, TEST_SECRET).expect("valid verification");
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn token_contains_user_id() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("valid token");

        let claims = verify_token(&token, TEST_SECRET).expect("valid verification");
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn wrong_secret_fails() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("valid token");

        let result = verify_token(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_token_fails() {
        let result = verify_token("invalid.token.string", TEST_SECRET);
        assert!(result.is_err());
    }

    #[test]
    fn token_expiry_set() {
        let user_id = Uuid::new_v4();
        let token = create_token(user_id, TEST_SECRET).expect("valid token");

        let claims = verify_token(&token, TEST_SECRET).expect("valid verification");
        let expiry_time = claims.exp as i64;
        let now = Utc::now().timestamp();
        let in_8_hours = (Utc::now() + Duration::hours(8)).timestamp();

        // Token should expire within 8 hours (with some buffer for test execution time)
        assert!(expiry_time > now);
        assert!(expiry_time <= in_8_hours + 10); // 10 second buffer
    }
}
