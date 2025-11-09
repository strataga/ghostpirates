use async_trait::async_trait;
use uuid::Uuid;
use crate::domain::team::Team;

/// Repository trait for Team aggregate
///
/// Defines the contract for persisting and retrieving teams.
/// Implementations should handle database-specific details.
#[async_trait]
pub trait TeamRepository: Send + Sync {
    /// Save a team (insert or update)
    async fn save(&self, team: &Team) -> Result<(), String>;

    /// Find a team by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, String>;

    /// Find all teams for a company
    async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, String>;

    /// Find all teams created by a specific user
    async fn find_by_creator(&self, user_id: Uuid) -> Result<Vec<Team>, String>;

    /// Delete a team by ID
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
