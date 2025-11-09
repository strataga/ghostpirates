# Phase 3: Task Orchestration System

**Duration**: Weeks 5-6 (14 days)
**Goal**: TaskOrchestrator → Hierarchical Decomposition → Skill-Based Assignment → Queue Management
**Dependencies**: Phase 2 complete (Manager and Worker agents operational)

---

## Epic 1: TaskOrchestrator Implementation

### Task 1.1: Create TaskOrchestrator Core

**Type**: Backend
**Dependencies**: Agent system from Phase 2

**Subtasks**:

- [ ] 1.1.1: Create TaskOrchestrator struct

```rust
// apps/api/src/orchestration/task_orchestrator.rs
use uuid::Uuid;
use sqlx::PgPool;
use std::collections::HashMap;
use crate::domain::tasks::{Task, TaskStatus};
use crate::domain::teams::TeamMember;
use crate::infrastructure::redis::TaskQueue;

pub struct TaskOrchestrator {
    db: PgPool,
    task_queue: TaskQueue,
    assignment_scorer: AssignmentScorer,
}

impl TaskOrchestrator {
    pub fn new(db: PgPool, task_queue: TaskQueue) -> Self {
        Self {
            db,
            task_queue,
            assignment_scorer: AssignmentScorer::new(),
        }
    }

    pub async fn orchestrate_team(&self, team_id: Uuid) -> Result<(), OrchestrationError> {
        // Main orchestration loop
        loop {
            // 1. Check for pending tasks
            let pending_tasks = self.get_pending_tasks(team_id).await?;

            if pending_tasks.is_empty() {
                break;
            }

            // 2. Assign tasks to available workers
            for task in pending_tasks {
                self.assign_task_to_worker(&task).await?;
            }

            // 3. Brief delay before next iteration
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        Ok(())
    }

    async fn get_pending_tasks(&self, team_id: Uuid) -> Result<Vec<Task>, OrchestrationError> {
        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT
                id, team_id, parent_task_id, title, description,
                acceptance_criteria as "acceptance_criteria: _",
                assigned_to, assigned_by,
                status as "status: _",
                priority, start_time, completion_time,
                revision_count, max_revisions,
                input_data as "input_data: _",
                output_data as "output_data: _",
                error_message,
                required_skills as "required_skills: _",
                estimated_tokens, actual_tokens,
                created_at, updated_at
            FROM tasks
            WHERE team_id = $1
                AND status = 'pending'
                AND (parent_task_id IS NULL OR
                     parent_task_id IN (
                         SELECT id FROM tasks WHERE status = 'completed'
                     ))
            ORDER BY priority DESC, created_at ASC
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    async fn assign_task_to_worker(&self, task: &Task) -> Result<(), OrchestrationError> {
        // Get available workers
        let workers = self.get_available_workers(task.team_id).await?;

        if workers.is_empty() {
            // No workers available, task stays pending
            return Ok(());
        }

        // Score each worker for this task
        let best_worker = self.assignment_scorer.find_best_worker(task, &workers)?;

        // Assign task using database function
        sqlx::query!(
            "SELECT assign_task_to_best_worker($1, $2)",
            task.id,
            serde_json::to_value(&task.required_skills)?
        )
        .execute(&self.db)
        .await?;

        // Add to Redis queue for worker to pick up
        self.task_queue.enqueue(QueuedTask {
            task_id: task.id,
            team_id: task.team_id,
            priority: task.priority,
            created_at: chrono::Utc::now().timestamp(),
        }).await?;

        // Publish assignment event
        self.publish_task_assigned(task.id, best_worker.id).await?;

        Ok(())
    }

    async fn get_available_workers(&self, team_id: Uuid) -> Result<Vec<TeamMember>, OrchestrationError> {
        let workers = sqlx::query_as!(
            TeamMember,
            r#"
            SELECT
                id, team_id, agent_id,
                role as "role: _",
                specialization,
                status as "status: _",
                current_workload, max_concurrent_tasks,
                tasks_completed, tasks_failed,
                total_tokens_used, total_cost, joined_at, last_active_at
            FROM team_members
            WHERE team_id = $1
                AND role = 'worker'
                AND status IN ('idle', 'active')
                AND current_workload < max_concurrent_tasks
            ORDER BY current_workload ASC
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(workers)
    }

    async fn publish_task_assigned(&self, task_id: Uuid, worker_id: Uuid) -> Result<(), OrchestrationError> {
        // Implementation in Epic 5
        Ok(())
    }
}
```

- [ ] 1.1.2: Create error types for orchestration

```rust
// apps/api/src/orchestration/error.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("No available workers for task {0}")]
    NoAvailableWorkers(uuid::Uuid),

    #[error("Task dependency cycle detected")]
    DependencyCycle,

    #[error("Invalid task state: {0}")]
    InvalidTaskState(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type OrchestrationResult<T> = Result<T, OrchestrationError>;
```

- [ ] 1.1.3: Implement orchestration lifecycle management

