# Migration-Based Schema Management Pattern

**Category**: Database
**Type**: Infrastructure
**Complexity**: Intermediate
**Status**: Recommended

---

## Problem

In a multi-tenant SaaS application with evolving database schemas:

1. **Data Loss Risk**: Using `db:push` overwrites schema without preserving data
2. **No Version Control**: Can't track schema changes over time
3. **No Rollback**: Can't undo failed schema deployments
4. **Team Conflicts**: Developers can't coordinate schema changes
5. **Production Risk**: Schema changes applied without testing or review
6. **Multi-Tenant Complexity**: Need to apply schema changes to hundreds of tenant databases

---

## Solution

Use **migration-based schema management** with SQL migrations and SQLx to generate version-controlled, incremental schema change scripts that can be reviewed, tested, and safely applied to production databases.

### Core Principles

1. **Schema as Code**: Define schema in TypeScript, generate SQL migrations
2. **Version Control**: Commit migration files to git with application code
3. **Incremental Changes**: Each migration represents one logical schema change
4. **Idempotency**: Migrations track what's been applied (no duplicate runs)
5. **Transactional**: PostgreSQL transactions ensure all-or-nothing application
6. **Reviewable**: Migration SQL can be code-reviewed before deployment

---

## Implementation

### 1. SQLx Configuration (Multi-Tenant)

**Master Database Config** (`.env`):

```bash
# Master database connection
MASTER_DATABASE_URL=postgresql://localhost/master

# Tenant template database connection
TENANT_TEMPLATE_DATABASE_URL=postgresql://localhost/tenant_template
```

**SQLx Migration Directories**:

```
apps/api/src/infrastructure/database/
├── migrations/
│   ├── master/
│   │   ├── 0001_initial_schema.sql
│   │   └── 0002_add_tenants.sql
│   └── tenant/
│       ├── 0001_initial_schema.sql
│       └── 0002_add_wells.sql
```

### 2. Migration Scripts (Rust + SQLx)

**Master Migration Runner** (`scripts/migrate-master.rs`):

```rust
use sqlx::{postgres::PgPool, migrate::MigrateDatabase};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = env::var("MASTER_DATABASE_URL")?;
    let pool = PgPool::connect(&connection_string).await?;

    println!("🔄 Running master database migrations...");

    sqlx::migrate!("./migrations/master")
        .run(&pool)
        .await?;

    println!("✅ Master migrations completed");
    pool.close().await;
    Ok(())
}
```

**Tenant Migration Runner** (`scripts/migrate-tenant.rs`):

```rust
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;

struct Tenant {
    id: String,
    name: String,
    database_url: String,
    database_type: String,
}

async fn get_tenants(master_pool: &PgPool) -> Result<Vec<Tenant>, sqlx::Error> {
    let tenants = sqlx::query_as!(
        Tenant,
        r#"
        SELECT id, name, database_url, database_type
        FROM tenants
        WHERE deleted_at IS NULL
        "#
    )
    .fetch_all(master_pool)
    .await?;

    Ok(tenants)
}

async fn migrate_tenant(tenant: &Tenant) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Migrating: {}", tenant.name);

    // Only migrate PostgreSQL databases
    if tenant.database_type != "POSTGRESQL" {
        println!("⏭️  Skipping ({} uses adapter)", tenant.database_type);
        return Ok(());
    }

    let tenant_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&tenant.database_url)
        .await?;

    sqlx::migrate!("./migrations/tenant")
        .run(&tenant_pool)
        .await?;

    println!("✅ Completed: {}", tenant.name);
    tenant_pool.close().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let master_url = env::var("MASTER_DATABASE_URL")?;
    let master_pool = PgPool::connect(&master_url).await?;

    let tenants = get_tenants(&master_pool).await?;
    println!("📊 Migrating {} tenants", tenants.len());

    for tenant in tenants {
        migrate_tenant(&tenant).await?;
    }

    println!("✅ All migrations completed");
    master_pool.close().await;
    Ok(())
}
```

### 3. Package Scripts

**package.json**:

