# Conflict Resolution Pattern for Offline Field Data

**Category**: Architecture Pattern
**Complexity**: Advanced
**Status**: ✅ Production Ready
**Related Patterns**: Offline Batch Sync, Event Sourcing, CQRS
**Industry Context**: Oil & Gas Field Data Management

---

## Overview

The Conflict Resolution pattern provides strategies for resolving data conflicts when multiple field operators independently edit the same data offline, then sync their changes to the cloud. This pattern ensures:

- **Data integrity**: No data is lost due to conflicting changes
- **Safety-first bias**: For oil & gas, safety-critical data is never auto-resolved
- **Clear audit trail**: Every conflict resolution is logged
- **Operator trust**: Field workers understand how conflicts are handled

This pattern is critical for WellOS because:

1. **Multiple operators visit the same well** on the same day (shift changes, inspections)
2. **Internet connectivity is unreliable** (conflicts only detected during sync)
3. **Data accuracy matters** (incorrect production volumes = financial/regulatory issues)
4. **Safety is paramount** (conflicting equipment inspection results must be reviewed)

---

## The Problem

**Scenario**: Two field operators work the same well on the same day:

```
7:00 AM - Operator A arrives at Well #42 (offline)
7:15 AM - Operator A records production: 120 barrels

9:00 AM - Operator B arrives at Well #42 (offline)
9:15 AM - Operator B records production: 125 barrels (didn't know A already recorded it)

3:00 PM - Operator A returns to office, syncs data ✓
4:00 PM - Operator B returns to office, syncs data ⚠️ CONFLICT!
```

**The Conflict**:

- **Cloud database** (from Operator A): Well #42 produced 120 barrels today
- **Operator B's device**: Well #42 produced 125 barrels today
- **Question**: Which value is correct? Or are both valid?

**Bad Solutions**:

```typescript
// ❌ BAD: Last-write-wins (Operator B overwrites A)
await db.update(productionData).set({ volume: 125 }).where(eq(productionData.wellId, 'well-42'));
// Result: Operator A's data is lost!

// ❌ BAD: First-write-wins (Ignore Operator B)
if (existingData) {
  return; // Skip Operator B's data
}
// Result: Operator B's data is lost!

// ❌ BAD: Auto-merge without context
const avgVolume = (120 + 125) / 2; // 122.5 barrels
// Result: Neither value is correct!
```

---

## The Solution

### Conflict Resolution Strategies

Different types of conflicts require different resolution strategies:

| Conflict Type             | Strategy      | Rationale                                           |
| ------------------------- | ------------- | --------------------------------------------------- |
| **Sensor Readings**       | NEWEST_WINS   | Latest reading is most accurate                     |
| **Production Volumes**    | HIGHEST_VALUE | Never underreport production (financial/regulatory) |
| **Equipment Inspections** | MANUAL_REVIEW | Safety-critical, must be reviewed by supervisor     |
| **Notes/Comments**        | MERGE         | Append both, preserve all operator observations     |
| **Equipment Repairs**     | MANUAL_REVIEW | Safety-critical, need full context                  |
| **Photo Attachments**     | KEEP_BOTH     | Both photos may be valuable                         |

---

## Implementation

### 1. Conflict Detection (Server-Side)

