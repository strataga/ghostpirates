# Pattern 95: Rust SAGA Orchestration Pattern for Multi-Tenant Operations

**Version**: 1.0
**Last Updated**: November 3, 2025
**Category**: Distributed Transactions & Workflow Orchestration
**Complexity**: Advanced
**Status**: Recommended

---

## Table of Contents

1. [Overview](#overview)
2. [Problem](#problem)
3. [Solution](#solution)
4. [WellOS Use Cases](#wellos-use-cases)
5. [Rust Implementation](#rust-implementation)
6. [SAGA Types](#saga-types)
7. [Orchestration vs Choreography](#orchestration-vs-choreography)
8. [Compensation Strategies](#compensation-strategies)
9. [State Persistence](#state-persistence)
10. [Error Handling](#error-handling)
11. [Testing](#testing)
12. [Benefits](#benefits)
13. [Trade-offs](#trade-offs)
14. [Related Patterns](#related-patterns)

---

## Overview

The **SAGA Pattern** manages distributed transactions across multiple services or databases without requiring two-phase commit (2PC). A SAGA is a sequence of local transactions where each step can be compensated (rolled back) if a later step fails.

**Core Principle**: Maintain data consistency across distributed systems through compensating transactions instead of locking.

```
Traditional ACID Transaction (Impossible in Distributed Systems)
┌────────────────────────────────────────────────────────────┐
│ BEGIN TRANSACTION                                          │
│   INSERT INTO tenant_db.wells ...                          │
│   INSERT INTO master_db.audit_log ...                      │
│   POST to Azure Blob Storage API                           │
│ COMMIT                                                     │
└────────────────────────────────────────────────────────────┘
❌ Can't span multiple databases/services

SAGA Pattern (Forward + Compensating Actions)
┌────────────────────────────────────────────────────────────┐
│ Step 1: Create well in tenant DB                          │
│   ↓ Success                                                │
│ Step 2: Log to master DB audit log                        │
│   ↓ Success                                                │
│ Step 3: Upload well diagram to Azure Blob                 │
│   ↓ FAILURE! ❌                                            │
│ Compensate Step 2: Delete audit log entry                 │
│ Compensate Step 1: Delete well from tenant DB             │
└────────────────────────────────────────────────────────────┘
✅ Eventual consistency with rollback capability
```

---

## Problem

WellOS faces several distributed transaction challenges:

### 1. Tenant Provisioning (Master DB + Tenant DB + Azure Resources)

```rust
// ❌ Can't do this atomically across services
BEGIN TRANSACTION
  INSERT INTO master.tenants ...           // Master database
  CREATE DATABASE tenant_acme;             // Tenant database
  POST to Azure API (create container);    // Azure Blob Storage
  POST to Azure API (create Service Bus);  // Azure Service Bus
COMMIT
```

**What if Azure API fails?** Database records exist but no Azure resources.

### 2. Offline Field Data Sync (Local SQLite → Cloud PostgreSQL)

```rust
// ❌ Can't guarantee atomicity
BEGIN TRANSACTION
  INSERT INTO cloud_db.field_entries ...   // Cloud PostgreSQL
  UPDATE local_db.sync_queue ...           // Local SQLite
  DELETE FROM local_db.pending_entries ... // Local SQLite
COMMIT
```

**What if cloud insert succeeds but local update fails?** Duplicate data on next sync.

### 3. Invoice Generation (Production Data + Accounting + External Sync)

```rust
// ❌ Multi-service transaction
BEGIN TRANSACTION
  INSERT INTO invoices ...                 // Tenant database
  UPDATE production_data SET invoiced=true // Tenant database
  POST to QuickBooks API                   // External system
COMMIT
```

**What if QuickBooks API is down?** Invoice created but not synced.

### 4. SCADA Data Ingestion (Validation → Storage → Alarm Check)

```rust
// ❌ Complex multi-step workflow
BEGIN TRANSACTION
  INSERT INTO scada_readings ...           // Time-series database
  UPDATE well_status ...                   // Tenant database
  POST to Alarm Service API                // Alarm microservice
  POST to WebSocket Gateway                // Real-time updates
COMMIT
```

**What if alarm service fails?** Data stored but no alerts triggered.

---

## Solution

Implement **SAGA orchestration** in Rust to manage multi-step workflows with compensating transactions.

### SAGA Execution Flow

```
┌─────────────────────────────────────────────────────────────┐
│                  SAGA Orchestrator                          │
│  - Executes steps sequentially                              │
│  - Tracks completed steps                                   │
│  - Triggers compensation on failure                         │
└─────────────────────────────────────────────────────────────┘
                        │
        ┌───────────────┼───────────────┐
        ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │ Step 1  │    │ Step 2  │    │ Step 3  │
    │ Forward │    │ Forward │    │ Forward │
    └─────────┘    └─────────┘    └─────────┘
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │Compensate│   │Compensate│   │Compensate│
    └─────────┘    └─────────┘    └─────────┘
```

---

## WellOS Use Cases

### 1. Tenant Provisioning (Sprint 1)

**Scenario**: Create new tenant account with master DB record, tenant database, and Azure resources.

**SAGA Steps**:
1. Create tenant record in master database
2. Create tenant PostgreSQL database
3. Run tenant database migrations
4. Create Azure Blob Storage container
5. Create Azure Service Bus queue
6. Send welcome email

**Compensation**:
- Delete Azure resources
- Drop tenant database
- Delete master DB record

### 2. Offline Field Data Batch Sync (Sprint 4)

**Scenario**: Sync 1000+ field entries from mobile SQLite to cloud PostgreSQL at end of shift.

**SAGA Steps**:
1. Validate field entries (business rules)
2. Insert entries into cloud database (batch)
3. Mark local entries as synced
4. Delete local entries from pending queue
5. Trigger well status recalculation

**Compensation**:
- Delete cloud entries
- Unmark local entries
- Restore pending queue

### 3. Invoice Generation with QuickBooks Sync (Sprint 5)

**Scenario**: Generate invoice from production data and sync to QuickBooks.

**SAGA Steps**:
1. Create invoice in tenant database
2. Mark production volumes as invoiced
3. Sync invoice to QuickBooks
4. Send invoice email to client

**Compensation**:
- Delete QuickBooks invoice
- Unmark production volumes
- Delete invoice

### 4. SCADA Alarm Processing (Sprint 3-5)

**Scenario**: Process abnormal SCADA reading and trigger alarms.

**SAGA Steps**:
1. Insert SCADA reading into time-series database
2. Update well status (if threshold exceeded)
3. Create alarm record
4. Send alarm notification (email/SMS/Slack)
5. Broadcast alarm via WebSocket

**Compensation**:
- Delete alarm notification
- Delete alarm record
- Revert well status
- Delete SCADA reading

---

## Rust Implementation

### Core SAGA Types

```rust
// apps/scada-ingestion/src/saga/mod.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// SAGA execution context
/// Carries state across all steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaContext {
    pub saga_id: Uuid,
    pub correlation_id: String,
    pub tenant_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_steps: Vec<String>,
    pub current_step: Option<String>,
    pub metadata: serde_json::Value,
}

impl SagaContext {
    pub fn new(tenant_id: String, correlation_id: String) -> Self {
        Self {
            saga_id: Uuid::new_v4(),
            correlation_id,
            tenant_id,
            started_at: Utc::now(),
            completed_steps: Vec::new(),
            current_step: None,
            metadata: serde_json::json!({}),
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: serde_json::Value) {
        if let Some(obj) = self.metadata.as_object_mut() {
            obj.insert(key.to_string(), value);
        }
    }

    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

/// SAGA step status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SagaStatus {
    Pending,
    InProgress,
    Completed,
    Compensating,
    Compensated,
    Failed,
}

/// SAGA step definition
#[async_trait]
pub trait SagaStep: Send + Sync {
    /// Step name (for logging and tracking)
    fn name(&self) -> &str;

    /// Execute the step (forward action)
    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>>;

    /// Compensate the step (rollback action)
    /// Must be idempotent (safe to call multiple times)
    async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>>;

    /// Optional: Check if step can be skipped
    async fn should_skip(&self, _ctx: &SagaContext) -> bool {
        false
    }
}

/// SAGA orchestrator
pub struct SagaOrchestrator {
    steps: Vec<Box<dyn SagaStep>>,
}

impl SagaOrchestrator {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add_step(&mut self, step: Box<dyn SagaStep>) {
        self.steps.push(step);
    }

    pub async fn execute(&self, mut ctx: SagaContext) -> Result<SagaContext, Box<dyn Error>> {
        log::info!("Starting SAGA execution: {}", ctx.saga_id);

        for (index, step) in self.steps.iter().enumerate() {
            ctx.current_step = Some(step.name().to_string());

            // Check if step should be skipped
            if step.should_skip(&ctx).await {
                log::info!("Skipping step {}: {}", index + 1, step.name());
                continue;
            }

            log::info!("Executing step {}: {}", index + 1, step.name());

            match step.execute(&mut ctx).await {
                Ok(_) => {
                    ctx.completed_steps.push(step.name().to_string());
                    log::info!("Step {} completed: {}", index + 1, step.name());
                }
                Err(error) => {
                    log::error!("Step {} failed: {} - {}", index + 1, step.name(), error);
                    return self.compensate(ctx, index).await;
                }
            }
        }

        ctx.current_step = None;
        log::info!("SAGA completed successfully: {}", ctx.saga_id);
        Ok(ctx)
    }

    async fn compensate(
        &self,
        mut ctx: SagaContext,
        failed_step_index: usize,
    ) -> Result<SagaContext, Box<dyn Error>> {
        log::warn!("Starting compensation for SAGA: {}", ctx.saga_id);

        // Compensate completed steps in reverse order
        for step_name in ctx.completed_steps.iter().rev() {
            let step = self.steps.iter()
                .find(|s| s.name() == step_name)
                .expect("Step not found");

            log::info!("Compensating step: {}", step.name());

            match step.compensate(&ctx).await {
                Ok(_) => {
                    log::info!("Compensation successful: {}", step.name());
                }
                Err(error) => {
                    log::error!(
                        "Compensation failed for step: {} - {}",
                        step.name(),
                        error
                    );
                    // Continue compensating other steps even if one fails
                    // Log to monitoring system for manual intervention
                }
            }
        }

        Err(format!("SAGA failed at step: {}", failed_step_index).into())
    }
}
```

---

## SAGA Types

### Orchestration-Based SAGA (Recommended)

**Central orchestrator** coordinates all steps.

```rust
// apps/scada-ingestion/src/saga/tenant_provisioning.rs
use super::{SagaContext, SagaStep, SagaOrchestrator};
use async_trait::async_trait;
use std::error::Error;

// ===== Step 1: Create Tenant in Master DB =====
struct CreateTenantStep {
    master_db: Arc<MasterDatabase>,
}

#[async_trait]
impl SagaStep for CreateTenantStep {
    fn name(&self) -> &str {
        "create_tenant"
    }

    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = ctx.tenant_id.clone();
        let tenant_name = ctx.get_metadata("tenant_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing tenant_name in metadata")?;

        // Insert tenant into master database
        let tenant = self.master_db.create_tenant(&tenant_id, tenant_name).await?;

        // Store tenant_id in context for next steps
        ctx.add_metadata("tenant_db_id", serde_json::json!(tenant.id));

        Ok(())
    }

    async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = &ctx.tenant_id;

        // Delete tenant from master database
        self.master_db.delete_tenant(tenant_id).await?;

        log::info!("Compensated: Deleted tenant from master DB: {}", tenant_id);
        Ok(())
    }
}

// ===== Step 2: Create Tenant Database =====
struct CreateTenantDatabaseStep {
    db_provisioner: Arc<DatabaseProvisioner>,
}

#[async_trait]
impl SagaStep for CreateTenantDatabaseStep {
    fn name(&self) -> &str {
        "create_tenant_database"
    }

    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = &ctx.tenant_id;

        // Create PostgreSQL database for tenant
        let database_url = self.db_provisioner.create_database(tenant_id).await?;

        // Store database URL in context
        ctx.add_metadata("database_url", serde_json::json!(database_url));

        Ok(())
    }

    async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = &ctx.tenant_id;

        // Drop tenant database
        self.db_provisioner.drop_database(tenant_id).await?;

        log::info!("Compensated: Dropped tenant database: {}", tenant_id);
        Ok(())
    }
}

// ===== Step 3: Run Database Migrations =====
struct RunMigrationsStep {
    migration_runner: Arc<MigrationRunner>,
}

#[async_trait]
impl SagaStep for RunMigrationsStep {
    fn name(&self) -> &str {
        "run_migrations"
    }

    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let database_url = ctx.get_metadata("database_url")
            .and_then(|v| v.as_str())
            .ok_or("Missing database_url in metadata")?;

        // Run Drizzle migrations on tenant database
        self.migration_runner.run_migrations(database_url).await?;

        Ok(())
    }

    async fn compensate(&self, _ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
        // Migrations are compensated by dropping the database (previous step)
        // No additional action needed
        Ok(())
    }
}

// ===== Step 4: Create Azure Blob Storage Container =====
struct CreateAzureBlobContainerStep {
    azure_client: Arc<AzureBlobClient>,
}

#[async_trait]
impl SagaStep for CreateAzureBlobContainerStep {
    fn name(&self) -> &str {
        "create_azure_blob_container"
    }

    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = &ctx.tenant_id;
        let container_name = format!("tenant-{}", tenant_id);

        // Create Azure Blob Storage container
        self.azure_client.create_container(&container_name).await?;

        ctx.add_metadata("blob_container", serde_json::json!(container_name));

        Ok(())
    }

    async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
        let container_name = ctx.get_metadata("blob_container")
            .and_then(|v| v.as_str())
            .ok_or("Missing blob_container in metadata")?;

        // Delete Azure Blob Storage container
        self.azure_client.delete_container(container_name).await?;

        log::info!("Compensated: Deleted Azure Blob container: {}", container_name);
        Ok(())
    }
}

// ===== Step 5: Send Welcome Email =====
struct SendWelcomeEmailStep {
    email_service: Arc<EmailService>,
}

#[async_trait]
impl SagaStep for SendWelcomeEmailStep {
    fn name(&self) -> &str {
        "send_welcome_email"
    }

    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_name = ctx.get_metadata("tenant_name")
            .and_then(|v| v.as_str())
            .ok_or("Missing tenant_name in metadata")?;

        let admin_email = ctx.get_metadata("admin_email")
            .and_then(|v| v.as_str())
            .ok_or("Missing admin_email in metadata")?;

        // Send welcome email
        self.email_service.send_welcome_email(admin_email, tenant_name).await?;

        Ok(())
    }

    async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
        // Can't unsend email, but could send "Account creation failed" email
        let admin_email = ctx.get_metadata("admin_email")
            .and_then(|v| v.as_str())
            .ok_or("Missing admin_email in metadata")?;

        self.email_service.send_provisioning_failed_email(admin_email).await?;

        log::info!("Compensated: Sent provisioning failed email");
        Ok(())
    }
}

// ===== SAGA Orchestrator Usage =====
pub async fn provision_tenant(
    tenant_id: String,
    tenant_name: String,
    admin_email: String,
    master_db: Arc<MasterDatabase>,
    db_provisioner: Arc<DatabaseProvisioner>,
    migration_runner: Arc<MigrationRunner>,
    azure_client: Arc<AzureBlobClient>,
    email_service: Arc<EmailService>,
) -> Result<SagaContext, Box<dyn Error>> {
    // Create SAGA context
    let mut ctx = SagaContext::new(tenant_id.clone(), format!("provision-{}", tenant_id));
    ctx.add_metadata("tenant_name", serde_json::json!(tenant_name));
    ctx.add_metadata("admin_email", serde_json::json!(admin_email));

    // Build SAGA orchestrator
    let mut saga = SagaOrchestrator::new();
    saga.add_step(Box::new(CreateTenantStep { master_db }));
    saga.add_step(Box::new(CreateTenantDatabaseStep { db_provisioner }));
    saga.add_step(Box::new(RunMigrationsStep { migration_runner }));
    saga.add_step(Box::new(CreateAzureBlobContainerStep { azure_client }));
    saga.add_step(Box::new(SendWelcomeEmailStep { email_service }));

    // Execute SAGA
    saga.execute(ctx).await
}
```

---

## Orchestration vs Choreography

### Orchestration (Recommended for WellOS)

**Pros**:
- ✅ Clear business logic flow
- ✅ Easy to understand and debug
- ✅ Centralized error handling
- ✅ Easy to add/remove steps

**Cons**:
- ❌ Orchestrator is single point of failure
- ❌ Tighter coupling to orchestrator

**Use When**: Complex multi-step workflows (tenant provisioning, invoice generation)

### Choreography (Event-Driven)

**Pros**:
- ✅ Loose coupling
- ✅ No single point of failure
- ✅ Scales well

**Cons**:
- ❌ Hard to understand flow
- ❌ Difficult to track SAGA state
- ❌ Cyclic dependency risk

**Use When**: Simple event-driven workflows (SCADA alarm notifications)

---

## Compensation Strategies

### 1. Backward Recovery (Undo)

Restore exact previous state.

```rust
async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
    // Restore previous state from context
    let previous_status = ctx.get_metadata("previous_well_status")
        .and_then(|v| v.as_str())
        .ok_or("Missing previous_well_status")?;

    self.well_repo.update_status(&ctx.tenant_id, previous_status).await?;
    Ok(())
}
```

### 2. Forward Recovery (Semantic Compensation)

Logically undo without restoring exact state.

```rust
async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
    // Mark invoice as cancelled (don't delete)
    let invoice_id = ctx.get_metadata("invoice_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing invoice_id")?;

    self.invoice_repo.mark_as_cancelled(invoice_id).await?;
    Ok(())
}
```

### 3. No Compensation (Irreversible Actions)

Some actions can't be undone (e.g., sending email).

```rust
async fn compensate(&self, ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
    // Send "Please disregard previous email" message
    let email = ctx.get_metadata("admin_email")
        .and_then(|v| v.as_str())
        .ok_or("Missing admin_email")?;

    self.email_service.send_cancellation_notice(email).await?;
    Ok(())
}
```

---

## State Persistence

### PostgreSQL SAGA State Table

```sql
-- apps/api/src/infrastructure/database/schema/saga_state.sql
CREATE TABLE saga_state (
    saga_id UUID PRIMARY KEY,
    saga_type VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(100) NOT NULL,
    correlation_id VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL,
    context JSONB NOT NULL,
    completed_steps TEXT[] NOT NULL DEFAULT '{}',
    current_step VARCHAR(100),
    error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_saga_state_tenant_id ON saga_state(tenant_id);
CREATE INDEX idx_saga_state_correlation_id ON saga_state(correlation_id);
CREATE INDEX idx_saga_state_status ON saga_state(status);
```

### Persist SAGA State in Rust

```rust
// apps/scada-ingestion/src/saga/state_manager.rs
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde_json;

pub struct SagaStateManager {
    pool: PgPool,
}

impl SagaStateManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save_saga(
        &self,
        ctx: &SagaContext,
        status: SagaStatus,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query(
            r#"
            INSERT INTO saga_state (
                saga_id, saga_type, tenant_id, correlation_id,
                status, context, completed_steps, current_step
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (saga_id)
            DO UPDATE SET
                status = $5,
                context = $6,
                completed_steps = $7,
                current_step = $8,
                updated_at = NOW()
            "#,
        )
        .bind(ctx.saga_id)
        .bind("tenant_provisioning")
        .bind(&ctx.tenant_id)
        .bind(&ctx.correlation_id)
        .bind(format!("{:?}", status))
        .bind(serde_json::to_value(&ctx.metadata)?)
        .bind(&ctx.completed_steps)
        .bind(&ctx.current_step)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_saga(
        &self,
        saga_id: Uuid,
    ) -> Result<SagaContext, Box<dyn std::error::Error>> {
        let row = sqlx::query(
            r#"
            SELECT tenant_id, correlation_id, context, completed_steps, current_step, created_at
            FROM saga_state
            WHERE saga_id = $1
            "#,
        )
        .bind(saga_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(SagaContext {
            saga_id,
            correlation_id: row.get("correlation_id"),
            tenant_id: row.get("tenant_id"),
            started_at: row.get("created_at"),
            completed_steps: row.get("completed_steps"),
            current_step: row.get("current_step"),
            metadata: row.get("context"),
        })
    }
}
```

---

## Error Handling

### Retry with Exponential Backoff

```rust
// apps/scada-ingestion/src/saga/retry.rs
use tokio::time::{sleep, Duration};
use std::error::Error;

pub async fn retry_step<F, Fut>(
    step_name: &str,
    mut operation: F,
    max_retries: u32,
) -> Result<(), Box<dyn Error>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<(), Box<dyn Error>>>,
{
    let mut retries = 0;

    loop {
        match operation().await {
            Ok(()) => return Ok(()),
            Err(error) => {
                retries += 1;
                if retries > max_retries {
                    return Err(error);
                }

                // Exponential backoff: 1s, 2s, 4s, 8s
                let delay = Duration::from_secs(2_u64.pow(retries - 1));
                log::warn!(
                    "Step '{}' failed (attempt {}/{}), retrying in {:?}: {}",
                    step_name,
                    retries,
                    max_retries,
                    delay,
                    error
                );

                sleep(delay).await;
            }
        }
    }
}
```

### Idempotent Steps

Ensure steps can be safely retried.

```rust
#[async_trait]
impl SagaStep for CreateTenantStep {
    async fn execute(&self, ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
        let tenant_id = &ctx.tenant_id;

        // Check if tenant already exists (idempotency)
        if self.master_db.tenant_exists(tenant_id).await? {
            log::info!("Tenant already exists, skipping creation: {}", tenant_id);
            return Ok(());
        }

        // Create tenant
        self.master_db.create_tenant(tenant_id, "Tenant Name").await?;
        Ok(())
    }

    // ... compensate implementation
}
```

---

## Testing

### Unit Test: SAGA Orchestrator

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_saga_success() {
        let mut saga = SagaOrchestrator::new();

        saga.add_step(Box::new(MockStep::new("step1", Ok(()), Ok(()))));
        saga.add_step(Box::new(MockStep::new("step2", Ok(()), Ok(()))));
        saga.add_step(Box::new(MockStep::new("step3", Ok(()), Ok(()))));

        let ctx = SagaContext::new("tenant-123".to_string(), "test-correlation".to_string());
        let result = saga.execute(ctx).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().completed_steps.len(), 3);
    }

    #[tokio::test]
    async fn test_saga_compensation() {
        let mut saga = SagaOrchestrator::new();

        saga.add_step(Box::new(MockStep::new("step1", Ok(()), Ok(()))));
        saga.add_step(Box::new(MockStep::new("step2", Ok(()), Ok(()))));
        saga.add_step(Box::new(MockStep::new(
            "step3",
            Err("Simulated failure".into()),
            Ok(()),
        )));

        let ctx = SagaContext::new("tenant-123".to_string(), "test-correlation".to_string());
        let result = saga.execute(ctx).await;

        assert!(result.is_err());
        // Verify compensation was triggered for step1 and step2
    }

    struct MockStep {
        name: String,
        execute_result: Result<(), Box<dyn Error>>,
        compensate_result: Result<(), Box<dyn Error>>,
    }

    impl MockStep {
        fn new(
            name: &str,
            execute_result: Result<(), Box<dyn Error>>,
            compensate_result: Result<(), Box<dyn Error>>,
        ) -> Self {
            Self {
                name: name.to_string(),
                execute_result,
                compensate_result,
            }
        }
    }

    #[async_trait]
    impl SagaStep for MockStep {
        fn name(&self) -> &str {
            &self.name
        }

        async fn execute(&self, _ctx: &mut SagaContext) -> Result<(), Box<dyn Error>> {
            self.execute_result.clone()
        }

        async fn compensate(&self, _ctx: &SagaContext) -> Result<(), Box<dyn Error>> {
            self.compensate_result.clone()
        }
    }
}
```

---

## Benefits

### 1. Distributed Transaction Management

✅ Maintain consistency across multiple databases/services
✅ No two-phase commit (2PC) required
✅ Eventual consistency with rollback capability

### 2. Resilience

✅ Recover from partial failures
✅ Compensate completed steps on failure
✅ Retry transient errors

### 3. Observability

✅ Track SAGA progress step-by-step
✅ Persist SAGA state for debugging
✅ Monitor compensation frequency

### 4. Flexibility

✅ Easy to add/remove steps
✅ Conditional step execution
✅ Reusable step implementations

---

## Trade-offs

### Cons

❌ **Complexity** - More code than simple transactions
❌ **Eventual Consistency** - Not immediately consistent
❌ **Compensation Logic** - Must implement rollback for each step
❌ **Testing Overhead** - Must test failure scenarios

### Mitigation

- **Start Simple** - Use SAGAs only for multi-service transactions
- **Thorough Testing** - Test all failure scenarios
- **Monitoring** - Track SAGA success/failure rates
- **Documentation** - Document compensation logic

---

## Related Patterns

- **Pattern #05: CQRS Pattern** - Commands initiate SAGAs
- **Pattern #12: Observer Pattern** - Event-driven choreography
- **Pattern #13: Circuit Breaker Pattern** - Protect SAGA steps from cascading failures
- **Pattern #45: Background Job Patterns** - Long-running SAGA steps as jobs
- **Pattern #49: Event Sourcing Pattern** - SAGA state as event stream
- **Pattern #94: Rust Anti-Corruption Layer Pattern** - External integrations in SAGA steps

---

## Summary

The **Rust SAGA Orchestration Pattern** manages distributed transactions across multiple services:

✅ **Sequential execution** - Steps run in order
✅ **Compensating transactions** - Rollback on failure
✅ **State persistence** - Survive crashes and resume
✅ **Idempotent steps** - Safe to retry
✅ **Observability** - Track progress and failures
✅ **Flexible** - Add/remove steps easily

**Key Takeaway**: Use SAGAs for multi-service transactions. For single-service, use local ACID transactions.

---

**Tags**: #rust #saga #distributed-transactions #orchestration #compensation #multi-tenant #eventual-consistency #workflow #sqlx
