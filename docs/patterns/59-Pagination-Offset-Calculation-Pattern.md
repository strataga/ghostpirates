# Pattern 59: Pagination Offset Calculation Pattern

**Category**: Data Access
**Complexity**: Low
**Last Updated**: October 17, 2025

---

## Overview

The **Pagination Offset Calculation Pattern** ensures consistent and correct pagination behavior across all repository implementations. This pattern addresses the critical distinction between 0-based and 1-based page indexing when calculating SQL OFFSET values.

### Problem

A subtle but critical bug in pagination offset calculation causes the first page of results to be skipped:

```typescript
// ❌ INCORRECT: Treats page as 0-based when it's actually 1-based
query.offset(pagination.page * pagination.limit);

// Example:
// - Frontend requests page=1, limit=10
// - Offset = 1 * 10 = 10 (skips first 10 records!)
// - Returns records 11-20 instead of 1-10
```

This bug manifests as:

- **Empty first page**: When total records < limit, first page appears empty
- **Data mismatch**: API returns `total: 5` but `clients: []`
- **Skipped records**: First page of results is never visible

### Solution

Always use 1-based page indexing with the correct offset calculation:

```typescript
// ✅ CORRECT: Convert 1-based page to 0-based offset
query.offset((pagination.page - 1) * pagination.limit);

// Example:
// - Frontend requests page=1, limit=10
// - Offset = (1 - 1) * 10 = 0 (starts at first record)
// - Returns records 1-10 ✓
```

---

## When to Use

Apply this pattern in **every repository method** that implements pagination:

- ✅ **All `findByOrganization()` methods**
- ✅ **All `search()` methods**
- ✅ **All `findByAssignedUser()` methods**
- ✅ **Any custom query with pagination support**

Affected repositories in this codebase:

- `DrizzleClientRepository`
- `DrizzleProjectRepository`
- `DrizzleInvoiceRepository`
- `DrizzleTimeEntryRepository`

---

## Implementation

### Repository Interface

Define pagination options with **1-based page numbers**:

```typescript
export interface PaginationOptions {
  page: number; // 1-based: first page = 1
  limit: number; // Number of items per page
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
}

export interface PaginatedResult<T> {
  data: T[];
  total: number;
  page: number; // Echo back the requested page
  limit: number;
  totalPages: number;
}
```

### Repository Implementation

**Pattern Template (Rust + SQLx)**:

```rust
pub async fn find_by_organization(
    pool: &sqlx::PgPool,
    organization_id: &str,
    filters: Option<Filters>,
    pagination: Option<Pagination>,
) -> Result<PaginatedResult<Entity>, sqlx::Error> {
    let pagination = pagination.unwrap_or(Pagination {
        page: 1,
        limit: 50,
    });

    // Apply pagination (1-based page indexing)
    let offset = (pagination.page - 1) * pagination.limit; // ✓ CRITICAL

    // Build WHERE conditions
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM table WHERE organization_id = "
    );
    query_builder.push_bind(organization_id);

    // Add additional filters...
    // (filter building logic here)

    query_builder.push(" ORDER BY name ASC");

    // Execute query with pagination in parallel with count query
    let query_sql = query_builder.sql();

    let (rows, total) = tokio::try_join!(
        async {
            sqlx::query_as::<_, EntityRow>(
                &format!("{} LIMIT $1 OFFSET $2", query_sql)
            )
            .bind(pagination.limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        },
        async {
            sqlx::query_scalar::<_, i64>(
                &format!("SELECT COUNT(*) FROM ({})", query_sql)
            )
            .fetch_one(pool)
            .await
        }
    )?;

    Ok(PaginatedResult {
        data: rows.into_iter().map(|row| to_domain(row)).collect(),
        total: total as usize,
        page: pagination.page,
        limit: pagination.limit,
        total_pages: ((total as usize + pagination.limit - 1) / pagination.limit),
    })
}
```

### Controller/API Layer

Controllers should pass through 1-based page numbers from query params:

```typescript
@Get()
async getAll(
  @CurrentUser() user: CurrentUserData,
  @Query('page') page?: number,
  @Query('limit') limit?: number,
): Promise<PaginatedResponse> {
  const query = new GetClientsQuery(
    user.organizationId!,
    undefined, // search
    page ?? 1,  // Default to page 1 (not 0)
    limit ?? 10,
  );

  return await this.queryBus.execute(query);
}
```