```rust
// apps/api/src/application/field_data/conflict_detector.rs
use sqlx::PgPool;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub event_id: String,
    pub event_type: String,
    pub reason: String,
    pub local_data: serde_json::Value,
    pub server_data: serde_json::Value,
    pub recommended_resolution: ConflictResolutionStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConflictResolutionStrategy {
    NewestWins,
    HighestValue,
    ManualReview,
    Merge,
    KeepBoth,
}

pub struct ConflictDetectorService;
  /**
   * Detect if an incoming event conflicts with existing data.
   */
  async detectConflict(db: NodePgDatabase, event: any): Promise<Conflict | null> {
    switch (event.type) {
      case 'PRODUCTION_LOGGED':
        return this.detectProductionConflict(db, event);

      case 'EQUIPMENT_INSPECTED':
        return this.detectInspectionConflict(db, event);

      case 'WELL_READING_RECORDED':
        return this.detectReadingConflict(db, event);

      case 'NOTES_ADDED':
        return this.detectNotesConflict(db, event);

      default:
        return null; // No conflict detection for this event type
    }
  }

  private async detectProductionConflict(db: NodePgDatabase, event: any): Promise<Conflict | null> {
    const { wellId, recordedAt } = event.payload;

    // Check if production data already exists for this well + date
    const existing = await db
      .select()
      .from(productionDataTable)
      .where(
        and(
          eq(productionDataTable.wellId, wellId),
          eq(productionDataTable.recordedAt, new Date(recordedAt)),
        ),
      )
      .limit(1);

    if (existing.length === 0) {
      return null; // No conflict
    }

    // Conflict detected!
    return {
      eventId: event.id,
      eventType: event.type,
      reason: `Production data already exists for well ${wellId} on ${recordedAt}`,
      localData: event.payload,
      serverData: existing[0],
      recommendedResolution: ConflictResolutionStrategy.HIGHEST_VALUE,
    };
  }

  private async detectInspectionConflict(db: NodePgDatabase, event: any): Promise<Conflict | null> {
    const { wellId, equipmentId, inspectedAt } = event.payload;

    // Check if inspection already exists for same equipment on same day
    const dayStart = new Date(inspectedAt);
    dayStart.setHours(0, 0, 0, 0);

    const dayEnd = new Date(inspectedAt);
    dayEnd.setHours(23, 59, 59, 999);

    const existing = await db
      .select()
      .from(equipmentInspectionsTable)
      .where(
        and(
          eq(equipmentInspectionsTable.wellId, wellId),
          eq(equipmentInspectionsTable.equipmentId, equipmentId),
          gte(equipmentInspectionsTable.inspectedAt, dayStart),
          lte(equipmentInspectionsTable.inspectedAt, dayEnd),
        ),
      )
      .limit(1);

    if (existing.length === 0) {
      return null;
    }

    // Safety-critical: Require manual review
    return {
      eventId: event.id,
      eventType: event.type,
      reason: `Equipment inspection conflict: ${equipmentId} already inspected today`,
      localData: event.payload,
      serverData: existing[0],
      recommendedResolution: ConflictResolutionStrategy.MANUAL_REVIEW,
    };
  }

  private async detectReadingConflict(db: NodePgDatabase, event: any): Promise<Conflict | null> {
    // Sensor readings: NEWEST_WINS (latest reading is most accurate)
    // We still detect conflict but auto-resolve in favor of newest
    // ... implementation
  }

  private async detectNotesConflict(db: NodePgDatabase, event: any): Promise<Conflict | null> {
    // Notes: MERGE (append both operators' notes)
    // ... implementation
  }
}
```

---

### 2. Conflict Resolution Service

