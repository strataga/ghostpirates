# Sprint 1 - Foundation: Database, API & Authentication

**Phase:** Phase 1 of 8
**Duration:** 2 Weeks (Weeks 1-2)
**Goal:** Establish PostgreSQL database schema, Rust API foundation with Axum, and JWT authentication system

---

## 📋 How to Use This Sprint Document

### Daily Workflow

1. **Start of Day**: Review your assigned tasks in the Progress Dashboard below
2. **During Work**: Check off `[ ]` boxes as you complete each sub-task (use `[x]`)
3. **End of Day**: Update the Progress Dashboard with % complete for each user story
4. **Blockers**: Immediately document any blockers in the "Sprint Blockers" section
5. **Questions**: Add questions to the "Questions & Decisions" section for team discussion

### Task Completion Guidelines

- ✅ **Check off tasks** by replacing `[ ]` with `[x]` in the markdown
- 📝 **Add notes** inline using `<!-- Note: ... -->` for context or decisions
- 🚫 **Mark blockers** by adding `🚫 BLOCKED:` prefix to task descriptions
- ⚠️ **Flag issues** by adding `⚠️ ISSUE:` prefix for items needing attention
- 🔄 **Track dependencies** between tasks by referencing task numbers (e.g., "Depends on 101.15")

---

## 📊 Progress Dashboard

**Last Updated:** 2025-11-08
**Overall Sprint Progress:** 0% Complete (0 of 5 user stories done)

| User Story                          | Tasks Complete | Progress             | Status         | Assignee | Blockers |
| ----------------------------------- | -------------- | -------------------- | -------------- | -------- | -------- |
| US-101: Database Schema             | 0/50 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-102: Domain Models (DDD)         | 0/40 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-103: Repository Layer            | 0/35 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-104: API Foundation (Axum)       | 0/30 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-105: JWT Authentication          | 0/25 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |

**Status Legend:**

- 🔴 Not Started (0%)
- 🟡 In Progress (1-99%)
- 🟢 Complete (100%)
- 🔵 In Review (awaiting PR approval)
- 🟣 Blocked (cannot proceed)

---

## 🎯 Sprint Objectives

### Primary Goal

Enable team creation with database persistence and secure user authentication.

At the end of this sprint, the system will:

- Store teams, users, tasks, and messages in PostgreSQL
- Expose REST API endpoints for team management
- Authenticate users via JWT tokens with 8-hour expiry
- Follow Hexagonal Architecture and DDD patterns

### Success Metrics

**Technical Metrics:**

- [ ] All 5 user stories completed and tested
- [ ] API response time < 200ms (P95)
- [ ] Zero critical bugs in staging
- [ ] Test coverage ≥ 80% for domain logic
- [ ] All CI/CD pipelines passing

**Business Metrics:**

- [ ] Database schema supports multi-tenant isolation
- [ ] API documented with OpenAPI spec
- [ ] Authentication flow tested end-to-end

**Quality Metrics:**

- [ ] Code follows Hexagonal Architecture pattern
- [ ] Domain models use DDD principles (entities, value objects)
- [ ] Security scan passes (zero high/critical vulnerabilities)
- [ ] All migrations reversible (up + down migrations)

---

## ✅ Prerequisites Checklist

> **IMPORTANT:** Complete ALL prerequisites before starting sprint work.

### Sprint Dependencies

**This sprint depends on:**

- [ ] PostgreSQL 16+ database accessible (local or Azure)
- [ ] Redis instance running (for future caching)
- [ ] Azure account configured (if using Azure PostgreSQL)

### Development Environment Setup

**Required Tools:**

- [ ] Rust 1.75+ installed (`rustc --version` shows 1.75.0 or higher)
- [ ] PostgreSQL client tools (`psql --version` shows 16.0 or higher)
- [ ] SQLx CLI installed (`cargo install sqlx-cli --features postgres`)
- [ ] Docker Desktop running (for local PostgreSQL if needed)
- [ ] Git configured with user name and email

**Validation Steps:**

```bash
# Verify Rust toolchain
rustc --version  # Should show 1.75.0+
cargo --version

# Verify PostgreSQL access
psql --version   # Should show 16.0+

# Verify SQLx CLI
sqlx --version

# Verify Docker (if using local DB)
docker ps
```

