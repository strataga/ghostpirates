# Database Architecture: PostgreSQL Schema & Redis Patterns

**Duration**: Week 2 (included in Phase 1)
**Goal**: Complete database schema → Migrations → Indexes → Redis patterns
**Dependencies**: PostgreSQL 15+ with pgvector, Redis 7+

---

## Epic 1: PostgreSQL Schema Design

### Task 1.1: Core Entity Tables

**Type**: Database
**Dependencies**: PostgreSQL server running

**Subtasks**:

- [ ] 1.1.1: Create companies and users tables

```sql
-- migrations/001_create_companies_users.sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Companies (workspaces)
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    stripe_customer_id VARCHAR(255),
    subscription_tier VARCHAR(50) DEFAULT 'free',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_tier CHECK (subscription_tier IN ('free', 'starter', 'professional', 'enterprise'))
);

CREATE INDEX idx_companies_created_at ON companies(created_at DESC);
CREATE INDEX idx_companies_stripe_customer ON companies(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    is_admin BOOLEAN NOT NULL DEFAULT false,
    last_login TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_company_id ON users(company_id);
CREATE INDEX idx_users_active ON users(is_active) WHERE is_active = true;
CREATE INDEX idx_users_last_login ON users(last_login DESC) WHERE last_login IS NOT NULL;

-- Auto-update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

- [ ] 1.1.2: Create teams and team members tables

```sql
-- migrations/002_create_teams.sql
CREATE TYPE team_status AS ENUM (
    'pending',      -- Created but not yet initialized
    'planning',     -- Manager analyzing goal
    'active',       -- Team executing tasks
    'paused',       -- Temporarily paused
    'completed',    -- All tasks complete
    'failed',       -- Unrecoverable failure
    'archived'      -- Soft deleted
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
    paused_at TIMESTAMPTZ,
    budget_limit DECIMAL(12,2),
    actual_cost DECIMAL(12,2) DEFAULT 0,
    metadata JSONB DEFAULT '{}'::jsonb,
    CONSTRAINT positive_budget CHECK (budget_limit IS NULL OR budget_limit > 0),
    CONSTRAINT valid_cost CHECK (actual_cost >= 0)
);

CREATE INDEX idx_teams_company_id ON teams(company_id);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_teams_created_by ON teams(created_by);
CREATE INDEX idx_teams_created_at ON teams(created_at DESC);
CREATE INDEX idx_teams_active ON teams(company_id, status) WHERE status IN ('planning', 'active');
CREATE INDEX idx_teams_metadata ON teams USING GIN (metadata jsonb_path_ops);

CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Team members (agents in team)
CREATE TYPE member_role AS ENUM ('manager', 'worker');
CREATE TYPE member_status AS ENUM ('active', 'idle', 'busy', 'offline', 'failed');

CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    role member_role NOT NULL,
    specialization VARCHAR(100),
    status member_status NOT NULL DEFAULT 'idle',
    current_workload INT NOT NULL DEFAULT 0,
    max_concurrent_tasks INT NOT NULL DEFAULT 3,
    tasks_completed INT NOT NULL DEFAULT 0,
    tasks_failed INT NOT NULL DEFAULT 0,
    total_tokens_used BIGINT NOT NULL DEFAULT 0,
    total_cost DECIMAL(10,6) NOT NULL DEFAULT 0,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ,
    UNIQUE(team_id, agent_id),
    CONSTRAINT valid_workload CHECK (current_workload >= 0 AND current_workload <= max_concurrent_tasks),
    CONSTRAINT valid_counts CHECK (tasks_completed >= 0 AND tasks_failed >= 0)
);

CREATE INDEX idx_team_members_team_id ON team_members(team_id);
CREATE INDEX idx_team_members_role ON team_members(role);
CREATE INDEX idx_team_members_status ON team_members(status);
CREATE INDEX idx_team_members_specialization ON team_members(specialization) WHERE specialization IS NOT NULL;
CREATE INDEX idx_team_members_available ON team_members(team_id, status, current_workload)
    WHERE status IN ('idle', 'active') AND current_workload < max_concurrent_tasks;
