# Phase 4: Tool Execution System

**Duration**: Weeks 7-8 (14 days)
**Goal**: Tool Registry → Tool Executor → Retry Logic → Semantic Caching → Performance Monitoring
**Dependencies**: Phase 3 complete (Task orchestration operational)

---

## Epic 1: Tool Registry System

### Task 1.1: Create Tool Registry Database Schema

**Type**: Database
**Dependencies**: PostgreSQL from Phase 1

**Subtasks**:

- [ ] 1.1.1: Create tools and tool executions tables

```sql
-- migrations/011_create_tools_registry.sql
CREATE TYPE tool_category AS ENUM (
    'web_search',
    'code_execution',
    'data_analysis',
    'file_io',
    'api_integration',
    'image_processing',
    'text_processing'
);

CREATE TYPE tool_status AS ENUM (
    'active',
    'deprecated',
    'disabled',
    'maintenance'
);

CREATE TABLE tools (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) UNIQUE NOT NULL,
    category tool_category NOT NULL,
    version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
    status tool_status NOT NULL DEFAULT 'active',
    description TEXT NOT NULL,
    input_schema JSONB NOT NULL,
    output_schema JSONB NOT NULL,
    capabilities JSONB DEFAULT '[]'::jsonb,
    rate_limit_per_minute INT,
    timeout_seconds INT DEFAULT 30,
    cost_per_execution DECIMAL(10,6),
    provider VARCHAR(100),
    api_endpoint TEXT,
    configuration JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_timeout CHECK (timeout_seconds > 0 AND timeout_seconds <= 300),
    CONSTRAINT valid_rate_limit CHECK (rate_limit_per_minute IS NULL OR rate_limit_per_minute > 0)
);

CREATE INDEX idx_tools_category ON tools(category);
CREATE INDEX idx_tools_status ON tools(status) WHERE status = 'active';
CREATE INDEX idx_tools_name ON tools(name);
CREATE INDEX idx_tools_capabilities ON tools USING GIN (capabilities jsonb_path_ops);

CREATE TRIGGER update_tools_updated_at BEFORE UPDATE ON tools
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Tool execution history
CREATE TYPE execution_status AS ENUM (
    'pending',
    'running',
    'succeeded',
    'failed',
    'timeout',
    'rate_limited'
);

CREATE TABLE tool_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id),
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    member_id UUID REFERENCES team_members(id) ON DELETE SET NULL,
    status execution_status NOT NULL DEFAULT 'pending',
    input_params JSONB NOT NULL,
    output_data JSONB,
    error_message TEXT,
    execution_time_ms INT,
    tokens_used INT,
    cost DECIMAL(10,6),
    retry_count INT NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_tool_executions_tool_id ON tool_executions(tool_id);
CREATE INDEX idx_tool_executions_task_id ON tool_executions(task_id) WHERE task_id IS NOT NULL;
CREATE INDEX idx_tool_executions_team_id ON tool_executions(team_id);
CREATE INDEX idx_tool_executions_status ON tool_executions(status);
CREATE INDEX idx_tool_executions_created_at ON tool_executions(created_at DESC);
CREATE INDEX idx_tool_executions_team_date ON tool_executions(team_id, created_at DESC);

-- Tool permissions (which agent types can use which tools)
CREATE TABLE tool_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    specialization VARCHAR(100),
    max_executions_per_hour INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tool_id, specialization)
);

CREATE INDEX idx_tool_permissions_tool_id ON tool_permissions(tool_id);
CREATE INDEX idx_tool_permissions_specialization ON tool_permissions(specialization);
```

- [ ] 1.1.2: Create initial tool definitions

