# Cost Optimization Strategy

**Focus**: LLM Cost Reduction → Intelligent Caching → Model Routing → Budget Management
**Priority**: High (directly impacts profitability)
**Cross-cutting**: Applies across all phases

---

## Epic 1: Semantic Caching with Redis

### Task 1.1: Implement Semantic Cache

**Type**: Backend
**Dependencies**: Redis available

**Subtasks**:

- [ ] 1.1.1: Add vector similarity dependencies

```toml
# apps/api/Cargo.toml
[dependencies]
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
serde_json = "1.0"
sha2 = "0.10"
```

- [ ] 1.1.2: Create semantic cache service

```rust
// apps/api/src/infrastructure/cache/semantic_cache.rs
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct CachedResponse {
    pub response: String,
    pub tokens_used: i32,
    pub cost: f64,
    pub created_at: i64,
}

pub struct SemanticCache {
    redis: redis::Client,
    similarity_threshold: f64,
}

impl SemanticCache {
    pub fn new(redis_url: &str, similarity_threshold: f64) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis: redis::Client::open(redis_url)?,
            similarity_threshold,
        })
    }

    pub async fn get(
        &self,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<Option<CachedResponse>, CacheError> {
        let cache_key = self.generate_cache_key(prompt, system_prompt);
        let mut conn = self.redis.get_async_connection().await?;

        if let Ok(cached) = conn.get::<_, String>(&cache_key).await {
            if let Ok(response) = serde_json::from_str::<CachedResponse>(&cached) {
                tracing::info!(
                    "Cache HIT for prompt hash: {} (saved ${:.4})",
                    &cache_key[..8],
                    response.cost
                );
                return Ok(Some(response));
            }
        }

        tracing::debug!("Cache MISS for prompt hash: {}", &cache_key[..8]);
        Ok(None)
    }

    pub async fn set(
        &self,
        prompt: &str,
        system_prompt: &str,
        response: String,
        tokens_used: i32,
        cost: f64,
        ttl: Duration,
    ) -> Result<(), CacheError> {
        let cache_key = self.generate_cache_key(prompt, system_prompt);
        let mut conn = self.redis.get_async_connection().await?;

        let cached_response = CachedResponse {
            response,
            tokens_used,
            cost,
            created_at: chrono::Utc::now().timestamp(),
        };

        let serialized = serde_json::to_string(&cached_response)?;
        conn.set_ex(&cache_key, serialized, ttl.as_secs() as usize).await?;

        tracing::info!(
            "Cached response: {} (cost: ${:.4}, tokens: {})",
            &cache_key[..8],
            cost,
            tokens_used
        );

        Ok(())
    }

    fn generate_cache_key(&self, prompt: &str, system_prompt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(system_prompt.as_bytes());
        hasher.update(b"|");
        hasher.update(prompt.as_bytes());
        format!("llm_cache:{:x}", hasher.finalize())
    }

    pub async fn get_cache_stats(&self) -> Result<CacheStats, CacheError> {
        let mut conn = self.redis.get_async_connection().await?;

        let keys: Vec<String> = conn.keys("llm_cache:*").await?;
        let mut total_cost_saved = 0.0;
        let mut total_tokens_saved = 0;

        for key in &keys {
            if let Ok(cached) = conn.get::<_, String>(key).await {
                if let Ok(response) = serde_json::from_str::<CachedResponse>(&cached) {
                    total_cost_saved += response.cost;
                    total_tokens_saved += response.tokens_used;
                }
            }
        }

        Ok(CacheStats {
            cached_responses: keys.len(),
            total_cost_saved,
            total_tokens_saved,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct CacheStats {
    pub cached_responses: usize,
    pub total_cost_saved: f64,
    pub total_tokens_saved: i32,
}

#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

- [ ] 1.1.3: Integrate cache with LLM client

```rust
// apps/api/src/infrastructure/llm/cached_client.rs
use crate::infrastructure::cache::semantic_cache::{SemanticCache, CachedResponse};
use std::time::Duration;

pub struct CachedLlmClient {
    client: ClaudeClient,
    cache: SemanticCache,
    cache_ttl: Duration,
}