```

- [ ] 1.1.3: Create tasks and subtasks tables

```sql
-- migrations/003_create_tasks.sql
CREATE TYPE task_status AS ENUM (
    'pending',              -- Created but not yet assigned
    'assigned',             -- Assigned to worker
    'in_progress',          -- Worker actively working
    'review',               -- Submitted for manager review
    'completed',            -- Approved and complete
    'failed',               -- Failed after max retries
    'revision_requested',   -- Manager requested changes
    'blocked'               -- Waiting on dependency
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
    priority INT NOT NULL DEFAULT 5,
    start_time TIMESTAMPTZ,
    completion_time TIMESTAMPTZ,
    revision_count INT NOT NULL DEFAULT 0,
    max_revisions INT NOT NULL DEFAULT 3,
    input_data JSONB,
    output_data JSONB,
    error_message TEXT,
    required_skills JSONB DEFAULT '[]'::jsonb,
    estimated_tokens INT,
    actual_tokens INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_revisions CHECK (revision_count >= 0 AND revision_count <= max_revisions),
    CONSTRAINT valid_priority CHECK (priority BETWEEN 1 AND 10)
);

CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_parent_task_id ON tasks(parent_task_id) WHERE parent_task_id IS NOT NULL;
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned_to ON tasks(assigned_to) WHERE assigned_to IS NOT NULL;
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX idx_tasks_pending ON tasks(team_id, status, priority DESC)
    WHERE status = 'pending';
CREATE INDEX idx_tasks_in_progress ON tasks(assigned_to, status)
    WHERE status IN ('assigned', 'in_progress');
CREATE INDEX idx_tasks_review_queue ON tasks(team_id, status)
    WHERE status = 'review';

CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Task revisions (history of feedback loops)
CREATE TABLE task_revisions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    revision_number INT NOT NULL,
    feedback TEXT NOT NULL,
    requested_by UUID NOT NULL REFERENCES team_members(id),
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    previous_output JSONB,
    revised_output JSONB,
    UNIQUE(task_id, revision_number)
);

CREATE INDEX idx_task_revisions_task_id ON task_revisions(task_id);
CREATE INDEX idx_task_revisions_requested_at ON task_revisions(requested_at DESC);
```

- [ ] 1.1.4: Create messages and audit log tables

```sql
-- migrations/004_create_messages_audit.sql
CREATE TYPE message_type AS ENUM (
    'task_assignment',
    'task_completion',
    'revision_request',
    'approval',
    'rejection',
    'agent_communication',
    'system_event',
    'error',
    'warning',
    'info'
);

-- Agent communication and audit trail
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    from_agent_id UUID NOT NULL,
    to_agent_id UUID,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    message_type message_type NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_team_id ON messages(team_id);
CREATE INDEX idx_messages_task_id ON messages(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_messages_type ON messages(message_type);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);
CREATE INDEX idx_messages_conversation ON messages(team_id, task_id, created_at)
    WHERE task_id IS NOT NULL;

-- Audit log for compliance
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID REFERENCES teams(id) ON DELETE SET NULL,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    event_type VARCHAR(100) NOT NULL,
    entity_type VARCHAR(50),
    entity_id UUID,
    details JSONB DEFAULT '{}'::jsonb,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_log_team_id ON audit_log(team_id) WHERE team_id IS NOT NULL;
CREATE INDEX idx_audit_log_user_id ON audit_log(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);
CREATE INDEX idx_audit_log_entity ON audit_log(entity_type, entity_id) WHERE entity_type IS NOT NULL;
```

- [ ] 1.1.5: Create checkpoints and cost tracking tables

```sql
-- migrations/005_create_checkpoints_costs.sql

-- Checkpoints for error recovery
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_number INT NOT NULL,
    step_description TEXT,
    step_output JSONB NOT NULL,
    accumulated_context JSONB NOT NULL,
    token_count INT,
    cost_estimate DECIMAL(10,6),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, step_number)
);

CREATE INDEX idx_checkpoints_task_id ON checkpoints(task_id);
CREATE INDEX idx_checkpoints_created_at ON checkpoints(created_at DESC);
CREATE INDEX idx_checkpoints_resumable ON checkpoints(task_id, step_number DESC);