```sql
-- migrations/012_seed_initial_tools.sql

-- Web Search Tool
INSERT INTO tools (name, category, description, input_schema, output_schema, capabilities, provider, cost_per_execution, timeout_seconds)
VALUES (
    'brave_web_search',
    'web_search',
    'Search the web using Brave Search API',
    '{
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Search query"},
            "count": {"type": "integer", "default": 10, "minimum": 1, "maximum": 20}
        },
        "required": ["query"]
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "results": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "title": {"type": "string"},
                        "url": {"type": "string"},
                        "description": {"type": "string"}
                    }
                }
            }
        }
    }'::jsonb,
    '["search", "research", "information_gathering"]'::jsonb,
    'Brave',
    0.001,
    10
);

-- Code Execution Tool
INSERT INTO tools (name, category, description, input_schema, output_schema, capabilities, provider, cost_per_execution, timeout_seconds)
VALUES (
    'python_sandbox',
    'code_execution',
    'Execute Python code in sandboxed environment',
    '{
        "type": "object",
        "properties": {
            "code": {"type": "string", "description": "Python code to execute"},
            "timeout": {"type": "integer", "default": 30, "maximum": 60}
        },
        "required": ["code"]
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "stdout": {"type": "string"},
            "stderr": {"type": "string"},
            "result": {"type": "string"},
            "exit_code": {"type": "integer"}
        }
    }'::jsonb,
    '["code_execution", "scripting", "computation"]'::jsonb,
    'E2B',
    0.002,
    60
);

-- Data Analysis Tool
INSERT INTO tools (name, category, description, input_schema, output_schema, capabilities, provider, cost_per_execution, timeout_seconds)
VALUES (
    'pandas_analyzer',
    'data_analysis',
    'Analyze structured data using pandas',
    '{
        "type": "object",
        "properties": {
            "data": {"type": "object", "description": "JSON data or CSV content"},
            "analysis_type": {"type": "string", "enum": ["summary", "correlation", "groupby", "pivot"]},
            "operations": {"type": "array", "items": {"type": "object"}}
        },
        "required": ["data", "analysis_type"]
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "summary": {"type": "object"},
            "visualizations": {"type": "array"},
            "insights": {"type": "array", "items": {"type": "string"}}
        }
    }'::jsonb,
    '["data_analysis", "statistics", "data_processing"]'::jsonb,
    'Internal',
    0.0015,
    45
);

-- File I/O Tool
INSERT INTO tools (name, category, description, input_schema, output_schema, capabilities, provider, cost_per_execution, timeout_seconds)
VALUES (
    'file_operations',
    'file_io',
    'Read and write files in isolated workspace',
    '{
        "type": "object",
        "properties": {
            "operation": {"type": "string", "enum": ["read", "write", "list"]},
            "path": {"type": "string"},
            "content": {"type": "string"}
        },
        "required": ["operation", "path"]
    }'::jsonb,
    '{
        "type": "object",
        "properties": {
            "success": {"type": "boolean"},
            "content": {"type": "string"},
            "files": {"type": "array", "items": {"type": "string"}}
        }
    }'::jsonb,
    '["file_io", "storage", "workspace"]'::jsonb,
    'Internal',
    0.0001,
    15
);

-- Set permissions for tools
INSERT INTO tool_permissions (tool_id, specialization, max_executions_per_hour)
SELECT id, 'Researcher/Analyzer', 30
FROM tools WHERE name = 'brave_web_search';

INSERT INTO tool_permissions (tool_id, specialization, max_executions_per_hour)
SELECT id, 'Technical Executor', 20
FROM tools WHERE name = 'python_sandbox';

INSERT INTO tool_permissions (tool_id, specialization, max_executions_per_hour)
SELECT id, 'Researcher/Analyzer', 15
FROM tools WHERE name = 'pandas_analyzer';

INSERT INTO tool_permissions (tool_id, specialization, max_executions_per_hour)
SELECT id, 'Content Creator', 50
FROM tools WHERE name = 'file_operations';

INSERT INTO tool_permissions (tool_id, specialization, max_executions_per_hour)
SELECT id, 'Technical Executor', 50
FROM tools WHERE name = 'file_operations';
```

- [ ] 1.1.3: Run migrations

```bash
sqlx migrate run --database-url "${DATABASE_URL}"
psql "${DATABASE_URL}" -c "\d tools"
psql "${DATABASE_URL}" -c "SELECT name, category, status FROM tools;"
```

**Acceptance Criteria**:

- [ ] Tools table created successfully
- [ ] Tool executions table created
- [ ] Tool permissions table created
- [ ] Initial tools seeded (4 tools minimum)
- [ ] All constraints enforced
- [ ] Can query tools by category

---

### Task 1.2: Implement Tool Registry Service

**Type**: Backend
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 1.2.1: Create Tool domain model

```rust
// apps/api/src/domain/tools/tool.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCategory {
    WebSearch,
    CodeExecution,
    DataAnalysis,
    FileIo,
    ApiIntegration,
    ImageProcessing,
    TextProcessing,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ToolStatus {
    Active,
    Deprecated,
    Disabled,
    Maintenance,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Tool {
    pub id: Uuid,
    pub name: String,
    #[sqlx(try_from = "String")]
    pub category: ToolCategory,
    pub version: String,
    #[sqlx(try_from = "String")]
    pub status: ToolStatus,
    pub description: String,
    #[sqlx(json)]
    pub input_schema: serde_json::Value,
    #[sqlx(json)]
    pub output_schema: serde_json::Value,
    #[sqlx(json)]
    pub capabilities: Vec<String>,
    pub rate_limit_per_minute: Option<i32>,
    pub timeout_seconds: i32,
    pub cost_per_execution: Option<Decimal>,
    pub provider: Option<String>,
    pub api_endpoint: Option<String>,
    #[sqlx(json)]
    pub configuration: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Timeout,
    RateLimited,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ToolExecution {
    pub id: Uuid,
    pub tool_id: Uuid,
    pub task_id: Option<Uuid>,
    pub team_id: Uuid,
    pub member_id: Option<Uuid>,
    #[sqlx(try_from = "String")]
    pub status: ExecutionStatus,
    #[sqlx(json)]
    pub input_params: serde_json::Value,
    #[sqlx(json)]
    pub output_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub tokens_used: Option<i32>,
    pub cost: Option<Decimal>,
    pub retry_count: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
```

- [ ] 1.2.2: Implement Tool Registry service

