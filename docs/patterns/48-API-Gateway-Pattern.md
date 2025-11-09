# Pattern 48: API Gateway Pattern

**Version**: 1.0
**Last Updated**: October 8, 2025
**Category**: Architecture & Integration

---

## Table of Contents

1. [Overview](#overview)
2. [When to Use](#when-to-use)
3. [Core Responsibilities](#core-responsibilities)
4. [Architecture](#architecture)
5. [Request Routing](#request-routing)
6. [Authentication & Authorization](#authentication--authorization)
7. [Rate Limiting](#rate-limiting)
8. [Request/Response Transformation](#requestresponse-transformation)
9. [Load Balancing](#load-balancing)
10. [API Versioning](#api-versioning)
11. [Caching](#caching)
12. [Error Handling](#error-handling)
13. [Implementation Options](#implementation-options)
14. [Best Practices](#best-practices)
15. [Anti-Patterns](#anti-patterns)
16. [Related Patterns](#related-patterns)
17. [References](#references)

---

## Overview

**API Gateway Pattern** provides a single entry point for all client requests to backend services. It acts as a reverse proxy, routing requests to appropriate microservices and handling cross-cutting concerns.

In WellOS's current monolithic architecture, an API Gateway may not be immediately necessary. However, as the application scales and potentially transitions to microservices, the gateway becomes critical for:

- **Unified API interface** - Single endpoint for frontend clients
- **Security enforcement** - Centralized authentication and authorization
- **Traffic management** - Rate limiting, throttling, circuit breaking
- **Protocol translation** - REST to gRPC, HTTP to WebSocket
- **API composition** - Aggregating data from multiple services

**Key Benefits**:

- 🔒 **Centralized security** - Single point for authentication/authorization
- 🚀 **Performance** - Caching, compression, load balancing
- 📊 **Observability** - Unified logging, metrics, tracing
- 🔄 **Flexibility** - Easy to add/remove backend services

---

## When to Use

### ✅ Use API Gateway When:

1. **Multiple backend services** - Microservices architecture
   - Time tracking service
   - Invoicing service
   - Project management service
   - QuickBooks integration service

2. **Different client types** - Web, mobile, third-party integrations
   - Web app needs full API
   - Mobile app needs lightweight responses
   - Partner integrations need specific data formats

3. **Cross-cutting concerns** - Authentication, logging, rate limiting
   - All requests need JWT validation
   - All endpoints need request logging
   - All public APIs need rate limiting

4. **Service aggregation** - Combine data from multiple services
   - Dashboard data from time tracking + projects + invoices
   - User profile from auth service + organization service

### ❌ Don't Use API Gateway When:

1. **Simple monolithic app** - Single backend service (WellOS's current state)
2. **Internal-only services** - No external clients
3. **Minimal traffic** - Over-engineering for small scale
4. **Tight coupling required** - Services need direct communication

---

## Core Responsibilities

### 1. Request Routing

Route incoming requests to appropriate backend services based on path, headers, or content.

```typescript
// Example routing logic
const routes = {
  '/api/auth/*': 'http://auth-service:4001',
  '/api/projects/*': 'http://project-service:4002',
  '/api/time-entries/*': 'http://time-tracking-service:4003',
  '/api/invoices/*': 'http://invoicing-service:4004',
  '/api/integrations/quickbooks/*': 'http://quickbooks-service:4005',
};
```

### 2. Authentication & Authorization

Validate tokens, enforce permissions, inject user context.

```typescript
// Verify JWT token
const user = await verifyToken(request.headers.authorization);

// Check permissions
if (!user.hasPermission('projects:read')) {
  throw new ForbiddenException();
}

// Inject user context for downstream services
request.headers['X-User-Id'] = user.id;
request.headers['X-Organization-Id'] = user.organizationId;
```

### 3. Rate Limiting

Prevent abuse by limiting requests per client.

```typescript
// Rate limit: 100 requests per minute per API key
const rateLimiter = new RateLimiter({
  windowMs: 60 * 1000,
  max: 100,
  keyGenerator: (req) => req.headers['x-api-key'],
});
```

### 4. Request/Response Transformation

Adapt requests and responses for different clients.

```typescript
// Transform response for mobile clients
if (request.headers['user-agent'].includes('Mobile')) {
  return {
    ...response,
    data: simplifyDataForMobile(response.data),
  };
}
```

### 5. Aggregation

Combine data from multiple services into a single response.

```typescript
// Dashboard endpoint aggregates data from multiple services
async getDashboard(userId: string) {
  const [projects, timeEntries, invoices] = await Promise.all([
    projectService.getProjects(userId),
    timeService.getTimeEntries(userId),
    invoiceService.getInvoices(userId),
  ]);

  return { projects, timeEntries, invoices };
}
```

### 6. Caching

Cache responses to reduce backend load.

```typescript
// Cache project list for 5 minutes
const cacheKey = `projects:${userId}`;
const cached = await cache.get(cacheKey);
if (cached) return cached;

const projects = await projectService.getProjects(userId);
await cache.set(cacheKey, projects, 300);
return projects;
```

### 7. Logging & Monitoring

Centralized logging, metrics, and distributed tracing.

```typescript
// Log all requests
logger.info('API Gateway request', {
  method: request.method,
  path: request.path,
  userId: request.user?.id,
  duration: Date.now() - startTime,
});

// Record metrics
metrics.recordRequest(request.method, request.path, response.status);
```

---

## Architecture

### API Gateway Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         Clients                              │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐       │
│  │ Web App │  │ Mobile  │  │ Partner │  │ Admin   │       │
│  │         │  │ App     │  │ API     │  │ Panel   │       │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘       │
└───────┼────────────┼────────────┼────────────┼─────────────┘
        │            │            │            │
        └────────────┴────────────┴────────────┘
                     │
                     ▼
        ┌────────────────────────────┐
        │      API Gateway           │
        │                            │
        │  ┌──────────────────────┐ │
        │  │ Authentication       │ │
        │  │ Authorization        │ │
        │  │ Rate Limiting        │ │
        │  │ Request Routing      │ │
        │  │ Response Caching     │ │
        │  │ Logging & Metrics    │ │
        │  │ Error Handling       │ │
        │  └──────────────────────┘ │
        └────────────┬───────────────┘
                     │
        ┌────────────┴────────────┐
        │                         │
        ▼                         ▼
┌───────────────┐         ┌───────────────┐
│ Load Balancer │         │ Load Balancer │
└───────┬───────┘         └───────┬───────┘
        │                         │
   ┌────┴────┐               ┌────┴────┐
   ▼         ▼               ▼         ▼
┌─────┐   ┌─────┐       ┌─────┐   ┌─────┐
│Auth │   │Auth │       │Time │   │Time │
│Svc  │   │Svc  │       │Svc  │   │Svc  │
│(1)  │   │(2)  │       │(1)  │   │(2)  │
└─────┘   └─────┘       └─────┘   └─────┘

┌─────────────────────────────────────────────┐
│         Shared Infrastructure               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │ Database │  │  Redis   │  │  Queue   │ │
│  └──────────┘  └──────────┘  └──────────┘ │
└─────────────────────────────────────────────┘
```

---

## Request Routing

### Path-Based Routing

```rust
// apps/gateway/src/routing/path_router.rs
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
};
use reqwest::Client;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RouteConfig {
    pub path: String,
    pub service: String,
    pub methods: Option<Vec<Method>>,
    pub strip_prefix: bool,
}

pub struct PathRouter {
    routes: Vec<RouteConfig>,
    http_client: Client,
}

impl PathRouter {
    pub fn new() -> Self {
        Self {
            routes: vec![
                RouteConfig {
                    path: "/api/v1/auth".to_string(),
                    service: "http://auth-service:4001".to_string(),
                    methods: None,
                    strip_prefix: false,
                },
                RouteConfig {
                    path: "/api/v1/projects".to_string(),
                    service: "http://project-service:4002".to_string(),
                    methods: None,
                    strip_prefix: false,
                },
                RouteConfig {
                    path: "/api/v1/time-entries".to_string(),
                    service: "http://time-service:4003".to_string(),
                    methods: None,
                    strip_prefix: false,
                },
                RouteConfig {
                    path: "/api/v1/invoices".to_string(),
                    service: "http://invoice-service:4004".to_string(),
                    methods: None,
                    strip_prefix: false,
                },
            ],
            http_client: Client::new(),
        }
    }

    pub async fn route(&self, request: Request) -> Result<Response, StatusCode> {
        let path = request.uri().path();
        let method = request.method();

        let route = self
            .find_route(path, method)
            .ok_or(StatusCode::NOT_FOUND)?;

        let target_url = self.build_target_url(&route, request.uri());

        // Forward request to backend service
        let response = self
            .http_client
            .request(method.clone(), &target_url)
            .headers(Self::forward_headers(request.headers()))
            .body(request.into_body())
            .send()
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)?;

        Ok(response.into())
    }

    fn find_route(&self, path: &str, method: &Method) -> Option<&RouteConfig> {
        self.routes.iter().find(|route| {
            let path_matches = path.starts_with(&route.path);
            let method_matches = route
                .methods
                .as_ref()
                .map(|methods| methods.contains(method))
                .unwrap_or(true);
            path_matches && method_matches
        })
    }

    fn build_target_url(&self, route: &RouteConfig, uri: &Uri) -> String {
        let path = if route.strip_prefix {
            uri.path().replace(&route.path, "")
        } else {
            uri.path().to_string()
        };

        let query = uri.query().map(|q| format!("?{}", q)).unwrap_or_default();

        format!("{}{}{}", route.service, path, query)
    }

    fn forward_headers(headers: &axum::http::HeaderMap) -> reqwest::header::HeaderMap {
        let mut forwarded = reqwest::header::HeaderMap::new();

        // Forward important headers
        let headers_to_forward = [
            "authorization",
            "content-type",
            "accept",
            "user-agent",
            "x-request-id",
        ];

        for header_name in &headers_to_forward {
            if let Some(value) = headers.get(*header_name) {
                if let Ok(value) = value.to_str() {
                    forwarded.insert(
                        reqwest::header::HeaderName::from_static(header_name),
                        value.parse().unwrap(),
                    );
                }
            }
        }

        forwarded
    }
}
```

### Header-Based Routing

```typescript
// Route based on API version header
const version = request.headers['x-api-version'];
const service = version === 'v2' ? 'http://projects-v2:4002' : 'http://projects-v1:4002';
```

### Canary Releases

```typescript
// Route 10% of traffic to new version
const routeToCanary = Math.random() < 0.1;
const service = routeToCanary ? 'http://projects-canary:4002' : 'http://projects-stable:4002';
```

---

## Authentication & Authorization

### JWT Validation

```rust
// apps/gateway/src/auth/auth_middleware.rs
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct JwtPayload {
    user_id: String,
    organization_id: String,
    email: String,
    role: String,
    exp: usize,
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = extract_token(req.headers())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let decoding_key = DecodingKey::from_secret(b"your-secret-key");
    let validation = Validation::default();

    let token_data = decode::<JwtPayload>(&token, &decoding_key, &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Inject user context into headers for downstream services
    let headers = req.headers_mut();
    headers.insert("x-user-id", token_data.claims.user_id.parse().unwrap());
    headers.insert("x-organization-id", token_data.claims.organization_id.parse().unwrap());
    headers.insert("x-user-email", token_data.claims.email.parse().unwrap());
    headers.insert("x-user-role", token_data.claims.role.parse().unwrap());

    Ok(next.run(req).await)
}

fn extract_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?.to_str().ok()?;
    if !auth_header.starts_with("Bearer ") {
        return None;
    }
    Some(auth_header[7..].to_string())
}
```

### Permission Checking

```rust
// apps/gateway/src/auth/permission_guard.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

pub struct PermissionGuard {
    required_permissions: Vec<String>,
}

impl PermissionGuard {
    pub fn new(permissions: Vec<String>) -> Self {
        Self {
            required_permissions: permissions,
        }
    }

    pub async fn check(
        &self,
        req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        if self.required_permissions.is_empty() {
            return Ok(next.run(req).await);
        }

        let user_permissions = req
            .headers()
            .get("x-user-permissions")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').map(String::from).collect::<Vec<_>>())
            .unwrap_or_default();

        let has_permissions = self
            .required_permissions
            .iter()
            .all(|perm| user_permissions.contains(perm));

        if has_permissions {
            Ok(next.run(req).await)
        } else {
            Err(StatusCode::FORBIDDEN)
        }
    }
}

// Usage with Axum
use axum::{routing::get, Router};

async fn get_projects() -> &'static str {
    "projects"
}

let app = Router::new()
    .route("/projects", get(get_projects))
    .layer(axum::middleware::from_fn(|req, next| {
        PermissionGuard::new(vec!["projects:read".to_string()])
            .check(req, next)
    }));
```

---

## Rate Limiting

### Token Bucket Algorithm

```rust
// apps/gateway/src/rate_limiting/token_bucket_service.rs
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct BucketConfig {
    pub capacity: f64,     // Max tokens
    pub refill_rate: f64,  // Tokens per second
}

#[derive(Debug, Serialize, Deserialize)]
struct BucketState {
    tokens: f64,
    last_refill: f64,
}

pub struct TokenBucketService {
    redis_client: redis::Client,
}

impl TokenBucketService {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis_client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn allow_request(&self, key: &str, config: &BucketConfig) -> Result<bool, redis::RedisError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let bucket_key = format!("rate-limit:{}", key);
        let mut conn = self.redis_client.get_async_connection().await?;

        // Get current bucket state
        let bucket_json: Option<String> = conn.get(&bucket_key).await?;
        let mut bucket = if let Some(json) = bucket_json {
            serde_json::from_str::<BucketState>(&json).unwrap_or(BucketState {
                tokens: config.capacity,
                last_refill: now,
            })
        } else {
            BucketState {
                tokens: config.capacity,
                last_refill: now,
            }
        };

        // Refill tokens based on time elapsed
        let time_passed = now - bucket.last_refill;
        let tokens_to_add = time_passed * config.refill_rate;
        bucket.tokens = f64::min(config.capacity, bucket.tokens + tokens_to_add);
        bucket.last_refill = now;

        // Check if we have tokens available
        if bucket.tokens < 1.0 {
            let _: () = conn.set_ex(&bucket_key, serde_json::to_string(&bucket).unwrap(), 3600).await?;
            return Ok(false); // Rate limit exceeded
        }

        // Consume one token
        bucket.tokens -= 1.0;
        let _: () = conn.set_ex(&bucket_key, serde_json::to_string(&bucket).unwrap(), 3600).await?;
        Ok(true) // Request allowed
    }

    pub async fn get_remaining_tokens(&self, key: &str, config: &BucketConfig) -> Result<f64, redis::RedisError> {
        let bucket_key = format!("rate-limit:{}", key);
        let mut conn = self.redis_client.get_async_connection().await?;

        let bucket_json: Option<String> = conn.get(&bucket_key).await?;
        if let Some(json) = bucket_json {
            let bucket: BucketState = serde_json::from_str(&json).unwrap();
            Ok(bucket.tokens)
        } else {
            Ok(config.capacity)
        }
    }
}
```

### Rate Limit Middleware

```rust
// apps/gateway/src/rate_limiting/rate_limit_middleware.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;

pub struct RateLimitMiddleware {
    token_bucket: Arc<TokenBucketService>,
}

impl RateLimitMiddleware {
    pub fn new(token_bucket: Arc<TokenBucketService>) -> Self {
        Self { token_bucket }
    }

    pub async fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let key = Self::get_rate_limit_key(&req);
        let config = Self::get_rate_limit_config(&req);

        let allowed = self.token_bucket.allow_request(&key, &config).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if !allowed {
            let remaining = self.token_bucket.get_remaining_tokens(&key, &config).await
                .unwrap_or(0.0);

            let retry_after = ((1.0 - remaining) / config.refill_rate).ceil() as u64;

            return Ok((
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!({
                    "statusCode": 429,
                    "message": "Too many requests",
                    "retryAfter": retry_after,
                })),
            ).into_response());
        }

        Ok(next.run(req).await)
    }

    fn get_rate_limit_key(req: &Request) -> String {
        // Rate limit by API key (if present) or IP address
        if let Some(api_key) = req.headers().get("x-api-key") {
            if let Ok(key) = api_key.to_str() {
                return format!("api-key:{}", key);
            }
        }

        // Fallback to IP address (would need ConnectInfo extractor in practice)
        format!("ip:unknown")
    }

    fn get_rate_limit_config(req: &Request) -> BucketConfig {
        let path = req.uri().path();

        // Different limits for different endpoints
        if path.starts_with("/api/v1/public") {
            BucketConfig {
                capacity: 100.0,
                refill_rate: 10.0,
            }
        } else if path.starts_with("/api/v1/auth") {
            BucketConfig {
                capacity: 20.0,
                refill_rate: 2.0,
            }
        } else {
            BucketConfig {
                capacity: 1000.0,
                refill_rate: 100.0,
            }
        }
    }
}
```

---

## Request/Response Transformation

### Request Transformation

```rust
// apps/gateway/src/transformation/request_transformer.rs
use axum::extract::Request;
use serde_json::{Value, Map};
use regex::Regex;

pub struct RequestTransformer;

impl RequestTransformer {
    pub fn transform(mut req: Request) -> Request {
        // Implementation would need to extract and modify body
        // This is a simplified version showing the pattern
        req
    }

    fn transform_keys(obj: Value, transformer: fn(&str) -> String) -> Value {
        match obj {
            Value::Array(arr) => {
                Value::Array(
                    arr.into_iter()
                        .map(|item| Self::transform_keys(item, transformer))
                        .collect()
                )
            }
            Value::Object(map) => {
                let mut new_map = Map::new();
                for (key, value) in map {
                    let transformed_key = transformer(&key);
                    new_map.insert(
                        transformed_key,
                        Self::transform_keys(value, transformer)
                    );
                }
                Value::Object(new_map)
            }
            _ => obj,
        }
    }

    fn to_camel_case(s: &str) -> String {
        let re = Regex::new(r"_([a-z])").unwrap();
        re.replace_all(s, |caps: &regex::Captures| {
            caps[1].to_uppercase()
        }).to_string()
    }

    fn to_snake_case(s: &str) -> String {
        let re = Regex::new(r"[A-Z]").unwrap();
        re.replace_all(s, |caps: &regex::Captures| {
            format!("_{}", caps[0].to_lowercase())
        }).to_string()
    }
}
```

### Response Transformation

```typescript
// apps/gateway/src/transformation/response-transformer.ts
export class ResponseTransformer {
  transform(response: any, request: Request): any {
    // Convert camelCase to snake_case for mobile clients
    if (this.isMobileClient(request)) {
      response = this.transformKeys(response, this.toSnakeCase);
    }

    // Wrap response in standard envelope
    return {
      success: true,
      data: response,
      meta: {
        timestamp: new Date().toISOString(),
        requestId: request.headers['x-request-id'],
      },
    };
  }

  private isMobileClient(request: Request): boolean {
    const userAgent = request.headers['user-agent'] || '';
    return /mobile|android|iphone|ipad/i.test(userAgent);
  }

  private transformKeys(obj: any, transformer: (key: string) => string): any {
    if (Array.isArray(obj)) {
      return obj.map((item) => this.transformKeys(item, transformer));
    }

    if (obj !== null && typeof obj === 'object') {
      return Object.keys(obj).reduce((result, key) => {
        const transformedKey = transformer(key);
        result[transformedKey] = this.transformKeys(obj[key], transformer);
        return result;
      }, {} as any);
    }

    return obj;
  }

  private toSnakeCase(str: string): string {
    return str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
  }
}
```

---

## Load Balancing

### Round-Robin Load Balancer

```typescript
// apps/gateway/src/load-balancing/round-robin.service.ts
export class RoundRobinLoadBalancer {
  private counters = new Map<string, number>();

  getNextInstance(service: string, instances: string[]): string {
    const counter = this.counters.get(service) || 0;
    const instance = instances[counter % instances.length];

    this.counters.set(service, counter + 1);
    return instance;
  }
}

// Usage
const instances = [
  'http://project-service-1:4002',
  'http://project-service-2:4002',
  'http://project-service-3:4002',
];

const target = loadBalancer.getNextInstance('project-service', instances);
```

### Least Connections Load Balancer

```typescript
// apps/gateway/src/load-balancing/least-connections.service.ts
export class LeastConnectionsLoadBalancer {
  private connections = new Map<string, number>();

  getNextInstance(instances: string[]): string {
    // Find instance with fewest active connections
    let minConnections = Infinity;
    let selectedInstance = instances[0];

    for (const instance of instances) {
      const connections = this.connections.get(instance) || 0;
      if (connections < minConnections) {
        minConnections = connections;
        selectedInstance = instance;
      }
    }

    // Increment connection count
    this.connections.set(selectedInstance, (this.connections.get(selectedInstance) || 0) + 1);

    return selectedInstance;
  }

  releaseConnection(instance: string) {
    const connections = this.connections.get(instance) || 0;
    this.connections.set(instance, Math.max(0, connections - 1));
  }
}
```

---

## API Versioning

### URL Path Versioning

```typescript
// /api/v1/projects -> Version 1
// /api/v2/projects -> Version 2

const routes = {
  '/api/v1/projects': 'http://projects-v1:4002',
  '/api/v2/projects': 'http://projects-v2:4002',
};
```

### Header Versioning

```typescript
// X-API-Version: 1
// X-API-Version: 2

const version = request.headers['x-api-version'] || '1';
const service = version === '2' ? 'http://projects-v2:4002' : 'http://projects-v1:4002';
```

### Content Negotiation

```typescript
// Accept: application/vnd.onwellos.v2+json

const acceptHeader = request.headers.accept;
const version = acceptHeader.match(/v(\d+)/)?.[1] || '1';
```

---

## Caching

### Response Caching

```rust
// apps/gateway/src/caching/response_cache_middleware.rs
use axum::{
    body::Body,
    extract::Request,
    http::{Method, HeaderValue},
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::sync::Arc;

pub struct ResponseCacheMiddleware {
    redis_client: redis::Client,
}

impl ResponseCacheMiddleware {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis_client: redis::Client::open(redis_url)?,
        })
    }

    pub async fn handle(
        &self,
        req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Only cache GET requests
        if req.method() != Method::GET {
            return Ok(next.run(req).await);
        }

        let cache_key = Self::get_cache_key(&req);
        let mut conn = self.redis_client
            .get_async_connection()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Check cache
        let cached: Option<String> = conn.get(&cache_key).await.ok().flatten();

        if let Some(cached_response) = cached {
            let mut response = Response::new(Body::from(cached_response));
            response.headers_mut().insert("x-cache", HeaderValue::from_static("HIT"));
            return Ok(response);
        }

        // Cache miss - proceed with request
        let mut response = next.run(req).await;
        response.headers_mut().insert("x-cache", HeaderValue::from_static("MISS"));

        // TODO: Cache response body (would need to buffer and clone body)
        // In practice, you'd extract the body, cache it, and return a new response

        Ok(response)
    }

    fn get_cache_key(req: &Request) -> String {
        let user_id = req.headers()
            .get("x-user-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("anonymous");

        let query = req.uri().query().unwrap_or("");

        format!("api-cache:{}:{}:{}", req.uri().path(), user_id, query)
    }

    fn get_ttl(path: &str) -> usize {
        // Different TTLs for different endpoints
        if path.contains("/projects") {
            300 // 5 minutes
        } else if path.contains("/users") {
            600 // 10 minutes
        } else if path.contains("/organizations") {
            1800 // 30 minutes
        } else {
            60 // Default 1 minute
        }
    }
}
```

---

## Error Handling

### Unified Error Response

```rust
// apps/gateway/src/errors/error_handler.rs
use axum::{
    extract::Request,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

pub struct GlobalErrorHandler;

impl GlobalErrorHandler {
    pub fn handle_error(
        req: &Request,
        exception: &dyn std::error::Error,
        status: StatusCode,
    ) -> Response {
        let error_code = Self::get_error_code(status);
        let message = exception.to_string();

        // Log error
        error!(
            path = req.uri().path(),
            method = ?req.method(),
            status = status.as_u16(),
            message = %message,
            "API Gateway error"
        );

        // Return standardized error response
        (
            status,
            Json(json!({
                "success": false,
                "error": {
                    "code": error_code,
                    "message": message,
                },
                "meta": {
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "requestId": req.headers()
                        .get("x-request-id")
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("unknown"),
                    "path": req.uri().path(),
                },
            })),
        ).into_response()
    }

    fn get_error_code(status: StatusCode) -> &'static str {
        match status {
            StatusCode::BAD_REQUEST => "BAD_REQUEST",
            StatusCode::UNAUTHORIZED => "UNAUTHORIZED",
            StatusCode::FORBIDDEN => "FORBIDDEN",
            StatusCode::NOT_FOUND => "NOT_FOUND",
            StatusCode::TOO_MANY_REQUESTS => "RATE_LIMIT_EXCEEDED",
            StatusCode::INTERNAL_SERVER_ERROR => "INTERNAL_ERROR",
            StatusCode::BAD_GATEWAY => "BAD_GATEWAY",
            StatusCode::SERVICE_UNAVAILABLE => "SERVICE_UNAVAILABLE",
            _ => "UNKNOWN_ERROR",
        }
    }
}
```

### Circuit Breaker Integration

```rust
// apps/gateway/src/resilience/circuit_breaker_service.rs
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

pub struct CircuitBreakerService {
    state: Arc<Mutex<CircuitState>>,
    failure_count: Arc<Mutex<u32>>,
    success_count: Arc<Mutex<u32>>,
    last_failure_time: Arc<Mutex<Option<SystemTime>>>,
    threshold: u32,
    timeout: Duration,
    success_threshold: u32,
}

impl CircuitBreakerService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitState::Closed)),
            failure_count: Arc::new(Mutex::new(0)),
            success_count: Arc::new(Mutex::new(0)),
            last_failure_time: Arc::new(Mutex::new(None)),
            threshold: 5,
            timeout: Duration::from_secs(60),
            success_threshold: 2,
        }
    }

    pub async fn execute<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        let state = *self.state.lock().unwrap();

        if state == CircuitState::Open {
            let last_failure = *self.last_failure_time.lock().unwrap();
            if let Some(last_failure_time) = last_failure {
                if SystemTime::now().duration_since(last_failure_time).unwrap() > self.timeout {
                    *self.state.lock().unwrap() = CircuitState::HalfOpen;
                } else {
                    return Err(CircuitBreakerError::Open);
                }
            }
        }

        match f.await {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(error) => {
                self.on_failure();
                Err(CircuitBreakerError::RequestFailed(error))
            }
        }
    }

    fn on_success(&self) {
        *self.failure_count.lock().unwrap() = 0;

        let state = *self.state.lock().unwrap();
        if state == CircuitState::HalfOpen {
            let mut success_count = self.success_count.lock().unwrap();
            *success_count += 1;
            if *success_count >= self.success_threshold {
                *self.state.lock().unwrap() = CircuitState::Closed;
                *success_count = 0;
            }
        }
    }

    fn on_failure(&self) {
        let mut failure_count = self.failure_count.lock().unwrap();
        *failure_count += 1;
        *self.last_failure_time.lock().unwrap() = Some(SystemTime::now());
        *self.success_count.lock().unwrap() = 0;

        if *failure_count >= self.threshold {
            *self.state.lock().unwrap() = CircuitState::Open;
        }
    }

    pub fn get_state(&self) -> CircuitState {
        *self.state.lock().unwrap()
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    Open,
    RequestFailed(E),
}
```

---

## Implementation Options

### 1. Custom Rust Gateway (Axum)

**Pros**:

- ✅ Full control over logic
- ✅ Type safety and memory safety
- ✅ High performance and low resource usage

**Cons**:

- ❌ More development effort
- ❌ Steeper learning curve

```rust
// apps/gateway/src/main.rs
use axum::{routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(/* middleware layers */);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Gateway listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### 2. Kong Gateway

**Pros**:

- ✅ Production-ready
- ✅ Extensive plugin ecosystem
- ✅ Kubernetes-native

**Cons**:

- ❌ Additional infrastructure
- ❌ Learning curve

```yaml
# kong.yml
services:
  - name: project-service
    url: http://project-service:4002
    routes:
      - name: projects-route
        paths:
          - /api/v1/projects
    plugins:
      - name: jwt
      - name: rate-limiting
        config:
          minute: 100
```

### 3. AWS API Gateway

**Pros**:

- ✅ Fully managed
- ✅ Auto-scaling
- ✅ AWS ecosystem integration

**Cons**:

- ❌ Vendor lock-in
- ❌ Cost for high traffic

### 4. NGINX

**Pros**:

- ✅ Lightweight
- ✅ High performance
- ✅ Battle-tested

**Cons**:

- ❌ Limited programmability
- ❌ Requires Lua for complex logic

```nginx
# nginx.conf
http {
  upstream project_service {
    server project-service-1:4002;
    server project-service-2:4002;
  }

  server {
    location /api/v1/projects {
      proxy_pass http://project_service;
      proxy_set_header X-User-Id $http_x_user_id;
    }
  }
}
```

---

## Best Practices

### ✅ DO

1. **Implement health checks** - Gateway and downstream services

   ```typescript
   @Get('/health')
   async health() {
     return { status: 'ok', timestamp: new Date() };
   }
   ```

2. **Use circuit breakers** - Prevent cascading failures

   ```typescript
   await circuitBreaker.execute(() => projectService.getProjects());
   ```

3. **Add request timeouts** - Don't wait forever

   ```typescript
   const response = await axios.get(url, { timeout: 5000 });
   ```

4. **Log all requests** - Centralized observability

   ```typescript
   logger.info('Gateway request', { method, path, duration });
   ```

5. **Version your APIs** - Use /api/v1, /api/v2 prefixes

   ```typescript
   '/api/v1/projects' vs '/api/v2/projects'
   ```

6. **Cache aggressively** - Reduce backend load

   ```typescript
   if (method === 'GET') cache.set(key, response, ttl);
   ```

7. **Rate limit by client** - API key or IP address
   ```typescript
   const key = req.headers['x-api-key'] || req.ip;
   ```

---

### ❌ DON'T

1. **Don't add business logic** - Gateway should be thin

   ```typescript
   // ❌ Bad: Business logic in gateway
   if (project.budget > 100000) {
     /* ... */
   }

   // ✅ Good: Delegate to service
   await projectService.validateBudget(project);
   ```

2. **Don't store state** - Gateway should be stateless

   ```typescript
   // ❌ Bad: In-memory state
   this.sessions.set(userId, session);

   // ✅ Good: Store in Redis
   await cache.set(`session:${userId}`, session);
   ```

3. **Don't ignore errors** - Always handle gracefully

   ```typescript
   // ❌ Bad: Swallow errors
   try { await service.call(); } catch {}

   // ✅ Good: Handle and log
   try { await service.call(); }
   catch (error) { logger.error('Service error', error); throw; }
   ```

---

## Anti-Patterns

### 1. God Gateway

**Problem**: Gateway does too much (business logic, data transformations, complex aggregations).

```typescript
// ❌ Anti-pattern: Too much logic in gateway
async getDashboard() {
  const projects = await projectService.getAll();
  const filtered = projects.filter(p => p.status === 'active');
  const sorted = filtered.sort((a, b) => b.priority - a.priority);
  const enriched = sorted.map(p => ({ ...p, total: calculateTotal(p) }));
  return enriched;
}

// ✅ Solution: Delegate to services
async getDashboard() {
  return projectService.getDashboard(); // Service handles logic
}
```

### 2. Tight Coupling

**Problem**: Gateway knows too much about backend services.

```typescript
// ❌ Anti-pattern: Gateway knows service internals
const url = `${projectService}/internal/database/projects?table=projects`;

// ✅ Solution: Use service interfaces
const url = `${projectService}/api/projects`;
```

---

## Related Patterns

- **Pattern 13: Circuit Breaker Pattern** - Handle service failures
- **Pattern 46: Caching Strategy Patterns** - Cache responses at gateway
- **Pattern 47: Monitoring & Observability Patterns** - Gateway metrics
- **Pattern 41: REST API Best Practices** - API design for gateway

---

## References

### Documentation

- [Kong Gateway](https://docs.konghq.com/)
- [AWS API Gateway](https://aws.amazon.com/api-gateway/)
- [NGINX](https://nginx.org/en/docs/)
- [Envoy Proxy](https://www.envoyproxy.io/docs/)

### Books & Articles

- **"Building Microservices"** by Sam Newman - Gateway patterns
- **"Microservices Patterns"** by Chris Richardson - API Gateway pattern
- **"Release It!"** by Michael Nygard - Stability patterns (circuit breaker)

### Tools

- **Kong** - Open-source API Gateway
- **Tyk** - Open-source API Gateway
- **AWS API Gateway** - Managed service
- **Google Cloud API Gateway** - Managed service
- **Azure API Management** - Managed service

---

## Summary

**API Gateway Pattern** provides a unified entry point for client requests:

✅ **Single entry point** - All clients go through gateway
✅ **Cross-cutting concerns** - Auth, rate limiting, logging, caching
✅ **Service aggregation** - Combine data from multiple services
✅ **Protocol translation** - REST, gRPC, WebSocket
✅ **Load balancing** - Distribute traffic across instances
✅ **API versioning** - Support multiple API versions
✅ **Circuit breaking** - Prevent cascading failures

**Remember**: API Gateway is essential for microservices but may be over-engineering for simple monoliths. Start simple, add gateway when complexity justifies it.

---

**Next Steps**:

1. Evaluate if API Gateway is needed for your architecture
2. Choose implementation (custom, Kong, AWS, NGINX)
3. Implement authentication and rate limiting
4. Add health checks and circuit breakers
5. Set up monitoring and logging
6. Test failover and load balancing scenarios