```rust
// apps/api/src/orchestration/lifecycle.rs
use tokio::task::JoinHandle;
use std::collections::HashMap;
use uuid::Uuid;

pub struct OrchestrationManager {
    orchestrators: HashMap<Uuid, JoinHandle<()>>,
    db: PgPool,
    redis_url: String,
}

impl OrchestrationManager {
    pub fn new(db: PgPool, redis_url: String) -> Self {
        Self {
            orchestrators: HashMap::new(),
            db,
            redis_url,
        }
    }

    pub async fn start_orchestration(&mut self, team_id: Uuid) -> Result<(), OrchestrationError> {
        // Don't start if already running
        if self.orchestrators.contains_key(&team_id) {
            return Ok(());
        }

        let db = self.db.clone();
        let redis_url = self.redis_url.clone();

        let handle = tokio::spawn(async move {
            let task_queue = TaskQueue::new(&redis_url).expect("Redis connection failed");
            let orchestrator = TaskOrchestrator::new(db, task_queue);

            if let Err(e) = orchestrator.orchestrate_team(team_id).await {
                tracing::error!("Orchestration error for team {}: {}", team_id, e);
            }
        });

        self.orchestrators.insert(team_id, handle);
        Ok(())
    }

    pub async fn stop_orchestration(&mut self, team_id: Uuid) -> Result<(), OrchestrationError> {
        if let Some(handle) = self.orchestrators.remove(&team_id) {
            handle.abort();
        }
        Ok(())
    }

    pub fn is_running(&self, team_id: Uuid) -> bool {
        self.orchestrators.contains_key(&team_id)
    }

    pub async fn shutdown_all(&mut self) {
        for (team_id, handle) in self.orchestrators.drain() {
            tracing::info!("Shutting down orchestration for team {}", team_id);
            handle.abort();
        }
    }
}
```

- [ ] 1.1.4: Add orchestration to team startup

```rust
// apps/api/src/services/team_service.rs
use crate::orchestration::OrchestrationManager;

pub struct TeamService {
    db: PgPool,
    orchestration_manager: Arc<Mutex<OrchestrationManager>>,
}

impl TeamService {
    pub async fn start_team(&self, team_id: Uuid) -> Result<(), ServiceError> {
        // Update team status to active
        sqlx::query!(
            "UPDATE teams SET status = 'active', started_at = NOW() WHERE id = $1",
            team_id
        )
        .execute(&self.db)
        .await?;

        // Start orchestration
        let mut manager = self.orchestration_manager.lock().await;
        manager.start_orchestration(team_id).await?;

        Ok(())
    }

    pub async fn pause_team(&self, team_id: Uuid) -> Result<(), ServiceError> {
        // Update team status
        sqlx::query!(
            "UPDATE teams SET status = 'paused', paused_at = NOW() WHERE id = $1",
            team_id
        )
        .execute(&self.db)
        .await?;

        // Stop orchestration
        let mut manager = self.orchestration_manager.lock().await;
        manager.stop_orchestration(team_id).await?;

        Ok(())
    }
}
```

- [ ] 1.1.5: Test orchestration loop

```bash
cargo test orchestration::
```

**Acceptance Criteria**:

- [ ] TaskOrchestrator can be instantiated
- [ ] Orchestration loop runs continuously
- [ ] Pending tasks are detected
- [ ] Workers are retrieved correctly
- [ ] Orchestration can be started and stopped
- [ ] No memory leaks in long-running orchestration

---

## Epic 2: Hierarchical Task Decomposition

### Task 2.1: Implement Task Decomposition Service

**Type**: Backend
**Dependencies**: Manager agent from Phase 2

**Subtasks**:

- [ ] 2.1.1: Create task decomposition service

```rust
// apps/api/src/services/task_decomposition.rs
use crate::agents::manager::ManagerAgent;
use crate::domain::tasks::Task;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DecompositionRequest {
    pub team_id: Uuid,
    pub goal: String,
    pub max_depth: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskNode {
    pub title: String,
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub required_skills: Vec<String>,
    pub estimated_tokens: i32,
    pub priority: i32,
    pub subtasks: Vec<TaskNode>,
}

pub struct TaskDecompositionService {
    db: PgPool,
}

impl TaskDecompositionService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn decompose_goal(
        &self,
        request: DecompositionRequest,
    ) -> Result<Vec<Task>, DecompositionError> {
        // Get manager agent for team
        let manager = self.get_team_manager(request.team_id).await?;

        // Use manager to analyze and decompose goal
        let decomposition = manager.decompose_goal(&request.goal).await?;

        // Convert to task hierarchy
        let mut all_tasks = Vec::new();
        self.create_task_hierarchy(
            request.team_id,
            None,
            decomposition,
            0,
            request.max_depth,
            &mut all_tasks,
        ).await?;

        // Save all tasks to database in transaction
        let mut tx = self.db.begin().await?;

        for task in &all_tasks {
            self.save_task(&mut tx, task).await?;
        }

        tx.commit().await?;

        Ok(all_tasks)
    }

    async fn create_task_hierarchy(
        &self,
        team_id: Uuid,
        parent_id: Option<Uuid>,
        node: TaskNode,
        depth: usize,
        max_depth: usize,
        all_tasks: &mut Vec<Task>,
    ) -> Result<Uuid, DecompositionError> {
        // Create task from node
        let task = Task {
            id: Uuid::new_v4(),
            team_id,
            parent_task_id: parent_id,
            title: node.title,
            description: node.description,
            acceptance_criteria: node.acceptance_criteria,
            assigned_to: None,
            assigned_by: None,
            status: TaskStatus::Pending,
            priority: node.priority,
            start_time: None,
            completion_time: None,
            revision_count: 0,
            max_revisions: 3,
            input_data: None,
            output_data: None,
            error_message: None,
            required_skills: serde_json::to_value(&node.required_skills)?,
            estimated_tokens: Some(node.estimated_tokens),
            actual_tokens: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let task_id = task.id;
        all_tasks.push(task);

        // Recursively create subtasks if not at max depth
        if depth < max_depth && !node.subtasks.is_empty() {
            for subtask_node in node.subtasks {
                self.create_task_hierarchy(
                    team_id,
                    Some(task_id),
                    subtask_node,
                    depth + 1,
                    max_depth,
                    all_tasks,
                ).await?;
            }
        }

        Ok(task_id)
    }

    async fn save_task(&self, tx: &mut sqlx::Transaction<'_, sqlx::Postgres>, task: &Task) -> Result<(), DecompositionError> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, team_id, parent_task_id, title, description,
                acceptance_criteria, status, priority,
                revision_count, max_revisions,
                required_skills, estimated_tokens,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::task_status, $8, $9, $10, $11, $12, $13, $14)
            "#,
            task.id,
            task.team_id,
            task.parent_task_id,
            task.title,
            task.description,
            serde_json::to_value(&task.acceptance_criteria)?,
            "pending",
            task.priority,
            task.revision_count,
            task.max_revisions,
            task.required_skills,
            task.estimated_tokens,
            task.created_at,
            task.updated_at
        )
        .execute(tx)
        .await?;

        Ok(())
    }

    async fn get_team_manager(&self, team_id: Uuid) -> Result<ManagerAgent, DecompositionError> {
        let member = sqlx::query_as!(
            TeamMember,
            r#"
            SELECT
                id, team_id, agent_id,
                role as "role: _",
                specialization,
                status as "status: _",
                current_workload, max_concurrent_tasks,
                tasks_completed, tasks_failed,
                total_tokens_used, total_cost, joined_at, last_active_at
            FROM team_members
            WHERE team_id = $1 AND role = 'manager'
            LIMIT 1
            "#,
            team_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(ManagerAgent::from_member(member))
    }
}
```