```rust
// apps/api/src/services/tool_registry.rs
use crate::domain::tools::{Tool, ToolCategory, ToolStatus};
use sqlx::PgPool;
use uuid::Uuid;

pub struct ToolRegistry {
    db: PgPool,
}

impl ToolRegistry {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_tool(&self, tool_id: Uuid) -> Result<Option<Tool>, RegistryError> {
        let tool = sqlx::query_as!(
            Tool,
            r#"
            SELECT
                id, name,
                category as "category: _",
                version,
                status as "status: _",
                description,
                input_schema,
                output_schema,
                capabilities,
                rate_limit_per_minute,
                timeout_seconds,
                cost_per_execution,
                provider,
                api_endpoint,
                configuration,
                created_at,
                updated_at
            FROM tools
            WHERE id = $1
            "#,
            tool_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(tool)
    }

    pub async fn get_tool_by_name(&self, name: &str) -> Result<Option<Tool>, RegistryError> {
        let tool = sqlx::query_as!(
            Tool,
            r#"
            SELECT
                id, name,
                category as "category: _",
                version,
                status as "status: _",
                description,
                input_schema,
                output_schema,
                capabilities,
                rate_limit_per_minute,
                timeout_seconds,
                cost_per_execution,
                provider,
                api_endpoint,
                configuration,
                created_at,
                updated_at
            FROM tools
            WHERE name = $1 AND status = 'active'
            "#,
            name
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(tool)
    }

    pub async fn get_tools_by_category(
        &self,
        category: ToolCategory,
    ) -> Result<Vec<Tool>, RegistryError> {
        let category_str = format!("{:?}", category).to_lowercase();

        let tools = sqlx::query_as!(
            Tool,
            r#"
            SELECT
                id, name,
                category as "category: _",
                version,
                status as "status: _",
                description,
                input_schema,
                output_schema,
                capabilities,
                rate_limit_per_minute,
                timeout_seconds,
                cost_per_execution,
                provider,
                api_endpoint,
                configuration,
                created_at,
                updated_at
            FROM tools
            WHERE category::text = $1 AND status = 'active'
            ORDER BY name
            "#,
            category_str
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tools)
    }

    pub async fn get_tools_by_capability(
        &self,
        capability: &str,
    ) -> Result<Vec<Tool>, RegistryError> {
        let tools = sqlx::query_as!(
            Tool,
            r#"
            SELECT
                id, name,
                category as "category: _",
                version,
                status as "status: _",
                description,
                input_schema,
                output_schema,
                capabilities,
                rate_limit_per_minute,
                timeout_seconds,
                cost_per_execution,
                provider,
                api_endpoint,
                configuration,
                created_at,
                updated_at
            FROM tools
            WHERE status = 'active'
                AND capabilities @> $1::jsonb
            ORDER BY name
            "#,
            serde_json::json!([capability])
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tools)
    }

    pub async fn get_available_tools_for_agent(
        &self,
        specialization: &str,
    ) -> Result<Vec<Tool>, RegistryError> {
        let tools = sqlx::query_as!(
            Tool,
            r#"
            SELECT DISTINCT
                t.id, t.name,
                t.category as "category: _",
                t.version,
                t.status as "status: _",
                t.description,
                t.input_schema,
                t.output_schema,
                t.capabilities,
                t.rate_limit_per_minute,
                t.timeout_seconds,
                t.cost_per_execution,
                t.provider,
                t.api_endpoint,
                t.configuration,
                t.created_at,
                t.updated_at
            FROM tools t
            INNER JOIN tool_permissions tp ON t.id = tp.tool_id
            WHERE t.status = 'active'
                AND tp.specialization = $1
            ORDER BY t.name
            "#,
            specialization
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tools)
    }

    pub async fn check_rate_limit(
        &self,
        tool_id: Uuid,
        member_id: Uuid,
    ) -> Result<bool, RegistryError> {
        // Check executions in last hour
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM tool_executions te
            INNER JOIN tool_permissions tp ON te.tool_id = tp.tool_id
            INNER JOIN team_members tm ON te.member_id = tm.id
            WHERE te.tool_id = $1
                AND te.member_id = $2
                AND te.created_at > NOW() - INTERVAL '1 hour'
                AND tp.specialization = tm.specialization
            "#,
            tool_id,
            member_id
        )
        .fetch_one(&self.db)
        .await?;

        let limit = sqlx::query_scalar!(
            r#"
            SELECT tp.max_executions_per_hour
            FROM tool_permissions tp
            INNER JOIN team_members tm ON tp.specialization = tm.specialization
            WHERE tp.tool_id = $1 AND tm.id = $2
            "#,
            tool_id,
            member_id
        )
        .fetch_optional(&self.db)
        .await?;

        match (count, limit) {
            (Some(c), Some(Some(l))) => Ok(c < l as i64),
            (Some(_), Some(None)) => Ok(true), // No limit
            _ => Ok(false), // No permission
        }
    }

    pub async fn register_execution(
        &self,
        execution: &ToolExecution,
    ) -> Result<Uuid, RegistryError> {
        let id = sqlx::query_scalar!(
            r#"
            INSERT INTO tool_executions (
                id, tool_id, task_id, team_id, member_id,
                status, input_params, retry_count, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6::execution_status, $7, $8, $9)
            RETURNING id
            "#,
            execution.id,
            execution.tool_id,
            execution.task_id,
            execution.team_id,
            execution.member_id,
            format!("{:?}", execution.status).to_lowercase(),
            execution.input_params,
            execution.retry_count,
            execution.created_at
        )
        .fetch_one(&self.db)
        .await?;

        Ok(id)
    }

    pub async fn update_execution_result(
        &self,
        execution_id: Uuid,
        status: ExecutionStatus,
        output: Option<serde_json::Value>,
        error: Option<String>,
        execution_time_ms: i32,
        cost: Option<Decimal>,
    ) -> Result<(), RegistryError> {
        sqlx::query!(
            r#"
            UPDATE tool_executions
            SET
                status = $2::execution_status,
                output_data = $3,
                error_message = $4,
                execution_time_ms = $5,
                cost = $6,
                completed_at = NOW()
            WHERE id = $1
            "#,
            execution_id,
            format!("{:?}", status).to_lowercase(),
            output,
            error,
            execution_time_ms,
            cost
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}
```

**Acceptance Criteria**:

- [ ] Can retrieve tools by ID
- [ ] Can retrieve tools by name
- [ ] Can get tools by category
- [ ] Can get tools by capability
- [ ] Can check agent permissions
- [ ] Rate limiting enforced
- [ ] Execution history tracked

