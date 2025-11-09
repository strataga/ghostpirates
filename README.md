# Ghost Pirates 🏴‍☠️👻

**Ephemeral AI Teams for Mission-Based Work**

Deploy autonomous AI teams that form, execute, and dissolve on demand. Each team consists of a Manager agent coordinating specialized Worker agents to complete complex missions through hierarchical task decomposition and quality review loops.

---

## 🎯 Quick Start

```bash
# Clone the repository
git clone https://github.com/strataga/ghostpirates.git
cd ghostpirates

# Review the documentation
cat CLAUDE.md                                    # Development guide for Claude Code
cat docs/plans/README.md                         # Implementation plan index
cat docs/patterns/16-Pattern-Integration-Guide.md  # Architecture patterns

# Install dependencies
pnpm install

# Set up local environment
pnpm docker:up            # Start PostgreSQL + Redis
cp apps/api/.env.example apps/api/.env
# Edit apps/api/.env with your database URL

# Run everything in dev mode
pnpm dev

# Or run individually:
pnpm api:dev              # Rust backend only
turbo run dev --filter=web  # Next.js frontend only
```

---

## 📚 Documentation Structure

### **For Developers (Start Here)**
1. **[CLAUDE.md](./CLAUDE.md)** - Complete developer guide for working in this codebase
2. **[docs/plans/README.md](./docs/plans/README.md)** - Master index for 20 detailed implementation plans
3. **[docs/patterns/16-Pattern-Integration-Guide.md](./docs/patterns/16-Pattern-Integration-Guide.md)** - Architectural pattern guide

### **Implementation Plans** (`/docs/plans/`)
Detailed, actionable plans with exact commands and code examples:
- `00-project-overview.md` - Vision, goals, success metrics
- `01-technology-stack.md` - Tech decisions (Rust, PostgreSQL, Next.js, Azure)
- `02-infrastructure-setup.md` - Terraform, Kubernetes, CI/CD
- `03-database-architecture.md` - Schema design, migrations, indexes
- `04-phase-1-foundation.md` → `11-phase-8-testing-polish.md` - 8 implementation phases
- `12-cost-optimization.md` → `18-success-metrics.md` - Operations & business

**Total:** 570KB of detailed implementation guidance with 100+ code examples

### **Architectural Patterns** (`/docs/patterns/`)
101 comprehensive pattern documents covering:
- **Core:** Hexagonal Architecture, DDD, CQRS, Repository Pattern
- **Resilience:** Circuit Breaker, Retry Pattern, Bulkhead
- **Multi-tenancy:** Database-per-tenant isolation strategies
- **Authorization:** RBAC with CASL permissions
- **Integration Guide:** How to choose and combine patterns

### **Research & Design** (`/docs/research/`)
Original research and design documents:
- `GHOST_PIRATES_PROJECT_PLAN.md` - Comprehensive 100-page system design
- `technical-architecture-and-business-operations.md` - AI agent patterns and economics
- `ENGINEERING_CHECKLIST.md` - Sprint-by-sprint implementation checklist
- `README_MASTER_INDEX.md` - Navigation guide

### **Sprints** (`/docs/sprints/`)
Detailed sprint documents with task breakdowns:
- `TEMPLATE.md` - Sprint document template
- `sprint-1-foundation.md` - Database, domain layer, and auth (5 user stories, 180+ tasks)

---

## 🏗️ System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    GHOST PIRATES PLATFORM                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────┐                                            │
│  │  Next.js UI  │  Team creation wizard, dashboard, audit    │
│  └────────┬─────┘                                            │
│           │ REST API + WebSocket                             │
│           ▼                                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │         Rust Backend (Axum + Tokio)                    │  │
│  │                                                         │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │ Team Manager │  │ Task         │  │ Tool        │  │  │
│  │  │              │  │ Orchestrator │  │ Executor    │  │  │
│  │  └──────────────┘  └──────────────┘  └─────────────┘  │  │
│  │                                                         │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐  │  │
│  │  │ Agent        │  │ Checkpoint   │  │ Error       │  │  │
│  │  │ Runtime      │  │ Manager      │  │ Recovery    │  │  │
│  │  └──────────────┘  └──────────────┘  └─────────────┘  │  │
│  └─────────────────────────────────────────────────────────┘  │
│           │                                                   │
│           ▼                                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  PostgreSQL + Redis                                    │  │
│  │  - Teams, Tasks, Agents, Messages, Checkpoints         │  │
│  │  - Cost Tracking, Audit Logs                           │  │
│  │  - pgvector for semantic caching                       │  │
│  └────────────────────────────────────────────────────────┘  │
│           │                                                   │
│           ▼                                                   │
│  ┌────────────────────────────────────────────────────────┐  │
│  │  LLM APIs                                              │  │
│  │  - Claude 3.5 Sonnet (Manager agents)                  │  │
│  │  - GPT-4 (Worker agents)                               │  │
│  └────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Ephemeral Teams** - AI teams form on-demand, execute missions, and dissolve when complete
2. **Hierarchical Organization** - Manager agent leads specialized worker agents
3. **Quality Feedback Loops** - Manager reviews work and requests revisions until standards are met
4. **Checkpoint-Based Recovery** - Resume from last successful step on failure (saves token costs)
5. **Complete Transparency** - Full audit trail of all decisions and communications
6. **Cost Optimization** - Semantic caching, model routing, prompt compression (60%+ savings)