- [ ] 2.1.2: Create dependency validation

```rust
// apps/api/src/services/task_dependency.rs
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct DependencyValidator {
    db: PgPool,
}

impl DependencyValidator {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn validate_no_cycles(&self, team_id: Uuid) -> Result<bool, sqlx::Error> {
        let tasks = self.get_team_tasks(team_id).await?;

        // Build adjacency list
        let mut graph: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for task in &tasks {
            if let Some(parent_id) = task.parent_task_id {
                graph.entry(parent_id).or_insert_with(Vec::new).push(task.id);
            }
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task in &tasks {
            if !visited.contains(&task.id) {
                if self.has_cycle(&task.id, &graph, &mut visited, &mut rec_stack) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn has_cycle(
        &self,
        node: &Uuid,
        graph: &HashMap<Uuid, Vec<Uuid>>,
        visited: &mut HashSet<Uuid>,
        rec_stack: &mut HashSet<Uuid>,
    ) -> bool {
        visited.insert(*node);
        rec_stack.insert(*node);

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.has_cycle(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    async fn get_team_tasks(&self, team_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as!(
            Task,
            r#"
            SELECT
                id, team_id, parent_task_id, title, description,
                acceptance_criteria as "acceptance_criteria: _",
                assigned_to, assigned_by,
                status as "status: _",
                priority, start_time, completion_time,
                revision_count, max_revisions,
                input_data as "input_data: _",
                output_data as "output_data: _",
                error_message,
                required_skills as "required_skills: _",
                estimated_tokens, actual_tokens,
                created_at, updated_at
            FROM tasks
            WHERE team_id = $1
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await
    }

    pub async fn get_task_depth(&self, task_id: Uuid) -> Result<usize, sqlx::Error> {
        let result = sqlx::query!(
            r#"
            WITH RECURSIVE task_hierarchy AS (
                SELECT id, parent_task_id, 0 as depth
                FROM tasks
                WHERE id = $1

                UNION ALL

                SELECT t.id, t.parent_task_id, th.depth + 1
                FROM tasks t
                INNER JOIN task_hierarchy th ON t.parent_task_id = th.id
            )
            SELECT MAX(depth) as max_depth FROM task_hierarchy
            "#,
            task_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(result.max_depth.unwrap_or(0) as usize)
    }
}
```

- [ ] 2.1.3: Create API endpoint for task decomposition

```rust
// apps/api/src/api/handlers/tasks.rs
use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::services::task_decomposition::DecompositionRequest;

#[derive(Deserialize)]
pub struct DecomposeGoalRequest {
    pub goal: String,
    pub max_depth: Option<usize>,
}

pub async fn decompose_team_goal(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
    Json(request): Json<DecomposeGoalRequest>,
) -> Result<Json<Vec<Task>>, StatusCode> {
    let decomposition_service = TaskDecompositionService::new(state.db.clone());

    let tasks = decomposition_service
        .decompose_goal(DecompositionRequest {
            team_id,
            goal: request.goal,
            max_depth: request.max_depth.unwrap_or(3),
        })
        .await
        .map_err(|e| {
            tracing::error!("Decomposition failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(tasks))
}

#[derive(Serialize)]
pub struct TaskHierarchyResponse {
    pub task: Task,
    pub subtasks: Vec<TaskHierarchyResponse>,
}

pub async fn get_task_hierarchy(
    State(state): State<AppState>,
    Path((team_id, task_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TaskHierarchyResponse>, StatusCode> {
    let hierarchy = build_task_tree(&state.db, task_id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(hierarchy))
}

async fn build_task_tree(db: &PgPool, task_id: Uuid) -> Result<TaskHierarchyResponse, sqlx::Error> {
    let task = sqlx::query_as!(
        Task,
        r#"
        SELECT
            id, team_id, parent_task_id, title, description,
            acceptance_criteria as "acceptance_criteria: _",
            assigned_to, assigned_by,
            status as "status: _",
            priority, start_time, completion_time,
            revision_count, max_revisions,
            input_data as "input_data: _",
            output_data as "output_data: _",
            error_message,
            required_skills as "required_skills: _",
            estimated_tokens, actual_tokens,
            created_at, updated_at
        FROM tasks
        WHERE id = $1
        "#,
        task_id
    )
    .fetch_one(db)
    .await?;

    let children = sqlx::query_as!(
        Task,
        r#"
        SELECT
            id, team_id, parent_task_id, title, description,
            acceptance_criteria as "acceptance_criteria: _",
            assigned_to, assigned_by,
            status as "status: _",
            priority, start_time, completion_time,
            revision_count, max_revisions,
            input_data as "input_data: _",
            output_data as "output_data: _",
            error_message,
            required_skills as "required_skills: _",
            estimated_tokens, actual_tokens,
            created_at, updated_at
        FROM tasks
        WHERE parent_task_id = $1
        ORDER BY priority DESC, created_at ASC
        "#,
        task_id
    )
    .fetch_all(db)
    .await?;

    let mut subtasks = Vec::new();
    for child in children {
        subtasks.push(build_task_tree(db, child.id).await?);
    }

    Ok(TaskHierarchyResponse { task, subtasks })
}
```