```rust
// apps/api/src/application/field_data/conflict_resolver.rs
use sqlx::PgPool;
use uuid::Uuid;
use super::conflict_detector::{Conflict, ConflictResolutionStrategy};
use chrono::Utc;

pub struct ConflictResolverService;

  /**
   * Resolve a detected conflict using the appropriate strategy.
   */
  async resolveConflict(
    db: NodePgDatabase,
    conflict: Conflict,
    tenantId: string,
  ): Promise<ConflictResolution> {
    this.logger.log(
      `Resolving conflict: ${conflict.eventType} (strategy: ${conflict.recommendedResolution})`,
    );

    switch (conflict.recommendedResolution) {
      case ConflictResolutionStrategy.NEWEST_WINS:
        return this.resolveNewestWins(db, conflict);

      case ConflictResolutionStrategy.HIGHEST_VALUE:
        return this.resolveHighestValue(db, conflict);

      case ConflictResolutionStrategy.MANUAL_REVIEW:
        return this.flagForManualReview(db, conflict, tenantId);

      case ConflictResolutionStrategy.MERGE:
        return this.resolveMerge(db, conflict);

      case ConflictResolutionStrategy.KEEP_BOTH:
        return this.resolveKeepBoth(db, conflict);

      default:
        // Default to manual review if strategy unknown
        return this.flagForManualReview(db, conflict, tenantId);
    }
  }

  /**
   * NEWEST_WINS: Latest timestamp wins.
   */
  private async resolveNewestWins(
    db: NodePgDatabase,
    conflict: Conflict,
  ): Promise<ConflictResolution> {
    const localTimestamp = new Date(conflict.localData.recordedAt || conflict.localData.timestamp);
    const serverTimestamp = new Date(
      conflict.serverData.recorded_at || conflict.serverData.timestamp,
    );

    if (localTimestamp > serverTimestamp) {
      // Local data is newer - update server
      this.logger.log(`Newest wins: Local data (${localTimestamp}) is newer`);

      return {
        action: 'UPDATE_SERVER',
        winner: conflict.localData,
        loser: conflict.serverData,
        reason: `Local data timestamp (${localTimestamp.toISOString()}) is newer than server (${serverTimestamp.toISOString()})`,
      };
    } else {
      // Server data is newer - keep server, ignore local
      this.logger.log(`Newest wins: Server data (${serverTimestamp}) is newer`);

      return {
        action: 'KEEP_SERVER',
        winner: conflict.serverData,
        loser: conflict.localData,
        reason: `Server data timestamp (${serverTimestamp.toISOString()}) is newer than local (${localTimestamp.toISOString()})`,
      };
    }
  }

  /**
   * HIGHEST_VALUE: Higher numeric value wins (for production volumes).
   */
  private async resolveHighestValue(
    db: NodePgDatabase,
    conflict: Conflict,
  ): Promise<ConflictResolution> {
    const localValue = parseFloat(conflict.localData.volume);
    const serverValue = parseFloat(conflict.serverData.volume);

    if (localValue > serverValue) {
      this.logger.log(`Highest value wins: Local (${localValue}) > Server (${serverValue})`);

      return {
        action: 'UPDATE_SERVER',
        winner: conflict.localData,
        loser: conflict.serverData,
        reason: `Local production volume (${localValue} barrels) is higher than server (${serverValue} barrels). Using highest to avoid underreporting.`,
      };
    } else {
      this.logger.log(`Highest value wins: Server (${serverValue}) >= Local (${localValue})`);

      return {
        action: 'KEEP_SERVER',
        winner: conflict.serverData,
        loser: conflict.localData,
        reason: `Server production volume (${serverValue} barrels) is higher than local (${localValue} barrels)`,
      };
    }
  }

  /**
   * MANUAL_REVIEW: Flag conflict for supervisor review (safety-critical data).
   */
  private async flagForManualReview(
    db: NodePgDatabase,
    conflict: Conflict,
    tenantId: string,
  ): Promise<ConflictResolution> {
    this.logger.warn(`Flagging conflict for manual review: ${conflict.reason}`);

    // Store conflict in database for dashboard review
    await db.insert(conflictsTable).values({
      id: uuidv4(),
      tenantId,
      eventId: conflict.eventId,
      eventType: conflict.eventType,
      reason: conflict.reason,
      localData: conflict.localData,
      serverData: conflict.serverData,
      status: 'PENDING_REVIEW',
      resolvedAt: null,
      resolvedBy: null,
      resolution: null,
      createdAt: new Date(),
    });

    return {
      action: 'MANUAL_REVIEW',
      winner: null,
      loser: null,
      reason: `Safety-critical conflict requires supervisor review: ${conflict.reason}`,
    };
  }

  /**
   * MERGE: Combine both values (for notes/comments).
   */
  private async resolveMerge(db: NodePgDatabase, conflict: Conflict): Promise<ConflictResolution> {
    const mergedNotes = `${conflict.serverData.notes}\n\n---\n\n${conflict.localData.notes}`;

    this.logger.log('Merging notes from both operators');

    return {
      action: 'MERGE',
      winner: {
        ...conflict.localData,
        notes: mergedNotes,
      },
      loser: null,
      reason: 'Merged notes from both operators',
    };
  }

  /**
   * KEEP_BOTH: Store both records (for photos, attachments).
   */
  private async resolveKeepBoth(
    db: NodePgDatabase,
    conflict: Conflict,
  ): Promise<ConflictResolution> {
    this.logger.log('Keeping both records');

    return {
      action: 'KEEP_BOTH',
      winner: conflict.localData,
      loser: conflict.serverData,
      reason: 'Both records are valuable - keeping both',
    };
  }
}

export interface ConflictResolution {
  action: 'UPDATE_SERVER' | 'KEEP_SERVER' | 'MANUAL_REVIEW' | 'MERGE' | 'KEEP_BOTH';
  winner: any;
  loser: any;
  reason: string;
}
```