### Required External Accounts & Services

- [ ] PostgreSQL database created
  - **Validation:** `psql $DATABASE_URL -c "SELECT version();"` returns PostgreSQL 16.x
- [ ] GitHub repository cloned
  - **Validation:** `git remote -v` shows origin pointing to strataga/ghostpirates

### Environment Variables

Create `.env` file in project root:

- [ ] `DATABASE_URL` - PostgreSQL connection string
- [ ] `JWT_SECRET` - Secret key for JWT signing (generate with `openssl rand -hex 32`)
- [ ] `RUST_LOG` - Log level (set to `info` or `debug`)

**Validation:**

```bash
# Create .env file
cat > .env << 'EOF'
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/ghostpirates_dev
JWT_SECRET=your-secret-key-here-generate-with-openssl-rand-hex-32
RUST_LOG=info
EOF

# Test database connectivity
psql $DATABASE_URL -c "SELECT version();"
```

### Required Knowledge & Reading

> **⚠️ CRITICAL:** Review all relevant patterns from `docs/patterns/` BEFORE starting implementation.

**MUST READ - Patterns Documentation:**

- [ ] **[Pattern Integration Guide](../patterns/16-Pattern-Integration-Guide.md)** - READ FIRST
- [ ] **[Hexagonal Architecture](../patterns/03-Hexagonal-Architecture.md)** - REQUIRED for all features
- [ ] **[Domain-Driven Design](../patterns/04-Domain-Driven-Design.md)** - Entity/Value Object design
- [ ] **[Repository Pattern](../patterns/06-Repository-Pattern.md)** - Data access abstraction
- [ ] **[CQRS Pattern](../patterns/05-CQRS-Pattern.md)** - Command/Query separation
- [ ] **[Multi-Tenancy Pattern](../patterns/17-Multi-Tenancy-Pattern.md)** - Tenant isolation

**MUST READ - Research & Planning:**

- [ ] [Phase 1 Implementation Plan](../plans/04-phase-1-foundation.md) - Complete phase 1 details
- [ ] [Technology Stack](../plans/01-technology-stack.md) - Rust + PostgreSQL decisions
- [ ] [Database Architecture](../plans/03-database-architecture.md) - Schema design patterns

**Time Estimate:** 3-4 hours to complete prerequisite reading and environment setup

---

## 📚 Key References

### Technical Documentation

