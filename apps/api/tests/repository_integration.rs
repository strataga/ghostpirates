//! Integration tests for repository layer
//!
//! These tests verify that repository implementations correctly interact
//! with the PostgreSQL database, including CRUD operations, tenant isolation,
//! and transaction handling.

use ghostpirates_api::auth::password::hash_password;
use ghostpirates_api::domain::repositories::team_repository::TeamRepository;
use ghostpirates_api::domain::repositories::user_repository::{User, UserRepository};
use ghostpirates_api::domain::team::Team;
use ghostpirates_api::domain::user::value_objects::Email;
use ghostpirates_api::infrastructure::repositories::{
    PostgresTeamRepository, PostgresUserRepository,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Set up test database connection pool
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");

    PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create a test company for isolation
async fn create_test_company(pool: &PgPool) -> Uuid {
    let company_id = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO companies (id, name) VALUES ($1, $2)",
        company_id,
        "Test Company"
    )
    .execute(pool)
    .await
    .expect("Failed to create test company");

    company_id
}

/// Create a test user for team ownership
async fn create_test_user(pool: &PgPool, company_id: Uuid, email: &str) -> Uuid {
    let user_id = Uuid::new_v4();
    let password_hash = hash_password("testpass").expect("hash password");

    sqlx::query!(
        "INSERT INTO users (id, company_id, email, password_hash, full_name, is_active)
         VALUES ($1, $2, $3, $4, $5, $6)",
        user_id,
        company_id,
        email,
        password_hash,
        "Test User",
        true
    )
    .execute(pool)
    .await
    .expect("Failed to create test user");

    user_id
}

