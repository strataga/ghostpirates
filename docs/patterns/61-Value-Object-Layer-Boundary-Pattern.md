# Value Object Layer Boundary Pattern

**Category:** Domain-Driven Design
**Difficulty:** Intermediate
**Last Updated:** October 17, 2025

---

## Overview

When working with Hexagonal Architecture + Domain-Driven Design, value objects (Duration, Money, HourlyRate, etc.) exist in the **domain layer** but must be converted to primitives for API responses. Understanding **which layer you're in** and **how to access value object properties** is critical to avoid null/undefined bugs.

This pattern documents the correct way to access value objects across layer boundaries.

---

## Problem

**Symptom:** API returns `null` or `undefined` for calculated values that should have data.

**Root Cause:** Confusion about value object property access when crossing layer boundaries.

### Example Bug

```typescript
// ❌ WRONG: Treating domain entities as if they were DTOs
const projects = await queryBus.execute(new GetProjectsQuery());
for (const project of projects.data) {
  // project is a Project entity with HourlyRate value object
  const rate = project.defaultHourlyRate; // This is a HourlyRate object!
  const amount = hours * rate; // ❌ NaN (multiplying by object)
}
```

**Result:** `totalAmount` calculates as `NaN` or `0` because we're multiplying by an object instead of a number.

---

## Solution

### Layer-Aware Value Object Access

**Rule:** Know which layer you're in and access value objects accordingly.

```typescript
// Domain Layer → Value Objects
export class Project {
  private defaultHourlyRate: HourlyRate; // Value object

  get defaultHourlyRate(): HourlyRate {
    return this.defaultHourlyRate;
  }
}

// Presentation Layer (DTO) → Primitives
export class ProjectResponseDto {
  defaultHourlyRate: number; // Primitive for JSON

  static fromEntity(entity: Project): ProjectResponseDto {
    return {
      // Extract primitive from value object
      defaultHourlyRate: entity.defaultHourlyRate.amount,
    };
  }
}
```

---

## Pattern Structure

### 1. Value Object in Domain Layer

```typescript
/**
 * HourlyRate Value Object
 * Encapsulates billing rate with validation
 */
export class HourlyRate {
  private readonly _amount: number;

  private constructor(amount: number) {
    if (amount < 0) {
      throw new Error('Hourly rate cannot be negative');
    }
    this._amount = amount;
  }

  static fromAmount(amount: number): HourlyRate {
    return new HourlyRate(amount);
  }

  // Getter to extract primitive
  get amount(): number {
    return this._amount;
  }
}
```

### 2. Entity Uses Value Object

```typescript
export class Project {
  private props: {
    id: string;
    name: string;
    defaultHourlyRate: HourlyRate | null; // Value object, not number
  };

  get defaultHourlyRate(): HourlyRate | null {
    return this.props.defaultHourlyRate;
  }
}
```

### 3. DTO Extracts Primitive

```typescript
export class ProjectResponseDto {
  id: string;
  name: string;
  defaultHourlyRate: number | null; // Primitive for JSON serialization

  static fromEntity(entity: Project): ProjectResponseDto {
    return {
      id: entity.id,
      name: entity.name,
      // Extract primitive from value object
      defaultHourlyRate: entity.defaultHourlyRate?.amount ?? null,
    };
  }
}
```

### 4. Controller Converts Entities to DTOs

```typescript
@Get()
async getProjects(): Promise<ProjectResponseDto[]> {
  // Query handler returns domain entities
  const result = await this.queryBus.execute(new GetProjectsQuery());

  // Convert entities to DTOs for API response
  return result.data.map(project => ProjectResponseDto.fromEntity(project));
}
```

---

## Decision Tree: How to Access Value Objects

