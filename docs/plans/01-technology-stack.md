# Technology Stack: Ghost Pirates

**Version**: 1.0
**Last Updated**: November 8, 2025
**Status**: Finalized for MVP

---

## Overview

This document provides comprehensive technology decisions for Ghost Pirates, following the "second mouse" positioning - proven enterprise technology adapted for autonomous AI teams, not bleeding edge.

---

## Backend Technology

### Rust + Axum Framework

**Choice**: Rust 1.75+ with Axum 0.7 web framework

**Rationale**:
- **Memory safety without GC**: Perfect for long-running agent processes
- **Fearless concurrency**: Essential for managing multiple agents simultaneously
- **Performance**: 10-100x faster than Python for agent orchestration
- **Type safety**: Protocol Buffers integration for frontend communication
- **Mature ecosystem**: Production-ready crates for all requirements

**Key Dependencies**:

```toml
[dependencies]
# Web Framework
axum = "0.7"
tokio = { version = "1.35", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "cors", "limit"] }

# Database
sqlx = { version = "0.7", features = ["postgres", "json", "uuid", "chrono", "runtime-tokio-native-tls"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# UUID & Time
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# HTTP Client (for LLM APIs)
reqwest = { version = "0.11", features = ["json"] }

# WebSocket
tokio-tungstenite = "0.21"

# Redis
redis = { version = "0.24", features = ["aio", "connection-manager"] }

# Environment
dotenv = "0.15"

# Logging & Tracing
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json"] }

# Security
jsonwebtoken = "9.2"
argon2 = "0.5"

# Error Handling
thiserror = "1.0"
anyhow = "1.0"
```

**Project Structure**:

```
ghostpirates-api/
├── src/
│   ├── main.rs                    # Application entry point
│   ├── config.rs                  # Configuration management
│   ├── domain/                    # Domain logic (DDD)
│   │   ├── mod.rs
│   │   ├── teams/                 # Team aggregate
│   │   │   ├── mod.rs
│   │   │   ├── team.rs            # Team entity
│   │   │   ├── member.rs          # Team member
│   │   │   └── lifecycle.rs       # Team state machine
│   │   ├── tasks/                 # Task aggregate
│   │   │   ├── mod.rs
│   │   │   ├── task.rs
│   │   │   ├── subtask.rs
│   │   │   └── revision.rs
│   │   └── agents/                # Agent aggregate
│   │       ├── mod.rs
│   │       ├── manager.rs
│   │       ├── worker.rs
│   │       └── specialization.rs
│   ├── application/               # Application services
│   │   ├── mod.rs
│   │   ├── team_service.rs
│   │   ├── task_service.rs
│   │   └── agent_service.rs
│   ├── infrastructure/            # External concerns
│   │   ├── mod.rs
│   │   ├── database/
│   │   │   ├── mod.rs
│   │   │   ├── pool.rs
│   │   │   └── repositories/
│   │   ├── llm/
│   │   │   ├── mod.rs
│   │   │   ├── claude.rs
│   │   │   └── openai.rs
│   │   ├── redis/
│   │   │   ├── mod.rs
│   │   │   └── cache.rs
│   │   └── tools/
│   │       ├── mod.rs
│   │       ├── registry.rs
│   │       └── executor.rs
│   ├── api/                       # API layer
│   │   ├── mod.rs
│   │   ├── routes.rs
│   │   ├── handlers/
│   │   │   ├── teams.rs
│   │   │   ├── tasks.rs
│   │   │   └── audit.rs
│   │   ├── middleware/
│   │   │   ├── auth.rs
│   │   │   └── logging.rs
│   │   └── websocket.rs
│   ├── orchestration/             # Multi-agent orchestration
│   │   ├── mod.rs
│   │   ├── team_orchestrator.rs
│   │   ├── task_orchestrator.rs
│   │   └── checkpointing.rs
│   └── errors.rs                  # Error types
├── migrations/                     # Database migrations
├── tests/                          # Integration tests
├── Cargo.toml
└── Dockerfile
```

---

## Database Layer