---

### 3. Database Schema for Conflicts

```typescript
// apps/api/src/infrastructure/database/schema/tenant/conflicts.schema.ts
import { pgTable, varchar, text, timestamp, jsonb } from 'drizzle-orm/pg-core';

export const conflictsTable = pgTable('conflicts', {
  id: varchar('id', { length: 255 }).primaryKey(),
  tenantId: varchar('tenant_id', { length: 255 }).notNull(),
  eventId: varchar('event_id', { length: 255 }).notNull(),
  eventType: varchar('event_type', { length: 100 }).notNull(),

  // Conflict details
  reason: text('reason').notNull(),
  localData: jsonb('local_data').notNull(),
  serverData: jsonb('server_data').notNull(),

  // Resolution tracking
  status: varchar('status', { length: 50 }).notNull().default('PENDING_REVIEW'),
  // "PENDING_REVIEW" | "RESOLVED" | "IGNORED"
  resolvedAt: timestamp('resolved_at'),
  resolvedBy: varchar('resolved_by', { length: 255 }),
  resolution: jsonb('resolution'), // How it was resolved

  // Metadata
  createdAt: timestamp('created_at').notNull().defaultNow(),
  updatedAt: timestamp('updated_at').notNull().defaultNow(),
});
```

---

### 4. Sync Handler with Conflict Resolution

```typescript
// apps/api/src/application/field-data/commands/sync-field-data.handler.ts
@CommandHandler(SyncFieldDataCommand)
export class SyncFieldDataHandler implements ICommandHandler<SyncFieldDataCommand> {
  constructor(
    private readonly tenantDbService: TenantDatabaseService,
    private readonly conflictDetector: ConflictDetectorService,
    private readonly conflictResolver: ConflictResolverService,
  ) {}

  async execute(command: SyncFieldDataCommand): Promise<SyncResponseDto> {
    const syncedEventIds: string[] = [];
    const conflicts: Conflict[] = [];
    const errors: string[] = [];

    const db = await this.tenantDbService.getTenantDatabase(command.tenantId);

    for (const event of command.events) {
      try {
        // Step 1: Detect conflict
        const conflict = await this.conflictDetector.detectConflict(db, event);

        if (conflict) {
          // Step 2: Attempt auto-resolution
          const resolution = await this.conflictResolver.resolveConflict(
            db,
            conflict,
            command.tenantId,
          );

          if (resolution.action === 'MANUAL_REVIEW') {
            // Cannot auto-resolve - flag for review
            conflicts.push(conflict);
            continue;
          } else if (resolution.action === 'UPDATE_SERVER') {
            // Auto-resolved: Update server with local data
            await this.comlyEvent(db, event, command.userId);
            syncedEventIds.push(event.id);
          } else if (resolution.action === 'KEEP_SERVER') {
            // Auto-resolved: Keep server data, ignore local
            // Still mark as synced (don't retry)
            syncedEventIds.push(event.id);
          } else if (resolution.action === 'MERGE') {
            // Auto-resolved: Merge both
            await this.comlyMergedEvent(db, event, resolution.winner);
            syncedEventIds.push(event.id);
          } else if (resolution.action === 'KEEP_BOTH') {
            // Auto-resolved: Store both
            await this.comlyEvent(db, event, command.userId);
            syncedEventIds.push(event.id);
          }
        } else {
          // No conflict - apply event normally
          await this.comlyEvent(db, event, command.userId);
          syncedEventIds.push(event.id);
        }
      } catch (error) {
        errors.push(`Event ${event.id}: ${error.message}`);
      }
    }

    return {
      syncedEventIds,
      conflicts,
      errors,
    };
  }

  private async applyEvent(db: NodePgDatabase, event: any, userId: string): Promise<void> {
    // Apply event to database (insert/update based on event type)
  }

  private async applyMergedEvent(db: NodePgDatabase, event: any, mergedData: any): Promise<void> {
    // Apply merged data to database
  }
}
```

