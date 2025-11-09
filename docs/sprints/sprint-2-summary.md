# Sprint 2 Summary: Agent System Implementation

## Overview

**Duration:** 2 Weeks (Weeks 3-4)
**Status:** Ready to Start
**Total Tasks:** 170 tasks across 4 user stories

## User Stories Breakdown

### US-201: LLM Client Infrastructure (38 tasks, 16 hours)
**Goal:** Robust Claude API client with error handling and token tracking

**Key Deliverables:**
- Claude API client with streaming support
- Token usage tracking and cost calculation
- Retry logic with exponential backoff
- Prompt caching for 90% cost reduction
- Integration tests with real API

**Technologies:**
- Reqwest for HTTP client
- Rust async/await
- PostgreSQL for usage tracking
- Claude 3.5 Sonnet API

### US-202: Manager Agent Core (48 tasks, 20 hours)
**Goal:** Autonomous manager agent for goal analysis and team formation

**Key Deliverables:**
- Goal analysis using Claude API
- Team formation (3-5 specialized workers)
- Task decomposition into concrete tasks
- Manager agent persistence
- Prompt templates for analysis

**Technologies:**
- Domain-Driven Design patterns
- CQRS for commands/queries
- PostgreSQL for agent state
- LLM prompt engineering

### US-203: Worker Agent System (42 tasks, 18 hours)
**Goal:** Dynamic worker agent creation and task execution

**Key Deliverables:**
- Worker agents with specializations
- Task assignment and execution
- Specialization-specific prompts
- Worker state management
- Maximum 5 workers per team

**Technologies:**
- Factory pattern for worker creation
- Strategy pattern for specializations
- LLM-powered task execution

### US-204: Review and Revision Loops (42 tasks, 18 hours)
**Goal:** Quality control through manager review and worker revisions

**Key Deliverables:**
- Manager reviews worker output
- Revision requests with feedback
- Maximum 3 revision rounds
- Review history tracking
- Automatic approval workflow

**Technologies:**
- State machine for review states
- Event sourcing for history
- LLM-powered quality assessment

## Sprint Objectives

### Primary Goal
Enable autonomous agent teams to analyze goals, form specialized worker teams, decompose tasks, execute work using Claude API, and iterate based on manager review feedback.

### Success Metrics
- [ ] All 4 user stories completed and tested
- [ ] API response time < 500ms for non-LLM endpoints (P95)
- [ ] LLM calls complete within 30 seconds (P95)
- [ ] Test coverage ≥ 80% for domain logic
- [ ] Manager agent successfully analyzes 100% of valid goals
- [ ] Review loop reduces task quality issues by 70%
- [ ] Cost tracking accurate within $0.01 per request

## Prerequisites

### Required Before Starting
- [x] Sprint 1 - Foundation 100% complete
- [ ] Anthropic API key obtained
- [ ] Environment variables configured
- [ ] All pattern documentation read

### Critical Reading
- Pattern Integration Guide
- Hexagonal Architecture Pattern
- Domain-Driven Design Pattern
- Anti-Corruption Layer Pattern
- External API Cost Tracking Pattern

## Architecture

### Layered Structure (Hexagonal Architecture)

**Domain Layer:**
- `ManagerAgent` aggregate
- `WorkerAgent` aggregate
- `GoalAnalysis`, `WorkerSpecification`, `TaskReview` entities
- Value objects: `Model`, `TokenCount`, `Cost`, `Specialization`
- Domain events for all agent operations

**Application Layer:**
- Commands: `CreateManagerAgent`, `AnalyzeGoal`, `FormTeam`, `ExecuteTask`, `ReviewTask`
- Queries: `GetManagerAgent`, `GetUsageByTeam`, `GetReviewHistory`
- Command/Query handlers with business logic

**Infrastructure Layer:**
- `ClaudeClient` with retry logic
- PostgreSQL repositories
- Prompt templates
- Token usage tracking

**Presentation Layer:**
- REST API endpoints for agent operations
- DTOs for request/response
- Error handling middleware

## Database Changes

### New Tables (6 total)
1. `token_usage_log` - Track all LLM API calls
2. `manager_agents` - Manager agent state
3. `goal_analyses` - Cached goal analyses
4. `worker_specifications` - Worker blueprints
5. `worker_agents` - Worker agent state
6. `task_reviews` - Review history

## API Endpoints

