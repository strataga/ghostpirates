# Phase 1: Foundation & Database Setup

**Duration**: Weeks 1-2 (14 days)
**Goal**: Database schema → API foundation → Authentication working
**Dependencies**: Infrastructure from Phase 0 (Terraform + Azure setup)

---

## Epic 1: Database Schema Implementation

### Task 1.1: Create Core Database Migrations

**Type**: Database
**Dependencies**: PostgreSQL running on Azure

**Subtasks**:

- [ ] 1.1.1: Initialize migration system

```bash
cd ghostpirates-api
sqlx database create
sqlx migrate add create_companies_table
sqlx migrate add create_users_table
sqlx migrate add create_teams_table
sqlx migrate add create_team_members_table
sqlx migrate add create_tasks_table
sqlx migrate add create_messages_table
sqlx migrate add create_checkpoints_table
sqlx migrate add create_cost_tracking_table
```

- [ ] 1.1.2: Create companies table migration

```sql
-- migrations/XXXXXX_create_companies_table.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_companies_created_at ON companies(created_at);
```

- [ ] 1.1.3: Create users table migration

```sql
-- migrations/XXXXXX_create_users_table.sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_company_id ON users(company_id);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
```

- [ ] 1.1.4: Create teams table migration

```sql
-- migrations/XXXXXX_create_teams_table.sql
CREATE TYPE team_status AS ENUM (
    'pending',
    'planning',
    'active',
    'completed',
    'failed',
    'archived'
);

CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    goal TEXT NOT NULL,
    status team_status NOT NULL DEFAULT 'pending',
    manager_agent_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    budget_limit DECIMAL(12,2),
    metadata JSONB DEFAULT '{}'::jsonb,
    CONSTRAINT positive_budget CHECK (budget_limit IS NULL OR budget_limit > 0)
);

CREATE INDEX idx_teams_company_id ON teams(company_id);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_teams_created_by ON teams(created_by);
CREATE INDEX idx_teams_created_at ON teams(created_at DESC);
```

- [ ] 1.1.5: Create team_members table migration

```sql
-- migrations/XXXXXX_create_team_members_table.sql
CREATE TYPE member_role AS ENUM ('manager', 'worker');
CREATE TYPE member_status AS ENUM ('active', 'idle', 'busy', 'offline');

CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    role member_role NOT NULL,
    specialization VARCHAR(100),
    status member_status NOT NULL DEFAULT 'active',
    current_workload INT NOT NULL DEFAULT 0,
    max_concurrent_tasks INT NOT NULL DEFAULT 3,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, agent_id),
    CONSTRAINT valid_workload CHECK (current_workload >= 0 AND current_workload <= max_concurrent_tasks)
);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);
CREATE INDEX idx_team_members_role ON team_members(role);
CREATE INDEX idx_team_members_status ON team_members(status);
```

- [ ] 1.1.6: Create tasks table migration

```sql
-- migrations/XXXXXX_create_tasks_table.sql
CREATE TYPE task_status AS ENUM (
    'pending',
    'assigned',
    'in_progress',
    'review',
    'completed',
    'failed',
    'revision_requested'
);

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    parent_task_id UUID REFERENCES tasks(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    acceptance_criteria JSONB NOT NULL DEFAULT '[]'::jsonb,
    assigned_to UUID REFERENCES team_members(id),
    assigned_by UUID REFERENCES team_members(id),
    status task_status NOT NULL DEFAULT 'pending',
    start_time TIMESTAMPTZ,
    completion_time TIMESTAMPTZ,
    revision_count INT NOT NULL DEFAULT 0,
    max_revisions INT NOT NULL DEFAULT 3,
    input_data JSONB,
    output_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_revisions CHECK (revision_count >= 0 AND revision_count <= max_revisions)
);

CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_parent_task_id ON tasks(parent_task_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
```

- [ ] 1.1.7: Create messages table migration

```sql
-- migrations/XXXXXX_create_messages_table.sql
CREATE TYPE message_type AS ENUM (
    'task_assignment',
    'task_completion',
    'revision_request',
    'approval',
    'rejection',
    'agent_communication',
    'system_event'
);

CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    from_agent_id UUID NOT NULL,
    to_agent_id UUID,
    message_type message_type NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_team_id ON messages(team_id);
CREATE INDEX idx_messages_type ON messages(message_type);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX idx_messages_from_agent ON messages(from_agent_id);
```

- [ ] 1.1.8: Create checkpoints table migration

