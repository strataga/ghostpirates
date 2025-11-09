# Pattern 63: Undefined vs Null in Database Operations

## Problem

In Rust with SQLx, nullable database columns are represented using `Option<T>`. The key difference from TypeScript is that Rust doesn't have "undefined" - only `Some(value)` and `None`. This makes handling nullable fields more explicit and type-safe.

However, when working with external APIs or JSON deserialization, you need to handle missing fields correctly:

```
❌ Runtime errors from improperly handled Option types
❌ Serialization errors when None values aren't handled
```

This commonly occurs with:

- Optional parameters in domain entity factory methods
- Partial updates where some fields should remain unchanged
- Audit logging with optional metadata fields
- JSON API payloads with missing fields

## Anti-Pattern: Relying on Optional Parameter Defaults

❌ **Don't do this:**

```typescript
// Domain entity factory method
class AuditLog {
  static create(
    id: string,
    action: string,
    userId?: string | null,
    organizationId?: string | null,
    entityType?: string | null,
    entityId?: string | null,
    changes?: Record<string, any> | null,
    oldData?: Record<string, any> | null,
    newData?: Record<string, any> | null,
    metadata?: Record<string, any> | null,
    ipAddress?: string | null, // ⚠️ Optional
    userAgent?: string | null, // ⚠️ Optional
  ): AuditLog {
    return new AuditLog({
      id,
      userId,
      organizationId,
      action,
      entityType,
      entityId,
      changes,
      oldData,
      newData,
      metadata,
      ipAddress, // undefined if not passed! 💥
      userAgent, // undefined if not passed! 💥
      createdAt: new Date(),
    });
  }
}

// Usage - missing last two parameters
const auditLog = AuditLog.create(
  randomUUID(),
  'USER_IMPERSONATION_STARTED',
  superAdminId,
  null,
  'user',
  targetUserId,
  null,
  null,
  null,
  { targetUserId, superAdminId },
  // ❌ ipAddress and userAgent are undefined!
);

await repository.save(auditLog); // 💥 UNDEFINED_VALUE error
```

**Why it fails:**

- TypeScript optional parameters (`param?: Type`) default to `undefined` when omitted
- PostgreSQL rejects `undefined` values, requiring explicit `NULL`
- Drizzle ORM doesn't auto-convert `undefined` → `null`

## Solution 1: Explicit Null Parameters

✅ **Do this:**

```typescript
// Always explicitly pass null for nullable database columns
const auditLog = AuditLog.create(
  randomUUID(),
  'USER_IMPERSONATION_STARTED',
  superAdminId,
  null,
  'user',
  targetUserId,
  null,
  null,
  null,
  { targetUserId, superAdminId },
  null, // ✅ Explicit null for ipAddress
  null, // ✅ Explicit null for userAgent
);

await repository.save(auditLog); // ✅ Works!
```

## Solution 2: Builder Pattern in Rust

For domain entities with many optional parameters, use the builder pattern with `Option<T>`:

```rust
#[derive(Debug, Clone)]
pub struct AuditLogBuilder {
    id: String,
    action: String,
    user_id: Option<String>,
    organization_id: Option<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    changes: Option<serde_json::Value>,
    old_data: Option<serde_json::Value>,
    new_data: Option<serde_json::Value>,
    metadata: Option<serde_json::Value>,
    ip_address: Option<String>,
    user_agent: Option<String>,
}

impl AuditLogBuilder {
    pub fn new(id: String, action: String) -> Self {
        Self {
            id,
            action,
            user_id: None,
            organization_id: None,
            entity_type: None,
            entity_id: None,
            changes: None,
            old_data: None,
            new_data: None,
            metadata: None,
            ip_address: None,
            user_agent: None,
        }
    }

    // ✅ Explicit Option handling with builder methods
    pub fn user_id(mut self, user_id: Option<String>) -> Self {
        self.user_id = user_id;
        self
    }

    pub fn metadata(mut self, metadata: Option<serde_json::Value>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn ip_address(mut self, ip_address: Option<String>) -> Self {
        self.ip_address = ip_address;
        self
    }

    pub fn user_agent(mut self, user_agent: Option<String>) -> Self {
        self.user_agent = user_agent;
        self
    }

    pub fn build(self) -> AuditLog {
        AuditLog {
            id: self.id,
            action: self.action,
            user_id: self.user_id,
            organization_id: self.organization_id,
            entity_type: self.entity_type,
            entity_id: self.entity_id,
            changes: self.changes,
            old_data: self.old_data,
            new_data: self.new_data,
            metadata: self.metadata,
            ip_address: self.ip_address, // ✅ None by default
            user_agent: self.user_agent,  // ✅ None by default
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

// Usage - cleaner and type-safe
let audit_log = AuditLogBuilder::new(
    Uuid::new_v4().to_string(),
    "USER_IMPERSONATION_STARTED".to_string(),
)
.user_id(Some(super_admin_id))
.entity_type(Some("user".to_string()))
.entity_id(Some(target_user_id))
.metadata(Some(serde_json::json!({
    "targetUserId": target_user_id,
    "superAdminId": super_admin_id,
})))
// ip_address and user_agent remain None
.build();
```

