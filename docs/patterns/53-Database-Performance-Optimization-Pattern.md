# Database Performance Optimization Pattern

**Pattern Number:** 53
**Category:** Performance & Infrastructure
**Last Updated:** October 10, 2025

---

## Overview

Systematic approach to optimizing database performance for production workloads through strategic indexing, connection pool tuning, and query optimization.

**Key Principle:** Database performance optimization is not a one-time task but an iterative process driven by load testing and production metrics.

---

## Problem Statement

Applications often suffer from poor database performance under load due to:

1. **Missing Indexes**: Sequential scans on foreign keys and frequently queried columns
2. **Inadequate Connection Pooling**: Default connection limits insufficient for concurrent load
3. **Unoptimized Queries**: N+1 queries, missing JOINs, or inefficient filtering
4. **Lack of Monitoring**: No visibility into slow queries or bottlenecks

**Symptoms:**

- High P95/P99 response times under load
- Database CPU spikes during peak usage
- Connection pool exhaustion errors
- Timeout errors on database operations

---

## Solution

### 1. Strategic Index Management

**Define indexes in SQL migrations**:

```sql
-- apps/scada-ingestion/migrations/0001_create_users_table.sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    organization_id TEXT NOT NULL REFERENCES organizations(id),
    is_active BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Foreign key index - essential for JOINs
CREATE INDEX users_organization_id_idx ON users(organization_id);

-- Composite partial index - optimizes complex queries
CREATE INDEX users_org_active_idx
ON users(organization_id, is_active)
WHERE deleted_at IS NULL;

-- Soft delete index - fast filtering
CREATE INDEX users_deleted_at_idx ON users(deleted_at);
```

```rust
// apps/scada-ingestion/src/infrastructure/database/models/user.rs
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: String,
    pub organization_id: String,
    pub is_active: bool,
    pub deleted_at: Option<chrono::NaiveDateTime>,
    pub created_at: chrono::NaiveDateTime,
}
```

**Index Types to Consider:**

| Index Type            | Use Case                   | Example                    |
| --------------------- | -------------------------- | -------------------------- |
| **Foreign Key Index** | JOINs, lookups             | `users.organization_id`    |
| **Partial Index**     | Queries with WHERE clauses | `WHERE deleted_at IS NULL` |
| **Composite Index**   | Multi-column queries       | `(org_id, is_active)`      |
| **DESC Index**        | Time-series queries        | `created_at DESC`          |

### 2. Connection Pool Optimization

**Calculate optimal pool size** based on concurrent load:

```rust
// apps/scada-ingestion/src/infrastructure/database/pool.rs
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};

pub async fn create_pool(database_url: &str) -> Result<sqlx::PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(25) // 2:1 ratio: 50 concurrent users → 25 connections
        .idle_timeout(Duration::from_secs(30)) // Close idle connections after 30s
        .acquire_timeout(Duration::from_secs(10)) // Fail fast on connection issues
        .max_lifetime(Duration::from_secs(1800)) // 30 min max connection age (prevents leaks)
        .connect(database_url)
        .await
}
```

**Sizing Formula:**

```
Pool Size = (Peak Concurrent Requests × 0.5) + Buffer
Example: (50 × 0.5) + 5 = 30 connections
```

### 3. Query Optimization Patterns

**Prevent N+1 Queries** with proper eager loading:

```rust
// ❌ BAD: N+1 query problem
let users = sqlx::query_as!(UserRow, "SELECT * FROM users")
    .fetch_all(&pool)
    .await?;

for user in users {
    // Separate query for each user!
    let org = sqlx::query!(
        "SELECT * FROM organizations WHERE id = $1",
        user.organization_id
    )
    .fetch_one(&pool)
    .await?;
}

// ✅ GOOD: Single query with JOIN
let users_with_orgs = sqlx::query!(
    r#"
    SELECT
        u.*,
        o.id as org_id,
        o.name as org_name
    FROM users u
    LEFT JOIN organizations o ON u.organization_id = o.id
    "#
)
.fetch_all(&pool)
.await?;
```

---

## Implementation Checklist

### Phase 1: Analysis

- [ ] Run load tests to establish baseline metrics
- [ ] Identify slow queries using `EXPLAIN ANALYZE`
- [ ] Check for missing indexes on foreign keys
- [ ] Review connection pool utilization

### Phase 2: Indexing

- [ ] Add indexes to SQL migration files
- [ ] Index all foreign keys
- [ ] Add composite indexes for common query patterns
- [ ] Use partial indexes for filtered queries
- [ ] Apply indexes: `sqlx migrate run` (dev)

### Phase 3: Connection Pool

- [ ] Calculate optimal pool size based on load
- [ ] Configure timeouts and lifetime limits
- [ ] Enable prepared statements
- [ ] Monitor pool metrics in production

### Phase 4: Validation

- [ ] Re-run load tests
- [ ] Verify P95/P99 metrics improved
- [ ] Check database CPU/memory usage
- [ ] Monitor index usage: `pg_stat_user_indexes`

---

## Performance Metrics

**Target Benchmarks:**

| Metric                        | Target    | Production Ready |
| ----------------------------- | --------- | ---------------- |
| **P95 Response Time**         | < 2,000ms | < 500ms          |
| **P99 Response Time**         | < 5,000ms | < 1,000ms        |
| **Error Rate**                | < 1%      | < 0.1%           |
| **DB Connection Utilization** | < 80%     | < 70%            |