```sql
-- migrations/XXXXXX_create_checkpoints_table.sql
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_number INT NOT NULL,
    step_output JSONB NOT NULL,
    accumulated_context JSONB NOT NULL,
    token_count INT,
    cost_estimate DECIMAL(10,6),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, step_number)
);

CREATE INDEX idx_checkpoints_task_id ON checkpoints(task_id);
CREATE INDEX idx_checkpoints_created_at ON checkpoints(created_at DESC);
```

- [ ] 1.1.9: Create cost_tracking table migration

```sql
-- migrations/XXXXXX_create_cost_tracking_table.sql
CREATE TYPE cost_category AS ENUM ('api_call', 'token', 'tool', 'storage', 'compute');

CREATE TABLE cost_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    category cost_category NOT NULL,
    provider VARCHAR(50),
    model VARCHAR(100),
    amount DECIMAL(10,6) NOT NULL,
    unit_count INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_amount CHECK (amount >= 0)
);

CREATE INDEX idx_cost_tracking_team_id ON cost_tracking(team_id);
CREATE INDEX idx_cost_tracking_task_id ON cost_tracking(task_id);
CREATE INDEX idx_cost_tracking_category ON cost_tracking(category);
CREATE INDEX idx_cost_tracking_created_at ON cost_tracking(created_at DESC);
```

- [ ] 1.1.10: Run all migrations

```bash
sqlx migrate run --database-url "${DATABASE_URL}"
```

- [ ] 1.1.11: Verify migrations

```bash
psql "${DATABASE_URL}" -c "\dt"
psql "${DATABASE_URL}" -c "\d+ teams"
psql "${DATABASE_URL}" -c "\d+ tasks"
```

**Acceptance Criteria**:

- [ ] All 9 tables created successfully
- [ ] All indexes created
- [ ] All foreign keys enforced
- [ ] All constraints working (try violating them)
- [ ] No migration errors
- [ ] Can insert test data into all tables

---

### Task 1.2: Create Rust Domain Models

**Type**: Backend
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 1.2.1: Create domain module structure

```bash
cd ghostpirates-api
mkdir -p src/domain/{teams,tasks,agents}
touch src/domain/mod.rs
touch src/domain/teams/{mod.rs,team.rs,member.rs}
touch src/domain/tasks/{mod.rs,task.rs,revision.rs}
touch src/domain/agents/{mod.rs,manager.rs,worker.rs}
```

- [ ] 1.2.2: Define Team domain model

```rust
// src/domain/teams/team.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TeamStatus {
    Pending,
    Planning,
    Active,
    Completed,
    Failed,
    Archived,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Team {
    pub id: Uuid,
    pub company_id: Uuid,
    pub goal: String,
    #[sqlx(try_from = "String")]
    pub status: TeamStatus,
    pub manager_agent_id: Option<Uuid>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub budget_limit: Option<Decimal>,
    #[sqlx(json)]
    pub metadata: serde_json::Value,
}

impl Team {
    pub fn new(company_id: Uuid, goal: String, created_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            company_id,
            goal,
            status: TeamStatus::Pending,
            manager_agent_id: None,
            created_by,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            budget_limit: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn start(&mut self, manager_id: Uuid) {
        self.status = TeamStatus::Planning;
        self.manager_agent_id = Some(manager_id);
        self.started_at = Some(Utc::now());
    }

    pub fn activate(&mut self) {
        self.status = TeamStatus::Active;
    }

    pub fn complete(&mut self) {
        self.status = TeamStatus::Completed;
        self.completed_at = Some(Utc::now());
    }
}
```

- [ ] 1.2.3: Define TeamMember domain model

