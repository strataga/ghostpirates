use super::events::TeamEvent;
use super::value_objects::TeamStatus;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Team aggregate root
///
/// Represents a team of AI agents working together on a mission.
/// Enforces all business rules related to team lifecycle.
///
/// # Invariants
/// - Goal cannot be empty
/// - Budget must be positive (if specified)
/// - Status transitions must follow defined rules
/// - Timestamps maintain chronological order
///
/// # Example
/// ```
/// use ghostpirates_api::domain::team::Team;
/// use uuid::Uuid;
///
/// let (team, events) = Team::new(
///     Uuid::new_v4(),
///     "Complete mission".to_string(),
///     Uuid::new_v4(),
///     None,
/// ).expect("valid team");
///
/// assert_eq!(team.goal(), "Complete mission");
/// assert!(!events.is_empty());
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Team {
    id: Uuid,
    company_id: Uuid,
    goal: String,
    status: TeamStatus,
    manager_agent_id: Option<Uuid>,
    created_by: Uuid,
    #[allow(dead_code)]
    created_at: DateTime<Utc>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    budget_limit: Option<Decimal>,
}

#[allow(dead_code)]
impl Team {
    /// Creates a new Team aggregate
    ///
    /// # Arguments
    /// * `company_id` - The company this team belongs to
    /// * `goal` - The team's objective (cannot be empty)
    /// * `created_by` - ID of the user creating the team
    /// * `budget_limit` - Optional budget limit (must be positive if specified)
    ///
    /// # Returns
    /// * `Ok((Team, Vec<TeamEvent>))` - New team and events generated
    /// * `Err(String)` - If any invariant is violated
    ///
    /// # Business Rules Enforced
    /// - Goal must not be empty
    /// - Budget must be positive (if provided)
    /// - Initial status is always Pending
    /// - Team generates a Created event
    pub fn new(
        company_id: Uuid,
        goal: String,
        created_by: Uuid,
        budget_limit: Option<Decimal>,
    ) -> Result<(Self, Vec<TeamEvent>), String> {
        // Validate business rules
        if goal.is_empty() {
            return Err("Goal cannot be empty".to_string());
        }

        if let Some(budget) = budget_limit {
            if budget <= Decimal::ZERO {
                return Err("Budget must be positive".to_string());
            }
        }

        let team = Self {
            id: Uuid::new_v4(),
            company_id,
            goal,
            status: TeamStatus::Pending,
            manager_agent_id: None,
            created_by,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            budget_limit,
        };

        let events = vec![TeamEvent::Created {
            team_id: team.id,
            company_id: team.company_id,
            goal: team.goal.clone(),
            created_by: team.created_by,
        }];

        Ok((team, events))
    }

    /// Starts the team (transitions from Planning to Active)
    ///
    /// # Returns
    /// * `Ok(TeamEvent)` - Started event generated
    /// * `Err(String)` - If team cannot transition from current status
    ///
    /// # Business Rules
    /// - Team must be in Planning status
    /// - Records the start timestamp
    pub fn start(&mut self) -> Result<TeamEvent, String> {
        let next_status = TeamStatus::Active;
        if !self.status.can_transition_to(next_status) {
            return Err(format!("Cannot start team in {:?} status", self.status));
        }

        self.status = next_status;
        self.started_at = Some(Utc::now());

        Ok(TeamEvent::Started { team_id: self.id })
    }

    /// Completes the team successfully
    ///
    /// # Returns
    /// * `Ok(TeamEvent)` - Completed event generated
    /// * `Err(String)` - If team cannot be completed from current status
    #[allow(dead_code)]
    pub fn complete(&mut self) -> Result<TeamEvent, String> {
        let next_status = TeamStatus::Completed;
        if !self.status.can_transition_to(next_status) {
            return Err(format!("Cannot complete team in {:?} status", self.status));
        }

        self.status = next_status;
        self.completed_at = Some(Utc::now());

        Ok(TeamEvent::Completed { team_id: self.id })
    }

    /// Marks the team as failed
    ///
    /// # Arguments
    /// * `reason` - Reason for failure
    ///
    /// # Returns
    /// * `Ok(TeamEvent)` - Failed event generated
    /// * `Err(String)` - If team cannot be marked failed from current status
    #[allow(dead_code)]
    pub fn fail(&mut self, reason: String) -> Result<TeamEvent, String> {
        let next_status = TeamStatus::Failed;
        if !self.status.can_transition_to(next_status) {
            return Err(format!("Cannot fail team in {:?} status", self.status));
        }

        self.status = next_status;
        self.completed_at = Some(Utc::now());

        Ok(TeamEvent::Failed {
            team_id: self.id,
            reason,
        })
    }

