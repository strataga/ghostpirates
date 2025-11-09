# Phase 7: Error Recovery & Resilience

**Duration**: Weeks 13-14 (14 days)
**Goal**: Checkpoint System → Failure Detection → Recovery Logic → Human Escalation
**Dependencies**: Phase 6 complete (Real-time & Audit Trail)

---

## Epic 1: Checkpoint Manager Implementation

### Task 1.1: Checkpoint Storage System

**Type**: Backend
**Dependencies**: Checkpoints table exists from Phase 1

**Subtasks**:

- [ ] 1.1.1: Review and enhance checkpoints table

```sql
-- migrations/XXXXXX_enhance_checkpoints.sql
ALTER TABLE checkpoints
ADD COLUMN checkpoint_type VARCHAR(50) DEFAULT 'automatic',
ADD COLUMN retry_count INT DEFAULT 0,
ADD COLUMN is_resumable BOOLEAN DEFAULT true,
ADD COLUMN context_hash TEXT;

CREATE INDEX idx_checkpoints_resumable ON checkpoints(task_id, is_resumable)
  WHERE is_resumable = true;

CREATE INDEX idx_checkpoints_type ON checkpoints(checkpoint_type);
```

- [ ] 1.1.2: Create CheckpointManager domain model

```rust
// apps/api/src/domain/recovery/checkpoint.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointType {
    Automatic,
    Manual,
    PreToolExecution,
    PostToolExecution,
    LlmCallComplete,
    ErrorRecovery,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub task_id: Uuid,
    pub step_number: i32,
    #[sqlx(json)]
    pub step_output: serde_json::Value,
    #[sqlx(json)]
    pub accumulated_context: serde_json::Value,
    pub token_count: Option<i32>,
    pub cost_estimate: Option<Decimal>,
    pub checkpoint_type: String,
    pub retry_count: i32,
    pub is_resumable: bool,
    pub context_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Checkpoint {
    pub fn new(
        task_id: Uuid,
        step_number: i32,
        step_output: serde_json::Value,
        accumulated_context: serde_json::Value,
    ) -> Self {
        let context_hash = Self::compute_hash(&accumulated_context);

        Self {
            id: Uuid::new_v4(),
            task_id,
            step_number,
            step_output,
            accumulated_context,
            token_count: None,
            cost_estimate: None,
            checkpoint_type: CheckpointType::Automatic.to_string(),
            retry_count: 0,
            is_resumable: true,
            context_hash: Some(context_hash),
            created_at: Utc::now(),
        }
    }

    pub fn with_type(mut self, checkpoint_type: CheckpointType) -> Self {
        self.checkpoint_type = checkpoint_type.to_string();
        self
    }

    pub fn with_token_count(mut self, count: i32) -> Self {
        self.token_count = Some(count);
        self
    }

    pub fn with_cost(mut self, cost: Decimal) -> Self {
        self.cost_estimate = Some(cost);
        self
    }

    pub fn mark_not_resumable(mut self) -> Self {
        self.is_resumable = false;
        self
    }

    fn compute_hash(context: &serde_json::Value) -> String {
        use sha2::{Sha256, Digest};
        let json_str = serde_json::to_string(context).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
```

- [ ] 1.1.3: Implement CheckpointManager service

```rust
// apps/api/src/services/checkpoint_manager.rs
use crate::domain::recovery::checkpoint::{Checkpoint, CheckpointType};
use crate::infrastructure::database::repositories::checkpoints::CheckpointRepository;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CheckpointManager {
    repository: CheckpointRepository,
}

impl CheckpointManager {
    pub fn new(pool: PgPool) -> Self {
        Self {
            repository: CheckpointRepository::new(pool),
        }
    }

    pub async fn create_checkpoint(
        &self,
        task_id: Uuid,
        step_number: i32,
        step_output: serde_json::Value,
        accumulated_context: serde_json::Value,
        checkpoint_type: CheckpointType,
    ) -> Result<Checkpoint, CheckpointError> {
        let checkpoint = Checkpoint::new(
            task_id,
            step_number,
            step_output,
            accumulated_context,
        )
        .with_type(checkpoint_type);

        self.repository.create(&checkpoint).await?;

        tracing::info!(
            "Checkpoint created: task_id={}, step={}, type={:?}",
            task_id,
            step_number,
            checkpoint_type
        );

        Ok(checkpoint)
    }

    pub async fn get_latest_checkpoint(
        &self,
        task_id: Uuid,
    ) -> Result<Option<Checkpoint>, CheckpointError> {
        self.repository
            .find_latest_resumable(task_id)
            .await
            .map_err(CheckpointError::DatabaseError)
    }

    pub async fn get_checkpoint_at_step(
        &self,
        task_id: Uuid,
        step_number: i32,
    ) -> Result<Option<Checkpoint>, CheckpointError> {
        self.repository
            .find_by_task_and_step(task_id, step_number)
            .await
            .map_err(CheckpointError::DatabaseError)
    }

    pub async fn get_all_checkpoints(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Checkpoint>, CheckpointError> {
        self.repository
            .find_all_by_task(task_id)
            .await
            .map_err(CheckpointError::DatabaseError)
    }

    pub async fn mark_checkpoint_used(
        &self,
        checkpoint_id: Uuid,
    ) -> Result<(), CheckpointError> {
        self.repository
            .mark_not_resumable(checkpoint_id)
            .await
            .map_err(CheckpointError::DatabaseError)
    }

    pub async fn cleanup_old_checkpoints(
        &self,
        task_id: Uuid,
        keep_latest: i32,
    ) -> Result<i32, CheckpointError> {
        let deleted = self.repository
            .delete_old_checkpoints(task_id, keep_latest)
            .await?;

        tracing::info!(
            "Cleaned up {} old checkpoints for task {}",
            deleted,
            task_id
        );

        Ok(deleted)
    }

    pub async fn compute_total_cost(
        &self,
        task_id: Uuid,
    ) -> Result<rust_decimal::Decimal, CheckpointError> {
        self.repository
            .sum_costs(task_id)
            .await
            .map_err(CheckpointError::DatabaseError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("No resumable checkpoint found")]
    NoCheckpointFound,

    #[error("Invalid checkpoint state")]
    InvalidState,
}
```