**Acceptance Criteria**:

- [ ] Can decompose goal into task hierarchy
- [ ] Hierarchy depth is configurable
- [ ] Parent-child relationships stored correctly
- [ ] No circular dependencies allowed
- [ ] Can retrieve full task tree
- [ ] Tasks saved in correct order

---

## Epic 3: Skill-Based Assignment Algorithm

### Task 3.1: Implement Assignment Scoring

**Type**: Backend
**Dependencies**: Team members with specializations

**Subtasks**:

- [ ] 3.1.1: Create skill matching system

```rust
// apps/api/src/orchestration/skill_matcher.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkillProfile {
    pub skills: HashMap<String, f32>, // skill_name -> proficiency (0.0-1.0)
}

impl SkillProfile {
    pub fn from_specialization(spec: &str) -> Self {
        let skills = match spec {
            "Technical Executor" => {
                let mut s = HashMap::new();
                s.insert("coding".to_string(), 0.9);
                s.insert("debugging".to_string(), 0.8);
                s.insert("testing".to_string(), 0.7);
                s.insert("deployment".to_string(), 0.6);
                s
            }
            "Content Creator" => {
                let mut s = HashMap::new();
                s.insert("writing".to_string(), 0.9);
                s.insert("editing".to_string(), 0.8);
                s.insert("research".to_string(), 0.7);
                s.insert("documentation".to_string(), 0.8);
                s
            }
            "Researcher/Analyzer" => {
                let mut s = HashMap::new();
                s.insert("research".to_string(), 0.9);
                s.insert("analysis".to_string(), 0.9);
                s.insert("data_processing".to_string(), 0.7);
                s.insert("summarization".to_string(), 0.8);
                s
            }
            _ => HashMap::new(),
        };

        Self { skills }
    }

    pub fn match_score(&self, required_skills: &[String]) -> f32 {
        if required_skills.is_empty() {
            return 0.5; // Neutral score if no requirements
        }

        let mut total_score = 0.0;
        let mut matched_count = 0;

        for skill in required_skills {
            if let Some(proficiency) = self.skills.get(skill) {
                total_score += proficiency;
                matched_count += 1;
            }
        }

        if matched_count == 0 {
            0.0
        } else {
            total_score / required_skills.len() as f32
        }
    }
}
```

- [ ] 3.1.2: Implement assignment scorer

