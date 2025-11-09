// Agent state management (US-304.9 - US-304.12)
//
// This module will track the state of the agent system

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub team_id: Uuid,
    pub current_phase: String,
    pub active_workers: Vec<Uuid>,
    pub pending_tasks: Vec<Uuid>,
}

// TODO: Implement StateManager in US-304.10
