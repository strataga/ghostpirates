# Database Seeding Pattern

## Context

Multi-tenant applications need to seed initial data for development, testing, and production environments. In WellOS, we have two types of databases that need seeding:

- **Master Database**: Platform-wide data (admin users, tenants, billing)
- **Tenant Databases**: Tenant-specific data (wells, users, production data)

## Problem

How do you structure seed files for a multi-database, multi-tenant application while keeping the seeding process simple, maintainable, and environment-aware?

## Solution

Use a Rust-based seeding approach with SQLx and separate seed files for master and tenant databases:

### Structure

```
apps/api/src/infrastructure/database/
├── seeds/
│   ├── master.rs         # Seeds master database
│   └── tenant.rs         # Seeds tenant database(s)
└── migrations/
    ├── master/
    │   └── *.sql         # Master schema migrations
    └── tenant/
        └── *.sql         # Tenant schema migrations
```

### Implementation

#### 1. **Master Seed File (Rust + SQLx)**

```rust
// src/infrastructure/database/seeds/master.rs
use sqlx::PgPool;
use bcrypt::{hash, DEFAULT_COST};
use uuid::Uuid;

pub async fn seed_master(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Starting master database seed...\n");

    // 1. Create Super Admin
    let password_hash = hash("SecurePassword2025!", DEFAULT_COST)?;

    let admin_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO admin_users (email, password_hash, role)
        VALUES ($1, $2, $3)
        ON CONFLICT (email) DO NOTHING
        RETURNING id
        "#
    )
    .bind("admin@platform.com")
    .bind(&password_hash)
    .bind("SUPER_ADMIN")
    .fetch_optional(pool)
    .await?
    .unwrap_or(Uuid::new_v4());

    // 2. Create Sample Tenants
    sqlx::query(
        r#"
        INSERT INTO tenants (slug, subdomain, name, database_name, status, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (slug) DO NOTHING
        "#
    )
    .bind("acme")
    .bind("acme")
    .bind("ACME Corp")
    .bind("acme_db")
    .bind("ACTIVE")
    .bind(admin_id)
    .execute(pool)
    .await?;

    println!("✅ Master database seed completed!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("MASTER_DATABASE_URL")?;
    let pool = PgPool::connect(&database_url).await?;

    seed_master(&pool).await?;

    pool.close().await;
    Ok(())
}
```

#### 2. **Tenant Seed File (Rust + SQLx)**

```rust
// src/infrastructure/database/seeds/tenant.rs
use sqlx::PgPool;
use std::env;

pub async fn seed_tenant(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌱 Starting tenant database seed...\n");

    // Seed tenant-specific data
    sqlx::query(
        r#"
        INSERT INTO wells (api_number, name, status)
        VALUES ($1, $2, $3)
        ON CONFLICT (api_number) DO NOTHING
        "#
    )
    .bind("API-42-123-45678")
    .bind("Sample Well #1")
    .bind("ACTIVE")
    .execute(pool)
    .await?;

    println!("✅ Tenant database seed completed!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("TENANT_SEED_DATABASE_URL")
        .unwrap_or("postgresql://user:pass@localhost:5432/default_tenant".to_string());

    println!("📦 Target: {}\n", database_url.split('@').nth(1).unwrap_or(""));

    let pool = PgPool::connect(&database_url).await?;
    seed_tenant(&pool).await?;
    pool.close().await;

    Ok(())
}
```

#### 3. **Cargo Configuration**

```toml
# Cargo.toml
[[bin]]
name = "seed-master"
path = "src/infrastructure/database/seeds/master.rs"

[[bin]]
name = "seed-tenant"
path = "src/infrastructure/database/seeds/tenant.rs"
```

#### 4. **Environment Configuration**

```bash
# .env
MASTER_DATABASE_URL=postgresql://user:pass@localhost:5432/master
TENANT_SEED_DATABASE_URL=postgresql://user:pass@localhost:5432/tenant_db
```