```bash
# Create migration files manually or use SQLx CLI
cargo install sqlx-cli --no-default-features --features postgres

# Generate migration
sqlx migrate add <migration_name> --source migrations/master
sqlx migrate add <migration_name> --source migrations/tenant

# Run migrations
cargo run --bin migrate-master
cargo run --bin migrate-tenant
# Or: sqlx migrate run --source migrations/master
```

### 4. Workflow

**Development** (adding a new table):

```typescript
// 1. Define schema
export const equipment = pgTable('equipment', {
  id: uuid('id').primaryKey().defaultRandom(),
  wellId: uuid('well_id')
    .notNull()
    .references(() => wells.id),
  name: varchar('name', { length: 255 }).notNull(),
  type: varchar('type', { length: 100 }).notNull(),
  status: varchar('status', { length: 50 }).notNull().default('ACTIVE'),
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});
```

```bash
# 2. Create migration file
sqlx migrate add add_equipment_table --source migrations/tenant

# 3. Edit generated SQL file with schema changes
# migrations/tenant/<timestamp>_add_equipment_table.sql

# 4. Apply migration locally
cargo run --bin migrate-tenant

# 5. Commit migration file
git add migrations/tenant/
git commit -m "feat(db): add equipment table"
```

**Production Deployment**:

```bash
# 1. Backup databases (Azure automated backups)

# 2. Apply migrations
MASTER_DATABASE_URL=$PROD_MASTER cargo run --bin migrate-master
MASTER_DATABASE_URL=$PROD_MASTER cargo run --bin migrate-tenant

# 3. Deploy application code (expects new schema)
```

---

## Examples

### Example 1: Adding a Column

**Schema Change**:

```typescript
export const wells = pgTable('wells', {
  // ... existing columns
  operatorEmail: varchar('operator_email', { length: 255 }), // NEW
});
```

**Generated Migration** (`0004_add_operator_email.sql`):

```sql
ALTER TABLE "wells" ADD COLUMN "operator_email" varchar(255);
```

**Apply**:

```bash
sqlx migrate add add_operator_email --source migrations/tenant
# Edit migration file, then:
cargo run --bin migrate-tenant
```

### Example 2: Adding a NOT NULL Column (Safe Pattern)

**Problem**: Can't add NOT NULL column to table with existing rows.

**Solution**: 3-step migration:

**Step 1**: Add column as nullable:

```typescript
export const wells = pgTable('wells', {
  operatorEmail: varchar('operator_email', { length: 255 }), // Nullable first
});
```

```bash
sqlx migrate add add_operator_email_nullable --source migrations/tenant
cargo run --bin migrate-tenant
```

**Step 2**: Backfill data (application code or SQL):

```typescript
// Application code backfills data
await wellRepository.updateAll({ operatorEmail: 'default@example.com' });
```

**Step 3**: Add NOT NULL constraint:

```typescript
export const wells = pgTable('wells', {
  operatorEmail: varchar('operator_email', { length: 255 }).notNull(), // Now NOT NULL
});
```

```bash
sqlx migrate add set_operator_email_not_null --source migrations/tenant
cargo run --bin migrate-tenant
```

### Example 3: Renaming a Column (Safe Pattern)

**Problem**: Direct rename causes data loss.

**Solution**: 3-step migration with dual-write period.

**Step 1**: Add new column:

```typescript
export const wells = pgTable('wells', {
  operator: varchar('operator', { length: 255 }), // OLD
  operatorName: varchar('operator_name', { length: 255 }), // NEW
});
```

**Step 2**: Dual-write application code (writes to both columns):

```typescript
await wellRepository.update(wellId, {
  operator: name, // OLD column
  operatorName: name, // NEW column (duplicate data)
});
```

**Step 3**: Drop old column (after full backfill):

```typescript
export const wells = pgTable('wells', {
  // operator removed
  operatorName: varchar('operator_name', { length: 255 }).notNull(),
});
```

### Example 4: Adding PostgreSQL Enum (Idempotent Pattern)

**Problem**: PostgreSQL enum types can't be created if they already exist. This is problematic in multi-tenant systems where some databases may already have the enum while others don't.

**Solution**: Use PostgreSQL DO block with exception handling to make enum creation idempotent.

**Schema Change (SQL Migration)**:

