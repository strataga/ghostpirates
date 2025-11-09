use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::repositories::TeamRepository;
use crate::domain::team::value_objects::TeamStatus;
use crate::domain::team::Team;

/// PostgreSQL implementation of TeamRepository
///
/// Provides persistence for Team aggregates using SQLx for compile-time
/// verified queries against PostgreSQL.
pub struct PostgresTeamRepository {
    pool: PgPool,
}

impl PostgresTeamRepository {
    /// Creates a new PostgresTeamRepository
    ///
    /// # Arguments
    /// * `pool` - SQLx connection pool for PostgreSQL
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TeamRepository for PostgresTeamRepository {
    async fn save(&self, team: &Team) -> Result<(), String> {
        sqlx::query!(
            r#"
            INSERT INTO teams (
                id, company_id, goal, status, manager_agent_id,
                created_by, created_at, started_at, completed_at, budget_limit
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (id) DO UPDATE SET
                goal = EXCLUDED.goal,
                status = EXCLUDED.status,
                manager_agent_id = EXCLUDED.manager_agent_id,
                started_at = EXCLUDED.started_at,
                completed_at = EXCLUDED.completed_at,
                budget_limit = EXCLUDED.budget_limit
            "#,
            team.id(),
            team.company_id(),
            team.goal(),
            team.status() as TeamStatus,
            team.manager_agent_id(),
            team.created_by(),
            team.created_at(),
            team.started_at(),
            team.completed_at(),
            team.budget_limit()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to save team: {}", e))?;

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, String> {
        let row = sqlx::query!(
            r#"
            SELECT
                id, company_id, goal,
                status as "status: TeamStatus",
                manager_agent_id, created_by,
                created_at, started_at, completed_at,
                budget_limit as "budget_limit: Decimal"
            FROM teams
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Failed to find team by id: {}", e))?;

        Ok(row.map(|r| {
            Team::from_persistence(
                r.id,
                r.company_id,
                r.goal,
                r.status,
                r.manager_agent_id,
                r.created_by,
                r.created_at,
                r.started_at,
                r.completed_at,
                r.budget_limit,
            )
        }))
    }

    async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, company_id, goal,
                status as "status: TeamStatus",
                manager_agent_id, created_by,
                created_at, started_at, completed_at,
                budget_limit as "budget_limit: Decimal"
            FROM teams
            WHERE company_id = $1
            ORDER BY created_at DESC
            "#,
            company_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find teams by company: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                Team::from_persistence(
                    r.id,
                    r.company_id,
                    r.goal,
                    r.status,
                    r.manager_agent_id,
                    r.created_by,
                    r.created_at,
                    r.started_at,
                    r.completed_at,
                    r.budget_limit,
                )
            })
            .collect())
    }

    async fn find_by_creator(&self, user_id: Uuid) -> Result<Vec<Team>, String> {
        let rows = sqlx::query!(
            r#"
            SELECT
                id, company_id, goal,
                status as "status: TeamStatus",
                manager_agent_id, created_by,
                created_at, started_at, completed_at,
                budget_limit as "budget_limit: Decimal"
            FROM teams
            WHERE created_by = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to find teams by creator: {}", e))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                Team::from_persistence(
                    r.id,
                    r.company_id,
                    r.goal,
                    r.status,
                    r.manager_agent_id,
                    r.created_by,
                    r.created_at,
                    r.started_at,
                    r.completed_at,
                    r.budget_limit,
                )
            })
            .collect())
    }

    async fn delete(&self, id: Uuid) -> Result<(), String> {
        let result = sqlx::query!(
            r#"
            DELETE FROM teams WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Failed to delete team: {}", e))?;

        if result.rows_affected() == 0 {
            return Err(format!("Team not found: {}", id));
        }

        Ok(())
    }
}
