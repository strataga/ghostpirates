# Pattern 63: PostgreSQL JSONB Undefined Value Handling

**Category**: Infrastructure / Data Access
**Status**: Production-Ready
**Created**: October 2025

---

## Problem

PostgreSQL JSONB columns reject JavaScript's `undefined` values, throwing `UNDEFINED_VALUE: Undefined values are not allowed` errors. This is especially problematic when:

1. **Domain entities** have optional fields that may be `undefined`
2. **Metadata objects** contain nested properties with undefined values
3. **API responses** are converted to JSONB for audit logging
4. **Value objects** return `undefined` for optional properties

The issue is subtle because:

- Top-level undefined values can be caught with `?? null`
- **Nested undefined values** in objects pass through undetected
- TypeScript allows `undefined` but PostgreSQL rejects it

## Solution

### 1. JSONB Cleaner Utility

Create a utility that recursively removes all undefined values from objects:

```typescript
// apps/api/src/infrastructure/database/repositories/jsonb-cleaner.util.ts

/**
 * Recursively removes undefined values from objects (for JSONB fields)
 * Converts undefined to null or removes the key entirely
 */
export function cleanJsonbObject(obj: any): any {
  // Null or undefined at top level → null
  if (obj === null || obj === undefined) {
    return null;
  }

  // Arrays: recursively clean each item
  if (Array.isArray(obj)) {
    return obj.map((item) => cleanJsonbObject(item));
  }

  // Objects: recursively clean each property, skip undefined values
  if (typeof obj === 'object') {
    const cleaned: Record<string, any> = {};
    for (const key in obj) {
      const value = obj[key];
      if (value === undefined) {
        // Skip undefined values entirely (remove key from object)
        continue;
      }
      cleaned[key] = cleanJsonbObject(value);
    }
    return cleaned;
  }

  // Primitives: return as-is
  return obj;
}
```

### 2. Repository `toPersistence` Pattern

Apply the cleaner to ALL JSONB fields in the repository's `toPersistence` method:

```typescript
// ✅ CORRECT: Clean all JSONB fields
private toPersistence(auditLog: AuditLog): any {
  return {
    id: auditLog.id,
    userId: auditLog.userId ?? null,          // Regular field: ?? null
    action: auditLog.action,
    changes: cleanJsonbObject(auditLog.changes),      // JSONB: clean
    oldData: cleanJsonbObject(auditLog.oldData),      // JSONB: clean
    newData: cleanJsonbObject(auditLog.newData),      // JSONB: clean
    metadata: cleanJsonbObject(auditLog.metadata),    // JSONB: clean
    createdAt: auditLog.createdAt,
  };
}
```

```typescript
// ❌ WRONG: Using ?? null on JSONB fields
metadata: auditLog.metadata ?? null,  // Only checks top-level, not nested properties
```

### 3. Test Coverage

```typescript
describe('cleanJsonbObject', () => {
  it('should remove undefined properties from nested objects', () => {
    const input = {
      user: {
        name: 'John',
        email: undefined,
        address: {
          city: 'NYC',
          zip: undefined,
        },
      },
    };

    const result = cleanJsonbObject(input);

    expect(result).toEqual({
      user: {
        name: 'John',
        address: {
          city: 'NYC',
        },
      },
    });
  });

  it('should keep null values but remove undefined', () => {
    const input = { name: 'John', age: null, city: undefined };
    const result = cleanJsonbObject(input);
    expect(result).toEqual({ name: 'John', age: null });
  });
});
```

---

## When to Use This Pattern

### ✅ Use When:

1. **Any repository** saving data to JSONB columns
2. **Audit logging** with dynamic metadata objects
3. **Event sourcing** storing event payloads
4. **Analytics data** with flexible schemas
5. **User preferences** stored as JSONB

### ❌ Don't Use When:

- Working with regular (non-JSONB) columns
- Data is guaranteed to never have undefined values
- Using MongoDB or other NoSQL (they handle undefined differently)

---

## Implementation Checklist

When adding a new repository with JSONB fields:

- [ ] Import `cleanJsonbObject` from `jsonb-cleaner.util.ts`
- [ ] Apply to ALL JSONB fields in `toPersistence` method
- [ ] Write unit tests with nested undefined values
- [ ] Test with real database inserts/updates
- [ ] Document which fields are JSONB in schema comments

