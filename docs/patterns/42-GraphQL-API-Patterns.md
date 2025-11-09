# Pattern 42: GraphQL API Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Status**: Active

---

## Table of Contents

1. [Overview](#overview)
2. [Schema Design](#schema-design)
3. [Query Patterns](#query-patterns)
4. [Mutation Patterns](#mutation-patterns)
5. [Subscription Patterns](#subscription-patterns)
6. [DataLoader Pattern](#dataloader-pattern)
7. [Error Handling](#error-handling)
8. [Security & Authorization](#security--authorization)
9. [Performance Optimization](#performance-optimization)
10. [Rust Implementation](#rust-implementation)
11. [Testing GraphQL](#testing-graphql)

---

## Overview

GraphQL provides a powerful alternative to REST APIs, offering clients precise control over data fetching and reducing over-fetching and under-fetching issues.

### When to Use GraphQL

**✅ Use GraphQL When**:

- Clients need flexible data queries
- Multiple related resources are frequently fetched together
- Mobile apps need to minimize data transfer
- Frontend teams need rapid iteration without backend changes
- Real-time updates are important (subscriptions)

**❌ Use REST When**:

- Simple CRUD operations
- File uploads/downloads
- Heavy caching requirements
- Team unfamiliar with GraphQL
- Third-party integrations (most expect REST)

### GraphQL vs REST

| Aspect             | GraphQL                             | REST                               |
| ------------------ | ----------------------------------- | ---------------------------------- |
| **Data Fetching**  | Single request, multiple resources  | Multiple requests or over-fetching |
| **Versioning**     | Schema evolution                    | URL versioning                     |
| **Caching**        | Complex (requires normalized cache) | Simple (HTTP caching)              |
| **File Uploads**   | Requires special handling           | Native support                     |
| **Learning Curve** | Steeper                             | Gentler                            |

---

## Schema Design

### 1. Schema-First vs Code-First

**Schema-First** (Recommended for API contracts):

```graphql
# schema.graphql
type Organization {
  id: ID!
  name: String!
  slug: String!
  primaryDomain: String!
  owner: User!
  users: [User!]!
  projects: [Project!]!
  createdAt: DateTime!
  updatedAt: DateTime!
}

type User {
  id: ID!
  email: String!
  firstName: String!
  lastName: String!
  role: Role!
  organization: Organization!
}

enum Role {
  ORG_OWNER
  ADMIN
  MANAGER
  MEMBER
}

type Query {
  organization(id: ID!): Organization
  organizations(limit: Int, offset: Int): OrganizationConnection!
  me: User!
}

type Mutation {
  createOrganization(input: CreateOrganizationInput!): Organization!
  updateOrganization(id: ID!, input: UpdateOrganizationInput!): Organization!
  deleteOrganization(id: ID!): Boolean!
}
```

**Code-First** (Rust with async-graphql):

```rust
use async_graphql::{Object, Context, ID, Enum, SimpleObject};
use chrono::{DateTime, Utc};

#[derive(SimpleObject)]
pub struct Organization {
    pub id: ID,
    pub name: String,
    pub slug: String,
    pub primary_domain: String,
    #[graphql(skip)]
    pub owner_id: ID,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[Object]
impl Organization {
    async fn owner(&self, ctx: &Context<'_>) -> Result<User> {
        let user_repo = ctx.data::<Arc<dyn IUserRepository>>()?;
        user_repo.find_by_id(&self.owner_id).await
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let user_repo = ctx.data::<Arc<dyn IUserRepository>>()?;
        user_repo.find_by_organization(&self.id).await
    }

    async fn projects(&self, ctx: &Context<'_>) -> Result<Vec<Project>> {
        let project_repo = ctx.data::<Arc<dyn IProjectRepository>>()?;
        project_repo.find_by_organization(&self.id).await
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
/// User role within organization
pub enum Role {
    OrgOwner,
    Admin,
    Manager,
    Member,
}
```

### 2. Type Design Best Practices

**Use Descriptive Names**:

```graphql
# ✅ Good
type TimeEntry {
  id: ID!
  description: String!
  hours: Float!
  billableAmount: Money
}

# ❌ Bad
type TE {
  id: ID!
  desc: String!
  hrs: Float!
  amt: Int
}
```

**Non-Nullable Fields**:

```graphql
type User {
  id: ID! # Required, always present
  email: String! # Required
  firstName: String! # Required
  lastName: String! # Required
  phone: String # Optional
  deletedAt: DateTime # Optional
}
```

**Connection Pattern for Lists**:

```graphql
type OrganizationConnection {
  edges: [OrganizationEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type OrganizationEdge {
  node: Organization!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}
```

### 3. Input Types

```graphql
input CreateOrganizationInput {
  name: String!
  primaryDomain: String!
  settings: OrganizationSettingsInput
}

input UpdateOrganizationInput {
  name: String
  settings: OrganizationSettingsInput
}

input OrganizationSettingsInput {
  timeApprovalRequired: Boolean
  clientPortalEnabled: Boolean
  defaultHourlyRate: Float
}
```

**Rust Code-First with async-graphql**:

```rust
use async_graphql::InputObject;
use validator::Validate;

#[derive(InputObject, Validate)]
pub struct CreateOrganizationInput {
    #[validate(length(min = 2, max = 100))]
    pub name: String,

    #[validate(regex = "DOMAIN_REGEX")]
    pub primary_domain: String,

    pub settings: Option<OrganizationSettingsInput>,
}
```

---

## Query Patterns

### 1. Basic Queries

```rust
use async_graphql::{Context, Object, Result, ID};
use std::sync::Arc;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get organization by ID (requires authentication)
    async fn organization(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Organization>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationByIdQuery::new(id.to_string());
        query_bus.execute(query).await
    }

    /// Get all organizations with pagination
    async fn organizations(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 25)] limit: i32,
        #[graphql(default = 0)] offset: i32,
    ) -> Result<Vec<Organization>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationsQuery::new(limit, offset);
        query_bus.execute(query).await
    }

    /// Get current authenticated user
    async fn me(&self, ctx: &Context<'_>) -> Result<User> {
        let current_user = ctx.data::<CurrentUserData>()?;
        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetUserByIdQuery::new(current_user.user_id.clone());
        query_bus.execute(query).await
    }
}
```

### 2. Pagination

**Cursor-Based Pagination** (Recommended):

```rust
#[Object]
impl QueryRoot {
    async fn organizations(
        &self,
        ctx: &Context<'_>,
        first: Option<i32>,
        after: Option<String>,
        last: Option<i32>,
        before: Option<String>,
    ) -> Result<OrganizationConnection> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationConnectionQuery::new(first, after, last, before);
        query_bus.execute(query).await
    }
}
```

**GraphQL Query**:

```graphql
query GetOrganizations {
  organizations(first: 10, after: "cursor123") {
    edges {
      node {
        id
        name
        slug
      }
      cursor
    }
    pageInfo {
      hasNextPage
      endCursor
    }
    totalCount
  }
}
```

### 3. Filtering & Sorting

```rust
use async_graphql::InputObject;

#[derive(InputObject)]
pub struct OrganizationFilterInput {
    pub status: Option<OrganizationStatus>,
    pub search: Option<String>,
    pub created_at: Option<DateRangeInput>,
}

#[derive(InputObject)]
pub struct OrganizationSortInput {
    pub field: OrganizationSortField,
    pub order: SortOrder,
}

#[Object]
impl QueryRoot {
    async fn organizations(
        &self,
        ctx: &Context<'_>,
        filter: Option<OrganizationFilterInput>,
        sort: Option<OrganizationSortInput>,
    ) -> Result<OrganizationConnection> {
        // Implementation
        todo!()
    }
}
```

---

## Mutation Patterns

### 1. Create Mutations

```rust
pub struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> Result<Organization> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let current_user = ctx.data::<CurrentUserData>()?;
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;

        let command = CreateOrganizationCommand::new(
            current_user.email.clone(),
            current_user.user_id.clone(),
            input.name,
        );

        command_bus.execute(command).await
    }
}
```

**GraphQL Mutation**:

```graphql
mutation CreateOrganization {
  createOrganization(
    input: {
      name: "Acme Corporation"
      primaryDomain: "acme.com"
      settings: { timeApprovalRequired: true, defaultHourlyRate: 150 }
    }
  ) {
    id
    name
    slug
    createdAt
  }
}
```

### 2. Update Mutations

```rust
#[Object]
impl MutationRoot {
    async fn update_organization(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateOrganizationInput,
    ) -> Result<Organization> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;
        auth_guard.require_tenant_access()?;

        let current_user = ctx.data::<CurrentUserData>()?;
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;

        let command = UpdateOrganizationCommand::new(
            id.to_string(),
            current_user.user_id.clone(),
            input.name,
            input.settings,
        );

        command_bus.execute(command).await
    }
}
```

### 3. Delete Mutations

```rust
#[Object]
impl MutationRoot {
    async fn delete_organization(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<bool> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;
        auth_guard.require_tenant_access()?;

        let current_user = ctx.data::<CurrentUserData>()?;
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;

        let command = DeleteOrganizationCommand::new(
            id.to_string(),
            current_user.user_id.clone(),
        );

        command_bus.execute(command).await?;
        Ok(true)
    }
}
```

### 4. Optimistic Response Pattern

**Client-Side (React/Apollo)**:

```typescript
const [updateOrganization] = useMutation(UPDATE_ORGANIZATION, {
  optimisticResponse: {
    __typename: 'Mutation',
    updateOrganization: {
      __typename: 'Organization',
      id: organizationId,
      name: newName,
      updatedAt: new Date().toISOString(),
    },
  },
  update: (cache, { data }) => {
    // Update cache with real response
  },
});
```

---

## Subscription Patterns

### 1. Real-Time Updates

```rust
use async_graphql::{Subscription, Context};
use futures_util::Stream;
use tokio::sync::broadcast;

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    async fn organization_updated(
        &self,
        ctx: &Context<'_>,
        organization_id: ID,
    ) -> Result<impl Stream<Item = Organization>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let broadcaster = ctx.data::<Broadcaster>()?;
        let mut receiver = broadcaster.subscribe();

        Ok(async_stream::stream! {
            while let Ok(event) = receiver.recv().await {
                if let Event::OrganizationUpdated(org) = event {
                    if org.id.to_string() == organization_id.to_string() {
                        yield org;
                    }
                }
            }
        })
    }
}
```

**Trigger Subscription**:

```rust
// Event handler that publishes to subscribers
pub struct OnOrganizationUpdatedHandler {
    broadcaster: Arc<Broadcaster>,
}

impl EventHandler<OrganizationUpdatedEvent> for OnOrganizationUpdatedHandler {
    async fn handle(&self, event: OrganizationUpdatedEvent) -> Result<()> {
        self.broadcaster.publish(Event::OrganizationUpdated(event.organization)).await;
        Ok(())
    }
}
```

**Client Subscription**:

```graphql
subscription OnOrganizationUpdated($organizationId: ID!) {
  organizationUpdated(organizationId: $organizationId) {
    id
    name
    updatedAt
  }
}
```

### 2. Redis PubSub for Scaling

```rust
use redis::aio::MultiplexedConnection;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

pub struct RedisBroadcaster {
    publisher: MultiplexedConnection,
    subscriber: MultiplexedConnection,
}

impl RedisBroadcaster {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)?;
        let publisher = client.get_multiplexed_async_connection().await?;
        let subscriber = client.get_multiplexed_async_connection().await?;

        Ok(Self {
            publisher,
            subscriber,
        })
    }

    pub async fn publish<T: Serialize>(&mut self, channel: &str, event: &T) -> Result<()> {
        let json = serde_json::to_string(event)?;
        self.publisher.publish(channel, json).await?;
        Ok(())
    }

    pub async fn subscribe<T: for<'de> Deserialize<'de>>(
        &mut self,
        channel: &str,
    ) -> Result<impl Stream<Item = T>> {
        // Implementation using redis pubsub
        todo!()
    }
}
```

---

## DataLoader Pattern

### 1. Preventing N+1 Queries

**Without DataLoader** (N+1 Problem):

```rust
#[Object]
impl Organization {
    async fn owner(&self, ctx: &Context<'_>) -> Result<User> {
        // Called N times for N organizations - N+1 queries!
        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetUserByIdQuery::new(self.owner_id.clone());
        query_bus.execute(query).await
    }
}
```

**With DataLoader (async-graphql built-in)**:

```rust
use async_graphql::dataloader::{DataLoader, Loader};
use std::collections::HashMap;

pub struct UserLoader {
    user_repository: Arc<dyn IUserRepository>,
}

#[async_trait::async_trait]
impl Loader<String> for UserLoader {
    type Value = User;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let users = self.user_repository
            .find_by_ids(keys)
            .await
            .map_err(|e| Arc::new(e))?;

        // Return users as HashMap
        Ok(users.into_iter()
            .map(|user| (user.id.clone(), user))
            .collect())
    }
}

pub struct OrganizationLoader {
    org_repository: Arc<dyn IOrganizationRepository>,
}

#[async_trait::async_trait]
impl Loader<String> for OrganizationLoader {
    type Value = Organization;
    type Error = Arc<anyhow::Error>;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let organizations = self.org_repository
            .find_by_ids(keys)
            .await
            .map_err(|e| Arc::new(e))?;

        Ok(organizations.into_iter()
            .map(|org| (org.id.clone(), org))
            .collect())
    }
}
```

**Using DataLoader**:

```rust
#[Object]
impl Organization {
    async fn owner(&self, ctx: &Context<'_>) -> Result<User> {
        // Batched and cached!
        let loader = ctx.data::<DataLoader<UserLoader>>()?;
        loader.load_one(self.owner_id.clone()).await?
            .ok_or_else(|| "User not found".into())
    }
}
```

### 2. DataLoader Setup in Context

```rust
use async_graphql::{Schema, EmptySubscription, dataloader::DataLoader};
use axum::{Extension, Router};

async fn graphql_handler(
    schema: Extension<Schema<QueryRoot, MutationRoot, SubscriptionRoot>>,
    req: HttpRequest,
) -> GraphQLResponse {
    let user_repository = get_user_repository(); // Dependency injection
    let org_repository = get_org_repository();

    // Create loaders for this request
    let user_loader = DataLoader::new(
        UserLoader { user_repository: user_repository.clone() },
        tokio::spawn,
    );
    let org_loader = DataLoader::new(
        OrganizationLoader { org_repository: org_repository.clone() },
        tokio::spawn,
    );

    // Build context with loaders
    let request = req.into_inner()
        .data(user_loader)
        .data(org_loader);

    schema.execute(request).await.into()
}

pub fn create_schema() -> Schema<QueryRoot, MutationRoot, SubscriptionRoot> {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .finish()
}
```

---

## Error Handling

### 1. GraphQL Errors

```rust
use async_graphql::{Error, ErrorExtensions};

#[Object]
impl MutationRoot {
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> Result<Organization> {
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;
        let command = CreateOrganizationCommand::new(/* ... */);

        match command_bus.execute(command).await {
            Ok(org) => Ok(org),
            Err(e) if e.is::<DuplicateOrganizationDomainException>() => {
                Err(Error::new(format!(
                    "Organization with domain '{}' already exists",
                    input.primary_domain
                ))
                .extend_with(|_, e| {
                    e.set("code", "DUPLICATE_DOMAIN");
                    e.set("domain", input.primary_domain.clone());
                }))
            }
            Err(e) => Err(e.into()),
        }
    }
}
```

### 2. Custom Error Codes

```rust
use async_graphql::ErrorExtensions;

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Unauthenticated,
    Forbidden,
    NotFound,
    BadUserInput,
    DuplicateResource,
    InternalServerError,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "UNAUTHENTICATED",
            Self::Forbidden => "FORBIDDEN",
            Self::NotFound => "NOT_FOUND",
            Self::BadUserInput => "BAD_USER_INPUT",
            Self::DuplicateResource => "DUPLICATE_RESOURCE",
            Self::InternalServerError => "INTERNAL_SERVER_ERROR",
        }
    }
}

// Usage
Err(Error::new("Invalid input")
    .extend_with(|_, e| {
        e.set("code", ErrorCode::BadUserInput.as_str());
        e.set("field", "email");
        e.set("message", "Email already exists");
    }))
```

### 3. Error Response Format

```json
{
  "errors": [
    {
      "message": "Organization with domain 'acme.com' already exists",
      "extensions": {
        "code": "DUPLICATE_DOMAIN",
        "domain": "acme.com"
      },
      "path": ["createOrganization"]
    }
  ],
  "data": null
}
```

---

## Security & Authorization

### 1. Authentication Guard

```rust
use async_graphql::Context;
use jsonwebtoken::{decode, DecodingKey, Validation};

pub struct AuthGuard;

impl AuthGuard {
    pub fn require_authenticated(&self) -> Result<()> {
        // Authentication check - extract from context
        Ok(())
    }

    pub fn extract_user_from_context(ctx: &Context<'_>) -> Result<CurrentUserData> {
        // Extract JWT from request headers
        let token = ctx.data_opt::<String>()
            .ok_or_else(|| Error::new("Unauthenticated"))?;

        // Decode and validate JWT
        let decoding_key = DecodingKey::from_secret(b"secret");
        let validation = Validation::default();
        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|_| Error::new("Invalid token"))?;

        Ok(CurrentUserData {
            user_id: token_data.claims.sub,
            email: token_data.claims.email,
            role: token_data.claims.role,
            organization_id: token_data.claims.organization_id,
        })
    }
}
```

### 2. Field-Level Authorization

```rust
#[Object]
impl Organization {
    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let current_user = AuthGuard::extract_user_from_context(ctx)?;

        // Only org members can see user list
        if current_user.organization_id != self.id {
            return Err(Error::new("Cannot access users from another organization")
                .extend_with(|_, e| e.set("code", "FORBIDDEN")));
        }

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationUsersQuery::new(self.id.clone());
        query_bus.execute(query).await
    }
}
```

### 3. Directive-Based Authorization

```graphql
directive @auth(requires: Role!) on FIELD_DEFINITION

type Mutation {
  deleteOrganization(id: ID!): Boolean! @auth(requires: ORG_OWNER)
  transferOwnership(id: ID!, newOwnerId: ID!): Organization! @auth(requires: ORG_OWNER)
}
```

```rust
use async_graphql::*;

// Custom directive for role-based authorization
#[derive(Debug)]
pub struct RoleDirective {
    pub requires: Role,
}

impl RoleDirective {
    pub fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let current_user = AuthGuard::extract_user_from_context(ctx)?;

        if current_user.role != self.requires {
            return Err(Error::new(format!("Requires {:?} role", self.requires))
                .extend_with(|_, e| e.set("code", "FORBIDDEN")));
        }

        Ok(())
    }
}

// Usage in resolver
#[Object]
impl MutationRoot {
    async fn delete_organization(&self, ctx: &Context<'_>, id: ID) -> Result<bool> {
        // Check role requirement
        let role_check = RoleDirective { requires: Role::OrgOwner };
        role_check.check(ctx)?;

        // Continue with mutation
        // ...
        Ok(true)
    }
}
```

---

## Performance Optimization

### 1. Query Complexity Analysis

```rust
use async_graphql::*;
use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute};

pub struct QueryComplexity {
    max_complexity: usize,
}

#[async_trait::async_trait]
impl Extension for QueryComplexity {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        // Calculate query complexity
        let complexity = calculate_complexity(ctx.query());

        if complexity > self.max_complexity {
            return Response::from_errors(vec![
                Error::new(format!(
                    "Query too complex: {}. Maximum allowed: {}",
                    complexity, self.max_complexity
                ))
            ]);
        }

        next.run(ctx, operation_name).await
    }
}

// Usage in schema builder
Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .extension(QueryComplexity { max_complexity: 1000 })
    .finish()
```

### 2. Query Depth Limiting

```rust
use async_graphql::*;

pub struct DepthLimit {
    max_depth: usize,
}

#[async_trait::async_trait]
impl Extension for DepthLimit {
    async fn execute(
        &self,
        ctx: &ExtensionContext<'_>,
        operation_name: Option<&str>,
        next: NextExecute<'_>,
    ) -> Response {
        let depth = calculate_query_depth(ctx.query());

        if depth > self.max_depth {
            return Response::from_errors(vec![
                Error::new(format!(
                    "Query depth {} exceeds maximum allowed depth of {}",
                    depth, self.max_depth
                ))
            ]);
        }

        next.run(ctx, operation_name).await
    }
}

// Usage
Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
    .extension(DepthLimit { max_depth: 5 })
    .finish()
```

### 3. Persisted Queries

```rust
use async_graphql::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PersistedQueryCache {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl PersistedQueryCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, hash: &str) -> Option<String> {
        self.cache.read().await.get(hash).cloned()
    }

    pub async fn set(&self, hash: String, query: String) {
        self.cache.write().await.insert(hash, query);
    }
}

// Can also use Redis for distributed caching
// use redis::AsyncCommands;
```

**Client**:

```typescript
import { createPersistedQueryLink } from '@apollo/client/link/persisted-queries';
import { sha256 } from 'crypto-hash';

const link = createPersistedQueryLink({ sha256 });
```

---

## Rust Implementation

### 1. Server Setup with Axum

```rust
use async_graphql::{Schema, EmptySubscription, http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use std::sync::Arc;

pub type AppSchema = Schema<QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema() -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, SubscriptionRoot)
        .extension(QueryComplexity { max_complexity: 1000 })
        .extension(DepthLimit { max_depth: 5 })
        .finish()
}

async fn graphql_handler(
    schema: Extension<AppSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn create_router(schema: AppSchema) -> Router {
    Router::new()
        .route("/graphql", get(graphiql).post(graphql_handler))
        .route("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
}
```

### 2. Complete Resolver Example

```rust
use async_graphql::{Context, Object, Result, ID};
use async_graphql::dataloader::DataLoader;
use std::sync::Arc;

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Get organization by ID
    async fn organization(&self, ctx: &Context<'_>, id: ID) -> Result<Option<Organization>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationByIdQuery::new(id.to_string());
        query_bus.execute(query).await
    }

    /// Get all organizations
    async fn organizations(&self, ctx: &Context<'_>) -> Result<Vec<Organization>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        query_bus.execute(GetOrganizationsQuery).await
    }
}

pub struct MutationRoot;

#[Object]
impl MutationRoot {
    /// Create a new organization
    async fn create_organization(
        &self,
        ctx: &Context<'_>,
        input: CreateOrganizationInput,
    ) -> Result<Organization> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let current_user = AuthGuard::extract_user_from_context(ctx)?;
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;

        let command = CreateOrganizationCommand::new(
            current_user.email,
            current_user.user_id,
            input.name,
        );
        command_bus.execute(command).await
    }

    /// Update an organization
    async fn update_organization(
        &self,
        ctx: &Context<'_>,
        id: ID,
        input: UpdateOrganizationInput,
    ) -> Result<Organization> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;
        auth_guard.require_tenant_access()?;

        let current_user = AuthGuard::extract_user_from_context(ctx)?;
        let command_bus = ctx.data::<Arc<dyn ICommandBus>>()?;

        let command = UpdateOrganizationCommand::new(
            id.to_string(),
            current_user.user_id,
            input.name,
            input.settings,
        );
        command_bus.execute(command).await
    }
}

// Field resolvers
#[Object]
impl Organization {
    async fn owner(&self, ctx: &Context<'_>) -> Result<User> {
        let loader = ctx.data::<DataLoader<UserLoader>>()?;
        loader.load_one(self.owner_id.clone()).await?
            .ok_or_else(|| "User not found".into())
    }

    async fn users(&self, ctx: &Context<'_>) -> Result<Vec<User>> {
        let auth_guard = ctx.data::<AuthGuard>()?;
        auth_guard.require_authenticated()?;

        let query_bus = ctx.data::<Arc<dyn IQueryBus>>()?;
        let query = GetOrganizationUsersQuery::new(self.id.clone());
        query_bus.execute(query).await
    }
}
```

---

## Testing GraphQL

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        CommandBus {}
        #[async_trait::async_trait]
        impl ICommandBus for CommandBus {
            async fn execute<C: Command>(&self, command: C) -> Result<C::Output>;
        }
    }

    mock! {
        QueryBus {}
        #[async_trait::async_trait]
        impl IQueryBus for QueryBus {
            async fn execute<Q: Query>(&self, query: Q) -> Result<Q::Output>;
        }
    }

    #[tokio::test]
    async fn test_create_organization() {
        let mut mock_command_bus = MockCommandBus::new();
        let mock_org = Organization {
            id: "uuid".into(),
            name: "Test Org".to_string(),
            // ... other fields
        };

        mock_command_bus
            .expect_execute()
            .returning(move |_| Ok(mock_org.clone()));

        let input = CreateOrganizationInput {
            name: "Test Org".to_string(),
            primary_domain: "test.com".to_string(),
            settings: None,
        };

        let ctx = create_test_context(mock_command_bus);
        let mutation_root = MutationRoot;

        let result = mutation_root.create_organization(&ctx, input).await;

        assert!(result.is_ok());
        let org = result.unwrap();
        assert_eq!(org.name, "Test Org");
    }
}
```

### 2. E2E Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_organization_e2e() {
        // Setup test app
        let schema = create_test_schema();
        let app = create_router(schema);

        // Login to get token
        let login_query = json!({
            "query": r#"
                mutation {
                    login(email: "test@example.com", password: "password") {
                        accessToken
                    }
                }
            "#
        });

        let login_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("content-type", "application/json")
                    .body(Body::from(login_query.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        let login_body: serde_json::Value =
            serde_json::from_slice(&hyper::body::to_bytes(login_response.into_body()).await.unwrap())
            .unwrap();
        let token = login_body["data"]["login"]["accessToken"].as_str().unwrap();

        // Test create organization
        let create_query = json!({
            "query": r#"
                mutation CreateOrganization($input: CreateOrganizationInput!) {
                    createOrganization(input: $input) {
                        id
                        name
                        slug
                        primaryDomain
                    }
                }
            "#,
            "variables": {
                "input": {
                    "name": "Test Organization",
                    "primaryDomain": "test.com"
                }
            }
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/graphql")
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(create_query.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body: serde_json::Value =
            serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap())
            .unwrap();

        let org = &body["data"]["createOrganization"];
        assert!(org["id"].is_string());
        assert_eq!(org["name"], "Test Organization");
        assert_eq!(org["slug"], "test-organization");
        assert_eq!(org["primaryDomain"], "test.com");
    }
}
```

---

## Summary

### GraphQL Best Practices Checklist

#### ✅ Schema Design

- [ ] Use descriptive, business-focused type names
- [ ] Mark required fields with `!`
- [ ] Use Input types for mutations
- [ ] Implement Connection pattern for pagination
- [ ] Version schema through field deprecation

#### ✅ Performance

- [ ] Implement DataLoader for N+1 prevention
- [ ] Add query complexity analysis
- [ ] Limit query depth
- [ ] Use persisted queries in production
- [ ] Batch database queries

#### ✅ Security

- [ ] Require authentication on sensitive queries/mutations
- [ ] Implement field-level authorization
- [ ] Validate all inputs
- [ ] Sanitize user input
- [ ] Rate limit GraphQL endpoint

#### ✅ Error Handling

- [ ] Use custom error codes
- [ ] Provide helpful error messages
- [ ] Include field context in validation errors
- [ ] Log errors server-side
- [ ] Don't leak implementation details

#### ✅ Testing

- [ ] Unit test all resolvers
- [ ] E2E test critical workflows
- [ ] Test authorization rules
- [ ] Test error scenarios
- [ ] Load test complex queries

---

## Related Patterns

- **Pattern 05**: [CQRS Pattern](./05-CQRS-Pattern.md)
- **Pattern 06**: [Repository Pattern](./06-Repository-Pattern.md)
- **Pattern 39**: [Security Patterns Guide](./39-Security-Patterns-Guide.md)
- **Pattern 41**: [REST API Best Practices](./41-REST-API-Best-Practices.md)

---

## References

- [GraphQL Specification](https://spec.graphql.org/)
- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/en/index.html)
- [async-graphql GitHub](https://github.com/async-graphql/async-graphql)
- [GraphQL Rust](https://graphql-rust.github.io/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)

---

**Last Updated**: October 8, 2025
**Version**: 1.0
**Status**: Active
