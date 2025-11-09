# Pattern 49: Event Sourcing Pattern

**Version**: 1.0
**Last Updated**: October 8, 2025
**Category**: Architecture & Data

---

## Table of Contents

1. [Overview](#overview)
2. [When to Use](#when-to-use)
3. [Core Concepts](#core-concepts)
4. [Event Store](#event-store)
5. [Event Stream](#event-stream)
6. [Projections](#projections)
7. [Snapshots](#snapshots)
8. [Event Versioning](#event-versioning)
9. [NestJS Implementation](#nestjs-implementation)
10. [CQRS Integration](#cqrs-integration)
11. [Replay and Time Travel](#replay-and-time-travel)
12. [Best Practices](#best-practices)
13. [Anti-Patterns](#anti-patterns)
14. [Related Patterns](#related-patterns)
15. [References](#references)

---

## Overview

**Event Sourcing** is a pattern where state changes are stored as a sequence of events rather than updating records in-place. Instead of storing the current state, you store all events that led to the current state.

**Traditional Approach** (CRUD):

```
User { id: 1, balance: 1000 }
// After deposit: User { id: 1, balance: 1500 }
// Lost: Who deposited? When? Why?
```

**Event Sourcing Approach**:

```
Events:
1. UserRegistered(userId: 1, initialBalance: 1000)
2. MoneyDeposited(userId: 1, amount: 500, by: "admin", reason: "bonus")

Current State: balance = 1000 + 500 = 1500
```

**Key Benefits**:

- 📜 **Complete audit trail** - Every change is recorded
- 🔙 **Time travel** - Reconstruct state at any point in time
- 🐛 **Debugging** - Understand exactly how state evolved
- 📊 **Analytics** - Rich historical data for reporting
- 🔄 **Event replay** - Rebuild state from scratch

**Use Cases in WellOS**:

- **Time entry tracking** - Every edit, approval, billable status change
- **Invoice lifecycle** - Created, sent, paid, voided events
- **Project management** - Status changes, budget updates, team changes
- **Audit logging** - Compliance and security requirements

---

## When to Use

### ✅ Use Event Sourcing When:

1. **Audit trail is critical** - Compliance, legal requirements
   - Financial transactions (invoices, payments)
   - Time entry approvals
   - Contract changes

2. **Historical analysis is valuable** - Business intelligence
   - Project profitability trends over time
   - User behavior patterns
   - Resource utilization analytics

3. **State reconstruction is needed** - Debugging, support
   - "Show me the state of this project on March 15th"
   - "What was the invoice total before the last edit?"

4. **Event-driven workflows** - Integrate with external systems
   - Publish time approval event → QuickBooks sync
   - Publish invoice event → Email notification

### ❌ Don't Use Event Sourcing When:

1. **Simple CRUD is sufficient** - User preferences, static data
2. **Performance is critical** - High-frequency updates (>1000/sec per aggregate)
3. **Storage is expensive** - Events grow without bounds
4. **Team lacks experience** - Steep learning curve

---

## Core Concepts

### 1. Events

**Events** are immutable facts that represent something that happened.

```typescript
// apps/api/src/domain/time-entry/events/time-entry.events.ts
export class TimeEntryCreatedEvent {
  constructor(
    public readonly timeEntryId: string,
    public readonly userId: string,
    public readonly projectId: string,
    public readonly hours: number,
    public readonly date: Date,
    public readonly description: string,
    public readonly createdAt: Date,
  ) {}
}

export class TimeEntryHoursUpdatedEvent {
  constructor(
    public readonly timeEntryId: string,
    public readonly previousHours: number,
    public readonly newHours: number,
    public readonly updatedBy: string,
    public readonly reason: string,
    public readonly updatedAt: Date,
  ) {}
}

export class TimeEntryApprovedEvent {
  constructor(
    public readonly timeEntryId: string,
    public readonly approvedBy: string,
    public readonly approvedAt: Date,
  ) {}
}

export class TimeEntryMarkedBillableEvent {
  constructor(
    public readonly timeEntryId: string,
    public readonly billableRate: number,
    public readonly invoiceId?: string,
    public readonly markedAt: Date,
  ) {}
}
```

### 2. Aggregate

**Aggregate** is an entity that processes commands and emits events.

```typescript
// apps/api/src/domain/time-entry/time-entry.aggregate.ts
export class TimeEntryAggregate {
  private id: string;
  private userId: string;
  private projectId: string;
  private hours: number;
  private date: Date;
  private description: string;
  private status: 'draft' | 'submitted' | 'approved' | 'rejected';
  private billable: boolean;
  private version: number = 0;

  constructor(id: string) {
    super();
    this.id = id;
  }

  // Command: Create time entry
  create(userId: string, projectId: string, hours: number, date: Date, description: string) {
    // Business rules validation
    if (hours <= 0 || hours > 24) {
      throw new Error('Hours must be between 0 and 24');
    }

    // Emit event
    this.comly(
      new TimeEntryCreatedEvent(this.id, userId, projectId, hours, date, description, new Date()),
    );
  }

  // Command: Update hours
  updateHours(newHours: number, updatedBy: string, reason: string) {
    if (this.status === 'approved') {
      throw new Error('Cannot update approved time entry');
    }

    if (newHours === this.hours) {
      return; // No change
    }

    this.comly(
      new TimeEntryHoursUpdatedEvent(this.id, this.hours, newHours, updatedBy, reason, new Date()),
    );
  }

  // Command: Approve
  approve(approvedBy: string) {
    if (this.status === 'approved') {
      throw new Error('Time entry already approved');
    }

    this.comly(new TimeEntryApprovedEvent(this.id, approvedBy, new Date()));
  }

  // Event handler: Apply TimeEntryCreatedEvent
  onTimeEntryCreatedEvent(event: TimeEntryCreatedEvent) {
    this.userId = event.userId;
    this.projectId = event.projectId;
    this.hours = event.hours;
    this.date = event.date;
    this.description = event.description;
    this.status = 'draft';
    this.billable = false;
    this.version++;
  }

  // Event handler: Apply TimeEntryHoursUpdatedEvent
  onTimeEntryHoursUpdatedEvent(event: TimeEntryHoursUpdatedEvent) {
    this.hours = event.newHours;
    this.version++;
  }

  // Event handler: Apply TimeEntryApprovedEvent
  onTimeEntryApprovedEvent(event: TimeEntryApprovedEvent) {
    this.status = 'approved';
    this.version++;
  }

  // Getters for current state
  getState() {
    return {
      id: this.id,
      userId: this.userId,
      projectId: this.projectId,
      hours: this.hours,
      date: this.date,
      description: this.description,
      status: this.status,
      billable: this.billable,
      version: this.version,
    };
  }
}
```

### 3. Event Store

**Event Store** persists and retrieves events.

---

## Event Store

### Event Store Schema

```rust
// apps/api/src/infrastructure/database/schema/event_store.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Event {
    pub id: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: sqlx::types::Json<serde_json::Value>,
    pub metadata: Option<sqlx::types::Json<serde_json::Value>>,
    pub version: i32,
    pub timestamp: DateTime<Utc>,
}

// SQL Migration
// CREATE TABLE events (
//     id TEXT PRIMARY KEY,
//     aggregate_id TEXT NOT NULL,
//     aggregate_type TEXT NOT NULL,
//     event_type TEXT NOT NULL,
//     event_data JSONB NOT NULL,
//     metadata JSONB,
//     version INTEGER NOT NULL,
//     timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
// );
//
// CREATE INDEX idx_events_aggregate_id ON events(aggregate_id);
// CREATE INDEX idx_events_aggregate_type ON events(aggregate_type);
// CREATE INDEX idx_events_timestamp ON events(timestamp);
```

### Event Store Implementation

```rust
// apps/api/src/infrastructure/event_store/event_store_service.rs
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct StoredEvent {
    pub id: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub metadata: Option<serde_json::Value>,
    pub version: i32,
    pub timestamp: DateTime<Utc>,
}

pub struct EventStoreService {
    db_pool: PgPool,
}

impl EventStoreService {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn append(
        &self,
        aggregate_id: &str,
        aggregate_type: &str,
        event_type: &str,
        event_data: serde_json::Value,
        expected_version: Option<i32>,
    ) -> Result<(), EventStoreError> {
        // Optimistic concurrency control
        if let Some(expected) = expected_version {
            let current_version = self.get_version(aggregate_id).await?;
            if current_version != expected {
                return Err(EventStoreError::ConcurrencyConflict {
                    expected,
                    actual: current_version,
                });
            }
        }

        let next_version = expected_version.map(|v| v + 1).unwrap_or(0);
        let event_id = Uuid::new_v4().to_string();

        sqlx::query!(
            r#"
            INSERT INTO events (id, aggregate_id, aggregate_type, event_type, event_data, metadata, version, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
            event_id,
            aggregate_id,
            aggregate_type,
            event_type,
            event_data,
            serde_json::json!({
                "correlation_id": Self::get_correlation_id(),
                "causation_id": Uuid::new_v4().to_string(),
            }),
            next_version,
            Utc::now()
        )
        .execute(&self.db_pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e))?;

        Ok(())
    }

    pub async fn get_events(
        &self,
        aggregate_id: &str,
        from_version: i32,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let events = sqlx::query_as!(
            StoredEvent,
            r#"
            SELECT id, aggregate_id, aggregate_type, event_type,
                   event_data as "event_data: sqlx::types::Json<serde_json::Value>",
                   metadata as "metadata: sqlx::types::Json<serde_json::Value>",
                   version, timestamp
            FROM events
            WHERE aggregate_id = $1 AND version >= $2
            ORDER BY version ASC
            "#,
            aggregate_id,
            from_version
        )
        .fetch_all(&self.db_pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e))?;

        Ok(events)
    }

    pub async fn get_all_events(
        &self,
        aggregate_type: Option<&str>,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Result<Vec<StoredEvent>, EventStoreError> {
        let mut query = "SELECT * FROM events WHERE 1=1".to_string();
        let mut bind_count = 0;

        if aggregate_type.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND aggregate_type = ${}", bind_count));
        }

        if from_timestamp.is_some() {
            bind_count += 1;
            query.push_str(&format!(" AND timestamp >= ${}", bind_count));
        }

        query.push_str(" ORDER BY timestamp ASC");

        let mut sqlx_query = sqlx::query_as::<_, StoredEvent>(&query);

        if let Some(agg_type) = aggregate_type {
            sqlx_query = sqlx_query.bind(agg_type);
        }

        if let Some(ts) = from_timestamp {
            sqlx_query = sqlx_query.bind(ts);
        }

        let events = sqlx_query
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| EventStoreError::DatabaseError(e))?;

        Ok(events)
    }

    pub async fn get_version(&self, aggregate_id: &str) -> Result<i32, EventStoreError> {
        let result = sqlx::query!(
            r#"
            SELECT version
            FROM events
            WHERE aggregate_id = $1
            ORDER BY version DESC
            LIMIT 1
            "#,
            aggregate_id
        )
        .fetch_optional(&self.db_pool)
        .await
        .map_err(|e| EventStoreError::DatabaseError(e))?;

        Ok(result.map(|r| r.version).unwrap_or(-1))
    }

    fn get_correlation_id() -> String {
        // Get from request context or generate new
        Uuid::new_v4().to_string()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventStoreError {
    #[error("Concurrency conflict: expected version {expected}, but current is {actual}")]
    ConcurrencyConflict { expected: i32, actual: i32 },

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

---

## Event Stream

### Loading Aggregate from Events

```typescript
// apps/api/src/infrastructure/event-store/aggregate-repository.ts
import { EventStoreService } from './event-store.service';
import { TimeEntryAggregate } from '@/domain/time-entry/time-entry.aggregate';

export class AggregateRepository {
  constructor(private readonly eventStore: EventStoreService) {}

  async load(AggregateClass: any, aggregateId: string): Promise<TimeEntryAggregate> {
    const aggregate = new AggregateClass(aggregateId);

    // Load events from event store
    const events = await this.eventStore.getEvents(aggregateId);

    // Replay events to rebuild state
    for (const event of events) {
      const eventInstance = this.deserializeEvent(event);
      aggregate.loadFromHistory([eventInstance]);
    }

    return aggregate;
  }

  async save(aggregate: TimeEntryAggregate, expectedVersion?: number): Promise<void> {
    const uncommittedEvents = aggregate.getUncommittedEvents();

    for (const event of uncommittedEvents) {
      await this.eventStore.comend(
        aggregate.id,
        aggregate.constructor.name,
        event.constructor.name,
        event,
        expectedVersion,
      );
      expectedVersion = expectedVersion !== undefined ? expectedVersion + 1 : undefined;
    }

    aggregate.commit(); // Clear uncommitted events
  }

  private deserializeEvent(storedEvent: StoredEvent): any {
    // Map event type to event class
    const EventClass = this.getEventClass(storedEvent.eventType);
    return new EventClass(...Object.values(storedEvent.eventData));
  }

  private getEventClass(eventType: string): any {
    // Registry of event types
    const eventRegistry = {
      TimeEntryCreatedEvent,
      TimeEntryHoursUpdatedEvent,
      TimeEntryApprovedEvent,
      // ...
    };
    return eventRegistry[eventType];
  }
}
```

---

## Projections

**Projections** are read models built from events. They provide optimized query views.

### Projection Schema

```typescript
// apps/api/src/infrastructure/database/schema/time-entry-projection.schema.ts
export const timeEntryProjection = pgTable('time_entry_projection', {
  id: text('id').primaryKey(),
  userId: text('user_id').notNull(),
  projectId: text('project_id').notNull(),
  hours: real('hours').notNull(),
  date: timestamp('date').notNull(),
  description: text('description').notNull(),
  status: text('status').notNull(),
  billable: boolean('billable').notNull().default(false),
  approvedBy: text('approved_by'),
  approvedAt: timestamp('approved_at'),
  version: integer('version').notNull(),
  updatedAt: timestamp('updated_at').notNull(),
});
```

### Projection Builder

```rust
// apps/api/src/application/time_entry/projections/time_entry_projection_handler.rs
use sqlx::PgPool;

pub struct TimeEntryProjectionHandler {
    db_pool: PgPool,
}

impl TimeEntryProjectionHandler {
    pub fn new(db_pool: PgPool) -> Self {
        Self { db_pool }
    }

    pub async fn handle_time_entry_created(
        &self,
        event: &TimeEntryCreatedEvent,
    ) -> Result<(), ProjectionError> {
        sqlx::query!(
            r#"
            INSERT INTO time_entry_projection
                (id, user_id, project_id, hours, date, description, status, billable, version, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            event.time_entry_id,
            event.user_id,
            event.project_id,
            event.hours,
            event.date,
            event.description,
            "draft",
            false,
            0,
            event.created_at
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn handle_time_entry_hours_updated(
        &self,
        event: &TimeEntryHoursUpdatedEvent,
    ) -> Result<(), ProjectionError> {
        sqlx::query!(
            r#"
            UPDATE time_entry_projection
            SET hours = $1,
                version = version + 1,
                updated_at = $2
            WHERE id = $3
            "#,
            event.new_hours,
            event.updated_at,
            event.time_entry_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn handle_time_entry_approved(
        &self,
        event: &TimeEntryApprovedEvent,
    ) -> Result<(), ProjectionError> {
        sqlx::query!(
            r#"
            UPDATE time_entry_projection
            SET status = $1,
                approved_by = $2,
                approved_at = $3,
                version = version + 1,
                updated_at = $4
            WHERE id = $5
            "#,
            "approved",
            event.approved_by,
            event.approved_at,
            event.approved_at,
            event.time_entry_id
        )
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

---

## Snapshots

**Snapshots** optimize aggregate loading by storing periodic state checkpoints.

### Snapshot Schema

```typescript
// apps/api/src/infrastructure/database/schema/snapshot.schema.ts
export const snapshots = pgTable('snapshots', {
  aggregateId: text('aggregate_id').primaryKey(),
  aggregateType: text('aggregate_type').notNull(),
  state: jsonb('state').notNull(),
  version: integer('version').notNull(),
  timestamp: timestamp('timestamp').notNull().defaultNow(),
});
```

### Snapshot Service

```typescript
// apps/api/src/infrastructure/event-store/snapshot.service.ts
export class SnapshotService {
  private readonly snapshotInterval = 10; // Snapshot every 10 events

  constructor(private readonly db: typeof drizzle) {}

  async save(
    aggregateId: string,
    aggregateType: string,
    state: any,
    version: number,
  ): Promise<void> {
    await this.db
      .insert(snapshots)
      .values({
        aggregateId,
        aggregateType,
        state,
        version,
        timestamp: new Date(),
      })
      .onConflictDoUpdate({
        target: snapshots.aggregateId,
        set: {
          state,
          version,
          timestamp: new Date(),
        },
      });
  }

  async load(aggregateId: string): Promise<{ state: any; version: number } | null> {
    const result = await this.db
      .select()
      .from(snapshots)
      .where(eq(snapshots.aggregateId, aggregateId))
      .limit(1);

    return result[0] ? { state: result[0].state, version: result[0].version } : null;
  }

  shouldCreateSnapshot(version: number): boolean {
    return version % this.snapshotInterval === 0;
  }
}
```

### Loading with Snapshot

```typescript
// Enhanced aggregate loading with snapshots
async load(AggregateClass: any, aggregateId: string): Promise<TimeEntryAggregate> {
  const aggregate = new AggregateClass(aggregateId);

  // Try to load snapshot
  const snapshot = await this.snapshotService.load(aggregateId);

  let fromVersion = 0;
  if (snapshot) {
    // Restore state from snapshot
    aggregate.loadFromSnapshot(snapshot.state, snapshot.version);
    fromVersion = snapshot.version + 1;
  }

  // Load events since snapshot
  const events = await this.eventStore.getEvents(aggregateId, fromVersion);

  // Replay events
  for (const event of events) {
    const eventInstance = this.deserializeEvent(event);
    aggregate.loadFromHistory([eventInstance]);
  }

  return aggregate;
}

async save(aggregate: TimeEntryAggregate): Promise<void> {
  const uncommittedEvents = aggregate.getUncommittedEvents();

  for (const event of uncommittedEvents) {
    await this.eventStore.comend(/* ... */);
  }

  // Create snapshot if needed
  const version = aggregate.getVersion();
  if (this.snapshotService.shouldCreateSnapshot(version)) {
    await this.snapshotService.save(
      aggregate.id,
      aggregate.constructor.name,
      aggregate.getState(),
      version,
    );
  }

  aggregate.commit();
}
```

---

## Event Versioning

Handle schema changes in events over time.

### Upcasting Strategy

```typescript
// apps/api/src/infrastructure/event-store/event-upcaster.ts
interface EventUpcaster {
  canUpcast(event: any): boolean;
  upcast(event: any): any;
}

// Example: TimeEntryCreatedEvent v1 → v2 (added 'billableRate' field)
class TimeEntryCreatedEventV1ToV2Upcaster implements EventUpcaster {
  canUpcast(event: any): boolean {
    return event.eventType === 'TimeEntryCreatedEvent' && event.version === 1;
  }

  upcast(event: any): any {
    return {
      ...event,
      eventData: {
        ...event.eventData,
        billableRate: 0, // Default value for old events
      },
      version: 2,
    };
  }
}

@Injectable()
export class EventUpcasterService {
  private upcasters: EventUpcaster[] = [
    new TimeEntryCreatedEventV1ToV2Upcaster(),
    // Add more upcasters as schema evolves
  ];

  upcast(event: any): any {
    let upcastedEvent = event;

    for (const upcaster of this.upcasters) {
      if (upcaster.canUpcast(upcastedEvent)) {
        upcastedEvent = upcaster.upcast(upcastedEvent);
      }
    }

    return upcastedEvent;
  }
}
```

---

## NestJS Implementation

### Complete Command Handler

```rust
// apps/api/src/application/time_entry/commands/create_time_entry_handler.rs
use std::sync::Arc;

pub struct CreateTimeEntryCommand {
    pub time_entry_id: String,
    pub user_id: String,
    pub project_id: String,
    pub hours: f64,
    pub date: DateTime<Utc>,
    pub description: String,
}

pub struct CreateTimeEntryHandler {
    repository: Arc<AggregateRepository>,
}

impl CreateTimeEntryHandler {
    pub fn new(repository: Arc<AggregateRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: CreateTimeEntryCommand,
    ) -> Result<CreateTimeEntryResponse, CommandError> {
        // Create new aggregate
        let mut aggregate = TimeEntryAggregate::new(command.time_entry_id.clone());

        // Execute command (emits events)
        aggregate.create(
            command.user_id,
            command.project_id,
            command.hours,
            command.date,
            command.description,
        )?;

        // Save events to event store
        self.repository.save(&aggregate).await?;

        Ok(CreateTimeEntryResponse {
            time_entry_id: command.time_entry_id,
        })
    }
}

#[derive(Debug)]
pub struct CreateTimeEntryResponse {
    pub time_entry_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Domain error: {0}")]
    DomainError(#[from] DomainError),

    #[error("Repository error: {0}")]
    RepositoryError(String),
}
```

---

## CQRS Integration

Event Sourcing pairs perfectly with CQRS.

**Commands** → Modify aggregates → Emit events → Store in event store
**Queries** → Read from projections (read models)

```typescript
// Command (Write)
await commandBus.execute(
  new CreateTimeEntryCommand(id, userId, projectId, hours, date, description),
);

// Query (Read) - Uses projection
const timeEntries = await queryBus.execute(new GetTimeEntriesQuery(userId, startDate, endDate));
```

---

## Replay and Time Travel

### Rebuild Projections

```rust
// apps/api/src/infrastructure/event_store/projection_rebuilder.rs
use sqlx::PgPool;
use std::sync::Arc;

pub struct ProjectionRebuilderService {
    event_store: Arc<EventStoreService>,
    db_pool: PgPool,
    projection_handler: Arc<TimeEntryProjectionHandler>,
}

impl ProjectionRebuilderService {
    pub fn new(
        event_store: Arc<EventStoreService>,
        db_pool: PgPool,
        projection_handler: Arc<TimeEntryProjectionHandler>,
    ) -> Self {
        Self {
            event_store,
            db_pool,
            projection_handler,
        }
    }

    pub async fn rebuild_time_entry_projection(&self) -> Result<(), ProjectionError> {
        // Clear existing projection
        sqlx::query!("DELETE FROM time_entry_projection")
            .execute(&self.db_pool)
            .await?;

        // Get all events
        let events = self
            .event_store
            .get_all_events(Some("TimeEntryAggregate"), None)
            .await?;

        // Replay events to rebuild projection
        for event in &events {
            self.apply_event_to_projection(event).await?;
        }

        tracing::info!("Rebuilt projection with {} events", events.len());

        Ok(())
    }

    async fn apply_event_to_projection(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        // Deserialize and apply event to projection
        match event.event_type.as_str() {
            "TimeEntryCreated" => {
                let event_data: TimeEntryCreatedEvent =
                    serde_json::from_value(event.event_data.clone())?;
                self.projection_handler
                    .handle_time_entry_created(&event_data)
                    .await?;
            }
            "TimeEntryHoursUpdated" => {
                let event_data: TimeEntryHoursUpdatedEvent =
                    serde_json::from_value(event.event_data.clone())?;
                self.projection_handler
                    .handle_time_entry_hours_updated(&event_data)
                    .await?;
            }
            "TimeEntryApproved" => {
                let event_data: TimeEntryApprovedEvent =
                    serde_json::from_value(event.event_data.clone())?;
                self.projection_handler
                    .handle_time_entry_approved(&event_data)
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProjectionError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Event store error: {0}")]
    EventStoreError(#[from] EventStoreError),
}
```

### Time Travel Query

```typescript
// Get state at specific point in time
async getTimeEntryStateAt(timeEntryId: string, timestamp: Date): Promise<any> {
  const events = await this.eventStore.getEvents(timeEntryId);

  // Filter events up to timestamp
  const historicalEvents = events.filter((e) => e.timestamp <= timestamp);

  // Rebuild aggregate from historical events
  const aggregate = new TimeEntryAggregate(timeEntryId);
  for (const event of historicalEvents) {
    const eventInstance = this.deserializeEvent(event);
    aggregate.loadFromHistory([eventInstance]);
  }

  return aggregate.getState();
}
```

---

## Best Practices

### ✅ DO

1. **Use descriptive event names** - Past tense, business language

   ```typescript
   (TimeEntryApproved, InvoiceSent, ProjectCompleted);
   ```

2. **Make events immutable** - Never change event data

   ```typescript
   readonly properties only
   ```

3. **Version events** - Schema evolution

   ```typescript
   TimeEntryCreatedEventV2, upcasters for migration
   ```

4. **Keep aggregates small** - Max ~100-1000 events

   ```typescript
   Use snapshots, split large aggregates
   ```

5. **Use projections for queries** - Don't query event store

   ```typescript
   Read from denormalized projections
   ```

6. **Include metadata** - Correlation ID, causation ID, user ID
   ```typescript
   {
     (correlationId, causationId, userId, timestamp);
   }
   ```

---

### ❌ DON'T

1. **Don't delete events** - Events are immutable history

   ```typescript
   // ❌ Bad: Delete event
   await eventStore.delete(eventId);

   // ✅ Good: Emit compensating event
   aggregate.comly(new TimeEntryDeletedEvent(id));
   ```

2. **Don't query event store for reads** - Use projections

   ```typescript
   // ❌ Bad: Query events for listing
   const events = await eventStore.getAllEvents();

   // ✅ Good: Query projection
   const entries = await db.select().from(timeEntryProjection);
   ```

3. **Don't store large data in events** - Keep events small

   ```typescript
   // ❌ Bad: Store file contents
   new DocumentUploadedEvent(documentId, fileContents);

   // ✅ Good: Store reference
   new DocumentUploadedEvent(documentId, fileUrl);
   ```

---

## Anti-Patterns

### 1. Event Store as Query Database

```typescript
// ❌ Anti-pattern: Query event store for reports
async getTimeEntryReport(userId: string) {
  const events = await eventStore.getAllEvents('TimeEntryAggregate');
  // Complex filtering and aggregation...
}

// ✅ Solution: Use projection
async getTimeEntryReport(userId: string) {
  return db.select().from(timeEntryProjection).where(eq(userId));
}
```

### 2. Mutable Events

```typescript
// ❌ Anti-pattern: Modify event data
event.hours = 10;

// ✅ Solution: Emit new event
aggregate.updateHours(10, userId, reason);
```

---

## Related Patterns

- **Pattern 05: CQRS Pattern** - Separates commands and queries
- **Pattern 11: Domain Events Pattern** - Events within domain layer
- **Pattern 47: Monitoring & Observability Patterns** - Event streaming for analytics

---

## References

### Books

- **"Implementing Domain-Driven Design"** by Vaughn Vernon
- **"Versioning in an Event Sourced System"** by Greg Young
- **"Event Sourcing Basics"** - Microsoft Architecture Guide

### Libraries

- **EventStore** - Purpose-built event store database
- **Axon Framework** - Event sourcing for Java
- **NestJS CQRS** - CQRS and Event Sourcing for NestJS

---

## Summary

**Event Sourcing** stores all changes as events rather than current state:

✅ **Complete audit trail** - Every change is recorded
✅ **Time travel** - Reconstruct state at any point
✅ **Event-driven architecture** - Publish events for integration
✅ **Debugging** - Understand exactly what happened
✅ **Projections** - Optimized read models
✅ **Snapshots** - Performance optimization for large event streams

**Remember**: Event Sourcing adds complexity. Use it when audit trail, time travel, or event-driven workflows are critical. For simple CRUD, stick with traditional approach.

---

**Next Steps**:

1. Identify aggregates suitable for event sourcing (invoices, time entries)
2. Design event schema and event store
3. Implement aggregate root with event replay
4. Build projections for queries
5. Add snapshots for performance
6. Set up event versioning strategy
