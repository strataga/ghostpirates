# Sprint 2 - Agent System: Manager & Worker Agents with LLM Integration

**Phase:** Phase 2 of 8
**Duration:** 2 Weeks (Weeks 3-4)
**Goal:** Implement autonomous agent system with Claude API integration, manager agent for goal analysis and team formation, worker agents for task execution, and review/revision loops

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
- 🔄 **Track dependencies** between tasks by referencing task numbers (e.g., "Depends on 201.15")

---

## 📊 Progress Dashboard

**Last Updated:** 2025-11-08
**Overall Sprint Progress:** 0% Complete (0 of 170 tasks done)

| User Story                              | Tasks Complete | Progress             | Status         | Assignee | Blockers |
| --------------------------------------- | -------------- | -------------------- | -------------- | -------- | -------- |
| US-201: LLM Client Infrastructure       | 0/38 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-202: Manager Agent Core              | 0/48 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-203: Worker Agent System             | 0/42 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |
| US-204: Review and Revision Loops       | 0/42 (0%)      | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | 🔴 Not Started | -        | None     |

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

Enable autonomous agent teams to analyze goals, form specialized worker teams, decompose tasks, execute work using Claude API, and iterate based on manager review feedback.

At the end of this sprint, the system will:

- Accept a natural language goal from users
- Use Claude API to analyze goals and create execution plans
- Dynamically generate 3-5 specialized worker agents
- Decompose goals into concrete, measurable tasks
- Execute tasks with LLM-powered workers
- Review worker output and request revisions when needed
- Track token usage and costs across all LLM interactions

### Success Metrics

**Technical Metrics:**

- [ ] All 4 user stories completed and tested
- [ ] API response time < 500ms for non-LLM endpoints (P95)
- [ ] LLM calls complete within 30 seconds (P95)
- [ ] Zero critical bugs in staging
- [ ] Test coverage ≥ 80% for domain logic
- [ ] All CI/CD pipelines passing

**Business Metrics:**

- [ ] Manager agent successfully analyzes 100% of valid goals
- [ ] Team formation generates 3-5 workers consistently
- [ ] Task decomposition produces actionable tasks (100% have acceptance criteria)
- [ ] Review loop reduces task quality issues by 70%
- [ ] Cost tracking accurate within $0.01 per request

**Quality Metrics:**

- [ ] Code follows Hexagonal Architecture pattern
- [ ] Domain models use DDD principles (agents as aggregates)
- [ ] Prompt engineering patterns documented
- [ ] Token usage optimized (< 50k tokens per goal on average)
- [ ] Error handling covers all LLM failure modes

---

## ✅ Prerequisites Checklist

> **IMPORTANT:** Complete ALL prerequisites before starting sprint work.

### Sprint Dependencies

**This sprint depends on:**

- [x] Sprint 1 - Foundation **MUST BE 100% COMPLETE** before starting this sprint
  - **Validation:** Run `cargo build` in `apps/api` - build succeeds
  - **Validation:** Run `sqlx migrate info` - all 8 migrations applied
  - **Validation:** `curl http://localhost:3000/health` returns "OK"

**Blocking items from previous sprint:**

- [x] Database schema includes teams, users, tasks, messages tables
- [x] API server runs and handles requests
- [x] Authentication endpoints functional (register, login)

### Development Environment Setup

**Required Tools:**

- [x] Rust 1.75+ installed (`rustc --version` shows 1.75.0 or higher)
- [x] PostgreSQL client tools (`psql --version` shows 16.0 or higher)
- [x] SQLx CLI installed (`cargo install sqlx-cli --features postgres`)
- [x] Docker Desktop running (for local testing)
- [ ] Anthropic API key obtained (sign up at console.anthropic.com)

**Validation Steps:**

```bash
# Verify Sprint 1 complete
cd apps/api
cargo build --release  # Should succeed
sqlx migrate info      # Should show 8 applied migrations

# Verify Anthropic API key
echo $ANTHROPIC_API_KEY  # Should show your key

# Test API connection (optional)
curl https://api.anthropic.com/v1/messages \
  -H "x-api-key: $ANTHROPIC_API_KEY" \
  -H "anthropic-version: 2023-06-01" \
  -H "content-type: application/json" \
  -d '{"model":"claude-3-5-sonnet-20241022","max_tokens":10,"messages":[{"role":"user","content":"Hi"}]}'
```

### Required External Accounts & Services

- [ ] Anthropic API account created at console.anthropic.com
  - **Validation:** API key obtained and stored in `.env`
  - **Validation:** Test API call succeeds (see above)
  - **Pricing:** Claude 3.5 Sonnet: $3/MTok input, $15/MTok output
- [x] PostgreSQL database from Sprint 1 accessible
  - **Validation:** `psql $DATABASE_URL -c "SELECT COUNT(*) FROM teams;"`

### Environment Variables

Add to `.env` file:

- [x] `DATABASE_URL` - PostgreSQL connection string (from Sprint 1)
- [x] `JWT_SECRET` - JWT signing key (from Sprint 1)
- [ ] `ANTHROPIC_API_KEY` - Your Anthropic API key
- [ ] `ANTHROPIC_MODEL` - Default model (claude-3-5-sonnet-20241022)
- [ ] `MAX_TOKENS_PER_REQUEST` - Safety limit (default: 4096)

**Validation:**

```bash
# Add to .env
cat >> .env << 'EOF'
ANTHROPIC_API_KEY=sk-ant-your-key-here
ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
MAX_TOKENS_PER_REQUEST=4096
EOF

# Verify all env vars set
source .env
echo $ANTHROPIC_API_KEY | grep -q "sk-ant" && echo "✅ Anthropic key set"
```

### Required Knowledge & Reading

> **⚠️ CRITICAL:** Review all relevant patterns from `docs/patterns/` BEFORE starting implementation.

**MUST READ - Patterns Documentation:**

- [ ] **[Pattern Integration Guide](../patterns/16-Pattern-Integration-Guide.md)** - READ FIRST
- [ ] **[Hexagonal Architecture](../patterns/03-Hexagonal-Architecture.md)** - REQUIRED for all features
- [ ] **[Domain-Driven Design](../patterns/04-Domain-Driven-Design.md)** - Agent as aggregate root
- [ ] **[Repository Pattern](../patterns/06-Repository-Pattern.md)** - Agent persistence
- [ ] **[CQRS Pattern](../patterns/05-CQRS-Pattern.md)** - Command/Query separation
- [ ] **[Anti-Corruption Layer](../patterns/14-Anti-Corruption-Layer-Pattern.md)** - LLM API wrapper
- [ ] **[Retry Pattern](../patterns/15-Retry-Pattern.md)** - LLM retry logic
- [ ] **[External API Cost Tracking](../patterns/66-External-API-Cost-Tracking-with-Prompt-Caching.md)** - Token tracking

**MUST READ - Research & Planning:**

- [ ] [Phase 2: Agent System](../plans/05-phase-2-agent-system.md) - Complete phase details
- [ ] [Technology Stack](../plans/01-technology-stack.md) - Architecture decisions
- [ ] Sprint 1 Retrospective - Lessons learned from foundation phase

**SHOULD READ (highly recommended):**

- [ ] [Rust Testing Patterns](../patterns/40-Rust-Testing-Patterns.md) - Testing async code
- [ ] [Security Patterns Guide](../patterns/39-Security-Patterns-Guide.md) - API key management
- [ ] [Monitoring Patterns](../patterns/47-Monitoring-Observability-Patterns.md) - LLM call tracking

**Time Estimate:** 4-5 hours to complete prerequisite reading and environment setup

---

## 📚 Key References

### Technical Documentation

