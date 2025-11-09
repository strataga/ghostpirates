# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Ghost Pirates** is a SaaS platform for ephemeral AI team orchestration. Users define project goals via natural language, and the system autonomously creates specialized AI teams (manager + workers) that execute missions and dissolve upon completion.

**Tech Stack:**
- Backend: Rust + Axum + Tokio
- Frontend: Next.js 14 + React 18 + Tailwind CSS
- Database: PostgreSQL (with JSONB for flexible schemas)
- Cache/Queue: Redis (Streams for task queuing, pub/sub for real-time)
- AI Models: Claude 3.5 Sonnet (managers), GPT-4 (workers)
- Deployment: Kubernetes (AWS ECS or managed K8s)

**Current Status:** Documentation complete, ready for development (Week 0)

## Architecture

### Core Concepts

1. **Ephemeral Teams**: AI teams are created on-demand, operate in isolation, and dissolve after mission completion
2. **Hierarchical Organization**: Manager agent leads specialized worker agents
3. **Autonomous Execution**: Manager decomposes goals → assigns tasks → workers execute → manager reviews
4. **Quality Feedback Loop**: Manager reviews work and requests revisions until standards met
5. **Full Transparency**: Complete audit trail of all decisions and communications

### System Components

```
Frontend (Next.js) → REST API + WebSocket → Backend (Rust)
                                                ↓
                        ┌───────────────────────────────────┐
                        │  Team Manager (formation/lifecycle) │
                        │  Task Orchestrator (decomposition)  │
                        │  Agent Runtime (manager + workers)  │
                        │  Tool Execution System              │
                        │  Memory System (learning/context)   │
                        │  Error Recovery (checkpointing)     │
                        └───────────────────────────────────┘
                                        ↓
                        PostgreSQL + Redis + LLM APIs
```

### Key Backend Structure

```
ghost-pirates-api/
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── db/                    # Database pool, migrations
│   ├── models/                # Team, Agent, Task, Message
│   ├── handlers/              # API endpoint handlers
│   ├── agents/                # Manager/Worker agent logic
│   │   ├── manager.rs         # Goal analysis, team formation, review
│   │   ├── worker.rs          # Task execution
│   │   ├── runtime.rs         # Agent execution runtime
│   │   └── memory.rs          # Context & learning
│   ├── orchestration/         # Team & task orchestration
│   │   ├── team_orchestrator.rs
│   │   ├── task_orchestrator.rs
│   │   ├── checkpointing.rs
│   │   └── failure_handling.rs
│   ├── tools/                 # Tool registry & execution
│   ├── api/                   # REST + WebSocket
│   ├── auth/                  # JWT authentication
│   └── observability/         # Logging, metrics, tracing
```

### Database Schema Highlights

- **teams**: Goal, status, manager_agent_id, budget_limit, metadata (JSONB)
- **team_members**: Links agents to teams with roles (manager/worker)
- **tasks**: Hierarchical structure with parent_task_id, acceptance_criteria, revision tracking
- **messages**: Agent communication audit trail
- **checkpoints**: For task resumption on failure
- **cost_tracking**: Real-time API cost monitoring per team

## Development Commands

### Backend (Rust)

```bash
# Local development
cargo run

# Run tests
cargo test --all

# Database migrations
sqlx migrate add -r <migration_name>
sqlx migrate run --database-url $DATABASE_URL

# Code quality
cargo fmt --check
cargo clippy -- -D warnings

# Build Docker image
docker build -t ghostpirates-api:latest .
```

### Frontend (Next.js)

```bash
# Development server
npm run dev

# Build production
npm run build

# Linting
npm run lint

# Tests
npm run test
```

### Local Environment Setup

```bash
# Start local services (PostgreSQL + Redis)
docker-compose up -d

# Set up environment variables
cp .env.example .env.local
# Edit .env.local with API keys and database URLs

# Run backend
cd ghost-pirates-api && cargo run

# Run frontend (separate terminal)
cd ghost-pirates-web && npm run dev
```

## Architectural Patterns

**CRITICAL**: Ghost Pirates uses established software engineering patterns documented in `/docs/patterns/`. **Always consult the relevant pattern documentation before implementing features.**

