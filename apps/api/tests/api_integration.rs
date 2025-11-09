//! End-to-end API integration tests
//!
//! These tests verify the complete HTTP API flows including:
//! - User registration and authentication
//! - Team creation and management
//! - JWT authentication on protected endpoints
//! - Database persistence verification

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use ghostpirates_api::api::handlers::{auth as auth_handlers, teams};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::util::ServiceExt; // for oneshot

/// Setup test application with routes
async fn setup_app(pool: PgPool) -> Router {
    use axum::routing::{delete, get, post};

    Router::new()
        .route("/api/auth/register", post(auth_handlers::register))
        .route("/api/auth/login", post(auth_handlers::login))
        .route("/api/teams", post(teams::create_team))
        .route("/api/teams/:id", get(teams::get_team))
        .route(
            "/api/teams/company/:company_id",
            get(teams::get_teams_by_company),
        )
        .route("/api/teams/:id", delete(teams::delete_team))
        .route("/health", get(auth_handlers::health_check))
        .with_state(pool)
}

/// Setup test database connection
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create a test company for isolation
async fn create_test_company(pool: &PgPool) -> uuid::Uuid {
    let company_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO companies (id, name) VALUES ($1, $2)",
        company_id,
        "E2E Test Company"
    )
    .execute(pool)
    .await
    .expect("Failed to create test company");

    company_id
}

/// Clean up test data
async fn cleanup_test_company(pool: &PgPool, company_id: uuid::Uuid) {
    sqlx::query!("DELETE FROM companies WHERE id = $1", company_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup test company");
}

#[tokio::test]
async fn test_health_check() {
    let pool = setup_test_db().await;
    let app = setup_app(pool).await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn test_register_user() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    let register_payload = json!({
        "email": "e2e-register@test.com",
        "password": "testpassword123",
        "full_name": "E2E Test User",
        "company_id": company_id.to_string()
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["user_id"].is_string());
    assert_eq!(json["message"], "User registered successfully");

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_register_and_login_flow() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    // Step 1: Register user
    let register_payload = json!({
        "email": "e2e-login-flow@test.com",
        "password": "securepass456",
        "full_name": "Login Flow User",
        "company_id": company_id.to_string()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    // Step 2: Login with registered credentials
    let login_payload = json!({
        "email": "e2e-login-flow@test.com",
        "password": "securepass456"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert!(json["token"].is_string());
    assert!(json["user_id"].is_string());
    assert!(!json["token"].as_str().unwrap().is_empty());

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_create_team_via_api_and_verify_in_database() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    // Register user first
    let register_payload = json!({
        "email": "e2e-team-creator@test.com",
        "password": "teampass789",
        "full_name": "Team Creator",
        "company_id": company_id.to_string()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user_json: Value = serde_json::from_slice(&body).unwrap();
    let user_id = user_json["user_id"].as_str().unwrap();

    // Create team via API
    let team_payload = json!({
        "goal": "E2E Test Mission",
        "company_id": company_id.to_string(),
        "created_by": user_id,
        "budget_limit": 250.50
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/teams")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&team_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_json: Value = serde_json::from_slice(&body).unwrap();

    let team_id = team_json["id"].as_str().unwrap();
    assert_eq!(team_json["goal"], "E2E Test Mission");
    assert_eq!(team_json["status"], "Pending");

    // Verify team exists in database
    let db_team = sqlx::query!(
        "SELECT id, goal, status::text as status, budget_limit FROM teams WHERE id = $1",
        uuid::Uuid::parse_str(team_id).unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Team should exist in database");

    assert_eq!(db_team.goal, "E2E Test Mission");
    assert_eq!(db_team.status.as_deref(), Some("pending"));
    assert_eq!(
        db_team.budget_limit.unwrap(),
        rust_decimal::Decimal::new(25050, 2)
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_full_user_journey_register_login_create_team() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    // Step 1: Register
    let register_payload = json!({
        "email": "e2e-full-journey@test.com",
        "password": "journey123",
        "full_name": "Journey User",
        "company_id": company_id.to_string()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user_json: Value = serde_json::from_slice(&body).unwrap();
    let user_id = user_json["user_id"].as_str().unwrap().to_string();

    // Step 2: Login
    let login_payload = json!({
        "email": "e2e-full-journey@test.com",
        "password": "journey123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_json: Value = serde_json::from_slice(&body).unwrap();
    let _token = login_json["token"].as_str().unwrap();

    // Step 3: Create Team
    let team_payload = json!({
        "goal": "Complete user journey mission",
        "company_id": company_id.to_string(),
        "created_by": user_id,
        "budget_limit": 500.00
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/teams")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&team_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(team_json["goal"], "Complete user journey mission");
    assert_eq!(team_json["company_id"], company_id.to_string());
    assert_eq!(team_json["created_by"], user_id);

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_protected_endpoint_requires_authentication() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    // Create a team first (so we have a valid ID)
    let user_id = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO users (id, company_id, email, password_hash, full_name, is_active)
         VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        company_id,
        "protected-test@test.com",
        "hash",
        "Protected Test",
        true
    )
    .execute(&pool)
    .await
    .unwrap();

    let team_id = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO teams (id, company_id, goal, status, created_by, budget_limit)
         VALUES ($1, $2, $3, $4::team_status, $5, $6)"
    )
    .bind(team_id)
    .bind(company_id)
    .bind("Protected Team")
    .bind("pending")
    .bind(user_id)
    .bind(rust_decimal::Decimal::new(10000, 2))
    .execute(&pool)
    .await
    .unwrap();

    // Try to access protected endpoint WITHOUT token
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/teams/{}", team_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["error"], "Missing authorization header");

    // Now login and get token
    let login_payload = json!({
        "email": "protected-test@test.com",
        "password": "testpass"
    });

    // Note: This will fail because the password hash is wrong, but we're testing
    // the protected endpoint authentication, not the login flow
    // For a real test with valid token, we'd need to properly hash the password

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_protected_endpoint_with_valid_token() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let app = setup_app(pool.clone()).await;

    // Register and login to get valid token
    let register_payload = json!({
        "email": "protected-valid@test.com",
        "password": "validpass123",
        "full_name": "Protected Valid User",
        "company_id": company_id.to_string()
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&register_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let user_json: Value = serde_json::from_slice(&body).unwrap();
    let user_id = user_json["user_id"].as_str().unwrap();

    // Login to get token
    let login_payload = json!({
        "email": "protected-valid@test.com",
        "password": "validpass123"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let login_json: Value = serde_json::from_slice(&body).unwrap();
    let token = login_json["token"].as_str().unwrap();

    // Create a team
    let team_payload = json!({
        "goal": "Protected endpoint test",
        "company_id": company_id.to_string(),
        "created_by": user_id,
        "budget_limit": 100.00
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/teams")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&team_payload).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let team_json: Value = serde_json::from_slice(&body).unwrap();
    let team_id = team_json["id"].as_str().unwrap();

    // Access protected endpoint WITH valid token
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/teams/{}", team_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let retrieved_team: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(retrieved_team["id"], team_id);
    assert_eq!(retrieved_team["goal"], "Protected endpoint test");

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}