---

## Epic 2: Tool Executor with Retry Logic

### Task 2.1: Implement Tool Executor

**Type**: Backend
**Dependencies**: Tool Registry from Epic 1

**Subtasks**:

- [ ] 2.1.1: Create Tool Executor service

```rust
// apps/api/src/services/tool_executor.rs
use crate::domain::tools::{Tool, ToolExecution, ExecutionStatus};
use crate::services::tool_registry::ToolRegistry;
use chrono::Utc;
use reqwest::Client;
use serde_json::Value;
use std::time::Instant;
use uuid::Uuid;

pub struct ToolExecutor {
    registry: ToolRegistry,
    http_client: Client,
    db: PgPool,
}

impl ToolExecutor {
    pub fn new(registry: ToolRegistry, db: PgPool) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            registry,
            http_client,
            db,
        }
    }

    pub async fn execute(
        &self,
        tool_name: &str,
        input_params: Value,
        context: ExecutionContext,
    ) -> Result<ToolExecutionResult, ExecutorError> {
        // Get tool from registry
        let tool = self.registry
            .get_tool_by_name(tool_name)
            .await?
            .ok_or_else(|| ExecutorError::ToolNotFound(tool_name.to_string()))?;

        // Check rate limit if member_id provided
        if let Some(member_id) = context.member_id {
            let allowed = self.registry.check_rate_limit(tool.id, member_id).await?;
            if !allowed {
                return Err(ExecutorError::RateLimited);
            }
        }

        // Validate input against schema
        self.validate_input(&tool.input_schema, &input_params)?;

        // Create execution record
        let execution = ToolExecution {
            id: Uuid::new_v4(),
            tool_id: tool.id,
            task_id: context.task_id,
            team_id: context.team_id,
            member_id: context.member_id,
            status: ExecutionStatus::Pending,
            input_params: input_params.clone(),
            output_data: None,
            error_message: None,
            execution_time_ms: None,
            tokens_used: None,
            cost: None,
            retry_count: 0,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        };

        let execution_id = self.registry.register_execution(&execution).await?;

        // Execute with timeout
        let start = Instant::now();
        let timeout = std::time::Duration::from_secs(tool.timeout_seconds as u64);

        let result = tokio::time::timeout(
            timeout,
            self.execute_tool(&tool, &input_params)
        ).await;

        let execution_time_ms = start.elapsed().as_millis() as i32;

        // Process result
        match result {
            Ok(Ok(output)) => {
                // Success
                self.registry.update_execution_result(
                    execution_id,
                    ExecutionStatus::Succeeded,
                    Some(output.clone()),
                    None,
                    execution_time_ms,
                    tool.cost_per_execution,
                ).await?;

                // Track cost if applicable
                if let Some(cost) = tool.cost_per_execution {
                    self.track_cost(context.team_id, context.task_id, cost).await?;
                }

                Ok(ToolExecutionResult {
                    execution_id,
                    status: ExecutionStatus::Succeeded,
                    output: Some(output),
                    error: None,
                    execution_time_ms,
                })
            }
            Ok(Err(e)) => {
                // Tool execution failed
                self.registry.update_execution_result(
                    execution_id,
                    ExecutionStatus::Failed,
                    None,
                    Some(e.to_string()),
                    execution_time_ms,
                    None,
                ).await?;

                Err(ExecutorError::ExecutionFailed(e.to_string()))
            }
            Err(_) => {
                // Timeout
                self.registry.update_execution_result(
                    execution_id,
                    ExecutionStatus::Timeout,
                    None,
                    Some(format!("Execution exceeded {}s timeout", tool.timeout_seconds)),
                    execution_time_ms,
                    None,
                ).await?;

                Err(ExecutorError::Timeout(tool.timeout_seconds))
            }
        }
    }

    async fn execute_tool(
        &self,
        tool: &Tool,
        input_params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        match tool.category {
            ToolCategory::WebSearch => self.execute_web_search(tool, input_params).await,
            ToolCategory::CodeExecution => self.execute_code(tool, input_params).await,
            ToolCategory::DataAnalysis => self.execute_data_analysis(tool, input_params).await,
            ToolCategory::FileIo => self.execute_file_io(tool, input_params).await,
            _ => Err("Tool category not implemented".into()),
        }
    }

    async fn execute_web_search(
        &self,
        tool: &Tool,
        input_params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let query = input_params["query"]
            .as_str()
            .ok_or("Missing query parameter")?;
        let count = input_params["count"].as_i64().unwrap_or(10);

        let endpoint = tool.api_endpoint.as_ref()
            .ok_or("No API endpoint configured")?;

        let api_key = tool.configuration["api_key"]
            .as_str()
            .ok_or("No API key configured")?;

        let response = self.http_client
            .get(endpoint)
            .query(&[("q", query), ("count", &count.to_string())])
            .header("X-Subscription-Token", api_key)
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data)
    }

    async fn execute_code(
        &self,
        tool: &Tool,
        input_params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let code = input_params["code"]
            .as_str()
            .ok_or("Missing code parameter")?;

        // Use E2B or similar sandbox service
        let endpoint = tool.api_endpoint.as_ref()
            .ok_or("No API endpoint configured")?;

        let api_key = tool.configuration["api_key"]
            .as_str()
            .ok_or("No API key configured")?;

        let response = self.http_client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "code": code,
                "language": "python",
                "timeout": input_params["timeout"].as_i64().unwrap_or(30)
            }))
            .send()
            .await?;

        let data: Value = response.json().await?;
        Ok(data)
    }

    async fn execute_data_analysis(
        &self,
        _tool: &Tool,
        input_params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        // Internal implementation using pandas/polars
        let data = &input_params["data"];
        let analysis_type = input_params["analysis_type"]
            .as_str()
            .ok_or("Missing analysis_type")?;

        // Simplified - real implementation would use actual data analysis
        Ok(serde_json::json!({
            "summary": {
                "analysis_type": analysis_type,
                "row_count": 0,
                "column_count": 0
            },
            "insights": []
        }))
    }

    async fn execute_file_io(
        &self,
        _tool: &Tool,
        input_params: &Value,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let operation = input_params["operation"]
            .as_str()
            .ok_or("Missing operation")?;
        let path = input_params["path"]
            .as_str()
            .ok_or("Missing path")?;

        match operation {
            "read" => {
                // Read file from workspace
                Ok(serde_json::json!({
                    "success": true,
                    "content": "file contents would be here"
                }))
            }
            "write" => {
                let content = input_params["content"]
                    .as_str()
                    .ok_or("Missing content")?;

                // Write file to workspace
                Ok(serde_json::json!({
                    "success": true
                }))
            }
            "list" => {
                // List files in workspace
                Ok(serde_json::json!({
                    "success": true,
                    "files": []
                }))
            }
            _ => Err("Unknown operation".into()),
        }
    }

    fn validate_input(&self, schema: &Value, input: &Value) -> Result<(), ExecutorError> {
        // Use jsonschema crate for validation
        let compiled = jsonschema::JSONSchema::compile(schema)
            .map_err(|e| ExecutorError::InvalidSchema(e.to_string()))?;

        compiled.validate(input)
            .map_err(|e| {
                let errors: Vec<String> = e.map(|e| e.to_string()).collect();
                ExecutorError::InvalidInput(errors.join(", "))
            })?;

        Ok(())
    }

    async fn track_cost(
        &self,
        team_id: Uuid,
        task_id: Option<Uuid>,
        cost: Decimal,
    ) -> Result<(), ExecutorError> {
        sqlx::query!(
            "SELECT track_cost($1, $2, NULL, 'tool_execution', NULL, NULL, $3, NULL)",
            team_id,
            task_id,
            cost
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct ExecutionContext {
    pub team_id: Uuid,
    pub task_id: Option<Uuid>,
    pub member_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct ToolExecutionResult {
    pub execution_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<Value>,
    pub error: Option<String>,
    pub execution_time_ms: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutorError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Execution timeout after {0}s")]
    Timeout(i32),

    #[error("Registry error: {0}")]
    Registry(#[from] RegistryError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

- [ ] 2.1.2: Implement retry logic with exponential backoff

```rust
// apps/api/src/services/tool_retry.rs
use std::time::Duration;
use tokio::time::sleep;

pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    pub async fn execute_with_retry<F, T, E>(
        &self,
        mut operation: F,
    ) -> Result<T, E>
    where
        F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
        E: std::fmt::Debug,
    {
        let mut attempt = 0;
        let mut delay_ms = self.initial_delay_ms;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;

                    if attempt > self.max_retries {
                        tracing::error!("Max retries ({}) exceeded: {:?}", self.max_retries, e);
                        return Err(e);
                    }

                    let actual_delay = if self.jitter {
                        let jitter = rand::random::<f64>() * 0.2 - 0.1; // ±10%
                        let jittered = delay_ms as f64 * (1.0 + jitter);
                        jittered.min(self.max_delay_ms as f64) as u64
                    } else {
                        delay_ms.min(self.max_delay_ms)
                    };

                    tracing::warn!(
                        "Attempt {} failed, retrying in {}ms: {:?}",
                        attempt,
                        actual_delay,
                        e
                    );

                    sleep(Duration::from_millis(actual_delay)).await;

                    delay_ms = (delay_ms as f64 * self.multiplier) as u64;
                }
            }
        }
    }
}
```

- [ ] 2.1.3: Add retry support to ToolExecutor

```rust
// Update apps/api/src/services/tool_executor.rs
impl ToolExecutor {
    pub async fn execute_with_retry(
        &self,
        tool_name: &str,
        input_params: Value,
        context: ExecutionContext,
        retry_policy: Option<RetryPolicy>,
    ) -> Result<ToolExecutionResult, ExecutorError> {
        let policy = retry_policy.unwrap_or_default();

        let executor = self.clone();
        let tool_name = tool_name.to_string();
        let params = input_params.clone();
        let ctx = context.clone();

        policy.execute_with_retry(move || {
            let executor = executor.clone();
            let tool_name = tool_name.clone();
            let params = params.clone();
            let ctx = ctx.clone();

            Box::pin(async move {
                executor.execute(&tool_name, params, ctx).await
            })
        }).await
    }
}
```

**Acceptance Criteria**:

- [ ] Can execute web search tools
- [ ] Can execute code execution tools
- [ ] Can execute data analysis tools
- [ ] Can execute file I/O tools
- [ ] Input validation working
- [ ] Timeout enforcement working
- [ ] Retry logic with exponential backoff
- [ ] Rate limiting enforced
- [ ] Execution history tracked

---

## Epic 3: Fallback Strategies

### Task 3.1: Implement Tool Fallback System

**Type**: Backend
**Dependencies**: Tool Executor from Epic 2

**Subtasks**:

- [ ] 3.1.1: Create fallback configuration

```sql
-- migrations/013_create_tool_fallbacks.sql
CREATE TABLE tool_fallbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    primary_tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    fallback_tool_id UUID NOT NULL REFERENCES tools(id) ON DELETE CASCADE,
    priority INT NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(primary_tool_id, fallback_tool_id),
    CONSTRAINT different_tools CHECK (primary_tool_id != fallback_tool_id)
);