-- Cost tracking
CREATE TYPE cost_category AS ENUM (
    'api_call',
    'token_input',
    'token_output',
    'tool_execution',
    'storage',
    'compute'
);

CREATE TABLE cost_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    member_id UUID REFERENCES team_members(id) ON DELETE SET NULL,
    category cost_category NOT NULL,
    provider VARCHAR(50),
    model VARCHAR(100),
    amount DECIMAL(10,6) NOT NULL,
    unit_count INT,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT positive_amount CHECK (amount >= 0)
);

CREATE INDEX idx_cost_tracking_team_id ON cost_tracking(team_id);
CREATE INDEX idx_cost_tracking_task_id ON cost_tracking(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_cost_tracking_member_id ON cost_tracking(member_id) WHERE member_id IS NOT NULL;
CREATE INDEX idx_cost_tracking_category ON cost_tracking(category);
CREATE INDEX idx_cost_tracking_created_at ON cost_tracking(created_at DESC);
CREATE INDEX idx_cost_tracking_team_date ON cost_tracking(team_id, created_at DESC);
CREATE INDEX idx_cost_tracking_provider_model ON cost_tracking(provider, model) WHERE provider IS NOT NULL;

-- Materialized view for cost aggregation
CREATE MATERIALIZED VIEW team_cost_summary AS
SELECT
    team_id,
    category,
    provider,
    model,
    COUNT(*) as transaction_count,
    SUM(amount) as total_cost,
    SUM(unit_count) as total_units,
    DATE_TRUNC('day', created_at) as date
FROM cost_tracking
GROUP BY team_id, category, provider, model, DATE_TRUNC('day', created_at);

CREATE UNIQUE INDEX idx_team_cost_summary_unique
    ON team_cost_summary(team_id, category, COALESCE(provider, ''), COALESCE(model, ''), date);

CREATE INDEX idx_team_cost_summary_team_date ON team_cost_summary(team_id, date DESC);
```

**Acceptance Criteria**:

- [ ] All tables created successfully
- [ ] All constraints enforced
- [ ] All indexes created
- [ ] Triggers working for updated_at
- [ ] Can insert test data into all tables
- [ ] Foreign key relationships correct

---

### Task 1.2: Advanced Database Features

**Type**: Database
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 1.2.1: Install and configure pgvector extension

```sql
-- migrations/006_configure_pgvector.sql
CREATE EXTENSION IF NOT EXISTS vector;

-- Add embedding columns for semantic search
ALTER TABLE messages ADD COLUMN embedding vector(1536);
ALTER TABLE tasks ADD COLUMN description_embedding vector(1536);

-- HNSW indexes for fast similarity search
CREATE INDEX idx_messages_embedding ON messages
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

CREATE INDEX idx_tasks_embedding ON tasks
    USING hnsw (description_embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- Function to find similar messages
CREATE OR REPLACE FUNCTION find_similar_messages(
    query_embedding vector(1536),
    team_id_filter UUID,
    limit_count INT DEFAULT 5
)
RETURNS TABLE (
    message_id UUID,
    content TEXT,
    similarity FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        id,
        content,
        1 - (embedding <=> query_embedding) as similarity
    FROM messages
    WHERE
        team_id = team_id_filter
        AND embedding IS NOT NULL
    ORDER BY embedding <=> query_embedding
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;

-- Function to find similar tasks
CREATE OR REPLACE FUNCTION find_similar_tasks(
    query_embedding vector(1536),
    team_id_filter UUID,
    limit_count INT DEFAULT 5
)
RETURNS TABLE (
    task_id UUID,
    title TEXT,
    description TEXT,
    similarity FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        id,
        title,
        description,
        1 - (description_embedding <=> query_embedding) as similarity
    FROM tasks
    WHERE
        team_id = team_id_filter
        AND description_embedding IS NOT NULL
    ORDER BY description_embedding <=> query_embedding
    LIMIT limit_count;
END;
$$ LANGUAGE plpgsql;
```

- [ ] 1.2.2: Create database functions for business logic

```sql
-- migrations/007_create_functions.sql

-- Function to assign task to best available worker
CREATE OR REPLACE FUNCTION assign_task_to_best_worker(
    p_task_id UUID,
    p_required_skills JSONB DEFAULT '[]'::jsonb
)
RETURNS UUID AS $$
DECLARE
    v_team_id UUID;
    v_best_member_id UUID;
    v_manager_id UUID;
BEGIN
    -- Get team_id from task
    SELECT team_id INTO v_team_id FROM tasks WHERE id = p_task_id;

    -- Get manager member_id
    SELECT id INTO v_manager_id
    FROM team_members
    WHERE team_id = v_team_id AND role = 'manager';

    -- Find best available worker
    SELECT id INTO v_best_member_id
    FROM team_members
    WHERE
        team_id = v_team_id
        AND role = 'worker'
        AND status IN ('idle', 'active')
        AND current_workload < max_concurrent_tasks
    ORDER BY
        current_workload ASC,
        tasks_completed DESC
    LIMIT 1;

    IF v_best_member_id IS NULL THEN
        RAISE EXCEPTION 'No available workers for task %', p_task_id;
    END IF;

    -- Update task
    UPDATE tasks
    SET
        assigned_to = v_best_member_id,
        assigned_by = v_manager_id,
        status = 'assigned',
        updated_at = NOW()
    WHERE id = p_task_id;

    -- Update worker workload
    UPDATE team_members
    SET
        current_workload = current_workload + 1,
        status = CASE
            WHEN current_workload + 1 >= max_concurrent_tasks THEN 'busy'::member_status
            ELSE 'active'::member_status
        END,
        last_active_at = NOW()
    WHERE id = v_best_member_id;

    RETURN v_best_member_id;
END;
$$ LANGUAGE plpgsql;

-- Function to complete task and update worker
CREATE OR REPLACE FUNCTION complete_task(
    p_task_id UUID,
    p_output_data JSONB
)
RETURNS VOID AS $$
DECLARE
    v_worker_id UUID;
    v_team_id UUID;
BEGIN
    -- Get worker and team
    SELECT assigned_to, team_id INTO v_worker_id, v_team_id
    FROM tasks
    WHERE id = p_task_id;

    -- Update task
    UPDATE tasks
    SET
        status = 'review',
        output_data = p_output_data,
        completion_time = NOW(),
        updated_at = NOW()
    WHERE id = p_task_id;

    -- Update worker workload
    UPDATE team_members
    SET
        current_workload = GREATEST(current_workload - 1, 0),
        status = CASE
            WHEN current_workload - 1 = 0 THEN 'idle'::member_status
            ELSE 'active'::member_status
        END,
        last_active_at = NOW()
    WHERE id = v_worker_id;

    -- Log completion message
    INSERT INTO messages (team_id, from_agent_id, message_type, content, task_id)
    SELECT
        v_team_id,
        agent_id,
        'task_completion',
        'Task completed and submitted for review',
        p_task_id
    FROM team_members
    WHERE id = v_worker_id;
END;
$$ LANGUAGE plpgsql;

-- Function to track cost
CREATE OR REPLACE FUNCTION track_cost(
    p_team_id UUID,
    p_task_id UUID DEFAULT NULL,
    p_member_id UUID DEFAULT NULL,
    p_category cost_category,
    p_provider VARCHAR,
    p_model VARCHAR,
    p_amount DECIMAL,
    p_unit_count INT DEFAULT NULL
)
RETURNS VOID AS $$
BEGIN
    INSERT INTO cost_tracking (
        team_id, task_id, member_id, category,
        provider, model, amount, unit_count
    ) VALUES (
        p_team_id, p_task_id, p_member_id, p_category,
        p_provider, p_model, p_amount, p_unit_count
    );

    -- Update team actual cost
    UPDATE teams
    SET actual_cost = actual_cost + p_amount
    WHERE id = p_team_id;

    -- Update member cost if applicable
    IF p_member_id IS NOT NULL THEN
        UPDATE team_members
        SET
            total_cost = total_cost + p_amount,
            total_tokens_used = total_tokens_used + COALESCE(p_unit_count, 0)
        WHERE id = p_member_id;
    END IF;
END;
$$ LANGUAGE plpgsql;
```

- [ ] 1.2.3: Create views for common queries

```sql
-- migrations/008_create_views.sql

-- Active teams with member counts
CREATE VIEW active_teams_summary AS
SELECT
    t.id,
    t.company_id,
    t.goal,
    t.status,
    t.created_at,
    t.budget_limit,
    t.actual_cost,
    COUNT(DISTINCT tm.id) FILTER (WHERE tm.role = 'worker') as worker_count,
    COUNT(DISTINCT tasks.id) as total_tasks,
    COUNT(DISTINCT tasks.id) FILTER (WHERE tasks.status = 'completed') as completed_tasks,
    COUNT(DISTINCT tasks.id) FILTER (WHERE tasks.status IN ('pending', 'assigned', 'in_progress')) as active_tasks
FROM teams t
LEFT JOIN team_members tm ON t.id = tm.team_id
LEFT JOIN tasks ON t.id = tasks.team_id
WHERE t.status IN ('planning', 'active')
GROUP BY t.id;

-- Worker performance metrics
CREATE VIEW worker_performance AS
SELECT
    tm.id,
    tm.team_id,
    tm.agent_id,
    tm.specialization,
    tm.tasks_completed,
    tm.tasks_failed,
    CASE
        WHEN (tm.tasks_completed + tm.tasks_failed) > 0
        THEN ROUND(100.0 * tm.tasks_completed / (tm.tasks_completed + tm.tasks_failed), 2)
        ELSE 0
    END as success_rate_pct,
    tm.total_cost,
    tm.total_tokens_used,
    CASE
        WHEN tm.tasks_completed > 0
        THEN ROUND(tm.total_cost / tm.tasks_completed, 4)
        ELSE 0
    END as avg_cost_per_task,
    tm.current_workload,
    tm.max_concurrent_tasks,
    ROUND(100.0 * tm.current_workload / tm.max_concurrent_tasks, 2) as workload_pct
FROM team_members tm
WHERE tm.role = 'worker';

-- Team cost breakdown
CREATE VIEW team_cost_breakdown AS
SELECT
    t.id as team_id,
    t.goal,
    t.budget_limit,
    t.actual_cost,
    CASE
        WHEN t.budget_limit IS NOT NULL AND t.budget_limit > 0
        THEN ROUND(100.0 * t.actual_cost / t.budget_limit, 2)
        ELSE NULL
    END as budget_used_pct,
    COUNT(DISTINCT ct.id) as cost_transactions,
    SUM(ct.amount) FILTER (WHERE ct.category = 'api_call') as api_call_cost,
    SUM(ct.amount) FILTER (WHERE ct.category IN ('token_input', 'token_output')) as token_cost,
    SUM(ct.amount) FILTER (WHERE ct.category = 'tool_execution') as tool_cost
FROM teams t
LEFT JOIN cost_tracking ct ON t.id = ct.team_id
GROUP BY t.id;
```

- [ ] 1.2.4: Set up database monitoring queries

```sql
-- migrations/009_monitoring_queries.sql

-- Create extension for monitoring
CREATE EXTENSION IF NOT EXISTS pg_stat_statements;

-- View for slow queries
CREATE VIEW slow_queries AS
SELECT
    query,
    calls,
    total_exec_time,
    mean_exec_time,
    max_exec_time,
    stddev_exec_time
FROM pg_stat_statements
WHERE mean_exec_time > 100
ORDER BY mean_exec_time DESC
LIMIT 20;

-- View for table sizes
CREATE VIEW table_sizes AS
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) as table_size,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename) - pg_relation_size(schemaname||'.'||tablename)) as index_size
FROM pg_tables
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- View for index usage
CREATE VIEW index_usage AS
SELECT
    schemaname,
    tablename,
    indexname,
    idx_scan as scans,
    idx_tup_read as tuples_read,
    idx_tup_fetch as tuples_fetched,
    pg_size_pretty(pg_relation_size(indexrelid)) as index_size
FROM pg_stat_user_indexes
ORDER BY idx_scan ASC, pg_relation_size(indexrelid) DESC;
```

**Acceptance Criteria**:

- [ ] pgvector extension installed
- [ ] Embedding columns added
- [ ] HNSW indexes created
- [ ] Business logic functions working
- [ ] Views returning correct data
- [ ] Monitoring queries operational

---

## Epic 2: Redis Architecture

### Task 2.1: Redis Data Structures

**Type**: Infrastructure
**Dependencies**: Redis 7+ running

**Subtasks**:

- [ ] 2.1.1: Design Redis key naming conventions

```rust
// apps/api/src/infrastructure/redis/keys.rs
use uuid::Uuid;

pub struct RedisKeys;

impl RedisKeys {
    // Team state cache
    pub fn team_state(team_id: Uuid) -> String {
        format!("team:{}:state", team_id)
    }

    // Team member state
    pub fn member_state(member_id: Uuid) -> String {
        format!("member:{}:state", member_id)
    }

    // Task queue for workers
    pub fn task_queue(team_id: Uuid) -> String {
        format!("team:{}:task_queue", team_id)
    }

    // Task assignment lock
    pub fn task_lock(task_id: Uuid) -> String {
        format!("task:{}:lock", task_id)
    }

    // Real-time team updates channel
    pub fn team_updates_channel(team_id: Uuid) -> String {
        format!("team:{}:updates", team_id)
    }

    // LLM response cache (semantic)
    pub fn llm_cache(prompt_hash: &str) -> String {
        format!("llm:cache:{}", prompt_hash)
    }

    // Rate limiting
    pub fn rate_limit(key: &str) -> String {
        format!("rate_limit:{}", key)
    }

    // Session data
    pub fn session(session_id: &str) -> String {
        format!("session:{}", session_id)
    }
}
```

- [ ] 2.1.2: Implement team state caching

```rust
// apps/api/src/infrastructure/redis/team_cache.rs
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamState {
    pub id: Uuid,
    pub status: String,
    pub active_tasks: Vec<Uuid>,
    pub member_count: usize,
    pub current_cost: f64,
    pub last_updated: i64,
}

pub struct TeamCache {
    client: redis::Client,
}

impl TeamCache {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn get_team_state(&self, team_id: Uuid) -> Result<Option<TeamState>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::team_state(team_id);

        let json: Option<String> = con.get(&key).await?;
        match json {
            Some(data) => Ok(serde_json::from_str(&data).ok()),
            None => Ok(None),
        }
    }

    pub async fn set_team_state(&self, state: &TeamState) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::team_state(state.id);
        let json = serde_json::to_string(state).unwrap();

        con.set_ex(&key, json, 3600).await?; // 1 hour TTL
        Ok(())
    }

    pub async fn invalidate_team_state(&self, team_id: Uuid) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::team_state(team_id);
        con.del(&key).await?;
        Ok(())
    }
}
```

- [ ] 2.1.3: Implement task queue pattern

```rust
// apps/api/src/infrastructure/redis/task_queue.rs
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct QueuedTask {
    pub task_id: Uuid,
    pub team_id: Uuid,
    pub priority: i32,
    pub created_at: i64,
}