/// Clean up test data after each test
async fn cleanup_test_company(pool: &PgPool, company_id: Uuid) {
    // CASCADE DELETE will remove all related users, teams, etc.
    sqlx::query!("DELETE FROM companies WHERE id = $1", company_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup test company");
}

#[tokio::test]
async fn test_user_repository_create_and_find_by_email() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;

    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create test user
    let email = Email::new("test-unique-1@example.com").expect("valid email");
    let password_hash = hash_password("testpassword").expect("hash password");

    let user = User {
        id: Uuid::new_v4(),
        company_id,
        email: email.clone(),
        password_hash,
        full_name: "Test User".to_string(),
        is_active: true,
    };

    // Test: Create user
    let user_id = user_repo
        .create(user.clone())
        .await
        .expect("Failed to create user");

    assert_eq!(user_id, user.id, "User ID should match");

    // Test: Find user by email
    let found_user = user_repo
        .find_by_email(&email)
        .await
        .expect("Failed to find user by email");

    assert!(found_user.is_some(), "User should be found");
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id, "User IDs should match");
    assert_eq!(
        found_user.email.as_str(),
        "test-unique-1@example.com",
        "Emails should match"
    );
    assert_eq!(
        found_user.full_name, "Test User",
        "Full names should match"
    );
    assert_eq!(found_user.is_active, true, "User should be active");

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_user_repository_duplicate_email_fails() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;

    let user_repo = PostgresUserRepository::new(pool.clone());

    let email = Email::new("duplicate@example.com").expect("valid email");
    let password_hash = hash_password("testpassword").expect("hash password");

    // Create first user
    let user1 = User {
        id: Uuid::new_v4(),
        company_id,
        email: email.clone(),
        password_hash: password_hash.clone(),
        full_name: "User One".to_string(),
        is_active: true,
    };

    user_repo
        .create(user1)
        .await
        .expect("First user creation should succeed");

    // Try to create second user with same email
    let user2 = User {
        id: Uuid::new_v4(),
        company_id,
        email: email.clone(),
        password_hash,
        full_name: "User Two".to_string(),
        is_active: true,
    };

    let result = user_repo.create(user2).await;

    assert!(
        result.is_err(),
        "Creating user with duplicate email should fail"
    );

    let error = result.unwrap_err();
    assert!(
        error.to_lowercase().contains("duplicate")
            || error.to_lowercase().contains("unique"),
        "Error should mention duplicate or unique constraint: {}",
        error
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_user_repository_update_last_login() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;

    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create test user
    let email = Email::new("login@example.com").expect("valid email");
    let password_hash = hash_password("testpassword").expect("hash password");

    let user = User {
        id: Uuid::new_v4(),
        company_id,
        email: email.clone(),
        password_hash,
        full_name: "Login Test User".to_string(),
        is_active: true,
    };

    user_repo
        .create(user.clone())
        .await
        .expect("Failed to create user");

    // Update last login
    user_repo
        .update_last_login(user.id)
        .await
        .expect("Failed to update last login");

    // Verify last_login was updated
    let record = sqlx::query!("SELECT last_login FROM users WHERE id = $1", user.id)
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch user");

    assert!(
        record.last_login.is_some(),
        "last_login should be set"
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_team_repository_save_and_find_by_id() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "team-owner@test.com").await;

    let team_repo = PostgresTeamRepository::new(pool.clone());

    // Create test team
    let (team, _events) = Team::new(
        company_id,
        "Test Mission".to_string(),
        user_id,
        Some(rust_decimal::Decimal::new(10000, 2)), // $100.00
    )
    .expect("Valid team");

    // Test: Save team
    team_repo.save(&team).await.expect("Failed to save team");

    // Test: Find team by ID
    let found_team = team_repo
        .find_by_id(team.id())
        .await
        .expect("Failed to find team");

    assert!(found_team.is_some(), "Team should be found");
    let found_team = found_team.unwrap();
    assert_eq!(found_team.id(), team.id(), "Team IDs should match");
    assert_eq!(found_team.goal(), "Test Mission", "Goals should match");
    assert_eq!(
        found_team.company_id(),
        company_id,
        "Company IDs should match"
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_team_repository_find_by_company() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "team-creator@test.com").await;

    let team_repo = PostgresTeamRepository::new(pool.clone());

    // Create multiple teams
    let (team1, _) = Team::new(
        company_id,
        "Mission Alpha".to_string(),
        user_id,
        Some(rust_decimal::Decimal::new(5000, 2)),
    )
    .expect("Valid team");

    let (team2, _) = Team::new(
        company_id,
        "Mission Beta".to_string(),
        user_id,
        Some(rust_decimal::Decimal::new(7500, 2)),
    )
    .expect("Valid team");

    team_repo.save(&team1).await.expect("Failed to save team1");
    team_repo.save(&team2).await.expect("Failed to save team2");

    // Test: Find all teams by company
    let teams = team_repo
        .find_by_company(company_id)
        .await
        .expect("Failed to find teams by company");

    assert_eq!(teams.len(), 2, "Should find 2 teams");
    assert!(
        teams.iter().any(|t| t.id() == team1.id()),
        "Should contain team1"
    );
    assert!(
        teams.iter().any(|t| t.id() == team2.id()),
        "Should contain team2"
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_team_repository_delete() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "team-deleter@test.com").await;

    let team_repo = PostgresTeamRepository::new(pool.clone());

    // Create test team
    let (team, _) = Team::new(
        company_id,
        "Mission to Delete".to_string(),
        user_id,
        None,
    )
    .expect("Valid team");

    team_repo.save(&team).await.expect("Failed to save team");

    // Verify team exists
    let found = team_repo
        .find_by_id(team.id())
        .await
        .expect("Failed to find team");
    assert!(found.is_some(), "Team should exist before delete");

    // Test: Delete team
    team_repo
        .delete(team.id())
        .await
        .expect("Failed to delete team");

    // Verify team no longer exists
    let found = team_repo
        .find_by_id(team.id())
        .await
        .expect("Failed to find team after delete");
    assert!(found.is_none(), "Team should not exist after delete");

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_team_repository_upsert_updates_existing() {
    let pool = setup_test_db().await;
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, company_id, "team-upserter@test.com").await;

    let team_repo = PostgresTeamRepository::new(pool.clone());

    // Create initial team
    let (team, _) = Team::new(
        company_id,
        "Initial Goal".to_string(),
        user_id,
        Some(rust_decimal::Decimal::new(10000, 2)),
    )
    .expect("Valid team");

    // Save team first time
    team_repo.save(&team).await.expect("Failed to save team");

    // Save again (should update, not create new - upsert behavior)
    team_repo
        .save(&team)
        .await
        .expect("Failed to update team");

    // Verify only one team exists (not duplicated)
    let teams = team_repo
        .find_by_company(company_id)
        .await
        .expect("Failed to find teams");

    assert_eq!(teams.len(), 1, "Should only have 1 team (upserted)");

    // Verify it's the same team
    let found_team = teams.first().unwrap();
    assert_eq!(
        found_team.id(),
        team.id(),
        "Team ID should match original"
    );

    // Cleanup
    cleanup_test_company(&pool, company_id).await;
}