CREATE INDEX idx_tool_fallbacks_primary ON tool_fallbacks(primary_tool_id, priority);

-- Seed fallback for web search
INSERT INTO tool_fallbacks (primary_tool_id, fallback_tool_id, priority)
SELECT
    t1.id,
    t2.id,
    1
FROM tools t1
CROSS JOIN tools t2
WHERE t1.name = 'brave_web_search'
    AND t2.name = 'google_web_search';
```

- [ ] 3.1.2: Implement fallback executor

```rust
// apps/api/src/services/tool_fallback.rs
use crate::services::tool_executor::{ToolExecutor, ExecutionContext, ToolExecutionResult};
use crate::services::tool_registry::ToolRegistry;
use serde_json::Value;
use uuid::Uuid;

pub struct FallbackExecutor {
    executor: ToolExecutor,
    registry: ToolRegistry,
    db: PgPool,
}

impl FallbackExecutor {
    pub fn new(executor: ToolExecutor, registry: ToolRegistry, db: PgPool) -> Self {
        Self { executor, registry, db }
    }

    pub async fn execute_with_fallback(
        &self,
        tool_name: &str,
        input_params: Value,
        context: ExecutionContext,
    ) -> Result<ToolExecutionResult, ExecutorError> {
        // Try primary tool
        match self.executor.execute(tool_name, input_params.clone(), context.clone()).await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!("Primary tool {} failed: {}", tool_name, e);

                // Get fallback tools
                let fallbacks = self.get_fallback_tools(tool_name).await?;

                if fallbacks.is_empty() {
                    return Err(e);
                }

                // Try each fallback in order
                for fallback_name in fallbacks {
                    tracing::info!("Trying fallback tool: {}", fallback_name);

                    match self.executor.execute(&fallback_name, input_params.clone(), context.clone()).await {
                        Ok(result) => {
                            tracing::info!("Fallback tool {} succeeded", fallback_name);
                            return Ok(result);
                        }
                        Err(fe) => {
                            tracing::warn!("Fallback tool {} failed: {}", fallback_name, fe);
                            continue;
                        }
                    }
                }

                // All fallbacks failed
                Err(ExecutorError::AllFallbacksFailed(tool_name.to_string()))
            }
        }
    }

    async fn get_fallback_tools(&self, primary_tool_name: &str) -> Result<Vec<String>, ExecutorError> {
        let fallbacks = sqlx::query_scalar!(
            r#"
            SELECT t.name
            FROM tool_fallbacks tf
            INNER JOIN tools t ON tf.fallback_tool_id = t.id
            INNER JOIN tools pt ON tf.primary_tool_id = pt.id
            WHERE pt.name = $1
                AND t.status = 'active'
            ORDER BY tf.priority
            "#,
            primary_tool_name
        )
        .fetch_all(&self.db)
        .await?;

        Ok(fallbacks)
    }
}
```

**Acceptance Criteria**:

- [ ] Fallback configuration stored
- [ ] Can retrieve fallback tools
- [ ] Fallbacks tried in priority order
- [ ] Primary failure triggers fallback
- [ ] Success from fallback returned to caller
- [ ] All fallback failures logged

---

## Epic 4: Semantic Caching with Redis

### Task 4.1: Implement Semantic Cache for Tools

**Type**: Backend
**Dependencies**: Redis from Phase 1, Tool Executor from Epic 2

**Subtasks**:

- [ ] 4.1.1: Create semantic cache service

```rust
// apps/api/src/infrastructure/redis/semantic_cache.rs
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedToolResult {
    pub output: Value,
    pub execution_time_ms: i32,
    pub cached_at: i64,
    pub hit_count: usize,
}

pub struct SemanticToolCache {
    client: redis::Client,
}

