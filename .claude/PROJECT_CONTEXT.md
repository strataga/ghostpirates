# Ghost Pirates Project Context

## Project Identity

**Name**: Ghost Pirates
**Tagline**: Autonomous AI Teams for Knowledge Work
**Type**: Enterprise SaaS Platform
**Target Market**: Companies needing autonomous AI teams to augment human capabilities
**Status**: Sprint 1 Complete (Foundation), Sprint 2 In Progress (Agent System)

## Core Value Proposition

Build autonomous AI teams that can:

- **Execute complex workflows** autonomously with minimal human oversight
- **Collaborate as a team** with specialized agents working together
- **Learn and improve** over time through feedback loops
- **Integrate seamlessly** with existing tools and systems
- **Scale effortlessly** from small tasks to enterprise-wide operations

## Business Model

**Multi-Tenant SaaS**:
- Companies create teams with specific goals and budgets
- Teams are composed of autonomous AI agents
- Agents execute tasks, track costs, and report progress
- Built-in cost tracking and budget management

## Technical Architecture

### Technology Stack

**Backend**: Rust (Axum 0.7, SQLx 0.7, Tokio 1)
**Frontend**: Next.js 14 + React 18 + Tailwind CSS (planned)
**Database**: PostgreSQL 15 with multi-tenant isolation (company_id)
**Authentication**: JWT with BCrypt password hashing (8-hour token expiry)
**Infrastructure**: Docker Compose (local), Azure (production via Terraform)

### Key Architectural Decisions

1. **Hexagonal Architecture** - Domain logic independent of infrastructure
2. **Domain-Driven Design** - Entities, value objects, aggregates, domain events
3. **Repository Pattern** - Clean data access abstraction with traits
4. **Multi-tenant with company_id** - Tenant isolation via company_id foreign keys
5. **Event-Driven** - Domain events for agent actions and state changes
6. **Cost Tracking** - Built-in budget management and cost monitoring

### Domain Structure

Core domains:
- **Users & Companies** - Multi-tenant user management
- **Teams** - AI teams with goals, budgets, and status tracking
- **Agents** - AI agents with capabilities and roles (Sprint 2+)
- **Tasks** - Work items managed by agents
- **Messages** - Communication between agents and with humans
- **Checkpoints** - Team progress snapshots
- **Cost Tracking** - Budget monitoring and forecasting

## Database Schema

8 migrations completed:
1. Companies table (multi-tenant root)
2. Users table (authentication, last_login tracking)
3. Teams table (goals, budgets, status, created_by)
4. Team members table (user-team relationships)
5. Tasks table (work items, status, assignments)
6. Messages table (agent/user communication)
7. Checkpoints table (progress tracking)
8. Cost tracking table (budget monitoring)

All tables have:
- UUID primary keys
- Timestamps (created_at, updated_at where applicable)
- Foreign keys with CASCADE deletes for tenant isolation

## Implementation Sprints

**Sprint 1**: Foundation (✅ COMPLETE)
- PostgreSQL schema (8 migrations)
- Domain models (Team, User, value objects)
- Repository layer (PostgresTeamRepository, PostgresUserRepository)
- REST API (Axum handlers for auth and teams)
- JWT authentication middleware
- Integration tests (9 repository tests, 7 E2E API tests)
- Code quality (cargo fmt, clippy, audit)

**Sprint 2**: Agent System (🚧 IN PROGRESS)
- Agent architecture and capabilities
- LLM integration (Claude API)
- Tool execution framework
- Memory and context management
- Autonomous task execution
- Inter-agent communication

**Future Sprints**:
- Advanced agent workflows
- Web interface (Next.js)
- Real-time updates (WebSockets)
- Advanced analytics and reporting
- Marketplace for agent capabilities

## Success Criteria

### Technical

- ✅ 80%+ test coverage (achieved in Sprint 1)
- ✅ All tests passing (9 repository + 7 E2E)
- ✅ Zero clippy warnings
- ✅ API documented (7 endpoints)
- 🎯 P95 response time < 200ms
- 🎯 Support for 100+ concurrent agents per team

### Business

- 🎯 Agents complete tasks autonomously with >90% success rate
- 🎯 Cost tracking accurate to within 1%
- 🎯 Teams operate within budget constraints
- 🎯 Human oversight required < 10% of the time

## Competitive Advantages

1. **Autonomous Operation**: Agents work with minimal human oversight
2. **Team-Based**: Multiple agents collaborate vs single-agent systems
3. **Built-in Budgets**: Native cost tracking and management
4. **Rust Performance**: Fast, safe, concurrent execution
5. **Domain-Driven**: Clean architecture, maintainable codebase

## Development Approach

- **AI-First**: Claude Code AI as primary development partner
- **Sprint-Driven**: Detailed sprint documents with clear acceptance criteria
- **Test-Driven**: 80%+ coverage, integration and E2E tests
- **Quality-Focused**: cargo fmt, clippy, audit checks
- **Incremental**: Build sprint by sprint, validate before proceeding

## Key Technical Patterns

### Domain Layer
- Aggregates: `Team`, `User`, `Agent` (Sprint 2)
- Value Objects: `Email`, `TeamStatus`, `Budget`
- Domain Events: `TeamCreated`, `TeamStatusChanged`, etc.

### Repository Pattern
```rust
#[async_trait]
pub trait TeamRepository {
    async fn save(&self, team: &Team) -> Result<(), String>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, String>;
    async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, String>;
    async fn delete(&self, id: Uuid) -> Result<(), String>;
}
```

### API Layer
- Axum handlers extract JWT claims for authorization
- JSON request/response with serde
- Proper error handling with ApiError types
- CORS enabled for web client

## Current State (Post-Sprint 1)

### What Works
✅ PostgreSQL running on port 54320
✅ 8 database migrations applied
✅ User registration and login (JWT)
✅ Team CRUD operations
✅ Multi-tenant isolation verified
✅ Integration tests passing
✅ E2E API tests passing
✅ JWT middleware protecting endpoints
✅ API server running on port 3000

### What's Next (Sprint 2)
🚧 Agent domain model
🚧 LLM integration (Claude API)
🚧 Tool execution framework
🚧 Autonomous task execution
🚧 Agent-to-agent communication

---

_This context provides high-level understanding. Detailed sprint tasks are in `/docs/sprints/`._