## Usage

### Development Setup

```bash
# 1. Seed master database (creates admin users, tenants)
cargo run --bin seed-master

# 2. Seed a specific tenant database
TENANT_SEED_DATABASE_URL=postgresql://user:pass@localhost:5432/acme_db cargo run --bin seed-tenant

# Or use default tenant
cargo run --bin seed-tenant
```

### CI/CD Pipeline

```bash
# Automated seeding in tests
MASTER_DATABASE_URL=postgresql://test_db cargo run --bin seed-master
TENANT_SEED_DATABASE_URL=postgresql://tenant_test_db cargo run --bin seed-tenant
```

## Key Patterns

### 1. **Idempotent Seeds**

Use `ON CONFLICT DO NOTHING` to make seeds safe to run multiple times:

```rust
sqlx::query(
    r#"
    INSERT INTO admin_users (email, password_hash, role)
    VALUES ($1, $2, $3)
    ON CONFLICT (email) DO NOTHING
    "#
)
.bind("admin@app.com")
.bind(&password_hash)
.bind("ADMIN")
.execute(pool)
.await?;
```

### 2. **Environment-Aware**

```rust
let database_url = env::var("TENANT_SEED_DATABASE_URL")
    .or_else(|_| env::var("DATABASE_URL"))
    .unwrap_or("postgresql://localhost/default".to_string());
```

### 3. **Executable Binaries**

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPool::connect(&database_url).await?;
    seed_master(&pool).await?;
    pool.close().await;
    Ok(())
}
```

### 4. **Modular Seeds**

For large seed files, split into modules:

```rust
// seeds/master.rs
mod seed_admins;
mod seed_tenants;
mod seed_billing;

pub async fn seed_master(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    seed_admins::run(pool).await?;
    seed_tenants::run(pool).await?;
    seed_billing::run(pool).await?;
    Ok(())
}
```

## Benefits

1. **Simple**: Rust binaries with SQLx - no ORM overhead
2. **Flexible**: Environment variables control which database to seed
3. **Idempotent**: Safe to run multiple times (ON CONFLICT DO NOTHING)
4. **Type-Safe**: Compile-time SQL validation with SQLx
5. **Multi-Tenant**: Separate seeds for master vs tenant databases
6. **Performant**: Native Rust execution

## Trade-offs

- **Manual Execution**: Requires explicit cargo run (vs automatic on migrate)
- **SQL Writing**: Must write raw SQL (no ORM query builder)
- **Environment Management**: Must manage DATABASE_URL yourself

## When to Use

- ✅ Multi-tenant applications with separate databases
- ✅ Development data setup
- ✅ Test fixtures
- ✅ Initial production data (admin users, system configs)
- ✅ Demo environments

## When NOT to Use

- ❌ Production data migrations (use migrations instead)
- ❌ Large data imports (use bulk import tools)
- ❌ Real customer data (use proper data import flows)

## Related Patterns

- [Migration-Based Schema Management Pattern](./73-Migration-Based-Schema-Management-Pattern.md)
- [Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)

## Example: WellOS

```bash
# Master DB: Create admin, 3 tenants (wellos, acme, demo)
cargo run --bin seed-master

# Seed ACME tenant with sample wells
TENANT_SEED_DATABASE_URL=postgresql://wellos:wellos@localhost:5432/acme_wellos \
  cargo run --bin seed-tenant

# Seed Demo tenant
TENANT_SEED_DATABASE_URL=postgresql://wellos:wellos@localhost:5432/demo_wellos \
  cargo run --bin seed-tenant
```

## References

- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [SQLx Compile-time Verification](https://github.com/launchbadge/sqlx#sqlx-is-not-an-orm)
- [PostgreSQL INSERT ON CONFLICT](https://www.postgresql.org/docs/current/sql-insert.html#SQL-ON-CONFLICT)