### Frontend/Client Layer

Frontend should request 1-based pages:

```typescript
// React Query example
const { data } = useQuery({
  queryKey: ['clients', page],
  queryFn: () =>
    fetchClients({
      page: page, // 1 for first page, 2 for second, etc.
      limit: 10,
    }),
});
```

---

## Real-World Example

### Bug Manifestation (Fixed in Sprint 8A)

**Scenario**: Clients page at `http://localhost:3000/clients`

**API Response (Before Fix)**:

```json
{
  "clients": [],
  "total": 5,
  "page": 1,
  "limit": 10,
  "totalPages": 1
}
```

**Root Cause**:

```typescript
// drizzle-client.repository.ts:116
.offset(pagination.page * pagination.limit)
// With page=1, limit=10: offset = 10
// Skipped all 5 clients (records 0-4) and tried to fetch records 10-19
```

**Fix Applied**:

```typescript
// drizzle-client.repository.ts:116
.offset((pagination.page - 1) * pagination.limit)
// With page=1, limit=10: offset = 0
// Correctly fetches records 0-9 (all 5 clients)
```

**API Response (After Fix)**:

```json
{
  "clients": [
    { "id": "1", "name": "Acme Corporation", ... },
    { "id": "2", "name": "Global Enterprises", ... },
    { "id": "3", "name": "Healthcare Plus", ... },
    { "id": "4", "name": "Retail Solutions Co", ... },
    { "id": "5", "name": "TechStart Inc", ... }
  ],
  "total": 5,
  "page": 1,
  "limit": 10,
  "totalPages": 1
}
```

---

## Testing Strategy

### Unit Tests

Test pagination offset calculation:

```typescript
describe('Pagination', () => {
  it('should return first page correctly', async () => {
    const result = await repository.findByOrganization(orgId, {}, { page: 1, limit: 10 });

    expect(result.data).toHaveLength(5); // All 5 records
    expect(result.page).toBe(1);
    expect(result.total).toBe(5);
  });

  it('should handle second page correctly', async () => {
    // Given 25 total records
    const result = await repository.findByOrganization(orgId, {}, { page: 2, limit: 10 });

    expect(result.data).toHaveLength(10); // Records 11-20
    expect(result.page).toBe(2);
    expect(result.total).toBe(25);
  });

  it('should handle last partial page', async () => {
    // Given 25 total records
    const result = await repository.findByOrganization(orgId, {}, { page: 3, limit: 10 });

    expect(result.data).toHaveLength(5); // Records 21-25
    expect(result.page).toBe(3);
    expect(result.total).toBe(25);
  });
});
```

### Integration Tests

Test end-to-end pagination flow:

```typescript
describe('GET /api/v1/clients', () => {
  it('should return first page of clients', async () => {
    const response = await request(app.getHttpServer())
      .get('/api/v1/clients?page=1&limit=10')
      .expect(200);

    expect(response.body.clients).toHaveLength(5);
    expect(response.body.total).toBe(5);
    expect(response.body.page).toBe(1);
  });
});
```

---

## Common Mistakes

### ❌ Mistake 1: Inconsistent Page Indexing

```typescript
// Controller uses 1-based pages
page ??
  (1)

    // But repository treats it as 0-based
    .offset(pagination.page * pagination.limit);
```

**Impact**: First page is skipped, empty results appear

### ❌ Mistake 2: Missing Comment Documentation

```typescript
// No indication of page indexing convention
if (pagination) {
  query = query.limit(pagination.limit).offset(pagination.page * pagination.limit);
}
```

**Solution**: Always add comment indicating 1-based indexing:

```typescript
// Apply pagination (1-based page indexing)
if (pagination) {
  query = query.limit(pagination.limit).offset((pagination.page - 1) * pagination.limit);
}
```

### ❌ Mistake 3: Defaulting to Page 0

```typescript
// Controller defaults to page 0
page ?? 0; // ❌ Wrong!

// Should default to page 1
page ?? 1; // ✅ Correct
```

---

## Anti-Patterns