---

## Example: Audit Log Repository

```typescript
import { cleanJsonbObject } from './jsonb-cleaner.util';

@Injectable()
export class DrizzleAuditLogRepository implements IAuditLogRepository {
  private toPersistence(auditLog: AuditLog): any {
    return {
      id: auditLog.id,
      userId: auditLog.userId ?? null,
      organizationId: auditLog.organizationId ?? null,
      action: auditLog.action,
      entityType: auditLog.entityType ?? null,
      entityId: auditLog.entityId ?? null,
      // JSONB fields: use cleanJsonbObject
      changes: cleanJsonbObject(auditLog.changes),
      oldData: cleanJsonbObject(auditLog.oldData),
      newData: cleanJsonbObject(auditLog.newData),
      metadata: cleanJsonbObject(auditLog.metadata),
      ipAddress: auditLog.ipAddress ?? null,
      userAgent: auditLog.userAgent ?? null,
      createdAt: auditLog.createdAt,
    };
  }
}
```

---

## Example: Organization Health Repository

```typescript
import { cleanJsonbObject } from './jsonb-cleaner.util';

async trackMilestone(
  organizationId: string,
  milestone: ImplementationMilestone,
): Promise<void> {
  const updatedMilestones = {
    ...health.milestonesAchieved,
    [milestone]: new Date().toISOString(),
  };

  await this.db
    .update(schema.organizationHealth)
    .set({
      milestonesAchieved: cleanJsonbObject(updatedMilestones),  // ✅ Clean before save
      lastMilestoneAt: new Date(),
      updatedAt: new Date(),
    })
    .where(eq(schema.organizationHealth.organizationId, organizationId));
}
```

---

## Common Mistakes

### 1. Only Handling Top-Level Undefined

```typescript
// ❌ WRONG: Doesn't handle nested undefined
metadata: auditLog.metadata ?? null,
```

```typescript
// ✅ CORRECT: Recursively cleans nested undefined
metadata: cleanJsonbObject(auditLog.metadata),
```

### 2. Forgetting JSONB Fields

```typescript
// ❌ WRONG: Missed cleaning JSONB field
return {
  ...
  metadata: auditLog.metadata,  // Could have nested undefined
};
```

### 3. Not Testing Nested Cases

```typescript
// ❌ WRONG: Only tests top-level
it('should handle undefined', () => {
  const result = cleanJsonbObject({ foo: undefined });
  expect(result).toEqual({});
});

// ✅ CORRECT: Tests nested undefined
it('should handle nested undefined', () => {
  const result = cleanJsonbObject({
    user: { name: 'John', email: undefined },
  });
  expect(result).toEqual({ user: { name: 'John' } });
});
```

---

## Performance Considerations

1. **Recursive traversal**: O(n) where n = total number of properties (including nested)
2. **Memory overhead**: Creates new objects instead of mutating
3. **Acceptable for**: Audit logs, metadata, events (usually < 1000 properties)
4. **Not ideal for**: Extremely large documents with 100k+ properties

**Optimization tip**: If performance becomes an issue, consider:

- Cleaning data at the source (command handlers)
- Using schemas to validate "no undefined" before persistence
- Streaming large objects instead of loading entirely

---

## Related Patterns

- **Pattern 61: Value Object Layer Boundary Pattern** - Extracting primitives from value objects
- **Pattern 16: Pattern Integration Guide** - Combining multiple patterns
- **Repository Pattern** - Data access layer abstraction
- **DTO Pattern** - Data transfer objects

---

## References

- PostgreSQL JSONB Documentation: https://www.postgresql.org/docs/current/datatype-json.html
- SQLx Type Mappings: https://docs.rs/sqlx/latest/sqlx/types/index.html
- Rust Option Type: https://doc.rust-lang.org/std/option/
- JavaScript `undefined` vs `null`: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/undefined

---

## Changelog

- **2025-10-18**: Pattern created after fixing admin impersonation undefined value error
- Fixed audit log repository to clean all JSONB fields
- Created reusable `cleanJsonbObject` utility with 19 unit tests
- Applied to organization health repository for consistency

---

## Status: Production-Ready

This pattern is actively used in:

- ✅ Audit log repository (changes, oldData, newData, metadata)
- ✅ Organization health repository (milestonesAchieved)
- ✅ All repositories with JSONB columns should follow this pattern