- [ ] 1.1.4: Implement Checkpoint Repository

```rust
// apps/api/src/infrastructure/database/repositories/checkpoints.rs
use crate::domain::recovery::checkpoint::Checkpoint;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct CheckpointRepository {
    pool: PgPool,
}

impl CheckpointRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, checkpoint: &Checkpoint) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO checkpoints (
                id, task_id, step_number, step_output, accumulated_context,
                token_count, cost_estimate, checkpoint_type, retry_count,
                is_resumable, context_hash, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            checkpoint.id,
            checkpoint.task_id,
            checkpoint.step_number,
            checkpoint.step_output,
            checkpoint.accumulated_context,
            checkpoint.token_count,
            checkpoint.cost_estimate,
            checkpoint.checkpoint_type,
            checkpoint.retry_count,
            checkpoint.is_resumable,
            checkpoint.context_hash,
            checkpoint.created_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_latest_resumable(
        &self,
        task_id: Uuid,
    ) -> Result<Option<Checkpoint>, sqlx::Error> {
        sqlx::query_as!(
            Checkpoint,
            r#"
            SELECT
                id, task_id, step_number, step_output as "step_output: _",
                accumulated_context as "accumulated_context: _",
                token_count, cost_estimate, checkpoint_type, retry_count,
                is_resumable, context_hash, created_at
            FROM checkpoints
            WHERE task_id = $1 AND is_resumable = true
            ORDER BY step_number DESC
            LIMIT 1
            "#,
            task_id
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_by_task_and_step(
        &self,
        task_id: Uuid,
        step_number: i32,
    ) -> Result<Option<Checkpoint>, sqlx::Error> {
        sqlx::query_as!(
            Checkpoint,
            r#"
            SELECT
                id, task_id, step_number, step_output as "step_output: _",
                accumulated_context as "accumulated_context: _",
                token_count, cost_estimate, checkpoint_type, retry_count,
                is_resumable, context_hash, created_at
            FROM checkpoints
            WHERE task_id = $1 AND step_number = $2
            "#,
            task_id,
            step_number
        )
        .fetch_optional(&self.pool)
        .await
    }

    pub async fn find_all_by_task(
        &self,
        task_id: Uuid,
    ) -> Result<Vec<Checkpoint>, sqlx::Error> {
        sqlx::query_as!(
            Checkpoint,
            r#"
            SELECT
                id, task_id, step_number, step_output as "step_output: _",
                accumulated_context as "accumulated_context: _",
                token_count, cost_estimate, checkpoint_type, retry_count,
                is_resumable, context_hash, created_at
            FROM checkpoints
            WHERE task_id = $1
            ORDER BY step_number ASC
            "#,
            task_id
        )
        .fetch_all(&self.pool)
        .await
    }

    pub async fn mark_not_resumable(&self, checkpoint_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE checkpoints
            SET is_resumable = false
            WHERE id = $1
            "#,
            checkpoint_id
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete_old_checkpoints(
        &self,
        task_id: Uuid,
        keep_latest: i32,
    ) -> Result<i32, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            DELETE FROM checkpoints
            WHERE task_id = $1
            AND id NOT IN (
                SELECT id FROM checkpoints
                WHERE task_id = $1
                ORDER BY step_number DESC
                LIMIT $2
            )
            "#,
            task_id,
            keep_latest as i64
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as i32)
    }

    pub async fn sum_costs(&self, task_id: Uuid) -> Result<Decimal, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(cost_estimate), 0) as "total!"
            FROM checkpoints
            WHERE task_id = $1
            "#,
            task_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.total)
    }
}
```

- [ ] 1.1.5: Integrate checkpointing into WorkerAgent

