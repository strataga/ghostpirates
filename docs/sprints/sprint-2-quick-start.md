# Sprint 2 Quick Start Guide

## 🚀 Getting Started

### 1. Prerequisites Checklist

Before starting Sprint 2, verify:

```bash
# Check Sprint 1 is complete
cd /Users/jason/projects/ghostpirates/apps/api
cargo build --release  # Should succeed
sqlx migrate info      # Should show 8 migrations applied
curl http://localhost:3000/health  # Should return "OK"

# Check Anthropic API key
echo $ANTHROPIC_API_KEY  # Should show sk-ant-...
```

### 2. Environment Setup

Add to `.env`:

```bash
# LLM Configuration
ANTHROPIC_API_KEY=sk-ant-your-key-here
ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
MAX_TOKENS_PER_REQUEST=4096
LLM_DEFAULT_TEMPERATURE=0.7
```

### 3. Read Required Patterns (3-4 hours)

Priority order:
1. ✅ Pattern Integration Guide (30 min)
2. ✅ Hexagonal Architecture (45 min)
3. ✅ Domain-Driven Design (60 min)
4. ✅ Anti-Corruption Layer (30 min)
5. ✅ External API Cost Tracking (30 min)

### 4. Sprint Overview

**Total:** 170 tasks across 4 user stories
**Duration:** 2 weeks
**Team:** 2-3 backend engineers recommended

## 📋 User Story Execution Order

### Week 1

#### US-201: LLM Client Infrastructure (Days 1-2, 38 tasks)

**Phase 1: Database** (30 min)
```bash
# Create token_usage_log migration
cd apps/api
sqlx migrate add create_token_usage_log
# Add SQL from task 201.1
sqlx migrate run
```

**Phase 2: Domain Layer** (3 hours)
- Create value objects: `Model`, `TokenCount`, `Cost`
- Implement `PromptBuilder`
- Write unit tests

**Phase 3: Infrastructure** (6 hours)
- Build `ClaudeClient`
- Implement retry logic
- Create token tracking repository

**Phase 4: Application Layer** (3 hours)
- `SendLLMRequest` command
- `GetUsageByTeam` query
- Integration tests

**Validation:**
```bash
cargo test domain::llm
cargo test infrastructure::llm
ANTHROPIC_API_KEY=<key> cargo test --test llm -- --ignored
```

#### US-202: Manager Agent Core (Days 3-5, 48 tasks)

**Phase 1: Database** (1 hour)
```bash
# Create 3 tables
sqlx migrate add create_manager_agents
sqlx migrate add create_goal_analyses
sqlx migrate add create_worker_specifications
sqlx migrate run
```

**Phase 2: Domain Layer** (4 hours)
- `ManagerAgent` aggregate
- `GoalAnalysis`, `WorkerSpecification` entities
- Domain events
- Repository traits

**Phase 3: Infrastructure** (6 hours)
- PostgreSQL repositories
- Prompt templates (goal analysis, team formation, task decomposition)

**Phase 4: Application Layer** (4 hours)
- `CreateManagerAgent` command
- `AnalyzeGoal`, `FormTeam`, `DecomposeTasks` commands
- Queries

**Phase 5: API Layer** (3 hours)
- HTTP handlers
- DTOs
- Routes

**Validation:**
```bash
cargo test domain::agents::manager
cargo test --test manager_agent
curl -X POST http://localhost:3000/api/teams/{id}/manager
```

### Week 2

#### US-203: Worker Agent System (Days 1-2, 42 tasks)

**Key Tasks:**
- Create `worker_agents` table
- Implement `WorkerAgent` aggregate
- Build `WorkerFactory`
- Specialization-specific prompts
- API endpoints

**Validation:**
```bash
cargo test domain::agents::worker
curl -X POST http://localhost:3000/api/teams/{id}/workers
```

#### US-204: Review and Revision Loops (Days 3-4, 42 tasks)

**Key Tasks:**
- Create `task_reviews` table
- Implement `TaskReview` aggregate
- Review/revision commands
- Review prompt templates
- API endpoints

**Validation:**
```bash
cargo test domain::agents::review
# Test full workflow: Create → Execute → Review → Revise → Approve
```

#### Day 5: Integration & Testing

**Final Validation:**
```bash
# Run all tests
cargo test
cargo clippy -- -D warnings
cargo fmt --check

# Run smoke tests
./scripts/test_llm_integration.sh
./scripts/test_manager_agent.sh
./scripts/test_worker_agents.sh
./scripts/test_review_system.sh

# Test E2E workflow
# 1. Create team
# 2. Create manager agent
# 3. Analyze goal
# 4. Form worker team
# 5. Decompose tasks
# 6. Create workers
# 7. Execute task
# 8. Review output
# 9. Request revision
# 10. Approve
```

