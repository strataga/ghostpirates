# Sprint 3 - Agent System Implementation

**Phase:** Phase 3 of 9
**Duration:** 2 Weeks (Weeks 3-4)
**Goal:** Build autonomous agent system with Manager → Worker agents, LLM integration, and dynamic team formation

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
- 🔄 **Track dependencies** between tasks by referencing task numbers (e.g., "Depends on 301.5")

---

## 📊 Progress Dashboard

**Last Updated:** 2025-11-09
**Overall Sprint Progress:** 0% Complete

| User Story                              | Tasks Complete | Progress             | Status      | Assignee | Blockers |
| --------------------------------------- | -------------- | -------------------- | ----------- | -------- | -------- |
| US-301: Manager Agent Core              | 0/20 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |
| US-302: Worker Agent System             | 0/18 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |
| US-303: Claude API Integration          | 0/15 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |
| US-304: Agent Communication & Orchestration | 0/12 (0%)  | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | [@name]  | None     |

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

Build the core autonomous agent system that enables a Manager Agent to:
1. Analyze user goals using Claude API
2. Form specialized teams of 3-5 Worker Agents
3. Decompose goals into actionable tasks
4. Coordinate worker execution
5. Review and provide feedback on task outputs

At the end of this sprint, the system will have:

- Manager Agent capable of goal analysis and team formation
- Worker Agent system with dynamic specialization
- Claude API integration with prompt management
- Agent-to-agent communication system
- Basic orchestration and state management
- Foundation for task execution (implemented in Sprint 4)

### Success Metrics

**Technical Metrics:**

- [ ] Manager Agent can analyze goals via Claude API
- [ ] Team formation creates 3-5 specialized workers
- [ ] Worker agents instantiate with correct specializations
- [ ] Claude API client handles requests and responses
- [ ] Agent communication system functional
- [ ] All agent operations logged and observable
- [ ] Cost tracking for LLM API calls implemented

**Development Workflow:**

- [ ] `cargo test` passes with agent system tests
- [ ] Manager can create teams for sample goals
- [ ] API endpoints for agent operations work
- [ ] Claude API calls return structured responses
- [ ] Error handling covers API failures
- [ ] Integration tests verify agent workflows

**Quality Metrics:**

- [ ] Agent code follows hexagonal architecture
- [ ] LLM prompts are version-controlled and testable
- [ ] Agent state is properly managed
- [ ] API rate limiting and retries implemented
- [ ] Comprehensive error messages for debugging

---

## ✅ Prerequisites Checklist

> **IMPORTANT:** Complete ALL prerequisites before starting sprint work.

### Sprint Dependencies

**This sprint depends on:**

- [ ] Sprint 2 - Infrastructure **MUST BE 100% COMPLETE** before starting this sprint
  - **Validation:** `cargo test` shows 24 tests passing (7 API + 9 repository + 8 doctests)
  - **Validation:** Docker containers healthy (`docker compose ps`)
  - **Validation:** Terraform validates (`terraform validate` in environments/local and environments/dev)

### Development Environment Setup

**Required Tools:**

- [ ] Rust 1.70+ installed (`cargo --version`)
- [ ] Docker Desktop running (`docker ps` returns without error)
- [ ] PostgreSQL client tools (`psql --version`)
- [ ] Anthropic API access and API key

**Environment Variables:**

Create `apps/api/.env` with:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:54320/ghostpirates_dev
JWT_SECRET=your-secret-key-here
RUST_LOG=info,ghostpirates_api=debug
ANTHROPIC_API_KEY=sk-ant-...  # Get from https://console.anthropic.com/
```

**API Key Setup:**

- [ ] Anthropic Console account created
- [ ] API key generated and added to `.env`
- [ ] API key tested with curl:

```bash
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{
    "model": "claude-3-5-sonnet-20241022",
    "max_tokens": 1024,
    "messages": [{"role": "user", "content": "Hello"}]
  }'
