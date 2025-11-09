# Pattern 41: Database Constraint Race Condition Handling

## Problem

When multiple concurrent requests try to create entities with unique constraints (e.g., organization slugs), a **check-then-act race condition** can occur:

```
Time    Thread A                          Thread B
----    --------                          --------
T1      Check if "acme" exists → false
T2                                        Check if "acme" exists → false
T3      Create org with "acme" → SUCCESS
T4                                        Create org with "acme" → 💥 409!
```

The problem: Between checking availability and inserting, another thread can insert the same value.

## Anti-Pattern: Pre-Check with No Atomicity

❌ **Don't do this:**

```typescript
async execute(command: CreateCommand) {
  // Check if slug exists
  const exists = await repository.existsBySlug(slug);

  if (exists) {
    // Generate new slug and retry
    slug = generateUniqueSlug();
  }

  // Race condition window here! 👈
  await repository.save(entity);
}
```

**Why it fails:** No atomicity between check and insert.

## Solution: Database-Level Retry Pattern

✅ **Do this instead:**

```typescript
async execute(command: CreateCommand) {
  const maxAttempts = 5;
  let attempt = 0;

  while (attempt < maxAttempts) {
    try {
      // Generate slug
      const slug = attempt === 0
        ? baseSlug
        : generateUniqueSlug(baseSlug, attempt);

      // Let database enforce uniqueness
      await repository.save(entity);

      return success; // ✅ Success!

    } catch (error: any) {
      // Check if unique constraint violation
      const isSlugConflict =
        error.code === '23505' && // PostgreSQL unique violation
        error.constraint?.includes('slug');

      if (!isSlugConflict) {
        throw error; // Different error, rethrow
      }

      // Retry with exponential backoff
      attempt++;
      if (attempt < maxAttempts) {
        await sleep(Math.pow(2, attempt) * 10); // 10ms, 20ms, 40ms, 80ms
      }
    }
  }

  throw new Error('Max retries exceeded');
}
```

## Key Principles

### 1. **Let Database Enforce Uniqueness**

- Database unique constraints are atomic
- No race condition possible at DB level
- Simpler and more reliable than application-level locking

### 2. **Catch and Retry Pattern**

- Catch database unique constraint errors
- Retry with new value
- Exponential backoff prevents database hammering

### 3. **Error Code Detection**

- PostgreSQL: error code `23505` = unique violation
- MySQL: error code `ER_DUP_ENTRY` = duplicate entry
- Check `error.constraint` to identify which constraint failed

### 4. **Bounded Retries**

- Always set max retry limit (e.g., 5 attempts)
- Use exponential backoff: `Math.pow(2, attempt) * baseDelay`
- Fail gracefully after max retries

## Complete Implementation

### Handler with Retry Logic

```typescript
@CommandHandler(CreateOrganizationCommand)
export class CreateOrganizationHandler {
  async execute(command: CreateOrganizationCommand) {
    const baseSlug = OrganizationSlug.fromEmail(command.ownerEmail);
    const maxAttempts = 5;
    let attempt = 0;
    let lastError: Error | null = null;

    while (attempt < maxAttempts) {
      try {
        // Generate slug (with suffix on retries)
        const slug = attempt === 0 ? baseSlug : this.generateUniqueSlug(baseSlug, attempt);

        // Create entity
        const organization = Organization.create({
          ...command,
          slug,
        });

        // Let database enforce uniqueness
        await this.repository.save(organization);

        // Success!
        return this.toResult(organization);
      } catch (error: any) {
        // Detect unique constraint violation
        const isSlugConflict = error.code === '23505' && error.constraint?.includes('slug');

        if (!isSlugConflict) {
          throw error; // Not a slug conflict, rethrow
        }

        // Prepare for retry
        lastError = error;
        attempt++;

        // Exponential backoff
        if (attempt < maxAttempts) {
          await this.sleep(Math.pow(2, attempt) * 10);
        }
      }
    }

    // Max retries exceeded
    throw new Error(`Failed after ${maxAttempts} attempts. Last error: ${lastError?.message}`);
  }

  private generateUniqueSlug(base: Slug, attempt: number): Slug {
    if (attempt <= 3) {
      // Random suffix for first 3 attempts
      const suffix = randomBytes(3).toString('hex'); // 6 hex chars
      return Slug.withSuffix(base.getValue(), suffix);
    } else {
      // Timestamp suffix for remaining (guaranteed unique)
      const timestamp = Date.now().toString(36);
      return Slug.withSuffix(base.getValue(), timestamp);
    }
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
```

