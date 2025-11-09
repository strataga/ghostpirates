// Agent event system (US-304.5 - US-304.8)
//
// This module will handle event-based coordination between agents

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    TaskAssigned { task_id: Uuid, worker_id: Uuid },
    TaskCompleted { task_id: Uuid, worker_id: Uuid },
    WorkerCreated { worker_id: Uuid, specialization: String },
    TeamFormed { team_id: Uuid, worker_count: usize },
}

// TODO: Implement EventBus in US-304.6