    // ===== Getters =====

    /// Returns the team's ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Returns the company ID this team belongs to
    pub fn company_id(&self) -> Uuid {
        self.company_id
    }

    /// Returns the team's goal
    pub fn goal(&self) -> &str {
        &self.goal
    }

    /// Returns the team's current status
    pub fn status(&self) -> TeamStatus {
        self.status
    }

    /// Returns the manager agent ID if assigned
    pub fn manager_agent_id(&self) -> Option<Uuid> {
        self.manager_agent_id
    }

    /// Returns the ID of the user who created the team
    pub fn created_by(&self) -> Uuid {
        self.created_by
    }

    /// Returns the creation timestamp
    #[allow(dead_code)]
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the start timestamp if team has started
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }

    /// Returns the completion timestamp if team has completed
    #[allow(dead_code)]
    pub fn completed_at(&self) -> Option<DateTime<Utc>> {
        self.completed_at
    }

    /// Returns the budget limit if one was set
    #[allow(dead_code)]
    pub fn budget_limit(&self) -> Option<Decimal> {
        self.budget_limit
    }

    /// Reconstructs a Team from persistence layer data
    ///
    /// This method bypasses business rules validation since the data
    /// is already validated and stored in the database.
    ///
    /// # Note
    /// Only to be used by repository implementations for data reconstruction.
    #[allow(clippy::too_many_arguments)]
    pub fn from_persistence(
        id: Uuid,
        company_id: Uuid,
        goal: String,
        status: TeamStatus,
        manager_agent_id: Option<Uuid>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        budget_limit: Option<Decimal>,
    ) -> Self {
        Self {
            id,
            company_id,
            goal,
            status,
            manager_agent_id,
            created_by,
            created_at,
            started_at,
            completed_at,
            budget_limit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_team_with_valid_goal() {
        let company_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let result = Team::new(company_id, "Test goal".to_string(), created_by, None);

        assert!(result.is_ok());
        let (team, events) = result.unwrap();

        assert_eq!(team.goal(), "Test goal");
        assert_eq!(team.company_id(), company_id);
        assert_eq!(team.created_by(), created_by);
        assert_eq!(team.status(), TeamStatus::Pending);
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn create_team_with_empty_goal_fails() {
        let result = Team::new(Uuid::new_v4(), "".to_string(), Uuid::new_v4(), None);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Goal cannot be empty"));
    }

    #[test]
    fn create_team_with_valid_budget() {
        let budget = Decimal::from(1000);
        let result = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
            Some(budget),
        );

        assert!(result.is_ok());
        let (team, _) = result.unwrap();
        assert_eq!(team.budget_limit(), Some(budget));
    }

    #[test]
    fn create_team_with_zero_budget_fails() {
        let result = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
            Some(Decimal::ZERO),
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Budget must be positive"));
    }

    #[test]
    fn create_team_with_negative_budget_fails() {
        let result = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
            Some(Decimal::from(-100)),
        );

        assert!(result.is_err());
    }

    #[test]
    fn team_generates_created_event() {
        let company_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let goal = "Test goal".to_string();

        let (team, events) = Team::new(company_id, goal.clone(), created_by, None).unwrap();

        assert_eq!(events.len(), 1);
        match &events[0] {
            TeamEvent::Created {
                team_id,
                company_id: cid,
                goal: g,
                created_by: cb,
            } => {
                assert_eq!(*team_id, team.id());
                assert_eq!(*cid, company_id);
                assert_eq!(g, &goal);
                assert_eq!(*cb, created_by);
            }
            _ => panic!("Expected Created event"),
        }
    }

    #[test]
    fn team_start_requires_transition_through_planning() {
        let (mut team, _) = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
            None,
        )
        .unwrap();

        // Pending cannot transition directly to Active
        let result = team.start();
        assert!(result.is_err());
    }

    #[test]
    fn team_getters() {
        let company_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let goal = "Test goal".to_string();
        let budget = Some(Decimal::from(500));

        let (team, _) = Team::new(company_id, goal.clone(), created_by, budget).unwrap();

        assert_eq!(team.company_id(), company_id);
        assert_eq!(team.goal(), goal);
        assert_eq!(team.created_by(), created_by);
        assert_eq!(team.budget_limit(), budget);
        assert_eq!(team.status(), TeamStatus::Pending);
        assert_eq!(team.manager_agent_id(), None);
        assert!(team.started_at().is_none());
        assert!(team.completed_at().is_none());
    }
}
