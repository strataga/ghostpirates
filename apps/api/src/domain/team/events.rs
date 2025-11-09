use uuid::Uuid;

/// Domain events that occur within the Team aggregate
///
/// These events represent important business moments in a team's lifecycle.
/// They are used for:
/// - Event sourcing
/// - Publishing to external systems
/// - Auditing team activities
///
/// # Example
/// ```
/// use ghostpirates_api::domain::team::events::TeamEvent;
/// use uuid::Uuid;
///
/// let event = TeamEvent::Created {
///     team_id: Uuid::new_v4(),
///     company_id: Uuid::new_v4(),
///     goal: "Complete mission".to_string(),
///     created_by: Uuid::new_v4(),
/// };
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TeamEvent {
    /// Fired when a team is created
    Created {
        /// ID of the newly created team
        team_id: Uuid,
        /// Company the team belongs to
        company_id: Uuid,
        /// The team's objective
        goal: String,
        /// User who created the team
        created_by: Uuid,
    },
    /// Fired when a team transitions from planning to active
    Started {
        /// ID of the started team
        team_id: Uuid,
    },
    /// Fired when a team completes successfully
    Completed {
        /// ID of the completed team
        team_id: Uuid,
    },
    /// Fired when a team fails
    Failed {
        /// ID of the failed team
        team_id: Uuid,
        /// Reason for failure
        #[allow(dead_code)]
        reason: String,
    },
}

impl TeamEvent {
    /// Returns the team_id for this event
    #[allow(dead_code)]
    pub fn team_id(&self) -> Uuid {
        match self {
            TeamEvent::Created { team_id, .. } => *team_id,
            TeamEvent::Started { team_id } => *team_id,
            TeamEvent::Completed { team_id } => *team_id,
            TeamEvent::Failed { team_id, .. } => *team_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_created_event() {
        let team_id = Uuid::new_v4();
        let company_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();

        let event = TeamEvent::Created {
            team_id,
            company_id,
            goal: "Test goal".to_string(),
            created_by,
        };

        assert_eq!(event.team_id(), team_id);
    }

    #[test]
    fn team_started_event() {
        let team_id = Uuid::new_v4();
        let event = TeamEvent::Started { team_id };

        assert_eq!(event.team_id(), team_id);
    }

    #[test]
    fn team_completed_event() {
        let team_id = Uuid::new_v4();
        let event = TeamEvent::Completed { team_id };

        assert_eq!(event.team_id(), team_id);
    }

    #[test]
    fn team_failed_event() {
        let team_id = Uuid::new_v4();
        let event = TeamEvent::Failed {
            team_id,
            reason: "Out of budget".to_string(),
        };

        assert_eq!(event.team_id(), team_id);
    }

    #[test]
    fn event_clone() {
        let team_id = Uuid::new_v4();
        let event = TeamEvent::Started { team_id };
        let cloned = event.clone();

        assert_eq!(event.team_id(), cloned.team_id());
    }
}