---

### 5. Manager Dashboard for Conflict Review

```typescript
// apps/web/app/(dashboard)/conflicts/page.tsx
import React from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { conflictRepository } from '@/lib/repositories/conflict.repository';

export default function ConflictsPage() {
  const { data: conflicts, isLoading } = useQuery({
    queryKey: ['conflicts', 'pending'],
    queryFn: () => conflictRepository.getPendingConflicts(),
  });

  const resolveConflictMutation = useMutation({
    mutationFn: ({ conflictId, resolution }: any) =>
      conflictRepository.resolveConflict(conflictId, resolution),
  });

  const handleResolve = (conflictId: string, chosenData: 'local' | 'server') => {
    resolveConflictMutation.mutate({
      conflictId,
      resolution: {
        action: chosenData === 'local' ? 'USE_LOCAL' : 'USE_SERVER',
        resolvedBy: 'current-user-id',
      },
    });
  };

  if (isLoading) return <div>Loading conflicts...</div>;

  return (
    <div className="p-6">
      <h1 className="text-2xl font-bold mb-6">Data Conflicts Requiring Review</h1>

      {conflicts.length === 0 ? (
        <p className="text-gray-600">No pending conflicts. All data synced successfully!</p>
      ) : (
        <div className="space-y-6">
          {conflicts.map((conflict) => (
            <div key={conflict.id} className="border border-yellow-300 bg-yellow-50 rounded p-4">
              <h3 className="font-semibold text-lg mb-2">
                {conflict.eventType.replace(/_/g, ' ')}
              </h3>
              <p className="text-sm text-gray-700 mb-4">{conflict.reason}</p>

              <div className="grid grid-cols-2 gap-4">
                {/* Local Data (Field Operator) */}
                <div className="bg-white border border-gray-300 rounded p-3">
                  <h4 className="font-semibold text-sm mb-2">Field Operator Data</h4>
                  <pre className="text-xs bg-gray-100 p-2 rounded overflow-auto">
                    {JSON.stringify(conflict.localData, null, 2)}
                  </pre>
                  <button
                    onClick={() => handleResolve(conflict.id, 'local')}
                    className="mt-3 bg-green-600 text-white px-4 py-2 rounded text-sm hover:bg-green-700"
                  >
                    Use This Data
                  </button>
                </div>

                {/* Server Data (Cloud) */}
                <div className="bg-white border border-gray-300 rounded p-3">
                  <h4 className="font-semibold text-sm mb-2">Cloud Data (Already Synced)</h4>
                  <pre className="text-xs bg-gray-100 p-2 rounded overflow-auto">
                    {JSON.stringify(conflict.serverData, null, 2)}
                  </pre>
                  <button
                    onClick={() => handleResolve(conflict.id, 'server')}
                    className="mt-3 bg-blue-600 text-white px-4 py-2 rounded text-sm hover:bg-blue-700"
                  >
                    Keep This Data
                  </button>
                </div>
              </div>

              <div className="mt-4 text-xs text-gray-600">
                <p>Created: {new Date(conflict.createdAt).toLocaleString()}</p>
                <p>Event ID: {conflict.eventId}</p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

---

## Resolution Strategy Matrix

| Data Type                | Example                              | Strategy      | Auto-Resolve? | Reason                                    |
| ------------------------ | ------------------------------------ | ------------- | ------------- | ----------------------------------------- |
| **Sensor Reading**       | Temperature: 180°F vs 185°F          | NEWEST_WINS   | ✅ Yes        | Latest reading is most accurate           |
| **Production Volume**    | 120 bbl vs 125 bbl                   | HIGHEST_VALUE | ✅ Yes        | Regulatory compliance (never underreport) |
| **Equipment Inspection** | PASS vs FAIL                         | MANUAL_REVIEW | ❌ No         | Safety-critical, needs supervisor         |
| **Equipment Repair**     | "Fixed pump" vs "Replaced pump"      | MANUAL_REVIEW | ❌ No         | Different actions, need full context      |
| **Notes/Comments**       | "Leak detected" vs "Vibration issue" | MERGE         | ✅ Yes        | Both observations valuable                |
| **Photo Attachment**     | photo_1.jpg vs photo_2.jpg           | KEEP_BOTH     | ✅ Yes        | Both photos may show different angles     |
| **Well Status**          | ACTIVE vs SHUT_IN                    | MANUAL_REVIEW | ❌ No         | Business-critical status change           |

---

## Benefits

### 1. **No Data Loss**

Every conflict is detected and resolved (or flagged):

```typescript
// ✅ With Conflict Resolution
if (conflict) {
  const resolution = await resolver.resolveConflict(conflict);
  // Either auto-resolved or flagged for review
}

