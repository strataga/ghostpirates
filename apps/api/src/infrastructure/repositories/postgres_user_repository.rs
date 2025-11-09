use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repositories::user_repository::{User, UserRepository};
use crate::domain::user::value_objects::Email;

/// PostgreSQL implementation of UserRepository
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    /// Creates a new PostgresUserRepository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, user: User) -> Result<Uuid, String> {
        sqlx::query!(
            r#"
            INSERT INTO users (
                id, company_id, email, password_hash, full_name, is_active
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            user.id,
            user.company_id,
            user.email.as_str(),
            user.password_hash,
            user.full_name,
            user.is_active
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to create user: {}", e))?;

        Ok(user.id)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, company_id, email, password_hash, full_name, is_active
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user by id: {}", e))?;

        Ok(row
            .map(|r| {
                Email::new(&r.email).map(|email| User {
                    id: r.id,
                    company_id: r.company_id,
                    email,
                    password_hash: r.password_hash,
                    full_name: r.full_name,
                    is_active: r.is_active,
                })
            })
            .transpose()
            .map_err(|e| format!("Invalid email from database: {}", e))?)
    }

    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, String> {
        let row = sqlx::query!(
            r#"
            SELECT id, company_id, email, password_hash, full_name, is_active
            FROM users
            WHERE email = $1
            "#,
            email.as_str()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find user by email: {}", e))?;

        Ok(row
            .map(|r| {
                Email::new(&r.email).map(|email| User {
                    id: r.id,
                    company_id: r.company_id,
                    email,
                    password_hash: r.password_hash,
                    full_name: r.full_name,
                    is_active: r.is_active,
                })
            })
            .transpose()
            .map_err(|e| format!("Invalid email from database: {}", e))?)
    }

    async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<User>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT id, company_id, email, password_hash, full_name, is_active
            FROM users
            WHERE company_id = $1
            ORDER BY full_name
            "#,
            company_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find users by company: {}", e))?;

        rows.into_iter()
            .map(|r| {
                Email::new(&r.email).map(|email| User {
                    id: r.id,
                    company_id: r.company_id,
                    email,
                    password_hash: r.password_hash,
                    full_name: r.full_name,
                    is_active: r.is_active,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("Invalid email from database: {}", e))
    }

    async fn update_last_login(&self, user_id: Uuid) -> Result<(), String> {
        sqlx::query!(
            r#"
            UPDATE users
            SET last_login = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
            user_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to update last login: {}", e))?;

        Ok(())
    }
}
