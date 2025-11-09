# Pattern 60: Pagination Index Convention Pattern

**Category**: API Design
**Phase**: All Phases
**Related Patterns**: #16 (Repository Pattern), #12 (DTO Pattern)

## Problem

Inconsistent pagination indexing between frontend and backend causes errors when calculating database offsets, leading to "OFFSET must not be negative" errors in PostgreSQL.

## Context

- Backend repositories use 1-based pagination (page 1 is the first page)
- Frontend initially used 0-based pagination in some places (page 0 as first page)
- Offset calculation formula: `(page - 1) * limit`
- When page=0: `(0 - 1) * limit = -limit` → PostgreSQL error

## Solution

**Use 1-based pagination consistently across the entire application:**

### Backend (Already Correct)

```typescript
// Repository offset calculation
query = query.limit(pagination.limit).offset((pagination.page - 1) * pagination.limit);
```

### Frontend

**1. Query Hooks**

```typescript
// ✅ Correct: Use page: 1 as default
const { data } = useProjectsQuery({
  page: 1, // First page
  limit: 100,
});
```

**2. Stateful Pagination**

```typescript
// ✅ Correct: Initialize with page 1
const [page, setPage] = useState(1); // 1-based pagination

// Reset to first page when filters change
const handleSearchChange = (value: string) => {
  setSearch(value);
  setPage(1); // Reset to first page
};
```

**3. Pagination Controls**

```typescript
// Display calculation (1-based)
<p>
  Showing {(page - 1) * limit + 1} to {Math.min(page * limit, total)}
</p>

// Navigation buttons
<Button
  onClick={() => setPage(page - 1)}
  disabled={page === 1}  // First page check
>
  Previous
</Button>
<Button
  onClick={() => setPage(page + 1)}
  disabled={page === totalPages}  // Last page check
>
  Next
</Button>
```

## Benefits

1. **Consistency**: Same pagination indexing across frontend and backend
2. **No Negative Offsets**: `(page - 1) * limit` always yields non-negative results
3. **User-Friendly**: Page numbers match user expectations (page 1, 2, 3...)
4. **RESTful Convention**: Follows common API pagination standards

## Common Mistakes to Avoid

❌ **Mistake 1: Using 0-based pagination**

```typescript
const [page, setPage] = useState(0); // 0-based
```

❌ **Mistake 2: Hardcoding page: 0 in queries**

```typescript
useProjectsQuery({ page: 0, limit: 100 }); // Wrong
```

❌ **Mistake 3: Incorrect pagination display**

```typescript
// Wrong: Shows incorrect range
<p>Showing {page * limit + 1}...</p>
```

❌ **Mistake 4: Wrong disabled logic**

```typescript
disabled={page === 0}  // Should be page === 1
```

## Implementation Checklist

- [ ] All useState hooks use `useState(1)` for page
- [ ] All query hooks pass `page: 1` as default
- [ ] Filter reset handlers use `setPage(1)`
- [ ] Pagination display uses `(page - 1) * limit + 1` formula
- [ ] Previous button disabled check: `page === 1`
- [ ] Next button disabled check: `page === totalPages`
- [ ] Backend repositories use `(page - 1) * limit` for offset

## Testing Considerations

```typescript
describe('Pagination', () => {
  it('should calculate correct offset for first page', () => {
    const page = 1;
    const limit = 10;
    const offset = (page - 1) * limit;
    expect(offset).toBe(0); // ✅ First page starts at offset 0
  });

  it('should calculate correct offset for second page', () => {
    const page = 2;
    const limit = 10;
    const offset = (page - 1) * limit;
    expect(offset).toBe(10); // ✅ Second page starts at offset 10
  });

  it('should never produce negative offset', () => {
    const page = 1;
    const limit = 10;
    const offset = (page - 1) * limit;
    expect(offset).toBeGreaterThanOrEqual(0); // ✅ Never negative
  });
});
```

## Related Patterns

- **Repository Pattern (#16)**: Implements offset calculation
- **DTO Pattern (#12)**: Pagination parameters in API contracts
- **API Response Pattern**: Pagination metadata (page, limit, total, totalPages)

## Anti-Patterns

### Anti-Pattern 1: Mixed Pagination Indexing

```typescript
// Backend uses 1-based
offset = (page - 1) * limit;

// Frontend uses 0-based
const [page, setPage] = useState(0);
// ❌ Results in negative offset!
```

### Anti-Pattern 2: Off-by-One Display Errors

```typescript
// Wrong calculation
<p>Showing {page * limit + 1}...</p>  // ❌ Off by one

// Correct calculation
<p>Showing {(page - 1) * limit + 1}...</p>  // ✅
```

## Migration Guide

If you have existing 0-based pagination:

1. **Update State Initialization**

   ```typescript
   - const [page, setPage] = useState(0);
   + const [page, setPage] = useState(1);
   ```

2. **Update Reset Handlers**

   ```typescript
   -setPage(0);
   +setPage(1);
   ```

3. **Update Display Calculations**

   ```typescript
   - Showing {page * limit + 1} to {(page + 1) * limit}
   + Showing {(page - 1) * limit + 1} to {page * limit}
   ```

4. **Update Button Disabled Logic**

   ```typescript
   - disabled={page === 0}
   + disabled={page === 1}

   - disabled={page === totalPages - 1}
   + disabled={page === totalPages}
   ```

5. **Update Query Calls**
   ```typescript
   -useQuery({ page: 0, limit: 10 }) + useQuery({ page: 1, limit: 10 });
   ```

## Real-World Example

```typescript
// ✅ Complete example with 1-based pagination
export function InvoicesPage() {
  const [page, setPage] = useState(1);  // 1-based
  const limit = 10;

  const { data } = useInvoicesQuery({
    page,  // Sends 1, 2, 3...
    limit,
  });

  return (
    <>
      {/* Pagination Display */}
      <p>
        Showing {(page - 1) * limit + 1} to{' '}
        {Math.min(page * limit, data.total)} of {data.total}
      </p>

      {/* Navigation */}
      <Button
        onClick={() => setPage(page - 1)}
        disabled={page === 1}
      >
        Previous
      </Button>
      <Button
        onClick={() => setPage(page + 1)}
        disabled={page === data.totalPages}
      >
        Next
      </Button>
    </>
  );
}
```

---

**Last Updated**: October 17, 2025
**Status**: ✅ Active - Enforced Application-Wide
