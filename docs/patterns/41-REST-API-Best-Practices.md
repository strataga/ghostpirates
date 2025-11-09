# Pattern 41: REST API Best Practices & Design Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Status**: Active

---

## Table of Contents

1. [Overview](#overview)
2. [URL Design & Resource Naming](#url-design--resource-naming)
3. [HTTP Methods & Status Codes](#http-methods--status-codes)
4. [Request & Response Patterns](#request--response-patterns)
5. [Versioning Strategies](#versioning-strategies)
6. [Error Handling](#error-handling)
7. [Security Patterns](#security-patterns)
8. [Performance Patterns](#performance-patterns)
9. [Documentation Standards](#documentation-standards)
10. [Rust Implementation](#rust-implementation)
11. [Testing REST APIs](#testing-rest-apis)

---

## Overview

This document defines REST API design best practices for the WellOS project, ensuring consistency, maintainability, and excellent developer experience across all endpoints.

### Core Principles

1. **Resource-Oriented**: APIs should model business resources, not operations
2. **Consistent**: Predictable patterns across all endpoints
3. **Self-Documenting**: Clear, descriptive URLs and responses
4. **Secure**: Authentication, authorization, input validation
5. **Versioned**: Backward compatibility and evolution

### RESTful Maturity Model

**Level 0 - The Swamp of POX**: Single endpoint, single HTTP method
**Level 1 - Resources**: Multiple URIs, single HTTP method
**Level 2 - HTTP Verbs**: Multiple URIs, multiple HTTP methods ✅ **Our Target**
**Level 3 - HATEOAS**: Hypermedia controls (optional for internal APIs)

---

## URL Design & Resource Naming

### 1. Base URL Structure

```
https://api.example.com/api/v1/{resource}
```

**WellOS Standard**:

```
http://localhost:4001/api/v1/{resource}
```

### 2. Resource Naming Conventions

#### ✅ DO

```
GET    /api/v1/organizations              # Collection
GET    /api/v1/organizations/{id}         # Single resource
POST   /api/v1/organizations              # Create
PUT    /api/v1/organizations/{id}         # Full update
PATCH  /api/v1/organizations/{id}         # Partial update
DELETE /api/v1/organizations/{id}         # Delete
```

**Rules**:

- Use **plural nouns** for collections (`/users`, `/organizations`, `/projects`)
- Use **kebab-case** for multi-word resources (`/pending-users`, `/time-entries`)
- Nest sub-resources when there's a clear parent-child relationship
- Limit nesting to 2 levels for readability

#### ❌ DON'T

```
GET /api/v1/getOrganizations        # Verb in URL
GET /api/v1/organization            # Singular noun
GET /api/v1/Organizations           # PascalCase
GET /api/v1/org                     # Abbreviation
```

### 3. Sub-Resource Patterns

```
GET    /api/v1/organizations/{id}/users           # Nested collection
POST   /api/v1/organizations/{id}/users           # Add user to org
DELETE /api/v1/organizations/{id}/users/{userId}  # Remove user from org
```

**When to Nest**:

- ✅ When the sub-resource ALWAYS belongs to the parent
  - `/organizations/{id}/users` (users always belong to an org)
- ❌ When the sub-resource can exist independently
  - `/users/{id}/organizations` (users can belong to multiple orgs - use query params instead)

### 4. Action Endpoints (When Needed)

Sometimes resources need actions beyond CRUD:

```
POST /api/v1/organizations/{id}/transfer-ownership
POST /api/v1/organizations/{id}/whitelist-domain
POST /api/v1/pending-users/{id}/approve
POST /api/v1/pending-users/{id}/reject
POST /api/v1/invoices/{id}/send
POST /api/v1/projects/{id}/archive
```

**Rules**:

- Use POST for non-idempotent actions
- Use verb phrases after the resource ID
- Keep action names descriptive and business-focused

### 5. Query Parameters

```
GET /api/v1/users?role=ORG_OWNER&status=active&limit=20&offset=0
GET /api/v1/pending-users?status=PENDING&organizationId={id}
GET /api/v1/organizations/{id}/users?role=ADMIN&page=2&pageSize=25
```

**Standard Query Params**:

- **Filtering**: `?status=active&role=ADMIN`
- **Sorting**: `?sortBy=createdAt&order=desc`
- **Pagination**: `?page=2&pageSize=25` or `?limit=25&offset=50`
- **Search**: `?q=searchterm`
- **Field Selection**: `?fields=id,name,email` (sparse fieldsets)

---

## HTTP Methods & Status Codes

### 1. HTTP Method Semantics

| Method     | Purpose                 | Safe? | Idempotent? | Request Body? | Response Body? |
| ---------- | ----------------------- | ----- | ----------- | ------------- | -------------- |
| **GET**    | Retrieve resource(s)    | ✅    | ✅          | ❌            | ✅             |
| **POST**   | Create resource         | ❌    | ❌          | ✅            | ✅             |
| **PUT**    | Replace entire resource | ❌    | ✅          | ✅            | ✅             |
| **PATCH**  | Partial update          | ❌    | ❌          | ✅            | ✅             |
| **DELETE** | Remove resource         | ❌    | ✅          | ❌            | Optional       |

**Safe**: Does not modify server state
**Idempotent**: Multiple identical requests have the same effect as a single request

### 2. Status Code Usage

#### 2xx Success

```rust
200 OK                    // GET, PUT, PATCH successful with body
201 Created               // POST successful, resource created
204 No Content            // DELETE successful, no response body
```

**Example**:

```rust
use axum::{http::StatusCode, Json};

async fn create(
    Json(dto): Json<CreateOrganizationDto>
) -> Result<(StatusCode, Json<Organization>), AppError> {
    let organization = command_bus.execute(CreateOrganizationCommand::new(...)).await?;
    Ok((StatusCode::CREATED, Json(organization)))
}

async fn delete(Path(id): Path<String>) -> Result<StatusCode, AppError> {
    command_bus.execute(DeleteOrganizationCommand::new(id)).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

#### 3xx Redirection

```rust
301 Moved Permanently     // Resource permanently moved
302 Found                 // Temporary redirect
304 Not Modified          // Cached resource still valid
```

#### 4xx Client Errors

```rust
400 Bad Request          // Invalid input, validation failed
401 Unauthorized         // Missing or invalid authentication
403 Forbidden            // Authenticated but not authorized
404 Not Found            // Resource doesn't exist
405 Method Not Allowed   // HTTP method not supported for endpoint
409 Conflict             // Resource conflict (e.g., duplicate email)
422 Unprocessable Entity // Semantic errors (valid syntax, invalid logic)
429 Too Many Requests    // Rate limit exceeded
```

**Example Error Responses**:

```json
{
  "statusCode": 400,
  "message": ["email must be a valid email", "password is too short"],
  "error": "Bad Request"
}

{
  "statusCode": 409,
  "message": "Organization with domain 'acme.com' already exists",
  "error": "Conflict"
}

{
  "statusCode": 403,
  "message": "Only organization owner can transfer ownership",
  "error": "Forbidden"
}
```

#### 5xx Server Errors

```rust
500 Internal Server Error  // Unexpected server error
501 Not Implemented        // Feature not yet available
503 Service Unavailable    // Temporary unavailability (maintenance, overload)
```

---

## Request & Response Patterns

### 1. Request Body Validation

**Use Serde with validator**:

```rust
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrganizationDto {
    /// Organization name (2-100 characters)
    #[validate(length(min = 2, max = 100, message = "Name must be between 2 and 100 characters"))]
    #[serde(deserialize_with = "trim_string")]
    pub name: String,

    /// Primary email domain (e.g., acme.com)
    #[validate(regex(path = "DOMAIN_REGEX", message = "Invalid domain format"))]
    #[serde(deserialize_with = "trim_lowercase_string")]
    pub primary_domain: String,

    /// Optional organization settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OrganizationSettings>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OrganizationSettings {
    pub time_approval_required: Option<bool>,
    pub client_portal_enabled: Option<bool>,
    pub default_hourly_rate: Option<f64>,
}

lazy_static! {
    static ref DOMAIN_REGEX: Regex = Regex::new(
        r"^[a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,}$"
    ).unwrap();
}

fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

fn trim_lowercase_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}
```

### 2. Response Body Standards

**Consistent Response Structure**:

```typescript
// Single Resource
{
  "id": "uuid",
  "name": "Acme Corporation",
  "slug": "acme-corporation",
  "primaryDomain": "acme.com",
  "ownerId": "uuid",
  "createdAt": "2025-10-08T10:00:00Z",
  "updatedAt": "2025-10-08T10:00:00Z"
}

// Collection
{
  "data": [
    { "id": "uuid", "name": "Org 1", ... },
    { "id": "uuid", "name": "Org 2", ... }
  ],
  "meta": {
    "total": 42,
    "page": 1,
    "pageSize": 25,
    "totalPages": 2
  }
}

// Operation Result
{
  "success": true,
  "message": "Ownership transferred successfully",
  "previousOwnerId": "uuid",
  "newOwnerId": "uuid"
}
```

**Response DTO Pattern**:

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrganizationResponseDto {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub primary_domain: String,
    pub owner_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub billing: Option<BillingInfoDto>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OrganizationSettingsDto>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
}
```

### 3. Timestamp Standards

**Always use ISO 8601 format**:

```json
{
  "createdAt": "2025-10-08T14:30:00.000Z",
  "updatedAt": "2025-10-08T15:45:00.000Z",
  "expiresAt": "2025-10-15T14:30:00.000Z"
}
```

**Never**:

```json
{
  "createdAt": "10/08/2025", // Ambiguous
  "createdAt": 1728393000000, // Unix timestamp (not human-readable)
  "createdAt": "2025-10-08 14:30" // Missing timezone
}
```

---

## Versioning Strategies

### 1. URL Path Versioning (WellOS Standard)

```
/api/v1/organizations
/api/v2/organizations
```

**Pros**:

- ✅ Clear and visible
- ✅ Easy to route
- ✅ Works with all clients
- ✅ Cacheable

**Implementation**:

```rust
use axum::{Router, routing::get};

pub fn api_v1_routes() -> Router {
    Router::new()
        .route("/api/v1/organizations", get(list_organizations_v1).post(create_organization_v1))
        .route("/api/v1/organizations/:id", get(get_organization_v1).put(update_organization_v1).delete(delete_organization_v1))
}

pub fn api_v2_routes() -> Router {
    Router::new()
        .route("/api/v2/organizations", get(list_organizations_v2).post(create_organization_v2))
        .route("/api/v2/organizations/:id", get(get_organization_v2).put(update_organization_v2).delete(delete_organization_v2))
}
```

### 2. Alternative Versioning Strategies

**Header Versioning**:

```
GET /api/organizations
Accept: application/vnd.company.v1+json
```

**Query Parameter Versioning**:

```
GET /api/organizations?version=1
```

**Media Type Versioning**:

```
Content-Type: application/vnd.company.organization.v1+json
```

### 3. Versioning Best Practices

1. **Version only when breaking changes occur**
   - ✅ Breaking: Removing fields, changing field types, changing URL structure
   - ❌ Non-breaking: Adding optional fields, adding new endpoints

2. **Maintain at least N-1 versions** (current + previous)

3. **Deprecation Policy**:
   ```rust
   use axum::{
       http::HeaderMap,
       response::{IntoResponse, Response},
   };

   async fn list_organizations_v1() -> Response {
       let mut headers = HeaderMap::new();
       headers.insert(
           "X-API-Warn",
           "This API version will be removed on 2026-01-01. Please upgrade to v2.".parse().unwrap()
       );

       let organizations = get_organizations().await;
       (headers, Json(organizations)).into_response()
   }
   ```

---

## Error Handling

### 1. Standard Error Response Format

```typescript
{
  "statusCode": 400,
  "message": "Validation failed",
  "errors": [
    {
      "field": "email",
      "message": "Invalid email format",
      "value": "not-an-email"
    },
    {
      "field": "password",
      "message": "Password must be at least 8 characters",
      "value": "***"
    }
  ],
  "error": "Bad Request",
  "timestamp": "2025-10-08T14:30:00.000Z",
  "path": "/api/v1/auth/register"
}
```

### 2. Custom Error Handler

```rust
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use chrono::Utc;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    status_code: u16,
    message: String,
    error: String,
    timestamp: String,
    path: String,
}

pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    UnprocessableEntity(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, error_type) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg, "Bad Request"),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg, "Unauthorized"),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg, "Forbidden"),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg, "Not Found"),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg, "Conflict"),
            AppError::UnprocessableEntity(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg, "Unprocessable Entity"),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, "Internal Server Error"),
        };

        let error_response = ErrorResponse {
            status_code: status.as_u16(),
            message,
            error: error_type.to_string(),
            timestamp: Utc::now().to_rfc3339(),
            path: String::new(), // Populated by middleware
        };

        (status, Json(error_response)).into_response()
    }
}
```

### 3. Domain-Specific Errors

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrganizationError {
    #[error("Organization with ID '{0}' not found")]
    NotFound(String),

    #[error("Organization with domain '{0}' already exists")]
    DuplicateDomain(String),

    #[error("You do not have permission to access this organization")]
    UnauthorizedAccess,
}

impl From<OrganizationError> for AppError {
    fn from(err: OrganizationError) -> Self {
        match err {
            OrganizationError::NotFound(msg) => AppError::NotFound(msg),
            OrganizationError::DuplicateDomain(msg) => AppError::Conflict(msg),
            OrganizationError::UnauthorizedAccess => AppError::Forbidden(err.to_string()),
        }
    }
}
```

### 4. Validation Error Handling

```rust
use axum::{
    extract::Json,
    response::IntoResponse,
};
use validator::Validate;

pub async fn validate_request<T: Validate>(
    Json(payload): Json<T>
) -> Result<Json<T>, AppError> {
    payload.validate().map_err(|errors| {
        let error_messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errs)| {
                errs.iter().map(move |err| {
                    format!("{}: {}", field, err.message.as_ref().unwrap_or(&"validation error".into()))
                })
            })
            .collect();

        AppError::BadRequest(format!("Validation failed: {}", error_messages.join(", ")))
    })?;

    Ok(Json(payload))
}

// Usage in handler
async fn create_organization(
    Json(dto): Json<CreateOrganizationDto>
) -> Result<(StatusCode, Json<Organization>), AppError> {
    dto.validate().map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    // Process request...
    Ok((StatusCode::CREATED, Json(organization)))
}
```

---

## Security Patterns

### 1. Authentication

```rust
use axum::{
    extract::Extension,
    middleware,
    Router,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub organization_id: String,
}

pub async fn jwt_auth_middleware(
    headers: HeaderMap,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::Unauthorized("Missing token".to_string()))?;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

    let user = CurrentUser {
        user_id: token_data.claims.sub,
        email: token_data.claims.email,
        role: token_data.claims.role,
        organization_id: token_data.claims.organization_id,
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}

// Usage in handler
async fn create_organization(
    Extension(user): Extension<CurrentUser>,
    Json(dto): Json<CreateOrganizationDto>,
) -> Result<(StatusCode, Json<Organization>), AppError> {
    // user.user_id, user.email, user.role, user.organization_id available
    Ok((StatusCode::CREATED, Json(organization)))
}
```

### 2. Authorization

```rust
use axum::{
    extract::{Path, Extension},
    http::StatusCode,
    Json,
};

#[derive(Serialize)]
struct TransferOwnershipResponse {
    success: bool,
    message: String,
    previous_owner_id: String,
    new_owner_id: String,
}

async fn transfer_ownership(
    Path(organization_id): Path<String>,
    Extension(user): Extension<CurrentUser>,
    Extension(tenant): Extension<TenantContext>,
    Json(dto): Json<TransferOwnershipDto>,
) -> Result<Json<TransferOwnershipResponse>, AppError> {
    // Verify user is updating their own organization
    if organization_id != tenant.organization_id {
        return Err(AppError::Forbidden(
            "Cannot transfer ownership of another organization".to_string()
        ));
    }

    // Only ORG_OWNER can transfer ownership
    if user.role != "ORG_OWNER" {
        return Err(AppError::Forbidden(
            "Only organization owner can transfer ownership".to_string()
        ));
    }

    let command = TransferOwnershipCommand {
        organization_id: organization_id.clone(),
        user_id: user.user_id.clone(),
        new_owner_id: dto.new_owner_id.clone(),
    };

    command_bus.execute(command).await?;

    Ok(Json(TransferOwnershipResponse {
        success: true,
        message: "Ownership transferred successfully".to_string(),
        previous_owner_id: user.user_id,
        new_owner_id: dto.new_owner_id,
    }))
}
```

### 3. Input Validation & Sanitization

```rust
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePendingUserDto {
    #[validate(email(message = "Invalid email format"))]
    #[serde(deserialize_with = "trim_lowercase_string")]
    pub email: String,

    #[validate(length(min = 1, max = 50, message = "First name must be between 1 and 50 characters"))]
    #[serde(deserialize_with = "trim_string")]
    pub first_name: String,
}

fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

fn trim_lowercase_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}
```

### 4. Rate Limiting

```rust
use tower_governor::{
    governor::GovernorConfigBuilder,
    GovernorLayer,
};
use std::time::Duration;

pub fn create_rate_limiter() -> GovernorLayer {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .finish()
            .unwrap(),
    );

    GovernorLayer {
        config: Box::leak(governor_conf),
    }
}

// Apply to router
let app = Router::new()
    .route("/api/v1/auth/login", post(login))
    .layer(create_rate_limiter());

async fn login(
    Json(dto): Json<LoginDto>,
) -> Result<Json<LoginResponse>, AppError> {
    // Limited to configured requests per time window
    Ok(Json(response))
}
```

---

## Performance Patterns

### 1. Pagination

```rust
use serde::{Deserialize, Serialize};
use axum::extract::Query;

#[derive(Debug, Deserialize, Validate)]
pub struct PaginationQuery {
    #[validate(range(min = 1, message = "Page must be at least 1"))]
    #[serde(default = "default_page")]
    pub page: u32,

    #[validate(range(min = 1, max = 100, message = "Page size must be between 1 and 100"))]
    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_page() -> u32 { 1 }
fn default_page_size() -> u32 { 25 }

#[derive(Serialize)]
struct PaginatedResponse<T> {
    data: Vec<T>,
    meta: PaginationMeta,
}

#[derive(Serialize)]
struct PaginationMeta {
    page: u32,
    page_size: u32,
    total: u64,
    total_pages: u32,
}

async fn find_all(
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<Organization>>, AppError> {
    pagination.validate().map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    let skip = (pagination.page - 1) * pagination.page_size;
    let organizations = query_bus.execute(
        GetOrganizationsQuery::new(skip, pagination.page_size)
    ).await?;

    Ok(Json(PaginatedResponse {
        data: organizations.data,
        meta: PaginationMeta {
            page: pagination.page,
            page_size: pagination.page_size,
            total: organizations.total,
            total_pages: (organizations.total as f64 / pagination.page_size as f64).ceil() as u32,
        },
    }))
}
```

### 2. Field Selection (Sparse Fieldsets)

```
GET /api/v1/organizations?fields=id,name,slug
```

```rust
use axum::extract::Query;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FieldsQuery {
    pub fields: Option<String>,
}

async fn find_all(
    Query(query): Query<FieldsQuery>,
) -> Result<Json<Vec<Organization>>, AppError> {
    let selected_fields = query.fields
        .map(|f| f.split(',').map(String::from).collect::<Vec<_>>());

    let organizations = query_bus.execute(
        GetOrganizationsQuery::with_fields(selected_fields)
    ).await?;

    Ok(Json(organizations))
}
```

### 3. Filtering & Sorting

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct OrganizationQuery {
    #[serde(flatten)]
    pub pagination: PaginationQuery,

    #[serde(default)]
    pub status: Option<OrganizationStatus>,

    #[serde(default = "default_sort_by")]
    pub sort_by: SortField,

    #[serde(default = "default_order")]
    pub order: SortOrder,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationStatus {
    Active,
    Inactive,
    All,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    Name,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

fn default_sort_by() -> SortField { SortField::CreatedAt }
fn default_order() -> SortOrder { SortOrder::Desc }
```

### 4. ETag & Conditional Requests

```rust
use axum::{
    http::{header, StatusCode, HeaderMap},
    extract::Path,
    response::{IntoResponse, Response},
};

async fn get_by_id(
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, AppError> {
    let organization = query_bus.execute(
        GetOrganizationByIdQuery::new(id)
    ).await?;

    let etag = format!("\"{}\"", organization.updated_at.timestamp_millis());

    // Check If-None-Match header
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.to_str().unwrap_or("") == etag {
            return Ok(StatusCode::NOT_MODIFIED.into_response());
        }
    }

    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::ETAG, etag.parse().unwrap());
    response_headers.insert(
        header::CACHE_CONTROL,
        "private, max-age=60".parse().unwrap()
    );

    Ok((response_headers, Json(organization)).into_response())
}
```

---

## Documentation Standards

### 1. OpenAPI/Utoipa Documentation

```rust
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_organization,
        get_organization,
        list_organizations,
    ),
    components(
        schemas(OrganizationResponseDto, CreateOrganizationDto)
    ),
    tags(
        (name = "organizations", description = "Organization management endpoints")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;

/// Create a new organization
///
/// Manually create a new organization. Regular users auto-create organizations during registration.
#[utoipa::path(
    post,
    path = "/api/v1/organizations",
    request_body = CreateOrganizationDto,
    responses(
        (status = 201, description = "Organization successfully created", body = OrganizationResponseDto),
        (status = 400, description = "Invalid input data"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 403, description = "Forbidden - Insufficient permissions"),
        (status = 409, description = "Organization with this domain already exists"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
async fn create_organization(
    Extension(user): Extension<CurrentUser>,
    Json(dto): Json<CreateOrganizationDto>,
) -> Result<(StatusCode, Json<OrganizationResponseDto>), AppError> {
    // Implementation
    Ok((StatusCode::CREATED, Json(organization)))
}
```

### 2. DTO Documentation

```rust
use utoipa::ToSchema;

/// Request body for creating a new organization
#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateOrganizationDto {
    /// Organization name (2-100 characters)
    ///
    /// # Example
    /// ```
    /// "Acme Corporation"
    /// ```
    #[schema(example = "Acme Corporation", min_length = 2, max_length = 100)]
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    /// Primary email domain for the organization
    ///
    /// # Example
    /// ```
    /// "acme.com"
    /// ```
    #[schema(example = "acme.com", pattern = "^[a-z0-9]+([\\-\\.]{1}[a-z0-9]+)*\\.[a-z]{2,}$")]
    #[validate(regex(path = "DOMAIN_REGEX"))]
    pub primary_domain: String,

    /// Optional organization settings
    #[schema(example = json!({
        "timeApprovalRequired": true,
        "clientPortalEnabled": false,
        "defaultHourlyRate": 150
    }))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<OrganizationSettingsDto>,
}
```

---

## Rust Implementation

### 1. Router Structure

```rust
use axum::{
    Router,
    routing::{get, post, put, delete},
    middleware,
};

pub fn organization_routes() -> Router {
    Router::new()
        // Create - POST /api/v1/organizations
        .route("/api/v1/organizations", post(create_organization))
        // Read Collection - GET /api/v1/organizations
        .route("/api/v1/organizations", get(find_all_organizations))
        // Read Single - GET /api/v1/organizations/:id
        .route("/api/v1/organizations/:id", get(find_one_organization))
        // Update - PUT /api/v1/organizations/:id
        .route("/api/v1/organizations/:id", put(update_organization))
        // Delete - DELETE /api/v1/organizations/:id
        .route("/api/v1/organizations/:id", delete(remove_organization))
        // Action - POST /api/v1/organizations/:id/transfer-ownership
        .route("/api/v1/organizations/:id/transfer-ownership", post(transfer_ownership))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn(jwt_auth_middleware))
}

// Create handler
async fn create_organization(
    Extension(user): Extension<CurrentUser>,
    Json(dto): Json<CreateOrganizationDto>,
) -> Result<(StatusCode, Json<OrganizationResponseDto>), AppError> {
    dto.validate().map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    let command = CreateOrganizationCommand::new(
        user.email.clone(),
        user.user_id.clone(),
        dto.name,
    );

    let organization = command_bus.execute(command).await?;
    Ok((StatusCode::CREATED, Json(organization)))
}

// Read collection handler
async fn find_all_organizations(
    Query(query_dto): Query<OrganizationQueryDto>,
) -> Result<Json<PaginatedResponse<Organization>>, AppError> {
    query_dto.validate().map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    let organizations = query_bus.execute(
        GetOrganizationsQuery::new(query_dto)
    ).await?;

    Ok(Json(organizations))
}

// Read single handler
async fn find_one_organization(
    Path(id): Path<String>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<OrganizationResponseDto>, AppError> {
    let organization = query_bus.execute(
        GetOrganizationByIdQuery::new(id)
    ).await?;

    Ok(Json(organization))
}

// Update handler
async fn update_organization(
    Path(id): Path<String>,
    Extension(user): Extension<CurrentUser>,
    Extension(tenant): Extension<TenantContext>,
    Json(dto): Json<UpdateOrganizationDto>,
) -> Result<Json<OrganizationResponseDto>, AppError> {
    dto.validate().map_err(|e| AppError::BadRequest(format!("{}", e)))?;

    let command = UpdateOrganizationCommand::new(
        id,
        user.user_id.clone(),
        dto.name,
        dto.settings,
    );

    let organization = command_bus.execute(command).await?;
    Ok(Json(organization))
}

// Delete handler
async fn remove_organization(
    Path(id): Path<String>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<StatusCode, AppError> {
    let command = DeleteOrganizationCommand::new(id);
    command_bus.execute(command).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

### 2. Module Organization

```rust
// lib.rs or main.rs
mod handlers;
mod commands;
mod queries;
mod middleware;
mod errors;

use axum::Router;

pub fn create_app() -> Router {
    Router::new()
        .merge(organization_routes())
        .merge(user_routes())
        .merge(project_routes())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
}

// Dependency injection with state
#[derive(Clone)]
pub struct AppState {
    pub command_bus: Arc<CommandBus>,
    pub query_bus: Arc<QueryBus>,
    pub db_pool: Arc<PgPool>,
}

pub fn organization_routes_with_state(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/organizations", post(create_organization))
        .route("/api/v1/organizations", get(find_all_organizations))
        .route("/api/v1/organizations/:id", get(find_one_organization))
        .with_state(state)
}
```

---

## Testing REST APIs

### 1. Unit Tests (Handlers)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        CommandBus {}

        impl CommandBus {
            async fn execute<T>(&self, command: T) -> Result<Organization, AppError>;
        }
    }

    mock! {
        QueryBus {}

        impl QueryBus {
            async fn execute<T>(&self, query: T) -> Result<Organization, AppError>;
        }
    }

    #[tokio::test]
    async fn test_create_organization_with_correct_parameters() {
        let dto = CreateOrganizationDto {
            name: "Acme Corporation".to_string(),
            primary_domain: "acme.com".to_string(),
            settings: None,
        };

        let mock_user = CurrentUser {
            user_id: "uuid".to_string(),
            email: "owner@acme.com".to_string(),
            role: "ORG_OWNER".to_string(),
            organization_id: "org-uuid".to_string(),
        };

        let mock_organization = Organization {
            id: "uuid".to_string(),
            name: "Acme Corporation".to_string(),
            slug: "acme-corporation".to_string(),
            primary_domain: "acme.com".to_string(),
            owner_id: "uuid".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        };

        let mut mock_command_bus = MockCommandBus::new();
        mock_command_bus
            .expect_execute()
            .returning(|_| Ok(mock_organization.clone()));

        let result = create_organization_internal(
            mock_user,
            dto,
            mock_command_bus,
        ).await;

        assert!(result.is_ok());
        let (status, Json(org)) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(org.name, "Acme Corporation");
    }
}
```

### 2. E2E Tests

```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt; // for `oneshot`
    use serde_json::json;

    async fn setup_app() -> Router {
        // Setup test database, state, etc.
        let state = AppState {
            command_bus: Arc::new(CommandBus::new()),
            query_bus: Arc::new(QueryBus::new()),
            db_pool: Arc::new(create_test_pool().await),
        };

        create_app().with_state(state)
    }

    #[tokio::test]
    async fn test_create_organization_with_valid_data() {
        let app = setup_app().await;
        let owner_token = get_test_token().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header("Authorization", format!("Bearer {}", owner_token))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "name": "Test Organization",
                            "primaryDomain": "testorg.com"
                        })).unwrap()
                    ))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let org: OrganizationResponseDto = serde_json::from_slice(&body).unwrap();

        assert_eq!(org.name, "Test Organization");
        assert_eq!(org.slug, "test-organization");
        assert_eq!(org.primary_domain, "testorg.com");
    }

    #[tokio::test]
    async fn test_reject_duplicate_domain() {
        let app = setup_app().await;
        let owner_token = get_test_token().await;

        // Create first organization
        create_test_organization(&app, &owner_token, "testorg.com").await;

        // Try to create duplicate
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header("Authorization", format!("Bearer {}", owner_token))
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "name": "Duplicate Org",
                            "primaryDomain": "testorg.com"
                        })).unwrap()
                    ))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_require_authentication() {
        let app = setup_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/organizations")
                    .header("Content-Type", "application/json")
                    .body(Body::from(
                        serde_json::to_string(&json!({
                            "name": "Unauthorized Org",
                            "primaryDomain": "unauth.com"
                        })).unwrap()
                    ))
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_get_organization_by_id() {
        let app = setup_app().await;
        let owner_token = get_test_token().await;

        let org_id = create_test_organization(&app, &owner_token, "testorg.com").await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/v1/organizations/{}", org_id))
                    .header("Authorization", format!("Bearer {}", owner_token))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let org: OrganizationResponseDto = serde_json::from_slice(&body).unwrap();
        assert_eq!(org.id, org_id);
    }

    #[tokio::test]
    async fn test_return_404_for_nonexistent_organization() {
        let app = setup_app().await;
        let owner_token = get_test_token().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/v1/organizations/00000000-0000-0000-0000-000000000000")
                    .header("Authorization", format!("Bearer {}", owner_token))
                    .body(Body::empty())
                    .unwrap()
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
```

---

## Summary

### Quick Reference Checklist

#### ✅ URL Design

- [ ] Use plural nouns (`/users`, `/organizations`)
- [ ] Use kebab-case (`/pending-users`)
- [ ] Include version in path (`/api/v1/`)
- [ ] Limit nesting to 2 levels
- [ ] Use action verbs only when necessary

#### ✅ HTTP Methods

- [ ] GET for retrieval (safe, idempotent)
- [ ] POST for creation (non-idempotent)
- [ ] PUT for full replacement (idempotent)
- [ ] PATCH for partial updates
- [ ] DELETE for removal (idempotent)

#### ✅ Status Codes

- [ ] 200 OK for successful GET/PUT/PATCH
- [ ] 201 Created for successful POST
- [ ] 204 No Content for successful DELETE
- [ ] 400 Bad Request for validation errors
- [ ] 401 Unauthorized for missing/invalid auth
- [ ] 403 Forbidden for insufficient permissions
- [ ] 404 Not Found for missing resources
- [ ] 409 Conflict for duplicate resources
- [ ] 500 Internal Server Error for unexpected errors

#### ✅ Security

- [ ] Require authentication (JWT middleware)
- [ ] Validate and sanitize all inputs with `validator` crate
- [ ] Implement authorization checks in handlers
- [ ] Use HTTPS in production
- [ ] Implement rate limiting with `tower-governor`
- [ ] Return generic error messages (don't leak implementation details)

#### ✅ Documentation

- [ ] Use `#[utoipa::path]` for endpoint documentation
- [ ] Add summary and description to all routes
- [ ] Document all response codes with `responses()`
- [ ] Use `#[derive(ToSchema)]` on DTOs
- [ ] Include examples in documentation with `#[schema(example = ...)]`

#### ✅ Testing

- [ ] Unit test handlers (mock CommandBus/QueryBus with `mockall`)
- [ ] E2E test all endpoints with `tower::ServiceExt`
- [ ] Test authentication requirements
- [ ] Test authorization rules
- [ ] Test validation failures
- [ ] Test error handling

---

## Related Patterns

- **Pattern 03**: [Hexagonal Architecture](./03-Hexagonal-Architecture.md)
- **Pattern 05**: [CQRS Pattern](./05-CQRS-Pattern.md)
- **Pattern 07**: [DTO Pattern](./07-DTO-Pattern.md)
- **Pattern 39**: [Security Patterns Guide](./39-Security-Patterns-Guide.md)

---

## References

- [REST API Design Rulebook (O'Reilly)](https://www.oreilly.com/library/view/rest-api-design/9781449317904/)
- [Microsoft REST API Guidelines](https://github.com/microsoft/api-guidelines)
- [Google API Design Guide](https://cloud.google.com/apis/design)
- [HTTP Status Codes (MDN)](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status)
- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [Utoipa (OpenAPI) Documentation](https://docs.rs/utoipa/latest/utoipa/)
- [Validator Crate](https://docs.rs/validator/latest/validator/)

---

**Last Updated**: October 8, 2025
**Version**: 1.0
**Status**: Active