```rust
// apps/api/src/agents/worker.rs
impl WorkerAgent {
    pub async fn execute_task_with_checkpoints(
        &self,
        task: &Task,
        checkpoint_manager: &CheckpointManager,
    ) -> Result<TaskOutput, AgentError> {
        let mut step_number = 0;
        let mut accumulated_context = serde_json::json!({
            "task_id": task.id,
            "steps": []
        });

        // Check for existing checkpoint
        if let Some(checkpoint) = checkpoint_manager.get_latest_checkpoint(task.id).await? {
            tracing::info!(
                "Resuming from checkpoint: step {}",
                checkpoint.step_number
            );
            step_number = checkpoint.step_number + 1;
            accumulated_context = checkpoint.accumulated_context;
        }

        loop {
            let step_result = self.execute_step(task, &accumulated_context).await?;

            // Create checkpoint after each step
            accumulated_context["steps"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "step": step_number,
                    "result": step_result
                }));

            checkpoint_manager
                .create_checkpoint(
                    task.id,
                    step_number,
                    serde_json::to_value(&step_result)?,
                    accumulated_context.clone(),
                    CheckpointType::Automatic,
                )
                .await?;

            if step_result.is_complete {
                break;
            }

            step_number += 1;
        }

        Ok(TaskOutput {
            result: accumulated_context,
        })
    }
}
```

**Acceptance Criteria**:

- [ ] Checkpoints created after each task step
- [ ] Can retrieve latest checkpoint for task
- [ ] Can retrieve specific checkpoint by step number
- [ ] Context hash computed correctly
- [ ] Old checkpoints cleaned up automatically
- [ ] Cost tracking accurate

---

## Epic 2: Failure Detection & Classification

### Task 2.1: Failure Detection System

**Type**: Backend
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 2.1.1: Define failure types

```rust
// apps/api/src/domain/recovery/failure.rs
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FailureType {
    // Transient failures (retryable)
    NetworkTimeout,
    RateLimitExceeded,
    ApiUnavailable,
    TemporaryServiceError,

    // Task-level failures
    ValidationError,
    ToolExecutionFailed,
    InvalidInput,
    ResourceNotFound,

    // LLM failures
    LlmTimeout,
    LlmInvalidResponse,
    LlmContextLengthExceeded,
    LlmContentPolicyViolation,

    // System failures
    DatabaseConnectionLost,
    OutOfMemory,
    DiskSpaceExhausted,

    // Business logic failures
    BudgetExceeded,
    MaxRevisionsReached,
    DeadlineExceeded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureSeverity {
    Low,      // Can continue with degraded functionality
    Medium,   // Requires retry or alternative approach
    High,     // Task should pause, human intervention recommended
    Critical, // System-wide issue, escalate immediately
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    pub failure_type: FailureType,
    pub severity: FailureSeverity,
    pub message: String,
    pub context: serde_json::Value,
    pub retry_count: i32,
    pub occurred_at: chrono::DateTime<chrono::Utc>,
}

impl Failure {
    pub fn new(
        failure_type: FailureType,
        message: String,
    ) -> Self {
        let severity = Self::determine_severity(&failure_type);

        Self {
            failure_type,
            severity,
            message,
            context: serde_json::json!({}),
            retry_count: 0,
            occurred_at: chrono::Utc::now(),
        }
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }

    pub fn increment_retry(mut self) -> Self {
        self.retry_count += 1;
        self
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self.failure_type,
            FailureType::NetworkTimeout
                | FailureType::RateLimitExceeded
                | FailureType::ApiUnavailable
                | FailureType::TemporaryServiceError
                | FailureType::LlmTimeout
        )
    }

    pub fn should_escalate(&self) -> bool {
        matches!(self.severity, FailureSeverity::High | FailureSeverity::Critical)
            || self.retry_count > 3
    }

    fn determine_severity(failure_type: &FailureType) -> FailureSeverity {
        match failure_type {
            FailureType::NetworkTimeout
            | FailureType::RateLimitExceeded
            | FailureType::ApiUnavailable => FailureSeverity::Low,

            FailureType::ValidationError
            | FailureType::ToolExecutionFailed
            | FailureType::InvalidInput
            | FailureType::LlmTimeout => FailureSeverity::Medium,

            FailureType::LlmContextLengthExceeded
            | FailureType::BudgetExceeded
            | FailureType::MaxRevisionsReached => FailureSeverity::High,

            FailureType::DatabaseConnectionLost
            | FailureType::OutOfMemory
            | FailureType::LlmContentPolicyViolation => FailureSeverity::Critical,

            _ => FailureSeverity::Medium,
        }
    }
}
```

- [ ] 2.1.2: Implement retry strategies

```rust
// apps/api/src/services/retry_strategy.rs
use crate::domain::recovery::failure::{Failure, FailureType};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub enum RetryStrategy {
    Immediate,
    FixedDelay { delay: Duration },
    ExponentialBackoff { base_delay: Duration, max_delay: Duration },
    RateLimitBackoff { retry_after: Duration },
    NoRetry,
}

impl RetryStrategy {
    pub fn for_failure(failure: &Failure) -> Self {
        match failure.failure_type {
            FailureType::NetworkTimeout => Self::ExponentialBackoff {
                base_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(30),
            },

            FailureType::RateLimitExceeded => Self::RateLimitBackoff {
                retry_after: Duration::from_secs(60),
            },

            FailureType::ApiUnavailable => Self::ExponentialBackoff {
                base_delay: Duration::from_secs(5),
                max_delay: Duration::from_secs(120),
            },

            FailureType::TemporaryServiceError => Self::FixedDelay {
                delay: Duration::from_secs(3),
            },

            FailureType::LlmTimeout => Self::ExponentialBackoff {
                base_delay: Duration::from_secs(2),
                max_delay: Duration::from_secs(60),
            },

            // Non-retryable failures
            FailureType::ValidationError
            | FailureType::InvalidInput
            | FailureType::BudgetExceeded
            | FailureType::MaxRevisionsReached
            | FailureType::LlmContentPolicyViolation => Self::NoRetry,

            _ => Self::FixedDelay {
                delay: Duration::from_secs(5),
            },
        }
    }

    pub async fn wait(&self, retry_count: i32) {
        let delay = self.compute_delay(retry_count);
        if delay > Duration::ZERO {
            tracing::info!("Waiting {:?} before retry attempt {}", delay, retry_count + 1);
            sleep(delay).await;
        }
    }

    fn compute_delay(&self, retry_count: i32) -> Duration {
        match self {
            Self::Immediate => Duration::ZERO,

            Self::FixedDelay { delay } => *delay,

            Self::ExponentialBackoff { base_delay, max_delay } => {
                let multiplier = 2_u32.pow(retry_count as u32);
                let delay = *base_delay * multiplier;
                delay.min(*max_delay)
            }

            Self::RateLimitBackoff { retry_after } => *retry_after,

            Self::NoRetry => Duration::ZERO,
        }
    }

    pub fn should_retry(&self) -> bool {
        !matches!(self, Self::NoRetry)
    }
}
```

