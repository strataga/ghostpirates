// Agent message passing system (US-304.1 - US-304.4)
//
// This module will handle communication between agents

use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub from: Uuid,
    pub to: Uuid,
    pub message_type: String,
    pub payload: serde_json::Value,
}

// TODO: Implement MessageBus in US-304.2
