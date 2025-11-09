use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::api::errors::ApiError;
use crate::domain::repositories::TeamRepository;
use crate::domain::team::value_objects::TeamStatus;
use crate::domain::team::Team;
use crate::infrastructure::repositories::PostgresTeamRepository;

/// Request body for creating a team
#[derive(Debug, Deserialize)]
pub struct CreateTeamRequest {
    pub goal: String,
    pub company_id: Uuid,
    pub created_by: Uuid,
    pub budget_limit: Option<Decimal>,
}

/// Response from team creation
#[derive(Debug, Serialize)]
pub struct TeamResponse {
    pub id: Uuid,
    pub company_id: Uuid,
    pub goal: String,
    pub status: String,
    pub created_by: Uuid,
    pub budget_limit: Option<Decimal>,
}

impl From<&Team> for TeamResponse {
    fn from(team: &Team) -> Self {
        Self {
            id: team.id(),
            company_id: team.company_id(),
            goal: team.goal().to_string(),
            status: format!("{:?}", team.status()),
            created_by: team.created_by(),
            budget_limit: team.budget_limit(),
        }
    }
}

/// Create a new team
///
/// POST /api/teams
pub async fn create_team(
    State(pool): State<PgPool>,
    Json(req): Json<CreateTeamRequest>,
) -> Result<(StatusCode, Json<TeamResponse>), ApiError> {
    // Create team domain entity
    let (team, _events) = Team::new(
        req.company_id,
        req.goal,
        req.created_by,
        req.budget_limit,
    )
    .map_err(|e| ApiError::bad_request(e))?;

    // Save to database
    let team_repo = PostgresTeamRepository::new(pool);
    team_repo
        .save(&team)
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Failed to save team: {}", e)))?;

    Ok((StatusCode::CREATED, Json(TeamResponse::from(&team))))
}

/// Get a team by ID
///
/// GET /api/teams/:id
pub async fn get_team(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TeamResponse>, ApiError> {
    let team_repo = PostgresTeamRepository::new(pool);
    let team = team_repo
        .find_by_id(id)
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Database error: {}", e)))?
        .ok_or_else(|| ApiError::not_found(format!("Team not found: {}", id)))?;

    Ok(Json(TeamResponse::from(&team)))
}

/// Get all teams for a company
///
/// GET /api/teams/company/:company_id
pub async fn get_teams_by_company(
    State(pool): State<PgPool>,
    Path(company_id): Path<Uuid>,
) -> Result<Json<Vec<TeamResponse>>, ApiError> {
    let team_repo = PostgresTeamRepository::new(pool);
    let teams = team_repo
        .find_by_company(company_id)
        .await
        .map_err(|e| ApiError::internal_server_error(format!("Database error: {}", e)))?;

    let responses = teams.iter().map(TeamResponse::from).collect();

    Ok(Json(responses))
}

/// Delete a team
///
/// DELETE /api/teams/:id
pub async fn delete_team(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let team_repo = PostgresTeamRepository::new(pool);
    team_repo
        .delete(id)
        .await
        .map_err(|e| {
            if e.contains("not found") {
                ApiError::not_found(e)
            } else {
                ApiError::internal_server_error(format!("Failed to delete team: {}", e))
            }
        })?;

    Ok(StatusCode::NO_CONTENT)
}