### PostgreSQL 15+

**Choice**: PostgreSQL 15 with pgvector extension

**Rationale**:
- **ACID compliance**: Critical for team state consistency
- **JSONB support**: Flexible metadata storage
- **pgvector extension**: Semantic search for agent memory
- **Row-Level Security**: Multi-user access control
- **Mature, reliable**: Battle-tested in production

**Schema Design Principles**:
- Normalized for consistency
- JSONB for flexible metadata
- UUIDs for distributed IDs
- Timestamptz for all dates
- Foreign key constraints enforced
- Indexes on query paths

**Key Extensions**:

```sql
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgvector";
```

**Core Tables**:

```sql
-- Companies/Workspaces
CREATE TABLE companies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Teams
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES companies(id),
    goal TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    manager_agent_id UUID,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    budget_limit DECIMAL(12,2),
    metadata JSONB,
    CONSTRAINT valid_status CHECK (status IN ('pending', 'planning', 'active', 'completed', 'failed', 'archived'))
);

-- Team Members (Agents in Team)
CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL,
    role VARCHAR(50) NOT NULL CHECK (role IN ('manager', 'worker')),
    specialization VARCHAR(100),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    current_workload INT NOT NULL DEFAULT 0,
    max_concurrent_tasks INT NOT NULL DEFAULT 3,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(team_id, agent_id)
);

-- Tasks
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    parent_task_id UUID REFERENCES tasks(id),
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    acceptance_criteria JSONB NOT NULL DEFAULT '[]',
    assigned_to UUID REFERENCES team_members(id),
    assigned_by UUID REFERENCES team_members(id),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    start_time TIMESTAMPTZ,
    completion_time TIMESTAMPTZ,
    revision_count INT NOT NULL DEFAULT 0,
    max_revisions INT NOT NULL DEFAULT 3,
    input_data JSONB,
    output_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'assigned', 'in_progress', 'review', 'completed', 'failed', 'revision_requested'))
);

-- Messages (Agent Communication + Audit)
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    from_agent_id UUID NOT NULL,
    to_agent_id UUID,
    message_type VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Checkpoints (for resumption)
CREATE TABLE checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    step_number INT NOT NULL,
    step_output JSONB NOT NULL,
    accumulated_context JSONB NOT NULL,
    token_count INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(task_id, step_number)
);

-- Cost Tracking
CREATE TABLE cost_tracking (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    category VARCHAR(50) NOT NULL,
    provider VARCHAR(50),
    model VARCHAR(100),
    amount DECIMAL(10,6) NOT NULL,
    unit_count INT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_teams_company_id ON teams(company_id);
CREATE INDEX idx_teams_status ON teams(status);
CREATE INDEX idx_tasks_team_id ON tasks(team_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_messages_team_id ON messages(team_id);
CREATE INDEX idx_cost_tracking_team_id ON cost_tracking(team_id);
```

### Redis 7+

**Choice**: Redis 7 for caching and coordination

**Use Cases**:
- **Session cache**: Active conversation context
- **Pub/Sub**: Real-time team updates
- **Task queue**: Worker task distribution
- **Rate limiting**: LLM API throttling
- **Semantic caching**: Response deduplication

**Redis Patterns**:

```rust
// Session caching
pub async fn cache_team_state(redis: &Redis, team_id: Uuid, state: &TeamState) -> Result<()> {
    let key = format!("team:{}:state", team_id);
    let json = serde_json::to_string(state)?;
    redis.setex(&key, 3600, &json).await?; // 1 hour TTL
    Ok(())
}

// Pub/Sub for real-time updates
pub async fn publish_team_update(redis: &Redis, team_id: Uuid, update: TeamUpdate) -> Result<()> {
    let channel = format!("team:{}", team_id);
    let json = serde_json::to_string(&update)?;
    redis.publish(&channel, &json).await?;
    Ok(())
}

// Task queue (reliable queue pattern)
pub async fn enqueue_task(redis: &Redis, task: WorkerTask) -> Result<()> {
    let json = serde_json::to_string(&task)?;
    redis.lpush("agent_tasks", &json).await?;
    Ok(())
}
```

