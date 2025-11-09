use serde::{Deserialize, Serialize};

/// Represents the lifecycle status of a team
///
/// # Status Transitions
/// ```text
/// Pending -> Planning -> Active -> Completed
///                            └---> Failed -> Archived
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "team_status", rename_all = "lowercase")]
pub enum TeamStatus {
    /// Team is pending creation/initialization
    Pending,
    /// Team is in planning phase
    Planning,
    /// Team is actively executing
    Active,
    /// Team has completed successfully
    Completed,
    /// Team failed before completion
    Failed,
    /// Team is archived
    Archived,
}

impl TeamStatus {
    /// Checks if a transition from current status to next status is valid
    ///
    /// # Valid Transitions
    /// - Pending -> Planning
    /// - Planning -> Active
    /// - Active -> Completed
    /// - Active -> Failed
    /// - Completed -> Archived
    /// - Failed -> Archived
    ///
    /// # Example
    /// ```
    /// use ghostpirates_api::domain::team::value_objects::TeamStatus;
    ///
    /// assert!(TeamStatus::Pending.can_transition_to(TeamStatus::Planning));
    /// assert!(!TeamStatus::Pending.can_transition_to(TeamStatus::Active));
    /// ```
    pub fn can_transition_to(&self, next: TeamStatus) -> bool {
        use TeamStatus::*;
        matches!(
            (self, next),
            (Pending, Planning)
                | (Planning, Active)
                | (Active, Completed)
                | (Active, Failed)
                | (Completed, Archived)
                | (Failed, Archived)
        )
    }
}

impl std::fmt::Display for TeamStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TeamStatus::Pending => write!(f, "pending"),
            TeamStatus::Planning => write!(f, "planning"),
            TeamStatus::Active => write!(f, "active"),
            TeamStatus::Completed => write!(f, "completed"),
            TeamStatus::Failed => write!(f, "failed"),
            TeamStatus::Archived => write!(f, "archived"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transition_pending_to_planning() {
        assert!(TeamStatus::Pending.can_transition_to(TeamStatus::Planning));
    }

    #[test]
    fn valid_transition_planning_to_active() {
        assert!(TeamStatus::Planning.can_transition_to(TeamStatus::Active));
    }

    #[test]
    fn valid_transition_active_to_completed() {
        assert!(TeamStatus::Active.can_transition_to(TeamStatus::Completed));
    }

    #[test]
    fn valid_transition_active_to_failed() {
        assert!(TeamStatus::Active.can_transition_to(TeamStatus::Failed));
    }

    #[test]
    fn valid_transition_completed_to_archived() {
        assert!(TeamStatus::Completed.can_transition_to(TeamStatus::Archived));
    }

    #[test]
    fn valid_transition_failed_to_archived() {
        assert!(TeamStatus::Failed.can_transition_to(TeamStatus::Archived));
    }

    #[test]
    fn invalid_transition_pending_to_active() {
        assert!(!TeamStatus::Pending.can_transition_to(TeamStatus::Active));
    }

    #[test]
    fn invalid_transition_pending_to_completed() {
        assert!(!TeamStatus::Pending.can_transition_to(TeamStatus::Completed));
    }

    #[test]
    fn invalid_transition_active_to_pending() {
        assert!(!TeamStatus::Active.can_transition_to(TeamStatus::Pending));
    }

    #[test]
    fn invalid_transition_archived_to_anything() {
        assert!(!TeamStatus::Archived.can_transition_to(TeamStatus::Active));
        assert!(!TeamStatus::Archived.can_transition_to(TeamStatus::Pending));
    }

    #[test]
    fn status_display() {
        assert_eq!(TeamStatus::Pending.to_string(), "pending");
        assert_eq!(TeamStatus::Planning.to_string(), "planning");
        assert_eq!(TeamStatus::Active.to_string(), "active");
        assert_eq!(TeamStatus::Completed.to_string(), "completed");
        assert_eq!(TeamStatus::Failed.to_string(), "failed");
        assert_eq!(TeamStatus::Archived.to_string(), "archived");
    }
}