- **Architecture:** [Hexagonal Architecture Pattern](../patterns/03-Hexagonal-Architecture.md)
- **Patterns Used:** [Pattern Catalog](../patterns/16-Pattern-Integration-Guide.md)
- **API Framework:** [Axum Web Framework Docs](https://docs.rs/axum/latest/axum/)
- **Database:** [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)

### Research Documents

- [Phase 1: Foundation](../plans/04-phase-1-foundation.md) - Detailed implementation plan
- [Database Architecture](../plans/03-database-architecture.md) - Schema design
- [Technology Stack](../plans/01-technology-stack.md) - Tech decisions

---

## 🚀 User Stories

### US-101: Database Schema Implementation

**As a** system administrator
**I want** a PostgreSQL database schema
**So that** teams, users, tasks, and messages can be persisted

**Business Value:** Foundation for all data storage, enables multi-tenant isolation and audit trails

**Acceptance Criteria:**

- [ ] All 8 core tables created (companies, users, teams, team_members, tasks, messages, checkpoints, cost_tracking)
- [ ] Indexes created for performance on foreign keys and commonly queried fields
- [ ] Migrations are reversible (both up and down migrations work)
- [ ] Database constraints enforce business rules (positive budgets, valid workloads, etc.)
- [ ] Can create sample data successfully

**Technical Implementation:**

**Patterns Used:**

- [x] Multi-Tenancy Pattern (company_id for isolation)
- [x] Soft Delete Pattern (future: deleted_at timestamps)
- [x] Audit Trail Pattern (created_at, updated_at timestamps)

**File Structure:**

```
apps/api/
├── migrations/
│   ├── 20251108000001_create_companies_table.sql
│   ├── 20251108000002_create_users_table.sql
│   ├── 20251108000003_create_teams_table.sql
│   ├── 20251108000004_create_team_members_table.sql
│   ├── 20251108000005_create_tasks_table.sql
│   ├── 20251108000006_create_messages_table.sql
│   ├── 20251108000007_create_checkpoints_table.sql
│   └── 20251108000008_create_cost_tracking_table.sql
```

**Estimation:** 8 hours

---

#### 📋 Sub-Tasks Breakdown (US-101)

**Phase 1: Database Setup** (Tasks 101.1 - 101.10)

- [ ] **101.1** - Initialize Cargo project
  - **Command:** `cargo new ghostpirates-api && cd ghostpirates-api`
  - **Validation:** `cargo build` succeeds
  - **Estimate:** 10 minutes

- [ ] **101.2** - Add SQLx dependencies to Cargo.toml
  - **File:** `Cargo.toml`
  - **Code:**

    ```toml
    [dependencies]
    axum = "0.7"
    tokio = { version = "1", features = ["full"] }
    sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
    serde = { version = "1.0", features = ["derive"] }
    serde_json = "1.0"
    uuid = { version = "1.0", features = ["v4", "serde"] }
    chrono = { version = "0.4", features = ["serde"] }
    tracing = "0.1"
    tracing-subscriber = "0.3"
    dotenv = "0.15"
    ```

  - **Validation:** `cargo check` passes
  - **Estimate:** 5 minutes

- [ ] **101.3** - Install SQLx CLI
  - **Command:** `cargo install sqlx-cli --features postgres`
  - **Validation:** `sqlx --version` shows 0.7.x
  - **Estimate:** 5 minutes

- [ ] **101.4** - Create database
  - **Command:** `sqlx database create`
  - **Validation:** `psql $DATABASE_URL -c "\l"` shows database exists
  - **Estimate:** 2 minutes

- [ ] **101.5** - Create companies table migration
  - **Command:** `sqlx migrate add create_companies_table`
  - **File:** `migrations/XXXXXX_create_companies_table.sql`
  - **Code:**

    ```sql
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

    CREATE TABLE companies (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        name VARCHAR(255) NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX idx_companies_created_at ON companies(created_at);
    ```

  - **Validation:** File created in migrations/
  - **Estimate:** 10 minutes

- [ ] **101.6** - Create users table migration
  - **Command:** `sqlx migrate add create_users_table`
  - **Code:**

    ```sql
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

  - **Estimate:** 15 minutes

- [ ] **101.7** - Create teams table migration
  - **Command:** `sqlx migrate add create_teams_table`
  - **Code:**

    ```sql
    CREATE TYPE team_status AS ENUM (
        'pending', 'planning', 'active', 'completed', 'failed', 'archived'
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

  - **Estimate:** 20 minutes

- [ ] **101.8** - Create remaining tables (team_members, tasks, messages, checkpoints, cost_tracking)
  - **Commands:** Run `sqlx migrate add` for each table
  - **Refer to:** [Phase 1 Plan](../plans/04-phase-1-foundation.md) for complete SQL
  - **Estimate:** 60 minutes

- [ ] **101.9** - Run all migrations
  - **Command:** `sqlx migrate run`
  - **Validation:** `sqlx migrate info` shows all migrations applied
  - **Validation:** `psql $DATABASE_URL -c "\dt"` lists all 8 tables
  - **Estimate:** 5 minutes

- [ ] **101.10** - Verify schema with sample data
  - **Commands:**

    ```sql
    INSERT INTO companies (name) VALUES ('Test Company');
    INSERT INTO users (company_id, email, password_hash, full_name)
    SELECT id, 'test@example.com', 'hash', 'Test User'
    FROM companies WHERE name = 'Test Company';
    ```

  - **Validation:** Queries succeed, foreign keys work
  - **Estimate:** 10 minutes

---

### US-102: Domain Models (DDD)

**As a** backend developer
**I want** domain entities following DDD principles
**So that** business logic is isolated from infrastructure

**Business Value:** Clean architecture, testable business logic, maintainable codebase

**Acceptance Criteria:**

- [ ] Team, User, Task, Message entities created in `src/domain/`
- [ ] Value objects for Email, TeamStatus, TaskStatus
- [ ] Domain events for Team created, Task assigned, Task completed
- [ ] Business rules enforced in domain layer (budget limits, revision counts)
- [ ] Unit tests for all domain logic (≥80% coverage)

**Technical Implementation:**

**Patterns Used:**

- [x] Hexagonal Architecture (domain independent of infrastructure)
- [x] Domain-Driven Design (entities, value objects, aggregates)
- [x] Value Object Pattern (immutable, validated types)

**File Structure:**

```
src/
├── domain/
│   ├── mod.rs
│   ├── team/
│   │   ├── mod.rs
│   │   ├── team.rs              # Team aggregate
│   │   ├── value_objects.rs     # TeamStatus, Budget
│   │   └── events.rs            # TeamCreated, TeamCompleted
│   ├── user/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   └── value_objects.rs     # Email, FullName
│   └── task/
│       ├── mod.rs
│       ├── task.rs
│       └── value_objects.rs     # TaskStatus
```

**Estimation:** 10 hours

---

#### 📋 Sub-Tasks Breakdown (US-102)

- [ ] **102.1** - Create domain module structure
  - **Commands:**

    ```bash
    mkdir -p src/domain/{team,user,task}
    touch src/domain/mod.rs
    touch src/domain/team/{mod.rs,team.rs,value_objects.rs,events.rs}
    touch src/domain/user/{mod.rs,user.rs,value_objects.rs}
    touch src/domain/task/{mod.rs,task.rs,value_objects.rs}
    ```

  - **Estimate:** 5 minutes

- [ ] **102.2** - Implement Email value object
  - **File:** `src/domain/user/value_objects.rs`
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Email(String);

    impl Email {
        pub fn new(email: impl Into<String>) -> Result<Self, String> {
            let email = email.into();
            if Self::is_valid(&email) {
                Ok(Email(email))
            } else {
                Err(format!("Invalid email: {}", email))
            }
        }

        fn is_valid(email: &str) -> bool {
            email.contains('@') && email.len() >= 3
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }

    impl fmt::Display for Email {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn valid_email() {
            assert!(Email::new("test@example.com").is_ok());
        }

        #[test]
        fn invalid_email() {
            assert!(Email::new("invalid").is_err());
        }
    }
    ```

  - **Estimate:** 20 minutes

- [ ] **102.3** - Implement TeamStatus enum
  - **File:** `src/domain/team/value_objects.rs`
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum TeamStatus {
        Pending,
        Planning,
        Active,
        Completed,
        Failed,
        Archived,
    }

    impl TeamStatus {
        pub fn can_transition_to(&self, next: TeamStatus) -> bool {
            use TeamStatus::*;
            matches!(
                (self, next),
                (Pending, Planning) |
                (Planning, Active) |
                (Active, Completed) |
                (Active, Failed) |
                (Completed, Archived) |
                (Failed, Archived)
            )
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn valid_transition() {
            assert!(TeamStatus::Pending.can_transition_to(TeamStatus::Planning));
        }

        #[test]
        fn invalid_transition() {
            assert!(!TeamStatus::Pending.can_transition_to(TeamStatus::Completed));
        }
    }
    ```

  - **Estimate:** 20 minutes

- [ ] **102.4** - Implement Team entity
  - **File:** `src/domain/team/team.rs`
  - **Code:**

    ```rust
    use chrono::{DateTime, Utc};
    use rust_decimal::Decimal;
    use uuid::Uuid;
    use super::value_objects::TeamStatus;
    use super::events::TeamEvent;

    #[derive(Debug, Clone)]
    pub struct Team {
        id: Uuid,
        company_id: Uuid,
        goal: String,
        status: TeamStatus,
        manager_agent_id: Option<Uuid>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
        started_at: Option<DateTime<Utc>>,
        completed_at: Option<DateTime<Utc>>,
        budget_limit: Option<Decimal>,
    }

    impl Team {
        pub fn new(
            company_id: Uuid,
            goal: String,
            created_by: Uuid,
            budget_limit: Option<Decimal>,
        ) -> Result<(Self, Vec<TeamEvent>), String> {
            if goal.is_empty() {
                return Err("Goal cannot be empty".to_string());
            }

            if let Some(budget) = budget_limit {
                if budget <= Decimal::ZERO {
                    return Err("Budget must be positive".to_string());
                }
            }

            let team = Self {
                id: Uuid::new_v4(),
                company_id,
                goal,
                status: TeamStatus::Pending,
                manager_agent_id: None,
                created_by,
                created_at: Utc::now(),
                started_at: None,
                completed_at: None,
                budget_limit,
            };

            let events = vec![TeamEvent::Created {
                team_id: team.id,
                company_id: team.company_id,
                goal: team.goal.clone(),
                created_by: team.created_by,
            }];

            Ok((team, events))
        }

        pub fn start(&mut self) -> Result<TeamEvent, String> {
            if !self.status.can_transition_to(TeamStatus::Active) {
                return Err(format!("Cannot start team in {:?} status", self.status));
            }

            self.status = TeamStatus::Active;
            self.started_at = Some(Utc::now());

            Ok(TeamEvent::Started {
                team_id: self.id,
            })
        }

        // Getters
        pub fn id(&self) -> Uuid { self.id }
        pub fn status(&self) -> TeamStatus { self.status }
        pub fn goal(&self) -> &str { &self.goal }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn create_team_with_valid_goal() {
            let result = Team::new(
                Uuid::new_v4(),
                "Test goal".to_string(),
                Uuid::new_v4(),
                None,
            );
            assert!(result.is_ok());
        }

        #[test]
        fn create_team_with_empty_goal_fails() {
            let result = Team::new(
                Uuid::new_v4(),
                "".to_string(),
                Uuid::new_v4(),
                None,
            );
            assert!(result.is_err());
        }
    }
    ```

  - **Estimate:** 60 minutes

- [ ] **102.5** - Implement domain events
  - **File:** `src/domain/team/events.rs`
  - **Code:**

    ```rust
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub enum TeamEvent {
        Created {
            team_id: Uuid,
            company_id: Uuid,
            goal: String,
            created_by: Uuid,
        },
        Started {
            team_id: Uuid,
        },
        Completed {
            team_id: Uuid,
        },
        Failed {
            team_id: Uuid,
            reason: String,
        },
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **102.6** - Implement User, Task entities (similar pattern)
  - **Refer to:** Team entity implementation above
  - **Estimate:** 120 minutes

- [ ] **102.7** - Write unit tests for all domain entities
  - **Target:** ≥80% coverage
  - **Command:** `cargo test --lib`
  - **Estimate:** 60 minutes

---

### US-103: Repository Layer

**As a** backend developer
**I want** repository interfaces and implementations
**So that** domain logic is decoupled from database access

**Business Value:** Testable code, swappable persistence, clean architecture

**Acceptance Criteria:**

- [ ] Repository trait defined in domain layer
- [ ] PostgreSQL implementation in infrastructure layer
- [ ] CRUD operations for Team, User, Task
- [ ] Transaction support for multi-table operations
- [ ] Integration tests with test database (≥80% coverage)

**Patterns Used:**

- [x] Repository Pattern (data access abstraction)
- [x] Hexagonal Architecture (ports and adapters)
- [x] Dependency Inversion Principle (depend on abstractions)

**Estimation:** 8 hours

---

#### 📋 Sub-Tasks Breakdown (US-103)

- [ ] **103.1** - Define repository traits in domain
  - **File:** `src/domain/repositories/team_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::domain::team::Team;

    #[async_trait]
    pub trait TeamRepository: Send + Sync {
        async fn save(&self, team: &Team) -> Result<(), String>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, String>;
        async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, String>;
        async fn delete(&self, id: Uuid) -> Result<(), String>;
    }
    ```

  - **Estimate:** 20 minutes

- [ ] **103.2** - Implement PostgreSQL repository
  - **File:** `src/infrastructure/repositories/postgres_team_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use sqlx::PgPool;
    use uuid::Uuid;
    use crate::domain::team::Team;
    use crate::domain::repositories::TeamRepository;

    pub struct PostgresTeamRepository {
        pool: PgPool,
    }

    impl PostgresTeamRepository {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl TeamRepository for PostgresTeamRepository {
        async fn save(&self, team: &Team) -> Result<(), String> {
            sqlx::query!(
                r#"
                INSERT INTO teams (id, company_id, goal, status, created_by, budget_limit)
                VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (id) DO UPDATE SET
                    goal = EXCLUDED.goal,
                    status = EXCLUDED.status
                "#,
                team.id(),
                team.company_id(),
                team.goal(),
                team.status() as _,
                team.created_by(),
                team.budget_limit()
            )
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            Ok(())
        }

        async fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, String> {
            let row = sqlx::query!(
                r#"
                SELECT id, company_id, goal, status as "status: TeamStatus",
                       created_by, budget_limit, created_at, started_at, completed_at
                FROM teams
                WHERE id = $1
                "#,
                id
            )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            // Convert row to Team entity
            Ok(row.map(|r| {
                // Reconstruction logic here
                Team::from_persistence(...)
            }))
        }

        async fn find_by_company(&self, company_id: Uuid) -> Result<Vec<Team>, String> {
            // Similar implementation
            todo!()
        }

        async fn delete(&self, id: Uuid) -> Result<(), String> {
            sqlx::query!("DELETE FROM teams WHERE id = $1", id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
    ```

  - **Estimate:** 90 minutes

- [ ] **103.3** - Implement repositories for User, Task, Message
  - **Pattern:** Similar to TeamRepository
  - **Estimate:** 180 minutes

- [ ] **103.4** - Write integration tests
  - **File:** `tests/repositories/team_repository_tests.rs`
  - **Setup:** Use test database with SQLx test macros
  - **Estimate:** 60 minutes

---

### US-104: API Foundation (Axum)

**As a** frontend developer
**I want** REST API endpoints
**So that** I can create teams and manage tasks via HTTP

**Business Value:** Enables frontend integration, API-first development

**Acceptance Criteria:**

- [ ] Axum server running on port 3000
- [ ] Health check endpoint `GET /health` returns 200
- [ ] CORS configured for frontend access
- [ ] Error handling middleware (returns structured JSON errors)
- [ ] Request logging with tracing

**Patterns Used:**

- [x] Hexagonal Architecture (controllers as adapters)
- [x] Dependency Injection (via Axum state)

**Estimation:** 6 hours

---

#### 📋 Sub-Tasks Breakdown (US-104)

- [ ] **104.1** - Create main.rs with Axum server
  - **File:** `src/main.rs`
  - **Code:**

    ```rust
    use axum::{Router, routing::get};
    use sqlx::PgPool;
    use std::net::SocketAddr;
    use tracing_subscriber;

    #[tokio::main]
    async fn main() {
        // Initialize tracing
        tracing_subscriber::fmt::init();

        // Load environment variables
        dotenv::dotenv().ok();

        // Connect to database
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to database");

        // Build router
        let app = Router::new()
            .route("/health", get(health_check))
            .with_state(pool);

        // Start server
        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        tracing::info!("Server listening on {}", addr);

        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .expect("Server failed");
    }

    async fn health_check() -> &'static str {
        "OK"
    }
    ```

  - **Validation:** `cargo run` starts server, `curl http://localhost:3000/health` returns "OK"
  - **Estimate:** 30 minutes

- [ ] **104.2** - Add error handling middleware
  - **File:** `src/api/errors.rs`
  - **Code:**

    ```rust
    use axum::{
        http::StatusCode,
        response::{IntoResponse, Response},
        Json,
    };
    use serde_json::json;

    pub struct ApiError {
        pub status: StatusCode,
        pub message: String,
    }

    impl IntoResponse for ApiError {
        fn into_response(self) -> Response {
            (self.status, Json(json!({
                "error": self.message
            }))).into_response()
        }
    }

    impl From<String> for ApiError {
        fn from(message: String) -> Self {
            Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message,
            }
        }
    }
    ```

  - **Estimate:** 20 minutes

- [ ] **104.3** - Add CORS middleware
  - **Dependencies:** Add `tower-http = { version = "0.5", features = ["cors"] }`
  - **Code in main.rs:**

    ```rust
    use tower_http::cors::{CorsLayer, Any};

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(pool);
    ```

  - **Estimate:** 15 minutes

- [ ] **104.4** - Add request logging
  - **Dependencies:** Add `tower-http = { version = "0.5", features = ["trace"] }`
  - **Code:**

    ```rust
    use tower_http::trace::TraceLayer;

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(pool);
    ```

  - **Estimate:** 10 minutes

- [ ] **104.5** - Create team endpoints (POST, GET)
  - **File:** `src/api/handlers/teams.rs`
  - **Routes:** `POST /api/teams`, `GET /api/teams`, `GET /api/teams/:id`
  - **Estimate:** 120 minutes

---

### US-105: JWT Authentication

**As a** user
**I want** to log in with email and password
**So that** I can access protected API endpoints

**Business Value:** Secure access control, user session management

**Acceptance Criteria:**

- [ ] `POST /api/auth/register` creates new user
- [ ] `POST /api/auth/login` returns JWT token
- [ ] Password hashing with bcrypt
- [ ] JWT token validation middleware
- [ ] Tokens expire after 8 hours

**Patterns Used:**

- [x] RBAC Pattern (future: role-based access)
- [x] Security Best Practices (password hashing, token expiry)

**Estimation:** 8 hours

---

#### 📋 Sub-Tasks Breakdown (US-105)

- [ ] **105.1** - Add authentication dependencies
  - **Cargo.toml:**

    ```toml
    bcrypt = "0.15"
    jsonwebtoken = "9.2"
    ```

  - **Estimate:** 2 minutes

- [ ] **105.2** - Implement password hashing
  - **File:** `src/auth/password.rs`
  - **Code:**

    ```rust
    use bcrypt::{hash, verify, DEFAULT_COST};

    pub fn hash_password(password: &str) -> Result<String, String> {
        hash(password, DEFAULT_COST).map_err(|e| e.to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
        verify(password, hash).map_err(|e| e.to_string())
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **105.3** - Implement JWT token creation
  - **File:** `src/auth/jwt.rs`
  - **Code:**

    ```rust
    use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use chrono::{Utc, Duration};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Claims {
        pub sub: Uuid,  // user_id
        pub exp: usize, // expiry
    }

    pub fn create_token(user_id: Uuid, secret: &str) -> Result<String, String> {
        let expiry = Utc::now() + Duration::hours(8);
        let claims = Claims {
            sub: user_id,
            exp: expiry.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_ref()),
        )
        .map_err(|e| e.to_string())
    }

    pub fn verify_token(token: &str, secret: &str) -> Result<Claims, String> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| e.to_string())
    }
    ```

  - **Estimate:** 30 minutes

- [ ] **105.4** - Implement register endpoint
  - **File:** `src/api/handlers/auth.rs`
  - **Code:**

    ```rust
    use axum::{Json, extract::State};
    use serde::{Deserialize, Serialize};
    use sqlx::PgPool;
    use uuid::Uuid;
    use crate::auth::password::hash_password;
    use crate::api::errors::ApiError;

    #[derive(Deserialize)]
    pub struct RegisterRequest {
        pub email: String,
        pub password: String,
        pub full_name: String,
        pub company_id: Uuid,
    }

    #[derive(Serialize)]
    pub struct RegisterResponse {
        pub user_id: Uuid,
    }

    pub async fn register(
        State(pool): State<PgPool>,
        Json(req): Json<RegisterRequest>,
    ) -> Result<Json<RegisterResponse>, ApiError> {
        // Validate email
        if !req.email.contains('@') {
            return Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: "Invalid email".to_string(),
            });
        }

        // Hash password
        let password_hash = hash_password(&req.password)?;

        // Insert user
        let user_id = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO users (id, company_id, email, password_hash, full_name)
             VALUES ($1, $2, $3, $4, $5)",
            user_id,
            req.company_id,
            req.email,
            password_hash,
            req.full_name
        )
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(Json(RegisterResponse { user_id }))
    }
    ```

  - **Estimate:** 45 minutes

- [ ] **105.5** - Implement login endpoint
  - **Code:**

    ```rust
    #[derive(Deserialize)]
    pub struct LoginRequest {
        pub email: String,
        pub password: String,
    }

    #[derive(Serialize)]
    pub struct LoginResponse {
        pub token: String,
    }

    pub async fn login(
        State(pool): State<PgPool>,
        Json(req): Json<LoginRequest>,
    ) -> Result<Json<LoginResponse>, ApiError> {
        // Find user by email
        let user = sqlx::query!(
            "SELECT id, password_hash FROM users WHERE email = $1",
            req.email
        )
        .fetch_optional(&pool)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| ApiError {
            status: StatusCode::UNAUTHORIZED,
            message: "Invalid credentials".to_string(),
        })?;

        // Verify password
        let valid = verify_password(&req.password, &user.password_hash)?;
        if !valid {
            return Err(ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid credentials".to_string(),
            });
        }

        // Create JWT token
        let secret = std::env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");
        let token = create_token(user.id, &secret)?;

        Ok(Json(LoginResponse { token }))
    }
    ```

  - **Estimate:** 45 minutes

- [ ] **105.6** - Implement JWT validation middleware
  - **File:** `src/api/middleware/auth.rs`
  - **Estimate:** 60 minutes

- [ ] **105.7** - Test authentication flow end-to-end
  - **Test:** Register → Login → Access protected endpoint with token
  - **Estimate:** 30 minutes

---

## 🔗 Cross-Story Integration

**Integration Points:**

- US-101 (Database) provides schema for US-102 (Domain Models)
- US-102 (Domain) defines interfaces for US-103 (Repositories)
- US-103 (Repositories) used by US-104 (API handlers)
- US-105 (Auth) protects endpoints from US-104

**Integration Tests:**

- [ ] Create team via API → Verify in database
- [ ] Register user → Login → Create team (full flow)
- [ ] Authentication → Protected endpoint access

---

## 🚧 Sprint Blockers

**Active Blockers:** None

**Resolved Blockers:** None

---

## 💬 Questions & Decisions

**Open Questions:** None yet

**Decisions Made:**

| Decision | Context | Rationale | Made By | Date |
|----------|---------|-----------|---------|------|
| Use SQLx over Diesel | ORM choice | SQLx provides compile-time verification and async support | Team | 2025-11-08 |
| 8-hour JWT expiry | Token lifetime | Balance security and UX | Team | 2025-11-08 |

---

## ✅ Definition of Done

### Code Quality

- [ ] **Follows Ghost Pirates patterns** from `docs/patterns/`
  - [ ] Hexagonal Architecture (domain isolated from infrastructure)
  - [ ] Domain-Driven Design (entities, value objects)
  - [ ] Repository Pattern (database access abstracted)
- [ ] **Rust strict compiler settings**
  - [ ] `cargo clippy -- -D warnings` passes
  - [ ] All public APIs documented with `///` rustdoc comments