impl SemanticToolCache {
    pub fn new(redis_url: &str) -> Result<Self, RedisError> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
        })
    }

    fn cache_key(tool_name: &str, input_hash: &str) -> String {
        format!("tool_cache:{}:{}", tool_name, input_hash)
    }

    fn hash_input(input: &Value) -> String {
        let canonical = serde_json::to_string(input).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub async fn get(
        &self,
        tool_name: &str,
        input_params: &Value,
    ) -> Result<Option<CachedToolResult>, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let input_hash = Self::hash_input(input_params);
        let key = Self::cache_key(tool_name, &input_hash);

        let cached: Option<String> = con.get(&key).await?;

        match cached {
            Some(data) => {
                let mut result: CachedToolResult = serde_json::from_str(&data)
                    .map_err(|e| RedisError::from((redis::ErrorKind::TypeError, "Invalid cached data", e.to_string())))?;

                // Increment hit count
                result.hit_count += 1;
                let updated = serde_json::to_string(&result).unwrap();
                let _: () = con.set(&key, updated).await?;

                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    pub async fn set(
        &self,
        tool_name: &str,
        input_params: &Value,
        output: Value,
        execution_time_ms: i32,
        ttl_seconds: usize,
    ) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let input_hash = Self::hash_input(input_params);
        let key = Self::cache_key(tool_name, &input_hash);

        let cached = CachedToolResult {
            output,
            execution_time_ms,
            cached_at: chrono::Utc::now().timestamp(),
            hit_count: 0,
        };

        let json = serde_json::to_string(&cached).unwrap();
        con.set_ex(&key, json, ttl_seconds).await?;

        Ok(())
    }

    pub async fn invalidate(
        &self,
        tool_name: &str,
        input_params: &Value,
    ) -> Result<(), RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let input_hash = Self::hash_input(input_params);
        let key = Self::cache_key(tool_name, &input_hash);

        con.del(&key).await?;
        Ok(())
    }

    pub async fn get_stats(&self, tool_name: &str) -> Result<CacheStats, RedisError> {
        let mut con = self.client.get_async_connection().await?;
        let pattern = format!("tool_cache:{}:*", tool_name);

        let keys: Vec<String> = con.keys(&pattern).await?;

        let mut total_hits = 0;
        let mut total_entries = keys.len();

        for key in &keys {
            if let Some(data) = con.get::<_, Option<String>>(key).await? {
                if let Ok(cached) = serde_json::from_str::<CachedToolResult>(&data) {
                    total_hits += cached.hit_count;
                }
            }
        }

        Ok(CacheStats {
            total_entries,
            total_hits,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_hits: usize,
}
```

- [ ] 4.1.2: Integrate cache with ToolExecutor

```rust
// Update apps/api/src/services/tool_executor.rs
impl ToolExecutor {
    pub async fn execute_with_cache(
        &self,
        tool_name: &str,
        input_params: Value,
        context: ExecutionContext,
        cache_ttl_seconds: Option<usize>,
    ) -> Result<ToolExecutionResult, ExecutorError> {
        let cache = SemanticToolCache::new(&self.redis_url)?;

        // Check cache first
        if let Some(cached) = cache.get(tool_name, &input_params).await? {
            tracing::info!("Cache hit for tool {} (hit count: {})", tool_name, cached.hit_count);

            return Ok(ToolExecutionResult {
                execution_id: Uuid::new_v4(), // Dummy ID for cached result
                status: ExecutionStatus::Succeeded,
                output: Some(cached.output),
                error: None,
                execution_time_ms: cached.execution_time_ms,
            });
        }

        // Cache miss - execute tool
        let result = self.execute(tool_name, input_params.clone(), context).await?;

        // Cache successful results
        if result.status == ExecutionStatus::Succeeded {
            if let Some(output) = &result.output {
                let ttl = cache_ttl_seconds.unwrap_or(3600); // Default 1 hour
                cache.set(tool_name, &input_params, output.clone(), result.execution_time_ms, ttl).await?;
            }
        }

        Ok(result)
    }
}
```

**Acceptance Criteria**:

- [ ] Cache stores tool results
- [ ] Cache retrieval working
- [ ] Cache hit count tracked
- [ ] TTL enforced
- [ ] Cache invalidation working
- [ ] Cache stats available

---

## Epic 5: Tool Performance Monitoring

### Task 5.1: Implement Tool Metrics

**Type**: Backend
**Dependencies**: Tool Executor from Epic 2

**Subtasks**:

- [ ] 5.1.1: Create materialized view for tool metrics

```sql
-- migrations/014_create_tool_metrics_view.sql
CREATE MATERIALIZED VIEW tool_performance_metrics AS
SELECT
    t.id as tool_id,
    t.name as tool_name,
    t.category,
    COUNT(*) as total_executions,
    COUNT(*) FILTER (WHERE te.status = 'succeeded') as successful_executions,
    COUNT(*) FILTER (WHERE te.status = 'failed') as failed_executions,
    COUNT(*) FILTER (WHERE te.status = 'timeout') as timeout_executions,
    ROUND(100.0 * COUNT(*) FILTER (WHERE te.status = 'succeeded') / NULLIF(COUNT(*), 0), 2) as success_rate_pct,
    AVG(te.execution_time_ms) FILTER (WHERE te.status = 'succeeded') as avg_execution_time_ms,
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY te.execution_time_ms) FILTER (WHERE te.status = 'succeeded') as p50_execution_time_ms,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY te.execution_time_ms) FILTER (WHERE te.status = 'succeeded') as p95_execution_time_ms,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY te.execution_time_ms) FILTER (WHERE te.status = 'succeeded') as p99_execution_time_ms,
    SUM(te.cost) as total_cost,
    AVG(te.cost) as avg_cost_per_execution,
    MAX(te.created_at) as last_execution_at
FROM tools t
LEFT JOIN tool_executions te ON t.id = te.tool_id
WHERE te.created_at > NOW() - INTERVAL '7 days'
GROUP BY t.id, t.name, t.category;

CREATE UNIQUE INDEX idx_tool_performance_metrics_tool_id ON tool_performance_metrics(tool_id);