impl CachedLlmClient {
    pub fn new(
        client: ClaudeClient,
        redis_url: &str,
        similarity_threshold: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self {
            client,
            cache: SemanticCache::new(redis_url, similarity_threshold)?,
            cache_ttl: Duration::from_secs(3600), // 1 hour default
        })
    }

    pub async fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, LlmError> {
        // Check cache first
        if let Some(cached) = self.cache.get(user_prompt, system_prompt).await? {
            return Ok(cached.response);
        }

        // Cache miss - make actual API call
        let start = std::time::Instant::now();
        let response = self.client.complete(system_prompt, user_prompt).await?;
        let duration = start.elapsed();

        // Calculate cost (approximate)
        let tokens = estimate_tokens(&response);
        let cost = calculate_cost("claude-3-5-sonnet-20241022", tokens);

        // Store in cache
        self.cache
            .set(
                user_prompt,
                system_prompt,
                response.clone(),
                tokens,
                cost,
                self.cache_ttl,
            )
            .await?;

        tracing::info!(
            "LLM API call completed in {:?} (cost: ${:.4}, tokens: {})",
            duration,
            cost,
            tokens
        );

        Ok(response)
    }

    pub async fn get_cache_stats(&self) -> Result<CacheStats, CacheError> {
        self.cache.get_cache_stats().await
    }
}

fn estimate_tokens(text: &str) -> i32 {
    // Rough estimation: ~4 characters per token
    (text.len() / 4) as i32
}

fn calculate_cost(model: &str, tokens: i32) -> f64 {
    match model {
        "claude-3-5-sonnet-20241022" => {
            // $3 per 1M input tokens, $15 per 1M output tokens
            // Assume 50/50 split for simplicity
            let input_cost = (tokens as f64 / 2.0) * (3.0 / 1_000_000.0);
            let output_cost = (tokens as f64 / 2.0) * (15.0 / 1_000_000.0);
            input_cost + output_cost
        }
        _ => 0.0,
    }
}
```

- [ ] 1.1.4: Add cache warming strategy

```rust
// apps/api/src/services/cache_warmer.rs
pub struct CacheWarmer {
    cache: SemanticCache,
    llm_client: ClaudeClient,
}

impl CacheWarmer {
    pub async fn warm_common_prompts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let common_prompts = vec![
            (
                "You are a project manager analyzing goals.",
                "Analyze this goal: Build a REST API",
            ),
            (
                "You are a senior developer creating task breakdowns.",
                "Break down: Implement user authentication",
            ),
            // Add more common patterns
        ];

        for (system, user) in common_prompts {
            // Check if already cached
            if self.cache.get(user, system).await?.is_none() {
                tracing::info!("Warming cache for: {}", &user[..30]);

                let response = self.llm_client.complete(system, user).await?;
                let tokens = estimate_tokens(&response);
                let cost = calculate_cost("claude-3-5-sonnet-20241022", tokens);

                self.cache
                    .set(user, system, response, tokens, cost, Duration::from_secs(86400))
                    .await?;
            }
        }

        Ok(())
    }
}
```

- [ ] 1.1.5: Add cache analytics endpoint

```rust
// apps/api/src/api/handlers/analytics.rs
use axum::Json;

pub async fn get_cache_stats(
    State(cache): State<SemanticCache>,
) -> Result<Json<CacheStats>, StatusCode> {
    let stats = cache
        .get_cache_stats()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(stats))
}
```

**Acceptance Criteria**:

- [ ] Cache hit rate > 30% for similar prompts
- [ ] Cache reduces API costs by > 25%
- [ ] Cache TTL configurable
- [ ] Cache stats accessible via API
- [ ] No stale responses served

---

## Epic 2: Prompt Compression Techniques

### Task 2.1: Implement Prompt Optimization

**Type**: Backend
**Dependencies**: None

**Subtasks**:

- [ ] 2.1.1: Create prompt compressor

```rust
// apps/api/src/infrastructure/llm/prompt_compressor.rs
pub struct PromptCompressor;

impl PromptCompressor {
    pub fn compress(prompt: &str) -> String {
        let mut compressed = prompt.to_string();

        // Remove extra whitespace
        compressed = compressed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        // Remove redundant instructions
        compressed = Self::remove_redundancy(&compressed);

        // Use abbreviations for common terms
        compressed = Self::apply_abbreviations(&compressed);

        compressed
    }

    fn remove_redundancy(text: &str) -> String {
        // Remove phrases like "please", "kindly", etc.
        text.replace("please ", "")
            .replace("kindly ", "")
            .replace("I need you to ", "")
            .replace("Could you ", "")
    }

    fn apply_abbreviations(text: &str) -> String {
        text.replace("authentication", "auth")
            .replace("application", "app")
            .replace("configuration", "config")
            .replace("database", "db")
            .replace("repository", "repo")
    }

