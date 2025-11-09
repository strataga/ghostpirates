# Monitoring & Observability Implementation Plan

**Version**: 1.0
**Last Updated**: November 8, 2025
**Dependencies**: All phases (01-15), infrastructure (02), security (13)
**Estimated Duration**: 3-4 weeks
**Status**: Ready for Implementation

---

## Executive Summary

This document outlines the comprehensive monitoring and observability strategy for Ghost Pirates. A production-grade multi-agent AI platform requires deep visibility into system health, agent performance, cost tracking, and user experience metrics.

**Key Components**:
- Prometheus metrics collection (application + infrastructure)
- Azure Application Insights integration
- OpenTelemetry distributed tracing
- Structured JSON logging with tracing-subscriber
- Grafana dashboards for visualization
- PagerDuty alerting for incident response
- Azure Log Analytics for centralized log aggregation

**Success Criteria**:
- Mean Time to Detection (MTTD) < 2 minutes
- Mean Time to Resolution (MTTR) < 30 minutes
- 99.9% metrics collection reliability
- <100ms overhead from instrumentation
- Complete request tracing for 100% of traffic

---

## Table of Contents

1. [Epic 1: Prometheus Metrics Setup](#epic-1-prometheus-metrics-setup)
2. [Epic 2: Application Insights Integration](#epic-2-application-insights-integration)
3. [Epic 3: Distributed Tracing with OpenTelemetry](#epic-3-distributed-tracing-with-opentelemetry)
4. [Epic 4: Structured Logging](#epic-4-structured-logging)
5. [Epic 5: Grafana Dashboards](#epic-5-grafana-dashboards)
6. [Epic 6: Alerting Rules](#epic-6-alerting-rules)
7. [Epic 7: Log Aggregation](#epic-7-log-aggregation)

---

## Epic 1: Prometheus Metrics Setup

### Overview

Implement comprehensive Prometheus metrics collection for all critical system components. Metrics cover API performance, agent execution, database queries, external API calls, and cost tracking.

### Task 1.1: Install Prometheus Dependencies

**File**: `Cargo.toml`

```toml
[dependencies]
# Metrics
prometheus = "0.13"
metrics = "0.21"
metrics-exporter-prometheus = "0.13"
lazy_static = "1.4"

# For async metrics
tokio-metrics = "0.3"
```

**Acceptance Criteria**:
- [ ] Dependencies added to Cargo.toml
- [ ] Project compiles successfully
- [ ] No version conflicts with existing dependencies

### Task 1.2: Create Metrics Registry Module

**File**: `backend/src/observability/metrics.rs`

```rust
use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec,
    CounterVec, GaugeVec, HistogramVec, Registry,
};
use std::time::Instant;

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    // HTTP Request Metrics
    pub static ref HTTP_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "http_requests_total",
        "Total HTTP requests by method, path, and status",
        &["method", "path", "status"]
    ).unwrap();

    pub static ref HTTP_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds",
        "HTTP request latencies in seconds",
        &["method", "path"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
    ).unwrap();

    // Agent Execution Metrics
    pub static ref AGENT_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "agent_operations_total",
        "Total agent operations by type and status",
        &["agent_type", "operation", "status"]
    ).unwrap();

    pub static ref AGENT_EXECUTION_DURATION: HistogramVec = register_histogram_vec!(
        "agent_execution_duration_seconds",
        "Agent execution time in seconds",
        &["agent_type", "operation"],
        vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 120.0, 300.0]
    ).unwrap();

    pub static ref ACTIVE_TEAMS: GaugeVec = register_gauge_vec!(
        "active_teams_total",
        "Number of currently active teams",
        &["status"]
    ).unwrap();

    pub static ref ACTIVE_AGENTS: GaugeVec = register_gauge_vec!(
        "active_agents_total",
        "Number of currently active agents",
        &["agent_type"]
    ).unwrap();

    // Task Execution Metrics
    pub static ref TASK_OPERATIONS: CounterVec = register_counter_vec!(
        "task_operations_total",
        "Total task operations by status",
        &["operation", "status"]
    ).unwrap();

    pub static ref TASK_QUEUE_SIZE: GaugeVec = register_gauge_vec!(
        "task_queue_size",
        "Number of tasks in queue by priority",
        &["priority"]
    ).unwrap();

    // LLM API Metrics
    pub static ref LLM_API_CALLS: CounterVec = register_counter_vec!(
        "llm_api_calls_total",
        "Total LLM API calls by provider and model",
        &["provider", "model", "status"]
    ).unwrap();

    pub static ref LLM_API_DURATION: HistogramVec = register_histogram_vec!(
        "llm_api_duration_seconds",
        "LLM API call duration",
        &["provider", "model"],
        vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0, 30.0, 60.0]
    ).unwrap();

    pub static ref LLM_TOKENS_USED: CounterVec = register_counter_vec!(
        "llm_tokens_used_total",
        "Total tokens consumed by provider, model, and type",
        &["provider", "model", "token_type"]
    ).unwrap();

    pub static ref LLM_COST_USD: CounterVec = register_counter_vec!(
        "llm_cost_usd_total",
        "Total LLM costs in USD by provider and model",
        &["provider", "model"]
    ).unwrap();

    // Database Metrics
    pub static ref DB_QUERIES_TOTAL: CounterVec = register_counter_vec!(
        "db_queries_total",
        "Total database queries by operation and status",
        &["operation", "status"]
    ).unwrap();

    pub static ref DB_QUERY_DURATION: HistogramVec = register_histogram_vec!(
        "db_query_duration_seconds",
        "Database query duration",
        &["operation"],
        vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]
    ).unwrap();

    pub static ref DB_POOL_CONNECTIONS: GaugeVec = register_gauge_vec!(
        "db_pool_connections",
        "Database connection pool status",
        &["state"]
    ).unwrap();

    // Cache Metrics
    pub static ref CACHE_OPERATIONS: CounterVec = register_counter_vec!(
        "cache_operations_total",
        "Total cache operations by type and result",
        &["operation", "result"]
    ).unwrap();

    pub static ref CACHE_HIT_RATIO: GaugeVec = register_gauge_vec!(
        "cache_hit_ratio",
        "Cache hit ratio by cache type",
        &["cache_type"]
    ).unwrap();

    // Error Metrics
    pub static ref ERRORS_TOTAL: CounterVec = register_counter_vec!(
        "errors_total",
        "Total errors by component and error type",
        &["component", "error_type"]
    ).unwrap();

    pub static ref ERROR_RECOVERY_OPERATIONS: CounterVec = register_counter_vec!(
        "error_recovery_operations_total",
        "Error recovery operations by type and result",
        &["recovery_type", "result"]
    ).unwrap();

    // Business Metrics
    pub static ref MISSIONS_COMPLETED: CounterVec = register_counter_vec!(
        "missions_completed_total",
        "Total missions completed by outcome",
        &["outcome"]
    ).unwrap();

    pub static ref MISSION_DURATION: HistogramVec = register_histogram_vec!(
        "mission_duration_seconds",
        "Mission completion time",
        &["team_size"],
        vec![60.0, 300.0, 600.0, 1800.0, 3600.0, 7200.0]
    ).unwrap();

    pub static ref REVENUE_USD: CounterVec = register_counter_vec!(
        "revenue_usd_total",
        "Total revenue in USD by pricing tier",
        &["pricing_tier"]
    ).unwrap();
}

/// Middleware function to track HTTP request metrics
pub async fn track_http_request<F, T>(
    method: &str,
    path: &str,
    handler: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: std::future::Future<Output = Result<T, Box<dyn std::error::Error>>>,
{
    let start = Instant::now();
    let result = handler.await;
    let duration = start.elapsed().as_secs_f64();

    let status = if result.is_ok() { "success" } else { "error" };
    HTTP_REQUESTS_TOTAL.with_label_values(&[method, path, status]).inc();
    HTTP_REQUEST_DURATION.with_label_values(&[method, path]).observe(duration);

    result
}

/// Helper to track agent operations
pub struct AgentMetricsGuard {
    agent_type: String,
    operation: String,
    start: Instant,
}

impl AgentMetricsGuard {
    pub fn new(agent_type: &str, operation: &str) -> Self {
        ACTIVE_AGENTS.with_label_values(&[agent_type]).inc();
        Self {
            agent_type: agent_type.to_string(),
            operation: operation.to_string(),
            start: Instant::now(),
        }
    }

    pub fn finish(self, status: &str) {
        let duration = self.start.elapsed().as_secs_f64();
        AGENT_OPERATIONS_TOTAL
            .with_label_values(&[&self.agent_type, &self.operation, status])
            .inc();
        AGENT_EXECUTION_DURATION
            .with_label_values(&[&self.agent_type, &self.operation])
            .observe(duration);
        ACTIVE_AGENTS.with_label_values(&[&self.agent_type]).dec();
    }
}

/// Helper to track LLM API calls with automatic cost calculation
pub struct LLMCallMetrics {
    provider: String,
    model: String,
    start: Instant,
}

impl LLMCallMetrics {
    pub fn new(provider: &str, model: &str) -> Self {
        Self {
            provider: provider.to_string(),
            model: model.to_string(),
            start: Instant::now(),
        }
    }

    pub fn finish(self, status: &str, input_tokens: u64, output_tokens: u64) {
        let duration = self.start.elapsed().as_secs_f64();

        LLM_API_CALLS
            .with_label_values(&[&self.provider, &self.model, status])
            .inc();

        LLM_API_DURATION
            .with_label_values(&[&self.provider, &self.model])
            .observe(duration);

        LLM_TOKENS_USED
            .with_label_values(&[&self.provider, &self.model, "input"])
            .inc_by(input_tokens);

        LLM_TOKENS_USED
            .with_label_values(&[&self.provider, &self.model, "output"])
            .inc_by(output_tokens);

        // Calculate cost (example pricing)
        let cost = match (self.provider.as_str(), self.model.as_str()) {
            ("anthropic", "claude-3-5-sonnet-20241022") => {
                (input_tokens as f64 * 0.000003) + (output_tokens as f64 * 0.000015)
            }
            ("openai", "gpt-4") => {
                (input_tokens as f64 * 0.00003) + (output_tokens as f64 * 0.00006)
            }
            _ => 0.0,
        };

        LLM_COST_USD
            .with_label_values(&[&self.provider, &self.model])
            .inc_by(cost);
    }
}

/// Initialize metrics registry
pub fn init_metrics() {
    // Register all metrics with the registry
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(AGENT_OPERATIONS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(AGENT_EXECUTION_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_TEAMS.clone())).unwrap();
    REGISTRY.register(Box::new(ACTIVE_AGENTS.clone())).unwrap();
    REGISTRY.register(Box::new(TASK_OPERATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(TASK_QUEUE_SIZE.clone())).unwrap();
    REGISTRY.register(Box::new(LLM_API_CALLS.clone())).unwrap();
    REGISTRY.register(Box::new(LLM_API_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(LLM_TOKENS_USED.clone())).unwrap();
    REGISTRY.register(Box::new(LLM_COST_USD.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERIES_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(DB_QUERY_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(DB_POOL_CONNECTIONS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_OPERATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(CACHE_HIT_RATIO.clone())).unwrap();
    REGISTRY.register(Box::new(ERRORS_TOTAL.clone())).unwrap();
    REGISTRY.register(Box::new(ERROR_RECOVERY_OPERATIONS.clone())).unwrap();
    REGISTRY.register(Box::new(MISSIONS_COMPLETED.clone())).unwrap();
    REGISTRY.register(Box::new(MISSION_DURATION.clone())).unwrap();
    REGISTRY.register(Box::new(REVENUE_USD.clone())).unwrap();
}
```

**Acceptance Criteria**:
- [ ] All metric types defined (counters, gauges, histograms)
- [ ] Helper structs for automatic metric tracking
- [ ] Cost calculation integrated into LLM metrics
- [ ] Registry initialization function created

### Task 1.3: Expose Prometheus Metrics Endpoint

**File**: `backend/src/api/routes/metrics.rs`

```rust
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use prometheus::Encoder;
use crate::observability::metrics::REGISTRY;

/// GET /metrics - Prometheus metrics endpoint
pub async fn metrics_handler() -> Response {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();

    match encoder.encode_to_string(&metric_families) {
        Ok(metrics) => (
            StatusCode::OK,
            [("content-type", "text/plain; version=0.0.4")],
            metrics,
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to encode metrics: {}", e),
        ).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;

    #[tokio::test]
    async fn test_metrics_endpoint() {
        let response = metrics_handler().await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_str = String::from_utf8(body.to_vec()).unwrap();

        // Verify some expected metrics are present
        assert!(body_str.contains("http_requests_total"));
        assert!(body_str.contains("agent_operations_total"));
        assert!(body_str.contains("llm_tokens_used_total"));
    }
}
```

**Register in main router**:

```rust
// backend/src/api/mod.rs
use crate::api::routes::metrics::metrics_handler;

pub fn create_router() -> Router {
    Router::new()
        // ... other routes ...
        .route("/metrics", get(metrics_handler))
}
```

**Acceptance Criteria**:
- [ ] /metrics endpoint returns Prometheus format
- [ ] Endpoint accessible without authentication (for scraping)
- [ ] Content-type header set correctly
- [ ] Tests verify metrics are exposed

### Task 1.4: Add Metrics to Agent Runtime

**File**: `backend/src/agents/runtime.rs`

```rust
use crate::observability::metrics::{AgentMetricsGuard, LLMCallMetrics};

impl AgentRuntime {
    pub async fn execute_task(&self, task: &Task) -> Result<TaskResult> {
        let _guard = AgentMetricsGuard::new(&self.agent_type, "execute_task");

        match self.execute_task_internal(task).await {
            Ok(result) => {
                _guard.finish("success");
                Ok(result)
            }
            Err(e) => {
                _guard.finish("error");
                ERRORS_TOTAL
                    .with_label_values(&["agent_runtime", &e.to_string()])
                    .inc();
                Err(e)
            }
        }
    }

    async fn call_llm(&self, prompt: &str) -> Result<LLMResponse> {
        let metrics = LLMCallMetrics::new("anthropic", "claude-3-5-sonnet-20241022");

        match self.anthropic_client.call(prompt).await {
            Ok(response) => {
                metrics.finish(
                    "success",
                    response.usage.input_tokens,
                    response.usage.output_tokens,
                );
                Ok(response)
            }
            Err(e) => {
                metrics.finish("error", 0, 0);
                Err(e)
            }
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All agent operations tracked
- [ ] LLM calls automatically tracked with costs
- [ ] Error tracking integrated
- [ ] No performance degradation (< 1ms overhead)

---

## Epic 2: Application Insights Integration

### Overview

Integrate Azure Application Insights for deep application performance monitoring, automatic dependency tracking, and correlation across distributed services.

### Task 2.1: Install Application Insights SDK

```toml
[dependencies]
# Application Insights
applicationinsights = "0.1"
```

### Task 2.2: Configure Application Insights

**File**: `backend/src/observability/app_insights.rs`

```rust
use applicationinsights::{TelemetryClient, TelemetryConfig};
use std::sync::Arc;

pub struct AppInsights {
    client: Arc<TelemetryClient>,
}

impl AppInsights {
    pub fn new(instrumentation_key: &str) -> Self {
        let config = TelemetryConfig::new(instrumentation_key.to_string());
        let client = TelemetryClient::new(config);

        Self {
            client: Arc::new(client),
        }
    }

    pub fn track_request(&self, name: &str, duration: f64, success: bool) {
        self.client.track_request(
            name.to_string(),
            duration,
            if success { "200" } else { "500" },
        );
    }

    pub fn track_dependency(&self, name: &str, dep_type: &str, duration: f64) {
        self.client.track_dependency(
            name.to_string(),
            dep_type.to_string(),
            duration,
            true,
        );
    }

    pub fn track_exception(&self, error: &str, properties: Option<std::collections::HashMap<String, String>>) {
        self.client.track_exception(error.to_string(), properties);
    }

    pub fn track_event(&self, name: &str, properties: std::collections::HashMap<String, String>) {
        self.client.track_event(name.to_string(), Some(properties));
    }
}
```

**Acceptance Criteria**:
- [ ] Application Insights client initialized
- [ ] Request tracking implemented
- [ ] Dependency tracking for external APIs
- [ ] Exception tracking with context

---

## Epic 3: Distributed Tracing with OpenTelemetry

### Overview

Implement distributed tracing using OpenTelemetry to track requests across microservices, databases, and external APIs.

### Task 3.1: Install OpenTelemetry Dependencies

```toml
[dependencies]
# OpenTelemetry
opentelemetry = { version = "0.21", features = ["rt-tokio"] }
opentelemetry-otlp = "0.14"
opentelemetry-semantic-conventions = "0.13"
tracing = "0.1"
tracing-opentelemetry = "0.22"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

### Task 3.2: Configure OpenTelemetry Tracing

**File**: `backend/src/observability/tracing.rs`

```rust
use opentelemetry::{
    global,
    sdk::{
        trace::{self, Sampler},
        Resource,
    },
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use tracing_subscriber::{layer::SubscriberExt, Registry};

pub fn init_tracing(service_name: &str, otlp_endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create OTLP tracer
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", service_name.to_string()),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    // Create tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create subscriber
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer().json());

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

/// Macro to create traced spans
#[macro_export]
macro_rules! traced {
    ($name:expr, $($field:tt)*) => {
        tracing::info_span!($name, $($field)*)
    };
}
```

**Usage example**:

```rust
use tracing::instrument;

#[instrument(skip(self))]
async fn execute_mission(&self, mission: Mission) -> Result<MissionResult> {
    let span = traced!("execute_mission", mission.id = %mission.id);
    let _enter = span.enter();

    // Work happens here, automatically traced
    self.decompose_tasks(&mission).await?;
    self.assign_workers(&mission).await?;
    self.execute_tasks(&mission).await
}
```

**Acceptance Criteria**:
- [ ] OpenTelemetry pipeline configured
- [ ] Automatic span creation for all async functions
- [ ] Trace context propagation across services
- [ ] OTLP exporter sending to collector

---

## Epic 4: Structured Logging

### Overview

Implement structured JSON logging with tracing-subscriber for machine-readable logs that integrate with OpenTelemetry traces.

### Task 4.1: Configure Structured Logging

**File**: `backend/src/observability/logging.rs`

```rust
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Registry,
};

pub fn init_logging(env: &str) -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            if env == "production" {
                EnvFilter::new("info")
            } else {
                EnvFilter::new("debug")
            }
        });

    let fmt_layer = fmt::layer()
        .json()
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    Registry::default()
        .with(env_filter)
        .with(fmt_layer)
        .init();

    Ok(())
}

/// Structured logging macros
#[macro_export]
macro_rules! log_agent_event {
    ($level:ident, $agent_id:expr, $event:expr, $($key:tt = $value:expr),*) => {
        tracing::$level!(
            agent.id = %$agent_id,
            event = %$event,
            $($key = %$value),*
        );
    };
}

#[macro_export]
macro_rules! log_mission_event {
    ($level:ident, $mission_id:expr, $event:expr, $($key:tt = $value:expr),*) => {
        tracing::$level!(
            mission.id = %$mission_id,
            event = %$event,
            $($key = %$value),*
        );
    };
}
```

**Usage**:

```rust
log_agent_event!(info, agent.id, "task_started",
    task.id = task.id,
    task.type = task.task_type
);

log_mission_event!(error, mission.id, "mission_failed",
    error = error_msg,
    retry_count = retries
);
```

**Acceptance Criteria**:
- [ ] All logs in JSON format
- [ ] Structured fields for filtering
- [ ] Trace correlation IDs included
- [ ] Log levels configurable via environment

---

## Epic 5: Grafana Dashboards

### Overview

Create comprehensive Grafana dashboards for visualizing system health, agent performance, costs, and business metrics.

### Task 5.1: System Health Dashboard

**File**: `infrastructure/grafana/dashboards/system-health.json`

```json
{
  "dashboard": {
    "title": "Ghost Pirates - System Health",
    "uid": "ghostpirates-system-health",
    "timezone": "UTC",
    "panels": [
      {
        "id": 1,
        "title": "API Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (method, path)",
            "legendFormat": "{{method}} {{path}}"
          }
        ],
        "gridPos": {"x": 0, "y": 0, "w": 12, "h": 8}
      },
      {
        "id": 2,
        "title": "API Latency (P50, P95, P99)",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.50, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "P50"
          },
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "P95"
          },
          {
            "expr": "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (le))",
            "legendFormat": "P99"
          }
        ],
        "gridPos": {"x": 12, "y": 0, "w": 12, "h": 8}
      },
      {
        "id": 3,
        "title": "Error Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(errors_total[5m])) by (component, error_type)",
            "legendFormat": "{{component}}: {{error_type}}"
          }
        ],
        "gridPos": {"x": 0, "y": 8, "w": 12, "h": 8}
      },
      {
        "id": 4,
        "title": "Active Teams",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(active_teams_total)"
          }
        ],
        "gridPos": {"x": 12, "y": 8, "w": 6, "h": 4}
      },
      {
        "id": 5,
        "title": "Active Agents",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(active_agents_total)"
          }
        ],
        "gridPos": {"x": 18, "y": 8, "w": 6, "h": 4}
      },
      {
        "id": 6,
        "title": "Database Connection Pool",
        "type": "graph",
        "targets": [
          {
            "expr": "db_pool_connections",
            "legendFormat": "{{state}}"
          }
        ],
        "gridPos": {"x": 12, "y": 12, "w": 12, "h": 8}
      }
    ]
  }
}
```

### Task 5.2: Agent Performance Dashboard

**File**: `infrastructure/grafana/dashboards/agent-performance.json`

```json
{
  "dashboard": {
    "title": "Ghost Pirates - Agent Performance",
    "uid": "ghostpirates-agent-performance",
    "panels": [
      {
        "id": 1,
        "title": "Agent Operations by Type",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(agent_operations_total[5m])) by (agent_type, operation)",
            "legendFormat": "{{agent_type}}: {{operation}}"
          }
        ]
      },
      {
        "id": 2,
        "title": "Agent Success Rate",
        "type": "gauge",
        "targets": [
          {
            "expr": "sum(rate(agent_operations_total{status=\"success\"}[5m])) / sum(rate(agent_operations_total[5m]))",
            "legendFormat": "Success Rate"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "thresholds": {
              "mode": "absolute",
              "steps": [
                {"value": 0, "color": "red"},
                {"value": 0.75, "color": "yellow"},
                {"value": 0.85, "color": "green"}
              ]
            },
            "min": 0,
            "max": 1,
            "unit": "percentunit"
          }
        }
      },
      {
        "id": 3,
        "title": "LLM API Latency by Provider",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(llm_api_duration_seconds_bucket[5m])) by (provider, le))",
            "legendFormat": "{{provider}} P95"
          }
        ]
      },
      {
        "id": 4,
        "title": "Task Queue Size",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(task_queue_size) by (priority)",
            "legendFormat": "Priority: {{priority}}"
          }
        ]
      }
    ]
  }
}
```

### Task 5.3: Cost Tracking Dashboard

**File**: `infrastructure/grafana/dashboards/cost-tracking.json`

```json
{
  "dashboard": {
    "title": "Ghost Pirates - Cost Tracking",
    "uid": "ghostpirates-costs",
    "panels": [
      {
        "id": 1,
        "title": "Total LLM Costs (Hourly)",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(llm_cost_usd_total[1h])) * 3600",
            "legendFormat": "Total Cost/Hour"
          }
        ]
      },
      {
        "id": 2,
        "title": "Cost by Provider",
        "type": "piechart",
        "targets": [
          {
            "expr": "sum(increase(llm_cost_usd_total[24h])) by (provider)",
            "legendFormat": "{{provider}}"
          }
        ]
      },
      {
        "id": 3,
        "title": "Tokens Used by Model",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(llm_tokens_used_total[5m])) by (model, token_type)",
            "legendFormat": "{{model}} ({{token_type}})"
          }
        ]
      },
      {
        "id": 4,
        "title": "Average Cost per Mission",
        "type": "stat",
        "targets": [
          {
            "expr": "sum(increase(llm_cost_usd_total[24h])) / sum(increase(missions_completed_total[24h]))",
            "legendFormat": "Avg Cost"
          }
        ],
        "fieldConfig": {
          "defaults": {
            "unit": "currencyUSD"
          }
        }
      },
      {
        "id": 5,
        "title": "Revenue vs Cost",
        "type": "graph",
        "targets": [
          {
            "expr": "sum(rate(revenue_usd_total[1h])) * 3600",
            "legendFormat": "Revenue/Hour"
          },
          {
            "expr": "sum(rate(llm_cost_usd_total[1h])) * 3600",
            "legendFormat": "Cost/Hour"
          }
        ]
      }
    ]
  }
}
```

**Acceptance Criteria**:
- [ ] All dashboards created in Grafana
- [ ] Panels connected to Prometheus data source
- [ ] Thresholds configured for key metrics
- [ ] Dashboards exported as JSON for version control

---

## Epic 6: Alerting Rules

### Overview

Configure PagerDuty integration and alerting rules for critical system failures, performance degradation, and cost overruns.

### Task 6.1: Configure Prometheus Alerting Rules

**File**: `infrastructure/prometheus/alerts.yml`

```yaml
groups:
  - name: system_health
    interval: 30s
    rules:
      - alert: HighErrorRate
        expr: sum(rate(errors_total[5m])) > 10
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} errors/sec (threshold: 10)"

      - alert: HighAPILatency
        expr: histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > 2
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High API latency"
          description: "P95 latency is {{ $value }}s (threshold: 2s)"

      - alert: LowDatabaseConnections
        expr: db_pool_connections{state="idle"} < 2
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "Low database connections available"
          description: "Only {{ $value }} idle connections remaining"

  - name: agent_performance
    interval: 1m
    rules:
      - alert: LowAgentSuccessRate
        expr: sum(rate(agent_operations_total{status="success"}[10m])) / sum(rate(agent_operations_total[10m])) < 0.75
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "Agent success rate below target"
          description: "Success rate is {{ $value | humanizePercentage }} (threshold: 75%)"

      - alert: HighTaskQueueBacklog
        expr: sum(task_queue_size) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "Task queue backlog building up"
          description: "{{ $value }} tasks in queue (threshold: 100)"

  - name: cost_management
    interval: 5m
    rules:
      - alert: HighHourlyCosts
        expr: sum(rate(llm_cost_usd_total[1h])) * 3600 > 100
        for: 10m
        labels:
          severity: critical
        annotations:
          summary: "High hourly LLM costs"
          description: "Spending ${{ $value }}/hour (threshold: $100/hour)"

      - alert: CostPerMissionTooHigh
        expr: sum(increase(llm_cost_usd_total[1h])) / sum(increase(missions_completed_total[1h])) > 50
        for: 30m
        labels:
          severity: warning
        annotations:
          summary: "Cost per mission exceeds target"
          description: "Average cost is ${{ $value }} (threshold: $50)"

  - name: llm_api_health
    interval: 1m
    rules:
      - alert: LLMAPIFailureRate
        expr: sum(rate(llm_api_calls{status="error"}[5m])) / sum(rate(llm_api_calls[5m])) > 0.1
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "High LLM API failure rate"
          description: "{{ $value | humanizePercentage }} of LLM calls failing (threshold: 10%)"

      - alert: LLMAPIHighLatency
        expr: histogram_quantile(0.95, sum(rate(llm_api_duration_seconds_bucket[5m])) by (le)) > 30
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High LLM API latency"
          description: "P95 latency is {{ $value }}s (threshold: 30s)"
```

### Task 6.2: Configure PagerDuty Integration

**File**: `infrastructure/prometheus/alertmanager.yml`

```yaml
global:
  resolve_timeout: 5m
  pagerduty_url: 'https://events.pagerduty.com/v2/enqueue'

route:
  receiver: 'default'
  group_by: ['alertname', 'severity']
  group_wait: 30s
  group_interval: 5m
  repeat_interval: 4h
  routes:
    - match:
        severity: critical
      receiver: 'pagerduty-critical'
      continue: true
    - match:
        severity: warning
      receiver: 'pagerduty-warning'

receivers:
  - name: 'default'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_SERVICE_KEY}'
        description: '{{ .CommonAnnotations.summary }}'
        details:
          firing: '{{ .Alerts.Firing | len }}'
          resolved: '{{ .Alerts.Resolved | len }}'
          details: '{{ .CommonAnnotations.description }}'

  - name: 'pagerduty-critical'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_CRITICAL_KEY}'
        severity: 'critical'
        description: '[CRITICAL] {{ .CommonAnnotations.summary }}'

  - name: 'pagerduty-warning'
    pagerduty_configs:
      - service_key: '${PAGERDUTY_WARNING_KEY}'
        severity: 'warning'
        description: '[WARNING] {{ .CommonAnnotations.summary }}'

inhibit_rules:
  - source_match:
      severity: 'critical'
    target_match:
      severity: 'warning'
    equal: ['alertname']
```

**Acceptance Criteria**:
- [ ] Alert rules cover all critical scenarios
- [ ] PagerDuty integration configured
- [ ] Alert thresholds validated
- [ ] Test alerts successfully sent

---

## Epic 7: Log Aggregation

### Overview

Configure Azure Log Analytics for centralized log aggregation, searching, and correlation with metrics and traces.

### Task 7.1: Configure Azure Log Analytics Workspace

**File**: `infrastructure/terraform/log_analytics.tf`

```hcl
resource "azurerm_log_analytics_workspace" "ghostpirates" {
  name                = "ghostpirates-logs-${var.environment}"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  sku                 = "PerGB2018"
  retention_in_days   = 90

  tags = {
    Environment = var.environment
    Project     = "ghostpirates"
  }
}

resource "azurerm_log_analytics_solution" "container_insights" {
  solution_name         = "ContainerInsights"
  location              = azurerm_resource_group.main.location
  resource_group_name   = azurerm_resource_group.main.name
  workspace_resource_id = azurerm_log_analytics_workspace.ghostpirates.id
  workspace_name        = azurerm_log_analytics_workspace.ghostpirates.name

  plan {
    publisher = "Microsoft"
    product   = "OMSGallery/ContainerInsights"
  }
}

# AKS diagnostic settings
resource "azurerm_monitor_diagnostic_setting" "aks" {
  name                       = "aks-diagnostics"
  target_resource_id         = azurerm_kubernetes_cluster.main.id
  log_analytics_workspace_id = azurerm_log_analytics_workspace.ghostpirates.id

  log {
    category = "kube-apiserver"
    enabled  = true
  }

  log {
    category = "kube-controller-manager"
    enabled  = true
  }

  log {
    category = "kube-scheduler"
    enabled  = true
  }

  metric {
    category = "AllMetrics"
    enabled  = true
  }
}
```

### Task 7.2: Create Kusto Queries for Common Scenarios

**File**: `infrastructure/log_analytics/queries.kql`

```kql
// Query 1: Failed missions in last 24 hours
ContainerLog
| where TimeGenerated > ago(24h)
| where LogEntry contains "mission_failed"
| extend mission_id = extract("mission.id = ([a-f0-9-]+)", 1, LogEntry)
| extend error = extract("error = ([^,]+)", 1, LogEntry)
| project TimeGenerated, mission_id, error
| order by TimeGenerated desc

// Query 2: Slow LLM API calls (> 10s)
ContainerLog
| where TimeGenerated > ago(1h)
| where LogEntry contains "llm_api_duration"
| extend duration = extract("duration = ([0-9.]+)", 1, LogEntry)
| where todouble(duration) > 10
| extend provider = extract("provider = ([^,]+)", 1, LogEntry)
| extend model = extract("model = ([^,]+)", 1, LogEntry)
| project TimeGenerated, provider, model, duration
| order by todouble(duration) desc

// Query 3: Agent error patterns
ContainerLog
| where TimeGenerated > ago(7d)
| where LogEntry contains "agent" and LogEntry contains "error"
| extend agent_type = extract("agent_type = ([^,]+)", 1, LogEntry)
| extend error_type = extract("error_type = ([^,]+)", 1, LogEntry)
| summarize count() by agent_type, error_type
| order by count_ desc

// Query 4: Cost tracking per team
ContainerLog
| where TimeGenerated > ago(24h)
| where LogEntry contains "llm_cost_usd"
| extend team_id = extract("team.id = ([a-f0-9-]+)", 1, LogEntry)
| extend cost = extract("cost = ([0-9.]+)", 1, LogEntry)
| summarize total_cost = sum(todouble(cost)) by team_id
| order by total_cost desc

// Query 5: Mission completion times
ContainerLog
| where TimeGenerated > ago(7d)
| where LogEntry contains "mission_completed"
| extend mission_id = extract("mission.id = ([a-f0-9-]+)", 1, LogEntry)
| extend duration = extract("duration = ([0-9.]+)", 1, LogEntry)
| extend team_size = extract("team_size = ([0-9]+)", 1, LogEntry)
| project TimeGenerated, mission_id, duration, team_size
| summarize avg_duration = avg(todouble(duration)) by team_size
```

**Acceptance Criteria**:
- [ ] Log Analytics workspace created
- [ ] Container logs flowing to workspace
- [ ] Kusto queries saved and tested
- [ ] Retention policy configured (90 days)

---

## Testing & Validation

### Task 8.1: Metrics Testing

**File**: `backend/tests/integration/metrics_test.rs`

```rust
#[tokio::test]
async fn test_metrics_collection() {
    // Initialize metrics
    init_metrics();

    // Simulate operations
    HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/api/teams", "200"]).inc();
    AGENT_OPERATIONS_TOTAL.with_label_values(&["manager", "create_team", "success"]).inc();
    LLM_TOKENS_USED.with_label_values(&["anthropic", "claude-3-5-sonnet", "input"]).inc_by(1000);

    // Fetch metrics
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let metrics_text = encoder.encode_to_string(&metric_families).unwrap();

    // Verify metrics are present
    assert!(metrics_text.contains("http_requests_total"));
    assert!(metrics_text.contains("agent_operations_total"));
    assert!(metrics_text.contains("llm_tokens_used_total"));
}

#[tokio::test]
async fn test_agent_metrics_guard() {
    let guard = AgentMetricsGuard::new("manager", "execute_task");

    // Simulate work
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    guard.finish("success");

    // Verify metrics updated
    let encoder = prometheus::TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let metrics_text = encoder.encode_to_string(&metric_families).unwrap();

    assert!(metrics_text.contains("agent_operations_total"));
    assert!(metrics_text.contains("agent_execution_duration_seconds"));
}
```

### Task 8.2: Load Testing with Metrics

**File**: `backend/tests/load/metrics_performance_test.rs`

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use crate::observability::metrics::*;

fn benchmark_metrics_overhead(c: &mut Criterion) {
    c.bench_function("http_request_metric", |b| {
        b.iter(|| {
            HTTP_REQUESTS_TOTAL.with_label_values(&["GET", "/api/test", "200"]).inc();
        });
    });

    c.bench_function("agent_operation_metric", |b| {
        b.iter(|| {
            AGENT_OPERATIONS_TOTAL.with_label_values(&["worker", "execute", "success"]).inc();
        });
    });

    c.bench_function("histogram_observe", |b| {
        b.iter(|| {
            HTTP_REQUEST_DURATION.with_label_values(&["GET", "/api/test"]).observe(0.123);
        });
    });
}

criterion_group!(benches, benchmark_metrics_overhead);
criterion_main!(benches);
```

**Acceptance Criteria**:
- [ ] All metric types tested
- [ ] Performance overhead < 100µs per metric
- [ ] Integration tests verify data collection
- [ ] Load tests confirm scalability

---

## Deployment Checklist

### Pre-Deployment
- [ ] All dependencies installed
- [ ] Prometheus running and scraping /metrics endpoint
- [ ] Grafana connected to Prometheus data source
- [ ] Application Insights instrumentation key configured
- [ ] OpenTelemetry collector running
- [ ] Azure Log Analytics workspace created
- [ ] PagerDuty integration keys configured

### Deployment
- [ ] Deploy updated backend with metrics
- [ ] Verify /metrics endpoint accessible
- [ ] Import Grafana dashboards
- [ ] Test alert rules trigger correctly
- [ ] Verify logs flowing to Azure Log Analytics
- [ ] Test PagerDuty notifications

### Post-Deployment
- [ ] Monitor dashboard for 24 hours
- [ ] Verify no performance degradation
- [ ] Test alert escalation paths
- [ ] Document runbooks for common alerts
- [ ] Train team on dashboard usage

---

## Success Metrics

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Metrics Collection Uptime | 99.9% | TBD | ⏳ |
| MTTD (Mean Time to Detection) | <2 min | TBD | ⏳ |
| MTTR (Mean Time to Resolution) | <30 min | TBD | ⏳ |
| Dashboard Load Time | <3s | TBD | ⏳ |
| Alert False Positive Rate | <5% | TBD | ⏳ |
| Log Retention Coverage | 90 days | TBD | ⏳ |

---

## Next Steps

1. **Proceed to [17-pricing-model.md](./17-pricing-model.md)** for billing implementation
2. **Review [18-success-metrics.md](./18-success-metrics.md)** for launch criteria
3. **Monitor metrics** continuously during all testing phases
4. **Iterate on dashboards** based on team feedback

---

**Ghost Pirates Observability: Complete visibility into autonomous AI teams.**