## Solution 3: Repository-Level Option Handling (Rust + SQLx)

Use SQLx's native `Option<T>` support with proper bindings:

```rust
pub struct AuditLogRepository;

impl AuditLogRepository {
    pub async fn save(
        &self,
        pool: &sqlx::PgPool,
        audit_log: &AuditLog,
    ) -> Result<(), sqlx::Error> {
        // ✅ SQLx automatically handles Option<T> → NULL
        sqlx::query!(
            r#"
            INSERT INTO audit_logs (
                id, user_id, organization_id, action,
                entity_type, entity_id, changes, old_data, new_data,
                metadata, ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            "#,
            audit_log.id,
            audit_log.user_id,           // Option<String> → NULL if None
            audit_log.organization_id,   // Option<String> → NULL if None
            audit_log.action,
            audit_log.entity_type,       // Option<String> → NULL if None
            audit_log.entity_id,         // Option<String> → NULL if None
            audit_log.changes,           // Option<Value> → NULL if None
            audit_log.old_data,          // Option<Value> → NULL if None
            audit_log.new_data,          // Option<Value> → NULL if None
            audit_log.metadata,          // Option<Value> → NULL if None
            audit_log.ip_address,        // ✅ Automatically NULL if None
            audit_log.user_agent,        // ✅ Automatically NULL if None
            audit_log.created_at,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
```

## Key Principles

### 1. **Understand Rust vs SQL Semantics**

Rust:

- `Some(value)` = "value is present"
- `None` = "value is absent"
- No concept of `undefined`

SQL/PostgreSQL:

- `NULL` = "no value" (only option)
- Maps directly to Rust's `None`

### 2. **Option Unwrapping Patterns**

Use proper Option handling:

```rust
// ✅ Use unwrap_or for defaults
let value = option_value.unwrap_or_default();

// ✅ Use unwrap_or_else for computed defaults
let value = option_value.unwrap_or_else(|| expensive_computation());

// ✅ Use match for explicit handling
let value = match option_value {
    Some(v) => v,
    None => default_value,
};

// ❌ Don't use unwrap() - will panic on None
let value = option_value.unwrap();
```

### 3. **Factory Method Best Practices**

When designing domain entity factories:

✅ **Options object for >5 parameters:**

```typescript
static create(options: EntityOptions): Entity
```

✅ **Required parameters first, optional last:**

```typescript
static create(id: string, name: string, metadata?: Record<string, any>)
```

✅ **Explicit null normalization:**

```typescript
ipAddress: options.ipAddress ?? null;
```

### 4. **Type System Alignment**

Make TypeScript types match database schema:

```typescript
// ✅ Aligned with database
interface AuditLogProps {
  ipAddress: string | null; // Not undefined!
  userAgent: string | null; // Not undefined!
}

// ❌ Misaligned
interface AuditLogProps {
  ipAddress?: string; // Could be undefined
  userAgent?: string; // Could be undefined
}
```

## Complete Implementation

### Domain Entity with Options Pattern