    pub fn estimate_token_savings(original: &str, compressed: &str) -> i32 {
        let original_tokens = estimate_tokens(original);
        let compressed_tokens = estimate_tokens(compressed);
        original_tokens - compressed_tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression() {
        let original = "Please analyze this authentication configuration for the database repository";
        let compressed = PromptCompressor::compress(original);

        assert!(compressed.len() < original.len());
        assert_eq!(compressed, "analyze this auth config for the db repo");
    }
}
```

- [ ] 2.1.2: Create context pruning strategy

```rust
// apps/api/src/infrastructure/llm/context_pruner.rs
use serde_json::Value;

pub struct ContextPruner {
    max_tokens: i32,
}

impl ContextPruner {
    pub fn new(max_tokens: i32) -> Self {
        Self { max_tokens }
    }

    pub fn prune_context(&self, context: &Value) -> Value {
        let mut pruned = context.clone();

        // Keep only essential fields
        if let Some(steps) = pruned.get_mut("steps") {
            if let Some(array) = steps.as_array_mut() {
                // Keep only last N steps
                let max_steps = 5;
                if array.len() > max_steps {
                    *array = array[array.len() - max_steps..].to_vec();
                }

                // Remove verbose fields from each step
                for step in array.iter_mut() {
                    if let Some(obj) = step.as_object_mut() {
                        obj.remove("debug_info");
                        obj.remove("raw_output");
                        obj.remove("metadata");
                    }
                }
            }
        }

        pruned
    }

    pub fn summarize_history(&self, history: &[String]) -> String {
        if history.len() <= 3 {
            return history.join("; ");
        }

        // Summarize older entries
        let recent: Vec<_> = history.iter().rev().take(3).rev().collect();
        let older_count = history.len() - 3;

        format!(
            "[{} previous steps]; {}",
            older_count,
            recent.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("; ")
        )
    }
}
```

- [ ] 2.1.3: Implement smart prompt templates

```rust
// apps/api/src/infrastructure/llm/prompt_templates.rs
pub struct PromptTemplate {
    template: String,
    variables: Vec<String>,
}

impl PromptTemplate {
    pub fn new(template: &str) -> Self {
        let variables = Self::extract_variables(template);
        Self {
            template: template.to_string(),
            variables,
        }
    }

    fn extract_variables(template: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .map(|cap| cap[1].to_string())
            .collect()
    }

    pub fn render(&self, values: &std::collections::HashMap<String, String>) -> String {
        let mut result = self.template.clone();

        for (key, value) in values {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}

// Optimized templates
pub const TASK_DECOMPOSITION_TEMPLATE: &str = r#"
Break down: {{goal}}
Output JSON:
[{
  "title": "...",
  "desc": "...",
  "criteria": ["..."]
}]
"#;

pub const REVIEW_TEMPLATE: &str = r#"
Review:
Task: {{title}}
Criteria: {{criteria}}
Output: {{output}}

Decision: approve/revise/reject
Reason: ...
"#;
```

- [ ] 2.1.4: Measure compression impact

```rust
// apps/api/src/services/cost_tracker.rs
#[derive(Debug, Serialize)]
pub struct CompressionMetrics {
    pub original_tokens: i32,
    pub compressed_tokens: i32,
    pub tokens_saved: i32,
    pub cost_saved: f64,
    pub compression_ratio: f64,
}

impl CompressionMetrics {
    pub fn calculate(original: &str, compressed: &str) -> Self {
        let original_tokens = estimate_tokens(original);
        let compressed_tokens = estimate_tokens(compressed);
        let tokens_saved = original_tokens - compressed_tokens;

        let cost_saved = calculate_cost("claude-3-5-sonnet-20241022", tokens_saved);
        let compression_ratio = compressed_tokens as f64 / original_tokens as f64;

        Self {
            original_tokens,
            compressed_tokens,
            tokens_saved,
            cost_saved,
            compression_ratio,
        }
    }
}
```

**Acceptance Criteria**:

- [ ] Prompt compression reduces tokens by > 15%
- [ ] Context pruning keeps only essential data
- [ ] Templates optimized for minimal tokens
- [ ] No loss of semantic meaning
- [ ] Compression metrics tracked

---

## Epic 3: Model Routing Strategy

### Task 3.1: Intelligent Model Selection

**Type**: Backend
**Dependencies**: Multiple LLM providers configured

**Subtasks**:

- [ ] 3.1.1: Define model routing rules

```rust
// apps/api/src/infrastructure/llm/model_router.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelTier {
    Premium,   // Claude 3.5 Sonnet - complex tasks
    Standard,  // Claude 3 Haiku - moderate tasks
    Economy,   // GPT-3.5 - simple tasks
}

#[derive(Debug)]
pub struct ModelRouter {
    config: RouterConfig,
}

#[derive(Debug)]
pub struct RouterConfig {
    pub complexity_threshold_premium: f64,
    pub complexity_threshold_standard: f64,
}

impl ModelRouter {
    pub fn new(config: RouterConfig) -> Self {
        Self { config }
    }