```
┌─────────────────────────────────────────────┐
│ Are you working with entities or DTOs?      │
└────────────────┬────────────────────────────┘
                 │
      ┌──────────┴──────────┐
      │                     │
   ENTITY                  DTO
      │                     │
      ▼                     ▼
 Value Object           Primitive
      │                     │
      │                     │
Use `.getter`          Use directly
   property              as number
      │                     │
      ▼                     ▼
┌─────────────┐      ┌─────────────┐
│ .amount     │      │ rate * hours│
│ .seconds    │      │             │
│ .value      │      │             │
└─────────────┘      └─────────────┘
```

---

## Real-World Example: Time Entry Approval Calculation

### The Bug

```typescript
// Controller calculates totals for pending approvals
@Get('pending-approvals')
async getPendingApprovals(): Promise<any> {
  // Get time entries (entities)
  const entries = await this.queryBus.execute(new GetPendingApprovalsQuery());

  // Get projects (entities) for rate lookup
  const projects = await this.queryBus.execute(new GetProjectsQuery());

  let totalAmount = 0;

  for (const entry of entries.data) {
    const project = projects.data.find(p => p.id === entry.projectId);

    // ❌ BUG: Treating value objects as primitives
    const hours = entry.duration / 3600; // duration is Duration object!
    const rate = project.defaultHourlyRate; // This is HourlyRate object!
    totalAmount += hours * rate; // NaN result!
  }

  return { totalAmount }; // Returns NaN or 0
}
```

### The Fix

```typescript
// Controller calculates totals for pending approvals
@Get('pending-approvals')
async getPendingApprovals(): Promise<any> {
  // Get time entries (entities)
  const entries = await this.queryBus.execute(new GetPendingApprovalsQuery());

  // Get projects (entities) for rate lookup
  const projects = await this.queryBus.execute(new GetProjectsQuery());

  let totalAmount = 0;

  for (const entry of entries.data) {
    const project = projects.data.find(p => p.id === entry.projectId);

    // ✅ CORRECT: Extract primitives from value objects
    const durationInSeconds = entry.duration?.seconds || entry.duration;
    const hours = durationInSeconds / 3600;

    const rate = project.defaultHourlyRate?.amount; // Extract number

    if (rate && hours) {
      totalAmount += hours * rate; // Correct calculation
    }
  }

  return {
    totalAmount: Math.round(totalAmount * 100) / 100 // Round to 2 decimals
  };
}
```

---

## Common Value Objects and Their Getters

| Value Object  | Domain Layer    | Primitive Getter | Example            |
| ------------- | --------------- | ---------------- | ------------------ |
| `Duration`    | Time tracking   | `.seconds`       | `duration.seconds` |
| `HourlyRate`  | Billing         | `.amount`        | `rate.amount`      |
| `Money`       | Payments        | `.amount`        | `money.amount`     |
| `Email`       | User management | `.value`         | `email.value`      |
| `ProjectSlug` | Projects        | `.value`         | `slug.value`       |

---

## Anti-Patterns

### ❌ Mixing Layers

```typescript
// BAD: Using value objects in DTOs
export class ProjectResponseDto {
  defaultHourlyRate: HourlyRate; // ❌ Value object in DTO
}
```

### ❌ Skipping DTO Conversion

```typescript
// BAD: Returning entities directly from controller
@Get()
async getProjects(): Promise<Project[]> {
  return this.queryBus.execute(new GetProjectsQuery());
  // ❌ Returns entities with value objects, not JSON-safe
}
```

### ❌ Accessing Non-Existent Properties

```typescript
// BAD: Assuming DTO structure on entity
const rate = project.defaultHourlyRate.amount;
// ❌ Works for entities but fails if project is a DTO
// (DTOs have primitive, not value object)
```

---

## Best Practices

### 1. **Always Convert at Controller Boundary**

```typescript
@Get()
async getProjects(): Promise<ProjectResponseDto[]> {
  const result = await this.queryBus.execute(new GetProjectsQuery());

  // Convert at boundary (controller → response)
  return result.data.map(entity => ProjectResponseDto.fromEntity(entity));
}
```

### 2. **Use Null-Safe Access**