---

## 🛠️ Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Backend** | Rust + Axum + Tokio | Type-safe, performant, async-first API |
| **Frontend** | Next.js 15 + React 19 | SSR, streaming, server components |
| **Database** | PostgreSQL + pgvector | ACID + semantic search for caching |
| **Cache/Queue** | Redis (Streams + Pub/Sub) | Task queues + real-time updates |
| **AI Models** | Claude 3.5 Sonnet + GPT-4 | Manager agents + worker agents |
| **Deployment** | Kubernetes (Azure AKS) | Auto-scaling, zero-downtime deploys |
| **Observability** | Prometheus + Grafana | Metrics, dashboards, alerts |

See [docs/plans/01-technology-stack.md](./docs/plans/01-technology-stack.md) for detailed tech decisions.

---

## 📋 Implementation Roadmap

### Phase 1-8: MVP (16 weeks)

| Phase | Weeks | Focus | Status |
|-------|-------|-------|--------|
| **Phase 1** | 1-2 | Database + API + Auth | 🔴 Not Started |
| **Phase 2** | 3-4 | Manager/Worker Agents | 🔴 Not Started |
| **Phase 3** | 5-6 | Task Orchestration | 🔴 Not Started |
| **Phase 4** | 7-8 | Tool Execution System | 🔴 Not Started |
| **Phase 5** | 9-10 | Frontend (Team Creation, Dashboard) | 🔴 Not Started |
| **Phase 6** | 11-12 | Real-time (WebSocket, Audit Trail) | 🔴 Not Started |
| **Phase 7** | 13-14 | Error Recovery (Checkpoints) | 🔴 Not Started |
| **Phase 8** | 15-16 | Testing, Polish, Deployment | 🔴 Not Started |

### Operations (Ongoing)

- **Cost Optimization** - Semantic caching, model routing, token tracking
- **Security** - Prompt injection prevention, RBAC, secrets management
- **Monitoring** - Prometheus metrics, distributed tracing, alerts
- **Testing** - Unit, integration, E2E, chaos engineering

See [docs/plans/README.md](./docs/plans/README.md) for complete roadmap.

---

## 🎯 Key Features

### Autonomous Team Formation
Users describe a goal → Manager agent analyzes it → Creates specialized worker agents → Decomposes goal into tasks → Assigns tasks based on skill matching

### Quality Review Loops
Workers complete tasks → Manager reviews against acceptance criteria → Requests revisions if needed → Approves when quality standards are met

### Checkpoint-Based Recovery
System creates checkpoints at each step → On failure, resumes from last checkpoint → Saves 30-70% on token costs → Prevents re-execution waste

### Complete Audit Trail
Every decision logged → Reasoning visible → Communications tracked → Costs transparent → Perfect for compliance and debugging

### Cost Optimization
- **Semantic Caching**: 15-30% cost reduction via Redis-backed similarity search
- **Model Routing**: Use GPT-3.5 for simple tasks, GPT-4 for complex reasoning
- **Prompt Compression**: 20%+ token reduction without quality loss
- **RAG for Context**: 70%+ token savings by retrieving only relevant chunks

---

## 🧪 Common Commands

```bash
# Development
pnpm dev                  # Run all apps in dev mode
pnpm build                # Build all apps
pnpm lint                 # Lint all code
pnpm type-check           # TypeScript type checking
pnpm format               # Format code with Prettier
pnpm check-all            # Run all quality checks

# Rust API
pnpm api:dev              # Run API in dev mode
pnpm api:test             # Run Rust tests
pnpm api:fmt              # Check Rust formatting
pnpm api:clippy           # Run Rust linter
pnpm api:db:reset         # Reset database with migrations

# Docker
pnpm docker:up            # Start PostgreSQL + Redis
pnpm docker:down          # Stop all containers
pnpm docker:logs          # View container logs
```

## 🧪 Testing Strategy

```bash
# Rust tests
pnpm api:test             # All Rust tests
cd apps/api && cargo test --lib  # Unit tests only

# TypeScript tests (coming soon)
pnpm test                 # All tests via Turbo

# Load testing (k6)
k6 run tests/load/team-creation.js

# Coverage
cd apps/api && cargo tarpaulin --out Html
```

Target: 80%+ coverage for critical paths

---

## 📊 Success Metrics

### Technical KPIs
- **Team Success Rate**: >75% on first attempt
- **API Latency**: P50 <500ms, P95 <2s, P99 <5s
- **Error Recovery**: >85% graceful recovery without human intervention
- **Cost Accuracy**: ±5% prediction vs actual

### Business Metrics
- **Monthly Active Users**: 100 (Month 1) → 10,000 (Month 12)
- **Revenue**: $2.5K MRR (Month 1) → $250K MRR (Month 12)
- **ARPU**: $25/month average
- **Churn**: <5% monthly
- **NPS**: >40