    pub fn select_model(&self, task: &TaskAnalysis) -> ModelTier {
        let complexity = self.calculate_complexity(task);

        if complexity > self.config.complexity_threshold_premium {
            ModelTier::Premium
        } else if complexity > self.config.complexity_threshold_standard {
            ModelTier::Standard
        } else {
            ModelTier::Economy
        }
    }

    fn calculate_complexity(&self, task: &TaskAnalysis) -> f64 {
        let mut score = 0.0;

        // Factors that increase complexity
        score += task.description.split_whitespace().count() as f64 * 0.1;
        score += task.acceptance_criteria.len() as f64 * 5.0;
        score += if task.requires_reasoning { 20.0 } else { 0.0 };
        score += if task.requires_code_generation { 15.0 } else { 0.0 };
        score += task.estimated_steps as f64 * 3.0;

        score
    }

    pub fn get_model_config(&self, tier: &ModelTier) -> ModelConfig {
        match tier {
            ModelTier::Premium => ModelConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                cost_per_1m_input: 3.0,
                cost_per_1m_output: 15.0,
                max_tokens: 8192,
            },
            ModelTier::Standard => ModelConfig {
                provider: "anthropic".to_string(),
                model: "claude-3-haiku-20240307".to_string(),
                cost_per_1m_input: 0.25,
                cost_per_1m_output: 1.25,
                max_tokens: 4096,
            },
            ModelTier::Economy => ModelConfig {
                provider: "openai".to_string(),
                model: "gpt-3.5-turbo".to_string(),
                cost_per_1m_input: 0.5,
                cost_per_1m_output: 1.5,
                max_tokens: 4096,
            },
        }
    }
}

#[derive(Debug)]
pub struct TaskAnalysis {
    pub description: String,
    pub acceptance_criteria: Vec<String>,
    pub requires_reasoning: bool,
    pub requires_code_generation: bool,
    pub estimated_steps: i32,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub cost_per_1m_input: f64,
    pub cost_per_1m_output: f64,
    pub max_tokens: i32,
}
```

- [ ] 3.1.2: Implement multi-provider LLM client

```rust
// apps/api/src/infrastructure/llm/multi_provider_client.rs
pub struct MultiProviderClient {
    claude_client: ClaudeClient,
    openai_client: OpenAiClient,
    router: ModelRouter,
}

impl MultiProviderClient {
    pub fn new(
        claude_api_key: String,
        openai_api_key: String,
        router: ModelRouter,
    ) -> Self {
        Self {
            claude_client: ClaudeClient::new(claude_api_key),
            openai_client: OpenAiClient::new(openai_api_key),
            router,
        }
    }