// ❌ Without Conflict Resolution
// Data silently overwritten or ignored
```

### 2. **Safety-First Bias**

Safety-critical data always requires manual review:

```typescript
if (conflict.eventType === 'EQUIPMENT_INSPECTED') {
  return ConflictResolutionStrategy.MANUAL_REVIEW;
}
```

### 3. **Clear Audit Trail**

Every conflict resolution is logged:

```sql
SELECT * FROM conflicts WHERE status = 'RESOLVED';

-- Example output:
-- id | event_type | reason | resolved_at | resolved_by | resolution
-- 123 | EQUIPMENT_INSPECTED | Conflicting inspection results | 2025-10-23 15:30:00 | manager-456 | {"action": "USE_LOCAL"}
```

### 4. **Operator Trust**

Field operators know their data won't be silently discarded:

- Auto-resolved conflicts are explained ("Newest wins: your reading is more recent")
- Manual-review conflicts are visible in dashboard
- Supervisors make final call on safety-critical conflicts

---

## Testing Strategy

### Unit Tests: Conflict Detection

```typescript
describe('ConflictDetectorService', () => {
  let detector: ConflictDetectorService;
  let mockDb: any;

  it('should detect production conflict', async () => {
    mockDb = {
      select: jest.fn().mockReturnThis(),
      from: jest.fn().mockReturnThis(),
      where: jest.fn().mockReturnThis(),
      limit: jest.fn().mockResolvedValue([{ volume: 120 }]),
    };

    const event = {
      id: 'event-123',
      type: 'PRODUCTION_LOGGED',
      payload: { wellId: 'well-456', volume: 125, recordedAt: '2025-10-23' },
    };

    const conflict = await detector.detectConflict(mockDb, event);

    expect(conflict).not.toBeNull();
    expect(conflict.recommendedResolution).toBe('HIGHEST_VALUE');
    expect(conflict.localData.volume).toBe(125);
    expect(conflict.serverData.volume).toBe(120);
  });
});
```

### Integration Tests: Conflict Resolution

```typescript
describe('ConflictResolverService (E2E)', () => {
  it('should resolve production conflict with highest value', async () => {
    const conflict = {
      eventType: 'PRODUCTION_LOGGED',
      localData: { volume: 125 },
      serverData: { volume: 120 },
      recommendedResolution: 'HIGHEST_VALUE',
    };

    const resolution = await resolver.resolveConflict(db, conflict, 'tenant-123');

    expect(resolution.action).toBe('UPDATE_SERVER');
    expect(resolution.winner.volume).toBe(125);
    expect(resolution.reason).toContain('higher than server');
  });

  it('should flag equipment inspection for manual review', async () => {
    const conflict = {
      eventType: 'EQUIPMENT_INSPECTED',
      localData: { status: 'FAIL' },
      serverData: { status: 'PASS' },
      recommendedResolution: 'MANUAL_REVIEW',
    };

    const resolution = await resolver.resolveConflict(db, conflict, 'tenant-123');

    expect(resolution.action).toBe('MANUAL_REVIEW');

    // Verify conflict stored in database
    const storedConflict = await db.select().from(conflictsTable).limit(1);
    expect(storedConflict).toHaveLength(1);
    expect(storedConflict[0].status).toBe('PENDING_REVIEW');
  });
});
```

---

## Anti-Patterns

### ❌ **Don't: Auto-Resolve Safety-Critical Data**

```typescript
// ❌ BAD: Blindly auto-resolve equipment inspection conflicts
if (conflict.eventType === 'EQUIPMENT_INSPECTED') {
  return ConflictResolutionStrategy.NEWEST_WINS; // DANGEROUS!
}