- [ ] **Format passes:** `cargo fmt --check`
- [ ] **Type check passes:** `cargo check`
- [ ] **Build succeeds:** `cargo build --release`

### Testing

- [ ] **Unit tests** with ≥80% coverage for domain logic
  - **Run:** `cargo test --lib`
- [ ] **Integration tests** for repositories
  - **Run:** `cargo test --test integration`
- [ ] **E2E tests** for API endpoints
  - **Test:** Register → Login → Create Team flow

### Security

- [ ] **Password hashing** with bcrypt
- [ ] **JWT tokens** with 8-hour expiry
- [ ] **Input validation** on all endpoints
- [ ] **No secrets in code** (all in .env)
- [ ] **Security scan:** `cargo audit` passes

### Documentation

- [ ] **API endpoints documented** in README
- [ ] **Database schema** documented with ERD
- [ ] **Domain models** documented with rustdoc

### Review

- [ ] **Pull Request created** with clear description
- [ ] **CI/CD pipeline passing** (all checks green)
- [ ] **Deployed to staging** and smoke tested

---

## 📈 Sprint Retrospective

**What Went Well ✅:** (Update at sprint end)

**What to Improve ⚠️:** (Update at sprint end)

**Action Items for Next Sprint 🎯:** (Update at sprint end)

---

## 📊 Sprint Metrics

**Velocity:** TBD at sprint end

**Code Quality:**

- **Code Coverage:** TBD (target: ≥80%)
- **Lines of Code Added:** TBD

---

## 🎯 Next Steps

**After Sprint Completion:**

1. [ ] Conduct sprint retrospective meeting
2. [ ] Deploy to staging environment
3. [ ] Begin Sprint 2 planning (Agent System implementation)

**Handoff to Sprint 2:**

- Database schema ready for agent tables
- API foundation ready for agent endpoints
- Authentication ready for user context

---

**End of Sprint 1 Document**