pub struct TaskQueue {
    client: redis::Client,
}

impl TaskQueue {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    // Enqueue task with priority (higher priority = lower score)
    pub async fn enqueue(&self, task: QueuedTask) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::task_queue(task.team_id);
        let json = serde_json::to_string(&task).unwrap();

        // Use sorted set with priority as score (inverted so high priority = low score)
        let score = -(task.priority as f64);
        con.zadd(&key, json, score).await?;
        Ok(())
    }

    // Dequeue highest priority task
    pub async fn dequeue(&self, team_id: Uuid) -> Result<Option<QueuedTask>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::task_queue(team_id);

        // Get and remove highest priority item (lowest score)
        let result: Option<Vec<String>> = con.zpopmin(&key, 1).await?;

        match result {
            Some(items) if !items.is_empty() => {
                Ok(serde_json::from_str(&items[0]).ok())
            }
            _ => Ok(None),
        }
    }

    // Get queue length
    pub async fn length(&self, team_id: Uuid) -> Result<usize, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::task_queue(team_id);
        con.zcard(&key).await
    }
}
```

- [ ] 2.1.4: Implement Pub/Sub for real-time updates

```rust
// apps/api/src/infrastructure/redis/pubsub.rs
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TeamUpdate {
    TaskAssigned { task_id: Uuid, worker_id: Uuid },
    TaskCompleted { task_id: Uuid },
    TaskFailed { task_id: Uuid, error: String },
    CostUpdated { amount: f64 },
    StatusChanged { status: String },
}