```typescript
// Handle null/undefined value objects safely
const rate = project.defaultHourlyRate?.amount ?? null;
const duration = entry.duration?.seconds || 0;
```

### 3. **Document Value Object Getters**

```typescript
/**
 * Duration Value Object
 * Represents time duration in seconds
 */
export class Duration {
  private readonly _seconds: number;

  /**
   * Get duration in seconds
   * @returns {number} Duration as seconds
   */
  get seconds(): number {
    return this._seconds;
  }

  /**
   * Get duration in hours
   * @returns {number} Duration as hours (seconds / 3600)
   */
  get hours(): number {
    return this._seconds / 3600;
  }
}
```

### 4. **Type Guards for Layer Detection**

```typescript
// Helper to detect if working with entity or DTO
function isEntity(obj: any): obj is Project {
  return obj.defaultHourlyRate instanceof HourlyRate;
}

function getRate(project: Project | ProjectResponseDto): number | null {
  if (isEntity(project)) {
    return project.defaultHourlyRate?.amount ?? null; // Entity
  }
  return project.defaultHourlyRate ?? null; // DTO (already primitive)
}
```

---

## Testing Value Object Access

```typescript
describe('Value Object Layer Boundary', () => {
  it('should extract primitive from value object in entity', () => {
    const rate = HourlyRate.fromAmount(150);
    const project = Project.create({
      name: 'Test Project',
      defaultHourlyRate: rate,
    });

    // Access via getter
    expect(project.defaultHourlyRate.amount).toBe(150);
  });

  it('should convert entity to DTO with primitive', () => {
    const rate = HourlyRate.fromAmount(150);
    const project = Project.create({
      name: 'Test Project',
      defaultHourlyRate: rate,
    });

    const dto = ProjectResponseDto.fromEntity(project);

    // DTO has primitive, not value object
    expect(dto.defaultHourlyRate).toBe(150);
    expect(typeof dto.defaultHourlyRate).toBe('number');
  });

  it('should calculate correctly using value object getters', () => {
    const duration = Duration.fromSeconds(3600); // 1 hour
    const rate = HourlyRate.fromAmount(150);

    const hours = duration.seconds / 3600;
    const amount = hours * rate.amount;

    expect(amount).toBe(150);
  });
});
```

---

## Related Patterns

- **03-Hexagonal-Architecture.md** - Layer separation principles
- **04-Domain-Driven-Design.md** - Value object pattern
- **07-DTO-Pattern.md** - Data transfer object conversion
- **59-Pagination-Offset-Calculation-Pattern.md** - Similar boundary confusion issue

---

## Key Takeaways

`★ Insight ─────────────────────────────────────`

**1. Know Your Layer**

- **Domain Layer** → Value objects with getters (`.amount`, `.seconds`)
- **Presentation Layer** → DTOs with primitives (numbers, strings)

**2. Always Convert at Boundaries**

- Controllers receive entities from query handlers
- Controllers return DTOs to HTTP responses
- Use `fromEntity()` factory methods

**3. Access Value Objects Correctly**

- **Entities:** `project.defaultHourlyRate.amount`
- **DTOs:** `project.defaultHourlyRate` (already a number)

**4. Debug Checklist**
When calculations return null/NaN:

1. ✅ Are you accessing `.amount` or `.seconds` on value objects?
2. ✅ Are you working with entities or DTOs?
3. ✅ Did you convert entities to DTOs at the controller?
4. ✅ Are you handling null/undefined value objects safely?

`─────────────────────────────────────────────────`

---

## Conclusion

Value objects are a cornerstone of Domain-Driven Design, encapsulating business logic and validation. However, they must be converted to primitives when crossing architectural layer boundaries (domain → presentation). Understanding **which layer you're in** and **how to access value object properties** prevents common bugs like null calculations and NaN results.

**Golden Rule:** Domain entities use value objects. DTOs use primitives. Convert at the controller boundary.