    pub async fn complete_with_routing(
        &self,
        task_analysis: &TaskAnalysis,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<LlmResponse, LlmError> {
        let tier = self.router.select_model(task_analysis);
        let config = self.router.get_model_config(&tier);

        tracing::info!(
            "Routing to {} model: {} (estimated cost: ${:.4}/1K tokens)",
            config.provider,
            config.model,
            (config.cost_per_1m_input + config.cost_per_1m_output) / 1000.0
        );

        let start = std::time::Instant::now();

        let response = match config.provider.as_str() {
            "anthropic" => {
                self.claude_client
                    .complete_with_model(&config.model, system_prompt, user_prompt)
                    .await?
            }
            "openai" => {
                self.openai_client
                    .complete_with_model(&config.model, system_prompt, user_prompt)
                    .await?
            }
            _ => return Err(LlmError::UnsupportedProvider(config.provider)),
        };

        let duration = start.elapsed();

        Ok(LlmResponse {
            content: response.content,
            model: config.model,
            tier,
            tokens_used: response.tokens_used,
            cost: Self::calculate_actual_cost(&config, &response),
            duration,
        })
    }

    fn calculate_actual_cost(config: &ModelConfig, response: &ApiResponse) -> f64 {
        let input_cost = response.input_tokens as f64 * (config.cost_per_1m_input / 1_000_000.0);
        let output_cost = response.output_tokens as f64 * (config.cost_per_1m_output / 1_000_000.0);
        input_cost + output_cost
    }
}

#[derive(Debug)]
pub struct LlmResponse {
    pub content: String,
    pub model: String,
    pub tier: ModelTier,
    pub tokens_used: i32,
    pub cost: f64,
    pub duration: std::time::Duration,
}
```

- [ ] 3.1.3: Add model performance tracking

```rust
// apps/api/src/services/model_performance_tracker.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ModelPerformanceTracker {
    stats: Arc<RwLock<HashMap<String, ModelStats>>>,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ModelStats {
    pub total_calls: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub avg_duration_ms: f64,
    pub success_rate: f64,
}

impl ModelPerformanceTracker {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn record_call(
        &self,
        model: &str,
        tokens: i32,
        cost: f64,
        duration: std::time::Duration,
        success: bool,
    ) {
        let mut stats = self.stats.write().await;
        let entry = stats.entry(model.to_string()).or_default();

        entry.total_calls += 1;
        entry.total_tokens += tokens as u64;
        entry.total_cost += cost;

        // Update rolling average duration
        let new_avg = (entry.avg_duration_ms * (entry.total_calls - 1) as f64
            + duration.as_millis() as f64)
            / entry.total_calls as f64;
        entry.avg_duration_ms = new_avg;

        // Update success rate
        let successes = (entry.success_rate * (entry.total_calls - 1) as f64
            + if success { 1.0 } else { 0.0 })
            / entry.total_calls as f64;
        entry.success_rate = successes;
    }

    pub async fn get_stats(&self) -> HashMap<String, ModelStats> {
        self.stats.read().await.clone()
    }

    pub async fn get_cost_breakdown(&self) -> Vec<(String, f64, f64)> {
        let stats = self.stats.read().await;

        let mut breakdown: Vec<_> = stats
            .iter()
            .map(|(model, stats)| {
                let percentage = if stats.total_cost > 0.0 {
                    stats.total_cost / stats.total_cost * 100.0
                } else {
                    0.0
                };
                (model.clone(), stats.total_cost, percentage)
            })
            .collect();

        breakdown.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        breakdown
    }
}
```

**Acceptance Criteria**:

- [ ] Simple tasks routed to cheaper models
- [ ] Complex tasks routed to premium models
- [ ] Model selection accuracy > 90%
- [ ] Overall cost reduced by > 40%
- [ ] Model performance tracked
- [ ] Can override routing manually

---

## Epic 4: Token Tracking & Budget Enforcement

### Task 4.1: Build Budget Management System

**Type**: Fullstack
**Dependencies**: Cost tracking table exists

**Subtasks**:

- [ ] 4.1.1: Create budget enforcement service

```rust
// apps/api/src/services/budget_enforcer.rs
use crate::domain::teams::Team;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct BudgetEnforcer {
    pool: PgPool,
}

impl BudgetEnforcer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn check_budget(
        &self,
        team_id: Uuid,
        estimated_cost: Decimal,
    ) -> Result<BudgetCheckResult, BudgetError> {
        // Get team budget limit
        let team = sqlx::query_as!(
            Team,
            r#"
            SELECT * FROM teams WHERE id = $1
            "#,
            team_id
        )
        .fetch_one(&self.pool)
        .await?;

        let budget_limit = match team.budget_limit {
            Some(limit) => limit,
            None => return Ok(BudgetCheckResult::Allowed), // No limit set
        };

        // Calculate current spend
        let current_spend = sqlx::query!(
            r#"
            SELECT COALESCE(SUM(amount), 0) as "total!"
            FROM cost_tracking
            WHERE team_id = $1
            "#,
            team_id
        )
        .fetch_one(&self.pool)
        .await?
        .total;

        let projected_total = current_spend + estimated_cost;
        let utilization = (current_spend / budget_limit) * Decimal::from(100);

        if projected_total > budget_limit {
            Ok(BudgetCheckResult::Exceeded {
                current_spend,
                budget_limit,
                projected_total,
            })
        } else if utilization > Decimal::from(80) {
            Ok(BudgetCheckResult::Warning {
                current_spend,
                budget_limit,
                utilization: utilization.to_f64().unwrap(),
            })
        } else {
            Ok(BudgetCheckResult::Allowed)
        }
    }