```rust
// apps/api/src/orchestration/assignment_scorer.rs
use crate::domain::tasks::Task;
use crate::domain::teams::TeamMember;
use crate::orchestration::skill_matcher::SkillProfile;

pub struct AssignmentScorer {
    skill_weight: f32,
    workload_weight: f32,
    success_rate_weight: f32,
}

impl AssignmentScorer {
    pub fn new() -> Self {
        Self {
            skill_weight: 0.5,
            workload_weight: 0.3,
            success_rate_weight: 0.2,
        }
    }

    pub fn find_best_worker(
        &self,
        task: &Task,
        workers: &[TeamMember],
    ) -> Result<&TeamMember, AssignmentError> {
        if workers.is_empty() {
            return Err(AssignmentError::NoWorkersAvailable);
        }

        let required_skills: Vec<String> = match &task.required_skills {
            Some(value) => serde_json::from_value(value.clone())?,
            None => Vec::new(),
        };

        let mut best_worker = &workers[0];
        let mut best_score = 0.0;

        for worker in workers {
            let score = self.calculate_score(task, worker, &required_skills);
            if score > best_score {
                best_score = score;
                best_worker = worker;
            }
        }

        Ok(best_worker)
    }

    fn calculate_score(
        &self,
        task: &Task,
        worker: &TeamMember,
        required_skills: &[String],
    ) -> f32 {
        // 1. Skill match score (0.0-1.0)
        let skill_score = if let Some(spec) = &worker.specialization {
            let profile = SkillProfile::from_specialization(spec);
            profile.match_score(required_skills)
        } else {
            0.0
        };

        // 2. Workload score (inverse - lower workload = higher score)
        let workload_score = if worker.max_concurrent_tasks > 0 {
            1.0 - (worker.current_workload as f32 / worker.max_concurrent_tasks as f32)
        } else {
            0.0
        };

        // 3. Success rate score
        let total_tasks = worker.tasks_completed + worker.tasks_failed;
        let success_rate_score = if total_tasks > 0 {
            worker.tasks_completed as f32 / total_tasks as f32
        } else {
            0.5 // Neutral for new workers
        };

        // Weighted combination
        let total_score = (skill_score * self.skill_weight)
            + (workload_score * self.workload_weight)
            + (success_rate_score * self.success_rate_weight);

        total_score
    }

    pub fn get_assignment_explanation(
        &self,
        task: &Task,
        worker: &TeamMember,
        required_skills: &[String],
    ) -> String {
        let skill_profile = worker.specialization.as_ref()
            .map(|s| SkillProfile::from_specialization(s))
            .unwrap_or_else(|| SkillProfile { skills: HashMap::new() });

        let skill_score = skill_profile.match_score(required_skills);
        let workload = worker.current_workload as f32 / worker.max_concurrent_tasks as f32;
        let total_tasks = worker.tasks_completed + worker.tasks_failed;
        let success_rate = if total_tasks > 0 {
            (worker.tasks_completed as f32 / total_tasks as f32) * 100.0
        } else {
            0.0
        };

        format!(
            "Assigned to {} ({})\n\
             - Skill Match: {:.1}%\n\
             - Current Workload: {}/{} ({:.0}%)\n\
             - Success Rate: {:.1}% ({} completed, {} failed)",
            worker.agent_id,
            worker.specialization.as_deref().unwrap_or("Generalist"),
            skill_score * 100.0,
            worker.current_workload,
            worker.max_concurrent_tasks,
            workload * 100.0,
            success_rate,
            worker.tasks_completed,
            worker.tasks_failed
        )
    }
}

#[derive(Debug, Error)]
pub enum AssignmentError {
    #[error("No workers available")]
    NoWorkersAvailable,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

- [ ] 3.1.3: Add assignment logging

```rust
// apps/api/src/orchestration/assignment_logger.rs
use uuid::Uuid;