## 💡 Pro Tips

### Development Workflow

1. **Start with domain layer** - Pure business logic, no dependencies
2. **TDD for domain** - Write tests first, then implementation
3. **Integration tests for infrastructure** - Test real database/API
4. **Mock LLM for unit tests** - Use test fixtures for prompts/responses

### Cost Management

```rust
// Always track token usage
let response = send_llm_request(command).await?;
println!("Cost: ${}", response.cost.total());

// Use prompt caching for system prompts
let (system, messages) = PromptBuilder::new()
    .system("Your system prompt here")  // Will be cached
    .user("User message")
    .build()?;
```

### Error Handling

```rust
// Domain errors
pub enum AgentError {
    InvalidConfiguration(String),
    MaxWorkersExceeded,
    RevisionLimitExceeded,
}

// Infrastructure errors
pub enum ClaudeError {
    ApiError(String),
    RateLimit,
    NetworkError(reqwest::Error),
}
```

### Testing Strategy

```bash
# Unit tests (fast, no I/O)
cargo test --lib

# Integration tests (slower, real DB/API)
cargo test --test integration

# E2E tests (slowest, full workflows)
cargo test --test e2e

# Ignore tests requiring API key by default
#[tokio::test]
#[ignore]
async fn test_with_real_api() { ... }

# Run ignored tests when API key available
ANTHROPIC_API_KEY=<key> cargo test -- --ignored
```

## 🎯 Daily Checklist

### Every Morning
- [ ] Pull latest changes
- [ ] Review Progress Dashboard in sprint doc
- [ ] Check for blockers
- [ ] Update your task assignments

### Every Evening
- [ ] Check off completed tasks in sprint doc
- [ ] Update Progress Dashboard percentages
- [ ] Commit and push code
- [ ] Document any blockers or decisions
- [ ] Run tests before EOD

### Every 2-3 Days
- [ ] Run full test suite
- [ ] Check test coverage
- [ ] Review API costs (token usage)
- [ ] Update sprint metrics

## 🆘 Troubleshooting

### Common Issues

**Issue: `cargo build` fails with missing types**
```bash
# Solution: Ensure all modules exported
# Check src/domain/mod.rs, src/infrastructure/mod.rs, etc.
```

**Issue: SQLx compile-time verification fails**
```bash
# Solution: Run migrations first
sqlx migrate run

# Or use offline mode
cargo sqlx prepare
```

**Issue: LLM tests fail with auth error**
```bash
# Solution: Check API key
echo $ANTHROPIC_API_KEY

# Or set in .env
cat .env | grep ANTHROPIC_API_KEY
```

**Issue: High LLM costs**
```bash
# Solution: Check token usage
psql $DATABASE_URL -c "SELECT SUM(total_cost) FROM token_usage_log;"

# Enable prompt caching
# Check cached_tokens > 0 in logs
```

**Issue: Tests timeout on LLM calls**
```bash
# Solution: Increase timeout or use mocks
#[tokio::test]
#[timeout(Duration::from_secs(60))]
async fn test_llm_call() { ... }
```

## 📊 Progress Tracking

Update daily in `/Users/jason/projects/ghostpirates/docs/sprints/sprint-2-agent-system.md`:

```markdown
| User Story                      | Tasks Complete | Progress             | Status      |
| ------------------------------- | -------------- | -------------------- | ----------- |
| US-201: LLM Client              | 15/38 (39%)    | 🟩🟩🟩🟩⬜⬜⬜⬜⬜⬜ | 🟡 Progress |
```

## 🎉 Definition of Done

Sprint 2 complete when:
- [ ] All 170 tasks checked off
- [ ] All tests passing
- [ ] Code reviewed and approved
- [ ] Documentation complete
- [ ] Deployed to staging
- [ ] Demo successful
- [ ] Sprint retrospective held

## 📚 Resources

**Main Document:** `sprint-2-agent-system.md` (3,575 lines)
**Summary:** `sprint-2-summary.md`
**This Guide:** `sprint-2-quick-start.md`

**External:**
- [Anthropic API Docs](https://docs.anthropic.com)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [SQLx Guide](https://github.com/launchbadge/sqlx)

---

**Ready to start?** Begin with US-201, Task 201.1! 🚀