- [ ] 2.1.3: Create FailureHandler service

```rust
// apps/api/src/services/failure_handler.rs
use crate::domain::recovery::failure::{Failure, FailureType, FailureSeverity};
use crate::services::checkpoint_manager::CheckpointManager;
use crate::services::retry_strategy::RetryStrategy;
use uuid::Uuid;

pub struct FailureHandler {
    checkpoint_manager: CheckpointManager,
    max_retry_attempts: i32,
}

impl FailureHandler {
    pub fn new(checkpoint_manager: CheckpointManager) -> Self {
        Self {
            checkpoint_manager,
            max_retry_attempts: 3,
        }
    }

    pub async fn handle_failure(
        &self,
        task_id: Uuid,
        failure: Failure,
    ) -> Result<RecoveryAction, HandlerError> {
        tracing::error!(
            "Handling failure for task {}: {:?} - {}",
            task_id,
            failure.failure_type,
            failure.message
        );

        // Log failure to audit trail
        self.log_failure(task_id, &failure).await?;

        // Determine if we should retry
        if !failure.is_retryable() {
            return Ok(RecoveryAction::Escalate {
                reason: "Non-retryable failure".to_string(),
            });
        }

        if failure.retry_count >= self.max_retry_attempts {
            return Ok(RecoveryAction::Escalate {
                reason: format!("Max retries ({}) exceeded", self.max_retry_attempts),
            });
        }

        // Get retry strategy
        let strategy = RetryStrategy::for_failure(&failure);

        if !strategy.should_retry() {
            return Ok(RecoveryAction::Fail {
                reason: failure.message,
            });
        }

        // Get latest checkpoint for resumption
        let checkpoint = self
            .checkpoint_manager
            .get_latest_checkpoint(task_id)
            .await?;

        Ok(RecoveryAction::Retry {
            strategy,
            from_checkpoint: checkpoint.map(|c| c.step_number),
            retry_count: failure.retry_count + 1,
        })
    }

    async fn log_failure(&self, task_id: Uuid, failure: &Failure) -> Result<(), HandlerError> {
        // TODO: Log to audit_events table
        tracing::warn!(
            "Failure logged: task_id={}, type={:?}, severity={:?}",
            task_id,
            failure.failure_type,
            failure.severity
        );
        Ok(())
    }
}

#[derive(Debug)]
pub enum RecoveryAction {
    Retry {
        strategy: RetryStrategy,
        from_checkpoint: Option<i32>,
        retry_count: i32,
    },
    Fail {
        reason: String,
    },
    Escalate {
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Checkpoint error: {0}")]
    CheckpointError(#[from] crate::services::checkpoint_manager::CheckpointError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

**Acceptance Criteria**:

- [ ] Can detect all failure types
- [ ] Severity correctly assigned
- [ ] Retry strategies appropriate for failure type
- [ ] Exponential backoff working
- [ ] Rate limit backoff respects Retry-After header
- [ ] Non-retryable failures identified correctly

---

## Epic 3: Task Resumption Logic

### Task 3.1: Implement Resumption System

**Type**: Backend
**Dependencies**: Tasks 1.1 and 2.1 complete

**Subtasks**:

- [ ] 3.1.1: Create TaskResumer service

```rust
// apps/api/src/services/task_resumer.rs
use crate::domain::recovery::checkpoint::Checkpoint;
use crate::domain::tasks::Task;
use crate::services::checkpoint_manager::CheckpointManager;
use crate::services::failure_handler::{FailureHandler, RecoveryAction};
use uuid::Uuid;

pub struct TaskResumer {
    checkpoint_manager: CheckpointManager,
    failure_handler: FailureHandler,
}

impl TaskResumer {
    pub fn new(
        checkpoint_manager: CheckpointManager,
        failure_handler: FailureHandler,
    ) -> Self {
        Self {
            checkpoint_manager,
            failure_handler,
        }
    }