```rust
// src/domain/teams/member.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Manager,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MemberStatus {
    Active,
    Idle,
    Busy,
    Offline,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: Uuid,
    pub team_id: Uuid,
    pub agent_id: Uuid,
    #[sqlx(try_from = "String")]
    pub role: MemberRole,
    pub specialization: Option<String>,
    #[sqlx(try_from = "String")]
    pub status: MemberStatus,
    pub current_workload: i32,
    pub max_concurrent_tasks: i32,
    pub joined_at: DateTime<Utc>,
}

impl TeamMember {
    pub fn new_manager(team_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            agent_id: Uuid::new_v4(),
            role: MemberRole::Manager,
            specialization: Some("Team Management".to_string()),
            status: MemberStatus::Active,
            current_workload: 0,
            max_concurrent_tasks: 10,
            joined_at: Utc::now(),
        }
    }

    pub fn new_worker(team_id: Uuid, specialization: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            agent_id: Uuid::new_v4(),
            role: MemberRole::Worker,
            specialization: Some(specialization),
            status: MemberStatus::Idle,
            current_workload: 0,
            max_concurrent_tasks: 3,
            joined_at: Utc::now(),
        }
    }

    pub fn can_accept_task(&self) -> bool {
        self.status != MemberStatus::Offline
            && self.current_workload < self.max_concurrent_tasks
    }

    pub fn assign_task(&mut self) {
        self.current_workload += 1;
        self.status = MemberStatus::Busy;
    }

    pub fn complete_task(&mut self) {
        self.current_workload = self.current_workload.saturating_sub(1);
        if self.current_workload == 0 {
            self.status = MemberStatus::Idle;
        }
    }
}
```

- [ ] 1.2.4: Define Task domain model

```rust
// src/domain/tasks/task.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Review,
    Completed,
    Failed,
    RevisionRequested,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub team_id: Uuid,
    pub parent_task_id: Option<Uuid>,
    pub title: String,
    pub description: String,
    #[sqlx(json)]
    pub acceptance_criteria: Vec<String>,
    pub assigned_to: Option<Uuid>,
    pub assigned_by: Option<Uuid>,
    #[sqlx(try_from = "String")]
    pub status: TaskStatus,
    pub start_time: Option<DateTime<Utc>>,
    pub completion_time: Option<DateTime<Utc>>,
    pub revision_count: i32,
    pub max_revisions: i32,
    #[sqlx(json)]
    pub input_data: Option<serde_json::Value>,
    #[sqlx(json)]
    pub output_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        team_id: Uuid,
        title: String,
        description: String,
        acceptance_criteria: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            team_id,
            parent_task_id: None,
            title,
            description,
            acceptance_criteria,
            assigned_to: None,
            assigned_by: None,
            status: TaskStatus::Pending,
            start_time: None,
            completion_time: None,
            revision_count: 0,
            max_revisions: 3,
            input_data: None,
            output_data: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn assign(&mut self, worker_id: Uuid, manager_id: Uuid) {
        self.assigned_to = Some(worker_id);
        self.assigned_by = Some(manager_id);
        self.status = TaskStatus::Assigned;
        self.updated_at = Utc::now();
    }

    pub fn start_work(&mut self) {
        self.status = TaskStatus::InProgress;
        self.start_time = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn submit_for_review(&mut self, output: serde_json::Value) {
        self.status = TaskStatus::Review;
        self.output_data = Some(output);
        self.updated_at = Utc::now();
    }

    pub fn approve(&mut self) {
        self.status = TaskStatus::Completed;
        self.completion_time = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn request_revision(&mut self) -> Result<(), String> {
        if self.revision_count >= self.max_revisions {
            return Err("Maximum revisions exceeded".to_string());
        }
        self.revision_count += 1;
        self.status = TaskStatus::RevisionRequested;
        self.updated_at = Utc::now();
        Ok(())
    }
}
```

- [ ] 1.2.5: Create module exports

```rust
// src/domain/teams/mod.rs
mod team;
mod member;

pub use team::{Team, TeamStatus};
pub use member::{TeamMember, MemberRole, MemberStatus};
```

```rust
// src/domain/tasks/mod.rs
mod task;
mod revision;

pub use task::{Task, TaskStatus};
```

```rust
// src/domain/mod.rs
pub mod teams;
pub mod tasks;
pub mod agents;
```

- [ ] 1.2.6: Add FromRow impls for enums

```rust
// Add to each enum type in separate files
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

impl Type<Postgres> for TeamStatus {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("team_status")
    }
}

impl<'r> Decode<'r, Postgres> for TeamStatus {
    fn decode(value: PgValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
        let s: &str = Decode::<Postgres>::decode(value)?;
        match s {
            "pending" => Ok(TeamStatus::Pending),
            "planning" => Ok(TeamStatus::Planning),
            "active" => Ok(TeamStatus::Active),
            "completed" => Ok(TeamStatus::Completed),
            "failed" => Ok(TeamStatus::Failed),
            "archived" => Ok(TeamStatus::Archived),
            _ => Err(format!("Invalid team status: {}", s).into()),
        }
    }
}
```

- [ ] 1.2.7: Test compilation

```bash
cargo build
cargo test domain::
```

**Acceptance Criteria**:

- [ ] All domain models compile without errors
- [ ] Enums serialize to/from database correctly
- [ ] Business logic methods work (test in unit tests)
- [ ] No clippy warnings

---

### Task 1.3: Create Database Repository Layer

**Type**: Backend
**Dependencies**: Tasks 1.1 and 1.2 complete

**Subtasks**:

- [ ] 1.3.1: Create repository module structure

```bash
mkdir -p src/infrastructure/database/repositories
touch src/infrastructure/database/mod.rs
touch src/infrastructure/database/pool.rs
touch src/infrastructure/database/repositories/{mod.rs,teams.rs,tasks.rs}
```

- [ ] 1.3.2: Implement database pool

```rust
// src/infrastructure/database/pool.rs
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub async fn create_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(database_url)
        .await
}
```

- [ ] 1.3.3: Implement Teams repository

```rust
// src/infrastructure/database/repositories/teams.rs
use crate::domain::teams::{Team, TeamMember};
use sqlx::PgPool;
use uuid::Uuid;

pub struct TeamsRepository {
    pool: PgPool,
}

impl TeamsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, team: &Team) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO teams (
                id, company_id, goal, status, created_by,
                created_at, budget_limit, metadata
            )
            VALUES ($1, $2, $3, $4::team_status, $5, $6, $7, $8)
            "#,
            team.id,
            team.company_id,
            team.goal,
            team.status.to_string() as _,
            team.created_by,
            team.created_at,
            team.budget_limit,
            team.metadata
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, sqlx::Error> {
        let team = sqlx::query_as!(
            Team,
            r#"
            SELECT
                id, company_id, goal,
                status as "status: _",
                manager_agent_id, created_by,
                created_at, started_at, completed_at,
                budget_limit, metadata
            FROM teams
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        Ok(team)
    }

    pub async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, sqlx::Error> {
        let teams = sqlx::query_as!(
            Team,
            r#"
            SELECT
                id, company_id, goal,
                status as "status: _",
                manager_agent_id, created_by,
                created_at, started_at, completed_at,
                budget_limit, metadata
            FROM teams
            WHERE company_id = $1
            ORDER BY created_at DESC
            "#,
            company_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(teams)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE teams
            SET status = $2::team_status, updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn add_member(&self, member: &TeamMember) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO team_members (
                id, team_id, agent_id, role, specialization,
                status, current_workload, max_concurrent_tasks, joined_at
            )
            VALUES ($1, $2, $3, $4::member_role, $5, $6::member_status, $7, $8, $9)
            "#,
            member.id,
            member.team_id,
            member.agent_id,
            member.role.to_string() as _,
            member.specialization,
            member.status.to_string() as _,
            member.current_workload,
            member.max_concurrent_tasks,
            member.joined_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_members(&self, team_id: Uuid) -> Result<Vec<TeamMember>, sqlx::Error> {
        let members = sqlx::query_as!(
            TeamMember,
            r#"
            SELECT
                id, team_id, agent_id,
                role as "role: _",
                specialization,
                status as "status: _",
                current_workload, max_concurrent_tasks, joined_at
            FROM team_members
            WHERE team_id = $1
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(members)
    }
}
```

- [ ] 1.3.4: Implement Tasks repository

```rust
// src/infrastructure/database/repositories/tasks.rs
use crate::domain::tasks::Task;
use sqlx::PgPool;
use uuid::Uuid;

pub struct TasksRepository {
    pool: PgPool,
}

impl TasksRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            INSERT INTO tasks (
                id, team_id, parent_task_id, title, description,
                acceptance_criteria, status, revision_count,
                max_revisions, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::task_status, $8, $9, $10, $11)
            "#,
            task.id,
            task.team_id,
            task.parent_task_id,
            task.title,
            task.description,
            serde_json::to_value(&task.acceptance_criteria).unwrap(),
            task.status.to_string() as _,
            task.revision_count,
            task.max_revisions,
            task.created_at,
            task.updated_at
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn find_by_team(&self, team_id: Uuid) -> Result<Vec<Task>, sqlx::Error> {
        let tasks = sqlx::query_as!(
            Task,
            r#"
            SELECT
                id, team_id, parent_task_id, title, description,
                acceptance_criteria as "acceptance_criteria: _",
                assigned_to, assigned_by,
                status as "status: _",
                start_time, completion_time,
                revision_count, max_revisions,
                input_data as "input_data: _",
                output_data as "output_data: _",
                created_at, updated_at
            FROM tasks
            WHERE team_id = $1
            ORDER BY created_at ASC
            "#,
            team_id
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(tasks)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            UPDATE tasks
            SET status = $2::task_status, updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
```

