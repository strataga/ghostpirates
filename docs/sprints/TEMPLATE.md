# Sprint [NUMBER] - [SPRINT NAME]

**Phase:** [Phase X of Y]
**Duration:** [X Weeks] ([Start Date] - [End Date])
**Goal:** [One-sentence sprint goal that clearly states the outcome]

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

### Progress Tracking

This document is your **single source of truth** for sprint progress. Update it continuously:

- Check off individual sub-tasks as they're completed
- Update user story completion percentages in the Progress Dashboard
- Add retrospective notes during the sprint (don't wait until the end)
- Document decisions and trade-offs as they happen

### Creating Sprint Documents with Claude Code

When creating new sprint documents or enhancing existing ones, use the specialized documentation agent:

**Agent to Use:** `documentation-generation:tutorial-engineer`

**Why This Agent:**

- Creates tutorial-style documentation with progressive skill building
- Generates comprehensive code examples with explanations
- Ensures consistent structure and granularity across sprints
- Balances technical detail with readability

**How to Use:**

```bash
# In Claude Code, use the Task tool to launch the agent:
Task: "Enhance sprint-[N]-[name].md with detailed sub-tasks, code examples, and user stories using the tutorial-engineer agent"
```

**Agent Instructions:** When launching the agent, provide:

- Link to this template (`docs/sprints/TEMPLATE.md`)
- Link to relevant research documents (`docs/research/phase-*.md`)
- Link to relevant patterns (`docs/patterns/*.md`)
- Sprint number and name
- List of user stories to expand

The agent will create comprehensive sprint documents with:

- 30-70 tasks per user story
- Complete code examples with explanations
- Step-by-step validation commands
- Integration with GhostPirates patterns and architecture

---

## 📊 Progress Dashboard

**Last Updated:** [YYYY-MM-DD HH:MM]
**Overall Sprint Progress:** [X]% Complete ([Y] of [Z] tasks done)

| User Story        | Tasks Complete | Progress             | Status         | Assignee | Blockers |
| ----------------- | -------------- | -------------------- | -------------- | -------- | -------- |
| US-[XXX]: [Title] | 0/[N] (0%)     | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |
| US-[XXX]: [Title] | 0/[N] (0%)     | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |
| US-[XXX]: [Title] | 0/[N] (0%)     | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |

**Status Legend:**

- 🔴 Not Started (0%)
- 🟡 In Progress (1-99%)
- 🟢 Complete (100%)
- 🔵 In Review (awaiting PR approval)
- 🟣 Blocked (cannot proceed)

**Update Progress Bars:** Replace `⬜` with `🟩` based on completion % (each square = 10%)

---

## 🎯 Sprint Objectives

### Primary Goal

[Clear, measurable statement of what this sprint achieves. Should answer: "At the end of this sprint, what can users do that they couldn't before?"]

**Example:** "Enable operations managers to view real-time SCADA data from 100+ wells on a live dashboard with <1-second latency, including automated alarm notifications via Microsoft Teams."

### Success Metrics

**Technical Metrics:**

- [ ] All [N] user stories completed and deployed to staging
- [ ] API response time < [X]ms (P95)
- [ ] Zero critical bugs in production
- [ ] Test coverage ≥ 80%
- [ ] All CI/CD pipelines passing

**Business Metrics:**

- [ ] [X] beta users actively using new features daily
- [ ] User satisfaction score ≥ [X]/10
- [ ] [X]% reduction in [specific pain point]
- [ ] Feature adoption rate ≥ [X]% within 2 weeks

**Quality Metrics:**

- [ ] Code review approval from ≥2 engineers
- [ ] Security scan passes (zero high/critical vulnerabilities)
- [ ] Performance benchmarks met (load testing at [X] concurrent users)
- [ ] Documentation complete (API docs, user guides, architecture diagrams)

---

## ✅ Prerequisites Checklist

> **IMPORTANT:** Complete ALL prerequisites before starting sprint work. Each prerequisite has validation steps to confirm readiness.

### Sprint Dependencies

**This sprint depends on:**

- [ ] Sprint [N-1] - [Name] **MUST BE 100% COMPLETE** before starting this sprint
  - **Validation:** Run `pnpm test` in Sprint [N-1] deliverables - all tests pass
  - **Validation:** Check Sprint [N-1] Definition of Done - all items checked
  - **Validation:** Review Sprint [N-1] retrospective for action items affecting this sprint

**Blocking items from previous sprint:**

- [ ] [Specific feature/API from Sprint N-1] deployed and tested
- [ ] [Specific database migration] applied to all environments
- [ ] [Specific infrastructure component] provisioned and accessible

### Development Environment Setup

**Required Tools:**

- [ ] Rust 1.75+ installed (`rustc --version` shows 1.75.0 or higher)
- [ ] Node.js 20+ installed (`node --version` shows v20.0.0 or higher)
- [ ] Docker Desktop running (`docker ps` returns without error)
- [ ] PostgreSQL client tools (`psql --version` shows 16.0 or higher)
- [ ] Azure CLI installed and authenticated (`az account show`)
- [ ] GitHub CLI installed and authenticated (`gh auth status`)

**Validation Steps:**

```bash
# Run this validation script to check all prerequisites
./scripts/validate-sprint-[N]-prerequisites.sh

# Expected output:
# ✅ Rust toolchain: OK (1.75.0)
# ✅ Node.js: OK (v20.11.0)
# ✅ Docker: OK (running)
# ✅ PostgreSQL: OK (16.2)
# ✅ Azure CLI: OK (authenticated as user@example.com)
# ✅ GitHub CLI: OK (authenticated as username)
# ✅ All prerequisites met!
```

### Required External Accounts & Services

- [ ] Azure subscription with required permissions (Contributor role)
  - **Validation:** `az account show` returns subscription details
  - **Validation:** Create/delete test resource group succeeds
- [ ] [Service Name] API key obtained and added to `.env.local`
  - **Validation:** `echo $[SERVICE]_API_KEY` returns non-empty value
  - **Validation:** Test API call succeeds: `curl -H "Authorization: Bearer $[SERVICE]_API_KEY" [API_ENDPOINT]`
- [ ] [Database/Service] access credentials configured
  - **Validation:** Connection test succeeds from dev environment

### Environment Variables

Copy `.env.example` to `.env.local` and configure:

- [ ] `DATABASE_URL` - PostgreSQL connection string (validated: connection succeeds)
- [ ] `REDIS_URL` - Redis connection string (validated: `redis-cli ping` returns PONG)
- [ ] `AZURE_TENANT_ID` - Azure AD tenant ID (validated: non-empty)
- [ ] `[SERVICE]_API_KEY` - External API credentials (validated: test request succeeds)

**Validation:**

```bash
# Validate all environment variables are set
./scripts/validate-env-vars.sh --sprint [N]

# Test database connectivity
psql $DATABASE_URL -c "SELECT version();"

# Test Redis connectivity
redis-cli -u $REDIS_URL ping
```

### Required Knowledge & Reading

> **⚠️ CRITICAL:** Review all relevant patterns from `docs/patterns/` BEFORE starting implementation. Using the wrong pattern or skipping patterns leads to architectural inconsistencies and technical debt.

**MUST READ - Patterns Documentation:**

- [ ] **[Pattern Catalog](../patterns/README.md)** - Full pattern index (bookmark for reference)
- [ ] **[Pattern Integration Guide](../patterns/16-Pattern-Integration-Guide.md)** - How to choose the right pattern
- [ ] **Pattern-Specific Reads** (select based on sprint requirements):
  - [ ] [Hexagonal Architecture](../patterns/*-Hexagonal-Architecture-Pattern.md) - ALWAYS required
  - [ ] [Domain-Driven Design](../patterns/*-DDD-Pattern.md) - For complex business logic
  - [ ] [CQRS Pattern](../patterns/*-CQRS-Pattern.md) - For command/query separation
  - [ ] [Repository Pattern](../patterns/*-Repository-Pattern.md) - For data access
  - [ ] [Event Sourcing](../patterns/*-Event-Sourcing-Pattern.md) - For offline/audit trails
  - [ ] [Database-Per-Tenant Multi-Tenancy](../patterns/69-Database-Per-Tenant-Multi-Tenancy-Pattern.md) - For tenant features
  - [ ] [Additional patterns as needed for sprint features]

**Why Patterns Matter:**

- Ensures architectural consistency across the codebase
- Prevents reinventing solutions (use proven patterns)
- Makes code predictable and maintainable
- Enables new team members to understand code structure quickly
- Documents architectural decisions for future reference

**MUST READ - Research & Planning:**

- [ ] [Phase 27 Master Implementation Plan](../research/phase-27-GhostPirates-master-implementation-plan.md) - Sections relevant to this sprint
- [ ] [Phase-Specific Technical Research](../research/phase-[X]-[topic].md) - Deep dive on sprint technology
- [ ] Sprint-specific research documents (see Key References section below)

**SHOULD READ (highly recommended):**

- [ ] [Rust API Architecture Analysis](../research/phase-21-rust-api-architecture-analysis.md) - Performance patterns
- [ ] [Database Architecture](../research/phase-10-database-architecture.md) - Multi-tenancy patterns
- [ ] [RBAC Design](../research/phase-4-rbac-design.md) - Permission model
- [ ] Previous sprint retrospective (Sprint [N-1]) for lessons learned and action items

**Time Estimate:** 3-4 hours to complete prerequisite reading (patterns + research) and environment setup

### Team Coordination

- [ ] Sprint planning meeting completed (acceptance criteria agreed)
- [ ] User stories assigned to team members
- [ ] External stakeholders notified (if this sprint has external dependencies)
- [ ] Capacity confirmed (all team members available for full sprint duration)

---

## 📚 Key References

### Technical Documentation

- **Architecture:** [Master Implementation Plan (Section [X])](../research/phase-27-GhostPirates-master-implementation-plan.md#[section])
- **Patterns Used:** [Pattern Catalog](../patterns/README.md) - [Specific pattern links]
- **API Specs:** [Link to relevant API documentation]
- **Database Schema:** [Link to schema documentation or ERD]

### Research Documents

- [Phase [X]: [Title]](../research/phase-[X]-[topic].md) - [What this doc provides]
- [Phase [Y]: [Title]](../research/phase-[Y]-[topic].md) - [What this doc provides]

### External Resources

- [Rust crate documentation]
- [Framework/library documentation]
- [Cloud provider documentation]

---

## 🚀 User Stories

### US-[XXX]: [Story Title]

**As a** [user role]
**I want** [functionality]
**So that** [business value/outcome]

**Business Value:** [Why this matters - impact on users, revenue, compliance, etc.]

**Acceptance Criteria:**

- [ ] [Specific, testable criterion 1 - include success measurement]
- [ ] [Specific, testable criterion 2 - include success measurement]
- [ ] [Specific, testable criterion 3 - include success measurement]
- [ ] [Performance requirement: e.g., API response < 200ms P95]
- [ ] [Security requirement: e.g., Authorization enforced on all endpoints]
- [ ] [Usability requirement: e.g., Works on mobile devices]

**Technical Implementation:**

**Domain Layer:**

- **Entities:** [EntityName] with [business rules/validation]
- **Value Objects:** [VOName] (immutable, validated)
- **Aggregates:** [AggregateName] (transaction boundary)
- **Domain Events:** [EventName] (for audit trail, notifications)

**Application Layer (CQRS):**

- **Commands:** [CommandName] (write operations with validation)
- **Queries:** [QueryName] (read operations, optimized for UI)
- **Handlers:** [HandlerName] (orchestration logic)
- **DTOs:** [DtoName] (request/response contracts)

**Infrastructure Layer:**

- **Repositories:** [RepoName] using [ORM/database library]
- **External Services:** [ServiceName] adapter (API integration)
- **Caching:** [Redis/Moka] for [specific data]
- **Background Jobs:** [Job scheduler] for [async tasks]

**Presentation Layer:**

- **API Endpoints:** [List REST/gRPC endpoints with HTTP methods]
- **Controllers:** [ControllerName] with route handlers
- **Middleware:** [Authentication, authorization, validation]
- **Frontend Components:** [ComponentName] (React/Next.js)

**Patterns Used:**

- [x] Hexagonal Architecture (ports and adapters)
- [x] Domain-Driven Design (entities, value objects, aggregates)
- [x] CQRS Pattern (command/query separation)
- [x] Repository Pattern (data access abstraction)
- [ ] [Other relevant patterns from docs/patterns/]

**File Structure:**

```
apps/api-rust/src/
├── domain/
│   └── [entity-name]/
│       ├── [entity].rs           # Domain entity
│       ├── value_objects.rs       # Value objects (e.g., Email, Location)
│       └── events.rs              # Domain events
├── application/
│   └── [entity-name]/
│       ├── commands/
│       │   └── [command_name].rs  # Command handler
│       └── queries/
│           └── [query_name].rs    # Query handler
├── infrastructure/
│   └── repositories/
│       └── [entity]_repository.rs # Repository implementation
└── presentation/
    └── [entity]_controller.rs     # API handlers
```

**Estimation:** [X] story points / [Y] hours

---

#### 📋 Sub-Tasks Breakdown

**Task Numbering:** Use format `[US-Number].[Task-Number]` (e.g., `101.1`, `101.2`) for easy reference

**Phase 1: Database Schema & Migrations** (Tasks [XXX].1 - [XXX].[N])

- [ ] **[XXX].1** - Create database migration file
  - **File:** `apps/api/src/infrastructure/database/migrations/tenant/[XXXX]_create_[table].sql`
  - **Description:** Define table schema with proper indexes, constraints, and relationships
  - **Validation:** `sqlx migrate run` succeeds without errors
  - **Estimate:** 1 hour
  - **Dependencies:** None
  - **Code Example:**

    ```sql
    CREATE TABLE [table_name] (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
      -- Add columns with proper types and constraints
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      deleted_at TIMESTAMPTZ,
      deleted_by UUID REFERENCES users(id)
    );

    -- Indexes for performance
    CREATE INDEX idx_[table]_tenant ON [table_name](tenant_id) WHERE deleted_at IS NULL;
    CREATE INDEX idx_[table]_created ON [table_name](created_at DESC);
    ```

  - **Testing:** Verify migration applies cleanly, rollback works, indexes created

- [ ] **[XXX].2** - Apply migration to dev database
  - **Command:** `cd apps/api && sqlx migrate run`
  - **Validation:** `sqlx migrate info` shows migration applied
  - **Validation:** Verify table exists: `psql $DATABASE_URL -c "\dt [table_name]"`
  - **Estimate:** 15 minutes
  - **Dependencies:** Task [XXX].1

[Continue with 20-50 detailed sub-tasks per user story...]

**Phase 2: Domain Layer Implementation** (Tasks [XXX].[N+1] - [XXX].[M])

- [ ] **[XXX].[N+1]** - Create [Entity] domain entity
  - **File:** `apps/api-rust/src/domain/[entity]/[entity].rs`
  - **Description:** Implement domain entity with business rules and validation
  - **Validation:** Unit tests pass with ≥80% coverage
  - **Estimate:** 2 hours
  - **Dependencies:** None (pure domain logic)
  - **Code Example:**

    ```rust
    #[derive(Debug, Clone)]
    pub struct [Entity] {
        id: [Entity]Id,
        // Value objects (always use value objects for validation)
        [field]: [ValueObject],
        // Primitive fields (only when no business rules)
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    impl [Entity] {
        pub fn new(
            [params]: [types],
        ) -> Result<Self, DomainError> {
            // Validate business rules
            Self::validate_[rule]([params])?;

            Ok(Self {
                id: [Entity]Id::generate(),
                [field]: [ValueObject]::new([value])?,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            })
        }

        fn validate_[rule]([params]) -> Result<(), DomainError> {
            // Business rule validation logic
        }
    }
    ```

  - **Testing:** Create `apps/api-rust/src/domain/[entity]/[entity]_tests.rs` with unit tests

[Continue with domain layer tasks...]

**Phase 3: Application Layer (CQRS)** (Tasks [XXX].[M+1] - [XXX].[P])

[Command/Query handler tasks...]

**Phase 4: Infrastructure Layer** (Tasks [XXX].[P+1] - [XXX].[Q])

[Repository, external service adapter tasks...]

**Phase 5: Presentation Layer** (Tasks [XXX].[Q+1] - [XXX].[R])

[API endpoints, controllers tasks...]

**Phase 6: Frontend Implementation** (Tasks [XXX].[R+1] - [XXX].[S])

[React components, pages, hooks tasks...]

**Phase 7: Testing** (Tasks [XXX].[S+1] - [XXX].[T])

- [ ] **[XXX].[S+1]** - Write unit tests for [Entity]
  - **File:** `apps/api-rust/src/domain/[entity]/[entity]_tests.rs`
  - **Coverage Target:** ≥80% line coverage
  - **Test Cases:**
    - Valid entity creation succeeds
    - Invalid input returns proper DomainError
    - Business rules enforced (e.g., value ranges, format validation)
    - Value objects immutable
    - Domain events emitted correctly
  - **Validation:** `cargo test [entity]_tests` passes all tests
  - **Estimate:** 2 hours

- [ ] **[XXX].[S+2]** - Write integration tests for [Repository]
  - **File:** `apps/api/test/integration/[entity]_repository.spec.ts`
  - **Test Cases:**
    - CRUD operations succeed
    - Tenant isolation enforced (cannot access other tenant's data)
    - Transactions work correctly (rollback on error)
    - Indexes improve query performance (benchmark queries)
  - **Validation:** Integration tests pass, performance benchmarks met
  - **Estimate:** 3 hours

- [ ] **[XXX].[S+3]** - Write E2E tests for [feature flow]
  - **File:** `apps/api/test/e2e/[feature]-flow.e2e-spec.ts`
  - **Test Scenario:** [Describe complete user workflow]
  - **Test Steps:**
    1. Authenticate as [role]
    2. [Action 1]
    3. [Action 2]
    4. Verify expected outcome
  - **Validation:** E2E test passes, covers critical path
  - **Estimate:** 2 hours

---

#### 🧪 Testing Strategy

**Testing Pyramid:**

- 70% Unit Tests (domain logic, value objects, business rules)
- 20% Integration Tests (repositories, external APIs, database)
- 10% E2E Tests (critical user workflows)

**Unit Tests:**

- **Location:** `apps/api-rust/src/domain/[entity]/[entity]_tests.rs`
- **Coverage Target:** ≥80% line coverage
- **Focus:** Domain entities, value objects, business rule validation
- **Run:** `cargo test --lib`

**Integration Tests:**

- **Location:** `apps/api/test/integration/`
- **Focus:** Repository layer, database queries, external API adapters
- **Environment:** Use test database (DATABASE_URL with `_test` suffix)
- **Setup:** Seed test data before each test, cleanup after
- **Run:** `cargo test --test integration`

**E2E Tests:**

- **Location:** `apps/api/test/e2e/`
- **Focus:** Complete user workflows (login → action → verify)
- **Tools:** HTTP client for API testing, database assertions
- **Run:** `cargo test --test e2e`

**Performance Tests:**

- **Load Testing:** [Tool] with [X] concurrent users, [Y] requests/second
- **Latency Target:** P95 < [X]ms
- **Throughput Target:** [Y] requests/second sustained for [Z] minutes

---

#### 📖 Technical Reference

**Dependencies (Cargo.toml):**

```toml
[dependencies]
# Core web framework
axum = "0.7"
tower = "0.4"
tower-http = "0.5"

# Database
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# [Add other relevant dependencies]
```

**API Endpoints:**

```
POST   /api/[resource]           # Create [resource]
GET    /api/[resource]           # List [resources] (with pagination, filtering)
GET    /api/[resource]/:id       # Get [resource] by ID
PUT    /api/[resource]/:id       # Update [resource]
DELETE /api/[resource]/:id       # Soft delete [resource]
```

**Database Schema:**

- **Table:** `[table_name]`
- **Columns:** [List columns with types]
- **Indexes:** [List indexes for performance]
- **Relationships:** [Foreign keys, references]

**Configuration (.env):**

```bash
# Add required environment variables
[VARIABLE_NAME]=[description]
```

---

## 🔗 Cross-Story Integration

[Explain how user stories in this sprint connect and depend on each other]

**Example:**

- US-101 (Wells Management) provides well data consumed by US-102 (Production Data)
- US-103 (RBAC) protects all endpoints in US-101, US-102, US-104
- US-104 (Reporting) aggregates data from US-101 and US-102

**Integration Tests:**

- [ ] Wells → Production integration (production entries linked to wells)
- [ ] RBAC → All features (permission checks enforced)
- [ ] Reporting → Wells + Production (reports show accurate aggregated data)

---

## 🚧 Sprint Blockers

**Active Blockers:** (Update immediately when blocked)

| Blocker ID | Task      | Issue         | Impact            | Owner   | Status     | Resolution ETA |
| ---------- | --------- | ------------- | ----------------- | ------- | ---------- | -------------- |
| B-[N]-01   | [Task ID] | [Description] | [High/Medium/Low] | [@name] | 🔴 Blocked | [Date]         |

**Resolved Blockers:** (Move here when unblocked)

| Blocker ID | Task | Issue | Resolution | Resolved By | Date |
| ---------- | ---- | ----- | ---------- | ----------- | ---- |
| -          | -    | -     | -          | -           | -    |

---

## 💬 Questions & Decisions

**Open Questions:** (Add questions as they arise, update with decisions)

| Q-[N]-01 | [Question] | **Answer:** [TBD / Decision made on [date]] | **Impact:** [Tasks affected] |
| -------- | ---------- | ------------------------------------------- | ---------------------------- |

**Decisions Made:**

| Decision   | Context            | Rationale           | Made By | Date         |
| ---------- | ------------------ | ------------------- | ------- | ------------ |
| [Decision] | [Why this came up] | [Why we chose this] | [@name] | [YYYY-MM-DD] |

---

## 🔧 Dependencies

### Sprint Dependencies

**Depends On:**

- **Sprint [N-1]**: [Specific deliverable required] - **Status:** [Complete/In Progress/Blocked]
  - **Validation:** [How to verify dependency is met]
  - **Blocker If Not Complete:** [Impact on this sprint]

**Blocks:**

- **Sprint [N+1]**: [This sprint provides] for [that sprint's feature]
  - **Critical Deliverable:** [What must be complete]

### External Dependencies

**Third-Party Services:**

- [ ] [Service Name] API access granted
  - **Status:** [Pending/Active]
  - **Contact:** [@name or support email]
  - **Documentation:** [Link]
  - **Validation:** Test API call succeeds

**Infrastructure:**

- [ ] [Resource Type] provisioned in [Environment]
  - **Status:** [Pending/Active]
  - **Owner:** [@name]
  - **Validation:** [How to verify resource is ready]

---

## ✅ Definition of Done

> **CRITICAL:** All items must be checked before sprint is considered complete.

### Code Quality

- [ ] **Follows GhostPirates patterns** from `docs/patterns/`
  - [ ] Hexagonal Architecture (domain logic isolated from infrastructure)
  - [ ] Domain-Driven Design (entities, value objects, aggregates)
  - [ ] CQRS (commands and queries separated)
  - [ ] Repository Pattern (database access abstracted)
- [ ] **Rust strict compiler settings** (deny warnings in CI)
  - [ ] `cargo clippy -- -D warnings` passes
  - [ ] No `unsafe` code without documentation and review
  - [ ] All public APIs documented with `///` rustdoc comments
- [ ] **Lint passes** (zero errors, zero warnings)
  - [ ] Rust: `cargo clippy` passes
  - [ ] Frontend: `pnpm lint` passes
- [ ] **Format passes**
  - [ ] Rust: `cargo fmt --check` passes
  - [ ] Frontend: `pnpm format:check` passes
- [ ] **Type check passes**
  - [ ] Rust: `cargo check` passes
  - [ ] Frontend: `pnpm type-check` passes
- [ ] **Build succeeds** in CI/CD
  - [ ] Rust: `cargo build --release` succeeds
  - [ ] Frontend: `pnpm build` succeeds

### Testing

- [ ] **Unit tests** written with ≥80% coverage
  - **Run:** `cargo test --lib`
  - **Coverage Report:** `cargo tarpaulin --out Html`
  - **Validation:** All tests pass, coverage report shows ≥80%
- [ ] **Integration tests** written for repositories and external services
  - **Run:** `cargo test --test integration`
  - **Validation:** All tests pass, database operations work correctly
- [ ] **E2E tests** written for critical user paths
  - **Run:** `cargo test --test e2e`
  - **Validation:** Complete workflows tested end-to-end
- [ ] **Performance tests** pass benchmarks
  - **API Latency:** P95 < [X]ms
  - **Throughput:** [Y] requests/second sustained
  - **Load Test:** [Z] concurrent users without errors
- [ ] **All tests passing** in CI/CD pipeline
  - [ ] GitHub Actions PR checks: all green
  - [ ] No flaky tests (tests pass consistently)

### Security

- [ ] **Authentication** implemented and tested
  - [ ] JWT tokens validated on protected routes
  - [ ] Token expiry enforced (8-hour timeout)
  - [ ] Refresh tokens working correctly
- [ ] **Authorization (RBAC)** enforced on all endpoints
  - [ ] Permission checks in middleware/guards
  - [ ] Tenant isolation verified (cannot access other tenant's data)
  - [ ] Role-based access control tested for all user roles
- [ ] **Audit logging** implemented for all mutations
  - [ ] Create, Update, Delete operations logged
  - [ ] Log includes: user ID, tenant ID, action, timestamp, before/after values
  - [ ] Audit logs queryable for compliance reporting
- [ ] **Input validation** on all endpoints
  - [ ] Request DTOs validated with proper error messages
  - [ ] SQL injection prevention (parameterized queries with SQLx)
  - [ ] XSS prevention (output encoding)
- [ ] **No secrets in code** (all secrets in environment variables or Azure Key Vault)
  - [ ] `.env.example` updated with placeholder values
  - [ ] No API keys, passwords, or tokens committed to git
- [ ] **Security scan passes**
  - [ ] Rust: `cargo audit` reports zero vulnerabilities
  - [ ] Dependencies: `cargo outdated` shows no critical updates needed

### Documentation

- [ ] **API endpoints documented**
  - [ ] OpenAPI/Swagger spec generated
  - [ ] Request/response examples provided
  - [ ] Error codes documented
- [ ] **Complex logic commented**
  - [ ] Domain business rules explained with `///` rustdoc comments
  - [ ] Non-obvious algorithms documented
  - [ ] Performance optimizations explained
- [ ] **Architecture decisions recorded**
  - [ ] Add ADR (Architecture Decision Record) if significant choices made
  - [ ] Update relevant pattern documentation in `docs/patterns/`
- [ ] **README updated** (if public APIs changed)
  - [ ] Setup instructions current
  - [ ] API usage examples provided
  - [ ] Troubleshooting section updated

### Review

- [ ] **Pull Request created** with clear description
  - [ ] PR title follows convention: `feat([sprint-N]): [feature description]`
  - [ ] PR description includes:
    - [ ] Summary of changes
    - [ ] Link to this sprint document
    - [ ] Screenshots/videos (for UI changes)
    - [ ] Breaking changes (if any)
    - [ ] Testing instructions for reviewers
- [ ] **Code review** completed by ≥2 engineers
  - [ ] All review comments addressed
  - [ ] No "Request Changes" blocking approval
  - [ ] All conversations resolved
- [ ] **CI/CD pipeline passing**
  - [ ] GitHub Actions: all checks green
  - [ ] Build succeeds
  - [ ] Tests pass
  - [ ] Linting passes
  - [ ] Security scans pass
- [ ] **Deployed to staging** and tested
  - [ ] Smoke tests pass in staging environment
  - [ ] Manual testing completed for critical paths
  - [ ] Database migrations applied successfully
  - [ ] No errors in application logs
- [ ] **Demo-ready**
  - [ ] Demo script prepared (step-by-step walkthrough)
  - [ ] Test data seeded for demo
  - [ ] Demo recorded (video) or presented live to stakeholders

---

## 📈 Sprint Retrospective

> **Update this section throughout the sprint, not just at the end.**

### What Went Well ✅

**Technical Wins:**

- [Achievement 1 - what worked well from a technical perspective]
- [Achievement 2 - successful pattern implementation, performance optimization, etc.]

**Process Wins:**

- [Achievement 1 - effective collaboration, helpful documentation, etc.]
- [Achievement 2 - good communication, quick blocker resolution, etc.]

**Team Wins:**

- [Achievement 1 - team member growth, knowledge sharing, etc.]

### What to Improve ⚠️

**Technical Challenges:**

- [Challenge 1 - technical debt incurred, performance issues, etc.]
- [Challenge 2 - complexity underestimated, refactoring needed, etc.]

**Process Challenges:**

- [Challenge 1 - unclear requirements, blockers, dependency issues, etc.]
- [Challenge 2 - communication gaps, review delays, etc.]

**Team Challenges:**

- [Challenge 1 - skill gaps, workload distribution, etc.]

### Action Items for Next Sprint 🎯

- [ ] **[Action 1]** - [Description of improvement]
  - **Owner:** [@name]
  - **Target:** Sprint [N+1] kickoff
  - **Success Criteria:** [How we'll know this is done]
- [ ] **[Action 2]** - [Description of improvement]
  - **Owner:** [@name]
  - **Target:** Sprint [N+1]
  - **Success Criteria:** [Measurable outcome]

### Key Learnings 💡

**Technical Learnings:**

- [Learning 1 - new pattern discovered, better approach identified, etc.]
- [Learning 2 - performance optimization technique, debugging tip, etc.]

**Process Learnings:**

- [Learning 1 - better way to structure tasks, improved documentation approach, etc.]

---

## 📊 Sprint Metrics

**Velocity:**

- **Planned Story Points:** [X] (sum of all user story estimates)
- **Completed Story Points:** [Y]
- **Velocity:** [Y/X × 100]%
- **Comparison to Sprint [N-1]:** [+/-X%]

**Code Quality:**

- **Code Coverage:** [X]% (target: ≥80%)
- **Lines of Code Added:** [X]
- **Lines of Code Deleted:** [Y]
- **Net Code Change:** [X-Y]

**CI/CD:**

- **Build Success Rate:** [X]% (target: 100%)
- **Average Build Time:** [X] minutes
- **Deployments to Staging:** [X]
- **Deployments to Production:** [X]

**Bugs & Issues:**

- **Critical Bugs:** [X] (target: 0)
- **High Priority Bugs:** [X]
- **Medium/Low Bugs:** [X]
- **Bug Fix Time (Average):** [X] hours

**Performance:**

- **API P95 Latency:** [X]ms (target: <[Y]ms)
- **API P99 Latency:** [X]ms
- **Error Rate:** [X]% (target: <0.1%)
- **Uptime:** [X]% (target: 99.9%)

**User Adoption (if deployed to beta users):**

- **Daily Active Users:** [X]
- **Feature Adoption Rate:** [X]% (users who tried new feature)
- **User Satisfaction:** [X]/10 (from in-app surveys)
- **Support Tickets:** [X] (related to this sprint's features)

---

## 📝 Sprint Notes

**Daily Standup Highlights:**

**[YYYY-MM-DD]:**

- [Key update 1]
- [Blocker resolved/identified]
- [Decision made]

**[YYYY-MM-DD]:**

- [Key update 1]

**Mid-Sprint Check-In ([Date]):**

- **Progress:** [X]% complete
- **Risks:** [Any risks identified]
- **Adjustments:** [Any scope changes, re-prioritization]

---

## 🎯 Next Steps

**After Sprint Completion:**

1. [ ] Conduct sprint retrospective meeting (60 minutes)
2. [ ] Update sprint metrics in this document
3. [ ] Archive sprint-specific branches (merge or close PRs)
4. [ ] Deploy to production (if not already done)
5. [ ] Notify stakeholders of completion
6. [ ] Begin Sprint [N+1] planning

**Handoff to Sprint [N+1]:**

- [ ] Review this sprint's "Blocks" section to identify deliverables for next sprint
- [ ] Transfer open questions/decisions to Sprint [N+1] document
- [ ] Share retrospective action items with Sprint [N+1] team
- [ ] Update master implementation plan with any architecture changes

---

## 📞 Team Contacts

**Sprint Team:**

- **Product Owner:** [@name] - [role/expertise]
- **Tech Lead:** [@name] - [role/expertise]
- **Backend Engineers:** [@name1], [@name2] - [Rust/API expertise]
- **Frontend Engineers:** [@name1], [@name2] - [React/Next.js expertise]
- **DevOps:** [@name] - [Infrastructure/CI/CD]
- **QA:** [@name] - [Testing/quality assurance]

**Stakeholders:**

- **Executive Sponsor:** [@name] - [For escalations]
- **Key Users:** [@name] - [Beta testing feedback]

**Communication Channels:**

- **Daily Standups:** [Time] in [Location/Channel]
- **Sprint Planning:** [Day/Time]
- **Retrospective:** [Day/Time]
- **Slack Channel:** #sprint-[N]-[name]

---

**End of Sprint [NUMBER] Document**