    pub async fn resume_task(
        &self,
        task: &Task,
        recovery_action: RecoveryAction,
    ) -> Result<ResumptionPlan, ResumerError> {
        match recovery_action {
            RecoveryAction::Retry {
                strategy,
                from_checkpoint,
                retry_count,
            } => {
                // Wait according to retry strategy
                strategy.wait(retry_count - 1).await;

                let checkpoint = if let Some(step) = from_checkpoint {
                    self.checkpoint_manager
                        .get_checkpoint_at_step(task.id, step)
                        .await?
                } else {
                    None
                };

                Ok(ResumptionPlan::Resume {
                    checkpoint,
                    retry_count,
                })
            }

            RecoveryAction::Fail { reason } => {
                Ok(ResumptionPlan::MarkFailed { reason })
            }

            RecoveryAction::Escalate { reason } => {
                Ok(ResumptionPlan::EscalateToHuman { reason })
            }
        }
    }

    pub async fn execute_resumption_plan(
        &self,
        task: &Task,
        plan: ResumptionPlan,
        worker_agent: &WorkerAgent,
    ) -> Result<TaskOutput, ResumerError> {
        match plan {
            ResumptionPlan::Resume {
                checkpoint,
                retry_count,
            } => {
                tracing::info!(
                    "Resuming task {} (retry {})",
                    task.id,
                    retry_count
                );

                if let Some(cp) = checkpoint {
                    worker_agent
                        .resume_from_checkpoint(task, &cp)
                        .await
                        .map_err(ResumerError::ExecutionError)
                } else {
                    worker_agent
                        .execute_task(task)
                        .await
                        .map_err(ResumerError::ExecutionError)
                }
            }

            ResumptionPlan::MarkFailed { reason } => {
                tracing::error!("Task {} marked as failed: {}", task.id, reason);
                Err(ResumerError::TaskFailed(reason))
            }

            ResumptionPlan::EscalateToHuman { reason } => {
                tracing::warn!("Task {} escalated to human: {}", task.id, reason);
                Err(ResumerError::EscalationRequired(reason))
            }
        }
    }
}

#[derive(Debug)]
pub enum ResumptionPlan {
    Resume {
        checkpoint: Option<Checkpoint>,
        retry_count: i32,
    },
    MarkFailed {
        reason: String,
    },
    EscalateToHuman {
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ResumerError {
    #[error("Checkpoint error: {0}")]
    CheckpointError(#[from] crate::services::checkpoint_manager::CheckpointError),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("Task failed: {0}")]
    TaskFailed(String),

    #[error("Escalation required: {0}")]
    EscalationRequired(String),
}
```

- [ ] 3.1.2: Add resume capability to WorkerAgent

```rust
// apps/api/src/agents/worker.rs
impl WorkerAgent {
    pub async fn resume_from_checkpoint(
        &self,
        task: &Task,
        checkpoint: &Checkpoint,
    ) -> Result<TaskOutput, String> {
        tracing::info!(
            "Resuming task {} from step {}",
            task.id,
            checkpoint.step_number
        );

        let accumulated_context = checkpoint.accumulated_context.clone();
        let start_step = checkpoint.step_number + 1;

        // Continue execution from the checkpoint
        self.execute_from_step(task, accumulated_context, start_step)
            .await
    }

    async fn execute_from_step(
        &self,
        task: &Task,
        mut accumulated_context: serde_json::Value,
        start_step: i32,
    ) -> Result<TaskOutput, String> {
        let mut step_number = start_step;

        loop {
            let step_result = self.execute_step(task, &accumulated_context).await?;

            accumulated_context["steps"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({
                    "step": step_number,
                    "result": step_result
                }));

            // Create checkpoint
            self.checkpoint_manager
                .create_checkpoint(
                    task.id,
                    step_number,
                    serde_json::to_value(&step_result).unwrap(),
                    accumulated_context.clone(),
                    CheckpointType::Automatic,
                )
                .await
                .ok();

            if step_result.is_complete {
                break;
            }

            step_number += 1;
        }

        Ok(TaskOutput {
            result: accumulated_context,
        })
    }
}
```

- [ ] 3.1.3: Create automatic retry wrapper

```rust
// apps/api/src/services/resilient_executor.rs
use crate::domain::recovery::failure::Failure;
use crate::domain::tasks::Task;
use crate::services::failure_handler::FailureHandler;
use crate::services::task_resumer::TaskResumer;

pub struct ResilientExecutor {
    failure_handler: FailureHandler,
    task_resumer: TaskResumer,
}

impl ResilientExecutor {
    pub fn new(failure_handler: FailureHandler, task_resumer: TaskResumer) -> Self {
        Self {
            failure_handler,
            task_resumer,
        }
    }

    pub async fn execute_with_recovery<F, T, E>(
        &self,
        task: &Task,
        operation: F,
    ) -> Result<T, E>
    where
        F: Fn(&Task) -> futures::future::BoxFuture<'_, Result<T, E>>,
        E: Into<Failure>,
    {
        let mut retry_count = 0;
        let max_retries = 3;

        loop {
            match operation(task).await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    let failure = error.into().increment_retry();

                    let recovery_action = self
                        .failure_handler
                        .handle_failure(task.id, failure)
                        .await
                        .map_err(|_| /* convert error */)?;

                    let plan = self
                        .task_resumer
                        .resume_task(task, recovery_action)
                        .await
                        .map_err(|_| /* convert error */)?;

                    match plan {
                        ResumptionPlan::Resume { .. } => {
                            retry_count += 1;
                            if retry_count >= max_retries {
                                return Err(/* max retries error */);
                            }
                            continue;
                        }
                        ResumptionPlan::MarkFailed { reason } => {
                            return Err(/* failed error */);
                        }
                        ResumptionPlan::EscalateToHuman { reason } => {
                            return Err(/* escalation error */);
                        }
                    }
                }
            }
        }
    }
}
```

**Acceptance Criteria**:

- [ ] Can resume from latest checkpoint
- [ ] Can resume from specific checkpoint
- [ ] Retry strategy applied correctly
- [ ] Context preserved across retries
- [ ] Failed tasks marked correctly
- [ ] Escalation triggered when appropriate

---

## Epic 4: Circuit Breaker Pattern

### Task 4.1: Implement Circuit Breaker

**Type**: Backend
**Dependencies**: Task 2.1 complete

**Subtasks**:

- [ ] 4.1.1: Create CircuitBreaker implementation

```rust
// apps/api/src/infrastructure/resilience/circuit_breaker.rs
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failures exceeded threshold, blocking requests
    HalfOpen,    // Testing if service recovered
}

