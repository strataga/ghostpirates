use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Analysis of a user's goal by the Manager Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalAnalysis {
    pub core_objective: String,
    pub subtasks: Vec<String>,
    pub required_specializations: Vec<String>,
    pub estimated_timeline_hours: f32,
    pub potential_blockers: Vec<String>,
    pub success_criteria: Vec<String>,
}

/// Specification for a worker agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerSpec {
    pub specialization: String,
    pub skills: Vec<String>,
    pub responsibilities: Vec<String>,
    pub required_tools: Vec<String>,
}

/// Output from a worker's task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub result: serde_json::Value,
    pub artifacts: Vec<String>,
    pub logs: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Worker specialization types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Specialization {
    Researcher,
    Coder,
    Reviewer,
    Tester,
    Writer,
}

impl std::fmt::Display for Specialization {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Specialization::Researcher => write!(f, "Researcher"),
            Specialization::Coder => write!(f, "Coder"),
            Specialization::Reviewer => write!(f, "Reviewer"),
            Specialization::Tester => write!(f, "Tester"),
            Specialization::Writer => write!(f, "Writer"),
        }
    }
}

/// Worker status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Working,
    Blocked,
}

/// Review decision from Manager Agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewDecision {
    Approved,
    RevisionRequested { feedback: String },
    Rejected { reason: String },
}