- [ ] 1.3.5: Test repositories

```bash
cargo test repositories::
```

**Acceptance Criteria**:

- [ ] Can create teams in database
- [ ] Can retrieve teams by ID and company
- [ ] Can add team members
- [ ] Can create and retrieve tasks
- [ ] All queries type-safe with sqlx compile-time checking
- [ ] No SQL injection vulnerabilities

---

## Epic 2: API Foundation (Axum)

### Task 2.1: Create Axum Server Setup

**Type**: Backend
**Dependencies**: None

**Subtasks**:

- [ ] 2.1.1: Create main.rs with Axum server

```rust
// src/main.rs
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber;

mod api;
mod config;
mod domain;
mod infrastructure;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Load configuration
    let config = config::Config::from_env()?;

    // Create database pool
    let pool = infrastructure::database::pool::create_pool(&config.database_url).await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Build application state
    let app_state = api::AppState::new(pool);

    // Build router
    let app = Router::new()
        .route("/health", get(api::handlers::health::health_check))
        .route("/api/teams", post(api::handlers::teams::create_team))
        .route("/api/teams/:id", get(api::handlers::teams::get_team))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

- [ ] 2.1.2: Create configuration module

```rust
// src/config.rs
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub port: u16,
    pub claude_api_key: String,
    pub openai_api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        dotenv::dotenv().ok();
        envy::from_env::<Config>()
    }
}
```

- [ ] 2.1.3: Create API state

```rust
// src/api/mod.rs
use sqlx::PgPool;

pub mod handlers;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}
```

- [ ] 2.1.4: Create health check handler

```rust
// src/api/handlers/health.rs
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
    })
}
```

- [ ] 2.1.5: Test server startup

```bash
cargo run
# In another terminal:
curl http://localhost:4000/health
# Should return: {"status":"healthy"}
```

**Acceptance Criteria**:

- [ ] Server starts on port 4000
- [ ] Health check endpoint returns 200
- [ ] CORS enabled for all origins
- [ ] Request logging working
- [ ] Database migrations run on startup

---

## Epic 3: Authentication System

### Task 3.1: Implement JWT Authentication

**Type**: Backend
**Dependencies**: Task 2.1 complete

**Subtasks**:

- [ ] 3.1.1: Create auth module

```bash
mkdir -p src/api/auth
touch src/api/auth/{mod.rs,jwt.rs,password.rs}
```

- [ ] 3.1.2: Implement JWT token generation

```rust
// src/api/auth/jwt.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user_id
    pub company_id: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn generate_token(&self, user_id: Uuid, company_id: Uuid) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let expire = now + Duration::hours(24);

        let claims = Claims {
            sub: user_id.to_string(),
            company_id: company_id.to_string(),
            iat: now.timestamp() as usize,
            exp: expire.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
    }
}
```

- [ ] 3.1.3: Implement password hashing

```rust
// src/api/auth/password.rs
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub struct PasswordService;

impl PasswordService {
    pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
        Ok(password_hash.to_string())
    }

    pub fn verify(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }
}
```

- [ ] 3.1.4: Create auth middleware

```rust
// src/api/middleware/auth.rs
use crate::api::auth::jwt::{Claims, JwtService};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(
    State(jwt_service): State<JwtService>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let claims = jwt_service
        .verify_token(token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Insert claims into request extensions
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}
```

**Acceptance Criteria**:

- [ ] Can generate JWT tokens
- [ ] Can verify JWT tokens
- [ ] Tokens expire after 24 hours
- [ ] Password hashing works with Argon2
- [ ] Password verification correct
- [ ] Auth middleware blocks unauthorized requests

---

## Success Criteria - Phase 1 Complete

- [ ] All database tables created and indexed
- [ ] Can insert and query all entities
- [ ] Rust domain models working
- [ ] Repository layer functional
- [ ] Axum server running
- [ ] Health check endpoint working
- [ ] JWT authentication implemented
- [ ] Password hashing secure
- [ ] All unit tests passing
- [ ] No compiler warnings

---

## Next Steps

Proceed to [05-phase-2-agent-system.md](./05-phase-2-agent-system.md) for manager and worker agent implementation.

---

**Phase 1: Foundation laid, ready for agents 🚀**
