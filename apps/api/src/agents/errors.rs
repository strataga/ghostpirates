use thiserror::Error;

/// Errors that can occur in the agent system
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("LLM API error: {0}")]
    LlmError(String),

    #[error("Invalid team size: {0} (must be 3-5 workers)")]
    InvalidTeamSize(usize),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),

    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

pub type AgentResult<T> = Result<T, AgentError>;
