# Pricing Model & Billing Implementation Plan

**Version**: 1.0
**Last Updated**: November 8, 2025
**Dependencies**: Phase 1-8, monitoring (16), database (03)
**Estimated Duration**: 3-4 weeks
**Status**: Ready for Implementation

---

## Executive Summary

Ghost Pirates uses a **per-mission pricing model** instead of traditional SaaS subscriptions. Users pay only for completed missions based on actual resource consumption (LLM tokens, tool usage, execution time). This document outlines the complete implementation of cost tracking, billing, invoicing, and payment processing.

**Pricing Philosophy**:
- **Pay for outcomes, not seats**: Charge per mission completion
- **Transparent costs**: Real-time cost breakdown visible during execution
- **Predictable pricing**: Upfront estimates before mission starts
- **Fair usage**: Credits for failed missions, refunds for system failures

**Revenue Model**:
- **Starter Tier**: $5-25 per mission (simple missions, 3-4 agents)
- **Pro Tier**: $25-100 per mission (complex missions, 5-8 agents)
- **Enterprise Tier**: Custom pricing (volume discounts, dedicated support)

---

## Table of Contents

1. [Epic 1: Per-Mission Cost Calculation](#epic-1-per-mission-cost-calculation)
2. [Epic 2: Real-time Budget Monitoring](#epic-2-real-time-budget-monitoring)
3. [Epic 3: Billing API Implementation](#epic-3-billing-api-implementation)
4. [Epic 4: Invoice Generation](#epic-4-invoice-generation)
5. [Epic 5: Payment Integration (Stripe)](#epic-5-payment-integration-stripe)
6. [Epic 6: Cost Breakdown Dashboard](#epic-6-cost-breakdown-dashboard)
7. [Epic 7: Pricing Tiers](#epic-7-pricing-tiers)

---

## Epic 1: Per-Mission Cost Calculation

### Overview

Implement accurate cost tracking for every component of mission execution: LLM API calls, tool usage, infrastructure, and overhead. Costs are calculated in real-time and stored per mission.

### Task 1.1: Database Schema for Cost Tracking

**File**: `migrations/20250108_cost_tracking.sql`

```sql
-- Cost tracking tables
CREATE TABLE mission_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_id UUID NOT NULL REFERENCES missions(id) ON DELETE CASCADE,

    -- LLM Costs
    llm_input_tokens BIGINT NOT NULL DEFAULT 0,
    llm_output_tokens BIGINT NOT NULL DEFAULT 0,
    llm_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    -- Tool Costs
    tool_execution_count INTEGER NOT NULL DEFAULT 0,
    tool_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    -- Infrastructure Costs
    compute_time_seconds INTEGER NOT NULL DEFAULT 0,
    compute_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    -- Total
    total_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,
    markup_percentage DECIMAL(5, 2) NOT NULL DEFAULT 30.0,
    final_price_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    -- Metadata
    cost_breakdown JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_mission_costs_mission_id ON mission_costs(mission_id);
CREATE INDEX idx_mission_costs_total_cost ON mission_costs(total_cost_usd);

-- Cost tracking per agent
CREATE TABLE agent_costs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mission_cost_id UUID NOT NULL REFERENCES mission_costs(id) ON DELETE CASCADE,
    agent_id UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,

    -- LLM usage
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    llm_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    -- Tool usage
    tools_used JSONB NOT NULL DEFAULT '[]',
    tool_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    total_cost_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_agent_costs_mission ON agent_costs(mission_cost_id);
CREATE INDEX idx_agent_costs_agent ON agent_costs(agent_id);

-- LLM pricing configuration
CREATE TABLE llm_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    model VARCHAR(100) NOT NULL,

    input_token_price_per_million DECIMAL(10, 4) NOT NULL,
    output_token_price_per_million DECIMAL(10, 4) NOT NULL,

    effective_from TIMESTAMP WITH TIME ZONE NOT NULL,
    effective_until TIMESTAMP WITH TIME ZONE,

    UNIQUE(provider, model, effective_from)
);

-- Insert current pricing
INSERT INTO llm_pricing (provider, model, input_token_price_per_million, output_token_price_per_million, effective_from)
VALUES
    ('anthropic', 'claude-3-5-sonnet-20241022', 3.00, 15.00, '2024-11-01'),
    ('anthropic', 'claude-3-haiku-20240307', 0.25, 1.25, '2024-11-01'),
    ('openai', 'gpt-4-turbo-2024-04-09', 10.00, 30.00, '2024-11-01'),
    ('openai', 'gpt-4o-mini', 0.15, 0.60, '2024-11-01');

-- Tool pricing configuration
CREATE TABLE tool_pricing (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tool_name VARCHAR(100) NOT NULL,
    pricing_type VARCHAR(20) NOT NULL CHECK (pricing_type IN ('per_call', 'per_minute', 'per_mb')),
    price_usd DECIMAL(10, 4) NOT NULL,

    effective_from TIMESTAMP WITH TIME ZONE NOT NULL,
    effective_until TIMESTAMP WITH TIME ZONE,

    UNIQUE(tool_name, effective_from)
);

-- Insert tool pricing
INSERT INTO tool_pricing (tool_name, pricing_type, price_usd, effective_from)
VALUES
    ('web_search', 'per_call', 0.01, '2024-11-01'),
    ('code_execution', 'per_minute', 0.05, '2024-11-01'),
    ('file_storage', 'per_mb', 0.001, '2024-11-01'),
    ('api_call', 'per_call', 0.005, '2024-11-01');

-- Billing transactions
CREATE TABLE billing_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id),
    mission_id UUID REFERENCES missions(id),

    transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('charge', 'refund', 'credit', 'discount')),
    amount_usd DECIMAL(10, 4) NOT NULL,

    stripe_payment_intent_id VARCHAR(255),
    stripe_charge_id VARCHAR(255),

    status VARCHAR(20) NOT NULL CHECK (status IN ('pending', 'succeeded', 'failed', 'refunded')),

    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_billing_transactions_user ON billing_transactions(user_id);
CREATE INDEX idx_billing_transactions_mission ON billing_transactions(mission_id);
CREATE INDEX idx_billing_transactions_status ON billing_transactions(status);

-- User wallet/credits
CREATE TABLE user_wallets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL UNIQUE REFERENCES users(id),

    balance_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,
    reserved_usd DECIMAL(10, 4) NOT NULL DEFAULT 0.0,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_user_wallets_user ON user_wallets(user_id);
```

**Acceptance Criteria**:
- [ ] All cost tracking tables created
- [ ] Pricing configuration tables populated
- [ ] Indexes optimized for queries
- [ ] Foreign key constraints enforced

### Task 1.2: Cost Calculation Service

**File**: `backend/src/billing/cost_calculator.rs`

```rust
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub llm_cost: Decimal,
    pub tool_cost: Decimal,
    pub compute_cost: Decimal,
    pub total_cost: Decimal,
    pub markup_percentage: Decimal,
    pub final_price: Decimal,
}

#[derive(Debug, Clone)]
pub struct LLMUsage {
    pub provider: String,
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

#[derive(Debug, Clone)]
pub struct ToolUsage {
    pub tool_name: String,
    pub call_count: i32,
    pub duration_seconds: Option<i32>,
    pub data_size_mb: Option<f64>,
}

pub struct CostCalculator {
    pool: PgPool,
}

impl CostCalculator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Calculate total mission cost
    pub async fn calculate_mission_cost(
        &self,
        mission_id: Uuid,
        llm_usage: Vec<LLMUsage>,
        tool_usage: Vec<ToolUsage>,
        compute_seconds: i32,
    ) -> Result<CostBreakdown, Box<dyn std::error::Error>> {
        // Calculate LLM costs
        let llm_cost = self.calculate_llm_cost(&llm_usage).await?;

        // Calculate tool costs
        let tool_cost = self.calculate_tool_cost(&tool_usage).await?;

        // Calculate compute costs (example: $0.10 per hour)
        let compute_cost = Decimal::from(compute_seconds) * Decimal::new(10, 5); // $0.0001/sec

        // Total base cost
        let total_cost = llm_cost + tool_cost + compute_cost;

        // Apply markup (30% default)
        let markup_percentage = Decimal::new(30, 0);
        let markup_multiplier = Decimal::new(1, 0) + (markup_percentage / Decimal::new(100, 0));
        let final_price = total_cost * markup_multiplier;

        Ok(CostBreakdown {
            llm_cost,
            tool_cost,
            compute_cost,
            total_cost,
            markup_percentage,
            final_price,
        })
    }

    /// Calculate LLM API costs based on token usage
    async fn calculate_llm_cost(
        &self,
        usage: &[LLMUsage],
    ) -> Result<Decimal, Box<dyn std::error::Error>> {
        let mut total_cost = Decimal::ZERO;

        for u in usage {
            // Fetch current pricing
            let pricing = sqlx::query!(
                r#"
                SELECT input_token_price_per_million, output_token_price_per_million
                FROM llm_pricing
                WHERE provider = $1 AND model = $2
                  AND effective_from <= NOW()
                  AND (effective_until IS NULL OR effective_until > NOW())
                ORDER BY effective_from DESC
                LIMIT 1
                "#,
                u.provider,
                u.model
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(p) = pricing {
                let input_cost = (Decimal::from(u.input_tokens) / Decimal::from(1_000_000))
                    * p.input_token_price_per_million;
                let output_cost = (Decimal::from(u.output_tokens) / Decimal::from(1_000_000))
                    * p.output_token_price_per_million;

                total_cost += input_cost + output_cost;
            }
        }

        Ok(total_cost)
    }

    /// Calculate tool execution costs
    async fn calculate_tool_cost(
        &self,
        usage: &[ToolUsage],
    ) -> Result<Decimal, Box<dyn std::error::Error>> {
        let mut total_cost = Decimal::ZERO;

        for u in usage {
            let pricing = sqlx::query!(
                r#"
                SELECT pricing_type, price_usd
                FROM tool_pricing
                WHERE tool_name = $1
                  AND effective_from <= NOW()
                  AND (effective_until IS NULL OR effective_until > NOW())
                ORDER BY effective_from DESC
                LIMIT 1
                "#,
                u.tool_name
            )
            .fetch_optional(&self.pool)
            .await?;

            if let Some(p) = pricing {
                let cost = match p.pricing_type.as_str() {
                    "per_call" => p.price_usd * Decimal::from(u.call_count),
                    "per_minute" => {
                        p.price_usd * Decimal::from(u.duration_seconds.unwrap_or(0)) / Decimal::from(60)
                    }
                    "per_mb" => {
                        p.price_usd * Decimal::try_from(u.data_size_mb.unwrap_or(0.0)).unwrap()
                    }
                    _ => Decimal::ZERO,
                };

                total_cost += cost;
            }
        }

        Ok(total_cost)
    }

    /// Store cost breakdown in database
    pub async fn save_mission_cost(
        &self,
        mission_id: Uuid,
        breakdown: &CostBreakdown,
        llm_usage: &[LLMUsage],
        tool_usage: &[ToolUsage],
        compute_seconds: i32,
    ) -> Result<Uuid, Box<dyn std::error::Error>> {
        // Calculate totals
        let total_input_tokens: i64 = llm_usage.iter().map(|u| u.input_tokens).sum();
        let total_output_tokens: i64 = llm_usage.iter().map(|u| u.output_tokens).sum();
        let total_tool_calls: i32 = tool_usage.iter().map(|u| u.call_count).sum();

        // Create detailed breakdown JSON
        let cost_breakdown = serde_json::json!({
            "llm": {
                "total_cost": breakdown.llm_cost,
                "usage": llm_usage.iter().map(|u| serde_json::json!({
                    "provider": u.provider,
                    "model": u.model,
                    "input_tokens": u.input_tokens,
                    "output_tokens": u.output_tokens,
                })).collect::<Vec<_>>()
            },
            "tools": {
                "total_cost": breakdown.tool_cost,
                "usage": tool_usage.iter().map(|u| serde_json::json!({
                    "tool": u.tool_name,
                    "calls": u.call_count,
                    "duration_seconds": u.duration_seconds,
                    "data_size_mb": u.data_size_mb,
                })).collect::<Vec<_>>()
            },
            "compute": {
                "total_cost": breakdown.compute_cost,
                "duration_seconds": compute_seconds,
            }
        });

        let cost_id = sqlx::query_scalar!(
            r#"
            INSERT INTO mission_costs (
                mission_id,
                llm_input_tokens,
                llm_output_tokens,
                llm_cost_usd,
                tool_execution_count,
                tool_cost_usd,
                compute_time_seconds,
                compute_cost_usd,
                total_cost_usd,
                markup_percentage,
                final_price_usd,
                cost_breakdown
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id
            "#,
            mission_id,
            total_input_tokens,
            total_output_tokens,
            breakdown.llm_cost,
            total_tool_calls,
            breakdown.tool_cost,
            compute_seconds,
            breakdown.compute_cost,
            breakdown.total_cost,
            breakdown.markup_percentage,
            breakdown.final_price,
            cost_breakdown
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(cost_id)
    }

    /// Estimate mission cost upfront
    pub async fn estimate_cost(
        &self,
        estimated_agents: i32,
        estimated_tasks: i32,
        complexity: &str, // "simple", "medium", "complex"
    ) -> Result<Decimal, Box<dyn std::error::Error>> {
        // Estimation heuristics based on historical data
        let base_tokens_per_agent = match complexity {
            "simple" => 5_000,
            "medium" => 15_000,
            "complex" => 40_000,
            _ => 10_000,
        };

        let total_tokens = base_tokens_per_agent * estimated_agents;
        let input_tokens = (total_tokens as f64 * 0.6) as i64;
        let output_tokens = (total_tokens as f64 * 0.4) as i64;

        let llm_usage = vec![LLMUsage {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            input_tokens,
            output_tokens,
        }];

        let tool_usage = vec![
            ToolUsage {
                tool_name: "web_search".to_string(),
                call_count: estimated_tasks * 2,
                duration_seconds: None,
                data_size_mb: None,
            },
            ToolUsage {
                tool_name: "code_execution".to_string(),
                call_count: estimated_tasks,
                duration_seconds: Some(estimated_tasks * 30),
                data_size_mb: None,
            },
        ];

        let compute_seconds = estimated_agents * estimated_tasks * 60;

        let breakdown = self.calculate_mission_cost(
            Uuid::new_v4(), // Dummy ID for estimation
            llm_usage,
            tool_usage,
            compute_seconds,
        ).await?;

        Ok(breakdown.final_price)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_cost_calculation() {
        // Test cost calculation for Claude Sonnet
        let usage = vec![LLMUsage {
            provider: "anthropic".to_string(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            input_tokens: 1_000_000,
            output_tokens: 500_000,
        }];

        // Expected: (1M * $3) + (0.5M * $15) = $3 + $7.50 = $10.50
        // This test requires database access - mock or use test DB
    }

    #[tokio::test]
    async fn test_cost_estimation() {
        // Test upfront cost estimation
        // Verify estimates are reasonable and within expected ranges
    }
}
```

**Acceptance Criteria**:
- [ ] Cost calculation accurate for all LLM models
- [ ] Tool costs calculated based on pricing type
- [ ] Compute costs included
- [ ] 30% markup applied correctly
- [ ] Estimation function provides reasonable predictions

---

## Epic 2: Real-time Budget Monitoring

### Overview

Implement real-time budget tracking during mission execution with alerts and automatic stop mechanisms when budgets are exceeded.

### Task 2.1: Budget Enforcement Service

**File**: `backend/src/billing/budget_monitor.rs`

```rust
use rust_decimal::Decimal;
use sqlx::PgPool;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BudgetLimit {
    pub mission_id: Uuid,
    pub max_cost_usd: Decimal,
    pub current_cost_usd: Decimal,
    pub alert_threshold: Decimal, // Alert at 80% by default
}

pub struct BudgetMonitor {
    pool: PgPool,
    active_budgets: Arc<RwLock<HashMap<Uuid, BudgetLimit>>>,
}

impl BudgetMonitor {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            active_budgets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set budget limit for a mission
    pub async fn set_budget_limit(
        &self,
        mission_id: Uuid,
        max_cost_usd: Decimal,
        alert_threshold_percent: Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let budget = BudgetLimit {
            mission_id,
            max_cost_usd,
            current_cost_usd: Decimal::ZERO,
            alert_threshold: max_cost_usd * alert_threshold_percent / Decimal::from(100),
        };

        let mut budgets = self.active_budgets.write().await;
        budgets.insert(mission_id, budget);

        Ok(())
    }

    /// Update current cost and check against budget
    pub async fn update_cost(
        &self,
        mission_id: Uuid,
        additional_cost: Decimal,
    ) -> Result<BudgetStatus, Box<dyn std::error::Error>> {
        let mut budgets = self.active_budgets.write().await;

        if let Some(budget) = budgets.get_mut(&mission_id) {
            budget.current_cost_usd += additional_cost;

            // Check budget status
            if budget.current_cost_usd >= budget.max_cost_usd {
                return Ok(BudgetStatus::Exceeded {
                    current: budget.current_cost_usd,
                    limit: budget.max_cost_usd,
                });
            } else if budget.current_cost_usd >= budget.alert_threshold {
                return Ok(BudgetStatus::Warning {
                    current: budget.current_cost_usd,
                    limit: budget.max_cost_usd,
                    percentage: (budget.current_cost_usd / budget.max_cost_usd * Decimal::from(100))
                        .round_dp(2),
                });
            } else {
                return Ok(BudgetStatus::Ok {
                    current: budget.current_cost_usd,
                    limit: budget.max_cost_usd,
                });
            }
        }

        Err("Budget not found for mission".into())
    }

    /// Get current budget status
    pub async fn get_budget_status(&self, mission_id: Uuid) -> Option<BudgetLimit> {
        let budgets = self.active_budgets.read().await;
        budgets.get(&mission_id).cloned()
    }

    /// Remove budget tracking (mission completed)
    pub async fn remove_budget(&self, mission_id: Uuid) {
        let mut budgets = self.active_budgets.write().await;
        budgets.remove(&mission_id);
    }
}

#[derive(Debug, Clone)]
pub enum BudgetStatus {
    Ok {
        current: Decimal,
        limit: Decimal,
    },
    Warning {
        current: Decimal,
        limit: Decimal,
        percentage: Decimal,
    },
    Exceeded {
        current: Decimal,
        limit: Decimal,
    },
}

impl BudgetStatus {
    pub fn should_stop(&self) -> bool {
        matches!(self, BudgetStatus::Exceeded { .. })
    }

    pub fn should_alert(&self) -> bool {
        matches!(self, BudgetStatus::Warning { .. } | BudgetStatus::Exceeded { .. })
    }
}
```

**Acceptance Criteria**:
- [ ] Budget limits tracked per mission
- [ ] Real-time cost updates
- [ ] Alerts at configurable thresholds
- [ ] Automatic stop when exceeded
- [ ] Thread-safe budget tracking

---

## Epic 3: Billing API Implementation

### Overview

Create REST API endpoints for billing operations: cost queries, payment processing, refunds, and transaction history.

### Task 3.1: Billing API Routes

**File**: `backend/src/api/routes/billing.rs`

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;

use crate::{
    billing::{cost_calculator::CostCalculator, payment_processor::PaymentProcessor},
    middleware::auth::AuthUser,
};

#[derive(Debug, Serialize)]
pub struct CostEstimateResponse {
    pub estimated_cost_usd: Decimal,
    pub currency: String,
    pub breakdown: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CostEstimateRequest {
    pub estimated_agents: i32,
    pub estimated_tasks: i32,
    pub complexity: String,
}

#[derive(Debug, Serialize)]
pub struct MissionCostResponse {
    pub mission_id: Uuid,
    pub total_cost_usd: Decimal,
    pub final_price_usd: Decimal,
    pub breakdown: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct TransactionHistoryResponse {
    pub transactions: Vec<TransactionRecord>,
    pub total_spent_usd: Decimal,
}

#[derive(Debug, Serialize)]
pub struct TransactionRecord {
    pub id: Uuid,
    pub mission_id: Option<Uuid>,
    pub transaction_type: String,
    pub amount_usd: Decimal,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /api/billing/estimate - Estimate mission cost
pub async fn estimate_cost(
    State(calculator): State<CostCalculator>,
    Json(req): Json<CostEstimateRequest>,
) -> Result<Json<CostEstimateResponse>, StatusCode> {
    let estimated_cost = calculator
        .estimate_cost(req.estimated_agents, req.estimated_tasks, &req.complexity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(CostEstimateResponse {
        estimated_cost_usd: estimated_cost,
        currency: "USD".to_string(),
        breakdown: serde_json::json!({
            "agents": req.estimated_agents,
            "tasks": req.estimated_tasks,
            "complexity": req.complexity,
        }),
    }))
}

/// GET /api/billing/missions/:mission_id/cost - Get mission cost
pub async fn get_mission_cost(
    State(pool): State<sqlx::PgPool>,
    Path(mission_id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> Result<Json<MissionCostResponse>, StatusCode> {
    // Verify user owns this mission
    let mission = sqlx::query!(
        "SELECT user_id FROM missions WHERE id = $1",
        mission_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    if mission.user_id != user.id {
        return Err(StatusCode::FORBIDDEN);
    }

    let cost = sqlx::query!(
        r#"
        SELECT
            mission_id,
            total_cost_usd,
            final_price_usd,
            cost_breakdown,
            created_at
        FROM mission_costs
        WHERE mission_id = $1
        "#,
        mission_id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(MissionCostResponse {
        mission_id: cost.mission_id,
        total_cost_usd: cost.total_cost_usd,
        final_price_usd: cost.final_price_usd,
        breakdown: cost.cost_breakdown,
        created_at: cost.created_at.and_utc(),
    }))
}

/// GET /api/billing/transactions - Get transaction history
pub async fn get_transaction_history(
    State(pool): State<sqlx::PgPool>,
    AuthUser(user): AuthUser,
) -> Result<Json<TransactionHistoryResponse>, StatusCode> {
    let transactions = sqlx::query_as!(
        TransactionRecord,
        r#"
        SELECT
            id,
            mission_id,
            transaction_type,
            amount_usd,
            status,
            created_at
        FROM billing_transactions
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        user.id
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total_spent: Decimal = transactions
        .iter()
        .filter(|t| t.transaction_type == "charge" && t.status == "succeeded")
        .map(|t| t.amount_usd)
        .sum();

    Ok(Json(TransactionHistoryResponse {
        transactions,
        total_spent_usd: total_spent,
    }))
}

/// POST /api/billing/missions/:mission_id/pay - Process payment for mission
pub async fn process_payment(
    State(processor): State<PaymentProcessor>,
    Path(mission_id): Path<Uuid>,
    AuthUser(user): AuthUser,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = processor
        .charge_for_mission(user.id, mission_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(result))
}

pub fn billing_routes() -> Router {
    Router::new()
        .route("/estimate", post(estimate_cost))
        .route("/missions/:mission_id/cost", get(get_mission_cost))
        .route("/missions/:mission_id/pay", post(process_payment))
        .route("/transactions", get(get_transaction_history))
}
```

**Acceptance Criteria**:
- [ ] All billing endpoints implemented
- [ ] Authentication required
- [ ] Authorization checks (user owns resource)
- [ ] Error handling with proper status codes

---

## Epic 4: Invoice Generation

### Overview

Automatically generate PDF invoices for completed missions with detailed cost breakdowns.

### Task 4.1: Invoice Generator

**File**: `backend/src/billing/invoice_generator.rs`

```rust
use printpdf::*;
use rust_decimal::Decimal;
use uuid::Uuid;

pub struct Invoice {
    pub invoice_number: String,
    pub mission_id: Uuid,
    pub user_email: String,
    pub items: Vec<InvoiceItem>,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub issued_date: chrono::DateTime<chrono::Utc>,
}

pub struct InvoiceItem {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub total: Decimal,
}

pub struct InvoiceGenerator;

impl InvoiceGenerator {
    pub fn generate_pdf(invoice: &Invoice) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let (doc, page1, layer1) = PdfDocument::new("Ghost Pirates Invoice", Mm(210.0), Mm(297.0), "Layer 1");
        let current_layer = doc.get_page(page1).get_layer(layer1);

        // Add company header
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        current_layer.use_text("Ghost Pirates", 24.0, Mm(20.0), Mm(270.0), &font);
        current_layer.use_text("AI Team Orchestration Platform", 10.0, Mm(20.0), Mm(265.0), &font);

        // Add invoice details
        current_layer.use_text(
            &format!("Invoice #: {}", invoice.invoice_number),
            12.0,
            Mm(20.0),
            Mm(250.0),
            &font,
        );
        current_layer.use_text(
            &format!("Date: {}", invoice.issued_date.format("%Y-%m-%d")),
            12.0,
            Mm(20.0),
            Mm(245.0),
            &font,
        );
        current_layer.use_text(
            &format!("Mission ID: {}", invoice.mission_id),
            10.0,
            Mm(20.0),
            Mm(240.0),
            &font,
        );

        // Add customer info
        current_layer.use_text("Bill To:", 12.0, Mm(20.0), Mm(225.0), &font);
        current_layer.use_text(&invoice.user_email, 10.0, Mm(20.0), Mm(220.0), &font);

        // Add line items
        let mut y_pos = 200.0;
        current_layer.use_text("Description", 12.0, Mm(20.0), Mm(y_pos), &font);
        current_layer.use_text("Qty", 12.0, Mm(120.0), Mm(y_pos), &font);
        current_layer.use_text("Unit Price", 12.0, Mm(140.0), Mm(y_pos), &font);
        current_layer.use_text("Total", 12.0, Mm(170.0), Mm(y_pos), &font);
        y_pos -= 10.0;

        for item in &invoice.items {
            current_layer.use_text(&item.description, 10.0, Mm(20.0), Mm(y_pos), &font);
            current_layer.use_text(&item.quantity.to_string(), 10.0, Mm(120.0), Mm(y_pos), &font);
            current_layer.use_text(&format!("${}", item.unit_price), 10.0, Mm(140.0), Mm(y_pos), &font);
            current_layer.use_text(&format!("${}", item.total), 10.0, Mm(170.0), Mm(y_pos), &font);
            y_pos -= 7.0;
        }

        // Add totals
        y_pos -= 10.0;
        current_layer.use_text("Subtotal:", 12.0, Mm(140.0), Mm(y_pos), &font);
        current_layer.use_text(&format!("${}", invoice.subtotal), 12.0, Mm(170.0), Mm(y_pos), &font);

        y_pos -= 7.0;
        current_layer.use_text("Tax:", 12.0, Mm(140.0), Mm(y_pos), &font);
        current_layer.use_text(&format!("${}", invoice.tax), 12.0, Mm(170.0), Mm(y_pos), &font);

        y_pos -= 10.0;
        current_layer.use_text("Total:", 14.0, Mm(140.0), Mm(y_pos), &font);
        current_layer.use_text(&format!("${}", invoice.total), 14.0, Mm(170.0), Mm(y_pos), &font);

        // Save to bytes
        let bytes = doc.save_to_bytes()?;
        Ok(bytes)
    }
}
```

**Acceptance Criteria**:
- [ ] PDF invoices generated correctly
- [ ] All cost details included
- [ ] Professional formatting
- [ ] Stored in Azure Blob Storage

---

## Epic 5: Payment Integration (Stripe)

### Overview

Integrate Stripe for payment processing, including payment intents, webhooks, and refund handling.

### Task 5.1: Stripe Integration

**File**: `backend/src/billing/payment_processor.rs`

```rust
use stripe::{
    Client, CreatePaymentIntent, CreateRefund, Currency, PaymentIntent, PaymentIntentCaptureMethod,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PaymentProcessor {
    stripe_client: Client,
    pool: PgPool,
}

impl PaymentProcessor {
    pub fn new(stripe_secret_key: String, pool: PgPool) -> Self {
        Self {
            stripe_client: Client::new(stripe_secret_key),
            pool,
        }
    }

    /// Create payment intent for mission
    pub async fn charge_for_mission(
        &self,
        user_id: Uuid,
        mission_id: Uuid,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        // Get mission cost
        let cost = sqlx::query!(
            "SELECT final_price_usd FROM mission_costs WHERE mission_id = $1",
            mission_id
        )
        .fetch_one(&self.pool)
        .await?;

        let amount_cents = (cost.final_price_usd * Decimal::from(100)).to_u64().unwrap();

        // Create payment intent
        let mut create_intent = CreatePaymentIntent::new(amount_cents, Currency::USD);
        create_intent.metadata = Some(
            [
                ("user_id".to_string(), user_id.to_string()),
                ("mission_id".to_string(), mission_id.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        );
        create_intent.capture_method = Some(PaymentIntentCaptureMethod::Automatic);

        let intent = PaymentIntent::create(&self.stripe_client, create_intent).await?;

        // Record transaction
        sqlx::query!(
            r#"
            INSERT INTO billing_transactions (user_id, mission_id, transaction_type, amount_usd, stripe_payment_intent_id, status)
            VALUES ($1, $2, 'charge', $3, $4, 'pending')
            "#,
            user_id,
            mission_id,
            cost.final_price_usd,
            intent.id.to_string()
        )
        .execute(&self.pool)
        .await?;

        Ok(serde_json::json!({
            "client_secret": intent.client_secret,
            "amount": amount_cents,
            "currency": "usd"
        }))
    }

    /// Issue refund
    pub async fn refund_mission(
        &self,
        mission_id: Uuid,
        reason: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get original payment
        let transaction = sqlx::query!(
            r#"
            SELECT id, user_id, amount_usd, stripe_charge_id
            FROM billing_transactions
            WHERE mission_id = $1 AND transaction_type = 'charge' AND status = 'succeeded'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            mission_id
        )
        .fetch_one(&self.pool)
        .await?;

        if let Some(charge_id) = transaction.stripe_charge_id {
            // Process Stripe refund
            let mut refund_params = CreateRefund::new();
            refund_params.charge = Some(charge_id.parse()?);
            refund_params.reason = Some(stripe::RefundReason::RequestedByCustomer);

            stripe::Refund::create(&self.stripe_client, refund_params).await?;

            // Record refund transaction
            sqlx::query!(
                r#"
                INSERT INTO billing_transactions (user_id, mission_id, transaction_type, amount_usd, status, metadata)
                VALUES ($1, $2, 'refund', $3, 'succeeded', $4)
                "#,
                transaction.user_id,
                mission_id,
                transaction.amount_usd,
                serde_json::json!({"reason": reason})
            )
            .execute(&self.pool)
            .await?;

            // Update original transaction
            sqlx::query!(
                "UPDATE billing_transactions SET status = 'refunded' WHERE id = $1",
                transaction.id
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
```

**Acceptance Criteria**:
- [ ] Stripe client configured
- [ ] Payment intents created correctly
- [ ] Webhook handling for payment confirmation
- [ ] Refund processing implemented
- [ ] All transactions logged in database

---

## Epic 6: Cost Breakdown Dashboard

### Overview

Create frontend dashboard showing detailed cost breakdowns per team, per agent, and per mission with visualizations.

### Task 6.1: Cost Dashboard Component

**File**: `frontend/src/components/billing/CostDashboard.tsx`

```typescript
import React, { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import { BarChart, Bar, PieChart, Pie, LineChart, Line, XAxis, YAxis, Tooltip, Legend } from 'recharts';

interface CostBreakdown {
  mission_id: string;
  total_cost_usd: number;
  final_price_usd: number;
  breakdown: {
    llm: { total_cost: number; usage: any[] };
    tools: { total_cost: number; usage: any[] };
    compute: { total_cost: number; duration_seconds: number };
  };
}

export const CostDashboard: React.FC = () => {
  const { data: missions } = useQuery(['missions-costs'], async () => {
    const res = await fetch('/api/billing/missions');
    return res.json();
  });

  const costByCategory = missions
    ? [
        { name: 'LLM', value: missions.reduce((sum, m) => sum + m.breakdown.llm.total_cost, 0) },
        { name: 'Tools', value: missions.reduce((sum, m) => sum + m.breakdown.tools.total_cost, 0) },
        { name: 'Compute', value: missions.reduce((sum, m) => sum + m.breakdown.compute.total_cost, 0) },
      ]
    : [];

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Cost Analytics</h1>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 mb-8">
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Total Spent</h3>
          <p className="text-3xl font-bold text-gray-900">
            ${missions?.reduce((sum, m) => sum + m.final_price_usd, 0).toFixed(2)}
          </p>
        </div>

        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Average per Mission</h3>
          <p className="text-3xl font-bold text-gray-900">
            ${(missions?.reduce((sum, m) => sum + m.final_price_usd, 0) / (missions?.length || 1)).toFixed(2)}
          </p>
        </div>

        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-sm font-medium text-gray-500">Total Missions</h3>
          <p className="text-3xl font-bold text-gray-900">{missions?.length || 0}</p>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-lg font-semibold mb-4">Cost by Category</h3>
          <PieChart width={400} height={300}>
            <Pie data={costByCategory} dataKey="value" nameKey="name" cx="50%" cy="50%" outerRadius={100} fill="#8884d8" label />
            <Tooltip />
          </PieChart>
        </div>

        <div className="bg-white p-6 rounded-lg shadow">
          <h3 className="text-lg font-semibold mb-4">Cost per Mission</h3>
          <BarChart width={400} height={300} data={missions}>
            <XAxis dataKey="mission_id" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Bar dataKey="final_price_usd" fill="#3b82f6" />
          </BarChart>
        </div>
      </div>
    </div>
  );
};
```

**Acceptance Criteria**:
- [ ] Cost dashboard displays all metrics
- [ ] Charts render correctly
- [ ] Real-time updates via React Query
- [ ] Responsive design

---

## Epic 7: Pricing Tiers

### Overview

Implement three pricing tiers with different features, limits, and volume discounts.

### Task 7.1: Pricing Tier Configuration

**File**: `migrations/20250108_pricing_tiers.sql`

```sql
CREATE TABLE pricing_tiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier_name VARCHAR(50) NOT NULL UNIQUE,

    -- Mission limits
    max_agents_per_mission INTEGER NOT NULL,
    max_tasks_per_mission INTEGER NOT NULL,
    max_concurrent_missions INTEGER NOT NULL,

    -- Pricing
    base_price_usd DECIMAL(10, 4) NOT NULL,
    price_per_agent_usd DECIMAL(10, 4) NOT NULL,
    price_per_task_usd DECIMAL(10, 4) NOT NULL,
    volume_discount_percentage DECIMAL(5, 2) DEFAULT 0.0,

    -- Features
    features JSONB NOT NULL DEFAULT '{}',

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO pricing_tiers (tier_name, max_agents_per_mission, max_tasks_per_mission, max_concurrent_missions, base_price_usd, price_per_agent_usd, price_per_task_usd, volume_discount_percentage, features)
VALUES
    ('starter', 4, 10, 1, 5.00, 2.00, 0.50, 0.0, '{"support": "community", "sla": "none", "priority": "low"}'),
    ('pro', 8, 50, 5, 25.00, 5.00, 1.00, 10.0, '{"support": "email", "sla": "99%", "priority": "medium", "advanced_tools": true}'),
    ('enterprise', 20, 200, 20, 100.00, 10.00, 2.00, 20.0, '{"support": "dedicated", "sla": "99.9%", "priority": "high", "custom_agents": true, "api_access": true}');

-- User tier assignments
ALTER TABLE users ADD COLUMN pricing_tier_id UUID REFERENCES pricing_tiers(id);
UPDATE users SET pricing_tier_id = (SELECT id FROM pricing_tiers WHERE tier_name = 'starter');
```

**Acceptance Criteria**:
- [ ] Three pricing tiers configured
- [ ] Limits enforced per tier
- [ ] Volume discounts applied automatically
- [ ] Users can upgrade/downgrade tiers

---

## Testing & Validation

### Cost Calculation Tests

```rust
#[tokio::test]
async fn test_accurate_cost_calculation() {
    // Verify costs match expected values
    // Test all LLM models
    // Test all tool types
    // Verify markup applied correctly
}

#[tokio::test]
async fn test_budget_enforcement() {
    // Verify budget stops mission when exceeded
    // Test alert thresholds
}
```

---

## Success Metrics

| Metric | Target | Status |
|--------|--------|--------|
| Cost Calculation Accuracy | ±5% | ⏳ |
| Payment Success Rate | >99% | ⏳ |
| Invoice Generation Time | <5s | ⏳ |
| Budget Alert Latency | <1s | ⏳ |
| Pricing Tier Adoption | 60% Pro+ | ⏳ |

---

## Next Steps

1. **Review [18-success-metrics.md](./18-success-metrics.md)** for launch criteria
2. **Test payment flows** in Stripe test mode
3. **Validate cost calculations** with real missions
4. **Train support team** on refund policies

---

**Ghost Pirates Pricing: Fair, transparent, outcome-based billing.**