pub async fn log_assignment(
    db: &PgPool,
    task_id: Uuid,
    worker_id: Uuid,
    score: f32,
    explanation: String,
) -> Result<(), sqlx::Error> {
    let team_id = sqlx::query_scalar!(
        "SELECT team_id FROM tasks WHERE id = $1",
        task_id
    )
    .fetch_one(db)
    .await?;

    let worker_agent_id = sqlx::query_scalar!(
        "SELECT agent_id FROM team_members WHERE id = $1",
        worker_id
    )
    .fetch_one(db)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO messages (
            team_id, from_agent_id, to_agent_id, task_id,
            message_type, content, metadata
        )
        VALUES ($1, $2, $3, $4, 'task_assignment', $5, $6)
        "#,
        team_id,
        worker_agent_id,
        worker_agent_id,
        task_id,
        explanation,
        serde_json::json!({ "assignment_score": score })
    )
    .execute(db)
    .await?;

    Ok(())
}
```

- [ ] 3.1.4: Test assignment algorithm

```rust
// apps/api/tests/assignment_scoring_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_matching() {
        let profile = SkillProfile::from_specialization("Technical Executor");
        let required = vec!["coding".to_string(), "testing".to_string()];
        let score = profile.match_score(&required);

        assert!(score > 0.7, "Expected high skill match for coding/testing");
    }

    #[test]
    fn test_workload_scoring() {
        let scorer = AssignmentScorer::new();

        let worker_busy = TeamMember {
            current_workload: 3,
            max_concurrent_tasks: 3,
            ..Default::default()
        };

        let worker_free = TeamMember {
            current_workload: 0,
            max_concurrent_tasks: 3,
            ..Default::default()
        };

        let task = Task::default();
        let score_busy = scorer.calculate_score(&task, &worker_busy, &[]);
        let score_free = scorer.calculate_score(&task, &worker_free, &[]);

        assert!(score_free > score_busy, "Free worker should score higher");
    }

    #[test]
    fn test_best_worker_selection() {
        let scorer = AssignmentScorer::new();

        let workers = vec![
            TeamMember {
                specialization: Some("Technical Executor".to_string()),
                current_workload: 2,
                max_concurrent_tasks: 3,
                tasks_completed: 10,
                tasks_failed: 1,
                ..Default::default()
            },
            TeamMember {
                specialization: Some("Content Creator".to_string()),
                current_workload: 0,
                max_concurrent_tasks: 3,
                tasks_completed: 5,
                tasks_failed: 0,
                ..Default::default()
            },
        ];

        let task = Task {
            required_skills: Some(serde_json::json!(["coding", "testing"])),
            ..Default::default()
        };

        let best = scorer.find_best_worker(&task, &workers).unwrap();
        assert_eq!(best.specialization, Some("Technical Executor".to_string()));
    }
}
```

**Acceptance Criteria**:

- [ ] Skill matching returns accurate scores
- [ ] Workload is considered in scoring
- [ ] Success rate affects assignment
- [ ] Best worker consistently selected
- [ ] Assignment explanations are clear
- [ ] All tests passing

---

## Epic 4: Task Queue Management (Redis Streams)

### Task 4.1: Implement Redis Streams for Task Queue

**Type**: Backend
**Dependencies**: Redis 7+

**Subtasks**:

- [ ] 4.1.1: Create Redis Streams task queue

```rust
// apps/api/src/infrastructure/redis/task_stream.rs
use redis::{AsyncCommands, RedisError, streams::StreamReadOptions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskMessage {
    pub task_id: Uuid,
    pub team_id: Uuid,
    pub worker_id: Uuid,
    pub priority: i32,
    pub timestamp: i64,
}

pub struct TaskStream {
    client: redis::Client,
}

impl TaskStream {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    fn stream_key(team_id: Uuid) -> String {
        format!("team:{}:task_stream", team_id)
    }

    fn consumer_group(team_id: Uuid) -> String {
        format!("team:{}:workers", team_id)
    }

    pub async fn create_consumer_group(&self, team_id: Uuid) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(team_id);
        let group = Self::consumer_group(team_id);

        // Create group, ignore error if already exists
        let _: Result<String, RedisError> = con.xgroup_create_mkstream(&stream, &group, "0").await;

        Ok(())
    }

    pub async fn add_task(&self, message: TaskMessage) -> Result<String, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(message.team_id);

        let id: String = con.xadd(
            &stream,
            "*",
            &[
                ("task_id", message.task_id.to_string()),
                ("team_id", message.team_id.to_string()),
                ("worker_id", message.worker_id.to_string()),
                ("priority", message.priority.to_string()),
                ("timestamp", message.timestamp.to_string()),
            ]
        ).await?;

        Ok(id)
    }

    pub async fn read_tasks(
        &self,
        team_id: Uuid,
        worker_id: Uuid,
        count: usize,
        block_ms: usize,
    ) -> Result<Vec<(String, TaskMessage)>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(team_id);
        let group = Self::consumer_group(team_id);
        let consumer = format!("worker:{}", worker_id);

        let opts = StreamReadOptions::default()
            .group(&group, &consumer)
            .count(count)
            .block(block_ms);

        let results: redis::streams::StreamReadReply = con.xread_options(&[&stream], &[">"], &opts).await?;

        let mut messages = Vec::new();

        for stream_key in results.keys {
            for stream_id in stream_key.ids {
                let task_id = stream_id.map.get("task_id")
                    .and_then(|v| match v {
                        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        _ => None,
                    })
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .ok_or_else(|| RedisError::from((redis::ErrorKind::TypeError, "Invalid task_id")))?;

                let team_id_val = stream_id.map.get("team_id")
                    .and_then(|v| match v {
                        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        _ => None,
                    })
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .ok_or_else(|| RedisError::from((redis::ErrorKind::TypeError, "Invalid team_id")))?;

                let worker_id_val = stream_id.map.get("worker_id")
                    .and_then(|v| match v {
                        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        _ => None,
                    })
                    .and_then(|s| Uuid::parse_str(&s).ok())
                    .ok_or_else(|| RedisError::from((redis::ErrorKind::TypeError, "Invalid worker_id")))?;

                let priority = stream_id.map.get("priority")
                    .and_then(|v| match v {
                        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        _ => None,
                    })
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(5);

                let timestamp = stream_id.map.get("timestamp")
                    .and_then(|v| match v {
                        redis::Value::Data(bytes) => String::from_utf8(bytes.clone()).ok(),
                        _ => None,
                    })
                    .and_then(|s| s.parse::<i64>().ok())
                    .unwrap_or(0);

                messages.push((
                    stream_id.id.clone(),
                    TaskMessage {
                        task_id,
                        team_id: team_id_val,
                        worker_id: worker_id_val,
                        priority,
                        timestamp,
                    }
                ));
            }
        }

        Ok(messages)
    }

    pub async fn acknowledge_task(
        &self,
        team_id: Uuid,
        message_id: &str,
    ) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(team_id);
        let group = Self::consumer_group(team_id);

        con.xack(&stream, &group, &[message_id]).await?;
        Ok(())
    }

    pub async fn get_pending_count(&self, team_id: Uuid) -> Result<usize, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(team_id);

        let len: usize = con.xlen(&stream).await?;
        Ok(len)
    }

    pub async fn trim_stream(&self, team_id: Uuid, max_len: usize) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let stream = Self::stream_key(team_id);

        con.xtrim(&stream, redis::streams::StreamMaxlen::Approx(max_len)).await?;
        Ok(())
    }
}
```

- [ ] 4.1.2: Integrate task stream with orchestrator

```rust
// Update apps/api/src/orchestration/task_orchestrator.rs
impl TaskOrchestrator {
    async fn assign_task_to_worker(&self, task: &Task) -> Result<(), OrchestrationError> {
        // ... (existing code for scoring)

        // Add to Redis Stream instead of simple queue
        let task_stream = TaskStream::new(&self.redis_url)?;

        task_stream.add_task(TaskMessage {
            task_id: task.id,
            team_id: task.team_id,
            worker_id: best_worker.id,
            priority: task.priority,
            timestamp: chrono::Utc::now().timestamp(),
        }).await?;

        Ok(())
    }
}
```

- [ ] 4.1.3: Create worker task consumer

```rust
// apps/api/src/agents/worker/task_consumer.rs
use crate::infrastructure::redis::TaskStream;
use uuid::Uuid;

pub struct WorkerTaskConsumer {
    worker_id: Uuid,
    team_id: Uuid,
    task_stream: TaskStream,
    db: PgPool,
}

impl WorkerTaskConsumer {
    pub fn new(worker_id: Uuid, team_id: Uuid, redis_url: &str, db: PgPool) -> Result<Self, RedisError> {
        Ok(Self {
            worker_id,
            team_id,
            task_stream: TaskStream::new(redis_url)?,
            db,
        })
    }

