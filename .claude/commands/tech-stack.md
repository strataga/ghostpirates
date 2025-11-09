---
description: Quick reference for Ghost Pirates technology stack and architecture
tags: [reference, architecture]
---

# Ghost Pirates Technology Stack Quick Reference

Display the complete technology stack for quick reference.

## Backend

**Rust API** (apps/api/) - Port 3000

- Axum 0.7 (web framework)
- SQLx 0.7 (PostgreSQL with compile-time query verification)
- Tokio 1 (async runtime)
- Serde 1.0 (JSON serialization)
- UUID 1.0 (unique identifiers)
- BCrypt 0.15 (password hashing)
- JsonWebToken 9.2 (JWT authentication)
- rust_decimal 1.33 (precise decimal arithmetic)

## Frontend (Planned)

**Next.js Web** (apps/web/) - Port 3001

- Next.js 14+ (React framework)
- React 18+ (UI library)
- Tailwind CSS (styling)
- TanStack Query (data fetching)
- Zustand or Redux (state management)

## Database

**PostgreSQL 15+** - Port 54320 (local)

- Multi-tenant architecture with company_id isolation
- 8 migrations completed:
  1. Companies (tenant root)
  2. Users (authentication)
  3. Teams (AI teams)
  4. Team members (relationships)
  5. Tasks (work items)
  6. Messages (communication)
  7. Checkpoints (progress tracking)
  8. Cost tracking (budgets)

**Schema Features**:
- UUID primary keys
- Timestamps (created_at, updated_at)
- Foreign keys with CASCADE deletes
- Unique constraints (email globally unique)
- Custom types (team_status enum)

## Infrastructure

**Local Development**:
- Docker Compose (PostgreSQL + Redis)
- Port 54320 for PostgreSQL (avoiding conflicts)

**Production (Planned)**:
- Azure Container Apps (API hosting)
- Azure PostgreSQL Flexible Server
- Azure Blob Storage (file storage)
- Terraform (Infrastructure as Code)

## Domain Structure

**Hexagonal Architecture**:

```
domain/          - Pure business logic
├── team/        - Team aggregate
├── user/        - User aggregate
├── agent/       - Agent aggregate (Sprint 2)
└── repositories/- Repository traits

infrastructure/  - External concerns
└── repositories/- PostgreSQL implementations

api/             - HTTP layer
├── handlers/    - Axum route handlers
├── middleware/  - JWT authentication
└── errors/      - API error types

auth/            - Authentication
├── jwt.rs       - Token generation/validation
└── password.rs  - BCrypt hashing
```

## Authentication

**JWT Tokens**:
- 8-hour expiry
- Claims: user_id (sub), email
- HS256 algorithm
- Secret from JWT_SECRET env var

**Password Security**:
- BCrypt hashing (cost factor 12)
- Never stored in plain text

## Testing

**Integration Tests** (tests/repository_integration.rs):
- 9 tests covering repository CRUD operations
- Real PostgreSQL database required
- Tenant isolation verification

**E2E API Tests** (tests/api_integration.rs):
- 7 tests covering full user journeys
- HTTP request/response validation
- JWT authentication testing

**Test Coverage**: 80%+ (Sprint 1 target achieved)

## API Endpoints

1. `GET /health` - Health check
2. `POST /api/auth/register` - User registration
3. `POST /api/auth/login` - User login (returns JWT)
4. `POST /api/teams` - Create team (protected)
5. `GET /api/teams/:id` - Get team by ID (protected)
6. `DELETE /api/teams/:id` - Delete team (protected)
7. `GET /api/teams/company/:company_id` - List company teams (protected)

## Key Architecture Patterns

1. **Domain-Driven Design** - Rich domain models with behavior
2. **Repository Pattern** - Data access abstraction
3. **Hexagonal Architecture** - Ports & adapters
4. **Multi-tenancy** - company_id isolation
5. **Event Sourcing** - Domain events for state changes
6. **CQRS** - Command/query separation (planned)

## Environment Variables

Required in `.env`:
```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:54320/ghostpirates_dev
JWT_SECRET=your-secret-key-here
RUST_LOG=info
```

## Code Quality Tools

- `cargo fmt` - Code formatting
- `cargo clippy` - Linting (zero warnings required)
- `cargo audit` - Security vulnerability scanning
- `cargo test` - Test runner

## Current State (Sprint 1 Complete)

✅ Database: 8 migrations applied
✅ API Server: Running on port 3000
✅ Tests: 16 passing (9 integration + 7 E2E)
✅ Code Quality: fmt clean, clippy clean
✅ Documentation: API docs in README.md
✅ Security: JWT middleware, password hashing

## Sprint 2 Additions (In Progress)

🚧 Agent domain models
🚧 LLM integration (Claude API client)
🚧 Tool execution framework
🚧 Agent memory and context
🚧 Autonomous task execution

---

See `/docs/sprints/` for detailed sprint plans and `.claude/PROJECT_CONTEXT.md` for architectural decisions.