-- Refresh function
CREATE OR REPLACE FUNCTION refresh_tool_metrics()
RETURNS void AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY tool_performance_metrics;
END;
$$ LANGUAGE plpgsql;
```

- [ ] 5.1.2: Create tool metrics service

```rust
// apps/api/src/services/tool_metrics.rs
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ToolPerformanceMetrics {
    pub tool_id: Uuid,
    pub tool_name: String,
    pub category: String,
    pub total_executions: i64,
    pub successful_executions: i64,
    pub failed_executions: i64,
    pub timeout_executions: i64,
    pub success_rate_pct: Option<f64>,
    pub avg_execution_time_ms: Option<f64>,
    pub p50_execution_time_ms: Option<f64>,
    pub p95_execution_time_ms: Option<f64>,
    pub p99_execution_time_ms: Option<f64>,
    pub total_cost: Option<rust_decimal::Decimal>,
    pub avg_cost_per_execution: Option<rust_decimal::Decimal>,
    pub last_execution_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct ToolMetricsService {
    db: PgPool,
}

impl ToolMetricsService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    pub async fn get_all_metrics(&self) -> Result<Vec<ToolPerformanceMetrics>, sqlx::Error> {
        let metrics = sqlx::query_as!(
            ToolPerformanceMetrics,
            r#"
            SELECT
                tool_id,
                tool_name,
                category::text as "category!",
                total_executions,
                successful_executions,
                failed_executions,
                timeout_executions,
                success_rate_pct,
                avg_execution_time_ms,
                p50_execution_time_ms,
                p95_execution_time_ms,
                p99_execution_time_ms,
                total_cost,
                avg_cost_per_execution,
                last_execution_at
            FROM tool_performance_metrics
            ORDER BY total_executions DESC
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(metrics)
    }

    pub async fn get_tool_metrics(&self, tool_id: Uuid) -> Result<Option<ToolPerformanceMetrics>, sqlx::Error> {
        let metrics = sqlx::query_as!(
            ToolPerformanceMetrics,
            r#"
            SELECT
                tool_id,
                tool_name,
                category::text as "category!",
                total_executions,
                successful_executions,
                failed_executions,
                timeout_executions,
                success_rate_pct,
                avg_execution_time_ms,
                p50_execution_time_ms,
                p95_execution_time_ms,
                p99_execution_time_ms,
                total_cost,
                avg_cost_per_execution,
                last_execution_at
            FROM tool_performance_metrics
            WHERE tool_id = $1
            "#,
            tool_id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(metrics)
    }

    pub async fn refresh_metrics(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("SELECT refresh_tool_metrics()")
            .execute(&self.db)
            .await?;

        Ok(())
    }

    pub async fn get_execution_history(
        &self,
        tool_id: Uuid,
        limit: i64,
    ) -> Result<Vec<ToolExecution>, sqlx::Error> {
        let executions = sqlx::query_as!(
            ToolExecution,
            r#"
            SELECT
                id, tool_id, task_id, team_id, member_id,
                status as "status: _",
                input_params,
                output_data,
                error_message,
                execution_time_ms,
                tokens_used,
                cost,
                retry_count,
                started_at,
                completed_at,
                created_at
            FROM tool_executions
            WHERE tool_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
            tool_id,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(executions)
    }
}
```

- [ ] 5.1.3: Create API endpoints for metrics

```rust
// apps/api/src/api/handlers/tool_metrics.rs
use axum::{extract::{Path, State}, http::StatusCode, Json};
use crate::services::tool_metrics::{ToolMetricsService, ToolPerformanceMetrics};

pub async fn get_all_tool_metrics(
    State(state): State<AppState>,
) -> Result<Json<Vec<ToolPerformanceMetrics>>, StatusCode> {
    let metrics_service = ToolMetricsService::new(state.db.clone());

    let metrics = metrics_service
        .get_all_metrics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(metrics))
}

pub async fn get_tool_metrics(
    State(state): State<AppState>,
    Path(tool_id): Path<Uuid>,
) -> Result<Json<ToolPerformanceMetrics>, StatusCode> {
    let metrics_service = ToolMetricsService::new(state.db.clone());

    let metrics = metrics_service
        .get_tool_metrics(tool_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(metrics))
}

pub async fn refresh_tool_metrics(
    State(state): State<AppState>,
) -> Result<StatusCode, StatusCode> {
    let metrics_service = ToolMetricsService::new(state.db.clone());

    metrics_service
        .refresh_metrics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::OK)
}
```

- [ ] 5.1.4: Set up metrics refresh cron job

```rust
// apps/api/src/jobs/metrics_refresh.rs
use tokio_cron_scheduler::{JobScheduler, Job};
use sqlx::PgPool;

pub async fn start_metrics_refresh_job(db: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = JobScheduler::new().await?;

    let job = Job::new_async("0 */5 * * * *", move |_uuid, _l| {
        let db = db.clone();
        Box::pin(async move {
            tracing::info!("Refreshing tool performance metrics");

            if let Err(e) = sqlx::query!("SELECT refresh_tool_metrics()")
                .execute(&db)
                .await
            {
                tracing::error!("Failed to refresh metrics: {}", e);
            }
        })
    })?;

    scheduler.add(job).await?;
    scheduler.start().await?;

    Ok(())
}
```

**Acceptance Criteria**:

- [ ] Metrics view created successfully
- [ ] Can retrieve all tool metrics
- [ ] Can retrieve metrics for specific tool
- [ ] Metrics refresh working
- [ ] Execution history retrievable
- [ ] Cron job refreshes metrics every 5 minutes
- [ ] P50/P95/P99 latencies calculated correctly

---

## Success Criteria - Phase 4 Complete

- [ ] Tool registry operational with 4+ tools
- [ ] Tool executor can run all tool types
- [ ] Input validation enforced
- [ ] Retry logic with exponential backoff working
- [ ] Fallback strategies functional
- [ ] Semantic caching reduces API calls
- [ ] Tool performance metrics available
- [ ] Rate limiting prevents abuse
- [ ] All tool executions tracked
- [ ] Cost tracking accurate

---

## Next Steps

Proceed to [08-phase-5-frontend-basics.md](./08-phase-5-frontend-basics.md) for Next.js frontend implementation.

---

**Phase 4: Autonomous Tool Execution Online**