// ✅ GOOD: Always require manual review
if (conflict.eventType === 'EQUIPMENT_INSPECTED') {
  return ConflictResolutionStrategy.MANUAL_REVIEW;
}
```

### ❌ **Don't: Use Last-Write-Wins for Production Data**

```typescript
// ❌ BAD: Last operator to sync overwrites previous data
await db.update(production).set(localData);

// ✅ GOOD: Use HIGHEST_VALUE strategy
const resolution = await resolver.resolveConflict(conflict);
if (resolution.action === 'UPDATE_SERVER') {
  await db.update(production).set(resolution.winner);
}
```

### ❌ **Don't: Silently Discard Conflicts**

```typescript
// ❌ BAD: Ignore conflicts, hope they don't happen
if (existingData) {
  return; // Silently skip
}

// ✅ GOOD: Detect, log, and flag for review
const conflict = await detector.detectConflict(db, event);
if (conflict) {
  await resolver.resolveConflict(db, conflict, tenantId);
}
```

---

## Related Patterns

- **Offline Batch Sync Pattern**: Detects conflicts during sync
- **Event Sourcing Pattern**: Provides audit trail of all changes
- **CQRS Pattern**: Separate conflict detection (query) from resolution (command)
- **Observer Pattern**: Notify supervisors when conflicts require review

---

## When to Use This Pattern

### ✅ Use Conflict Resolution When:

- **Multiple users edit same data offline** (field operators at same well)
- **Consistency matters** (production volumes, safety data)
- **Audit trail required** (regulatory compliance)
- **Safety-critical domain** (oil & gas, healthcare, aviation)

### ❌ Don't Use Conflict Resolution When:

- **Real-time collaboration** (use Operational Transformation or CRDTs instead)
- **Conflicts are rare** (simpler last-write-wins is sufficient)
- **Data is not critical** (losing occasional edits is acceptable)

---

## Summary

The **Conflict Resolution Pattern** provides:

1. **Multiple resolution strategies** (newest wins, highest value, manual review, merge, keep both)
2. **Safety-first bias** (critical data always requires manual review)
3. **Clear audit trail** (every conflict logged and tracked)
4. **Manager dashboard** (supervisors resolve flagged conflicts)
5. **Operator trust** (field workers know their data is protected)

**Critical Implementation Points**:

- Detect conflicts on server side during sync
- Choose resolution strategy based on data type
- Never auto-resolve safety-critical data
- Store unresolved conflicts in database for review
- Provide manager dashboard for conflict resolution
- Log all resolutions for audit trail

This pattern is essential for WellOS because oil & gas operations involve multiple operators working independently offline, and data accuracy/safety are paramount.

---

**Related Documentation**:

- [Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)
- [Offline Batch Sync Pattern](./70-Offline-Batch-Sync-Pattern.md)
- [Azure Production Architecture](../deployment/azure-production-architecture.md)