```

### Required Knowledge & Reading

**MUST READ:**

- [ ] **[Anthropic API Documentation](https://docs.anthropic.com/claude/reference/messages_post)** - API structure and usage
- [ ] **[Hexagonal Architecture](../patterns/03-Hexagonal-Architecture.md)** - Agent system fits in domain layer
- [ ] **[Phase 3 Implementation Plan](../plans/05-phase-3-agent-system.md)** - Complete agent system design

**Time Estimate:** 2-3 hours to complete prerequisite reading and API key setup

---

## 📚 Key References

### Technical Documentation

- **Phase Plan:** [Phase 3: Agent System](../plans/05-phase-3-agent-system.md)
- **Anthropic API:** [Messages API Reference](https://docs.anthropic.com/claude/reference/messages_post)
- **Sprint 1:** [Foundation](./sprint-1-foundation.md) - Database and API setup
- **Sprint 2:** [Infrastructure](./sprint-2-infrastructure.md) - Terraform and local dev

### Architecture Patterns

- [Hexagonal Architecture](../patterns/03-Hexagonal-Architecture.md) - Domain isolation
- [Repository Pattern](../patterns/README.md) - Data access
- [Multi-Tenancy](../patterns/17-Multi-Tenancy-Pattern.md) - Team isolation

---

## 🚀 User Stories

### US-301: Manager Agent Core Implementation

**As a** system architect
**I want** a Manager Agent that can analyze goals and form specialized teams
**So that** the system can autonomously break down complex tasks

**Business Value:** Enables autonomous goal decomposition and team formation

**Acceptance Criteria:**

- [ ] Manager Agent struct created with Claude API client
- [ ] Goal analysis returns structured GoalAnalysis
- [ ] Team formation creates 3-5 Worker specifications
- [ ] Task decomposition generates actionable tasks
- [ ] Review logic provides specific feedback

#### 📋 Sub-Tasks Breakdown (US-301)

**Phase 1: Manager Agent Structure** (Tasks 301.1 - 301.5)

- [ ] **301.1** - Create Manager Agent module structure
  - **File:** `apps/api/src/agents/mod.rs`
  - **File:** `apps/api/src/agents/manager.rs`
  - **Content:**
    ```rust
    pub mod manager;
    pub mod worker;
    pub mod types;
    ```
  - **Validation:** Module compiles without errors
  - **Estimate:** 15 minutes

- [ ] **301.2** - Define Manager Agent struct
  - **File:** `apps/api/src/agents/manager.rs`
  - **Content:**
    ```rust
    use uuid::Uuid;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ManagerAgent {
        pub id: Uuid,
        pub team_id: Uuid,
        pub model: String,
        pub temperature: f32,
        pub max_tokens: u32,
    }

    impl ManagerAgent {
        pub fn new(team_id: Uuid) -> Self {
            Self {
                id: Uuid::new_v4(),
                team_id,
                model: "claude-3-5-sonnet-20241022".to_string(),
                temperature: 0.7,
                max_tokens: 4096,
            }
        }
    }
    ```
  - **Validation:** `cargo build` succeeds
  - **Estimate:** 20 minutes

- [ ] **301.3** - Define GoalAnalysis types
  - **File:** `apps/api/src/agents/types.rs`
  - **Content:**
    ```rust
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GoalAnalysis {
        pub core_objective: String,
        pub subtasks: Vec<String>,
        pub required_specializations: Vec<String>,
        pub estimated_timeline_hours: f32,
        pub potential_blockers: Vec<String>,
        pub success_criteria: Vec<String>,
    }
    ```
  - **Validation:** Types compile and can serialize/deserialize
  - **Estimate:** 15 minutes

- [ ] **301.4** - Implement analyze_goal skeleton
  - **Method:** `ManagerAgent::analyze_goal`
  - **Returns:** `Result<GoalAnalysis, AgentError>`
  - **Validation:** Function signature compiles
  - **Estimate:** 10 minutes
  - **Dependencies:** 301.2, 301.3

- [ ] **301.5** - Create AgentError types
  - **File:** `apps/api/src/agents/errors.rs`
  - **Content:**
    ```rust
    #[derive(Debug, thiserror::Error)]
    pub enum AgentError {
        #[error("LLM API error: {0}")]
        LlmError(String),
        #[error("Invalid team size: {0}")]
        InvalidTeamSize(usize),
        #[error("JSON parsing error: {0}")]
        JsonError(#[from] serde_json::Error),
    }
    ```
  - **Validation:** Error types compile
  - **Estimate:** 15 minutes

**Phase 2: Goal Analysis Implementation** (Tasks 301.6 - 301.10)

- [ ] **301.6** - Implement goal analysis with Claude
  - **Method:** Full implementation of `analyze_goal`
  - **Dependencies:** US-303 (Claude client)
  - **Validation:** Test with sample goal returns valid GoalAnalysis
  - **Estimate:** 1 hour

- [ ] **301.7** - Add goal analysis prompt template
  - **File:** `apps/api/src/agents/prompts.rs`
  - **Content:** System prompt for goal analysis
  - **Validation:** Prompt produces consistent JSON structure
  - **Estimate:** 30 minutes

- [ ] **301.8** - Implement response parsing
  - **Method:** Parse Claude JSON response to GoalAnalysis
  - **Validation:** Handles malformed JSON gracefully
  - **Estimate:** 30 minutes

- [ ] **301.9** - Add validation for GoalAnalysis
  - **Method:** Validate required fields are present
  - **Validation:** Rejects invalid analysis
  - **Estimate:** 20 minutes

- [ ] **301.10** - Write unit tests for goal analysis
  - **File:** `apps/api/src/agents/manager.rs` (test module)
  - **Tests:** Mock Claude responses, test parsing
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

**Phase 3: Team Formation** (Tasks 301.11 - 301.15)

- [ ] **301.11** - Define WorkerSpec type
  - **File:** `apps/api/src/agents/types.rs`
  - **Fields:** specialization, skills, responsibilities, required_tools
  - **Validation:** Type compiles
  - **Estimate:** 15 minutes

- [ ] **301.12** - Implement form_team method
  - **Method:** `ManagerAgent::form_team`
  - **Returns:** `Result<Vec<WorkerSpec>, AgentError>`
  - **Validation:** Creates 3-5 workers
  - **Estimate:** 1 hour

- [ ] **301.13** - Add team formation prompt
  - **File:** `apps/api/src/agents/prompts.rs`
  - **Content:** Prompt for worker creation
  - **Validation:** Produces valid worker specs
  - **Estimate:** 30 minutes

- [ ] **301.14** - Validate worker count (3-5)
  - **Logic:** Return error if not 3-5 workers
  - **Validation:** Test enforces limit
  - **Estimate:** 15 minutes

- [ ] **301.15** - Write tests for team formation
  - **Tests:** Various goals produce valid teams
  - **Validation:** All tests pass
  - **Estimate:** 45 minutes

**Phase 4: Task Decomposition** (Tasks 301.16 - 301.20)

- [ ] **301.16** - Implement decompose_goal method
  - **Method:** `ManagerAgent::decompose_goal`
  - **Returns:** `Result<Vec<Task>, AgentError>`
  - **Validation:** Generates tasks with acceptance criteria
  - **Estimate:** 1 hour

- [ ] **301.17** - Add task decomposition prompt
  - **File:** `apps/api/src/agents/prompts.rs`
  - **Content:** Prompt for task breakdown
  - **Validation:** Produces actionable tasks
  - **Estimate:** 30 minutes

- [ ] **301.18** - Create Task struct for agents
  - **File:** `apps/api/src/agents/types.rs`
  - **Fields:** title, description, acceptance_criteria, skills
  - **Validation:** Matches database Task model
  - **Estimate:** 20 minutes

- [ ] **301.19** - Map agent Task to database Task
  - **Logic:** Convert agent Task to repository Task
  - **Validation:** Preserves all fields correctly
  - **Estimate:** 30 minutes

- [ ] **301.20** - Write tests for task decomposition
  - **Tests:** Goal → tasks conversion
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

---

### US-302: Worker Agent System

**As a** Manager Agent
**I want** to instantiate specialized Worker Agents dynamically
**So that** tasks can be assigned to agents with appropriate skills

**Business Value:** Enables dynamic skill-based task assignment

**Acceptance Criteria:**

- [ ] Worker Agent struct with specialization field
- [ ] Workers instantiate from WorkerSpec
- [ ] Workers track assigned tasks
- [ ] Workers maintain execution state
- [ ] Workers report status to Manager

#### 📋 Sub-Tasks Breakdown (US-302)

**Phase 1: Worker Agent Structure** (Tasks 302.1 - 302.6)

- [ ] **302.1** - Create Worker Agent struct
  - **File:** `apps/api/src/agents/worker.rs`
  - **Fields:** id, team_id, specialization, skills, responsibilities, tools
  - **Validation:** Struct compiles
  - **Estimate:** 20 minutes

- [ ] **302.2** - Implement Worker::from_spec
  - **Method:** Create Worker from WorkerSpec
  - **Validation:** All fields copied correctly
  - **Estimate:** 15 minutes

- [ ] **302.3** - Add Worker status tracking
  - **Type:** WorkerStatus enum (Idle, Working, Blocked)
  - **Validation:** Status transitions work
  - **Estimate:** 20 minutes

- [ ] **302.4** - Implement Worker::assign_task
  - **Method:** Assign task to worker
  - **Validation:** Worker state updates
  - **Estimate:** 30 minutes

- [ ] **302.5** - Add Worker::get_status
  - **Method:** Return current worker status
  - **Validation:** Status reflects reality
  - **Estimate:** 15 minutes

- [ ] **302.6** - Write Worker unit tests
  - **Tests:** Creation, assignment, status
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

**Phase 2: Worker Specializations** (Tasks 302.7 - 302.12)

- [ ] **302.7** - Define Specialization enum
  - **File:** `apps/api/src/agents/types.rs`
  - **Variants:** Researcher, Coder, Reviewer, Tester, Writer
  - **Validation:** Enum compiles
  - **Estimate:** 15 minutes

- [ ] **302.8** - Add skill matching logic
  - **Method:** `Worker::can_handle_task`
  - **Logic:** Check if worker skills match task requirements
  - **Validation:** Correct matching behavior
  - **Estimate:** 30 minutes

- [ ] **302.9** - Implement worker selection
  - **Method:** `ManagerAgent::select_worker_for_task`
  - **Logic:** Find best worker for task
  - **Validation:** Selects appropriate worker
  - **Estimate:** 45 minutes

- [ ] **302.10** - Add worker load balancing
  - **Logic:** Prefer workers with fewer tasks
  - **Validation:** Tasks distributed evenly
  - **Estimate:** 30 minutes

- [ ] **302.11** - Implement worker pool management
  - **Struct:** WorkerPool to track all workers
  - **Validation:** Pool manages workers correctly
  - **Estimate:** 45 minutes

- [ ] **302.12** - Write worker selection tests
  - **Tests:** Selection logic, load balancing
  - **Validation:** Tests pass
  - **Estimate:** 1 hour

**Phase 3: Worker Execution Stubs** (Tasks 302.13 - 302.18)

- [ ] **302.13** - Create TaskOutput type
  - **File:** `apps/api/src/agents/types.rs`
  - **Fields:** result, artifacts, logs, metadata
  - **Validation:** Type compiles
  - **Estimate:** 20 minutes

- [ ] **302.14** - Implement Worker::execute_task skeleton
  - **Method:** Placeholder for task execution
  - **Returns:** `Result<TaskOutput, AgentError>`
  - **Note:** Full implementation in Sprint 4
  - **Validation:** Returns mock output
  - **Estimate:** 30 minutes

- [ ] **302.15** - Add execution logging
  - **Logic:** Log all worker actions
  - **Validation:** Logs appear in output
  - **Estimate:** 20 minutes

- [ ] **302.16** - Implement Worker::report_progress
  - **Method:** Report status to Manager
  - **Validation:** Progress updates work
  - **Estimate:** 30 minutes

- [ ] **302.17** - Add error recovery stubs
  - **Logic:** Handle task failures gracefully
  - **Validation:** Errors don't crash system
  - **Estimate:** 30 minutes

- [ ] **302.18** - Write worker execution tests
  - **Tests:** Mock task execution flow
  - **Validation:** Tests pass
  - **Estimate:** 1 hour

---

### US-303: Claude API Integration

**As a** agent developer
**I want** a robust Claude API client with prompt management
**So that** all LLM interactions are reliable and cost-tracked

**Business Value:** Enables reliable LLM integration with proper error handling and cost control

**Acceptance Criteria:**

- [ ] HTTP client for Anthropic API configured
- [ ] Request/response types defined
- [ ] Prompt templates versioned and testable
- [ ] Error handling with retries
- [ ] Cost tracking for API calls
- [ ] Rate limiting implemented

#### 📋 Sub-Tasks Breakdown (US-303)

**Phase 1: API Client Setup** (Tasks 303.1 - 303.5)

- [ ] **303.1** - Add dependencies to Cargo.toml
  - **Crates:** `reqwest`, `serde_json`
  - **Features:** `reqwest/json`
  - **Validation:** `cargo build` succeeds
  - **Estimate:** 10 minutes

- [ ] **303.2** - Create ClaudeClient struct
  - **File:** `apps/api/src/infrastructure/llm/client.rs`
  - **Fields:** api_key, http_client, model
  - **Validation:** Struct compiles
  - **Estimate:** 20 minutes

- [ ] **303.3** - Implement ClaudeClient::new
  - **Method:** Initialize with API key from env
  - **Validation:** Reads ANTHROPIC_API_KEY
  - **Estimate:** 15 minutes

- [ ] **303.4** - Define request/response types
  - **File:** `apps/api/src/infrastructure/llm/types.rs`
  - **Types:** MessageRequest, MessageResponse, Message
  - **Validation:** Types match Anthropic API spec
  - **Estimate:** 30 minutes

- [ ] **303.5** - Add API constants
  - **File:** `apps/api/src/infrastructure/llm/mod.rs`
  - **Constants:** API_URL, API_VERSION, DEFAULT_MODEL
  - **Validation:** Constants correct
  - **Estimate:** 10 minutes

**Phase 2: API Methods** (Tasks 303.6 - 303.10)

- [ ] **303.6** - Implement ClaudeClient::complete
  - **Method:** Send message, return response
  - **Validation:** Successful API call
  - **Estimate:** 1 hour

- [ ] **303.7** - Add error handling
  - **Errors:** Network, API, parsing errors
  - **Validation:** All error types handled
  - **Estimate:** 45 minutes

- [ ] **303.8** - Implement retry logic
  - **Logic:** Exponential backoff, max 3 retries
  - **Validation:** Retries on transient failures
  - **Estimate:** 45 minutes

- [ ] **303.9** - Add request timeout
  - **Timeout:** 30 seconds default
  - **Validation:** Times out correctly
  - **Estimate:** 20 minutes

- [ ] **303.10** - Write API client tests
  - **Tests:** Mock HTTP responses, test parsing
  - **Validation:** Tests pass
  - **Estimate:** 1 hour

**Phase 3: Prompt Management** (Tasks 303.11 - 303.15)

- [ ] **303.11** - Create PromptTemplate struct
  - **File:** `apps/api/src/agents/prompts.rs`
  - **Fields:** name, version, system, user_template
  - **Validation:** Struct compiles
  - **Estimate:** 20 minutes

- [ ] **303.12** - Implement template rendering
  - **Method:** `PromptTemplate::render`
  - **Logic:** Replace variables in template
  - **Validation:** Variables substituted correctly
  - **Estimate:** 45 minutes

- [ ] **303.13** - Add prompt versioning
  - **Logic:** Track prompt versions for reproducibility
  - **Validation:** Version stored with responses
  - **Estimate:** 30 minutes

- [ ] **303.14** - Create prompt library
  - **File:** `apps/api/src/agents/prompts/library.rs`
  - **Prompts:** Goal analysis, team formation, task decomposition
  - **Validation:** All prompts defined
  - **Estimate:** 1 hour

- [ ] **303.15** - Write prompt tests
  - **Tests:** Template rendering, variable substitution
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

---

### US-304: Agent Communication & Orchestration

**As a** system architect
**I want** agents to communicate and coordinate effectively
**So that** the system can orchestrate complex multi-agent workflows

**Business Value:** Enables coordinated multi-agent execution

**Acceptance Criteria:**

- [ ] Message passing system between agents
- [ ] Event system for agent coordination
- [ ] State management for agent workflows
- [ ] Logging and observability for debugging
- [ ] Cost tracking integrated

#### 📋 Sub-Tasks Breakdown (US-304)

**Phase 1: Message Passing** (Tasks 304.1 - 304.4)

- [ ] **304.1** - Define AgentMessage type
  - **File:** `apps/api/src/agents/messages.rs`
  - **Fields:** from, to, message_type, payload
  - **Validation:** Type compiles
  - **Estimate:** 20 minutes

- [ ] **304.2** - Create MessageBus struct
  - **File:** `apps/api/src/agents/messages.rs`
  - **Method:** Send/receive messages between agents
  - **Validation:** Messages delivered
  - **Estimate:** 1 hour

- [ ] **304.3** - Implement message routing
  - **Logic:** Route messages to correct agent
  - **Validation:** Routing works correctly
  - **Estimate:** 45 minutes

- [ ] **304.4** - Write message tests
  - **Tests:** Send/receive, routing
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

**Phase 2: Event System** (Tasks 304.5 - 304.8)

- [ ] **304.5** - Define AgentEvent enum
  - **File:** `apps/api/src/agents/events.rs`
  - **Variants:** TaskAssigned, TaskCompleted, WorkerCreated, etc.
  - **Validation:** Enum compiles
  - **Estimate:** 20 minutes

- [ ] **304.6** - Create EventBus
  - **File:** `apps/api/src/agents/events.rs`
  - **Method:** Publish/subscribe to events
  - **Validation:** Events delivered to subscribers
  - **Estimate:** 1 hour

- [ ] **304.7** - Add event logging
  - **Logic:** Log all events for audit trail
  - **Validation:** Events appear in logs
  - **Estimate:** 30 minutes

- [ ] **304.8** - Write event tests
  - **Tests:** Publish/subscribe, filtering
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

**Phase 3: State Management** (Tasks 304.9 - 304.12)

- [ ] **304.9** - Create AgentState type
  - **File:** `apps/api/src/agents/state.rs`
  - **Fields:** Current phase, active workers, tasks
  - **Validation:** Type compiles
  - **Estimate:** 20 minutes

- [ ] **304.10** - Implement StateManager
  - **File:** `apps/api/src/agents/state.rs`
  - **Method:** Track and update agent state
  - **Validation:** State persists correctly
  - **Estimate:** 1 hour

- [ ] **304.11** - Add state transitions
  - **Logic:** Validate state transitions
  - **Validation:** Invalid transitions rejected
  - **Estimate:** 45 minutes

- [ ] **304.12** - Write state tests
  - **Tests:** State updates, transitions
  - **Validation:** Tests pass
  - **Estimate:** 45 minutes

---

## 🧪 Testing Strategy

### Unit Tests

- **Manager Agent:** Goal analysis, team formation, task decomposition (10+ tests)
- **Worker Agent:** Creation, assignment, status tracking (8+ tests)
- **Claude Client:** Request/response, error handling, retries (8+ tests)
- **Message Bus:** Send/receive, routing (6+ tests)
- **Event System:** Publish/subscribe (6+ tests)

**Target:** 40+ unit tests, 100% pass rate

### Integration Tests

- **Manager → Claude API:** Real API calls with test prompts
- **Manager → Worker:** Team formation and assignment flow
- **Multi-agent coordination:** Message passing between agents
- **State management:** Workflow state persistence

**Target:** 10+ integration tests, 100% pass rate

### Manual Testing

- [ ] Create team for "Build a web scraper" goal
- [ ] Verify 3-5 workers created with correct specializations
- [ ] Test task decomposition for various goals
- [ ] Verify Claude API responses are properly parsed
- [ ] Test error handling with invalid API keys
- [ ] Monitor cost tracking for API calls

---

## 🚧 Sprint Blockers

> Document any blockers here immediately when they arise

**Current Blockers:** None

**Example Format:**
- **Blocker:** Description of issue
- **Impact:** Which tasks are blocked
- **Owner:** Who is resolving it
- **Status:** In progress / Resolved / Escalated

---

## ❓ Questions & Decisions

> Document technical questions and decisions made during the sprint

**Open Questions:** None

**Decisions Made:** None yet

**Example Format:**
- **Question:** What should we do about X?
- **Decision:** We decided Y because Z
- **Date:** 2025-11-09
- **Participants:** [@names]

---

## 📈 Sprint Retrospective

> Fill out at end of sprint

### What Went Well

-

### What Could Be Improved

-

### Action Items for Next Sprint

-

---

## 🎯 Sprint Summary

**Total User Stories:** 4
**Total Tasks:** 65
**Estimated Duration:** 2 weeks

**Sprint Goal:** Build autonomous agent system with Manager → Worker agents, LLM integration, and dynamic team formation

**Next Sprint:** Sprint 4 - Task Orchestration & Execution

---

**🤖 Sprint 3: Autonomous Agents Online**