### ❌ Anti-Pattern: Mixed Indexing

Don't mix 0-based and 1-based indexing in different parts of the system:

```typescript
// Frontend: 1-based
const page = 1;

// API: Converts to 0-based
const apiPage = page - 1;

// Repository: Expects 0-based
.offset(pagination.page * pagination.limit)

// Response: Converts back to 1-based
return { page: pagination.page + 1 };
```

**Why It's Bad**:

- Adds unnecessary complexity
- Error-prone during maintenance
- Confusing for developers

**Better Approach**: Use 1-based throughout, convert only at SQL layer:

```typescript
// Everywhere uses 1-based pages
const page = 1; // Frontend
pagination.page = 1; // API
offset = (pagination.page - 1) * limit; // SQL (only place that converts)
```

---

## Benefits

1. **Consistent UX**: First page always shows first records
2. **Predictable Behavior**: Page numbers match user expectations (1, 2, 3...)
3. **Easier Debugging**: "Page 1 is empty" becomes "offset calculation is wrong"
4. **Standard Compliance**: Aligns with common pagination conventions

---

## Related Patterns

- **Pattern 06: Repository Pattern** - Pagination is part of repository query methods
- **Pattern 05: CQRS Pattern** - Query handlers use pagination for list queries
- **Pattern 07: DTO Pattern** - PaginatedResult is a DTO that encapsulates pagination metadata

---

## References

- **Bug Fix Commit**: Sprint 8A - Fixed pagination offset in 4 repositories
- **Affected Files**:
  - `client_repository.rs`
  - `project_repository.rs`
  - `invoice_repository.rs`
  - `time_entry_repository.rs`

---

## Checklist for New Paginated Queries

- [ ] Use 1-based page numbers in API and query parameters
- [ ] Calculate offset as `(page - 1) * limit`
- [ ] Add comment: `// Apply pagination (1-based page indexing)`
- [ ] Default page to `1` (not `0`)
- [ ] Return page number in response
- [ ] Write tests for page 1, page 2, and last partial page
- [ ] Verify first page shows first records in browser/Postman

---

## Example: Complete Paginated Repository Method

```rust
pub async fn find_by_organization(
    pool: &sqlx::PgPool,
    organization_id: &str,
    filters: Option<ClientFilters>,
    pagination: Option<Pagination>,
) -> Result<PaginatedResult<Client>, sqlx::Error> {
    let pagination = pagination.unwrap_or(Pagination {
        page: 1,
        limit: 50,
    });

    // Apply pagination (1-based page indexing)
    let offset = (pagination.page - 1) * pagination.limit;

    // Build WHERE conditions
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT * FROM clients WHERE organization_id = "
    );
    query_builder.push_bind(organization_id);
    query_builder.push(" AND deleted_at IS NULL");

    // Add filters if provided
    if let Some(f) = filters {
        if let Some(search) = f.search {
            let pattern = format!("%{}%", search.to_lowercase());
            query_builder.push(" AND LOWER(name) LIKE ");
            query_builder.push_bind(pattern);
        }
    }

    query_builder.push(" ORDER BY name ASC");

    // Execute query with pagination in parallel with count
    let query_sql = query_builder.sql();

    let (rows, total) = tokio::try_join!(
        async {
            sqlx::query_as::<_, ClientRow>(
                &format!("{} LIMIT $1 OFFSET $2", query_sql)
            )
            .bind(pagination.limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        },
        async {
            sqlx::query_scalar::<_, i64>(
                &format!("SELECT COUNT(*) FROM ({})", query_sql)
            )
            .fetch_one(pool)
            .await
        }
    )?;

    // Return paginated result
    Ok(PaginatedResult {
        data: rows.into_iter().map(|row| to_domain(row)).collect(),
        total: total as usize,
        page: pagination.page,
        limit: pagination.limit,
        total_pages: ((total as usize + pagination.limit - 1) / pagination.limit),
    })
}
```

---

**★ Insight ─────────────────────────────────────**
This pattern documents a subtle but critical bug that affected every paginated query in the codebase. The fix required changing a single calculation across 4 repositories, demonstrating the importance of pattern consistency. Always document pagination indexing conventions explicitly to prevent this class of bug.
**─────────────────────────────────────────────────**