```typescript
export interface AuditLogProps {
  id: string;
  userId: string | null;
  organizationId: string | null;
  action: string;
  entityType: string | null;
  entityId: string | null;
  changes: Record<string, any> | null;
  oldData: Record<string, any> | null;
  newData: Record<string, any> | null;
  metadata: Record<string, any> | null;
  ipAddress: string | null;
  userAgent: string | null;
  createdAt: Date;
}

export class AuditLog {
  private props: AuditLogProps;

  private constructor(props: AuditLogProps) {
    this.props = props;
  }

  static create(options: {
    id: string;
    action: string;
    userId?: string | null;
    organizationId?: string | null;
    entityType?: string | null;
    entityId?: string | null;
    changes?: Record<string, any> | null;
    oldData?: Record<string, any> | null;
    newData?: Record<string, any> | null;
    metadata?: Record<string, any> | null;
    ipAddress?: string | null;
    userAgent?: string | null;
  }): AuditLog {
    if (!options.action || options.action.trim().length === 0) {
      throw new Error('Action cannot be empty');
    }

    return new AuditLog({
      id: options.id,
      userId: options.userId ?? null,
      organizationId: options.organizationId ?? null,
      action: options.action.trim(),
      entityType: options.entityType ?? null,
      entityId: options.entityId ?? null,
      changes: options.changes ?? null,
      oldData: options.oldData ?? null,
      newData: options.newData ?? null,
      metadata: options.metadata ?? null,
      ipAddress: options.ipAddress ?? null,
      userAgent: options.userAgent ?? null,
      createdAt: new Date(),
    });
  }

  // Getters
  get ipAddress(): string | null {
    return this.props.ipAddress;
  }
  get userAgent(): string | null {
    return this.props.userAgent;
  }
  // ... other getters
}
```

### Repository with Null Coercion

```typescript
export class DrizzleAuditLogRepository implements IAuditLogRepository {
  async save(auditLog: AuditLog): Promise<void> {
    const values = {
      id: auditLog.id,
      userId: auditLog.userId ?? null,
      organizationId: auditLog.organizationId ?? null,
      action: auditLog.action,
      entityType: auditLog.entityType ?? null,
      entityId: auditLog.entityId ?? null,
      changes: auditLog.changes ?? null,
      oldData: auditLog.oldData ?? null,
      newData: auditLog.newData ?? null,
      metadata: auditLog.metadata ?? null,
      ipAddress: auditLog.ipAddress ?? null,
      userAgent: auditLog.userAgent ?? null,
      createdAt: auditLog.createdAt,
    };

    await this.db.insert(schema.auditLogsTable).values(values);
  }
}
```

### Database Schema

```sql
CREATE TABLE audit_logs (
    id TEXT PRIMARY KEY,
    user_id TEXT,
    organization_id TEXT,
    action TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    changes JSONB,
    old_data JSONB,
    new_data JSONB,
    metadata JSONB,
    ip_address TEXT,  -- Nullable
    user_agent TEXT,  -- Nullable
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);
```

```rust
use sqlx::FromRow;
use serde_json::Value;

#[derive(Debug, FromRow)]
pub struct AuditLogRow {
    pub id: String,
    pub user_id: Option<String>,
    pub organization_id: Option<String>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub changes: Option<Value>,
    pub old_data: Option<Value>,
    pub new_data: Option<Value>,
    pub metadata: Option<Value>,
    pub ip_address: Option<String>,  // ✅ Option for nullable
    pub user_agent: Option<String>,  // ✅ Option for nullable
    pub created_at: chrono::NaiveDateTime,
}
```

## Testing

### Unit Test for Null Handling

```typescript
describe('AuditLog.create', () => {
  it('should convert undefined to null for optional fields', () => {
    const auditLog = AuditLog.create({
      id: 'test-id',
      action: 'TEST_ACTION',
      userId: 'user-123',
      // ipAddress and userAgent not provided
    });

    expect(auditLog.ipAddress).toBeNull(); // ✅ null, not undefined
    expect(auditLog.userAgent).toBeNull(); // ✅ null, not undefined
  });

  it('should accept explicit null values', () => {
    const auditLog = AuditLog.create({
      id: 'test-id',
      action: 'TEST_ACTION',
      userId: 'user-123',
      ipAddress: null,
      userAgent: null,
    });

    expect(auditLog.ipAddress).toBeNull();
    expect(auditLog.userAgent).toBeNull();
  });

  it('should accept explicit values', () => {
    const auditLog = AuditLog.create({
      id: 'test-id',
      action: 'TEST_ACTION',
      userId: 'user-123',
      ipAddress: '192.168.1.1',
      userAgent: 'Mozilla/5.0',
    });

    expect(auditLog.ipAddress).toBe('192.168.1.1');
    expect(auditLog.userAgent).toBe('Mozilla/5.0');
  });
});
```