pub struct TeamPublisher {
    client: redis::Client,
}

impl TeamPublisher {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn publish_update(&self, team_id: Uuid, update: TeamUpdate) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let channel = RedisKeys::team_updates_channel(team_id);
        let json = serde_json::to_string(&update).unwrap();

        con.publish(&channel, json).await?;
        Ok(())
    }
}

pub struct TeamSubscriber {
    client: redis::Client,
}

impl TeamSubscriber {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn subscribe(&self, team_id: Uuid) -> Result<impl futures::Stream<Item = TeamUpdate>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let channel = RedisKeys::team_updates_channel(team_id);

        let mut pubsub = con.into_pubsub();
        pubsub.subscribe(&channel).await?;

        Ok(futures::stream::unfold(pubsub, |mut pubsub| async move {
            match pubsub.on_message().next().await {
                Some(msg) => {
                    let payload: String = msg.get_payload().ok()?;
                    let update: TeamUpdate = serde_json::from_str(&payload).ok()?;
                    Some((update, pubsub))
                }
                None => None,
            }
        }))
    }
}
```

- [ ] 2.1.5: Implement semantic caching for LLM responses

```rust
// apps/api/src/infrastructure/redis/llm_cache.rs
use redis::{AsyncCommands, RedisError};
use sha2::{Sha256, Digest};

