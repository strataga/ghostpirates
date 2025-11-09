// Agent system modules
//
// This module contains the autonomous agent system that enables
// AI-powered task decomposition and execution.

pub mod manager;
pub mod worker;
pub mod types;
pub mod errors;
pub mod prompts;
pub mod messages;
pub mod events;
pub mod state;

// Re-export main types
pub use manager::ManagerAgent;
pub use worker::WorkerAgent;
pub use types::{GoalAnalysis, WorkerSpec, TaskOutput};
pub use errors::AgentError;