---

## Frontend Technology

### Next.js 14 + React 18

**Choice**: Next.js 14 with App Router

**Rationale**:
- **Server-Side Rendering**: Better SEO, faster initial load
- **Edge deployment**: Low latency globally
- **React Server Components**: Reduced client bundle size
- **Built-in API routes**: Simplified architecture
- **TypeScript support**: Type safety across stack

**Key Dependencies**:

```json
{
  "dependencies": {
    "next": "14.0.4",
    "react": "18.2.0",
    "react-dom": "18.2.0",
    "typescript": "5.3.3",

    "@tanstack/react-query": "5.14.2",
    "zustand": "4.4.7",

    "tailwindcss": "3.4.0",
    "@radix-ui/react-accordion": "1.1.2",
    "@radix-ui/react-dialog": "1.0.5",

    "recharts": "2.10.3",
    "socket.io-client": "4.6.1",

    "zod": "3.22.4",
    "react-hook-form": "7.49.2"
  }
}
```

**Project Structure**:

```
ghostpirates-web/
├── app/
│   ├── layout.tsx                 # Root layout
│   ├── page.tsx                   # Home page
│   ├── dashboard/
│   │   ├── page.tsx               # Dashboard home
│   │   └── layout.tsx
│   ├── teams/
│   │   ├── create/
│   │   │   └── page.tsx           # Team creation wizard
│   │   └── [id]/
│   │       ├── page.tsx           # Team dashboard
│   │       └── audit/
│   │           └── page.tsx       # Audit trail
│   └── api/
│       ├── teams/
│       │   └── route.ts
│       └── auth/
│           └── route.ts
├── components/
│   ├── teams/
│   │   ├── TeamCreationForm.tsx
│   │   ├── TeamDashboard.tsx
│   │   ├── TaskList.tsx
│   │   └── TaskDetailModal.tsx
│   ├── audit/
│   │   ├── AuditTrail.tsx
│   │   └── EventList.tsx
│   └── ui/                        # Shadcn components
│       ├── button.tsx
│       ├── card.tsx
│       └── dialog.tsx
├── lib/
│   ├── api/
│   │   ├── client.ts              # API client
│   │   └── hooks.ts               # React Query hooks
│   ├── store/
│   │   └── team-store.ts          # Zustand store
│   └── utils.ts
└── public/
```

---

## AI/ML Technology

### Claude 3.5 Sonnet (Primary LLM)

**Choice**: Anthropic Claude 3.5 Sonnet via API

**Rationale**:
- **Superior reasoning**: Best for manager agent decisions
- **Long context window**: 200K tokens for complex tasks
- **Tool use capability**: Native function calling
- **Safety alignment**: Reduced harmful outputs
- **Structured outputs**: JSON mode support

**API Integration**:

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<Tool>>,
}

#[derive(Deserialize)]
pub struct ClaudeResponse {
    id: String,
    content: Vec<ContentBlock>,
    usage: Usage,
}

pub struct ClaudeClient {
    client: Client,
    api_key: String,
}