### Pattern Catalog (101 Patterns Available)

The `/docs/patterns/` directory contains comprehensive pattern documentation covering:

**Core Architectural Patterns:**
- **Hexagonal Architecture** (`03-Hexagonal-Architecture.md`) - ALWAYS use this for all features
- **Domain-Driven Design** (`04-Domain-Driven-Design.md`) - For complex business logic
- **CQRS Pattern** (`05-CQRS-Pattern.md`) - Separate read/write models
- **Repository Pattern** (`06-Repository-Pattern.md`) - Data access abstraction
- **Event Sourcing** - For audit trails and temporal queries

**Essential Patterns:**
- **Multi-Tenancy** (`17-Multi-Tenancy-Pattern.md`) - Database-per-tenant isolation
- **RBAC with CASL** (`01-RBAC-CASL-Pattern.md`) - Authorization and permissions
- **Circuit Breaker** (`13-Circuit-Breaker-Pattern.md`) - Resilience for LLM API calls
- **Retry Pattern** (`15-Retry-Pattern.md`) - Transient failure handling
- **Strategy Pattern** (`10-Strategy-Pattern.md`) - Dynamic behavior (tool selection)
- **Factory Pattern** (`11-Factory-Pattern.md`) - Agent creation
- **Observer Pattern** (`12-Observer-Pattern.md`) - Event-driven updates

**How to Use Patterns:**
1. Read **Pattern Integration Guide** (`16-Pattern-Integration-Guide.md`) first
2. Identify which patterns apply to your feature
3. Follow the pattern's implementation guidelines exactly
4. Reference pattern examples in your code comments

### Agent System Implementation Patterns

**Manager Agent Responsibilities:**
- Analyze user goals and decompose into tasks (uses **Strategy Pattern** for different goal types)
- Determine required specializations and create workers (uses **Factory Pattern**)
- Assign tasks based on skill matching (uses **Specification Pattern**)
- Review worker outputs against acceptance criteria (uses **Chain of Responsibility**)
- Request revisions or approve completion (uses **State Machine Pattern**)

**Worker Agent Responsibilities:**
- Execute assigned tasks using available tools (uses **Strategy Pattern** for tool selection)
- Report results to manager with evidence (uses **Observer Pattern** for notifications)
- Handle revisions based on manager feedback (uses **Command Pattern**)

### Error Recovery

The system uses **checkpoint-based resumption** to handle failures gracefully:
- Checkpoints created at each task step
- On failure, resume from last successful checkpoint
- Saves token costs and prevents re-execution
- Retry with exponential backoff for transient errors
- Escalate to human after max retry attempts

### Tool Execution

Tools are registered with:
- Name, description, parameter schema
- Execution logic (async)
- Fallback strategies for failures

Workers select tools based on task requirements using LLM reasoning.

### Real-Time Updates

WebSocket connections at `/ws/teams/{team_id}` provide:
- Task status changes
- Worker progress updates
- Manager review decisions
- Cost accumulation
- Team lifecycle events

## Cost Management

**Critical**: LLM API costs are the primary expense (60-80% of total)

**Optimization strategies:**
- Model routing: Use GPT-4o-mini for simple tasks, GPT-4 for complex reasoning
- Semantic caching: Cache similar queries in Redis (15-30% savings)
- Prompt compression: Minimize input tokens
- RAG for documents: Only retrieve relevant chunks (70%+ token reduction)
- Checkpoint resumption: Avoid re-executing completed steps

## Testing Strategy

- **Unit tests**: Individual agent logic, tool implementations
- **Integration tests**: Multi-agent coordination, API endpoints
- **Contract tests**: Data exchange schemas between components
- **Chaos engineering**: Simulate tool failures, rate limits, timeouts

## Key Files to Reference

### Implementation Plans & Roadmaps
- `/docs/plans/README.md`: Master index for all 20 phased implementation plans
- `/docs/plans/00-project-overview.md`: Vision, goals, market opportunity, success metrics
- `/docs/plans/04-phase-1-foundation.md`: First sprint (database, API, auth) - **START HERE**
- `/docs/plans/05-phase-2-agent-system.md`: Agent implementation (manager + workers)