pub struct CircuitBreaker {
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: u32,
    success_threshold: u32,
    timeout: Duration,
}

struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        success_threshold: u32,
        timeout: Duration,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
            })),
            failure_threshold,
            success_threshold,
            timeout,
        }
    }

    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> futures::future::BoxFuture<'static, Result<T, E>>,
    {
        // Check if circuit is open
        {
            let state = self.state.read().await;
            match state.state {
                CircuitState::Open => {
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() < self.timeout {
                            return Err(CircuitBreakerError::CircuitOpen);
                        }
                    }
                }
                _ => {}
            }
        }

        // Transition to half-open if timeout expired
        {
            let mut state = self.state.write().await;
            if state.state == CircuitState::Open {
                state.state = CircuitState::HalfOpen;
                state.success_count = 0;
                tracing::info!("Circuit breaker transitioning to HalfOpen");
            }
        }

        // Execute operation
        match operation().await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.write().await;

        match state.state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.success_threshold {
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    tracing::info!("Circuit breaker closed after successful recovery");
                }
            }
            CircuitState::Closed => {
                state.failure_count = 0;
            }
            _ => {}
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.write().await;

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed | CircuitState::HalfOpen => {
                if state.failure_count >= self.failure_threshold {
                    state.state = CircuitState::Open;
                    state.success_count = 0;
                    tracing::error!(
                        "Circuit breaker opened after {} failures",
                        state.failure_count
                    );
                }
            }
            _ => {}
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.read().await.state.clone()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Operation failed: {0}")]
    OperationFailed(E),
}
```

- [ ] 4.1.2: Apply circuit breaker to LLM calls

```rust
// apps/api/src/infrastructure/llm/resilient_client.rs
use crate::infrastructure::resilience::circuit_breaker::CircuitBreaker;
use std::sync::Arc;
use std::time::Duration;