#[tokio::test]
async fn test_tenant_isolation_users() {
    let pool = setup_test_db().await;

    // Create two separate companies
    let company1_id = create_test_company(&pool).await;
    let company2_id = create_test_company(&pool).await;

    let user_repo = PostgresUserRepository::new(pool.clone());

    // Create user in company 1 with unique email
    let unique_id = Uuid::new_v4().to_string()[..8].to_string();
    let email1_str = format!("tenant-user1-{}@company1.com", unique_id);
    let email1 = Email::new(&email1_str).expect("valid email");
    let user1 = User {
        id: Uuid::new_v4(),
        company_id: company1_id,
        email: email1.clone(),
        password_hash: hash_password("password1").expect("hash"),
        full_name: "User One".to_string(),
        is_active: true,
    };

    user_repo
        .create(user1.clone())
        .await
        .expect("Failed to create user1");

    // Create user in company 2 with different unique email
    let email2_str = format!("tenant-user2-{}@company2.com", unique_id);
    let email2 = Email::new(&email2_str).expect("valid email");
    let user2 = User {
        id: Uuid::new_v4(),
        company_id: company2_id,
        email: email2,
        password_hash: hash_password("password2").expect("hash"),
        full_name: "User Two".to_string(),
        is_active: true,
    };

    user_repo
        .create(user2)
        .await
        .expect("Should allow different users in different companies");

    // Verify each company can only see their own users
    let company1_users = user_repo
        .find_by_company(company1_id)
        .await
        .expect("Failed to find company1 users");

    let company2_users = user_repo
        .find_by_company(company2_id)
        .await
        .expect("Failed to find company2 users");

    assert_eq!(company1_users.len(), 1, "Company 1 should have 1 user");
    assert_eq!(company2_users.len(), 1, "Company 2 should have 1 user");
    assert_ne!(
        company1_users[0].id, company2_users[0].id,
        "Users should be different"
    );

    // Cleanup
    cleanup_test_company(&pool, company1_id).await;
    cleanup_test_company(&pool, company2_id).await;
}

#[tokio::test]
async fn test_tenant_isolation_teams() {
    let pool = setup_test_db().await;

    // Create two separate companies
    let company1_id = create_test_company(&pool).await;
    let company2_id = create_test_company(&pool).await;

    let team_repo = PostgresTeamRepository::new(pool.clone());

    // Create users for each company
    let user1_id = create_test_user(&pool, company1_id, "user@company1.com").await;
    let user2_id = create_test_user(&pool, company2_id, "user@company2.com").await;

    // Create team in company 1
    let (team1, _) = Team::new(
        company1_id,
        "Company 1 Mission".to_string(),
        user1_id,
        None,
    )
    .expect("Valid team");

    team_repo.save(&team1).await.expect("Failed to save team1");

    // Create team in company 2
    let (team2, _) = Team::new(
        company2_id,
        "Company 2 Mission".to_string(),
        user2_id,
        None,
    )
    .expect("Valid team");

    team_repo.save(&team2).await.expect("Failed to save team2");

    // Verify each company can only see their own teams
    let company1_teams = team_repo
        .find_by_company(company1_id)
        .await
        .expect("Failed to find company1 teams");

    let company2_teams = team_repo
        .find_by_company(company2_id)
        .await
        .expect("Failed to find company2 teams");

    assert_eq!(company1_teams.len(), 1, "Company 1 should have 1 team");
    assert_eq!(company2_teams.len(), 1, "Company 2 should have 1 team");
    assert_ne!(
        company1_teams[0].id(),
        company2_teams[0].id(),
        "Teams should be different"
    );

    // Cleanup
    cleanup_test_company(&pool, company1_id).await;
    cleanup_test_company(&pool, company2_id).await;
}