    pub async fn record_cost(
        &self,
        team_id: Uuid,
        task_id: Option<Uuid>,
        category: &str,
        provider: &str,
        model: &str,
        amount: Decimal,
        unit_count: i32,
    ) -> Result<(), BudgetError> {
        sqlx::query!(
            r#"
            INSERT INTO cost_tracking (
                team_id, task_id, category, provider, model, amount, unit_count
            )
            VALUES ($1, $2, $3::cost_category, $4, $5, $6, $7)
            "#,
            team_id,
            task_id,
            category,
            provider,
            model,
            amount,
            unit_count
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_budget_status(&self, team_id: Uuid) -> Result<BudgetStatus, BudgetError> {
        let team = sqlx::query_as!(Team, "SELECT * FROM teams WHERE id = $1", team_id)
            .fetch_one(&self.pool)
            .await?;

        let current_spend = sqlx::query!(
            "SELECT COALESCE(SUM(amount), 0) as total FROM cost_tracking WHERE team_id = $1",
            team_id
        )
        .fetch_one(&self.pool)
        .await?
        .total;

        Ok(BudgetStatus {
            team_id,
            budget_limit: team.budget_limit,
            current_spend,
            remaining: team.budget_limit.map(|limit| limit - current_spend),
            utilization: team
                .budget_limit
                .map(|limit| (current_spend / limit * Decimal::from(100)).to_f64().unwrap()),
        })
    }
}

#[derive(Debug)]
pub enum BudgetCheckResult {
    Allowed,
    Warning {
        current_spend: Decimal,
        budget_limit: Decimal,
        utilization: f64,
    },
    Exceeded {
        current_spend: Decimal,
        budget_limit: Decimal,
        projected_total: Decimal,
    },
}

#[derive(Debug, Serialize)]
pub struct BudgetStatus {
    pub team_id: Uuid,
    pub budget_limit: Option<Decimal>,
    pub current_spend: Decimal,
    pub remaining: Option<Decimal>,
    pub utilization: Option<f64>,
}

#[derive(Debug, thiserror::Error)]
pub enum BudgetError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Budget exceeded")]
    BudgetExceeded,
}
```

- [ ] 4.1.2: Integrate budget checks with agent execution

```rust
// apps/api/src/agents/worker.rs
impl WorkerAgent {
    pub async fn execute_task_with_budget(
        &self,
        task: &Task,
        budget_enforcer: &BudgetEnforcer,
    ) -> Result<TaskOutput, AgentError> {
        let estimated_cost = self.estimate_task_cost(task);

        // Check budget before execution
        match budget_enforcer.check_budget(task.team_id, estimated_cost).await? {
            BudgetCheckResult::Exceeded { .. } => {
                return Err(AgentError::BudgetExceeded);
            }
            BudgetCheckResult::Warning { utilization, .. } => {
                tracing::warn!(
                    "Budget utilization high: {:.1}% for team {}",
                    utilization,
                    task.team_id
                );
            }
            BudgetCheckResult::Allowed => {}
        }

        // Execute task with cost tracking
        let result = self.execute_task(task).await?;

        // Record actual cost
        budget_enforcer
            .record_cost(
                task.team_id,
                Some(task.id),
                "api_call",
                "anthropic",
                "claude-3-5-sonnet-20241022",
                result.cost,
                result.tokens_used,
            )
            .await?;

        Ok(result)
    }