```sql
-- migrations/tenant/<timestamp>_add_well_type_enum.sql
-- Define enum type
CREATE TYPE well_type AS ENUM ('HORIZONTAL', 'VERTICAL', 'DIRECTIONAL');

-- Add column with enum type
ALTER TABLE wells ADD COLUMN well_type well_type NOT NULL DEFAULT 'HORIZONTAL';

-- Create index
CREATE INDEX wells_well_type_idx ON wells(well_type);
```

**Manual Migration** (`<timestamp>_add_well_type.sql`):

**Idempotent Migration for Multi-Tenant Safety**:

```sql
-- Add well_type enum (idempotent)
DO $$ BEGIN
 CREATE TYPE "public"."well_type" AS ENUM('HORIZONTAL', 'VERTICAL', 'DIRECTIONAL');
EXCEPTION
 WHEN duplicate_object THEN null;
END $$;
--> statement-breakpoint

-- Add well_type column to wells table (idempotent)
ALTER TABLE "wells" ADD COLUMN IF NOT EXISTS "well_type" "well_type" DEFAULT 'HORIZONTAL' NOT NULL;
--> statement-breakpoint

-- Create index on well_type for performance (idempotent)
CREATE INDEX IF NOT EXISTS "wells_well_type_idx" ON "wells" USING btree ("well_type");
```

**Key Changes**:
1. **DO block** wraps CREATE TYPE with `EXCEPTION WHEN duplicate_object THEN null;` to handle existing enum
2. **IF NOT EXISTS** for column addition (PostgreSQL 9.6+)
3. **IF NOT EXISTS** for index creation

**Domain Entity Update (Rust)**:

Update the domain entity to include the new field:

```rust
#[derive(Debug, Clone, Copy)]
pub enum WellType {
    Horizontal,
    Vertical,
    Directional,
}

pub struct Well {
    // ... existing fields
    well_type: WellType,
}

impl Well {
    pub fn well_type(&self) -> WellType {
        self.well_type
    }
}
```

**Repository Mapper Update (Rust + SQLx)**:

```rust
use sqlx::FromRow;

#[derive(FromRow)]
struct WellRow {
    // ... existing fields
    well_type: String,
}

// Mapper: Database → Domain
fn to_domain(row: WellRow) -> Well {
    Well {
        // ... existing fields
        well_type: match row.well_type.as_str() {
            "HORIZONTAL" => WellType::Horizontal,
            "VERTICAL" => WellType::Vertical,
            "DIRECTIONAL" => WellType::Directional,
            _ => WellType::Horizontal,
        },
    }
}

// Mapper: Domain → Database
fn to_persistence(well: &Well) -> String {
    match well.well_type() {
        WellType::Horizontal => "HORIZONTAL".to_string(),
        WellType::Vertical => "VERTICAL".to_string(),
        WellType::Directional => "DIRECTIONAL".to_string(),
    }
}
```

**Apply**:

```bash
sqlx migrate add add_well_type_enum --source migrations/tenant
# Edit migration SQL for idempotency (above)
cargo run --bin migrate-tenant  # Apply to all tenant databases
```

**Benefits of This Pattern**:
- ✅ Safe to run multiple times (idempotent)
- ✅ Works across tenant databases with different schema states
- ✅ No errors if enum/column/index already exists
- ✅ Database-level type safety (PostgreSQL enforces enum values)
- ✅ Performance optimization (indexed enum column for queries)

**Use Cases**:
- Status fields (ACTIVE, INACTIVE, PENDING)
- Type classifications (well types, equipment types, commodity types)
- Priority levels (HIGH, MEDIUM, LOW)
- Any field with a fixed set of allowed values

---

## Benefits

### 1. Zero Data Loss

- Migrations preserve existing data during schema changes
- Transactions ensure all-or-nothing application (PostgreSQL)
- Rollback capability if migration fails

### 2. Version Control

- Migration files tracked in git alongside code
- Full schema history (who, when, why)
- Code review for schema changes (SQL review)

### 3. Team Collaboration

- Merge conflicts resolved in migration files (not runtime)
- Consistent schema across all environments
- Clear ownership of schema changes (git blame)

### 4. Production Safety