See [docs/plans/18-success-metrics.md](./docs/plans/18-success-metrics.md) for complete metrics.

---

## 🔒 Security

- **Authentication**: JWT tokens with 8-hour expiry + refresh tokens
- **Authorization**: RBAC with fine-grained permissions (see `/docs/patterns/01-RBAC-CASL-Pattern.md`)
- **Prompt Injection Prevention**: Input sanitization + output validation
- **Rate Limiting**: Per-user and per-IP limits via Redis
- **Secrets Management**: Azure Key Vault (never commit secrets)
- **Audit Logging**: All mutations logged with before/after values

---

## 💰 Cost Management

LLM API costs are 60-80% of total expenses. **Cost optimization is critical.**

**Optimization Strategies:**
1. **Semantic Caching** - Store similar queries in Redis (pgvector similarity search)
2. **Model Routing** - Use cheaper models for simple tasks, expensive for complex
3. **Prompt Compression** - Reduce tokens while maintaining quality
4. **Checkpoint Resumption** - Don't re-execute completed steps on failure
5. **Budget Enforcement** - Hard limits per team/user to prevent runaway costs

See [docs/plans/12-cost-optimization.md](./docs/plans/12-cost-optimization.md) for implementation details.

---

## 📡 API Documentation

### Base URL
```
http://localhost:3000
```

### Authentication Endpoints

#### Register New User
```http
POST /api/auth/register
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword123",
  "full_name": "John Doe",
  "company_id": "uuid-here"
}
```

**Response (201 Created):**
```json
{
  "user_id": "uuid-here",
  "message": "User registered successfully"
}
```

#### Login
```http
POST /api/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "securepassword123"
}
```

**Response (200 OK):**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user_id": "uuid-here"
}
```

### Team Endpoints

#### Create Team
```http
POST /api/teams
Content-Type: application/json

{
  "goal": "Build a new feature",
  "company_id": "uuid-here",
  "created_by": "uuid-here",
  "budget_limit": 100.00
}
```

**Response (201 Created):**
```json
{
  "id": "team-uuid",
  "company_id": "uuid-here",
  "goal": "Build a new feature",
  "status": "Pending",
  "created_by": "uuid-here",
  "budget_limit": 100.00
}
```

#### Get Team by ID
```http
GET /api/teams/:id
```

**Response (200 OK):**
```json
{
  "id": "team-uuid",
  "company_id": "uuid-here",
  "goal": "Build a new feature",
  "status": "Active",
  "created_by": "uuid-here",
  "budget_limit": 100.00
}
```

#### List Teams for Company
```http
GET /api/teams/company/:company_id
```

**Response (200 OK):**
```json
[
  {
    "id": "team-uuid-1",
    "company_id": "uuid-here",
    "goal": "Build feature A",
    "status": "Active",
    "created_by": "uuid-here",
    "budget_limit": 100.00
  },
  {
    "id": "team-uuid-2",
    "company_id": "uuid-here",
    "goal": "Build feature B",
    "status": "Completed",
    "created_by": "uuid-here",
    "budget_limit": 150.00
  }
]
```

#### Delete Team
```http
DELETE /api/teams/:id
```

**Response (204 No Content)**

### Health Check

#### Server Health
```http
GET /health
```

**Response (200 OK):**
```
OK
```

### Error Responses

All endpoints return structured JSON errors:

```json
{
  "error": "Email already registered"
}
```

**HTTP Status Codes:**
- `200 OK` - Successful GET request
- `201 Created` - Successful POST request
- `204 No Content` - Successful DELETE request
- `400 Bad Request` - Invalid input
- `401 Unauthorized` - Authentication failed
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

---

## 🤝 Contributing

This project is currently in active development. Once the MVP is complete, we'll open for contributions.

### Development Workflow

1. Review [CLAUDE.md](./CLAUDE.md) for complete developer guide
2. Read relevant patterns from `/docs/patterns/` before implementing features
3. Follow the sprint plans in `/docs/plans/`
4. All code must pass `cargo clippy`, `cargo fmt --check`, and have 80%+ test coverage
5. PRs require approval from 2+ engineers

---

## 📞 Support

- **Documentation**: Start with [CLAUDE.md](./CLAUDE.md)
- **Patterns**: See `/docs/patterns/16-Pattern-Integration-Guide.md`
- **Issues**: [GitHub Issues](https://github.com/strataga/ghostpirates/issues)
- **Discussions**: [GitHub Discussions](https://github.com/strataga/ghostpirates/discussions)

---

## 📜 License

[License TBD]

---

## 🙏 Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Next.js](https://nextjs.org/) - React framework
- [PostgreSQL](https://www.postgresql.org/) - Relational database
- [Redis](https://redis.io/) - In-memory data store
- [Claude](https://www.anthropic.com/) & [GPT-4](https://openai.com/) - LLM providers

---

**Let's build autonomous AI teams that actually work.** 🏴‍☠️👻