    fn estimate_task_cost(&self, task: &Task) -> Decimal {
        // Rough estimation based on task complexity
        let description_tokens = estimate_tokens(&task.description);
        let criteria_tokens = estimate_tokens(&task.acceptance_criteria.join(" "));

        let estimated_tokens = (description_tokens + criteria_tokens) * 3; // 3x for round trips
        let cost = calculate_cost("claude-3-5-sonnet-20241022", estimated_tokens);

        Decimal::from_f64_retain(cost).unwrap()
    }
}
```

- [ ] 4.1.3: Create budget dashboard endpoint

```rust
// apps/api/src/api/handlers/budget.rs
pub async fn get_team_budget_status(
    Path(team_id): Path<Uuid>,
    State(enforcer): State<BudgetEnforcer>,
) -> Result<Json<BudgetStatus>, StatusCode> {
    let status = enforcer
        .get_budget_status(team_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(status))
}

#[derive(Serialize)]
pub struct CostBreakdown {
    pub by_category: Vec<CategoryCost>,
    pub by_model: Vec<ModelCost>,
    pub timeline: Vec<DailyCost>,
}

pub async fn get_cost_breakdown(
    Path(team_id): Path<Uuid>,
    State(pool): State<PgPool>,
) -> Result<Json<CostBreakdown>, StatusCode> {
    // By category
    let by_category = sqlx::query_as!(
        CategoryCost,
        r#"
        SELECT
            category as "category!",
            SUM(amount) as "total_cost!",
            SUM(unit_count) as "total_units!"
        FROM cost_tracking
        WHERE team_id = $1
        GROUP BY category
        ORDER BY total_cost DESC
        "#,
        team_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // By model
    let by_model = sqlx::query_as!(
        ModelCost,
        r#"
        SELECT
            model,
            SUM(amount) as "total_cost!",
            COUNT(*) as "call_count!"
        FROM cost_tracking
        WHERE team_id = $1
        GROUP BY model
        ORDER BY total_cost DESC
        "#,
        team_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Daily timeline
    let timeline = sqlx::query_as!(
        DailyCost,
        r#"
        SELECT
            DATE(created_at) as "date!",
            SUM(amount) as "total_cost!"
        FROM cost_tracking
        WHERE team_id = $1
        GROUP BY DATE(created_at)
        ORDER BY date ASC
        "#,
        team_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CostBreakdown {
        by_category,
        by_model,
        timeline,
    }))
}
```

**Acceptance Criteria**:

- [ ] Budget enforcement blocks over-budget tasks
- [ ] Budget warnings at 80% utilization
- [ ] Costs tracked per task
- [ ] Cost breakdown by category and model
- [ ] Budget status API working
- [ ] Real-time budget updates

---

## Epic 5: Cost Reporting Dashboard

### Task 5.1: Build Cost Analytics UI

**Type**: Frontend
**Dependencies**: Budget APIs complete

**Subtasks**:

- [ ] 5.1.1: Create budget status component

```typescript
// apps/frontend/src/components/budget/BudgetStatus.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { Card } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { AlertTriangle } from 'lucide-react';

export function BudgetStatus({ teamId }: { teamId: string }) {
  const { data: budget } = useQuery({
    queryKey: ['budget', teamId],
    queryFn: () => fetchBudgetStatus(teamId),
    refetchInterval: 30000,
  });

  if (!budget) return null;

  const utilization = budget.utilization || 0;
  const isWarning = utilization > 80;
  const isExceeded = utilization >= 100;

  return (
    <Card className={`p-6 ${isExceeded ? 'border-red-500' : isWarning ? 'border-yellow-500' : ''}`}>
      <div className="flex items-start justify-between mb-4">
        <div>
          <h3 className="text-lg font-semibold">Budget Status</h3>
          <p className="text-sm text-gray-500">
            {budget.budget_limit ? `$${budget.budget_limit} limit` : 'No limit set'}
          </p>
        </div>
        {isWarning && (
          <AlertTriangle className={`h-6 w-6 ${isExceeded ? 'text-red-500' : 'text-yellow-500'}`} />
        )}
      </div>

      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span>Current Spend:</span>
          <span className="font-semibold">${budget.current_spend.toFixed(2)}</span>
        </div>

        {budget.remaining !== null && (
          <div className="flex justify-between text-sm">
            <span>Remaining:</span>
            <span className={budget.remaining < 0 ? 'text-red-500 font-semibold' : ''}>
              ${Math.max(0, budget.remaining).toFixed(2)}
            </span>
          </div>
        )}

        {budget.budget_limit && (
          <Progress
            value={utilization}
            className={`h-2 ${isExceeded ? '[&>div]:bg-red-500' : isWarning ? '[&>div]:bg-yellow-500' : ''}`}
          />
        )}

        <p className="text-xs text-gray-500 text-right">
          {utilization.toFixed(1)}% utilized
        </p>
      </div>
    </Card>
  );
}
```

- [ ] 5.1.2: Create cost breakdown chart

```typescript
// apps/frontend/src/components/budget/CostBreakdown.tsx
'use client';

import { useQuery } from '@tanstack/react-query';
import { Card } from '@/components/ui/card';
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs';
import {
  BarChart,
  Bar,
  PieChart,
  Pie,
  Cell,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

const COLORS = ['#3b82f6', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6'];

export function CostBreakdown({ teamId }: { teamId: string }) {
  const { data } = useQuery({
    queryKey: ['cost-breakdown', teamId],
    queryFn: () => fetchCostBreakdown(teamId),
  });

  if (!data) return null;

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Cost Breakdown</h3>

      <Tabs defaultValue="category">
        <TabsList>
          <TabsTrigger value="category">By Category</TabsTrigger>
          <TabsTrigger value="model">By Model</TabsTrigger>
          <TabsTrigger value="timeline">Timeline</TabsTrigger>
        </TabsList>

        <TabsContent value="category" className="mt-4">
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={data.by_category}
                dataKey="total_cost"
                nameKey="category"
                cx="50%"
                cy="50%"
                outerRadius={100}
                label={(entry) => `$${entry.total_cost.toFixed(2)}`}
              >
                {data.by_category.map((entry, index) => (
                  <Cell key={entry.category} fill={COLORS[index % COLORS.length]} />
                ))}
              </Pie>
              <Tooltip formatter={(value) => `$${value.toFixed(4)}`} />
              <Legend />
            </PieChart>
          </ResponsiveContainer>
        </TabsContent>

        <TabsContent value="model" className="mt-4">
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={data.by_model}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="model" />
              <YAxis />
              <Tooltip formatter={(value) => `$${value.toFixed(4)}`} />
              <Legend />
              <Bar dataKey="total_cost" fill="#3b82f6" />
            </BarChart>
          </ResponsiveContainer>

          <div className="mt-4 space-y-2">
            {data.by_model.map((model) => (
              <div key={model.model} className="flex justify-between text-sm">
                <span>{model.model}</span>
                <span>
                  ${model.total_cost.toFixed(4)} ({model.call_count} calls)
                </span>
              </div>
            ))}
          </div>
        </TabsContent>

        <TabsContent value="timeline" className="mt-4">
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={data.timeline}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="date" />
              <YAxis />
              <Tooltip formatter={(value) => `$${value.toFixed(4)}`} />
              <Bar dataKey="total_cost" fill="#10b981" />
            </BarChart>
          </ResponsiveContainer>
        </TabsContent>
      </Tabs>
    </Card>
  );
}
```

- [ ] 5.1.3: Create cost optimization recommendations

```typescript
// apps/frontend/src/components/budget/CostOptimizationTips.tsx
export function CostOptimizationTips({ breakdown }: { breakdown: CostBreakdown }) {
  const recommendations = generateRecommendations(breakdown);

  return (
    <Card className="p-6">
      <h3 className="text-lg font-semibold mb-4">Optimization Opportunities</h3>

      <div className="space-y-3">
        {recommendations.map((rec, index) => (
          <div key={index} className="flex items-start gap-3 p-3 bg-blue-50 rounded-lg">
            <Lightbulb className="h-5 w-5 text-blue-500 mt-0.5" />
            <div>
              <p className="font-medium text-sm">{rec.title}</p>
              <p className="text-sm text-gray-600">{rec.description}</p>
              <p className="text-xs text-green-600 mt-1">
                Potential savings: ${rec.potential_savings.toFixed(2)}
              </p>
            </div>
          </div>
        ))}
      </div>
    </Card>
  );
}

function generateRecommendations(breakdown: CostBreakdown) {
  const recommendations = [];

  // Check if premium models are overused
  const premiumCost = breakdown.by_model
    .filter(m => m.model.includes('sonnet'))
    .reduce((sum, m) => sum + m.total_cost, 0);

  if (premiumCost > breakdown.total * 0.7) {
    recommendations.push({
      title: 'Consider using cheaper models',
      description: '70% of costs from premium models. Route simple tasks to Haiku or GPT-3.5.',
      potential_savings: premiumCost * 0.4,
    });
  }

  // Check cache effectiveness
  if (breakdown.cache_hit_rate < 0.3) {
    recommendations.push({
      title: 'Improve cache hit rate',
      description: 'Cache hit rate is low. Warm cache with common patterns.',
      potential_savings: breakdown.total * 0.25,
    });
  }

  return recommendations;
}
```

**Acceptance Criteria**:

- [ ] Budget status displays correctly
- [ ] Cost breakdown charts working
- [ ] Category, model, and timeline views
- [ ] Optimization recommendations shown
- [ ] Real-time cost updates
- [ ] Export cost reports to CSV

---

## Success Criteria - Cost Optimization Complete

- [ ] Semantic caching reduces API costs by > 25%
- [ ] Prompt compression saves > 15% tokens
- [ ] Model routing reduces overall costs by > 40%
- [ ] Budget enforcement prevents overages
- [ ] Cost tracking accurate to 4 decimal places
- [ ] Dashboard provides actionable insights
- [ ] Cache hit rate > 30%
- [ ] Total cost reduction > 60%

---

## Next Steps

Proceed to [13-security-compliance.md](./13-security-compliance.md) for security hardening.

---

**Cost Optimization: Intelligent caching, routing, and budget control**