- Test migrations in staging before production
- Dry-run capability (generate migration, review SQL, don't apply)
- Automated CI/CD integration (apply on deploy)

### 5. Multi-Tenant Scale

- Apply schema changes to hundreds of tenant databases
- Track migration status per tenant
- Skip non-PostgreSQL tenants (adapter pattern)

---

## Drawbacks

### 1. Complexity

- More steps than `db:push` (generate → review → apply)
- Requires discipline (never skip migration generation)

**Mitigation**: Clear documentation, CI/CD enforcement

### 2. Merge Conflicts

- Two developers modifying schema simultaneously
- Migration file timestamps conflict

**Mitigation**: Communicate schema changes, regenerate migrations after merge

### 3. Multi-Tenant Performance

- Sequential migration of 1000+ tenants takes time
- Network latency for remote databases

**Mitigation**: Parallelize with connection pooling, use job queue (Bull/BullMQ)

---

## When to Use

✅ **Use migration-based schema management when**:

- Working in a team (multiple developers)
- Deploying to production (data loss unacceptable)
- Need schema version control (audit trail)
- Multi-tenant architecture (consistent schema across tenants)
- Require rollback capability (undo failed deployments)

❌ **Use `db:push` when**:

- Solo local development (rapid prototyping)
- Throwaway databases (test data, no production consequences)
- Initial schema design (frequent changes, no existing data)

**Rule of Thumb**: If data matters, use migrations.

---

## Troubleshooting

### Error: Connection Pool Exhaustion

**Cause**: Too many concurrent migration connections.

**Solution**: Use connection limits with SQLx:

```rust
// ✅ Correct: Limited connection pool
use sqlx::postgres::PgPoolOptions;

let pool = PgPoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(30))
    .connect(&database_url)
    .await?;
```

**Why it matters**: Migration scripts should use minimal connections to avoid pool exhaustion and allow concurrent application access.

### Error: "relation already exists"

**Cause**: Table was created via `db:push` or previous migration, but Drizzle's migration tracking isn't aware.

**Solution**: Drop and recreate tenant databases in development:

```bash
# Development only - destroys all data!
psql -U superuser -c "DROP DATABASE tenant_db;"
psql -U superuser -c "CREATE DATABASE tenant_db OWNER tenant_user;"
cargo run --bin migrate-tenant
```

**Production**: Never drop databases. Instead, manually track which migrations have been applied.

### Error: "database does not exist"

**Cause**: Tenant database hasn't been created yet.

**Solution**: Migration scripts only apply schema changes - they don't create databases. Create databases first:

```bash
# Create tenant database
psql -U superuser -c "CREATE DATABASE acme_wellos OWNER wellos;"

# Then run migrations
cargo run --bin migrate-tenant
```

---

## Related Patterns

- **[Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)** - Tenant isolation strategy
- **[Repository Pattern](./02-Repository-Pattern.md)** - Database access abstraction
- **[Audit Log Pattern](./29-Audit-Log-Pattern.md)** - Track schema change audit trail
- **[Database Seeding Pattern](./74-Database-Seeding-Pattern.md)** - Populate databases with initial/test data

---

## References

- [SQLx Migrations](https://docs.rs/sqlx/latest/sqlx/migrate/index.html)
- [SQLx CLI](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
- [Evolutionary Database Design (Martin Fowler)](https://martinfowler.com/articles/evodb.html)
- [PostgreSQL Transactional DDL](https://wiki.postgresql.org/wiki/Transactional_DDL_in_PostgreSQL:_A_Competitive_Analysis)

---

## Implementation Checklist

- [ ] Create separate configs for master and tenant databases
- [ ] Implement migration runner scripts with proper error handling
- [ ] Add migration npm scripts to package.json
- [ ] Generate initial migrations for existing schemas
- [ ] Test migration workflow locally (generate → apply → verify)
- [ ] Document workflow in project README
- [ ] Integrate into CI/CD pipeline (automated migration on deploy)
- [ ] Train team on migration best practices
- [ ] Enforce "no db:push in production" rule (CI/CD check)
- [ ] Set up database backups before migration (Azure automated backups)

---

**Tags**: #database #migrations #schema-management #multi-tenancy #sqlx #rust #postgresql
