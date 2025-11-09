use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts},
};
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::auth::jwt::verify_token;

/// JWT authentication extractor for protected routes
///
/// Usage:
/// ```rust
/// async fn protected_handler(
///     JwtAuth(user_id): JwtAuth,
/// ) -> Result<String, ApiError> {
///     Ok(format!("Hello user {}", user_id))
/// }
/// ```
pub struct JwtAuth(pub Uuid);

#[async_trait]
impl<S> FromRequestParts<S> for JwtAuth
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the authorization header
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| ApiError::unauthorized("Missing authorization header"))?;

        // Extract bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| ApiError::unauthorized("Invalid authorization format. Use: Bearer <token>"))?;

        // Get JWT secret from environment
        let secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key".to_string());

        // Verify the token
        let claims = verify_token(token, &secret)
            .map_err(|e| ApiError::unauthorized(format!("Invalid token: {}", e)))?;

        Ok(JwtAuth(claims.sub))
    }
}
