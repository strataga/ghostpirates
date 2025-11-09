use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::auth::jwt::create_token;
use crate::auth::password::{hash_password, verify_password};
use crate::domain::repositories::user_repository::{User, UserRepository};
use crate::domain::user::value_objects::Email;
use crate::infrastructure::repositories::PostgresUserRepository;

/// Request body for user registration
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub company_id: Uuid,
}

/// Response from successful registration
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: Uuid,
    pub message: String,
}

/// Request body for user login
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

/// Response from successful login
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub user_id: Uuid,
}

/// Register a new user
///
/// POST /api/auth/register
pub async fn register(
    State(pool): State<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), ApiError> {
    // Validate email
    let email = Email::new(&req.email)
        .map_err(|e| ApiError::bad_request(format!("Invalid email: {}", e)))?;

    // Validate password (minimum 8 characters)
    if req.password.len() < 8 {
        return Err(ApiError::bad_request(
            "Password must be at least 8 characters",
        ));
    }

    // Hash password
    let password_hash = hash_password(&req.password)
        .map_err(|e| ApiError::internal_server_error(format!("Failed to hash password: {}", e)))?;

    // Create user
    let user = User {
        id: Uuid::new_v4(),
        company_id: req.company_id,
        email,
        password_hash,
        full_name: req.full_name,
        is_active: true,
    };

    // Save to database
    let user_repo = PostgresUserRepository::new(pool);
    let user_id = user_repo.create(user).await.map_err(|e| {
        if e.contains("duplicate") || e.contains("unique") {
            ApiError::bad_request("Email already registered")
        } else {
            ApiError::internal_server_error(format!("Failed to create user: {}", e))
        }
    })?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id,
            message: "User registered successfully".to_string(),
        }),
    ))
}

/// Login with email and password
///
/// POST /api/auth/login
pub async fn login(
    State(pool): State<PgPool>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate email
    let email = Email::new(&req.email)
        .map_err(|e| ApiError::bad_request(format!("Invalid email: {}", e)))?;

    // Find user by email
    let user_repo = PostgresUserRepository::new(pool.clone());
    let user = user_repo
        .find_by_email(&email)
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::unauthorized("Invalid credentials"))?;

    // Check if user is active
    if !user.is_active {
        return Err(ApiError::unauthorized("Account is disabled"));
    }

    // Verify password
    let valid = verify_password(&req.password, &user.password_hash).map_err(|e| {
        ApiError::internal_server_error(format!("Password verification failed: {}", e))
    })?;

    if !valid {
        return Err(ApiError::unauthorized("Invalid credentials"));
    }

    // Update last login
    let _ = user_repo.update_last_login(user.id).await;

    // Create JWT token
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-key".to_string());
    let token = create_token(user.id, &secret)
        .map_err(|e| ApiError::internal_server_error(format!("Failed to create token: {}", e)))?;

    Ok(Json(LoginResponse {
        token,
        user_id: user.id,
    }))
}

/// Health check endpoint
///
/// GET /health
pub async fn health_check() -> &'static str {
    "OK"
}