    pub async fn start_consuming(&self) -> Result<(), WorkerError> {
        loop {
            // Read tasks from stream (blocks for 5 seconds)
            let messages = self.task_stream.read_tasks(
                self.team_id,
                self.worker_id,
                1, // Read 1 task at a time
                5000, // Block for 5 seconds
            ).await?;

            for (message_id, task_message) in messages {
                // Process task
                match self.process_task(&task_message).await {
                    Ok(_) => {
                        // Acknowledge successful processing
                        self.task_stream.acknowledge_task(self.team_id, &message_id).await?;
                    }
                    Err(e) => {
                        tracing::error!("Task processing failed: {}", e);
                        // Don't acknowledge - message will be retried
                    }
                }
            }

            // Check if should continue
            if !self.should_continue().await? {
                break;
            }
        }

        Ok(())
    }

    async fn process_task(&self, message: &TaskMessage) -> Result<(), WorkerError> {
        // Load task from database
        let task = self.load_task(message.task_id).await?;

        // Execute task (implementation in Phase 4)
        tracing::info!("Processing task {} for worker {}", task.id, self.worker_id);

        Ok(())
    }

    async fn load_task(&self, task_id: Uuid) -> Result<Task, WorkerError> {
        let task = sqlx::query_as!(
            Task,
            r#"
            SELECT
                id, team_id, parent_task_id, title, description,
                acceptance_criteria as "acceptance_criteria: _",
                assigned_to, assigned_by,
                status as "status: _",
                priority, start_time, completion_time,
                revision_count, max_revisions,
                input_data as "input_data: _",
                output_data as "output_data: _",
                error_message,
                required_skills as "required_skills: _",
                estimated_tokens, actual_tokens,
                created_at, updated_at
            FROM tasks
            WHERE id = $1
            "#,
            task_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(task)
    }

    async fn should_continue(&self) -> Result<bool, WorkerError> {
        // Check if worker is still active
        let status = sqlx::query_scalar!(
            r#"SELECT status as "status: _" FROM team_members WHERE id = $1"#,
            self.worker_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(status != MemberStatus::Offline)
    }
}
```

**Acceptance Criteria**:

- [ ] Redis Streams consumer group created
- [ ] Tasks added to stream successfully
- [ ] Workers can read from stream
- [ ] Message acknowledgment working
- [ ] Failed tasks are retried
- [ ] Stream trimming prevents unbounded growth

---

## Epic 5: Dependency Tracking

### Task 5.1: Implement Task Dependency System

**Type**: Backend
**Dependencies**: Task hierarchy from Epic 2

**Subtasks**:

- [ ] 5.1.1: Create explicit dependency table

```sql
-- migrations/010_create_task_dependencies.sql
CREATE TABLE task_dependencies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    depends_on_task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    dependency_type VARCHAR(50) NOT NULL DEFAULT 'blocks',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, depends_on_task_id),
    CONSTRAINT no_self_dependency CHECK (task_id != depends_on_task_id)
);

CREATE INDEX idx_task_dependencies_task_id ON task_dependencies(task_id);
CREATE INDEX idx_task_dependencies_depends_on ON task_dependencies(depends_on_task_id);