### Research & Architecture
- `/docs/research/GHOST_PIRATES_PROJECT_PLAN.md`: Complete system design (100 pages)
- `/docs/research/technical-architecture-and-business-operations.md`: AI agent architecture patterns
- `/docs/research/ENGINEERING_CHECKLIST.md`: Sprint-by-sprint implementation roadmap
- `/docs/research/README_MASTER_INDEX.md`: Navigation guide for all documentation

### Patterns & Best Practices (MUST READ)
- `/docs/patterns/16-Pattern-Integration-Guide.md`: **READ FIRST** - How to choose patterns
- `/docs/patterns/03-Hexagonal-Architecture.md`: Core architecture (REQUIRED for all features)
- `/docs/patterns/04-Domain-Driven-Design.md`: Domain modeling, entities, value objects
- `/docs/patterns/06-Repository-Pattern.md`: Data access patterns
- `/docs/patterns/13-Circuit-Breaker-Pattern.md`: Resilience for external APIs (LLMs)
- `/docs/patterns/17-Multi-Tenancy-Pattern.md`: Tenant isolation strategies

### Sprint Templates
- `/docs/sprints/TEMPLATE.md`: Template for creating detailed sprint documents

## Development Workflow

### Sprint Structure (from ENGINEERING_CHECKLIST.md)

- **Sprint 1 (Weeks 1-2)**: Database schema + API foundation + auth
- **Sprint 2 (Weeks 3-4)**: Manager/Worker agent implementation
- **Sprint 3 (Weeks 5-6)**: Task orchestration + assignment algorithms
- **Sprint 4 (Weeks 7-8)**: Tool execution system + fallbacks
- **Sprint 5 (Weeks 9-10)**: Frontend team creation + dashboard
- **Sprint 6 (Weeks 11-12)**: Real-time features + WebSocket + audit trail
- **Sprint 7 (Weeks 13-14)**: Error recovery + checkpointing
- **Sprint 8 (Weeks 15-16)**: Testing, polish, deployment prep

### Definition of Done

A task is complete when:
- [ ] Code implemented and tested
- [ ] Unit tests passing (>80% coverage for critical paths)
- [ ] Integration tests passing
- [ ] Error handling implemented
- [ ] Logging/observability added
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Deployed to staging and verified

## Common Gotchas

1. **Agent State Management**: Agents must persist context across sessions using PostgreSQL + Redis
2. **Token Cost Explosions**: Always implement semantic caching and cost tracking before deploying
3. **Checkpoint Granularity**: Too frequent = storage bloat; too sparse = wasted work on failure
4. **WebSocket Scaling**: Use Redis pub/sub for multi-instance deployments
5. **Task Dependencies**: Track parent_task_id carefully to prevent circular dependencies
6. **Revision Loops**: Set max_revisions (default: 3) to prevent infinite loops

## Key Dependencies (Rust)

```toml
axum = "0.7"                    # Web framework
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "json", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
redis = { version = "0.24", features = ["aio"] }
reqwest = { version = "0.11", features = ["json"] }
tracing = "0.1"
jsonwebtoken = "9.2"
```

## Observability

**Metrics to Track:**
- Team creation rate
- Task success/failure rates
- API latency (P50, P99)
- Token consumption per team/task
- Revision frequency
- Cost per mission
- Agent-specific performance

**Logging:**
- Structured JSON via `tracing-subscriber`
- Correlation IDs for request tracing
- All agent decisions with rationale

**Monitoring:**
- Prometheus for metrics
- Grafana for dashboards
- ELK stack for log aggregation

## Security Considerations

- JWT-based authentication with refresh tokens
- Database connection pooling with `sqlx`
- Rate limiting on API endpoints
- Input validation for all LLM prompts (prevent injection)
- Budget limits per team to prevent runaway costs
- Secrets management via environment variables (never commit `.env`)

## Questions?

Refer to the comprehensive documentation in `/docs/research/`:
- Architecture questions → `GHOST_PIRATES_PROJECT_PLAN.md` (Section 3, 10)
- Implementation details → `ENGINEERING_CHECKLIST.md`
- Business context → `EXECUTIVE_SUMMARY.md`
- Technical patterns → `technical-architecture-and-business-operations.md`
