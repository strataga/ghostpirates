# Ghost Pirates - Getting Started with Claude Code

Welcome to the Ghost Pirates implementation! This guide will help you work efficiently with Claude Code AI.

## Project Overview

**Ghost Pirates** is an _Autonomous AI Teams_ platform designed to enable companies to create and manage AI-powered teams that execute complex workflows autonomously.

- **Current Status**: Sprint 1 Complete (Foundation), Sprint 2 In Progress
- **Architecture**: Multi-tenant, Rust backend, Next.js frontend (planned)
- **Database**: PostgreSQL with company_id isolation, port 54320

## Sprint Implementation Plan

All sprint details are in `/docs/sprints/`:

```
sprint-1-foundation.md       - ✅ COMPLETE (Foundation infrastructure)
sprint-2-agent-system.md     - 🚧 IN PROGRESS (Agent architecture)
sprint-2-summary.md          - Quick overview of Sprint 2
sprint-2-quick-start.md      - Fast-track Sprint 2 tasks
```

## How to Use This Project

### Starting a Sprint

```bash
# Use the custom slash command
/start-sprint 2    # Start Sprint 2 (Agent System)
/start-sprint 3    # Future sprints
```

Or manually:

```bash
# Ask Claude to read and implement a specific sprint
"Let's start Sprint 2 - read /docs/sprints/sprint-2-agent-system.md and begin implementing"
```

### Checking Progress

```bash
/review-progress  # See what's been completed across all sprints
```

### Quick Reference

```bash
/tech-stack       # View Ghost Pirates technology stack
```

## Implementation Workflow

Each sprint follows this pattern:

1. **Read the sprint file** - Understand all user stories and tasks
2. **Create todo list** - Track progress with TodoWrite tool
3. **Execute tasks sequentially** - Follow exact commands and code patterns
4. **Verify acceptance criteria** - Ensure each task passes its checks
5. **Run tests** - Integration and E2E tests must pass
6. **Move to next task** - Only after current task is complete

### Example Task Execution

From sprint file:

```markdown
- [ ] 1.1.1: Create Agent domain model

**File**: `src/domain/agent/mod.rs`
```

Claude will:

1. Create the file with proper domain structure
2. Implement the Agent aggregate
3. Write unit tests
4. Verify it compiles
5. Mark the task complete
6. Move to next task

## Key Principles

### ✅ DO

- Follow the sprint plans exactly as written
- Use hexagonal architecture patterns (domain → repository → API)
- Complete all acceptance criteria before moving on
- Mark tasks as completed in the todo list
- Run tests frequently (cargo test, cargo clippy, cargo fmt)
- Ask questions if something is unclear

### ❌ DON'T

- Skip tasks or acceptance criteria
- Modify domain logic without considering events
- Create files not specified in the sprint
- Move to next sprint before current sprint is complete
- Ignore test failures or clippy warnings
- Use `git reset` (per user's CLAUDE.md instructions)

## Development Environment

### Prerequisites (Already Set Up in Sprint 1)

- ✅ **Rust**: 1.90+ (`rustup`)
- ✅ **PostgreSQL**: Running on port 54320 via Docker
- ✅ **Database**: 8 migrations applied
- ✅ **.env file**: Configured with DATABASE_URL, JWT_SECRET

### Local Development

**Database**:
```bash
# PostgreSQL is running via docker-compose on port 54320
docker compose up -d postgres
```

**API Server**:
```bash
# Run the Rust API (port 3000)
cd apps/api
cargo run
```

**Run Tests**:
```bash
# Integration tests (require DATABASE_URL)
cargo test --test repository_integration

# E2E API tests
cargo test --test api_integration

# All tests
cargo test

# Check code quality
cargo fmt --check
cargo clippy -- -D warnings
```

### Ports

- Rust API: `http://localhost:3000`
- PostgreSQL: `localhost:54320`
- Redis: `localhost:6379` (planned)
- Next.js Web: `http://localhost:3001` (planned)

## Sprint 1 Summary (Completed)

✅ Database schema with 8 migrations
✅ Domain models (Team, User, Email value object, TeamStatus)
✅ Repository implementations (PostgresTeamRepository, PostgresUserRepository)
✅ REST API with 7 endpoints (auth + teams)
✅ JWT authentication with middleware
✅ 9 repository integration tests (all passing)
✅ 7 E2E API tests (all passing)
✅ Code quality: cargo fmt, clippy clean
✅ API documentation in README.md

## Sprint 2 Overview (In Progress)

Focus: **Agent System Architecture**

Key deliverables:
1. Agent domain model (capabilities, roles, state)
2. LLM integration (Claude API client)
3. Tool execution framework
4. Agent memory and context
5. Autonomous task execution
6. Inter-agent communication

See `/docs/sprints/sprint-2-agent-system.md` for full details.

## Custom Commands

Available slash commands:

- `/start-sprint <number>` - Start implementing a specific sprint
- `/review-progress` - Check completion status across all sprints
- `/tech-stack` - Quick reference for technologies and architecture

## Testing Strategy

### Repository Tests (`tests/repository_integration.rs`)
- Test CRUD operations with real PostgreSQL
- Verify tenant isolation (company_id)
- Check foreign key constraints
- Validate unique constraints

### E2E API Tests (`tests/api_integration.rs`)
- Full user journeys (Register → Login → Create Team)
- JWT authentication enforcement
- Protected endpoint validation
- Database persistence verification

### Running Tests
```bash
# Must set DATABASE_URL for integration tests
export DATABASE_URL="postgresql://postgres:postgres@localhost:54320/ghostpirates_dev"

# Run all tests
cargo test

# Run specific test file
cargo test --test repository_integration
cargo test --test api_integration
```

## Project Structure

```
ghostpirates/
├── apps/
│   ├── api/                 # Rust API (Axum, SQLx)
│   │   ├── src/
│   │   │   ├── domain/      # Domain layer (aggregates, value objects, events)
│   │   │   ├── infrastructure/ # Repository implementations
│   │   │   ├── api/         # HTTP handlers, middleware
│   │   │   ├── auth/        # JWT, password hashing
│   │   │   └── main.rs      # Server entry point
│   │   ├── tests/           # Integration and E2E tests
│   │   ├── migrations/      # SQLx database migrations (8 complete)
│   │   └── Cargo.toml
│   └── web/                 # Next.js frontend (planned)
├── docs/
│   └── sprints/             # Sprint planning documents
├── .claude/                 # Claude Code configuration
│   ├── commands/            # Custom slash commands
│   ├── PROJECT_CONTEXT.md   # High-level project overview
│   └── GETTING_STARTED.md   # This file
├── docker-compose.yml       # PostgreSQL + Redis
└── .env                     # Environment configuration
```

## Getting Help

- **Sprint details**: Read the specific sprint file in `/docs/sprints/`
- **Architecture**: See `.claude/PROJECT_CONTEXT.md`
- **API endpoints**: See `README.md` API Documentation section
- **Domain models**: Look in `apps/api/src/domain/`

## Common Commands

```bash
# Database migrations
cd apps/api
sqlx migrate run

# Build and run
cargo build
cargo run

# Code quality
cargo fmt
cargo clippy -- -D warnings
cargo audit

# Testing
cargo test                              # All tests
cargo test --test repository_integration # Repository tests only
cargo test --test api_integration       # E2E tests only
```

## Next Steps

Ready to start Sprint 2? Ask Claude:

```
"Let's begin Sprint 2 - Agent System. Read /docs/sprints/sprint-2-agent-system.md and let's get started!"
```

Or use the slash command:

```
/start-sprint 2
```

---

**Built with Claude Code AI** 🚀