-- Function to check if task can be started
CREATE OR REPLACE FUNCTION can_start_task(p_task_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    v_blocking_count INT;
BEGIN
    -- Check if all dependencies are completed
    SELECT COUNT(*) INTO v_blocking_count
    FROM task_dependencies td
    INNER JOIN tasks t ON td.depends_on_task_id = t.id
    WHERE td.task_id = p_task_id
        AND t.status != 'completed';

    RETURN v_blocking_count = 0;
END;
$$ LANGUAGE plpgsql;
```

- [ ] 5.1.2: Implement dependency manager

```rust
// apps/api/src/orchestration/dependency_manager.rs
use uuid::Uuid;
use sqlx::PgPool;

pub struct DependencyManager {
    db: PgPool,
}

impl DependencyManager {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn add_dependency(
        &self,
        task_id: Uuid,
        depends_on: Uuid,
    ) -> Result<(), DependencyError> {
        // Check for cycles before adding
        if self.would_create_cycle(task_id, depends_on).await? {
            return Err(DependencyError::CycleDetected);
        }

        sqlx::query!(
            r#"
            INSERT INTO task_dependencies (task_id, depends_on_task_id)
            VALUES ($1, $2)
            ON CONFLICT (task_id, depends_on_task_id) DO NOTHING
            "#,
            task_id,
            depends_on
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    async fn would_create_cycle(
        &self,
        task_id: Uuid,
        depends_on: Uuid,
    ) -> Result<bool, sqlx::Error> {
        // Check if depends_on transitively depends on task_id
        let result = sqlx::query_scalar!(
            r#"
            WITH RECURSIVE dep_chain AS (
                SELECT depends_on_task_id
                FROM task_dependencies
                WHERE task_id = $1

                UNION

                SELECT td.depends_on_task_id
                FROM task_dependencies td
                INNER JOIN dep_chain dc ON td.task_id = dc.depends_on_task_id
            )
            SELECT EXISTS(SELECT 1 FROM dep_chain WHERE depends_on_task_id = $2)
            "#,
            depends_on,
            task_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn can_start_task(&self, task_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            "SELECT can_start_task($1)",
            task_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(result.unwrap_or(false))
    }

    pub async fn get_blocked_tasks(&self, team_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        let tasks = sqlx::query_scalar!(
            r#"
            SELECT t.id
            FROM tasks t
            WHERE t.team_id = $1
                AND t.status = 'blocked'
                AND NOT can_start_task(t.id)
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    pub async fn unblock_tasks(&self, completed_task_id: Uuid) -> Result<Vec<Uuid>, sqlx::Error> {
        // Find tasks that were blocked by this task
        let potentially_unblocked = sqlx::query_scalar!(
            r#"
            SELECT DISTINCT task_id
            FROM task_dependencies
            WHERE depends_on_task_id = $1
            "#,
            completed_task_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut unblocked = Vec::new();

        for task_id in potentially_unblocked {
            if self.can_start_task(task_id).await? {
                // Update status from blocked to pending
                sqlx::query!(
                    r#"
                    UPDATE tasks
                    SET status = 'pending'
                    WHERE id = $1 AND status = 'blocked'
                    "#,
                    task_id
                )
                .execute(&self.db)
                .await?;

                unblocked.push(task_id);
            }
        }

        Ok(unblocked)
    }

    pub async fn get_dependency_graph(&self, team_id: Uuid) -> Result<DependencyGraph, sqlx::Error> {
        let edges = sqlx::query!(
            r#"
            SELECT
                td.task_id,
                td.depends_on_task_id,
                t1.title as task_title,
                t2.title as depends_on_title
            FROM task_dependencies td
            INNER JOIN tasks t1 ON td.task_id = t1.id
            INNER JOIN tasks t2 ON td.depends_on_task_id = t2.id
            WHERE t1.team_id = $1
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut graph = DependencyGraph::new();

        for edge in edges {
            graph.add_edge(
                edge.task_id,
                edge.depends_on_task_id,
                edge.task_title,
                edge.depends_on_title,
            );
        }

        Ok(graph)
    }
}

#[derive(Debug, Serialize)]
pub struct DependencyGraph {
    pub nodes: HashMap<Uuid, String>,
    pub edges: Vec<(Uuid, Uuid)>,
}

impl DependencyGraph {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    fn add_edge(&mut self, from: Uuid, to: Uuid, from_title: String, to_title: String) {
        self.nodes.insert(from, from_title);
        self.nodes.insert(to, to_title);
        self.edges.push((from, to));
    }
}

#[derive(Debug, Error)]
pub enum DependencyError {
    #[error("Adding dependency would create a cycle")]
    CycleDetected,

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

- [ ] 5.1.3: Update orchestrator to respect dependencies

```rust
// Update apps/api/src/orchestration/task_orchestrator.rs
impl TaskOrchestrator {
    async fn get_pending_tasks(&self, team_id: Uuid) -> Result<Vec<Task>, OrchestrationError> {
        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT
                id, team_id, parent_task_id, title, description,
                acceptance_criteria as "acceptance_criteria: _",
                assigned_to, assigned_by,
                status as "status: _",
                priority, start_time, completion_time,
                revision_count, max_revisions,
                input_data as "input_data: _",
                output_data as "output_data: _",
                error_message,
                required_skills as "required_skills: _",
                estimated_tokens, actual_tokens,
                created_at, updated_at
            FROM tasks
            WHERE team_id = $1
                AND status = 'pending'
                AND can_start_task(id) = true
            ORDER BY priority DESC, created_at ASC
            "#,
            team_id
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    pub async fn on_task_completed(&self, task_id: Uuid) -> Result<(), OrchestrationError> {
        let dependency_manager = DependencyManager::new(self.db.clone());

        // Unblock dependent tasks
        let unblocked = dependency_manager.unblock_tasks(task_id).await?;

        tracing::info!("Task {} completed, unblocked {} tasks", task_id, unblocked.len());

        for unblocked_task_id in unblocked {
            // Tasks are now pending and will be picked up in next orchestration loop
            tracing::debug!("Task {} is now unblocked", unblocked_task_id);
        }

        Ok(())
    }
}
```

- [ ] 5.1.4: Create API endpoints for dependencies

```rust
// apps/api/src/api/handlers/task_dependencies.rs
use axum::{extract::{Path, State}, http::StatusCode, Json};

#[derive(Deserialize)]
pub struct AddDependencyRequest {
    pub depends_on_task_id: Uuid,
}

pub async fn add_task_dependency(
    State(state): State<AppState>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<AddDependencyRequest>,
) -> Result<StatusCode, StatusCode> {
    let dep_manager = DependencyManager::new(state.db.clone());

    dep_manager
        .add_dependency(task_id, request.depends_on_task_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to add dependency: {}", e);
            StatusCode::BAD_REQUEST
        })?;

    Ok(StatusCode::CREATED)
}

pub async fn get_dependency_graph(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<DependencyGraph>, StatusCode> {
    let dep_manager = DependencyManager::new(state.db.clone());

    let graph = dep_manager
        .get_dependency_graph(team_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(graph))
}
```

**Acceptance Criteria**:

- [ ] Can add task dependencies
- [ ] Circular dependencies prevented
- [ ] Blocked tasks not assigned to workers
- [ ] Tasks unblocked when dependencies complete
- [ ] Dependency graph visualization data available
- [ ] All database constraints enforced

---

## Success Criteria - Phase 3 Complete

- [ ] TaskOrchestrator running for active teams
- [ ] Tasks decomposed into hierarchies
- [ ] Skill-based assignment working
- [ ] Redis Streams delivering tasks to workers
- [ ] Dependencies tracked and enforced
- [ ] No task assignment errors
- [ ] Orchestration can be paused and resumed
- [ ] All tests passing

---

## Next Steps

Proceed to [07-phase-4-tool-execution.md](./07-phase-4-tool-execution.md) for tool registry and execution system.

---

**Phase 3: Intelligent Task Orchestration Online**
