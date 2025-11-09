# Pattern 47: Monitoring & Observability Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Category**: Operations & Reliability

---

## Table of Contents

1. [Overview](#overview)
2. [Three Pillars of Observability](#three-pillars-of-observability)
3. [Logging](#logging)
4. [Metrics](#metrics)
5. [Distributed Tracing](#distributed-tracing)
6. [Structured Logging](#structured-logging)
7. [Application Performance Monitoring (APM)](#application-performance-monitoring-apm)
8. [Health Checks](#health-checks)
9. [Alerting](#alerting)
10. [Error Tracking](#error-tracking)
11. [Rust Implementation](#rust-implementation)
12. [Frontend Monitoring](#frontend-monitoring)
13. [Dashboard Setup](#dashboard-setup)
14. [Best Practices](#best-practices)
15. [Anti-Patterns](#anti-patterns)
16. [Related Patterns](#related-patterns)
17. [References](#references)

---

## Overview

**Observability** is the ability to understand the internal state of your system by examining its outputs. In WellOS, observability is critical for:

- **Debugging production issues** - Understand what went wrong
- **Performance optimization** - Identify bottlenecks
- **Business insights** - Track user behavior and revenue
- **SLA compliance** - Ensure uptime and response time targets
- **Security monitoring** - Detect anomalies and attacks

**Monitoring vs Observability**:

- **Monitoring**: Known problems, predefined dashboards, alerts
- **Observability**: Unknown problems, ad-hoc queries, exploration

---

## Three Pillars of Observability

### 1. Logs

**What**: Timestamped records of discrete events.

**Use Cases**:

- Debugging errors and exceptions
- Audit trails (who did what, when)
- Security events (login attempts, permission denials)

**Example**:

```
[2025-10-08T10:30:45.123Z] INFO: User login successful { userId: "abc123", email: "john@acme.com", ip: "192.168.1.1" }
[2025-10-08T10:30:47.456Z] ERROR: Failed to create project { userId: "abc123", error: "Insufficient permissions" }
```

---

### 2. Metrics

**What**: Numerical measurements over time.

**Use Cases**:

- System health (CPU, memory, disk)
- Application performance (request rate, latency, error rate)
- Business KPIs (active users, revenue, conversion rate)

**Example**:

```
http_requests_total{method="GET", path="/api/projects", status="200"} 1523
http_request_duration_seconds{method="GET", path="/api/projects"} 0.045
active_users_count 247
```

---

### 3. Traces

**What**: End-to-end journey of a request through the system.

**Use Cases**:

- Performance profiling (which service is slow?)
- Dependency mapping (how do services interact?)
- Root cause analysis (where did the error originate?)

**Example**:

```
Trace ID: abc123def456
├─ HTTP GET /api/projects (120ms)
│  ├─ JwtAuthGuard.canActivate (5ms)
│  ├─ TenantGuard.canActivate (3ms)
│  ├─ QueryBus.execute (110ms)
│  │  ├─ GetProjectsHandler.execute (108ms)
│  │  │  ├─ Cache.get (2ms) - miss
│  │  │  ├─ Repository.findAll (95ms)
│  │  │  │  └─ PostgreSQL query (92ms)
│  │  │  └─ Cache.set (3ms)
│  └─ Response serialization (2ms)
```

---

## Logging

### Log Levels

Use appropriate log levels to control verbosity:

| Level     | When to Use                                     | Examples                                               |
| --------- | ----------------------------------------------- | ------------------------------------------------------ |
| **ERROR** | Unexpected errors requiring immediate attention | Database connection failed, payment processing failed  |
| **WARN**  | Potentially harmful situations                  | Deprecated API usage, slow query (>1s)                 |
| **INFO**  | Important business events                       | User registered, invoice created, password reset       |
| **DEBUG** | Detailed diagnostic information                 | Query parameters, function entry/exit, variable values |
| **TRACE** | Very fine-grained information                   | HTTP request/response bodies, loop iterations          |

### Structured Logger Setup (Rust)

```rust
// apps/scada-ingestion/src/infrastructure/logging/logger.rs
use serde::Serialize;
use tracing::{error, warn, info, debug, trace, Level};
use tracing_appender::{rolling, non_blocking};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::collections::HashMap;

pub struct LoggerService {
    // tracing subscriber is global, no need to store instance
}

impl LoggerService {
    pub fn init() -> Self {
        // File appender for error logs
        let error_appender = rolling::daily("logs", "error.log");
        let (error_writer, _error_guard) = non_blocking(error_appender);

        // File appender for all logs
        let all_appender = rolling::daily("logs", "combined.log");
        let (all_writer, _all_guard) = non_blocking(all_appender);

        // Initialize tracing subscriber with multiple layers
        tracing_subscriber::registry()
            .with(
                EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| EnvFilter::new("info"))
            )
            .with(
                fmt::layer()
                    .with_writer(std::io::stdout)
                    .json()
                    .with_current_span(true)
            )
            .with(
                fmt::layer()
                    .with_writer(error_writer)
                    .with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
            )
            .with(
                fmt::layer()
                    .with_writer(all_writer)
            )
            .init();

        Self {}
    }

    pub fn error(&self, message: &str, context: Option<HashMap<String, String>>) {
        if let Some(ctx) = context {
            error!(message = message, context = ?ctx);
        } else {
            error!(message = message);
        }
    }

    pub fn warn(&self, message: &str, context: Option<HashMap<String, String>>) {
        if let Some(ctx) = context {
            warn!(message = message, context = ?ctx);
        } else {
            warn!(message = message);
        }
    }

    pub fn info(&self, message: &str, context: Option<HashMap<String, String>>) {
        if let Some(ctx) = context {
            info!(message = message, context = ?ctx);
        } else {
            info!(message = message);
        }
    }

    pub fn debug(&self, message: &str, context: Option<HashMap<String, String>>) {
        if let Some(ctx) = context {
            debug!(message = message, context = ?ctx);
        } else {
            debug!(message = message);
        }
    }

    pub fn trace(&self, message: &str, context: Option<HashMap<String, String>>) {
        if let Some(ctx) = context {
            trace!(message = message, context = ?ctx);
        } else {
            trace!(message = message);
        }
    }
}
```

### Logging Best Practices

```rust
// ✅ Good: Structured logging with context
use tracing::info;
use serde_json::json;

info!(
    user_id = %user.id,
    email = %user.email,
    organization_id = %user.organization_id,
    registration_method = "email",
    "User registered"
);

// ❌ Bad: Unstructured logging
info!("User {} registered", user.email);

// ✅ Good: Include correlation IDs for request tracing
use tracing::error;

error!(
    correlation_id = %request.id,
    user_id = %user.id,
    amount = payment.amount,
    error = %error,
    "Payment processing failed"
);

// ❌ Bad: Missing context
error!("Payment failed");
```

---

## Metrics

### Prometheus Metrics

```rust
// apps/scada-ingestion/src/infrastructure/metrics/metrics_service.rs
use prometheus::{
    Counter, CounterVec, Histogram, HistogramVec, Gauge, GaugeVec,
    Registry, Encoder, TextEncoder, Opts, HistogramOpts,
};
use std::sync::Arc;

pub struct MetricsService {
    registry: Arc<Registry>,

    // HTTP Metrics
    http_requests_total: CounterVec,
    http_request_duration: HistogramVec,
    http_request_size: HistogramVec,
    http_response_size: HistogramVec,

    // Application Metrics
    active_users: Gauge,
    database_connections: Gauge,
    cache_hit_rate: GaugeVec,

    // Business Metrics
    scada_readings_ingested: CounterVec,
    alarms_triggered: CounterVec,
    adapters_active: GaugeVec,
}

impl MetricsService {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let registry = Arc::new(Registry::new());

        // HTTP Metrics
        let http_requests_total = CounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests"),
            &["method", "path", "status"]
        )?;

        let http_request_duration = HistogramVec::new(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request latency in seconds")
                .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0]),
            &["method", "path", "status"]
        )?;

        let http_request_size = HistogramVec::new(
            HistogramOpts::new("http_request_size_bytes", "HTTP request size in bytes")
                .buckets(vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]),
            &["method", "path"]
        )?;

        let http_response_size = HistogramVec::new(
            HistogramOpts::new("http_response_size_bytes", "HTTP response size in bytes")
                .buckets(vec![100.0, 1000.0, 10000.0, 100000.0, 1000000.0]),
            &["method", "path"]
        )?;

        // Application Metrics
        let active_users = Gauge::new("active_users_count", "Number of active users")?;
        let database_connections = Gauge::new("database_connections_active", "Active database connections")?;
        let cache_hit_rate = GaugeVec::new(
            Opts::new("cache_hit_rate", "Cache hit rate (0-1)"),
            &["cache_type"]
        )?;

        // Business Metrics
        let scada_readings_ingested = CounterVec::new(
            Opts::new("scada_readings_ingested_total", "Total SCADA readings ingested"),
            &["tenant_id", "adapter_type"]
        )?;

        let alarms_triggered = CounterVec::new(
            Opts::new("alarms_triggered_total", "Total alarms triggered"),
            &["tenant_id", "severity"]
        )?;

        let adapters_active = GaugeVec::new(
            Opts::new("adapters_active", "Number of active protocol adapters"),
            &["adapter_type"]
        )?;

        // Register all metrics
        registry.register(Box::new(http_requests_total.clone()))?;
        registry.register(Box::new(http_request_duration.clone()))?;
        registry.register(Box::new(http_request_size.clone()))?;
        registry.register(Box::new(http_response_size.clone()))?;
        registry.register(Box::new(active_users.clone()))?;
        registry.register(Box::new(database_connections.clone()))?;
        registry.register(Box::new(cache_hit_rate.clone()))?;
        registry.register(Box::new(scada_readings_ingested.clone()))?;
        registry.register(Box::new(alarms_triggered.clone()))?;
        registry.register(Box::new(adapters_active.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_request_duration,
            http_request_size,
            http_response_size,
            active_users,
            database_connections,
            cache_hit_rate,
            scada_readings_ingested,
            alarms_triggered,
            adapters_active,
        })
    }

    // HTTP Metrics Methods
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: f64) {
        self.http_requests_total
            .with_label_values(&[method, path, &status.to_string()])
            .inc();
        self.http_request_duration
            .with_label_values(&[method, path, &status.to_string()])
            .observe(duration);
    }

    pub fn record_http_request_size(&self, method: &str, path: &str, size: f64) {
        self.http_request_size
            .with_label_values(&[method, path])
            .observe(size);
    }

    pub fn record_http_response_size(&self, method: &str, path: &str, size: f64) {
        self.http_response_size
            .with_label_values(&[method, path])
            .observe(size);
    }

    // Application Metrics Methods
    pub fn set_active_users(&self, count: f64) {
        self.active_users.set(count);
    }

    pub fn set_database_connections(&self, count: f64) {
        self.database_connections.set(count);
    }

    pub fn set_cache_hit_rate(&self, cache_type: &str, rate: f64) {
        self.cache_hit_rate
            .with_label_values(&[cache_type])
            .set(rate);
    }

    // Business Metrics Methods
    pub fn increment_scada_readings(&self, tenant_id: &str, adapter_type: &str) {
        self.scada_readings_ingested
            .with_label_values(&[tenant_id, adapter_type])
            .inc();
    }

    pub fn increment_alarms(&self, tenant_id: &str, severity: &str) {
        self.alarms_triggered
            .with_label_values(&[tenant_id, severity])
            .inc();
    }

    pub fn set_adapters_active(&self, adapter_type: &str, count: f64) {
        self.adapters_active
            .with_label_values(&[adapter_type])
            .set(count);
    }

    // Expose metrics for Prometheus scraping
    pub fn get_metrics(&self) -> Result<String, Box<dyn std::error::Error>> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = vec![];
        encoder.encode(&metric_families, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }
}
```

### Metrics Endpoint (Actix-web)

```rust
// apps/scada-ingestion/src/presentation/metrics/metrics_handler.rs
use actix_web::{web, HttpResponse, Result};
use crate::infrastructure::metrics::metrics_service::MetricsService;
use std::sync::Arc;

pub async fn get_metrics(
    metrics: web::Data<Arc<MetricsService>>,
) -> Result<HttpResponse> {
    match metrics.get_metrics() {
        Ok(metrics_text) => Ok(HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4")
            .body(metrics_text)),
        Err(e) => Ok(HttpResponse::InternalServerError()
            .body(format!("Error generating metrics: {}", e))),
    }
}

// Register route in main.rs
pub fn configure_metrics(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/metrics")
            .route(web::get().to(get_metrics))
    );
}
```

### Metrics Middleware (Actix-web)

```rust
// apps/scada-ingestion/src/infrastructure/middleware/metrics_middleware.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures::future::{ok, Ready};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use std::time::Instant;
use std::sync::Arc;
use crate::infrastructure::metrics::metrics_service::MetricsService;

pub struct MetricsMiddleware {
    metrics: Arc<MetricsService>,
}

impl MetricsMiddleware {
    pub fn new(metrics: Arc<MetricsService>) -> Self {
        Self { metrics }
    }
}

impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = MetricsMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(MetricsMiddlewareService {
            service,
            metrics: self.metrics.clone(),
        })
    }
}

pub struct MetricsMiddlewareService<S> {
    service: S,
    metrics: Arc<MetricsService>,
}

impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let method = req.method().to_string();
        let path = req.path().to_string();
        let start = Instant::now();
        let metrics = self.metrics.clone();

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            let duration = start.elapsed().as_secs_f64();
            let status = res.status().as_u16();

            metrics.record_http_request(&method, &path, status, duration);

            Ok(res)
        })
    }
}
```

---

## Distributed Tracing

### OpenTelemetry Setup (Rust)

```rust
// apps/scada-ingestion/src/infrastructure/tracing/tracing_service.rs
use opentelemetry::{
    global,
    sdk::{
        trace::{self, Tracer},
        Resource,
    },
    KeyValue,
};
use opentelemetry_jaeger::JaegerPipeline;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

pub struct TracingService;

impl TracingService {
    pub fn init() -> Result<Tracer, Box<dyn std::error::Error>> {
        // Configure Jaeger exporter
        let tracer = opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name("scada-ingestion")
            .with_endpoint(
                std::env::var("JAEGER_ENDPOINT")
                    .unwrap_or_else(|_| "localhost:6831".to_string())
            )
            .with_auto_split_batch(true)
            .install_batch(opentelemetry::runtime::Tokio)?;

        // Create OpenTelemetry tracing layer
        let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer.clone());

        // Set up the tracing subscriber
        let subscriber = Registry::default()
            .with(telemetry_layer)
            .with(tracing_subscriber::fmt::layer());

        tracing::subscriber::set_global_default(subscriber)?;

        Ok(tracer)
    }

    pub fn shutdown() {
        global::shutdown_tracer_provider();
    }
}
```

### Custom Spans (Rust)

```rust
// apps/scada-ingestion/src/application/scada/queries/get_well_production.rs
use tracing::{info_span, instrument};
use opentelemetry::trace::{SpanKind, Status, StatusCode};

pub struct GetWellProductionHandler {
    well_repository: Arc<dyn WellRepository>,
    production_repository: Arc<dyn ProductionRepository>,
}

impl GetWellProductionHandler {
    #[instrument(
        name = "get_well_production",
        skip(self),
        fields(
            well_id = %query.well_id,
            tenant_id = %query.tenant_id
        )
    )]
    pub async fn execute(&self, query: GetWellProductionQuery) -> Result<WellProduction, Error> {
        let root_span = info_span!("GetWellProduction");
        let _enter = root_span.enter();

        // Fetch well data
        let well = {
            let _span = info_span!("fetch_well").entered();
            self.well_repository
                .find_by_id(&query.tenant_id, &query.well_id)
                .await?
        };

        // Fetch production data
        let production = {
            let _span = info_span!("fetch_production").entered();
            self.production_repository
                .find_by_well(&query.tenant_id, &query.well_id, &query.date_range)
                .await?
        };

        // Calculate metrics
        let metrics = {
            let _span = info_span!("calculate_metrics").entered();
            self.calculate_metrics(&well, &production)?
        };

        info!(
            well_id = %query.well_id,
            production_days = production.len(),
            "Well production retrieved successfully"
        );

        Ok(WellProduction {
            well,
            production,
            metrics,
        })
    }

    fn calculate_metrics(
        &self,
        well: &Well,
        production: &[ProductionRecord],
    ) -> Result<ProductionMetrics, Error> {
        // Calculation logic
        Ok(ProductionMetrics::default())
    }
}
```

---

## Structured Logging

### Request Context (Actix-web)

```rust
// apps/scada-ingestion/src/infrastructure/middleware/request_context.rs
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ok, Ready};
use std::task::{Context, Poll};
use std::pin::Pin;
use std::future::Future;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct RequestContext {
    pub request_id: String,
    pub tenant_id: Option<String>,
    pub user_id: Option<String>,
    pub ip: String,
    pub user_agent: String,
}

pub struct RequestContextMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestContextMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestContextMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestContextMiddlewareService { service })
    }
}

pub struct RequestContextMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestContextMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        let ip = req
            .connection_info()
            .realip_remote_addr()
            .unwrap_or("unknown")
            .to_string();

        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let context = RequestContext {
            request_id,
            tenant_id: None, // Set by tenant resolver middleware
            user_id: None,   // Set by auth middleware
            ip,
            user_agent,
        };

        req.extensions_mut().insert(context);

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

// Helper to get context in handlers
pub fn get_request_context(req: &actix_web::HttpRequest) -> Option<RequestContext> {
    req.extensions().get::<RequestContext>().cloned()
}
```

---

## Application Performance Monitoring (APM)

### Sentry Integration (Rust)

```rust
// apps/scada-ingestion/src/infrastructure/apm/sentry_service.rs
use sentry::{ClientOptions, IntoDsn};
use std::collections::HashMap;

pub struct SentryService {
    // Sentry client is initialized globally
}

impl SentryService {
    pub fn init() -> Result<sentry::ClientInitGuard, Box<dyn std::error::Error>> {
        let dsn = std::env::var("SENTRY_DSN")?;
        let environment = std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        let release = format!("scada-ingestion@{}", env!("CARGO_PKG_VERSION"));

        let guard = sentry::init((
            dsn.into_dsn()?,
            ClientOptions {
                release: Some(release.into()),
                environment: Some(environment.into()),
                traces_sample_rate: 0.1, // 10% sampling
                ..Default::default()
            },
        ));

        Ok(guard)
    }

    pub fn capture_exception(&self, error: &dyn std::error::Error, context: Option<HashMap<String, String>>) {
        sentry::with_scope(
            |scope| {
                if let Some(ctx) = context {
                    for (key, value) in ctx {
                        scope.set_extra(&key, value.into());
                    }
                }
            },
            || {
                sentry::capture_error(error);
            },
        );
    }

    pub fn capture_message(&self, message: &str, level: sentry::Level) {
        sentry::capture_message(message, level);
    }

    pub fn set_user(&self, user_id: &str, email: &str, tenant_id: Option<&str>) {
        sentry::configure_scope(|scope| {
            scope.set_user(Some(sentry::User {
                id: Some(user_id.to_string()),
                email: Some(email.to_string()),
                username: tenant_id.map(|t| t.to_string()),
                ..Default::default()
            }));
        });
    }

    pub fn add_breadcrumb(&self, message: &str, category: &str, data: Option<HashMap<String, String>>) {
        let mut breadcrumb = sentry::Breadcrumb {
            message: Some(message.to_string()),
            category: Some(category.to_string()),
            level: sentry::Level::Info,
            ..Default::default()
        };

        if let Some(d) = data {
            breadcrumb.data = d.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect();
        }

        sentry::add_breadcrumb(breadcrumb);
    }
}
```

### Global Error Handler with APM (Actix-web)

```rust
// apps/scada-ingestion/src/infrastructure/errors/error_handler.rs
use actix_web::{
    error::{JsonPayloadError, ResponseError},
    http::StatusCode,
    HttpResponse,
};
use serde::Serialize;
use std::fmt;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status_code: u16,
    pub message: String,
    pub timestamp: String,
    pub path: String,
}

#[derive(Debug)]
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
    pub context: Option<HashMap<String, String>>,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let error_response = ErrorResponse {
            status_code: self.status_code.as_u16(),
            message: self.message.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            path: String::new(), // Set by middleware
        };

        // Log error
        error!(
            status_code = self.status_code.as_u16(),
            message = %self.message,
            context = ?self.context,
            "Error occurred"
        );

        // Send to Sentry for 5xx errors
        if self.status_code.is_server_error() {
            sentry::capture_message(&self.message, sentry::Level::Error);
        }

        HttpResponse::build(self.status_code).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        self.status_code
    }
}

// Convert from various error types
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        error!("Database error: {:?}", err);
        sentry::capture_error(&err);

        AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Database error occurred".to_string(),
            context: None,
        }
    }
}
```

---

## Health Checks

```rust
// apps/scada-ingestion/src/presentation/health/health_handler.rs
use actix_web::{web, HttpResponse, Result};
use serde::Serialize;
use sqlx::PgPool;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use sysinfo::{System, SystemExt};

#[derive(Serialize)]
pub struct HealthCheck {
    status: String,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: HealthStatus,
    redis: HealthStatus,
    memory: MemoryStatus,
    disk: DiskStatus,
}

#[derive(Serialize)]
pub struct HealthStatus {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Serialize)]
pub struct MemoryStatus {
    status: String,
    used_mb: u64,
    total_mb: u64,
    usage_percent: f64,
}

#[derive(Serialize)]
pub struct DiskStatus {
    status: String,
    available_gb: u64,
    total_gb: u64,
    usage_percent: f64,
}

pub async fn health_check(
    db_pool: web::Data<PgPool>,
    redis: web::Data<ConnectionManager>,
) -> Result<HttpResponse> {
    let mut all_healthy = true;

    // Database health check
    let db_status = match sqlx::query("SELECT 1").fetch_one(db_pool.get_ref()).await {
        Ok(_) => HealthStatus {
            status: "up".to_string(),
            message: None,
        },
        Err(e) => {
            all_healthy = false;
            HealthStatus {
                status: "down".to_string(),
                message: Some(e.to_string()),
            }
        }
    };

    // Redis health check
    let mut redis_conn = redis.get_ref().clone();
    let redis_status = match redis::cmd("PING").query_async::<_, String>(&mut redis_conn).await {
        Ok(_) => HealthStatus {
            status: "up".to_string(),
            message: None,
        },
        Err(e) => {
            all_healthy = false;
            HealthStatus {
                status: "down".to_string(),
                message: Some(e.to_string()),
            }
        }
    };

    // Memory health check
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory = sys.total_memory();
    let used_memory = sys.used_memory();
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;

    let memory_status = MemoryStatus {
        status: if memory_usage < 90.0 { "healthy" } else { "warning" }.to_string(),
        used_mb: used_memory / 1024 / 1024,
        total_mb: total_memory / 1024 / 1024,
        usage_percent: memory_usage,
    };

    // Disk health check
    let disk = sys.disks().first();
    let disk_status = if let Some(d) = disk {
        let total = d.total_space();
        let available = d.available_space();
        let usage = ((total - available) as f64 / total as f64) * 100.0;

        DiskStatus {
            status: if usage < 90.0 { "healthy" } else { "warning" }.to_string(),
            available_gb: available / 1024 / 1024 / 1024,
            total_gb: total / 1024 / 1024 / 1024,
            usage_percent: usage,
        }
    } else {
        DiskStatus {
            status: "unknown".to_string(),
            available_gb: 0,
            total_gb: 0,
            usage_percent: 0.0,
        }
    };

    let health_check = HealthCheck {
        status: if all_healthy { "healthy" } else { "unhealthy" }.to_string(),
        checks: HealthChecks {
            database: db_status,
            redis: redis_status,
            memory: memory_status,
            disk: disk_status,
        },
    };

    let status_code = if all_healthy { 200 } else { 503 };
    Ok(HttpResponse::build(actix_web::http::StatusCode::from_u16(status_code).unwrap())
        .json(health_check))
}

// Register route in main.rs
pub fn configure_health(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/health")
            .route(web::get().to(health_check))
    );
}
```

---

## Alerting

### Alert Rules (Prometheus)

```yaml
# prometheus-alerts.yml
groups:
  - name: wellos_api_alerts
    interval: 30s
    rules:
      # High error rate
      - alert: HighErrorRate
        expr: |
          (
            sum(rate(http_requests_total{status=~"5.."}[5m]))
            /
            sum(rate(http_requests_total[5m]))
          ) > 0.05
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: 'High error rate detected'
          description: 'Error rate is {{ $value | humanizePercentage }} (threshold: 5%)'

      # High latency
      - alert: HighLatency
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
          ) > 1
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: 'High API latency detected'
          description: 'P95 latency is {{ $value }}s (threshold: 1s)'

      # Database connection pool exhausted
      - alert: DatabaseConnectionPoolExhausted
        expr: database_connections_active > 80
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: 'Database connection pool nearly exhausted'
          description: '{{ $value }} active connections (max: 100)'

      # Low cache hit rate
      - alert: LowCacheHitRate
        expr: cache_hit_rate < 0.7
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: 'Cache hit rate is low'
          description: 'Hit rate is {{ $value | humanizePercentage }} (threshold: 70%)'

      # High memory usage
      - alert: HighMemoryUsage
        expr: process_resident_memory_bytes > 500000000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: 'High memory usage'
          description: 'Memory usage is {{ $value | humanize }}B (threshold: 500MB)'
```

---

## Error Tracking

### Error Categorization

```typescript
// apps/api/src/domain/shared/exceptions/base.exception.ts
export enum ErrorCategory {
  AUTHENTICATION = 'authentication',
  AUTHORIZATION = 'authorization',
  VALIDATION = 'validation',
  NOT_FOUND = 'not_found',
  CONFLICT = 'conflict',
  RATE_LIMIT = 'rate_limit',
  EXTERNAL_SERVICE = 'external_service',
  DATABASE = 'database',
  INTERNAL = 'internal',
}

export abstract class BaseException extends Error {
  abstract readonly category: ErrorCategory;
  abstract readonly statusCode: number;

  constructor(
    message: string,
    public readonly context?: Record<string, any>,
  ) {
    super(message);
    this.name = this.constructor.name;
  }
}

// Example: Authentication error
export class InvalidCredentialsException extends BaseException {
  readonly category = ErrorCategory.AUTHENTICATION;
  readonly statusCode = 401;

  constructor(email: string) {
    super('Invalid credentials', { email });
  }
}

// Example: External service error
export class QuickBooksIntegrationException extends BaseException {
  readonly category = ErrorCategory.EXTERNAL_SERVICE;
  readonly statusCode = 502;

  constructor(operation: string, error: Error) {
    super(`QuickBooks integration failed: ${operation}`, {
      operation,
      originalError: error.message,
    });
  }
}
```

---

## Rust Implementation

### Complete Observability Setup

```rust
// apps/scada-ingestion/src/infrastructure/observability/mod.rs
use crate::infrastructure::{
    logging::LoggerService,
    metrics::MetricsService,
    tracing::TracingService,
    apm::SentryService,
};
use std::sync::Arc;

pub struct ObservabilityStack {
    pub logger: LoggerService,
    pub metrics: Arc<MetricsService>,
    pub tracer: opentelemetry::sdk::trace::Tracer,
    pub sentry: sentry::ClientInitGuard,
}

impl ObservabilityStack {
    pub fn init() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize logger first (structured logging)
        let logger = LoggerService::init();

        // Initialize metrics
        let metrics = Arc::new(MetricsService::new()?);

        // Initialize distributed tracing
        let tracer = TracingService::init()?;

        // Initialize error tracking
        let sentry = SentryService::init()?;

        tracing::info!("Observability stack initialized successfully");

        Ok(Self {
            logger,
            metrics,
            tracer,
            sentry,
        })
    }

    pub fn shutdown(self) {
        TracingService::shutdown();
        tracing::info!("Observability stack shutdown complete");
    }
}

// Main application setup
// apps/scada-ingestion/src/main.rs
use actix_web::{web, App, HttpServer};
use crate::infrastructure::observability::ObservabilityStack;
use crate::infrastructure::middleware::{
    MetricsMiddleware,
    RequestContextMiddleware,
};
use crate::presentation::{
    health::configure_health,
    metrics::configure_metrics,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize observability
    let observability = ObservabilityStack::init()
        .expect("Failed to initialize observability");

    let metrics = observability.metrics.clone();

    // Start HTTP server with middleware
    HttpServer::new(move || {
        App::new()
            // Add observability middleware
            .wrap(RequestContextMiddleware)
            .wrap(MetricsMiddleware::new(metrics.clone()))
            .wrap(sentry_actix::Sentry::new())
            // Register routes
            .configure(configure_health)
            .configure(configure_metrics)
            // Application routes...
            .app_data(web::Data::new(metrics.clone()))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

---

## Frontend Monitoring

### Sentry for React

```typescript
// apps/web/lib/monitoring/sentry.ts
import * as Sentry from '@sentry/nextjs';

Sentry.init({
  dsn: process.env.NEXT_PUBLIC_SENTRY_DSN,
  environment: process.env.NODE_ENV,
  release: `wellos-web@${process.env.npm_package_version}`,

  // Tracing
  tracesSampleRate: 0.1,

  // Session replay
  replaysSessionSampleRate: 0.1,
  replaysOnErrorSampleRate: 1.0,

  integrations: [
    new Sentry.BrowserTracing(),
    new Sentry.Replay({
      maskAllText: true,
      blockAllMedia: true,
    }),
  ],

  // Performance monitoring
  beforeSend(event, hint) {
    // Filter out known errors
    if (event.exception?.values?.[0]?.type === 'ChunkLoadError') {
      return null; // User navigated away during chunk load
    }
    return event;
  },
});
```

### Error Boundary

```typescript
// apps/web/components/error-boundary.tsx
'use client';

import React from 'react';
import * as Sentry from '@sentry/nextjs';

interface Props {
  children: React.ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends React.Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    Sentry.captureException(error, {
      contexts: {
        react: {
          componentStack: errorInfo.componentStack,
        },
      },
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center">
          <div className="text-center">
            <h1 className="text-2xl font-bold mb-4">Something went wrong</h1>
            <p className="text-gray-600 mb-4">
              We've been notified and are working on a fix.
            </p>
            <button
              onClick={() => this.setState({ hasError: false })}
              className="px-4 py-2 bg-blue-600 text-white rounded"
            >
              Try again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
```

### Performance Monitoring

```typescript
// apps/web/lib/monitoring/performance.ts
import { onCLS, onFID, onFCP, onLCP, onTTFB } from 'web-vitals';

export function reportWebVitals() {
  onCLS((metric) => {
    console.log('CLS:', metric.value);
    // Send to analytics
  });

  onFID((metric) => {
    console.log('FID:', metric.value);
  });

  onFCP((metric) => {
    console.log('FCP:', metric.value);
  });

  onLCP((metric) => {
    console.log('LCP:', metric.value);
  });

  onTTFB((metric) => {
    console.log('TTFB:', metric.value);
  });
}
```

---

## Dashboard Setup

### Grafana Dashboard (JSON)

```json
{
  "dashboard": {
    "title": "WellOS API Metrics",
    "panels": [
      {
        "title": "Request Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total[5m])) by (status)"
          }
        ]
      },
      {
        "title": "P95 Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le, path))"
          }
        ]
      },
      {
        "title": "Error Rate",
        "targets": [
          {
            "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m])) / sum(rate(http_requests_total[5m]))"
          }
        ]
      },
      {
        "title": "Active Users",
        "targets": [
          {
            "expr": "active_users_count"
          }
        ]
      },
      {
        "title": "Cache Hit Rate",
        "targets": [
          {
            "expr": "cache_hit_rate"
          }
        ]
      },
      {
        "title": "Database Connections",
        "targets": [
          {
            "expr": "database_connections_active"
          }
        ]
      }
    ]
  }
}
```

---

## Best Practices

### ✅ DO

1. **Use structured logging** - Always log with context

   ```typescript
   logger.info('User action', { userId, action, resource });
   ```

2. **Include correlation IDs** - Track requests across services

   ```typescript
   const requestId = req.headers['x-request-id'] || uuidv4();
   ```

3. **Monitor the golden signals** - Latency, traffic, errors, saturation

   ```typescript
   // Latency: http_request_duration_seconds
   // Traffic: http_requests_total
   // Errors: http_requests_total{status=~"5.."}
   // Saturation: database_connections_active
   ```

4. **Set up alerts proactively** - Don't wait for users to report issues

   ```yaml
   - alert: HighErrorRate
     expr: error_rate > 0.05
     for: 5m
   ```

5. **Use log levels appropriately** - ERROR = requires action, INFO = business events

   ```typescript
   logger.error('Payment failed', { userId, amount, error });
   logger.info('Invoice created', { userId, invoiceId });
   ```

6. **Trace expensive operations** - Use custom spans for profiling

   ```typescript
   const span = tracer.startSpan('calculate-profitability');
   // ... expensive operation
   span.end();
   ```

7. **Monitor business metrics** - Not just technical metrics
   ```typescript
   metrics.incrementInvoicesGenerated(orgId);
   metrics.recordRevenueGenerated(amount);
   ```

---

### ❌ DON'T

1. **Don't log sensitive data** - Never log passwords, tokens, PII

   ```typescript
   // ❌ Bad
   logger.info('User login', { email, password });

   // ✅ Good
   logger.info('User login', { email });
   ```

2. **Don't over-alert** - Too many alerts = alert fatigue

   ```typescript
   // ❌ Bad: Alert on every 404
   // ✅ Good: Alert on high 5xx error rate
   ```

3. **Don't ignore log levels in production** - Set to INFO or WARN

   ```typescript
   // ❌ Bad: LOG_LEVEL=debug in production
   // ✅ Good: LOG_LEVEL=info in production
   ```

4. **Don't forget to aggregate logs** - Use centralized logging
   ```typescript
   // ✅ Good: Send to Elasticsearch, Datadog, CloudWatch
   ```

---

## Anti-Patterns

### 1. Console.log in Production

```typescript
// ❌ Anti-pattern
console.log('User created:', user);

// ✅ Solution
logger.info('User created', { userId: user.id, email: user.email });
```

### 2. Logging Inside Loops

```typescript
// ❌ Anti-pattern
for (const project of projects) {
  logger.info('Processing project', { projectId: project.id });
  // ...
}

// ✅ Solution
logger.info('Processing projects', { count: projects.length });
for (const project of projects) {
  // ...
}
logger.info('Projects processed', { count: projects.length });
```

### 3. Missing Error Context

```typescript
// ❌ Anti-pattern
catch (error) {
  logger.error(error.message);
}

// ✅ Solution
catch (error) {
  logger.error('Failed to create invoice', error.stack, {
    userId,
    projectId,
    amount,
  });
}
```

---

## Related Patterns

- **Pattern 13: Circuit Breaker Pattern** - Monitor external service failures
- **Pattern 45: Background Job Patterns** - Track job metrics and failures
- **Pattern 46: Caching Strategy Patterns** - Monitor cache hit rates
- **Pattern 41: REST API Best Practices** - HTTP status code monitoring

---

## References

### Documentation

- [Tracing (Rust)](https://docs.rs/tracing/)
- [Prometheus (Rust)](https://docs.rs/prometheus/)
- [OpenTelemetry (Rust)](https://docs.rs/opentelemetry/)
- [Sentry (Rust)](https://docs.rs/sentry/)
- [Grafana](https://grafana.com/docs/)
- [Jaeger](https://www.jaegertracing.io/docs/)

### Books & Articles

- **"Observability Engineering"** by Charity Majors - Modern observability practices
- **"Site Reliability Engineering"** by Google - SLIs, SLOs, error budgets
- **"The Art of Monitoring"** - Practical monitoring strategies

### Tools

- **Prometheus** - Metrics collection and alerting
- **Grafana** - Visualization and dashboards
- **Jaeger** - Distributed tracing
- **Sentry** - Error tracking and APM
- **Datadog** - All-in-one observability platform
- **New Relic** - APM and infrastructure monitoring

---

## Summary

**Monitoring & Observability Patterns** provide visibility into system health and user experience:

✅ **Log everything important** - Structured logging with context
✅ **Measure golden signals** - Latency, traffic, errors, saturation
✅ **Trace requests end-to-end** - Distributed tracing with OpenTelemetry
✅ **Alert on anomalies** - Proactive alerting based on thresholds
✅ **Track business metrics** - Not just technical metrics
✅ **Use APM tools** - Sentry, Datadog, New Relic for deep insights
✅ **Health checks** - Expose /health endpoint for monitoring

**Remember**: You can't fix what you can't see. Invest in observability early - it pays dividends when debugging production issues.

---

**Next Steps**:

1. Set up structured logging with tracing crate
2. Instrument code with prometheus crate metrics
3. Add health check endpoints
4. Configure Sentry for error tracking
5. Create Grafana dashboards
6. Set up alerts for critical thresholds