pub struct ResilientLlmClient {
    client: ClaudeClient,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl ResilientLlmClient {
    pub fn new(client: ClaudeClient) -> Self {
        let circuit_breaker = Arc::new(CircuitBreaker::new(
            5,                            // Open after 5 failures
            3,                            // Close after 3 successes
            Duration::from_secs(60),      // Wait 60s before retry
        ));

        Self {
            client,
            circuit_breaker,
        }
    }

    pub async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, LlmError> {
        let client = self.client.clone();
        let system = system_prompt.to_string();
        let user = user_prompt.to_string();

        self.circuit_breaker
            .call(|| {
                Box::pin(async move {
                    client.complete(&system, &user).await
                })
            })
            .await
            .map_err(|e| match e {
                CircuitBreakerError::CircuitOpen => {
                    LlmError::ServiceUnavailable("Circuit breaker open".to_string())
                }
                CircuitBreakerError::OperationFailed(err) => err,
            })
    }
}
```

**Acceptance Criteria**:

- [ ] Circuit opens after threshold failures
- [ ] Circuit stays open for timeout duration
- [ ] Circuit transitions to half-open for testing
- [ ] Circuit closes after successful recoveries
- [ ] LLM calls protected by circuit breaker
- [ ] Circuit state logged correctly

---

## Epic 5: Human Escalation System

### Task 5.1: Build Escalation Workflow

**Type**: Fullstack
**Dependencies**: All previous tasks complete

**Subtasks**:

- [ ] 5.1.1: Create escalations table

```sql
-- migrations/XXXXXX_create_escalations.sql
CREATE TYPE escalation_status AS ENUM (
    'pending',
    'acknowledged',
    'in_progress',
    'resolved',
    'cancelled'
);

CREATE TABLE escalations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    severity VARCHAR(20) NOT NULL,
    reason TEXT NOT NULL,
    context JSONB NOT NULL DEFAULT '{}'::jsonb,
    status escalation_status NOT NULL DEFAULT 'pending',
    assigned_to UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_escalations_team_id ON escalations(team_id);
CREATE INDEX idx_escalations_status ON escalations(status);
CREATE INDEX idx_escalations_assigned_to ON escalations(assigned_to);
CREATE INDEX idx_escalations_created_at ON escalations(created_at DESC);
```

- [ ] 5.1.2: Create Escalation domain model

```rust
// apps/api/src/domain/recovery/escalation.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EscalationStatus {
    Pending,
    Acknowledged,
    InProgress,
    Resolved,
    Cancelled,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Escalation {
    pub id: Uuid,
    pub team_id: Uuid,
    pub task_id: Option<Uuid>,
    pub severity: String,
    pub reason: String,
    #[sqlx(json)]
    pub context: serde_json::Value,
    #[sqlx(try_from = "String")]
    pub status: EscalationStatus,
    pub assigned_to: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl Escalation {
    pub fn new(
        team_id: Uuid,
        task_id: Option<Uuid>,
        severity: String,
        reason: String,
        context: serde_json::Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            task_id,
            severity,
            reason,
            context,
            status: EscalationStatus::Pending,
            assigned_to: None,
            acknowledged_at: None,
            resolved_at: None,
            resolution_notes: None,
            created_at: Utc::now(),
        }
    }

    pub fn acknowledge(&mut self, user_id: Uuid) {
        self.status = EscalationStatus::Acknowledged;
        self.assigned_to = Some(user_id);
        self.acknowledged_at = Some(Utc::now());
    }

    pub fn resolve(&mut self, notes: String) {
        self.status = EscalationStatus::Resolved;
        self.resolution_notes = Some(notes);
        self.resolved_at = Some(Utc::now());
    }
}
```

- [ ] 5.1.3: Create escalation API endpoints

```rust
// apps/api/src/api/handlers/escalations.rs
use crate::api::auth::jwt::Claims;
use crate::services::escalation_service::EscalationService;
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AcknowledgeRequest {
    user_id: Uuid,
}

#[derive(Deserialize)]
pub struct ResolveRequest {
    resolution_notes: String,
    should_retry: bool,
}

pub async fn get_pending_escalations(
    Extension(claims): Extension<Claims>,
    State(service): State<EscalationService>,
) -> Result<Json<Vec<Escalation>>, StatusCode> {
    let company_id = Uuid::parse_str(&claims.company_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let escalations = service
        .get_pending_for_company(company_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(escalations))
}

pub async fn acknowledge_escalation(
    Path(escalation_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<AcknowledgeRequest>,
    State(service): State<EscalationService>,
) -> Result<Json<Escalation>, StatusCode> {
    let escalation = service
        .acknowledge(escalation_id, req.user_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(escalation))
}

pub async fn resolve_escalation(
    Path(escalation_id): Path<Uuid>,
    Json(req): Json<ResolveRequest>,
    State(service): State<EscalationService>,
) -> Result<Json<Escalation>, StatusCode> {
    let escalation = service
        .resolve(escalation_id, req.resolution_notes, req.should_retry)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(escalation))
}
```

- [ ] 5.1.4: Create escalation UI component

```typescript
// apps/frontend/src/components/escalations/EscalationCard.tsx
'use client';

import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { useState } from 'react';
import { AlertTriangle, CheckCircle } from 'lucide-react';
import { format } from 'date-fns';

interface Escalation {
  id: string;
  team_id: string;
  task_id?: string;
  severity: string;
  reason: string;
  context: Record<string, any>;
  status: string;
  created_at: string;
}

export function EscalationCard({ escalation, onResolve }: {
  escalation: Escalation;
  onResolve: (id: string, notes: string, shouldRetry: boolean) => void;
}) {
  const [notes, setNotes] = useState('');
  const [isResolving, setIsResolving] = useState(false);

  const getSeverityColor = (severity: string) => {
    switch (severity) {
      case 'low': return 'bg-yellow-100 text-yellow-800';
      case 'medium': return 'bg-orange-100 text-orange-800';
      case 'high': return 'bg-red-100 text-red-800';
      case 'critical': return 'bg-red-600 text-white';
      default: return 'bg-gray-100 text-gray-800';
    }
  };

  const handleResolve = (shouldRetry: boolean) => {
    if (!notes.trim()) {
      alert('Please provide resolution notes');
      return;
    }
    onResolve(escalation.id, notes, shouldRetry);
  };

  return (
    <Card className="p-6 border-l-4 border-l-red-500">
      <div className="flex items-start justify-between mb-4">
        <div className="flex items-center gap-3">
          <AlertTriangle className="h-6 w-6 text-red-500" />
          <div>
            <h3 className="font-semibold text-lg">Escalation Required</h3>
            <p className="text-sm text-gray-500">
              {format(new Date(escalation.created_at), 'MMM d, yyyy HH:mm')}
            </p>
          </div>
        </div>
        <Badge className={getSeverityColor(escalation.severity)}>
          {escalation.severity}
        </Badge>
      </div>

      <div className="mb-4">
        <p className="font-medium mb-2">Reason:</p>
        <p className="text-gray-700">{escalation.reason}</p>
      </div>

      {Object.keys(escalation.context).length > 0 && (
        <details className="mb-4">
          <summary className="cursor-pointer text-sm text-gray-600 hover:text-gray-900">
            View context
          </summary>
          <pre className="mt-2 p-3 bg-gray-50 rounded text-xs overflow-x-auto">
            {JSON.stringify(escalation.context, null, 2)}
          </pre>
        </details>
      )}

      {isResolving ? (
        <div className="space-y-3">
          <Textarea
            placeholder="Resolution notes..."
            value={notes}
            onChange={(e) => setNotes(e.target.value)}
            rows={4}
          />
          <div className="flex gap-2">
            <Button
              onClick={() => handleResolve(true)}
              className="flex-1"
            >
              Resolve & Retry
            </Button>
            <Button
              onClick={() => handleResolve(false)}
              variant="outline"
              className="flex-1"
            >
              Resolve & Cancel
            </Button>
            <Button
              onClick={() => setIsResolving(false)}
              variant="ghost"
            >
              Cancel
            </Button>
          </div>
        </div>
      ) : (
        <Button
          onClick={() => setIsResolving(true)}
          className="w-full"
        >
          Resolve Escalation
        </Button>
      )}
    </Card>
  );
}
```

- [ ] 5.1.5: Create escalations dashboard

```typescript
// apps/frontend/src/app/escalations/page.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { EscalationCard } from '@/components/escalations/EscalationCard';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';

export default function EscalationsPage() {
  const { data: escalations } = useQuery({
    queryKey: ['escalations'],
    queryFn: fetchEscalations,
    refetchInterval: 30000, // Poll every 30s
  });

  const pending = escalations?.filter(e => e.status === 'pending') || [];
  const acknowledged = escalations?.filter(e => e.status === 'acknowledged') || [];
  const resolved = escalations?.filter(e => e.status === 'resolved') || [];

  const handleResolve = async (id: string, notes: string, shouldRetry: boolean) => {
    await resolveEscalation(id, { resolution_notes: notes, should_retry: shouldRetry });
    queryClient.invalidateQueries({ queryKey: ['escalations'] });
  };

  return (
    <div className="container mx-auto py-8">
      <h1 className="text-3xl font-bold mb-8">Escalations</h1>

      <Tabs defaultValue="pending">
        <TabsList>
          <TabsTrigger value="pending">
            Pending ({pending.length})
          </TabsTrigger>
          <TabsTrigger value="acknowledged">
            Acknowledged ({acknowledged.length})
          </TabsTrigger>
          <TabsTrigger value="resolved">
            Resolved ({resolved.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="pending" className="space-y-4 mt-6">
          {pending.map(escalation => (
            <EscalationCard
              key={escalation.id}
              escalation={escalation}
              onResolve={handleResolve}
            />
          ))}
          {pending.length === 0 && (
            <p className="text-center text-gray-500 py-8">
              No pending escalations
            </p>
          )}
        </TabsContent>

        <TabsContent value="acknowledged" className="space-y-4 mt-6">
          {acknowledged.map(escalation => (
            <EscalationCard
              key={escalation.id}
              escalation={escalation}
              onResolve={handleResolve}
            />
          ))}
        </TabsContent>

        <TabsContent value="resolved" className="space-y-4 mt-6">
          {resolved.map(escalation => (
            <EscalationCard
              key={escalation.id}
              escalation={escalation}
              onResolve={handleResolve}
            />
          ))}
        </TabsContent>
      </Tabs>
    </div>
  );
}
```

**Acceptance Criteria**:

- [ ] Escalations created when failures exceed thresholds
- [ ] Escalations visible in dashboard
- [ ] Can acknowledge escalations
- [ ] Can resolve with notes
- [ ] Can choose to retry or cancel task
- [ ] Email/Slack notifications sent (optional)
- [ ] Escalation status tracked correctly

---

## Epic 6: Integration Testing

### Task 6.1: Test Error Recovery Flow

**Type**: Testing
**Dependencies**: All previous tasks complete

**Subtasks**:

- [ ] 6.1.1: Create recovery integration tests

```rust
// apps/api/tests/recovery_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_checkpoint_and_resume() {
        // Create task
        // Execute with checkpoints
        // Simulate failure
        // Resume from checkpoint
        // Verify context preserved
    }

    #[tokio::test]
    async fn test_retry_with_exponential_backoff() {
        // Simulate transient failure
        // Verify retry strategy applied
        // Verify exponential delays
        // Verify eventual success
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens() {
        // Trigger multiple failures
        // Verify circuit opens
        // Verify requests blocked
        // Wait for timeout
        // Verify circuit half-opens
        // Verify successful recovery
    }

    #[tokio::test]
    async fn test_escalation_on_max_retries() {
        // Simulate persistent failures
        // Exceed max retries
        // Verify escalation created
        // Verify task paused
    }
}
```

**Acceptance Criteria**:

- [ ] All recovery tests passing
- [ ] Checkpointing tested
- [ ] Retry logic tested
- [ ] Circuit breaker tested
- [ ] Escalation tested
- [ ] Edge cases covered

---

## Success Criteria - Phase 7 Complete

- [ ] Checkpoints created automatically
- [ ] Task resumption working from checkpoints
- [ ] Failure detection and classification accurate
- [ ] Retry strategies appropriate for failure types
- [ ] Circuit breaker protecting LLM calls
- [ ] Escalations created for unrecoverable failures
- [ ] Escalation dashboard functional
- [ ] All integration tests passing
- [ ] System resilient to transient failures
- [ ] No data loss on failures

---

## Next Steps

Proceed to [11-phase-8-testing-polish.md](./11-phase-8-testing-polish.md) for comprehensive testing and production readiness.

---

**Phase 7: Error recovery and resilience complete**