### Manager Agent
- `POST /api/teams/:id/manager` - Create manager
- `POST /api/teams/:id/analyze` - Analyze goal
- `POST /api/teams/:id/form-team` - Generate workers
- `POST /api/teams/:id/decompose` - Create tasks
- `GET /api/teams/:id/manager` - Get manager

### Worker Agent
- `POST /api/teams/:id/workers` - Create worker
- `GET /api/teams/:id/workers` - List workers
- `POST /api/workers/:id/execute` - Execute task

### Review System
- `POST /api/tasks/:id/review` - Review output
- `POST /api/tasks/:id/approve` - Approve task
- `POST /api/tasks/:id/reject` - Reject task
- `GET /api/tasks/:id/reviews` - Review history

## Cost Optimization Strategies

1. **Prompt Caching:** System prompts cached for 90% cost reduction
2. **Token Limits:** Max 4096 tokens per request configurable
3. **Usage Tracking:** All requests logged with costs
4. **Smart Retries:** Only retry on transient errors
5. **Batch Operations:** Where possible, combine LLM calls

## Testing Strategy

### Unit Tests (60%)
- Domain entities and value objects
- Business rule validation
- Cost calculations
- Prompt building

### Integration Tests (30%)
- Real Anthropic API calls
- Repository CRUD operations
- Retry logic
- Error handling

### E2E Tests (10%)
- Full agent workflows
- Manager → Workers → Review cycles
- Cost tracking accuracy
- Concurrent operations

## Risk Mitigation

### LLM Reliability
- **Risk:** API downtime or rate limits
- **Mitigation:** Retry logic with exponential backoff, fallback strategies

### Cost Control
- **Risk:** Unexpected high costs
- **Mitigation:** Token limits, usage tracking, budget alerts

### Quality Assurance
- **Risk:** Poor LLM output quality
- **Mitigation:** Review loops, acceptance criteria, revision limits

### Performance
- **Risk:** Slow LLM responses
- **Mitigation:** Async operations, timeout configuration, caching

## Success Criteria

Sprint 2 is complete when:
- [ ] All 170 tasks completed
- [ ] All tests passing (unit, integration, E2E)
- [ ] Documentation complete
- [ ] Deployed to staging
- [ ] Demo successful
- [ ] Code reviewed and approved
- [ ] Performance benchmarks met
- [ ] Security scan passes

## Next Sprint Preview

**Sprint 3: Task Orchestration**
- Task queue management
- Parallel worker execution
- Dependency resolution
- Progress tracking
- Real-time updates

## Documentation Deliverables

1. `llm-client-guide.md` - Using the LLM client
2. `manager-agent-guide.md` - Manager agent usage
3. `worker-agent-guide.md` - Worker agent system
4. `worker-specializations.md` - Specialization catalog
5. `review-system-guide.md` - Review and revision loops
6. `worker-prompt-examples.md` - Prompt engineering examples
7. `review-prompt-guide.md` - Review prompt patterns

## Key Files Created

### Domain Layer (20+ files)
- `src/domain/llm/` - LLM value objects
- `src/domain/agents/manager/` - Manager agent
- `src/domain/agents/worker/` - Worker agent
- `src/domain/agents/review/` - Review system

### Infrastructure Layer (15+ files)
- `src/infrastructure/llm/` - Claude client
- `src/infrastructure/prompts/` - Prompt templates
- `src/infrastructure/repositories/` - PostgreSQL repos

### Application Layer (25+ files)
- `src/application/llm/` - LLM commands/queries
- `src/application/agents/manager/` - Manager operations
- `src/application/agents/worker/` - Worker operations
- `src/application/agents/review/` - Review operations

### API Layer (10+ files)
- `src/api/handlers/` - HTTP handlers
- `src/api/dto/` - Request/response DTOs

## Estimated Timeline

**Week 1:**
- Days 1-2: US-201 LLM Client Infrastructure
- Days 3-5: US-202 Manager Agent Core (Part 1)

**Week 2:**
- Days 1-2: US-202 Manager Agent Core (Part 2)
- Days 3-4: US-203 Worker Agent System
- Day 5: US-204 Review and Revision Loops

## Team Allocation

**Recommended:** 2-3 backend engineers

**Skills Required:**
- Rust proficiency
- Async programming
- LLM/AI experience helpful
- Domain-Driven Design knowledge
- API design expertise

---

**Document:** `/Users/jason/projects/ghostpirates/docs/sprints/sprint-2-agent-system.md`
**Lines:** 3,575
**Tasks:** 170
**Estimated Effort:** 72 hours (9 days for 2 engineers, 4.5 days for 4 engineers)
