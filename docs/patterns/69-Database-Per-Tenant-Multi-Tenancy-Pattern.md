# Database-Per-Tenant Multi-Tenancy Pattern

**Category**: Architecture Pattern
**Complexity**: Advanced
**Status**: ✅ Production Ready
**Related Patterns**: Repository Pattern, Unit of Work, Connection Pooling
**Industry Context**: Oil & Gas Field Data Management

---

## Overview

The Database-Per-Tenant Multi-Tenancy pattern provides complete data isolation by giving each tenant (oil & gas operator) their own dedicated database. Unlike shared-database approaches (row-level security), this pattern ensures:

- **Complete data isolation**: No risk of cross-tenant data leakage
- **Client data sovereignty**: Tenants choose where their database lives (Azure, AWS, on-premises)
- **Independent scaling**: Large tenants get dedicated resources
- **Compliance alignment**: Easier to meet regulatory requirements (data residency, audit trails)

This pattern is critical for WellOS because oil & gas operators demand:

1. Control over where their production data is stored
2. Assurance that competitors cannot access their data (even accidentally)
3. Ability to keep databases on-premises for security/compliance

---

## The Problem

**Scenario**: WellOS serves 50+ independent oil & gas operators. Each operator has:

- Proprietary production data (well outputs, equipment sensors, field notes)
- Regulatory compliance requirements (state-specific, data residency)
- Different infrastructure preferences (some use Azure, some AWS, some on-prem)

**Challenges with Shared Database + Row-Level Security (RLS)**:

```rust
// ❌ Shared database approach (not suitable for WellOS)
let wells = sqlx::query_as!(
    Well,
    "SELECT * FROM wells WHERE organization_id = $1",
    current_org_id
)
.fetch_all(&pool)
.await?;

// Problems:
// 1. Single database failure = all tenants down
// 2. Noisy neighbor: One tenant's heavy queries slow down others
// 3. Client cannot choose database location
// 4. Compliance risk: All data in same database (harder to audit/isolate)
// 5. Psychological barrier: "My data is mixed with competitors' data"
```

**What We Need**:

- Each tenant has a completely separate database
- WellOS API dynamically connects to the correct tenant database per request
- Efficient connection pooling (don't create new DB connections on every request)
- Support for tenant databases hosted anywhere (Azure, AWS, on-prem)

---

## The Solution

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     WellOS API (Azure)                       │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Tenant Identification Middleware                        │  │
│  │  (Extract subdomain → Lookup tenant in master DB)        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Tenant Context (Stored in Request)                      │  │
│  │  - tenantId: "acmeoil-uuid"                              │  │
│  │  - slug: "acmeoil"                                       │  │
│  │  - databaseUrl: "postgresql://..."                       │  │
│  │  - connectionType: "AZURE_PRIVATE_LINK"                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  TenantDatabaseService                                   │  │
│  │  - Maintains connection pool per tenant                  │  │
│  │  - Lazy-loads pools on first request                     │  │
│  │  - Caches pools for subsequent requests                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                           ↓                                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Repository Layer (Hexagonal Architecture)               │  │
│  │  - WellRepository.findById(tenantId, wellId)             │  │
│  │  - ProductionDataRepository.getByDate(tenantId, date)    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                           ↓
        ┌──────────────────┴──────────────────┬──────────────────┐
        ↓                                     ↓                  ↓
┌───────────────────┐              ┌───────────────────┐  ┌──────────────────┐
│ Tenant 1 DB       │              │ Tenant 2 DB       │  │ Tenant 3 DB      │
│ (Azure East US)   │              │ (AWS US-West-2)   │  │ (On-Premises)    │
├───────────────────┤              ├───────────────────┤  ├──────────────────┤
│ acmeoil_prod      │              │ permianprod_db    │  │ texasenergy_db   │
│ - wells           │              │ - wells           │  │ - wells          │
│ - production_data │              │ - production_data │  │ - production_data│
│ - equipment       │              │ - equipment       │  │ - equipment      │
└───────────────────┘              └───────────────────┘  └──────────────────┘
```

---

## Implementation

### 1. Master Database Schema (Tenant Registry)

The master database stores tenant metadata and routing information:

```typescript
// apps/api/src/infrastructure/database/schema/master/tenants.schema.ts
import { pgTable, varchar, text, timestamp, jsonb, boolean } from 'drizzle-orm/pg-core';

export const tenants = pgTable('tenants', {
  id: varchar('id', { length: 255 }).primaryKey(),
  slug: varchar('slug', { length: 100 }).notNull().unique(), // "acmeoil"
  name: varchar('name', { length: 255 }).notNull(), // "ACME Oil & Gas"

  // Database connection info
  databaseUrl: text('database_url').notNull(), // Full PostgreSQL connection string
  connectionType: varchar('connection_type', { length: 50 }).notNull(),
  // "AZURE_PRIVATE_LINK" | "AWS_VPN" | "ON_PREMISES_VPN" | "PUBLIC_SSL"

  // Deployment info
  region: varchar('region', { length: 50 }).notNull(), // "azure-east-us", "aws-us-west-2", "on-premises"
  provider: varchar('provider', { length: 50 }).notNull(), // "AZURE" | "AWS" | "ON_PREMISES" | "GCP"

  // Feature flags & tier
  tier: varchar('tier', { length: 50 }).notNull().default('STARTER'),
  // "STARTER" | "PROFESSIONAL" | "ENTERPRISE"
  features: jsonb('features').notNull().default({}), // { predictiveMaintenance: true, ... }

  // Status & lifecycle
  status: varchar('status', { length: 50 }).notNull().default('ACTIVE'),
  // "ACTIVE" | "SUSPENDED" | "TRIAL" | "MIGRATING"
  trialEndsAt: timestamp('trial_ends_at'),

  // Metadata
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
  deletedAt: timestamp('deleted_at'), // Soft delete
});

export type Tenant = typeof tenants.$inferSelect;
```

**Example Data**:

```sql
INSERT INTO tenants (id, slug, name, database_url, connection_type, region, provider, tier, status)
VALUES
  (
    'acmeoil-uuid',
    'acmeoil',
    'ACME Oil & Gas',
    'postgresql://acmeoil_user:password@acmeoil-db.postgres.database.azure.com:5432/acmeoil_prod?sslmode=require',
    'AZURE_PRIVATE_LINK',
    'azure-east-us',
    'AZURE',
    'PROFESSIONAL',
    'ACTIVE'
  ),
  (
    'permianprod-uuid',
    'permianprod',
    'Permian Production LLC',
    'postgresql://permian_user:password@permian-db.us-west-2.rds.amazonaws.com:5432/permian_db?sslmode=require',
    'AWS_VPN',
    'aws-us-west-2',
    'AWS',
    'ENTERPRISE',
    'ACTIVE'
  ),
  (
    'texasenergy-uuid',
    'texasenergy',
    'Texas Energy Co.',
    'postgresql://texasenergy:password@192.168.50.10:5432/texasenergy_db?sslmode=require',
    'ON_PREMISES_VPN',
    'on-premises',
    'ON_PREMISES',
    'ENTERPRISE',
    'ACTIVE'
  );
```

---

### 2. Tenant Identification Middleware

Extracts tenant from subdomain and injects context into request:

```rust
// apps/api/src/presentation/middleware/tenant_identification.rs
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use crate::infrastructure::database::services::TenantConfigService;

pub struct TenantIdentificationMiddleware {
    tenant_config_service: TenantConfigService,
}

impl TenantIdentificationMiddleware {
    pub fn new(tenant_config_service: TenantConfigService) -> Self {
        Self { tenant_config_service }
    }

    pub async fn extract_tenant(
        &self,
        mut req: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract subdomain from hostname
        let hostname = req
            .headers()
            .get("host")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        let subdomain = hostname.split('.').next().unwrap_or("");

        // Handle special cases (marketing site, admin portal)
        if ["www", "api", "admin", "app"].contains(&subdomain) {
            // Public routes or admin panel - no tenant context needed
            return Ok(next.run(req).await);
        }

        // Lookup tenant from master database (with caching)
        let tenant = self.tenant_config_service
            .get_tenant_by_slug(subdomain)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let tenant = match tenant {
            Some(t) => t,
            None => {
                return Err(StatusCode::NOT_FOUND);
            }
        };

        if tenant.status != "ACTIVE" {
            return Err(StatusCode::NOT_FOUND);
        }

        // Inject tenant context into request extensions
        req.extensions_mut().insert(tenant);

        Ok(next.run(req).await)
    }
}
```

**Apply middleware globally**:

```rust
// apps/api/src/main.rs
use axum::{
    routing::get,
    middleware,
    Router,
};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let tenant_config_service = TenantConfigService::new().await;
    let tenant_middleware = TenantIdentificationMiddleware::new(tenant_config_service);

    let app = Router::new()
        .route("/wells", get(list_wells))
        .layer(middleware::from_fn(move |req, next| {
            tenant_middleware.extract_tenant(req, next)
        }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

---

### 3. Tenant Config Service (Master DB Queries)

Fetches tenant metadata from master database with caching:

```rust
// apps/api/src/infrastructure/database/services/tenant_config_service.rs
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

pub struct Tenant {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub database_url: String,
    pub connection_type: String,
    pub region: String,
    pub provider: String,
    pub tier: String,
    pub status: String,
}

struct CachedTenant {
    tenant: Tenant,
    cached_at: Instant,
}

pub struct TenantConfigService {
    master_pool: PgPool,
    tenant_cache: Arc<RwLock<HashMap<String, CachedTenant>>>,
}

impl TenantConfigService {
    pub async fn new(master_db_url: &str) -> Result<Self, sqlx::Error> {
        let master_pool = PgPool::connect(master_db_url).await?;

        Ok(Self {
            master_pool,
            tenant_cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>, sqlx::Error> {
        // Check cache first (avoid DB query on every request)
        {
            let cache = self.tenant_cache.read().await;
            if let Some(cached) = cache.get(slug) {
                // Invalidate after 5 minutes
                if cached.cached_at.elapsed() < Duration::from_secs(300) {
                    return Ok(Some(cached.tenant.clone()));
                }
            }
        }

        // Query master database
        let result = sqlx::query(
            "SELECT id, slug, name, database_url, connection_type, region, provider, tier, status
             FROM tenants
             WHERE slug = $1
             LIMIT 1"
        )
        .bind(slug)
        .fetch_optional(&self.master_pool)
        .await?;

        let tenant = result.map(|row| Tenant {
            id: row.get("id"),
            slug: row.get("slug"),
            name: row.get("name"),
            database_url: row.get("database_url"),
            connection_type: row.get("connection_type"),
            region: row.get("region"),
            provider: row.get("provider"),
            tier: row.get("tier"),
            status: row.get("status"),
        });

        // Cache tenant
        if let Some(ref t) = tenant {
            let mut cache = self.tenant_cache.write().await;
            cache.insert(
                slug.to_string(),
                CachedTenant {
                    tenant: t.clone(),
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(tenant)
    }

    pub async fn get_tenant_by_id(&self, tenant_id: &str) -> Result<Option<Tenant>, sqlx::Error> {
        let result = sqlx::query(
            "SELECT id, slug, name, database_url, connection_type, region, provider, tier, status
             FROM tenants
             WHERE id = $1
             LIMIT 1"
        )
        .bind(tenant_id)
        .fetch_optional(&self.master_pool)
        .await?;

        Ok(result.map(|row| Tenant {
            id: row.get("id"),
            slug: row.get("slug"),
            name: row.get("name"),
            database_url: row.get("database_url"),
            connection_type: row.get("connection_type"),
            region: row.get("region"),
            provider: row.get("provider"),
            tier: row.get("tier"),
            status: row.get("status"),
        }))
    }

    pub async fn get_all_active_tenants(&self) -> Result<Vec<Tenant>, sqlx::Error> {
        let results = sqlx::query(
            "SELECT id, slug, name, database_url, connection_type, region, provider, tier, status
             FROM tenants
             WHERE status = 'ACTIVE'"
        )
        .fetch_all(&self.master_pool)
        .await?;

        Ok(results.into_iter().map(|row| Tenant {
            id: row.get("id"),
            slug: row.get("slug"),
            name: row.get("name"),
            database_url: row.get("database_url"),
            connection_type: row.get("connection_type"),
            region: row.get("region"),
            provider: row.get("provider"),
            tier: row.get("tier"),
            status: row.get("status"),
        }).collect())
    }

    // Invalidate cache when tenant is updated
    pub async fn invalidate_cache(&self, slug: &str) {
        let mut cache = self.tenant_cache.write().await;
        cache.remove(slug);
    }
}

impl Clone for Tenant {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            slug: self.slug.clone(),
            name: self.name.clone(),
            database_url: self.database_url.clone(),
            connection_type: self.connection_type.clone(),
            region: self.region.clone(),
            provider: self.provider.clone(),
            tier: self.tier.clone(),
            status: self.status.clone(),
        }
    }
}
```

---

### 4. Tenant Database Service (Connection Pooling)

**Core Pattern**: Lazy-load connection pools per tenant, cache them for reuse.

```rust
// apps/api/src/infrastructure/database/services/tenant_database_service.rs
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions, PgSslMode};
use std::collections::HashMap;
use std::sync::Arc;
use std::str::FromStr;
use tokio::sync::RwLock;
use tracing::{error, info};
use crate::infrastructure::database::services::TenantConfigService;

pub struct TenantDatabaseService {
    tenant_config_service: Arc<TenantConfigService>,
    connection_pools: Arc<RwLock<HashMap<String, PgPool>>>,
}

impl TenantDatabaseService {
    pub fn new(tenant_config_service: Arc<TenantConfigService>) -> Self {
        Self {
            tenant_config_service,
            connection_pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get database connection pool for a specific tenant.
    /// Lazy-loads connection pool on first request, caches for subsequent requests.
    pub async fn get_tenant_database(&self, tenant_id: &str) -> Result<PgPool, sqlx::Error> {
        // Return cached connection pool if available
        {
            let pools = self.connection_pools.read().await;
            if let Some(pool) = pools.get(tenant_id) {
                return Ok(pool.clone());
            }
        }

        // Fetch tenant configuration
        let tenant = self.tenant_config_service
            .get_tenant_by_id(tenant_id)
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)?;

        info!(
            tenant_slug = %tenant.slug,
            connection_type = %tenant.connection_type,
            "Creating connection pool for tenant"
        );

        // Parse connection URL
        let connect_options = PgConnectOptions::from_str(&tenant.database_url)?
            .ssl_mode(Self::get_ssl_mode(&tenant.connection_type));

        // Create new PostgreSQL connection pool
        let pool = PgPoolOptions::new()
            .max_connections(20) // Max connections per tenant (tune based on load)
            .idle_timeout(std::time::Duration::from_secs(30)) // Close idle connections after 30s
            .acquire_timeout(std::time::Duration::from_secs(5)) // Timeout if connection takes >5s
            .connect_with(connect_options)
            .await?;

        // Cache the connection pool
        {
            let mut pools = self.connection_pools.write().await;
            pools.insert(tenant_id.to_string(), pool.clone());
        }

        info!(tenant_slug = %tenant.slug, "Connection pool created for tenant");

        Ok(pool)
    }

    /// Close connection pool for a specific tenant.
    /// Useful when tenant is deleted or migrated.
    pub async fn close_tenant_pool(&self, tenant_id: &str) {
        let mut pools = self.connection_pools.write().await;

        if let Some(pool) = pools.remove(tenant_id) {
            pool.close().await;
            info!(tenant_id = %tenant_id, "Connection pool closed for tenant");
        }
    }

    /// Close all connection pools (used during graceful shutdown).
    pub async fn close_all_pools(&self) {
        info!("Closing all tenant connection pools...");

        let mut pools = self.connection_pools.write().await;

        for (tenant_id, pool) in pools.drain() {
            pool.close().await;
            info!(tenant_id = %tenant_id, "Closed connection pool");
        }

        info!("All tenant connection pools closed");
    }

    /// Get SSL mode based on connection type.
    fn get_ssl_mode(connection_type: &str) -> PgSslMode {
        match connection_type {
            "AZURE_PRIVATE_LINK" | "AWS_VPN" | "ON_PREMISES_VPN" => {
                PgSslMode::Require // Require valid SSL cert
            }
            "PUBLIC_SSL" => PgSslMode::Require,
            _ => PgSslMode::Prefer, // Prefer SSL but allow non-SSL (local dev)
        }
    }
}

impl Clone for TenantDatabaseService {
    fn clone(&self) -> Self {
        Self {
            tenant_config_service: Arc::clone(&self.tenant_config_service),
            connection_pools: Arc::clone(&self.connection_pools),
        }
    }
}
```

**Graceful Shutdown Hook**:

```rust
// apps/api/src/main.rs
use tokio::signal;

#[tokio::main]
async fn main() {
    let tenant_db_service = Arc::new(TenantDatabaseService::new(tenant_config_service));

    // ... setup routes

    // Graceful shutdown: Close all database connections
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Received shutdown signal, closing all connections");
        tenant_db_service.close_all_pools().await;
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

---

### 5. Repository Pattern with Tenant Context

Every repository method **requires** `tenantId` parameter:

```rust
// apps/api/src/infrastructure/database/repositories/well_repository.rs
use sqlx::{PgPool, Row};
use crate::domain::repositories::WellRepository as IWellRepository;
use crate::domain::wells::Well;
use crate::infrastructure::database::services::TenantDatabaseService;

pub struct WellRepository {
    tenant_db_service: TenantDatabaseService,
}

impl WellRepository {
    pub fn new(tenant_db_service: TenantDatabaseService) -> Self {
        Self { tenant_db_service }
    }
}

#[async_trait::async_trait]
impl IWellRepository for WellRepository {
    async fn find_by_id(&self, tenant_id: &str, well_id: &str) -> Result<Option<Well>, sqlx::Error> {
        let pool = self.tenant_db_service.get_tenant_database(tenant_id).await?;

        let result = sqlx::query(
            "SELECT id, name, api_number, latitude, longitude, status, created_at
             FROM wells
             WHERE id = $1 AND deleted_at IS NULL
             LIMIT 1"
        )
        .bind(well_id)
        .fetch_optional(&pool)
        .await?;

        Ok(result.map(|row| self.to_domain(row)))
    }

    async fn find_all(&self, tenant_id: &str) -> Result<Vec<Well>, sqlx::Error> {
        let pool = self.tenant_db_service.get_tenant_database(tenant_id).await?;

        let results = sqlx::query(
            "SELECT id, name, api_number, latitude, longitude, status, created_at
             FROM wells
             WHERE deleted_at IS NULL"
        )
        .fetch_all(&pool)
        .await?;

        Ok(results.into_iter().map(|row| self.to_domain(row)).collect())
    }

    async fn save(&self, tenant_id: &str, well: &Well) -> Result<(), sqlx::Error> {
        let pool = self.tenant_db_service.get_tenant_database(tenant_id).await?;

        sqlx::query(
            "INSERT INTO wells (id, name, api_number, latitude, longitude, status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
             ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                api_number = EXCLUDED.api_number,
                latitude = EXCLUDED.latitude,
                longitude = EXCLUDED.longitude,
                status = EXCLUDED.status,
                updated_at = NOW()"
        )
        .bind(&well.id)
        .bind(&well.name)
        .bind(&well.api_number)
        .bind(well.location.latitude)
        .bind(well.location.longitude)
        .bind(&well.status)
        .bind(well.created_at)
        .execute(&pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, tenant_id: &str, well_id: &str) -> Result<(), sqlx::Error> {
        let pool = self.tenant_db_service.get_tenant_database(tenant_id).await?;

        // Soft delete
        sqlx::query("UPDATE wells SET deleted_at = NOW() WHERE id = $1")
            .bind(well_id)
            .execute(&pool)
            .await?;

        Ok(())
    }
}

impl WellRepository {
    fn to_domain(&self, row: sqlx::postgres::PgRow) -> Well {
        // Map database row to domain entity
        Well {
            id: row.get("id"),
            name: row.get("name"),
            api_number: row.get("api_number"),
            location: Location {
                latitude: row.get("latitude"),
                longitude: row.get("longitude"),
            },
            status: row.get("status"),
            created_at: row.get("created_at"),
        }
    }
}
```

**Critical Rule**: ❌ **NEVER** allow a repository method without `tenantId`. This prevents accidental cross-tenant data access.

---

### 6. Controller with Tenant Context

Extract tenant from request extensions:

```rust
// apps/api/src/presentation/extractors/tenant_context.rs
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use crate::infrastructure::database::services::Tenant;

pub struct TenantContext(pub Tenant);

#[async_trait::async_trait]
impl<S> FromRequestParts<S> for TenantContext
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract tenant from request extensions (injected by TenantIdentificationMiddleware)
        parts
            .extensions
            .get::<Tenant>()
            .cloned()
            .map(TenantContext)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
```

**Usage in Handler**:

```rust
// apps/api/src/presentation/wells/wells_handlers.rs
use axum::{
    extract::Path,
    http::StatusCode,
    Json,
};
use crate::presentation::extractors::TenantContext;
use crate::application::wells::queries::GetWellByIdQuery;
use crate::domain::wells::Well;

pub async fn get_well_by_id(
    TenantContext(tenant): TenantContext, // Tenant context from subdomain
    Path(well_id): Path<String>,
    // Inject query bus or repository here
) -> Result<Json<Well>, StatusCode> {
    let query = GetWellByIdQuery {
        tenant_id: tenant.id,
        well_id,
    };

    // Execute query
    let well = execute_query(query)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match well {
        Some(w) => Ok(Json(w)),
        None => Err(StatusCode::NOT_FOUND),
    }
}
```

**Flow**:

1. Request: `GET https://acmeoil.onwellos.com/wells/well-123`
2. Middleware extracts subdomain `acmeoil`
3. Middleware fetches tenant from master DB
4. Middleware injects tenant into `req.tenant`
5. Controller extracts tenant via `@TenantContext()` decorator
6. Query handler uses `tenantId` to get correct database connection

---

## Benefits

### 1. **Complete Data Isolation**

Each tenant's data is in a separate database. No risk of accidental cross-tenant queries.

```typescript
// ✅ With Database-Per-Tenant
const acmeoilDb = await tenantDbService.getTenantDatabase('acmeoil-uuid');
const permianDb = await tenantDbService.getTenantDatabase('permianprod-uuid');

// These are completely separate databases
// No way for acmeoil to access permianprod data
```

### 2. **Client Data Sovereignty**

Clients choose where their database lives:

```typescript
// Client A: Azure East US (close to their headquarters)
tenants.databaseUrl = 'postgresql://...@azure-east-us.postgres.database.azure.com/...';

// Client B: AWS US-West-2 (they already use AWS)
tenants.databaseUrl = 'postgresql://...@us-west-2.rds.amazonaws.com/...';

// Client C: On-premises (behind their firewall)
tenants.databaseUrl = 'postgresql://...@192.168.50.10/...';
```

### 3. **Independent Scaling**

Large tenants get dedicated database servers:

```typescript
// Small tenant: Shared Azure PostgreSQL (Burstable B1ms, $30/month)
// Large tenant: Dedicated Azure PostgreSQL (General Purpose 8 vCores, $500/month)
```

### 4. **Easier Compliance**

Each tenant's database can have different:

- Backup retention policies
- Encryption keys
- Audit logging levels
- Geographic location (data residency requirements)

### 5. **Simplified Migrations**

Migrate one tenant at a time (not all-or-nothing):

```typescript
// Migrate acmeoil from on-prem to Azure
// 1. Create new Azure PostgreSQL database
// 2. Dump/restore data
// 3. Update tenants.databaseUrl in master DB
// 4. TenantDatabaseService automatically uses new connection
// 5. Other tenants unaffected
```

---

## Performance Considerations

### Connection Pool Sizing

**Rule of Thumb**: `max_connections = (number_of_replicas * pool_size) + buffer`

```typescript
// Example: 3 API replicas, 20 connections per replica
// Total connections needed: 3 * 20 = 60 connections

const pool = new Pool({
  max: 20, // Per-tenant pool size
  min: 5, // Keep 5 connections warm
  idleTimeoutMillis: 30000, // Close idle connections after 30s
});
```

**Azure PostgreSQL Flexible Server limits**:

- Burstable B1ms: 50 max connections
- General Purpose 2 vCores: 859 max connections
- General Purpose 4 vCores: 1719 max connections

**Recommendation**: For shared tenant databases (multiple small tenants on one server), use **lower pool sizes** (5-10 connections per tenant).

### Caching Strategy

Cache tenant metadata aggressively:

```typescript
// ✅ Good: Cache tenant for 5 minutes
this.tenantCache.set(slug, tenant);
setTimeout(() => this.tenantCache.delete(slug), 5 * 60 * 1000);

// ❌ Bad: Query master DB on every request
const tenant = await this.masterDb.select()...
```

**Redis Caching (Optional)**:

```typescript
// For production, use Redis for cross-process caching
const cachedTenant = await redis.get(`tenant:${slug}`);
if (cachedTenant) return JSON.parse(cachedTenant);

// Cache for 10 minutes
await redis.setex(`tenant:${slug}`, 600, JSON.stringify(tenant));
```

---

## Anti-Patterns

### ❌ **Don't: Forget Tenant Context**

```typescript
// ❌ BAD: No tenantId parameter
async findById(wellId: string): Promise<Well> {
  // Which tenant's database should we query?
  const db = await this.tenantDbService.getTenantDatabase(???);
}

// ✅ GOOD: Always require tenantId
async findById(tenantId: string, wellId: string): Promise<Well> {
  const db = await this.tenantDbService.getTenantDatabase(tenantId);
}
```

### ❌ **Don't: Create New Pools on Every Request**

```typescript
// ❌ BAD: New pool for every request (connection exhaustion)
async findById(tenantId: string, wellId: string): Promise<Well> {
  const pool = new Pool({ connectionString: tenant.databaseUrl });
  const db = drizzle(pool);
  // ...
}

// ✅ GOOD: Reuse cached connection pool
async findById(tenantId: string, wellId: string): Promise<Well> {
  const db = await this.tenantDbService.getTenantDatabase(tenantId); // Cached
  // ...
}
```

### ❌ **Don't: Store Tenant Databases in Code**

```typescript
// ❌ BAD: Hardcoded database URLs
const tenantDatabases = {
  acmeoil: 'postgresql://...',
  permianprod: 'postgresql://...',
};

// ✅ GOOD: Store in master database (dynamic, supports new tenants without code changes)
const tenant = await this.tenantConfigService.getTenantBySlug(slug);
```

---

## Testing Strategy

### Unit Tests: Mock TenantDatabaseService

```typescript
// apps/api/test/unit/repositories/well.repository.spec.ts
describe('WellRepository', () => {
  let repository: WellRepository;
  let mockTenantDbService: jest.Mocked<TenantDatabaseService>;

  beforeEach(() => {
    mockTenantDbService = {
      getTenantDatabase: jest.fn(),
    } as any;

    repository = new WellRepository(mockTenantDbService);
  });

  it('should find well by ID', async () => {
    const mockDb = {
      select: jest.fn().mockReturnThis(),
      from: jest.fn().mockReturnThis(),
      where: jest.fn().mockReturnThis(),
      limit: jest.fn().mockResolvedValue([{ id: 'well-123', name: 'Well A' }]),
    };

    mockTenantDbService.getTenantDatabase.mockResolvedValue(mockDb as any);

    const well = await repository.findById('tenant-uuid', 'well-123');

    expect(well).toBeDefined();
    expect(well.name).toBe('Well A');
    expect(mockTenantDbService.getTenantDatabase).toHaveBeenCalledWith('tenant-uuid');
  });
});
```

### Integration Tests: Use Test Tenant Database

```typescript
// apps/api/test/integration/wells/wells.e2e-spec.ts
describe('Wells API (E2E)', () => {
  let app: INestApplication;
  let tenantDbService: TenantDatabaseService;

  const TEST_TENANT_ID = 'test-tenant-uuid';

  beforeAll(async () => {
    const moduleFixture = await Test.createTestingModule({
      imports: [AppModule],
    }).compile();

    app = moduleFixture.createNestApplication();
    tenantDbService = app.get(TenantDatabaseService);

    // Create test tenant in master DB
    await createTestTenant(TEST_TENANT_ID);

    // Run migrations on test tenant database
    await runMigrationsForTenant(TEST_TENANT_ID);

    await app.init();
  });

  afterAll(async () => {
    // Clean up test tenant database
    await dropTestTenantDatabase(TEST_TENANT_ID);
    await app.close();
  });

  it('should create a well for test tenant', async () => {
    const response = await request(app.getHttpServer())
      .post('/wells')
      .set('Host', 'testtenant.onwellos.com') // Simulate subdomain
      .send({ name: 'Test Well', apiNumber: 'TX-12345' })
      .expect(201);

    expect(response.body.name).toBe('Test Well');
  });
});
```

---

## Migration Strategy

### Onboarding New Tenant

1. **Client provisions database** (Azure, AWS, or on-prem)
2. **Client shares connection string** with WellOS
3. **WellOS creates tenant record** in master database:

```typescript
await masterDb.insert(tenants).values({
  id: uuidv4(),
  slug: 'newclient',
  name: 'New Client Oil & Gas',
  databaseUrl: 'postgresql://...', // Provided by client
  connectionType: 'AZURE_PRIVATE_LINK',
  region: 'azure-west-us',
  provider: 'AZURE',
  tier: 'PROFESSIONAL',
  status: 'ACTIVE',
});
```

4. **Run migrations on tenant database**:

```bash
pnpm --filter=api db:migrate --tenant=newclient
```

5. **Test connection**:

```bash
pnpm --filter=api db:test-connection --tenant=newclient
```

6. **Tenant is live** (subdomain automatically works via middleware)

---

## Related Patterns

- **Repository Pattern**: All data access goes through repositories with `tenantId` parameter
- **Unit of Work**: Manage transactions across tenant-scoped repositories
- **Connection Pooling**: Reuse database connections for performance
- **Middleware Pattern**: Extract tenant context from subdomain
- **Strategy Pattern**: Support multiple database providers (Azure, AWS, on-prem)

---

## When to Use This Pattern

### ✅ Use Database-Per-Tenant When:

- **Clients demand data sovereignty** (choose where data lives)
- **Compliance requires data isolation** (healthcare, finance, oil & gas)
- **Tenants have different scaling needs** (small vs. large operators)
- **You support hybrid cloud** (on-prem + cloud databases)
- **Psychological isolation matters** ("My data is NOT mixed with competitors")

### ❌ Don't Use Database-Per-Tenant When:

- **Tenants are small/similar** (row-level security is simpler)
- **You need cross-tenant reporting** (hard with separate databases)
- **You can't manage connection pooling complexity**
- **Database costs are prohibitive** (every tenant = separate database cost)

---

## Real-World Example: WellOS

**Scenario**: ACME Oil & Gas (50 wells) wants to use WellOS but requires their database to stay on-premises for security.

**Setup**:

```yaml
1. ACME provisions on-prem PostgreSQL:
   - Server: 192.168.50.10 (internal network)
   - Database: acmeoil_prod

2. ACME configures VPN to Azure:
   - Site-to-Site VPN between ACME office and Azure VPN Gateway

3. WellOS creates tenant record:
   - Slug: acmeoil
   - Database URL: postgresql://...@192.168.50.10:5432/acmeoil_prod
   - Connection Type: ON_PREMISES_VPN

4. WellOS runs migrations over VPN

5. ACME accesses: https://acmeoil.onwellos.com
   - TenantIdentificationMiddleware extracts "acmeoil"
   - TenantDatabaseService connects to 192.168.50.10 via VPN
   - All data stays on ACME's premises
```

**Sales Advantage**: "Your data never leaves your building. We just connect to it securely."

---

## Summary

The **Database-Per-Tenant Multi-Tenancy Pattern** provides:

1. **Complete data isolation** (separate databases per tenant)
2. **Client data sovereignty** (tenants choose database location)
3. **Efficient connection pooling** (lazy-load, cache pools)
4. **Hybrid cloud support** (Azure, AWS, on-prem)
5. **Independent scaling** (small tenants share, large tenants get dedicated resources)

**Critical Implementation Points**:

- Store tenant metadata in master database
- Lazy-load connection pools (don't create upfront)
- Always require `tenantId` parameter in repositories
- Use middleware to extract tenant from subdomain
- Support multiple connection types (Private Link, VPN, SSL)

This pattern is essential for WellOS because oil & gas operators demand control over their data and won't accept shared database architectures.

---

**Related Documentation**:

- [Azure Production Architecture](../deployment/azure-production-architecture.md)
- [Offline Batch Sync Pattern](./70-Offline-Batch-Sync-Pattern.md)
- [Conflict Resolution Pattern](./71-Conflict-Resolution-Pattern.md)