pub struct LLMCache {
    client: redis::Client,
}

impl LLMCache {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    fn hash_prompt(prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn get(&self, prompt: &str) -> Result<Option<String>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::llm_cache(&Self::hash_prompt(prompt));
        con.get(&key).await
    }

    pub async fn set(&self, prompt: &str, response: &str, ttl_seconds: usize) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::llm_cache(&Self::hash_prompt(prompt));
        con.set_ex(&key, response, ttl_seconds).await?;
        Ok(())
    }

    pub async fn invalidate(&self, prompt: &str) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let key = RedisKeys::llm_cache(&Self::hash_prompt(prompt));
        con.del(&key).await?;
        Ok(())
    }
}
```

- [ ] 2.1.6: Implement rate limiting

```rust
// apps/api/src/infrastructure/redis/rate_limit.rs
use redis::{AsyncCommands, RedisError};

pub struct RateLimiter {
    client: redis::Client,
}

impl RateLimiter {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    // Token bucket rate limiting
    pub async fn check_rate_limit(
        &self,
        key: &str,
        max_requests: usize,
        window_seconds: usize,
    ) -> Result<bool, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let redis_key = RedisKeys::rate_limit(key);

        // Increment counter
        let count: usize = con.incr(&redis_key, 1).await?;

        // Set expiry on first request
        if count == 1 {
            con.expire(&redis_key, window_seconds).await?;
        }

        Ok(count <= max_requests)
    }

    // Get remaining requests
    pub async fn get_remaining(&self, key: &str, max_requests: usize) -> Result<usize, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let redis_key = RedisKeys::rate_limit(key);

        let count: Option<usize> = con.get(&redis_key).await?;
        Ok(max_requests.saturating_sub(count.unwrap_or(0)))
    }
}
```

**Acceptance Criteria**:

- [ ] Redis key naming conventions documented
- [ ] Team state caching operational
- [ ] Task queue enqueue/dequeue working
- [ ] Pub/Sub delivering real-time updates
- [ ] LLM response caching functional
- [ ] Rate limiting preventing abuse

---

## Success Criteria - Database Architecture Complete

- [ ] All PostgreSQL tables created
- [ ] All indexes optimized for queries
- [ ] pgvector extension configured
- [ ] Business logic functions tested
- [ ] Views returning correct data
- [ ] Redis data structures implemented
- [ ] Caching strategies operational
- [ ] Real-time pub/sub working
- [ ] Rate limiting functional
- [ ] All migrations tested

---

## Next Steps

Proceed to [04-phase-1-foundation.md](./04-phase-1-foundation.md) for API and authentication implementation.

---

**Database Architecture: Optimized for Agent Workloads 🗄️**