### Integration Test for Database Insert

```typescript
describe('DrizzleAuditLogRepository', () => {
  it('should save audit log with null values', async () => {
    const repository = new DrizzleAuditLogRepository(db);

    const auditLog = AuditLog.create({
      id: randomUUID(),
      action: 'TEST_ACTION',
      userId: 'user-123',
      // ipAddress and userAgent not provided
    });

    // Should not throw UNDEFINED_VALUE error
    await expect(repository.save(auditLog)).resolves.not.toThrow();

    // Verify database record
    const [saved] = await db
      .select()
      .from(schema.auditLogsTable)
      .where(eq(schema.auditLogsTable.id, auditLog.id));

    expect(saved.ipAddress).toBeNull();
    expect(saved.userAgent).toBeNull();
  });
});
```

## When to Use

✅ **Always use explicit null handling when:**

- Working with Drizzle ORM or any database layer
- Dealing with nullable database columns
- Domain entities have optional fields that map to nullable columns
- Receiving data from external APIs that may omit fields

❌ **May skip for:**

- Pure TypeScript logic with no database interaction
- Internal data structures that never persist
- Temporary variables

## Common Pitfalls

### 1. **Forgetting Optional Parameters**

```typescript
// ❌ Easy to miss last parameters
AuditLog.create(id, action, userId, null, 'user', entityId, null, null, null, metadata);
// Missing ipAddress and userAgent!

// ✅ Options object makes omissions visible
AuditLog.create({
  id,
  action,
  userId,
  entityType: 'user',
  entityId,
  metadata,
});
```

### 2. **Using || Instead of ??**

```typescript
const value = 0;
value || null; // ❌ Returns null (0 is falsy!)
value ?? null; // ✅ Returns 0
```

### 3. **Partial Type Mismatch**

```typescript
// ❌ Partial makes all fields optional (undefined possible)
type UpdateAuditLog = Partial<AuditLogProps>;

// ✅ Use Pick for specific nullable fields
type UpdateAuditLog = Pick<AuditLogProps, 'metadata'> & {
  ipAddress?: string | null;
  userAgent?: string | null;
};
```

## Related Patterns

- **Pattern 2:** Value Objects - Often need null handling
- **Pattern 6:** Repository Pattern - Where null coercion happens
- **Pattern 7:** DTO Pattern - Null handling during data transfer
- **Pattern 41:** Database Constraint Race Condition - Related database reliability pattern

## Real-World Example: User Impersonation Audit

**Scenario:** Super admin impersonates a user for troubleshooting

**Before fix:**

```typescript
const auditLog = AuditLog.create(
  randomUUID(),
  'USER_IMPERSONATION_STARTED',
  superAdminId,
  null,
  'user',
  targetUserId,
  null,
  null,
  null,
  { targetUserId, superAdminId },
);

await repository.save(auditLog);
// 💥 UNDEFINED_VALUE: Undefined values are not allowed
```

**After fix:**

```typescript
const auditLog = AuditLog.create(
  randomUUID(),
  'USER_IMPERSONATION_STARTED',
  superAdminId,
  null,
  'user',
  targetUserId,
  null,
  null,
  null,
  { targetUserId, superAdminId },
  null, // ipAddress explicitly null
  null, // userAgent explicitly null
);

await repository.save(auditLog);
// ✅ Success! Audit log saved
```

## Performance Considerations

**Null coercion overhead:** Negligible (simple ?? operator)
**Memory impact:** None (`null` and `undefined` are both primitive values)
**Database impact:** None (NULL is standard SQL)

## References

- PostgreSQL NULL Handling: https://www.postgresql.org/docs/current/functions-comparison.html
- Rust Option Type: https://doc.rust-lang.org/std/option/
- SQLx Type Mappings: https://docs.rs/sqlx/latest/sqlx/types/index.html
- Serde JSON: https://docs.rs/serde_json/latest/serde_json/