impl ClaudeClient {
    pub async fn complete(
        &self,
        messages: Vec<Message>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ClaudeResponse> {
        let request = ClaudeRequest {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 4096,
            messages,
            tools,
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?;

        let claude_response: ClaudeResponse = response.json().await?;
        Ok(claude_response)
    }
}
```

### GPT-4 (Fallback/Alternative)

**Choice**: OpenAI GPT-4 via API

**Use Cases**:
- **Fallback**: When Claude rate-limited
- **Alternative**: For specific tasks GPT-4 excels at
- **Cost optimization**: GPT-4-mini for simple tasks

**Model Routing Logic**:

```rust
pub enum LLMProvider {
    Claude,
    OpenAI,
}

pub struct LLMRouter {
    claude_client: ClaudeClient,
    openai_client: OpenAIClient,
}

impl LLMRouter {
    pub async fn complete(&self, task: &Task) -> Result<LLMResponse> {
        // Try primary provider (Claude)
        match self.claude_client.complete(task).await {
            Ok(response) => Ok(response),
            Err(e) if e.is_rate_limit() => {
                // Fallback to OpenAI on rate limit
                self.openai_client.complete(task).await
            }
            Err(e) => Err(e),
        }
    }
}
```

---

## Infrastructure & DevOps

### Azure Kubernetes Service (AKS)

**Choice**: Azure Kubernetes Service for container orchestration

**Rationale**:
- **Auto-scaling**: Handle variable load
- **Self-healing**: Automatic pod restarts
- **Zero-downtime deployments**: Rolling updates
- **Resource isolation**: Teams don't affect each other
- **Managed service**: Reduced operational overhead

**Deployment Configuration**:

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ghostpirates-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ghostpirates-api
  template:
    metadata:
      labels:
        app: ghostpirates-api
    spec:
      containers:
      - name: api
        image: ghostpirates.azurecr.io/api:latest
        ports:
        - containerPort: 4000
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: db-credentials
              key: url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-credentials
              key: url
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 4000
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health
            port: 4000
          initialDelaySeconds: 5
          periodSeconds: 10
```

### Azure Database for PostgreSQL Flexible Server

**Configuration**:
- **Tier**: General Purpose
- **vCores**: 4
- **Storage**: 128 GB SSD
- **High Availability**: Zone-redundant
- **Backups**: Automated, 7-day retention
- **Connection pooling**: PgBouncer

### Azure Managed Redis

**Configuration**:
- **Tier**: Standard
- **Capacity**: C1 (1 GB)
- **Clustering**: Disabled (MVP)
- **Persistence**: AOF enabled
- **Zone redundancy**: Enabled

### GitHub Actions (CI/CD)

**Pipeline Stages**:
1. **Test**: Run unit + integration tests
2. **Build**: Compile Rust, build Docker images
3. **Push**: Upload to Azure Container Registry
4. **Deploy**: Update AKS deployment

```yaml
# .github/workflows/deploy.yml
name: Deploy to Production

on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/login-action@v3
        with:
          registry: ghostpirates.azurecr.io
          username: ${{ secrets.ACR_USERNAME }}
          password: ${{ secrets.ACR_PASSWORD }}
      - run: docker build -t ghostpirates.azurecr.io/api:${{ github.sha }} .
      - run: docker push ghostpirates.azurecr.io/api:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: azure/k8s-set-context@v3
        with:
          kubeconfig: ${{ secrets.KUBE_CONFIG }}
      - run: kubectl set image deployment/ghostpirates-api api=ghostpirates.azurecr.io/api:${{ github.sha }}
```

---

## Development Tools

### Testing

- **Rust**: `cargo test` (unit + integration)
- **Frontend**: Jest + React Testing Library
- **E2E**: Playwright
- **Load testing**: k6

### Code Quality

- **Rust**: `cargo fmt`, `cargo clippy`
- **TypeScript**: ESLint, Prettier
- **Git hooks**: Husky + lint-staged

### Monitoring

- **Logs**: Azure Application Insights
- **Metrics**: Prometheus + Grafana
- **Tracing**: OpenTelemetry
- **Errors**: Sentry

---

## Decision Log

| Decision | Date | Rationale |
|----------|------|-----------|
| Rust backend | Nov 2025 | Memory safety + performance for agents |
| PostgreSQL + pgvector | Nov 2025 | ACID + semantic search for memory |
| Next.js 14 | Nov 2025 | SSR + edge deployment |
| Claude primary | Nov 2025 | Superior reasoning for managers |
| AKS | Nov 2025 | Managed K8s with auto-scaling |

---

## Next Steps

1. **Proceed to [02-infrastructure-setup.md](./02-infrastructure-setup.md)** for Azure setup
2. **Review [03-database-architecture.md](./03-database-architecture.md)** for schema details
3. **Begin implementation** with Phase 1

---

**Technology Stack: Production-Ready, Battle-Tested, Pragmatic 🚀**