---

## Index Management Best Practices

### ✅ DO

1. **Define indexes in schema files** - Single source of truth
2. **Index all foreign keys** - Essential for JOINs
3. **Use composite indexes** - For multi-column queries
4. **Add partial indexes** - For filtered queries (WHERE clauses)
5. **Monitor index usage** - Remove unused indexes
6. **Use IF NOT EXISTS** - Safe production deployments

### ❌ DON'T

1. **Don't create indexes in separate SQL files** - Hard to track
2. **Don't over-index** - Each index has write overhead
3. **Don't index low-cardinality columns** - Unless partial index
4. **Don't forget DESC** - Time-series queries need it
5. **Don't skip load testing** - Production will expose issues

---

## SQLx-Specific Patterns

### Partial Indexes (WHERE clause)

```sql
-- Partial index with WHERE clause
CREATE INDEX active_users_idx
ON users(is_active)
WHERE deleted_at IS NULL;
```

### DESC Indexes (Time-series)

```sql
-- Descending index for time-series queries
CREATE INDEX created_at_idx ON users(created_at DESC);
```

### Composite Indexes

```sql
-- Simple multi-column index
CREATE INDEX org_status_idx ON users(organization_id, status);
```

---

## Deployment Strategy

### Development Environment

**Use `sqlx migrate run`** for applying migrations:

```bash
sqlx migrate run --database-url postgresql://localhost/wellos_dev
```

- ✅ Fast iteration
- ✅ Version-controlled migrations
- ✅ Perfect for prototyping

### Production Environment

**Option 1: SQL Script (Recommended for hotfixes)**

```bash
psql -d production -f scripts/performance-indexes.sql
```

- ✅ Safe, idempotent (IF NOT EXISTS)
- ✅ No migration system dependency
- ✅ Fast deployment

**Option 2: SQLx Migration System**

```bash
# Apply pending migrations
sqlx migrate run --database-url $DATABASE_URL
```

- ✅ Full audit trail
- ✅ Rollback capability with `sqlx migrate revert`
- ✅ Compile-time verification of migrations

---

## Monitoring & Maintenance

### Index Health Queries

```sql
-- Check index usage
SELECT
  schemaname,
  tablename,
  indexname,
  idx_scan as scans,
  idx_tup_read as tuples_read
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY idx_scan DESC;

-- Find unused indexes
SELECT
  schemaname,
  tablename,
  indexname
FROM pg_stat_user_indexes
WHERE idx_scan = 0
  AND indexrelname NOT LIKE '%_pkey';

-- Check index sizes
SELECT
  tablename,
  indexname,
  pg_size_pretty(pg_relation_size(indexrelid)) as size
FROM pg_stat_user_indexes
WHERE schemaname = 'public'
ORDER BY pg_relation_size(indexrelid) DESC;
```

### Connection Pool Monitoring

```rust
// Log pool stats periodically
use tokio::time::{interval, Duration};

async fn monitor_pool(pool: &sqlx::PgPool) {
    let mut interval = interval(Duration::from_secs(60));

    loop {
        interval.tick().await;

        let size = pool.size();
        let idle = pool.num_idle();

        tracing::info!(
            max_connections = size,
            active_connections = size - idle,
            idle_connections = idle,
            "Connection pool stats"
        );
    }
}
```

---

## Real-World Example: WellOS PSA

**Challenge:** P95 response time of 5,272ms under 50 concurrent users

**Optimizations Applied:**

1. **Added 9 indexes** to schema files:
   - Foreign keys: `users.organization_id`, `user_roles.user_id`, `audit_logs.user_id`
   - Composite: `users(organization_id, is_active)` with partial filter
   - Time-series: `audit_logs.created_at DESC`

2. **Tuned connection pool**:
   - Increased from 10 → 25 connections
   - Enabled prepared statements
   - Added timeout configurations

3. **Fixed audit log race condition**:
   - Added retry logic with exponential backoff
   - Handled FK constraint violations gracefully

**Results:**

- **11.7x performance improvement** (P95: 5,272ms → 450ms)
- **100% error reduction** (81 errors → 0)
- **Production ready** (all thresholds met)

---

## Related Patterns

- **Pattern 41**: Database Constraint Race Condition Pattern
- **Pattern 06**: Repository Pattern (data access abstraction)
- **Pattern 09**: Unit of Work Pattern (transaction management)

---

## References

- [PostgreSQL Index Documentation](https://www.postgresql.org/docs/current/indexes.html)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Connection Pooling Best Practices](https://wiki.postgresql.org/wiki/Number_Of_Database_Connections)

---

## Summary

Database performance optimization requires a systematic approach:

1. **Measure first** - Establish baseline with load testing
2. **Index strategically** - Foreign keys, composite indexes, partial indexes
3. **Tune connection pool** - Size for peak load with proper timeouts
4. **Monitor continuously** - Track index usage and query performance
5. **Define in migrations** - Keep indexes in SQL migrations for source control

**Key Insight:** The fastest query is the one that uses an index-only scan. Every foreign key should have an index, and every common query pattern should be analyzed for potential composite indexes. SQLx provides compile-time SQL verification to catch errors before runtime.