### Database Schema

```typescript
export const organizationsTable = pgTable('organizations', {
  id: text('id').primaryKey(),
  name: text('name').notNull(),
  slug: text('slug').unique().notNull(), // 👈 Unique constraint
  createdAt: timestamp('created_at').notNull().defaultNow(),
});
```

## Testing

### Load Test Verification

```yaml
# artillery-registration.yml
config:
  phases:
    - duration: 60
      arrivalRate: 50 # 50 concurrent users

scenarios:
  - name: 'Concurrent Registration'
    flow:
      - post:
          url: '/api/v1/auth/register'
          json:
            email: 'user-{{ $randomString() }}@domain.com'
```

**Expected results:**

- ✅ No `organizations_slug_unique` constraint errors
- ✅ Slugs auto-suffixed on conflicts (e.g., `acme-a3f21c`)
- ✅ All registrations succeed (or fail for legitimate reasons)

### Manual Test

```bash
# Concurrent registration test
for i in {1..10}; do
  curl -X POST http://localhost:4001/api/v1/auth/register \
    -H "Content-Type: application/json" \
    -d "{
      \"email\": \"user${i}@acme.com\",
      \"password\": \"Test123!\",
      \"firstName\": \"User\",
      \"lastName\": \"${i}\"
    }" &
done
wait

# Check created organizations
psql -d wellos -c "SELECT slug FROM organizations WHERE slug LIKE 'acme%';"
```

Expected output:

```
     slug
---------------
 acme
 acme-a3f21c
 acme-b7e45d
 ...
```

## Performance Considerations

### Retry Overhead

- **First attempt:** No overhead (try original slug)
- **Retry 1:** 20ms delay + database round-trip
- **Retry 2:** 40ms delay + database round-trip
- **Retry 3:** 80ms delay + database round-trip

### When to Use

✅ **Use when:**

- Unique constraints on user-facing values (slugs, usernames)
- Concurrent creation expected
- Automatic suffix generation acceptable

❌ **Don't use when:**

- Hard uniqueness required (no suffixes allowed)
- Low concurrency (< 10 concurrent users)
- Pre-generated unique IDs available (UUIDs)

## Alternatives

### 1. **Optimistic Locking**

- Use version numbers
- Retry on version mismatch
- Similar to this pattern but for updates

### 2. **Database Locks**

```sql
SELECT * FROM organizations
WHERE slug = 'acme'
FOR UPDATE;
```

- ❌ Reduces concurrency
- ❌ Can cause deadlocks
- ✅ Guarantees exclusivity

### 3. **UUID-based Slugs**

```typescript
const slug = `${baseName}-${uuidv4().slice(0, 8)}`;
```

- ✅ No collisions
- ❌ Less user-friendly
- ❌ Not memorable

## Related Patterns

- **Pattern 10:** Unit of Work (for transactions)
- **Pattern 8:** Specification (for validation)
- **Pattern 7:** Circuit Breaker (for external service retries)

## Real-World Example: WellOS PSA

**Scenario:** Multiple users from `acme.com` register simultaneously

**Before fix:**

```
User1@acme.com → Check slug "acme" → available → Create → ✅
User2@acme.com → Check slug "acme" → available → Create → 💥 409!
User3@acme.com → Check slug "acme" → available → Create → 💥 409!
```

**After fix:**

```
User1@acme.com → Try "acme" → Success → ✅
User2@acme.com → Try "acme" → 409 → Retry "acme-a3f21c" → ✅
User3@acme.com → Try "acme" → 409 → Retry "acme-b7e45d" → ✅
```

## References

- PostgreSQL Error Codes: https://www.postgresql.org/docs/current/errcodes-appendix.html
- Circuit Breaker Pattern: Pattern 7
- Exponential Backoff: https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