- **Architecture:** [Hexagonal Architecture Pattern](../patterns/03-Hexagonal-Architecture.md)
- **Patterns Used:** [Pattern Integration Guide](../patterns/16-Pattern-Integration-Guide.md)
- **LLM Integration:** [Anthropic API Documentation](https://docs.anthropic.com/claude/reference/messages_post)
- **Prompt Engineering:** [Anthropic Prompt Library](https://docs.anthropic.com/claude/page/prompts)

### Research Documents

- [Phase 2: Agent System](../plans/05-phase-2-agent-system.md) - Detailed implementation plan
- [Phase 1: Foundation](../plans/04-phase-1-foundation.md) - Sprint 1 reference
- [Technology Stack](../plans/01-technology-stack.md) - Tech decisions

### External Resources

- [Claude API Reference](https://docs.anthropic.com/claude/reference/messages_post)
- [Reqwest HTTP Client](https://docs.rs/reqwest/latest/reqwest/)
- [Rust Async Programming](https://rust-lang.github.io/async-book/)

---

## 🚀 User Stories

### US-201: LLM Client Infrastructure

**As a** backend developer
**I want** a robust Claude API client with error handling and token tracking
**So that** agents can make LLM calls reliably and cost-effectively

**Business Value:** Foundation for all agent intelligence, enables accurate cost tracking, provides resilient LLM integration

**Acceptance Criteria:**

- [ ] Claude API client supports streaming and non-streaming responses
- [ ] Token usage tracked for every request (input + output + cached tokens)
- [ ] Cost calculated automatically using Anthropic pricing
- [ ] Retry logic handles rate limits (429) and transient errors (500, 502, 503, 504)
- [ ] Exponential backoff with jitter (3 retries max)
- [ ] Error types distinguish API errors, network errors, deserialization errors
- [ ] Prompt caching enabled for system prompts (reduces costs by 90% on cache hits)
- [ ] All LLM calls logged with request ID for debugging
- [ ] Integration tests validate real API calls (using test key)

**Technical Implementation:**

**Domain Layer:**

- **Value Objects:** 
  - `Model` - Validated model name (claude-3-5-sonnet-20241022, etc.)
  - `TokenCount` - Input/output/cached token counts
  - `Cost` - Calculated cost in USD
- **Domain Services:**
  - `PromptBuilder` - Constructs prompts with role separation
  - `TokenEstimator` - Estimates tokens before API call

**Application Layer (CQRS):**

- **Commands:** 
  - `SendLLMRequest` - Execute LLM call with retry logic
  - `TrackUsage` - Record token usage in database
- **Queries:** 
  - `GetUsageByTeam` - Aggregate token/cost by team
  - `GetUsageByTimeRange` - Usage analytics

**Infrastructure Layer:**

- **External Services:** 
  - `ClaudeApiClient` - HTTP client wrapping Anthropic API
  - `RetryPolicy` - Exponential backoff implementation
- **Repositories:**
  - `TokenUsageRepository` - Persist usage data
- **Adapters:**
  - `AnthropicAdapter` - Anti-corruption layer translating domain to API

**Presentation Layer:**

- **API Endpoints:** None (internal service only)

**Patterns Used:**

- [x] Hexagonal Architecture (LLM client in infrastructure layer)
- [x] Anti-Corruption Layer (protect domain from API changes)
- [x] Retry Pattern (handle transient failures)
- [x] Repository Pattern (persist token usage)
- [x] Value Object Pattern (Model, TokenCount, Cost)

**File Structure:**

```
apps/api/src/
├── domain/
│   └── llm/
│       ├── mod.rs
│       ├── model.rs              # Model value object
│       ├── token_count.rs        # TokenCount value object
│       ├── cost.rs               # Cost calculations
│       └── prompt_builder.rs     # Prompt construction
├── application/
│   └── llm/
│       ├── commands/
│       │   ├── send_llm_request.rs
│       │   └── track_usage.rs
│       └── queries/
│           └── get_usage.rs
├── infrastructure/
│   ├── llm/
│   │   ├── claude_client.rs      # HTTP client
│   │   ├── retry_policy.rs       # Retry logic
│   │   └── anthropic_adapter.rs  # API adapter
│   └── repositories/
│       └── token_usage_repository.rs
```

**Estimation:** 16 hours (38 tasks)

---

#### 📋 Sub-Tasks Breakdown (US-201)

**Phase 1: Database Schema for Token Tracking** (Tasks 201.1 - 201.3)

- [ ] **201.1** - Create token_usage_log table migration
  - **File:** `migrations/20251109000001_create_token_usage_log.sql`
  - **Description:** Track all LLM API calls with token counts and costs
  - **Code:**

    ```sql
    CREATE TABLE token_usage_log (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
        agent_id UUID NOT NULL,  -- manager or worker agent
        agent_type VARCHAR(50) NOT NULL,  -- 'manager' or 'worker'
        
        -- Request details
        model VARCHAR(100) NOT NULL,
        request_id VARCHAR(255),  -- Anthropic request ID
        prompt_cached BOOLEAN NOT NULL DEFAULT false,
        
        -- Token counts
        input_tokens INTEGER NOT NULL,
        output_tokens INTEGER NOT NULL,
        cached_tokens INTEGER NOT NULL DEFAULT 0,
        
        -- Cost tracking (in USD)
        input_cost DECIMAL(10, 6) NOT NULL,
        output_cost DECIMAL(10, 6) NOT NULL,
        cache_savings DECIMAL(10, 6) NOT NULL DEFAULT 0,
        total_cost DECIMAL(10, 6) NOT NULL,
        
        -- Metadata
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        
        CONSTRAINT positive_tokens CHECK (
            input_tokens >= 0 AND 
            output_tokens >= 0 AND 
            cached_tokens >= 0
        ),
        CONSTRAINT positive_costs CHECK (
            input_cost >= 0 AND 
            output_cost >= 0 AND 
            total_cost >= 0
        )
    );

    -- Indexes for analytics
    CREATE INDEX idx_token_usage_team ON token_usage_log(team_id);
    CREATE INDEX idx_token_usage_agent ON token_usage_log(agent_id);
    CREATE INDEX idx_token_usage_created ON token_usage_log(created_at DESC);
    CREATE INDEX idx_token_usage_team_created ON token_usage_log(team_id, created_at DESC);
    ```

  - **Validation:** Migration applies cleanly
  - **Estimate:** 30 minutes
  - **Dependencies:** None

- [ ] **201.2** - Apply migration to dev database
  - **Command:** `cd apps/api && sqlx migrate run`
  - **Validation:** `sqlx migrate info` shows migration applied
  - **Validation:** `psql $DATABASE_URL -c "\d token_usage_log"` shows table structure
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.1

- [ ] **201.3** - Create sample usage data for testing
  - **File:** `apps/api/scripts/seed_token_usage.sql`
  - **Description:** Insert test data to validate queries
  - **Code:**

    ```sql
    -- Insert sample team and agent for testing
    INSERT INTO companies (id, name) VALUES 
        ('00000000-0000-0000-0000-000000000001', 'Test Company');
    
    INSERT INTO users (id, company_id, email, password_hash, full_name) VALUES 
        ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 
         'test@example.com', 'hash', 'Test User');
    
    INSERT INTO teams (id, company_id, goal, status, created_by) VALUES 
        ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001',
         'Test Goal', 'planning', '00000000-0000-0000-0000-000000000002');
    
    -- Insert sample token usage
    INSERT INTO token_usage_log 
        (team_id, agent_id, agent_type, model, input_tokens, output_tokens, 
         input_cost, output_cost, total_cost)
    VALUES 
        ('00000000-0000-0000-0000-000000000003', 
         '00000000-0000-0000-0000-000000000004',
         'manager', 'claude-3-5-sonnet-20241022',
         1000, 500, 0.003000, 0.007500, 0.010500);
    ```

  - **Validation:** Query succeeds, aggregation works
  - **Estimate:** 15 minutes
  - **Dependencies:** Task 201.2

**Phase 2: Domain Layer - Value Objects** (Tasks 201.4 - 201.10)

- [ ] **201.4** - Create Model value object
  - **File:** `src/domain/llm/model.rs`
  - **Description:** Validated LLM model identifier with pricing information
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};
    use std::fmt;

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Model(String);

    impl Model {
        // Anthropic pricing (as of Nov 2024)
        const SONNET_3_5_INPUT_PER_MTOK: f64 = 3.00;
        const SONNET_3_5_OUTPUT_PER_MTOK: f64 = 15.00;
        const SONNET_3_5_CACHED_PER_MTOK: f64 = 0.30;  // 90% discount

        pub fn new(model: impl Into<String>) -> Result<Self, String> {
            let model = model.into();
            
            if !Self::is_valid(&model) {
                return Err(format!("Invalid model: {}", model));
            }
            
            Ok(Model(model))
        }

        fn is_valid(model: &str) -> bool {
            matches!(
                model,
                "claude-3-5-sonnet-20241022" | 
                "claude-3-5-sonnet-20240620" |
                "claude-3-opus-20240229" |
                "claude-3-haiku-20240307"
            )
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }

        pub fn default_sonnet() -> Self {
            Model("claude-3-5-sonnet-20241022".to_string())
        }

        pub fn input_cost_per_token(&self) -> f64 {
            match self.0.as_str() {
                "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => 
                    Self::SONNET_3_5_INPUT_PER_MTOK / 1_000_000.0,
                _ => 0.0,
            }
        }

        pub fn output_cost_per_token(&self) -> f64 {
            match self.0.as_str() {
                "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => 
                    Self::SONNET_3_5_OUTPUT_PER_MTOK / 1_000_000.0,
                _ => 0.0,
            }
        }

        pub fn cached_cost_per_token(&self) -> f64 {
            match self.0.as_str() {
                "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-20240620" => 
                    Self::SONNET_3_5_CACHED_PER_MTOK / 1_000_000.0,
                _ => 0.0,
            }
        }
    }

    impl fmt::Display for Model {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn valid_model() {
            assert!(Model::new("claude-3-5-sonnet-20241022").is_ok());
        }

        #[test]
        fn invalid_model() {
            assert!(Model::new("gpt-4").is_err());
        }

        #[test]
        fn pricing_calculation() {
            let model = Model::default_sonnet();
            assert_eq!(model.input_cost_per_token(), 3.00 / 1_000_000.0);
            assert_eq!(model.output_cost_per_token(), 15.00 / 1_000_000.0);
        }
    }
    ```

  - **Validation:** `cargo test domain::llm::model` passes
  - **Estimate:** 45 minutes
  - **Dependencies:** None

- [ ] **201.5** - Create TokenCount value object
  - **File:** `src/domain/llm/token_count.rs`
  - **Description:** Immutable token counts with validation
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TokenCount {
        input: u32,
        output: u32,
        cached: u32,
    }

    impl TokenCount {
        pub fn new(input: u32, output: u32, cached: u32) -> Result<Self, String> {
            // Basic validation
            if input == 0 && output == 0 {
                return Err("Token count cannot be zero for both input and output".to_string());
            }

            Ok(TokenCount {
                input,
                output,
                cached,
            })
        }

        pub fn input(&self) -> u32 {
            self.input
        }

        pub fn output(&self) -> u32 {
            self.output
        }

        pub fn cached(&self) -> u32 {
            self.cached
        }

        pub fn total(&self) -> u32 {
            self.input + self.output
        }

        pub fn billable_input(&self) -> u32 {
            // Cached tokens are billed at reduced rate
            self.input.saturating_sub(self.cached)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn valid_token_count() {
            let tokens = TokenCount::new(1000, 500, 200).unwrap();
            assert_eq!(tokens.input(), 1000);
            assert_eq!(tokens.output(), 500);
            assert_eq!(tokens.cached(), 200);
            assert_eq!(tokens.total(), 1500);
            assert_eq!(tokens.billable_input(), 800);
        }

        #[test]
        fn zero_tokens_fails() {
            assert!(TokenCount::new(0, 0, 0).is_err());
        }
    }
    ```

  - **Validation:** Unit tests pass
  - **Estimate:** 30 minutes
  - **Dependencies:** None

- [ ] **201.6** - Create Cost value object
  - **File:** `src/domain/llm/cost.rs`
  - **Description:** Calculate and represent API costs
  - **Code:**

    ```rust
    use rust_decimal::Decimal;
    use serde::{Deserialize, Serialize};
    use super::{Model, TokenCount};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Cost {
        input_cost: Decimal,
        output_cost: Decimal,
        cache_savings: Decimal,
    }

    impl Cost {
        pub fn calculate(model: &Model, tokens: &TokenCount) -> Self {
            let input_cost = Decimal::from_f64_retain(
                tokens.billable_input() as f64 * model.input_cost_per_token()
            ).unwrap_or(Decimal::ZERO);

            let output_cost = Decimal::from_f64_retain(
                tokens.output() as f64 * model.output_cost_per_token()
            ).unwrap_or(Decimal::ZERO);

            let full_input_cost = Decimal::from_f64_retain(
                tokens.input() as f64 * model.input_cost_per_token()
            ).unwrap_or(Decimal::ZERO);

            let cached_cost = Decimal::from_f64_retain(
                tokens.cached() as f64 * model.cached_cost_per_token()
            ).unwrap_or(Decimal::ZERO);

            let cache_savings = full_input_cost - input_cost - cached_cost;

            Cost {
                input_cost,
                output_cost,
                cache_savings,
            }
        }

        pub fn total(&self) -> Decimal {
            self.input_cost + self.output_cost
        }

        pub fn input_cost(&self) -> Decimal {
            self.input_cost
        }

        pub fn output_cost(&self) -> Decimal {
            self.output_cost
        }

        pub fn cache_savings(&self) -> Decimal {
            self.cache_savings
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn cost_calculation() {
            let model = Model::default_sonnet();
            let tokens = TokenCount::new(1000, 500, 0).unwrap();
            let cost = Cost::calculate(&model, &tokens);
            
            // Input: 1000 tokens * $3/MTok = $0.003
            // Output: 500 tokens * $15/MTok = $0.0075
            // Total: $0.0105
            assert!(cost.total() > Decimal::ZERO);
        }

        #[test]
        fn cache_savings_calculation() {
            let model = Model::default_sonnet();
            let tokens = TokenCount::new(1000, 500, 800).unwrap();
            let cost = Cost::calculate(&model, &tokens);
            
            // Cache savings should be positive
            assert!(cost.cache_savings() > Decimal::ZERO);
        }
    }
    ```

  - **Dependencies:** Add `rust_decimal = { version = "1.33", features = ["serde"] }` to Cargo.toml
  - **Validation:** Unit tests pass
  - **Estimate:** 45 minutes
  - **Dependencies:** Tasks 201.4, 201.5

- [ ] **201.7** - Create PromptBuilder domain service
  - **File:** `src/domain/llm/prompt_builder.rs`
  - **Description:** Construct prompts with proper role separation
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Message {
        pub role: Role,
        pub content: String,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    pub enum Role {
        User,
        Assistant,
    }

    pub struct PromptBuilder {
        system: Option<String>,
        messages: Vec<Message>,
    }

    impl PromptBuilder {
        pub fn new() -> Self {
            Self {
                system: None,
                messages: Vec::new(),
            }
        }

        pub fn system(mut self, content: impl Into<String>) -> Self {
            self.system = Some(content.into());
            self
        }

        pub fn user(mut self, content: impl Into<String>) -> Self {
            self.messages.push(Message {
                role: Role::User,
                content: content.into(),
            });
            self
        }

        pub fn assistant(mut self, content: impl Into<String>) -> Self {
            self.messages.push(Message {
                role: Role::Assistant,
                content: content.into(),
            });
            self
        }

        pub fn build(self) -> Result<(Option<String>, Vec<Message>), String> {
            if self.messages.is_empty() {
                return Err("At least one message required".to_string());
            }

            // Validate alternating roles (API requirement)
            for window in self.messages.windows(2) {
                if window[0].role == window[1].role {
                    return Err("Messages must alternate between user and assistant".to_string());
                }
            }

            Ok((self.system, self.messages))
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn build_simple_prompt() {
            let (system, messages) = PromptBuilder::new()
                .system("You are a helpful assistant")
                .user("Hello")
                .build()
                .unwrap();

            assert_eq!(system, Some("You are a helpful assistant".to_string()));
            assert_eq!(messages.len(), 1);
            assert_eq!(messages[0].role, Role::User);
        }

        #[test]
        fn empty_messages_fails() {
            assert!(PromptBuilder::new().build().is_err());
        }
    }
    ```

  - **Validation:** Unit tests pass
  - **Estimate:** 45 minutes
  - **Dependencies:** None

- [ ] **201.8** - Add LLM domain module exports
  - **File:** `src/domain/llm/mod.rs`
  - **Code:**

    ```rust
    mod model;
    mod token_count;
    mod cost;
    mod prompt_builder;

    pub use model::Model;
    pub use token_count::TokenCount;
    pub use cost::Cost;
    pub use prompt_builder::{PromptBuilder, Message, Role};
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Tasks 201.4-201.7

- [ ] **201.9** - Update domain/mod.rs
  - **File:** `src/domain/mod.rs`
  - **Code:**

    ```rust
    pub mod team;
    pub mod user;
    pub mod task;
    pub mod llm;  // Add this line
    pub mod repositories;
    ```

  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.8

- [ ] **201.10** - Write unit tests for all domain types
  - **Command:** `cargo test domain::llm`
  - **Coverage Target:** ≥80%
  - **Validation:** All tests pass
  - **Estimate:** 30 minutes
  - **Dependencies:** Tasks 201.4-201.9

**Phase 3: Infrastructure Layer - Claude API Client** (Tasks 201.11 - 201.20)

- [ ] **201.11** - Add HTTP client dependencies
  - **File:** `Cargo.toml`
  - **Code:**

    ```toml
    [dependencies]
    # Existing dependencies...
    
    # HTTP client
    reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
    
    # Decimal arithmetic
    rust_decimal = { version = "1.33", features = ["serde"] }
    
    # Rate limiting
    governor = "0.6"
    ```

  - **Validation:** `cargo check` passes
  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [ ] **201.12** - Create Claude API request/response types
  - **File:** `src/infrastructure/llm/types.rs`
  - **Description:** DTOs for Anthropic API
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub struct ClaudeRequest {
        pub model: String,
        pub max_tokens: u32,
        pub system: Option<String>,
        pub messages: Vec<ClaudeMessage>,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        pub temperature: Option<f32>,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct ClaudeMessage {
        pub role: String,
        pub content: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct ClaudeResponse {
        pub id: String,
        pub model: String,
        pub role: String,
        pub content: Vec<ContentBlock>,
        pub usage: Usage,
    }

    #[derive(Debug, Deserialize)]
    pub struct ContentBlock {
        #[serde(rename = "type")]
        pub content_type: String,
        pub text: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    pub struct Usage {
        pub input_tokens: u32,
        pub output_tokens: u32,
        
        #[serde(default)]
        pub cache_creation_input_tokens: u32,
        
        #[serde(default)]
        pub cache_read_input_tokens: u32,
    }

    #[derive(Debug, Deserialize)]
    pub struct ClaudeError {
        #[serde(rename = "type")]
        pub error_type: String,
        pub message: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct ClaudeErrorResponse {
        pub error: ClaudeError,
    }
    ```

  - **Estimate:** 30 minutes
  - **Dependencies:** None

- [ ] **201.13** - Create ClaudeClient implementation
  - **File:** `src/infrastructure/llm/claude_client.rs`
  - **Description:** HTTP client for Anthropic API
  - **Code:**

    ```rust
    use reqwest::{Client, StatusCode};
    use crate::domain::llm::{Model, TokenCount, Message};
    use super::types::*;

    const API_BASE_URL: &str = "https://api.anthropic.com/v1";
    const API_VERSION: &str = "2023-06-01";

    #[derive(Clone)]
    pub struct ClaudeClient {
        client: Client,
        api_key: String,
    }

    #[derive(Debug)]
    pub struct LLMResponse {
        pub content: String,
        pub tokens: TokenCount,
        pub request_id: Option<String>,
    }

    #[derive(Debug, thiserror::Error)]
    pub enum ClaudeError {
        #[error("API error: {0}")]
        ApiError(String),
        
        #[error("Rate limit exceeded")]
        RateLimit,
        
        #[error("Network error: {0}")]
        NetworkError(#[from] reqwest::Error),
        
        #[error("Deserialization error: {0}")]
        DeserializationError(String),
        
        #[error("Invalid response: {0}")]
        InvalidResponse(String),
    }

    impl ClaudeClient {
        pub fn new(api_key: String) -> Self {
            Self {
                client: Client::new(),
                api_key,
            }
        }

        pub async fn complete(
            &self,
            model: &Model,
            system: Option<String>,
            messages: Vec<Message>,
            max_tokens: u32,
            temperature: Option<f32>,
        ) -> Result<LLMResponse, ClaudeError> {
            let request = ClaudeRequest {
                model: model.as_str().to_string(),
                max_tokens,
                system,
                messages: messages
                    .into_iter()
                    .map(|m| ClaudeMessage {
                        role: match m.role {
                            crate::domain::llm::Role::User => "user".to_string(),
                            crate::domain::llm::Role::Assistant => "assistant".to_string(),
                        },
                        content: m.content,
                    })
                    .collect(),
                temperature,
            };

            let response = self
                .client
                .post(format!("{}/messages", API_BASE_URL))
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION)
                .header("content-type", "application/json")
                .json(&request)
                .send()
                .await?;

            let request_id = response
                .headers()
                .get("request-id")
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            match response.status() {
                StatusCode::OK => {
                    let claude_response: ClaudeResponse = response
                        .json()
                        .await
                        .map_err(|e| ClaudeError::DeserializationError(e.to_string()))?;

                    let content = claude_response
                        .content
                        .into_iter()
                        .find_map(|block| block.text)
                        .ok_or_else(|| ClaudeError::InvalidResponse("No text content".to_string()))?;

                    let tokens = TokenCount::new(
                        claude_response.usage.input_tokens,
                        claude_response.usage.output_tokens,
                        claude_response.usage.cache_read_input_tokens,
                    ).map_err(|e| ClaudeError::InvalidResponse(e))?;

                    Ok(LLMResponse {
                        content,
                        tokens,
                        request_id,
                    })
                }
                StatusCode::TOO_MANY_REQUESTS => Err(ClaudeError::RateLimit),
                _ => {
                    let error_response: ClaudeErrorResponse = response
                        .json()
                        .await
                        .map_err(|e| ClaudeError::DeserializationError(e.to_string()))?;
                    Err(ClaudeError::ApiError(error_response.error.message))
                }
            }
        }
    }
    ```

  - **Dependencies:** Add `thiserror = "1.0"` to Cargo.toml
  - **Estimate:** 90 minutes
  - **Dependencies:** Tasks 201.11, 201.12

- [ ] **201.14** - Create RetryPolicy implementation
  - **File:** `src/infrastructure/llm/retry_policy.rs`
  - **Description:** Exponential backoff with jitter
  - **Code:**

    ```rust
    use std::time::Duration;
    use tokio::time::sleep;
    use rand::Rng;

    pub struct RetryPolicy {
        max_retries: u32,
        base_delay_ms: u64,
    }

    impl RetryPolicy {
        pub fn new(max_retries: u32, base_delay_ms: u64) -> Self {
            Self {
                max_retries,
                base_delay_ms,
            }
        }

        pub fn default() -> Self {
            Self::new(3, 1000)  // 3 retries, 1 second base delay
        }

        pub async fn execute<F, T, E>(&self, mut operation: F) -> Result<T, E>
        where
            F: FnMut() -> futures::future::BoxFuture<'static, Result<T, E>>,
            E: std::fmt::Debug,
        {
            let mut attempt = 0;

            loop {
                match operation().await {
                    Ok(result) => return Ok(result),
                    Err(error) => {
                        attempt += 1;
                        
                        if attempt > self.max_retries {
                            return Err(error);
                        }

                        let delay = self.calculate_delay(attempt);
                        tracing::warn!(
                            "Attempt {}/{} failed: {:?}. Retrying in {}ms",
                            attempt,
                            self.max_retries,
                            error,
                            delay.as_millis()
                        );
                        
                        sleep(delay).await;
                    }
                }
            }
        }

        fn calculate_delay(&self, attempt: u32) -> Duration {
            // Exponential backoff: base_delay * 2^(attempt-1)
            let exponential_delay = self.base_delay_ms * 2_u64.pow(attempt - 1);
            
            // Add jitter (±25%)
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.75..1.25);
            let delay_with_jitter = (exponential_delay as f64 * jitter_factor) as u64;
            
            // Cap at 30 seconds
            let capped_delay = delay_with_jitter.min(30_000);
            
            Duration::from_millis(capped_delay)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn delay_increases_exponentially() {
            let policy = RetryPolicy::default();
            
            let delay1 = policy.calculate_delay(1);
            let delay2 = policy.calculate_delay(2);
            let delay3 = policy.calculate_delay(3);
            
            // Delays should increase (accounting for jitter)
            assert!(delay2 > delay1);
            assert!(delay3 > delay2);
        }
    }
    ```

  - **Dependencies:** Add `rand = "0.8"` and `futures = "0.3"` to Cargo.toml
  - **Estimate:** 60 minutes
  - **Dependencies:** None

- [ ] **201.15** - Create ClaudeClient with retry logic
  - **File:** `src/infrastructure/llm/resilient_client.rs`
  - **Description:** Wrap ClaudeClient with retry policy
  - **Code:**

    ```rust
    use crate::domain::llm::{Model, TokenCount, Message};
    use super::claude_client::{ClaudeClient, ClaudeError, LLMResponse};
    use super::retry_policy::RetryPolicy;

    pub struct ResilientClaudeClient {
        client: ClaudeClient,
        retry_policy: RetryPolicy,
    }

    impl ResilientClaudeClient {
        pub fn new(api_key: String) -> Self {
            Self {
                client: ClaudeClient::new(api_key),
                retry_policy: RetryPolicy::default(),
            }
        }

        pub async fn complete(
            &self,
            model: &Model,
            system: Option<String>,
            messages: Vec<Message>,
            max_tokens: u32,
            temperature: Option<f32>,
        ) -> Result<LLMResponse, ClaudeError> {
            let client = self.client.clone();
            let model = model.clone();
            let system_clone = system.clone();
            let messages_clone = messages.clone();

            self.retry_policy
                .execute(|| {
                    let client = client.clone();
                    let model = model.clone();
                    let system = system_clone.clone();
                    let messages = messages_clone.clone();

                    Box::pin(async move {
                        client.complete(&model, system, messages, max_tokens, temperature).await
                    })
                })
                .await
        }
    }
    ```

  - **Estimate:** 30 minutes
  - **Dependencies:** Tasks 201.13, 201.14

- [ ] **201.16** - Create infrastructure LLM module exports
  - **File:** `src/infrastructure/llm/mod.rs`
  - **Code:**

    ```rust
    mod types;
    mod claude_client;
    mod retry_policy;
    mod resilient_client;

    pub use claude_client::{ClaudeClient, ClaudeError, LLMResponse};
    pub use retry_policy::RetryPolicy;
    pub use resilient_client::ResilientClaudeClient;
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Tasks 201.12-201.15

- [ ] **201.17** - Update infrastructure/mod.rs
  - **File:** `src/infrastructure/mod.rs`
  - **Code:**

    ```rust
    pub mod repositories;
    pub mod llm;  // Add this line
    ```

  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.16

- [ ] **201.18** - Create integration test for Claude API
  - **File:** `tests/llm/claude_client_test.rs`
  - **Description:** Test real API calls (requires API key)
  - **Code:**

    ```rust
    use ghostpirates_api::infrastructure::llm::ResilientClaudeClient;
    use ghostpirates_api::domain::llm::{Model, PromptBuilder};

    #[tokio::test]
    #[ignore]  // Run only when ANTHROPIC_API_KEY is set
    async fn test_simple_completion() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("ANTHROPIC_API_KEY must be set");

        let client = ResilientClaudeClient::new(api_key);
        let model = Model::default_sonnet();

        let (system, messages) = PromptBuilder::new()
            .system("You are a helpful assistant")
            .user("Say hello in exactly 2 words")
            .build()
            .unwrap();

        let response = client
            .complete(&model, system, messages, 100, Some(0.7))
            .await
            .expect("API call failed");

        assert!(!response.content.is_empty());
        assert!(response.tokens.input() > 0);
        assert!(response.tokens.output() > 0);
        
        println!("Response: {}", response.content);
        println!("Tokens: {:?}", response.tokens);
    }

    #[tokio::test]
    #[ignore]
    async fn test_retry_on_rate_limit() {
        // Test that retry logic works
        // (Difficult to test without actually hitting rate limits)
    }
    ```

  - **Validation:** `cargo test --test llm -- --ignored` passes (when API key set)
  - **Estimate:** 30 minutes
  - **Dependencies:** Tasks 201.13-201.17

- [ ] **201.19** - Create TokenUsageRepository trait
  - **File:** `src/domain/repositories/token_usage_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    use crate::domain::llm::{Model, TokenCount, Cost};

    pub struct TokenUsageRecord {
        pub team_id: Uuid,
        pub agent_id: Uuid,
        pub agent_type: String,
        pub model: Model,
        pub tokens: TokenCount,
        pub cost: Cost,
        pub request_id: Option<String>,
        pub created_at: DateTime<Utc>,
    }

    #[async_trait]
    pub trait TokenUsageRepository: Send + Sync {
        async fn record_usage(&self, record: TokenUsageRecord) -> Result<(), String>;
        
        async fn get_team_usage(
            &self,
            team_id: Uuid,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> Result<Vec<TokenUsageRecord>, String>;
        
        async fn get_total_cost(
            &self,
            team_id: Uuid,
        ) -> Result<rust_decimal::Decimal, String>;
    }
    ```

  - **Estimate:** 20 minutes
  - **Dependencies:** None

- [ ] **201.20** - Implement PostgreSQL TokenUsageRepository
  - **File:** `src/infrastructure/repositories/postgres_token_usage_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;
    use uuid::Uuid;
    use rust_decimal::Decimal;
    use crate::domain::repositories::token_usage_repository::{TokenUsageRepository, TokenUsageRecord};
    use crate::domain::llm::{Model, TokenCount, Cost};

    pub struct PostgresTokenUsageRepository {
        pool: PgPool,
    }

    impl PostgresTokenUsageRepository {
        pub fn new(pool: PgPool) -> Self {
            Self { pool }
        }
    }

    #[async_trait]
    impl TokenUsageRepository for PostgresTokenUsageRepository {
        async fn record_usage(&self, record: TokenUsageRecord) -> Result<(), String> {
            sqlx::query!(
                r#"
                INSERT INTO token_usage_log 
                    (team_id, agent_id, agent_type, model, request_id,
                     input_tokens, output_tokens, cached_tokens,
                     input_cost, output_cost, cache_savings, total_cost)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                "#,
                record.team_id,
                record.agent_id,
                record.agent_type,
                record.model.as_str(),
                record.request_id,
                record.tokens.input() as i32,
                record.tokens.output() as i32,
                record.tokens.cached() as i32,
                record.cost.input_cost(),
                record.cost.output_cost(),
                record.cost.cache_savings(),
                record.cost.total(),
            )
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            Ok(())
        }

        async fn get_team_usage(
            &self,
            team_id: Uuid,
            start: DateTime<Utc>,
            end: DateTime<Utc>,
        ) -> Result<Vec<TokenUsageRecord>, String> {
            let rows = sqlx::query!(
                r#"
                SELECT agent_id, agent_type, model, request_id,
                       input_tokens, output_tokens, cached_tokens,
                       input_cost, output_cost, cache_savings, created_at
                FROM token_usage_log
                WHERE team_id = $1 AND created_at BETWEEN $2 AND $3
                ORDER BY created_at DESC
                "#,
                team_id,
                start,
                end
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            let records = rows
                .into_iter()
                .map(|row| {
                    let model = Model::new(row.model).unwrap();
                    let tokens = TokenCount::new(
                        row.input_tokens as u32,
                        row.output_tokens as u32,
                        row.cached_tokens as u32,
                    ).unwrap();
                    let cost = Cost::calculate(&model, &tokens);

                    TokenUsageRecord {
                        team_id,
                        agent_id: row.agent_id,
                        agent_type: row.agent_type,
                        model,
                        tokens,
                        cost,
                        request_id: row.request_id,
                        created_at: row.created_at,
                    }
                })
                .collect();

            Ok(records)
        }

        async fn get_total_cost(&self, team_id: Uuid) -> Result<Decimal, String> {
            let row = sqlx::query!(
                r#"
                SELECT COALESCE(SUM(total_cost), 0) as "total_cost!"
                FROM token_usage_log
                WHERE team_id = $1
                "#,
                team_id
            )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

            Ok(row.total_cost)
        }
    }
    ```

  - **Estimate:** 60 minutes
  - **Dependencies:** Tasks 201.1-201.3, 201.19

**Phase 4: Application Layer - Commands & Queries** (Tasks 201.21 - 201.30)

- [ ] **201.21** - Create SendLLMRequest command
  - **File:** `src/application/llm/commands/send_llm_request.rs`
  - **Description:** Execute LLM call and track usage
  - **Code:**

    ```rust
    use uuid::Uuid;
    use crate::domain::llm::{Model, Message, Cost};
    use crate::domain::repositories::token_usage_repository::{TokenUsageRepository, TokenUsageRecord};
    use crate::infrastructure::llm::{ResilientClaudeClient, ClaudeError};
    use chrono::Utc;

    pub struct SendLLMRequestCommand {
        pub team_id: Uuid,
        pub agent_id: Uuid,
        pub agent_type: String,
        pub model: Model,
        pub system: Option<String>,
        pub messages: Vec<Message>,
        pub max_tokens: u32,
        pub temperature: Option<f32>,
    }

    pub struct SendLLMRequestResult {
        pub content: String,
        pub cost: Cost,
    }

    pub struct SendLLMRequestHandler {
        client: ResilientClaudeClient,
        usage_repo: Box<dyn TokenUsageRepository>,
    }

    impl SendLLMRequestHandler {
        pub fn new(
            client: ResilientClaudeClient,
            usage_repo: Box<dyn TokenUsageRepository>,
        ) -> Self {
            Self { client, usage_repo }
        }

        pub async fn handle(
            &self,
            command: SendLLMRequestCommand,
        ) -> Result<SendLLMRequestResult, ClaudeError> {
            // Execute LLM call
            let response = self.client.complete(
                &command.model,
                command.system,
                command.messages,
                command.max_tokens,
                command.temperature,
            ).await?;

            // Calculate cost
            let cost = Cost::calculate(&command.model, &response.tokens);

            // Record usage
            let usage_record = TokenUsageRecord {
                team_id: command.team_id,
                agent_id: command.agent_id,
                agent_type: command.agent_type,
                model: command.model,
                tokens: response.tokens,
                cost,
                request_id: response.request_id,
                created_at: Utc::now(),
            };

            if let Err(e) = self.usage_repo.record_usage(usage_record).await {
                tracing::error!("Failed to record token usage: {}", e);
                // Don't fail the request, just log the error
            }

            Ok(SendLLMRequestResult {
                content: response.content,
                cost,
            })
        }
    }
    ```

  - **Estimate:** 45 minutes
  - **Dependencies:** Tasks 201.15, 201.20

- [ ] **201.22** - Create GetUsageByTeam query
  - **File:** `src/application/llm/queries/get_usage_by_team.rs`
  - **Code:**

    ```rust
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    use rust_decimal::Decimal;
    use crate::domain::repositories::token_usage_repository::TokenUsageRepository;

    pub struct GetUsageByTeamQuery {
        pub team_id: Uuid,
        pub start: DateTime<Utc>,
        pub end: DateTime<Utc>,
    }

    pub struct UsageSummary {
        pub total_requests: usize,
        pub total_input_tokens: u32,
        pub total_output_tokens: u32,
        pub total_cached_tokens: u32,
        pub total_cost: Decimal,
    }

    pub struct GetUsageByTeamHandler {
        usage_repo: Box<dyn TokenUsageRepository>,
    }

    impl GetUsageByTeamHandler {
        pub fn new(usage_repo: Box<dyn TokenUsageRepository>) -> Self {
            Self { usage_repo }
        }

        pub async fn handle(&self, query: GetUsageByTeamQuery) -> Result<UsageSummary, String> {
            let records = self.usage_repo
                .get_team_usage(query.team_id, query.start, query.end)
                .await?;

            let summary = UsageSummary {
                total_requests: records.len(),
                total_input_tokens: records.iter().map(|r| r.tokens.input()).sum(),
                total_output_tokens: records.iter().map(|r| r.tokens.output()).sum(),
                total_cached_tokens: records.iter().map(|r| r.tokens.cached()).sum(),
                total_cost: records.iter().map(|r| r.cost.total()).sum(),
            };

            Ok(summary)
        }
    }
    ```

  - **Estimate:** 30 minutes
  - **Dependencies:** Task 201.20

- [ ] **201.23** - Create application LLM module structure
  - **Command:** `mkdir -p src/application/llm/{commands,queries}`
  - **Files:**
    - `src/application/llm/mod.rs`
    - `src/application/llm/commands/mod.rs`
    - `src/application/llm/queries/mod.rs`
  - **Estimate:** 10 minutes
  - **Dependencies:** Tasks 201.21, 201.22

- [ ] **201.24** - Add application module exports
  - **File:** `src/application/mod.rs`
  - **Code:**

    ```rust
    pub mod llm;
    ```

  - **File:** `src/application/llm/mod.rs`
  - **Code:**

    ```rust
    pub mod commands;
    pub mod queries;

    pub use commands::send_llm_request::{SendLLMRequestCommand, SendLLMRequestHandler, SendLLMRequestResult};
    pub use queries::get_usage_by_team::{GetUsageByTeamQuery, GetUsageByTeamHandler, UsageSummary};
    ```

  - **Estimate:** 10 minutes
  - **Dependencies:** Task 201.23

- [ ] **201.25** - Write integration tests for SendLLMRequest
  - **File:** `tests/application/send_llm_request_test.rs`
  - **Code:**

    ```rust
    use ghostpirates_api::application::llm::{SendLLMRequestCommand, SendLLMRequestHandler};
    use ghostpirates_api::domain::llm::{Model, PromptBuilder};
    use ghostpirates_api::infrastructure::llm::ResilientClaudeClient;
    use ghostpirates_api::infrastructure::repositories::PostgresTokenUsageRepository;
    use sqlx::PgPool;
    use uuid::Uuid;

    #[sqlx::test]
    #[ignore]  // Requires API key
    async fn test_send_request_and_track_usage(pool: PgPool) {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        let client = ResilientClaudeClient::new(api_key);
        let usage_repo = Box::new(PostgresTokenUsageRepository::new(pool.clone()));

        let handler = SendLLMRequestHandler::new(client, usage_repo);

        let (system, messages) = PromptBuilder::new()
            .system("You are helpful")
            .user("Say hi")
            .build()
            .unwrap();

        let command = SendLLMRequestCommand {
            team_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_type: "test".to_string(),
            model: Model::default_sonnet(),
            system,
            messages,
            max_tokens: 100,
            temperature: Some(0.7),
        };

        let result = handler.handle(command).await.unwrap();

        assert!(!result.content.is_empty());
        assert!(result.cost.total() > rust_decimal::Decimal::ZERO);

        // Verify usage was recorded in database
        let total_cost = usage_repo.get_total_cost(command.team_id).await.unwrap();
        assert_eq!(total_cost, result.cost.total());
    }
    ```

  - **Estimate:** 30 minutes
  - **Dependencies:** Tasks 201.21, 201.24

- [ ] **201.26** - Add LLM configuration to .env
  - **File:** `.env`
  - **Code:**

    ```bash
    # LLM Configuration
    ANTHROPIC_API_KEY=sk-ant-your-key-here
    ANTHROPIC_MODEL=claude-3-5-sonnet-20241022
    MAX_TOKENS_PER_REQUEST=4096
    LLM_DEFAULT_TEMPERATURE=0.7
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [ ] **201.27** - Create LLM service factory
  - **File:** `src/infrastructure/llm/service_factory.rs`
  - **Description:** Build LLM services from environment config
  - **Code:**

    ```rust
    use crate::application::llm::{SendLLMRequestHandler, GetUsageByTeamHandler};
    use crate::infrastructure::llm::ResilientClaudeClient;
    use crate::infrastructure::repositories::PostgresTokenUsageRepository;
    use sqlx::PgPool;

    pub struct LLMServices {
        pub send_request_handler: SendLLMRequestHandler,
        pub get_usage_handler: GetUsageByTeamHandler,
    }

    impl LLMServices {
        pub fn new(pool: PgPool) -> Result<Self, String> {
            let api_key = std::env::var("ANTHROPIC_API_KEY")
                .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;

            let client = ResilientClaudeClient::new(api_key);
            let usage_repo = Box::new(PostgresTokenUsageRepository::new(pool));

            Ok(Self {
                send_request_handler: SendLLMRequestHandler::new(
                    client.clone(),
                    usage_repo.clone(),
                ),
                get_usage_handler: GetUsageByTeamHandler::new(usage_repo),
            })
        }
    }
    ```

  - **Estimate:** 20 minutes
  - **Dependencies:** Tasks 201.21, 201.22

- [ ] **201.28** - Update main.rs to include LLM services
  - **File:** `src/main.rs`
  - **Code:**

    ```rust
    // Add to main function
    use crate::infrastructure::llm::service_factory::LLMServices;

    let llm_services = LLMServices::new(pool.clone())
        .expect("Failed to initialize LLM services");

    tracing::info!("LLM services initialized");
    ```

  - **Estimate:** 10 minutes
  - **Dependencies:** Task 201.27

- [ ] **201.29** - Write unit tests for all application layer
  - **Command:** `cargo test application::llm`
  - **Coverage Target:** ≥80%
  - **Estimate:** 45 minutes
  - **Dependencies:** Tasks 201.21-201.24

- [ ] **201.30** - Create documentation for LLM client usage
  - **File:** `docs/llm-client-guide.md`
  - **Description:** Guide for using the LLM client
  - **Content:**

    ```markdown
    # LLM Client Usage Guide

    ## Quick Start

    ```rust
    use ghostpirates_api::application::llm::{SendLLMRequestCommand, SendLLMRequestHandler};
    use ghostpirates_api::domain::llm::{Model, PromptBuilder};

    let (system, messages) = PromptBuilder::new()
        .system("You are a helpful assistant")
        .user("What is 2+2?")
        .build()
        .unwrap();

    let command = SendLLMRequestCommand {
        team_id: team.id(),
        agent_id: agent.id(),
        agent_type: "manager".to_string(),
        model: Model::default_sonnet(),
        system,
        messages,
        max_tokens: 1000,
        temperature: Some(0.7),
    };

    let result = handler.handle(command).await?;
    println!("Response: {}", result.content);
    println!("Cost: ${}", result.cost.total());
    ```

    ## Token Usage Tracking

    All LLM calls are automatically tracked in `token_usage_log` table.

    ## Error Handling

    - Rate limits (429): Automatic retry with exponential backoff
    - Network errors: Retry up to 3 times
    - API errors: Return immediately with error details

    ## Cost Optimization

    - Use prompt caching for system prompts (90% cost reduction)
    - Set appropriate max_tokens limits
    - Monitor usage with GetUsageByTeam query
    ```

  - **Estimate:** 30 minutes
  - **Dependencies:** Tasks 201.21-201.28

**Phase 5: Testing & Validation** (Tasks 201.31 - 201.38)

- [ ] **201.31** - Run all unit tests
  - **Command:** `cargo test --lib`
  - **Validation:** All tests pass
  - **Estimate:** 10 minutes
  - **Dependencies:** All previous tasks

- [ ] **201.32** - Run integration tests with real API
  - **Command:** `ANTHROPIC_API_KEY=<key> cargo test --test llm -- --ignored`
  - **Validation:** Real API calls succeed
  - **Estimate:** 15 minutes
  - **Dependencies:** Tasks 201.18, 201.25

- [ ] **201.33** - Test token tracking accuracy
  - **Test:** Make LLM call, verify usage logged correctly
  - **Validation:** Tokens match API response, cost calculated correctly
  - **Estimate:** 20 minutes
  - **Dependencies:** Task 201.25

- [ ] **201.34** - Test retry logic with simulated failures
  - **Test:** Mock network errors, verify retries
  - **Estimate:** 30 minutes
  - **Dependencies:** Task 201.14

- [ ] **201.35** - Test rate limit handling
  - **Test:** Simulate 429 response, verify backoff
  - **Estimate:** 30 minutes
  - **Dependencies:** Task 201.14

- [ ] **201.36** - Benchmark LLM call latency
  - **Test:** Measure P50, P95, P99 latencies
  - **Target:** P95 < 30 seconds
  - **Estimate:** 30 minutes
  - **Dependencies:** Task 201.21

- [ ] **201.37** - Test cost calculations
  - **Test:** Verify costs match Anthropic pricing
  - **Validation:** Input/output/cache costs accurate to $0.000001
  - **Estimate:** 20 minutes
  - **Dependencies:** Task 201.6

- [ ] **201.38** - Create smoke test script
  - **File:** `scripts/test_llm_integration.sh`
  - **Description:** End-to-end test of LLM infrastructure
  - **Code:**

    ```bash
    #!/bin/bash
    set -e

    echo "Testing LLM Infrastructure..."

    # Check environment
    if [ -z "$ANTHROPIC_API_KEY" ]; then
        echo "❌ ANTHROPIC_API_KEY not set"
        exit 1
    fi

    echo "✅ API key configured"

    # Run migrations
    cd apps/api
    sqlx migrate run

    echo "✅ Migrations applied"

    # Run tests
    cargo test --lib domain::llm
    cargo test --lib infrastructure::llm

    echo "✅ Unit tests passed"

    # Run integration tests (if API key set)
    cargo test --test llm -- --ignored

    echo "✅ Integration tests passed"

    echo ""
    echo "🎉 All LLM infrastructure tests passed!"
    ```

  - **Validation:** Script runs successfully
  - **Estimate:** 20 minutes
  - **Dependencies:** All previous tasks

---

#### 🧪 Testing Strategy (US-201)

**Testing Pyramid:**

- 60% Unit Tests (domain value objects, cost calculations)
- 30% Integration Tests (real API calls with test key)
- 10% E2E Tests (full flow with usage tracking)

**Unit Tests:**

- **Location:** `src/domain/llm/*.rs` (inline tests)
- **Coverage Target:** ≥90% (critical business logic)
- **Focus:** Model validation, token counting, cost calculations, prompt building
- **Run:** `cargo test --lib domain::llm`

**Integration Tests:**

- **Location:** `tests/llm/`
- **Focus:** Real Anthropic API calls, retry logic, error handling
- **Environment:** Requires `ANTHROPIC_API_KEY` env var
- **Run:** `cargo test --test llm -- --ignored`

**Performance Tests:**

- **Latency Target:** P95 < 30 seconds for LLM calls
- **Retry Budget:** Max 3 retries, total time < 60 seconds
- **Token Estimation:** Within 10% of actual usage

---

### US-202: Manager Agent Core

**As a** team creator
**I want** an autonomous manager agent that analyzes goals and forms specialized teams
**So that** complex goals are broken down systematically and executed by expert workers

**Business Value:** Enables autonomous project planning, reduces manual task decomposition, creates optimal team structures

**Acceptance Criteria:**

- [ ] Manager agent analyzes natural language goals and extracts core objectives
- [ ] Goal analysis identifies required specializations (3-5 worker types)
- [ ] Team formation creates worker specifications with clear responsibilities
- [ ] Task decomposition generates 5-20 concrete tasks with acceptance criteria
- [ ] Each task assigned to appropriate worker specialization
- [ ] Manager agent persisted with configuration (model, temperature, max_tokens)
- [ ] Goal analysis cached to reduce repeat costs
- [ ] All manager decisions logged for audit trail

**Technical Implementation:**

**Domain Layer:**

- **Entities:** 
  - `ManagerAgent` - Aggregate root for manager agent
  - `GoalAnalysis` - Result of goal analysis
  - `WorkerSpecification` - Blueprint for worker creation
- **Value Objects:**
  - `AgentConfiguration` - Model, temperature, max_tokens
  - `Specialization` - Worker type (e.g., "backend-developer", "database-architect")
- **Domain Events:**
  - `ManagerAgentCreated`
  - `GoalAnalyzed`
  - `TeamFormed`
  - `TasksDecomposed`

**Application Layer (CQRS):**

- **Commands:**
  - `CreateManagerAgent` - Initialize manager for team
  - `AnalyzeGoal` - Analyze goal and create plan
  - `FormTeam` - Generate worker specifications
  - `DecomposeTasks` - Break goal into tasks
- **Queries:**
  - `GetManagerAgent` - Retrieve manager by team
  - `GetGoalAnalysis` - Get cached analysis

**Infrastructure Layer:**

- **Repositories:**
  - `ManagerAgentRepository` - Persist manager agents
  - `GoalAnalysisRepository` - Cache goal analyses
- **Prompt Templates:**
  - `GOAL_ANALYSIS_PROMPT` - System prompt for goal analysis
  - `TEAM_FORMATION_PROMPT` - Generate worker specs
  - `TASK_DECOMPOSITION_PROMPT` - Create task breakdown

**Presentation Layer:**

- **API Endpoints:**
  - `POST /api/teams/:id/manager` - Create manager agent
  - `POST /api/teams/:id/analyze` - Analyze goal
  - `POST /api/teams/:id/form-team` - Generate worker team
  - `POST /api/teams/:id/decompose` - Break into tasks

**Patterns Used:**

- [x] Hexagonal Architecture (domain logic isolated)
- [x] Domain-Driven Design (ManagerAgent as aggregate)
- [x] CQRS Pattern (separate read/write operations)
- [x] Repository Pattern (persistence abstraction)
- [x] Domain Events (track manager decisions)

**File Structure:**

```
apps/api/src/
├── domain/
│   └── agents/
│       ├── manager/
│       │   ├── manager_agent.rs      # Aggregate root
│       │   ├── goal_analysis.rs      # Analysis result
│       │   ├── worker_spec.rs        # Worker blueprint
│       │   ├── value_objects.rs      # Configuration, Specialization
│       │   └── events.rs             # Domain events
│       └── repositories/
│           └── manager_agent_repository.rs
├── application/
│   └── agents/
│       └── manager/
│           ├── commands/
│           │   ├── create_manager.rs
│           │   ├── analyze_goal.rs
│           │   ├── form_team.rs
│           │   └── decompose_tasks.rs
│           └── queries/
│               ├── get_manager.rs
│               └── get_analysis.rs
├── infrastructure/
│   ├── repositories/
│   │   └── postgres_manager_agent_repository.rs
│   └── prompts/
│       ├── goal_analysis_prompt.rs
│       ├── team_formation_prompt.rs
│       └── task_decomposition_prompt.rs
└── api/
    └── handlers/
        └── manager_agent_handlers.rs
```

**Estimation:** 20 hours (48 tasks)

---

#### 📋 Sub-Tasks Breakdown (US-202)

**Phase 1: Database Schema** (Tasks 202.1 - 202.4)

- [ ] **202.1** - Create manager_agents table migration
  - **File:** `migrations/20251110000001_create_manager_agents.sql`
  - **Code:**

    ```sql
    CREATE TABLE manager_agents (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE UNIQUE,
        
        -- Configuration
        model VARCHAR(100) NOT NULL,
        temperature DECIMAL(3, 2) NOT NULL DEFAULT 0.7,
        max_tokens INTEGER NOT NULL DEFAULT 4096,
        
        -- State
        status VARCHAR(50) NOT NULL DEFAULT 'active',
        
        -- Timestamps
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        
        CONSTRAINT valid_temperature CHECK (temperature >= 0 AND temperature <= 1),
        CONSTRAINT positive_max_tokens CHECK (max_tokens > 0)
    );

    CREATE INDEX idx_manager_agents_team ON manager_agents(team_id);
    CREATE INDEX idx_manager_agents_status ON manager_agents(status);
    ```

  - **Estimate:** 20 minutes

- [ ] **202.2** - Create goal_analyses table migration
  - **File:** `migrations/20251110000002_create_goal_analyses.sql`
  - **Code:**

    ```sql
    CREATE TABLE goal_analyses (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
        manager_agent_id UUID NOT NULL REFERENCES manager_agents(id) ON DELETE CASCADE,
        
        -- Analysis results
        core_objective TEXT NOT NULL,
        subtasks JSONB NOT NULL,
        required_specializations JSONB NOT NULL,
        estimated_timeline_hours DECIMAL(10, 2),
        potential_blockers JSONB NOT NULL,
        success_criteria JSONB NOT NULL,
        
        -- Metadata
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        
        CONSTRAINT positive_timeline CHECK (estimated_timeline_hours IS NULL OR estimated_timeline_hours > 0)
    );

    CREATE INDEX idx_goal_analyses_team ON goal_analyses(team_id);
    CREATE INDEX idx_goal_analyses_manager ON goal_analyses(manager_agent_id);
    CREATE INDEX idx_goal_analyses_created ON goal_analyses(created_at DESC);
    ```

  - **Estimate:** 25 minutes

- [ ] **202.3** - Create worker_specifications table migration
  - **File:** `migrations/20251110000003_create_worker_specifications.sql`
  - **Code:**

    ```sql
    CREATE TABLE worker_specifications (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
        manager_agent_id UUID NOT NULL REFERENCES manager_agents(id) ON DELETE CASCADE,
        
        -- Worker details
        specialization VARCHAR(100) NOT NULL,
        skills JSONB NOT NULL,
        responsibilities JSONB NOT NULL,
        required_tools JSONB NOT NULL,
        
        -- Worker instance (created when worker spawned)
        worker_agent_id UUID,
        
        -- Timestamps
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX idx_worker_specs_team ON worker_specifications(team_id);
    CREATE INDEX idx_worker_specs_manager ON worker_specifications(manager_agent_id);
    CREATE INDEX idx_worker_specs_specialization ON worker_specifications(specialization);
    ```

  - **Estimate:** 25 minutes

- [ ] **202.4** - Apply migrations
  - **Command:** `cd apps/api && sqlx migrate run`
  - **Validation:** All 3 migrations applied successfully
  - **Estimate:** 5 minutes

**Phase 2: Domain Layer** (Tasks 202.5 - 202.15)

- [ ] **202.5** - Create AgentConfiguration value object
  - **File:** `src/domain/agents/manager/value_objects.rs`
  - **Code:**

    ```rust
    use serde::{Deserialize, Serialize};
    use crate::domain::llm::Model;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AgentConfiguration {
        model: Model,
        temperature: f32,
        max_tokens: u32,
    }

    impl AgentConfiguration {
        pub fn new(model: Model, temperature: f32, max_tokens: u32) -> Result<Self, String> {
            if temperature < 0.0 || temperature > 1.0 {
                return Err("Temperature must be between 0 and 1".to_string());
            }

            if max_tokens == 0 {
                return Err("Max tokens must be positive".to_string());
            }

            Ok(Self {
                model,
                temperature,
                max_tokens,
            })
        }

        pub fn default() -> Self {
            Self {
                model: Model::default_sonnet(),
                temperature: 0.7,
                max_tokens: 4096,
            }
        }

        pub fn model(&self) -> &Model {
            &self.model
        }

        pub fn temperature(&self) -> f32 {
            self.temperature
        }

        pub fn max_tokens(&self) -> u32 {
            self.max_tokens
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Specialization(String);

    impl Specialization {
        pub fn new(spec: impl Into<String>) -> Result<Self, String> {
            let spec = spec.into();
            
            if spec.is_empty() {
                return Err("Specialization cannot be empty".to_string());
            }

            // Validate format: lowercase-with-dashes
            if !spec.chars().all(|c| c.is_lowercase() || c == '-') {
                return Err("Specialization must be lowercase with dashes".to_string());
            }

            Ok(Specialization(spec))
        }

        pub fn as_str(&self) -> &str {
            &self.0
        }
    }
    ```

  - **Estimate:** 30 minutes

- [ ] **202.6** - Create GoalAnalysis entity
  - **File:** `src/domain/agents/manager/goal_analysis.rs`
  - **Code:**

    ```rust
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct GoalAnalysis {
        id: Uuid,
        team_id: Uuid,
        manager_agent_id: Uuid,
        core_objective: String,
        subtasks: Vec<String>,
        required_specializations: Vec<String>,
        estimated_timeline_hours: Option<f32>,
        potential_blockers: Vec<String>,
        success_criteria: Vec<String>,
        created_at: DateTime<Utc>,
    }

    impl GoalAnalysis {
        pub fn new(
            team_id: Uuid,
            manager_agent_id: Uuid,
            core_objective: String,
            subtasks: Vec<String>,
            required_specializations: Vec<String>,
            estimated_timeline_hours: Option<f32>,
            potential_blockers: Vec<String>,
            success_criteria: Vec<String>,
        ) -> Result<Self, String> {
            if core_objective.is_empty() {
                return Err("Core objective cannot be empty".to_string());
            }

            if subtasks.is_empty() {
                return Err("At least one subtask required".to_string());
            }

            if required_specializations.is_empty() {
                return Err("At least one specialization required".to_string());
            }

            if success_criteria.is_empty() {
                return Err("At least one success criterion required".to_string());
            }

            Ok(Self {
                id: Uuid::new_v4(),
                team_id,
                manager_agent_id,
                core_objective,
                subtasks,
                required_specializations,
                estimated_timeline_hours,
                potential_blockers,
                success_criteria,
                created_at: Utc::now(),
            })
        }

        // Getters
        pub fn id(&self) -> Uuid { self.id }
        pub fn team_id(&self) -> Uuid { self.team_id }
        pub fn core_objective(&self) -> &str { &self.core_objective }
        pub fn subtasks(&self) -> &[String] { &self.subtasks }
        pub fn required_specializations(&self) -> &[String] { &self.required_specializations }
    }
    ```

  - **Estimate:** 40 minutes

- [ ] **202.7** - Create WorkerSpecification entity
  - **File:** `src/domain/agents/manager/worker_spec.rs`
  - **Code:**

    ```rust
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;
    use super::value_objects::Specialization;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkerSpecification {
        id: Uuid,
        team_id: Uuid,
        manager_agent_id: Uuid,
        specialization: Specialization,
        skills: Vec<String>,
        responsibilities: Vec<String>,
        required_tools: Vec<String>,
        worker_agent_id: Option<Uuid>,
        created_at: DateTime<Utc>,
    }

    impl WorkerSpecification {
        pub fn new(
            team_id: Uuid,
            manager_agent_id: Uuid,
            specialization: Specialization,
            skills: Vec<String>,
            responsibilities: Vec<String>,
            required_tools: Vec<String>,
        ) -> Result<Self, String> {
            if skills.is_empty() {
                return Err("At least one skill required".to_string());
            }

            if responsibilities.is_empty() {
                return Err("At least one responsibility required".to_string());
            }

            Ok(Self {
                id: Uuid::new_v4(),
                team_id,
                manager_agent_id,
                specialization,
                skills,
                responsibilities,
                required_tools,
                worker_agent_id: None,
                created_at: Utc::now(),
            })
        }

        pub fn assign_worker(&mut self, worker_id: Uuid) {
            self.worker_agent_id = Some(worker_id);
        }

        // Getters
        pub fn id(&self) -> Uuid { self.id }
        pub fn specialization(&self) -> &Specialization { &self.specialization }
        pub fn skills(&self) -> &[String] { &self.skills }
        pub fn responsibilities(&self) -> &[String] { &self.responsibilities }
    }
    ```

  - **Estimate:** 40 minutes

[Continuing in next message due to length...]

- [ ] **202.8** - Create ManagerAgent aggregate
  - **File:** `src/domain/agents/manager/manager_agent.rs`
  - **Description:** Aggregate root managing all manager agent operations
  - **Code:**

    ```rust
    use chrono::{DateTime, Utc};
    use uuid::Uuid;
    use super::{GoalAnalysis, WorkerSpecification};
    use super::value_objects::AgentConfiguration;
    use super::events::ManagerAgentEvent;

    #[derive(Debug, Clone)]
    pub struct ManagerAgent {
        id: Uuid,
        team_id: Uuid,
        config: AgentConfiguration,
        status: AgentStatus,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AgentStatus {
        Active,
        Inactive,
    }

    impl ManagerAgent {
        pub fn new(team_id: Uuid, config: AgentConfiguration) -> (Self, Vec<ManagerAgentEvent>) {
            let id = Uuid::new_v4();
            let now = Utc::now();

            let agent = Self {
                id,
                team_id,
                config,
                status: AgentStatus::Active,
                created_at: now,
                updated_at: now,
            };

            let events = vec![ManagerAgentEvent::Created {
                agent_id: id,
                team_id,
            }];

            (agent, events)
        }

        pub fn can_analyze_goal(&self) -> bool {
            self.status == AgentStatus::Active
        }

        pub fn record_goal_analysis(&mut self, _analysis: &GoalAnalysis) -> ManagerAgentEvent {
            self.updated_at = Utc::now();
            
            ManagerAgentEvent::GoalAnalyzed {
                agent_id: self.id,
                team_id: self.team_id,
            }
        }

        pub fn record_team_formation(&mut self, _workers: &[WorkerSpecification]) -> ManagerAgentEvent {
            self.updated_at = Utc::now();
            
            ManagerAgentEvent::TeamFormed {
                agent_id: self.id,
                team_id: self.team_id,
                worker_count: _workers.len(),
            }
        }

        // Getters
        pub fn id(&self) -> Uuid { self.id }
        pub fn team_id(&self) -> Uuid { self.team_id }
        pub fn config(&self) -> &AgentConfiguration { &self.config }
        pub fn status(&self) -> AgentStatus { self.status }
    }
    ```

  - **Estimate:** 60 minutes

- [ ] **202.9** - Create manager agent domain events
  - **File:** `src/domain/agents/manager/events.rs`
  - **Code:**

    ```rust
    use uuid::Uuid;

    #[derive(Debug, Clone)]
    pub enum ManagerAgentEvent {
        Created {
            agent_id: Uuid,
            team_id: Uuid,
        },
        GoalAnalyzed {
            agent_id: Uuid,
            team_id: Uuid,
        },
        TeamFormed {
            agent_id: Uuid,
            team_id: Uuid,
            worker_count: usize,
        },
        TasksDecomposed {
            agent_id: Uuid,
            team_id: Uuid,
            task_count: usize,
        },
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **202.10** - Create domain module structure
  - **Commands:**

    ```bash
    mkdir -p src/domain/agents/manager
    touch src/domain/agents/manager/{mod.rs,manager_agent.rs,goal_analysis.rs,worker_spec.rs,value_objects.rs,events.rs}
    ```

  - **Estimate:** 5 minutes

- [ ] **202.11** - Add domain exports
  - **File:** `src/domain/agents/manager/mod.rs`
  - **Code:**

    ```rust
    mod manager_agent;
    mod goal_analysis;
    mod worker_spec;
    mod value_objects;
    mod events;

    pub use manager_agent::{ManagerAgent, AgentStatus};
    pub use goal_analysis::GoalAnalysis;
    pub use worker_spec::WorkerSpecification;
    pub use value_objects::{AgentConfiguration, Specialization};
    pub use events::ManagerAgentEvent;
    ```

  - **File:** `src/domain/agents/mod.rs`
  - **Code:**

    ```rust
    pub mod manager;
    ```

  - **File:** `src/domain/mod.rs` (add line)
  - **Code:**

    ```rust
    pub mod agents;  // Add this
    ```

  - **Estimate:** 10 minutes

- [ ] **202.12** - Create ManagerAgentRepository trait
  - **File:** `src/domain/repositories/manager_agent_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::domain::agents::manager::ManagerAgent;

    #[async_trait]
    pub trait ManagerAgentRepository: Send + Sync {
        async fn save(&self, agent: &ManagerAgent) -> Result<(), String>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<ManagerAgent>, String>;
        async fn find_by_team(&self, team_id: Uuid) -> Result<Option<ManagerAgent>, String>;
        async fn delete(&self, id: Uuid) -> Result<(), String>;
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **202.13** - Create GoalAnalysisRepository trait
  - **File:** `src/domain/repositories/goal_analysis_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::domain::agents::manager::GoalAnalysis;

    #[async_trait]
    pub trait GoalAnalysisRepository: Send + Sync {
        async fn save(&self, analysis: &GoalAnalysis) -> Result<(), String>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<GoalAnalysis>, String>;
        async fn find_by_team(&self, team_id: Uuid) -> Result<Vec<GoalAnalysis>, String>;
        async fn latest_for_team(&self, team_id: Uuid) -> Result<Option<GoalAnalysis>, String>;
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **202.14** - Create WorkerSpecificationRepository trait
  - **File:** `src/domain/repositories/worker_spec_repository.rs`
  - **Code:**

    ```rust
    use async_trait::async_trait;
    use uuid::Uuid;
    use crate::domain::agents::manager::WorkerSpecification;

    #[async_trait]
    pub trait WorkerSpecificationRepository: Send + Sync {
        async fn save(&self, spec: &WorkerSpecification) -> Result<(), String>;
        async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkerSpecification>, String>;
        async fn find_by_team(&self, team_id: Uuid) -> Result<Vec<WorkerSpecification>, String>;
        async fn find_by_manager(&self, manager_id: Uuid) -> Result<Vec<WorkerSpecification>, String>;
    }
    ```

  - **Estimate:** 15 minutes

- [ ] **202.15** - Write unit tests for domain layer
  - **Command:** `cargo test domain::agents::manager`
  - **Coverage Target:** ≥80%
  - **Estimate:** 45 minutes

**Phase 3: Infrastructure - Repositories** (Tasks 202.16 - 202.20)

- [ ] **202.16** - Implement PostgresManagerAgentRepository
  - **File:** `src/infrastructure/repositories/postgres_manager_agent_repository.rs`
  - **Estimate:** 60 minutes

- [ ] **202.17** - Implement PostgresGoalAnalysisRepository
  - **File:** `src/infrastructure/repositories/postgres_goal_analysis_repository.rs`
  - **Estimate:** 60 minutes

- [ ] **202.18** - Implement PostgresWorkerSpecificationRepository
  - **File:** `src/infrastructure/repositories/postgres_worker_spec_repository.rs`
  - **Estimate:** 60 minutes

- [ ] **202.19** - Add repository exports
  - **File:** `src/infrastructure/repositories/mod.rs`
  - **Estimate:** 5 minutes

- [ ] **202.20** - Write integration tests for repositories
  - **File:** `tests/repositories/manager_agent_repository_test.rs`
  - **Estimate:** 45 minutes

**Phase 4: Infrastructure - Prompt Templates** (Tasks 202.21 - 202.24)

- [ ] **202.21** - Create goal analysis prompt template
  - **File:** `src/infrastructure/prompts/goal_analysis_prompt.rs`
  - **Code:**

    ```rust
    pub const GOAL_ANALYSIS_SYSTEM_PROMPT: &str = r#"You are an expert project manager and business analyst with deep expertise in breaking down complex goals into actionable plans.

Your task is to analyze project goals and provide structured analysis in JSON format.

Analyze the goal considering:
1. Core objective - what is the fundamental outcome desired?
2. Key subtasks - what are the major steps to achieve this?
3. Required specializations - what types of experts are needed?
4. Timeline estimation - realistic hours needed
5. Potential blockers - what could prevent success?
6. Success criteria - how will we know it's done?

Return JSON in this exact format:
{
  "core_objective": "One clear sentence stating the main goal",
  "subtasks": ["Task 1", "Task 2", "Task 3"],
  "required_specializations": ["specialization-1", "specialization-2"],
  "estimated_timeline_hours": 24.5,
  "potential_blockers": ["Blocker 1", "Blocker 2"],
  "success_criteria": ["Criterion 1", "Criterion 2"]
}

Be specific, measurable, and realistic in all assessments."#;

    pub fn build_goal_analysis_prompt(goal: &str) -> String {
        format!(
            "Analyze this project goal:\n\n\"{}\"\n\nProvide complete JSON analysis.",
            goal
        )
    }
    ```

  - **Estimate:** 30 minutes

- [ ] **202.22** - Create team formation prompt template
  - **File:** `src/infrastructure/prompts/team_formation_prompt.rs`
  - **Code:**

    ```rust
    pub const TEAM_FORMATION_SYSTEM_PROMPT: &str = r#"You are an expert in team composition and role design for software projects.

Your task is to design 3-5 specialized worker roles that will collaborate to achieve a goal.

For each worker, define:
- Specialization name (lowercase-with-dashes format)
- Key skills required
- Primary responsibilities
- Required tools/technologies

Return JSON array in this format:
[
  {
    "specialization": "backend-developer",
    "skills": ["Rust", "PostgreSQL", "API design"],
    "responsibilities": ["Build REST API", "Design database schema"],
    "required_tools": ["Cargo", "SQLx", "Postman"]
  }
]

Requirements:
- 3-5 workers (no more, no less)
- Non-overlapping responsibilities
- Balanced workload distribution
- Realistic skill combinations"#;

    pub fn build_team_formation_prompt(analysis: &crate::domain::agents::manager::GoalAnalysis) -> String {
        format!(
            "Goal: {}\n\nSubtasks:\n{}\n\nRequired specializations:\n{}\n\nDesign 3-5 specialized worker roles.",
            analysis.core_objective(),
            analysis.subtasks().iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n"),
            analysis.required_specializations().iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        )
    }
    ```

  - **Estimate:** 30 minutes

- [ ] **202.23** - Create task decomposition prompt template
  - **File:** `src/infrastructure/prompts/task_decomposition_prompt.rs`
  - **Code:**

    ```rust
    pub const TASK_DECOMPOSITION_SYSTEM_PROMPT: &str = r#"You are an expert at breaking down complex goals into concrete, actionable tasks.

Your task is to decompose a goal into 5-20 specific tasks.

For each task provide:
- Title (concise, action-oriented)
- Detailed description
- Acceptance criteria (3-5 checkable items)
- Required skills
- Estimated complexity (low/medium/high)

Return JSON array:
[
  {
    "title": "Design database schema",
    "description": "Create PostgreSQL schema for user management with proper indexes",
    "acceptance_criteria": [
      "Tables created with foreign keys",
      "Indexes on commonly queried columns",
      "Migration script tested"
    ],
    "required_skills": ["PostgreSQL", "database-design"],
    "complexity": "medium"
  }
]

Requirements:
- 5-20 tasks total
- Each task completable in < 8 hours
- Clear, measurable acceptance criteria
- Logical ordering (dependencies considered)"#;

    pub fn build_task_decomposition_prompt(goal: &str, analysis: &crate::domain::agents::manager::GoalAnalysis) -> String {
        format!(
            "Goal: {}\n\nCore objective: {}\n\nSubtasks:\n{}\n\nDecompose into 5-20 concrete tasks.",
            goal,
            analysis.core_objective(),
            analysis.subtasks().iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
        )
    }
    ```

  - **Estimate:** 30 minutes

- [ ] **202.24** - Create prompts module
  - **File:** `src/infrastructure/prompts/mod.rs`
  - **Code:**

    ```rust
    pub mod goal_analysis_prompt;
    pub mod team_formation_prompt;
    pub mod task_decomposition_prompt;

    pub use goal_analysis_prompt::*;
    pub use team_formation_prompt::*;
    pub use task_decomposition_prompt::*;
    ```

  - **Estimate:** 5 minutes

**Phase 5: Application Layer - Commands** (Tasks 202.25 - 202.35)

- [ ] **202.25** - Create CreateManagerAgent command
  - **File:** `src/application/agents/manager/commands/create_manager.rs`
  - **Estimate:** 45 minutes

- [ ] **202.26** - Create AnalyzeGoal command
  - **File:** `src/application/agents/manager/commands/analyze_goal.rs`
  - **Description:** Call LLM to analyze goal and cache result
  - **Estimate:** 60 minutes

- [ ] **202.27** - Create FormTeam command
  - **File:** `src/application/agents/manager/commands/form_team.rs`
  - **Description:** Generate worker specifications based on analysis
  - **Estimate:** 60 minutes

- [ ] **202.28** - Create DecomposeTasks command
  - **File:** `src/application/agents/manager/commands/decompose_tasks.rs`
  - **Description:** Break goal into concrete tasks
  - **Estimate:** 60 minutes

- [ ] **202.29** - Create GetManagerAgent query
  - **File:** `src/application/agents/manager/queries/get_manager.rs`
  - **Estimate:** 30 minutes

- [ ] **202.30** - Create GetGoalAnalysis query
  - **File:** `src/application/agents/manager/queries/get_analysis.rs`
  - **Estimate:** 30 minutes

- [ ] **202.31** - Create application module structure
  - **Commands:**

    ```bash
    mkdir -p src/application/agents/manager/{commands,queries}
    ```

  - **Estimate:** 5 minutes

- [ ] **202.32** - Add application exports
  - **File:** `src/application/agents/manager/mod.rs`
  - **Estimate:** 10 minutes

- [ ] **202.33** - Create manager agent service factory
  - **File:** `src/infrastructure/agents/manager_service_factory.rs`
  - **Estimate:** 30 minutes

- [ ] **202.34** - Write integration tests for commands
  - **File:** `tests/application/manager_agent_test.rs`
  - **Estimate:** 60 minutes

- [ ] **202.35** - Test goal analysis with real API
  - **Test:** Submit real goal, verify analysis quality
  - **Estimate:** 30 minutes

**Phase 6: API Layer** (Tasks 202.36 - 202.45)

- [ ] **202.36** - Create DTO types for requests/responses
  - **File:** `src/api/dto/manager_agent_dto.rs`
  - **Estimate:** 30 minutes

- [ ] **202.37** - Implement POST /api/teams/:id/manager
  - **File:** `src/api/handlers/manager_agent_handlers.rs`
  - **Estimate:** 45 minutes

- [ ] **202.38** - Implement POST /api/teams/:id/analyze
  - **Estimate:** 45 minutes

- [ ] **202.39** - Implement POST /api/teams/:id/form-team
  - **Estimate:** 45 minutes

- [ ] **202.40** - Implement POST /api/teams/:id/decompose
  - **Estimate:** 45 minutes

- [ ] **202.41** - Implement GET /api/teams/:id/manager
  - **Estimate:** 30 minutes

- [ ] **202.42** - Add routes to main router
  - **File:** `src/main.rs`
  - **Estimate:** 15 minutes

- [ ] **202.43** - Add error handling for manager operations
  - **Estimate:** 20 minutes

- [ ] **202.44** - Write API integration tests
  - **File:** `tests/api/manager_agent_api_test.rs`
  - **Estimate:** 60 minutes

- [ ] **202.45** - Test full manager workflow end-to-end
  - **Test:** Create manager → Analyze goal → Form team → Decompose tasks
  - **Estimate:** 45 minutes

**Phase 7: Testing & Documentation** (Tasks 202.46 - 202.48)

- [ ] **202.46** - Run all tests
  - **Command:** `cargo test`
  - **Validation:** All tests pass
  - **Estimate:** 15 minutes

- [ ] **202.47** - Create manager agent usage documentation
  - **File:** `docs/manager-agent-guide.md`
  - **Estimate:** 45 minutes

- [ ] **202.48** - Create smoke test script
  - **File:** `scripts/test_manager_agent.sh`
  - **Estimate:** 30 minutes

---

#### 🧪 Testing Strategy (US-202)

**Unit Tests:**
- Domain entities (ManagerAgent, GoalAnalysis, WorkerSpecification)
- Value objects (AgentConfiguration, Specialization)
- Business rule validation

**Integration Tests:**
- Repository CRUD operations
- LLM prompt generation
- Real API calls for goal analysis

**E2E Tests:**
- Complete manager workflow
- Error handling scenarios
- Cost tracking verification

---

### US-203: Worker Agent System

**As a** manager agent
**I want** to create and manage specialized worker agents
**So that** tasks can be executed by agents with the right skills

**Business Value:** Enables parallel task execution, specialist expertise, scalable team composition

**Acceptance Criteria:**

- [ ] Worker agents created dynamically based on specifications
- [ ] Each worker has specialization, skills, and tools configured
- [ ] Workers can be assigned tasks matching their specialization
- [ ] Workers execute tasks using LLM with specialist prompts
- [ ] Worker output tracked with task completion records
- [ ] Workers persist state between task executions
- [ ] Maximum 5 workers per team enforced

**Patterns Used:**

- [x] Hexagonal Architecture
- [x] Domain-Driven Design (WorkerAgent as aggregate)
- [x] Factory Pattern (dynamic worker creation)
- [x] Strategy Pattern (specialization-specific behavior)

**Estimation:** 18 hours (42 tasks)

---

#### 📋 Sub-Tasks Breakdown (US-203)

**Phase 1: Database Schema** (Tasks 203.1 - 203.2)

- [ ] **203.1** - Create worker_agents table migration
  - **File:** `migrations/20251111000001_create_worker_agents.sql`
  - **Estimate:** 25 minutes

- [ ] **203.2** - Apply migration
  - **Command:** `sqlx migrate run`
  - **Estimate:** 5 minutes

**Phase 2: Domain Layer** (Tasks 203.3 - 203.10)

- [ ] **203.3** - Create WorkerAgent aggregate
  - **File:** `src/domain/agents/worker/worker_agent.rs`
  - **Estimate:** 60 minutes

- [ ] **203.4** - Create worker domain events
  - **File:** `src/domain/agents/worker/events.rs`
  - **Estimate:** 20 minutes

- [ ] **203.5** - Create WorkerAgentRepository trait
  - **File:** `src/domain/repositories/worker_agent_repository.rs`
  - **Estimate:** 20 minutes

- [ ] **203.6** - Create worker domain module
  - **Files:** `src/domain/agents/worker/mod.rs`
  - **Estimate:** 10 minutes

- [ ] **203.7** - Add worker exports to agents module
  - **Estimate:** 5 minutes

- [ ] **203.8** - Write unit tests for WorkerAgent
  - **Estimate:** 45 minutes

- [ ] **203.9** - Test worker creation from specification
  - **Estimate:** 30 minutes

- [ ] **203.10** - Test worker state transitions
  - **Estimate:** 30 minutes

**Phase 3: Infrastructure** (Tasks 203.11 - 203.18)

- [ ] **203.11** - Implement PostgresWorkerAgentRepository
  - **File:** `src/infrastructure/repositories/postgres_worker_agent_repository.rs`
  - **Estimate:** 60 minutes

- [ ] **203.12** - Create worker-specific prompt templates
  - **File:** `src/infrastructure/prompts/worker_prompts.rs`
  - **Estimate:** 45 minutes

- [ ] **203.13** - Create WorkerFactory
  - **File:** `src/infrastructure/agents/worker_factory.rs`
  - **Description:** Create workers from specifications
  - **Estimate:** 45 minutes

- [ ] **203.14** - Test repository operations
  - **Estimate:** 30 minutes

- [ ] **203.15** - Test worker factory
  - **Estimate:** 30 minutes

- [ ] **203.16** - Test specialization-specific prompts
  - **Estimate:** 30 minutes

- [ ] **203.17** - Create worker prompt examples
  - **File:** `docs/worker-prompt-examples.md`
  - **Estimate:** 30 minutes

- [ ] **203.18** - Validate prompt quality
  - **Test:** Generate prompts for each specialization
  - **Estimate:** 30 minutes

**Phase 4: Application Layer** (Tasks 203.19 - 203.30)

- [ ] **203.19** - Create CreateWorkerAgent command
  - **File:** `src/application/agents/worker/commands/create_worker.rs`
  - **Estimate:** 45 minutes

- [ ] **203.20** - Create AssignTask command
  - **File:** `src/application/agents/worker/commands/assign_task.rs`
  - **Estimate:** 45 minutes

- [ ] **203.21** - Create ExecuteTask command
  - **File:** `src/application/agents/worker/commands/execute_task.rs`
  - **Description:** Worker executes assigned task using LLM
  - **Estimate:** 60 minutes

- [ ] **203.22** - Create GetWorkersByTeam query
  - **File:** `src/application/agents/worker/queries/get_workers.rs`
  - **Estimate:** 30 minutes

- [ ] **203.23** - Create worker application module
  - **Files:** Module structure
  - **Estimate:** 10 minutes

- [ ] **203.24** - Add application exports
  - **Estimate:** 10 minutes

- [ ] **203.25** - Write integration tests for commands
  - **Estimate:** 60 minutes

- [ ] **203.26** - Test worker creation flow
  - **Estimate:** 30 minutes

- [ ] **203.27** - Test task assignment
  - **Estimate:** 30 minutes

- [ ] **203.28** - Test task execution with real LLM
  - **Estimate:** 45 minutes

- [ ] **203.29** - Test worker team limit (max 5)
  - **Estimate:** 20 minutes

- [ ] **203.30** - Test worker specialization matching
  - **Estimate:** 30 minutes

**Phase 5: API Layer** (Tasks 203.31 - 203.38)

- [ ] **203.31** - Create worker DTOs
  - **File:** `src/api/dto/worker_agent_dto.rs`
  - **Estimate:** 30 minutes

- [ ] **203.32** - Implement POST /api/teams/:id/workers
  - **File:** `src/api/handlers/worker_agent_handlers.rs`
  - **Estimate:** 45 minutes

- [ ] **203.33** - Implement GET /api/teams/:id/workers
  - **Estimate:** 30 minutes

- [ ] **203.34** - Implement POST /api/workers/:id/execute
  - **Estimate:** 45 minutes

- [ ] **203.35** - Add routes to main router
  - **Estimate:** 15 minutes

- [ ] **203.36** - Write API tests
  - **File:** `tests/api/worker_agent_api_test.rs`
  - **Estimate:** 60 minutes

- [ ] **203.37** - Test full worker lifecycle
  - **Test:** Create → Assign → Execute → Complete
  - **Estimate:** 45 minutes

- [ ] **203.38** - Test error scenarios
  - **Estimate:** 30 minutes

**Phase 6: Documentation** (Tasks 203.39 - 203.42)

- [ ] **203.39** - Create worker agent guide
  - **File:** `docs/worker-agent-guide.md`
  - **Estimate:** 45 minutes

- [ ] **203.40** - Document specializations
  - **File:** `docs/worker-specializations.md`
  - **Estimate:** 30 minutes

- [ ] **203.41** - Create smoke test script
  - **File:** `scripts/test_worker_agents.sh`
  - **Estimate:** 30 minutes

- [ ] **203.42** - Run all tests
  - **Command:** `cargo test`
  - **Estimate:** 15 minutes

---

### US-204: Review and Revision Loops

**As a** manager agent
**I want** to review worker output and request revisions
**So that** task quality meets acceptance criteria

**Business Value:** Ensures quality control, reduces errors, improves output accuracy

**Acceptance Criteria:**

- [ ] Manager reviews worker output against acceptance criteria
- [ ] Review decision: Approve, Request Revision, or Reject
- [ ] Revision requests include specific feedback
- [ ] Workers can revise output based on feedback
- [ ] Maximum 3 revision rounds per task
- [ ] Review history tracked in database
- [ ] Approval automatically updates task status to complete

**Patterns Used:**

- [x] State Machine Pattern (task review states)
- [x] Chain of Responsibility (review pipeline)
- [x] Event Sourcing (track review history)

**Estimation:** 18 hours (42 tasks)

---

#### 📋 Sub-Tasks Breakdown (US-204)

**Phase 1: Database Schema** (Tasks 204.1 - 204.2)

- [ ] **204.1** - Create task_reviews table migration
  - **File:** `migrations/20251112000001_create_task_reviews.sql`
  - **Estimate:** 30 minutes

- [ ] **204.2** - Apply migration
  - **Estimate:** 5 minutes

**Phase 2: Domain Layer** (Tasks 204.3 - 204.12)

- [ ] **204.3** - Create ReviewDecision value object
  - **File:** `src/domain/agents/review/review_decision.rs`
  - **Estimate:** 30 minutes

- [ ] **204.4** - Create TaskReview aggregate
  - **File:** `src/domain/agents/review/task_review.rs`
  - **Estimate:** 60 minutes

- [ ] **204.5** - Create review domain events
  - **File:** `src/domain/agents/review/events.rs`
  - **Estimate:** 20 minutes

- [ ] **204.6** - Create TaskReviewRepository trait
  - **File:** `src/domain/repositories/task_review_repository.rs`
  - **Estimate:** 20 minutes

- [ ] **204.7** - Create review domain module
  - **Estimate:** 10 minutes

- [ ] **204.8** - Add review exports
  - **Estimate:** 5 minutes

- [ ] **204.9** - Write unit tests for ReviewDecision
  - **Estimate:** 30 minutes

- [ ] **204.10** - Write unit tests for TaskReview
  - **Estimate:** 45 minutes

- [ ] **204.11** - Test revision limit enforcement (max 3)
  - **Estimate:** 30 minutes

- [ ] **204.12** - Test review state transitions
  - **Estimate:** 30 minutes

**Phase 3: Infrastructure** (Tasks 204.13 - 204.20)

- [ ] **204.13** - Implement PostgresTaskReviewRepository
  - **File:** `src/infrastructure/repositories/postgres_task_review_repository.rs`
  - **Estimate:** 60 minutes

- [ ] **204.14** - Create review prompt templates
  - **File:** `src/infrastructure/prompts/review_prompts.rs`
  - **Estimate:** 45 minutes

- [ ] **204.15** - Create revision prompt templates
  - **File:** `src/infrastructure/prompts/revision_prompts.rs`
  - **Estimate:** 45 minutes

- [ ] **204.16** - Test repository operations
  - **Estimate:** 30 minutes

- [ ] **204.17** - Test review prompt generation
  - **Estimate:** 30 minutes

- [ ] **204.18** - Test revision prompt generation
  - **Estimate:** 30 minutes

- [ ] **204.19** - Create prompt quality validation
  - **Estimate:** 30 minutes

- [ ] **204.20** - Document review prompts
  - **File:** `docs/review-prompt-guide.md`
  - **Estimate:** 30 minutes

**Phase 4: Application Layer** (Tasks 204.21 - 204.30)

- [ ] **204.21** - Create ReviewTaskOutput command
  - **File:** `src/application/agents/review/commands/review_task.rs`
  - **Description:** Manager reviews worker output
  - **Estimate:** 60 minutes

- [ ] **204.22** - Create RequestRevision command
  - **File:** `src/application/agents/review/commands/request_revision.rs`
  - **Estimate:** 45 minutes

- [ ] **204.23** - Create ApproveTask command
  - **File:** `src/application/agents/review/commands/approve_task.rs`
  - **Estimate:** 45 minutes

- [ ] **204.24** - Create RejectTask command
  - **File:** `src/application/agents/review/commands/reject_task.rs`
  - **Estimate:** 45 minutes

- [ ] **204.25** - Create GetReviewHistory query
  - **File:** `src/application/agents/review/queries/get_review_history.rs`
  - **Estimate:** 30 minutes

- [ ] **204.26** - Create review application module
  - **Estimate:** 10 minutes

- [ ] **204.27** - Write integration tests
  - **Estimate:** 60 minutes

- [ ] **204.28** - Test review with real LLM
  - **Estimate:** 45 minutes

- [ ] **204.29** - Test revision loop
  - **Estimate:** 45 minutes

- [ ] **204.30** - Test revision limit
  - **Estimate:** 30 minutes

**Phase 5: API Layer** (Tasks 204.31 - 204.38)

- [ ] **204.31** - Create review DTOs
  - **File:** `src/api/dto/task_review_dto.rs`
  - **Estimate:** 30 minutes

- [ ] **204.32** - Implement POST /api/tasks/:id/review
  - **File:** `src/api/handlers/task_review_handlers.rs`
  - **Estimate:** 45 minutes

- [ ] **204.33** - Implement POST /api/tasks/:id/approve
  - **Estimate:** 30 minutes

- [ ] **204.34** - Implement POST /api/tasks/:id/reject
  - **Estimate:** 30 minutes

- [ ] **204.35** - Implement GET /api/tasks/:id/reviews
  - **Estimate:** 30 minutes

- [ ] **204.36** - Add routes
  - **Estimate:** 15 minutes

- [ ] **204.37** - Write API tests
  - **Estimate:** 60 minutes

- [ ] **204.38** - Test full review cycle
  - **Test:** Submit → Review → Revise → Approve
  - **Estimate:** 45 minutes

**Phase 6: Documentation & Testing** (Tasks 204.39 - 204.42)

- [ ] **204.39** - Create review system guide
  - **File:** `docs/review-system-guide.md`
  - **Estimate:** 45 minutes

- [ ] **204.40** - Create smoke test script
  - **File:** `scripts/test_review_system.sh`
  - **Estimate:** 30 minutes

- [ ] **204.41** - Run all tests
  - **Estimate:** 15 minutes

- [ ] **204.42** - Validate review quality
  - **Test:** Submit various outputs, verify review accuracy
  - **Estimate:** 45 minutes

---

## 🔗 Cross-Story Integration

**Integration Points:**

- US-201 (LLM Client) provides foundation for US-202, US-203, US-204
- US-202 (Manager Agent) creates workers (US-203)
- US-203 (Workers) produce output reviewed by US-204
- US-204 (Review) sends feedback to workers (US-203)

**Integration Tests:**

- [ ] Full agent workflow: Create team → Analyze → Form workers → Execute → Review → Revise → Approve
- [ ] Cost tracking across all agent operations
- [ ] Error propagation and recovery
- [ ] Concurrent worker execution

---

## 🚧 Sprint Blockers

**Active Blockers:** None

**Resolved Blockers:** None

---

## 💬 Questions & Decisions

**Open Questions:**

| ID     | Question                                                          | Answer | Impact           |
| ------ | ----------------------------------------------------------------- | ------ | ---------------- |
| Q-2-01 | Should we support models other than Claude 3.5 Sonnet initially? | TBD    | US-201 scope     |
| Q-2-02 | Maximum workers per team - 5 is reasonable?                       | TBD    | US-203 limits    |
| Q-2-03 | Revision limit - 3 rounds sufficient?                             | TBD    | US-204 logic     |

**Decisions Made:**

| Decision                          | Context                     | Rationale                                  | Made By | Date       |
| --------------------------------- | --------------------------- | ------------------------------------------ | ------- | ---------- |
| Use prompt caching for system prompts | Cost optimization           | 90% cost reduction on cache hits           | Team    | 2025-11-08 |
| Max 3 revision rounds per task    | Quality vs. cost tradeoff   | Prevents infinite loops, reasonable limit  | Team    | 2025-11-08 |
| Exponential backoff for retries   | LLM reliability             | Standard pattern for API resilience        | Team    | 2025-11-08 |

---

## 🔧 Dependencies

### Sprint Dependencies

**Depends On:**

- **Sprint 1: Foundation** - Database, API, Authentication **MUST BE 100% COMPLETE**
  - **Validation:** `cargo build` succeeds, migrations applied, health endpoint works
  - **Blocker If Not Complete:** Cannot start Sprint 2

**Blocks:**

- **Sprint 3: Task Orchestration** - Requires working agents from Sprint 2
  - **Critical Deliverable:** Manager and worker agents functional

### External Dependencies

**Third-Party Services:**

- [x] Anthropic API access
  - **Status:** API key required
  - **Contact:** console.anthropic.com
  - **Documentation:** https://docs.anthropic.com
  - **Validation:** Test API call succeeds

**Infrastructure:**

- [x] PostgreSQL database from Sprint 1
  - **Status:** Active
  - **Validation:** `psql $DATABASE_URL -c "SELECT 1"`

---

## ✅ Definition of Done

### Code Quality

- [ ] **Follows GhostPirates patterns** from `docs/patterns/`
  - [ ] Hexagonal Architecture (agents in domain layer)
  - [ ] Domain-Driven Design (aggregates, value objects, events)
  - [ ] CQRS (commands and queries separated)
  - [ ] Repository Pattern (data access abstracted)
  - [ ] Anti-Corruption Layer (LLM API wrapped)
- [ ] **Rust strict compiler settings**
  - [ ] `cargo clippy -- -D warnings` passes
  - [ ] All public APIs documented with `///` rustdoc comments
- [ ] **Format passes:** `cargo fmt --check`
- [ ] **Type check passes:** `cargo check`
- [ ] **Build succeeds:** `cargo build --release`

### Testing

- [ ] **Unit tests** with ≥80% coverage
  - **Run:** `cargo test --lib`
  - **Coverage:** `cargo tarpaulin --out Html`
- [ ] **Integration tests** for LLM calls and repositories
  - **Run:** `cargo test --test integration`
- [ ] **E2E tests** for full agent workflows
  - **Test:** Create team → Execute goal → Complete tasks
- [ ] **Performance tests** pass
  - **LLM Latency:** P95 < 30 seconds
  - **API Latency:** P95 < 500ms (non-LLM endpoints)

### Security

- [ ] **API key management** (no keys in code)
- [ ] **Input validation** on all endpoints
- [ ] **Rate limiting** on LLM calls
- [ ] **Cost limits** enforced per team
- [ ] **Audit logging** for all agent operations
- [ ] **Security scan:** `cargo audit` passes

### Documentation

- [ ] **API endpoints documented** in OpenAPI spec
- [ ] **Agent workflows documented** with diagrams
- [ ] **Prompt engineering patterns** documented
- [ ] **Usage guides** for manager and worker agents
- [ ] **Cost optimization guide** created

### Review

- [ ] **Pull Request created** with clear description
- [ ] **Code review** completed by ≥2 engineers
- [ ] **CI/CD pipeline passing** (all checks green)
- [ ] **Deployed to staging** and smoke tested
- [ ] **Demo-ready** with sample workflows

---

## 📈 Sprint Retrospective

> **Update this section throughout the sprint, not just at the end.**

### What Went Well ✅

**Technical Wins:**

- (Update during sprint)

**Process Wins:**

- (Update during sprint)

**Team Wins:**

- (Update during sprint)

### What to Improve ⚠️

**Technical Challenges:**

- (Update during sprint)

**Process Challenges:**

- (Update during sprint)

**Team Challenges:**

- (Update during sprint)

### Action Items for Next Sprint 🎯

- [ ] **(Action 1)** - TBD
  - **Owner:** TBD
  - **Target:** Sprint 3 kickoff
  - **Success Criteria:** TBD

### Key Learnings 💡

**Technical Learnings:**

- (Update during sprint)

**Process Learnings:**

- (Update during sprint)

---

## 📊 Sprint Metrics

**Velocity:**

- **Planned Story Points:** TBD (sum of estimates)
- **Completed Story Points:** TBD
- **Velocity:** TBD%
- **Comparison to Sprint 1:** TBD

**Code Quality:**

- **Code Coverage:** TBD% (target: ≥80%)
- **Lines of Code Added:** TBD
- **Lines of Code Deleted:** TBD

**CI/CD:**

- **Build Success Rate:** TBD% (target: 100%)
- **Average Build Time:** TBD minutes
- **Deployments to Staging:** TBD

**Bugs & Issues:**

- **Critical Bugs:** 0 (target)
- **High Priority Bugs:** TBD
- **Bug Fix Time (Average):** TBD hours

**Performance:**

- **API P95 Latency:** TBD ms (target: <500ms non-LLM, <30s LLM)
- **Error Rate:** TBD% (target: <0.1%)
- **LLM Cost Per Goal:** TBD (target: <$1.00)

---

## 📝 Sprint Notes

**Daily Standup Highlights:**

(Add notes as sprint progresses)

**Mid-Sprint Check-In (Week 1 End):**

- **Progress:** TBD%
- **Risks:** TBD
- **Adjustments:** TBD

---

## 🎯 Next Steps

**After Sprint Completion:**

1. [ ] Conduct sprint retrospective meeting (60 minutes)
2. [ ] Update sprint metrics in this document
3. [ ] Archive sprint-specific branches
4. [ ] Deploy to production (if approved)
5. [ ] Notify stakeholders of completion
6. [ ] Begin Sprint 3 planning (Task Orchestration)

**Handoff to Sprint 3:**

- [ ] Agent system functional and tested
- [ ] LLM integration stable and cost-effective
- [ ] Review loops working correctly
- [ ] Documentation complete for Sprint 3 team

---

## 📞 Team Contacts

**Sprint Team:**

- **Product Owner:** TBD
- **Tech Lead:** TBD
- **Backend Engineers:** TBD
- **DevOps:** TBD
- **QA:** TBD

**Stakeholders:**

- **Executive Sponsor:** TBD

**Communication Channels:**

- **Daily Standups:** TBD
- **Sprint Planning:** TBD
- **Retrospective:** TBD
- **Slack Channel:** #sprint-2-agent-system

---

**End of Sprint 2 Document**
