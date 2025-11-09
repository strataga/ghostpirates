# Domain-Driven Design (DDD) Pattern

## Overview

Domain-Driven Design is a software design approach that focuses on modeling
software to match the business domain. It emphasizes collaboration between
domain experts and developers to create a shared understanding of the problem
space through a ubiquitous language.

## Core Concepts

### Domain Model

The heart of DDD, representing the business logic and rules.

### Ubiquitous Language

A common vocabulary shared by all team members, including domain experts and
developers.

### Bounded Context

Clear boundaries that define where a particular model applies and where it
doesn't.

### Aggregates

Clusters of domain objects that can be treated as a single unit for data
changes.

### Entities

Objects that have a distinct identity that runs through time and different
representations.

### Value Objects

Objects that describe characteristics of a thing but have no identity.

## Benefits

- **Better Business Alignment**: Software structure reflects business domain
- **Improved Communication**: Ubiquitous language reduces misunderstandings
- **Maintainable Code**: Clear domain boundaries and responsibilities
- **Flexibility**: Well-defined contexts allow for independent evolution
- **Rich Domain Models**: Business logic is centralized and explicit

## Implementation in Our Project

### Before: Anemic Domain Model

```typescript
// Anemic entity with just data
class Vendor {
  id: string;
  name: string;
  code: string;
  status: string;
  createdAt: Date;
  updatedAt: Date;
}

// Business logic scattered in services
class VendorService {
  async activateVendor(vendorId: string) {
    const vendor = await this.repository.findById(vendorId);
    if (!vendor) throw new Error('Vendor not found');

    // Business rules scattered across services
    if (vendor.status === 'BLOCKED') {
      throw new Error('Cannot activate blocked vendor');
    }

    vendor.status = 'ACTIVE';
    vendor.updatedAt = new Date();

    await this.repository.save(vendor);
    await this.auditService.log('VENDOR_ACTIVATED', vendor.id);
  }
}
```

### After: Rich Domain Model

```typescript
// Rich domain entity with behavior
export class Vendor {
  private constructor(
    private readonly id: VendorId,
    private readonly organizationId: string,
    private name: VendorName,
    private code: VendorCode,
    private status: VendorStatus,
    private contactInfo: ContactInfo,
    private insurance: Insurance,
    private readonly createdAt: Date,
    private updatedAt: Date,
    private readonly domainEvents: DomainEvent[] = [],
  ) {}

  // Factory method
  static create(data: CreateVendorData): Vendor {
    const vendor = new Vendor(
      VendorId.generate(),
      data.organizationId,
      new VendorName(data.name),
      new VendorCode(data.code),
      VendorStatus.PENDING,
      new ContactInfo(data.contactInfo),
      new Insurance(data.insurance),
      new Date(),
      new Date(),
    );

    vendor.addDomainEvent(new VendorCreatedEvent(vendor.id, vendor.organizationId));
    return vendor;
  }

  // Business behavior encapsulated
  activate(): void {
    if (this.status.isBlocked()) {
      throw new VendorDomainError('Cannot activate blocked vendor');
    }

    if (this.status.isActive()) {
      return; // Already active, no-op
    }

    this.status = VendorStatus.ACTIVE;
    this.updatedAt = new Date();
    this.addDomainEvent(new VendorActivatedEvent(this.id));
  }

  // Invariants enforced
  updateInsurance(insurance: Insurance): void {
    if (this.status.isActive() && !insurance.isValid()) {
      throw new VendorDomainError('Active vendors must have valid insurance');
    }

    this.insurance = insurance;
    this.updatedAt = new Date();
  }

  // Domain events for side effects
  private addDomainEvent(event: DomainEvent): void {
    this.domainEvents.push(event);
  }

  getDomainEvents(): readonly DomainEvent[] {
    return this.domainEvents;
  }

  clearDomainEvents(): void {
    this.domainEvents.length = 0;
  }
}

// Value Objects with validation
export class VendorName {
  private readonly value: string;

  constructor(name: string) {
    if (!name || name.trim().length === 0) {
      throw new VendorDomainError('Vendor name is required');
    }

    if (name.length > 255) {
      throw new VendorDomainError('Vendor name cannot exceed 255 characters');
    }

    this.value = name.trim();
  }

  getValue(): string {
    return this.value;
  }

  equals(other: VendorName): boolean {
    return this.value === other.value;
  }
}

// Domain Services for complex business logic
export class VendorDomainService {
  constructor(private readonly vendorRepository: IVendorRepository) {}

  async canDeactivateVendor(vendorId: VendorId): Promise<boolean> {
    const vendor = await this.vendorRepository.findById(vendorId);
    if (!vendor) return false;

    // Complex business rule involving multiple aggregates
    const activeContracts = await this.vendorRepository.countActiveContracts(vendorId);
    const pendingPayments = await this.vendorRepository.countPendingPayments(vendorId);

    return activeContracts === 0 && pendingPayments === 0;
  }
}
```

### Bounded Context Example

```typescript
// Vendor Management Context
export namespace VendorManagement {
  export class Vendor {
    // Focus on vendor lifecycle, compliance, contracts
  }

  export class VendorRepository {
    // Persistence specific to vendor management
  }
}

// Financial Context
export namespace Financial {
  export class Vendor {
    // Focus on payment terms, tax info, banking
  }

  export class PaymentRepository {
    // Persistence specific to payments
  }
}

// The same real-world entity (vendor) has different representations
// in different contexts based on what's relevant in that context
```

### Application Service (Orchestration)

```typescript
@Injectable()
export class CreateVendorHandler implements ICommandHandler<CreateVendorCommand> {
  constructor(
    private readonly vendorRepository: IVendorRepository,
    private readonly domainEventPublisher: IDomainEventPublisher,
  ) {}

  async execute(command: CreateVendorCommand): Promise<string> {
    // Check uniqueness (domain rule)
    const existingVendor = await this.vendorRepository.findByCode(new VendorCode(command.code));

    if (existingVendor) {
      throw new VendorAlreadyExistsError(command.code);
    }

    // Create rich domain object
    const vendor = Vendor.create({
      organizationId: command.organizationId,
      name: command.name,
      code: command.code,
      contactInfo: command.contactInfo,
      insurance: command.insurance,
    });

    // Persist
    await this.vendorRepository.save(vendor);

    // Publish domain events
    const events = vendor.getDomainEvents();
    for (const event of events) {
      await this.domainEventPublisher.publish(event);
    }

    vendor.clearDomainEvents();

    return vendor.getId().getValue();
  }
}
```

## DDD Layers in Our Architecture

### 1. Domain Layer (Core)

- **Entities**: `Vendor`, `LeaseOperatingStatement`, `User`
- **Value Objects**: `VendorCode`, `StatementMonth`, `Money`
- **Domain Services**: `VendorDomainService`
- **Domain Events**: `VendorCreatedEvent`, `LosFinalized`
- **Repositories (Interfaces)**: `IVendorRepository`

### 2. Application Layer

- **Command Handlers**: `CreateVendorHandler`, `FinalizeLosHandler`
- **Query Handlers**: `GetVendorByIdHandler`
- **Application Services**: Orchestration of domain operations
- **DTOs**: Data transfer between boundaries

### 3. Infrastructure Layer

- **Repository Implementations**: `VendorRepositoryImpl`
- **Database Schemas**: SQL migrations + SQLx
- **External Services**: Email, notifications
- **Event Publishers**: Domain event infrastructure

### 4. Presentation Layer

- **Controllers**: REST API endpoints
- **Guards**: Authentication and authorization
- **Validation**: Input validation pipes

## Common DDD Patterns in Our Codebase

### Aggregate Root

```typescript
export class LeaseOperatingStatement {
  // Aggregate root that controls access to expense line items
  private expenseLineItems: ExpenseLineItem[] = [];

  addExpense(expense: ExpenseLineItemData): void {
    // Business rules enforced at aggregate level
    if (this.status.isFinalized()) {
      throw new LosDomainError('Cannot add expenses to finalized LOS');
    }

    const expenseItem = ExpenseLineItem.create(expense);
    this.expenseLineItems.push(expenseItem);
    this.recalculateTotals();

    this.addDomainEvent(new ExpenseAddedEvent(this.id, expenseItem.id));
  }

  // Aggregate consistency
  private recalculateTotals(): void {
    this.totalExpenses = Money.sum(...this.expenseLineItems.map((item) => item.amount));
  }
}
```

### Repository Pattern (Domain Interface)

```typescript
export interface IVendorRepository {
  findById(id: VendorId): Promise<Vendor | null>;
  findByCode(code: VendorCode): Promise<Vendor | null>;
  save(vendor: Vendor): Promise<void>;
  delete(id: VendorId): Promise<void>;

  // Domain-specific queries
  findActiveVendorsByOrganization(orgId: string): Promise<Vendor[]>;
  findVendorsWithExpiredInsurance(): Promise<Vendor[]>;
}
```

### Domain Events

```typescript
export class VendorActivatedEvent implements DomainEvent {
  constructor(
    public readonly vendorId: VendorId,
    public readonly occurredAt: Date = new Date(),
  ) {}

  getAggregateId(): string {
    return this.vendorId.getValue();
  }

  getEventName(): string {
    return 'VendorActivated';
  }
}

// Event Handler (Application Layer)
@EventHandler(VendorActivatedEvent)
export class VendorActivatedEventHandler {
  constructor(private readonly notificationService: NotificationService) {}

  async handle(event: VendorActivatedEvent): Promise<void> {
    await this.notificationService.notifyVendorActivated(event.vendorId);
  }
}
```

## Best Practices

### 1. Ubiquitous Language

- Use domain terminology consistently across code and documentation
- Avoid technical jargon in domain models
- Regular sessions with domain experts to refine language

### 2. Aggregate Design

- Keep aggregates small and focused
- Enforce invariants within aggregate boundaries
- Use eventual consistency between aggregates

### 3. Domain Events

- Use events for side effects and cross-aggregate communication
- Keep events focused on domain concepts
- Handle events asynchronously when possible

### 4. Value Objects

- Make them immutable
- Implement proper equality
- Encapsulate validation logic

### 5. Bounded Context Integration

- Use Anti-Corruption Layer for external systems
- Define clear contracts between contexts
- Avoid sharing domain models across contexts

## Domain Implementation Strategy

### Build Order: Bottom-Up Approach

When implementing a new feature with DDD, follow this proven order:

#### 1. Value Objects First

**Why:** Value objects are the building blocks of your entities. They encapsulate validation, formatting, and business rules for primitive concepts.

**Build reusable domain primitives before the entities that use them:**

```typescript
// Step 1: Create shared value objects
export class Money {
  private readonly _amount: number;
  private readonly _currency: string;

  static fromAmount(amount: number): Money {
    if (amount < 0) {
      throw new Error('Money cannot be negative');
    }
    return new Money(amount);
  }

  add(other: Money): Money {
    if (this._currency !== other._currency) {
      throw new Error('Cannot add different currencies');
    }
    return Money.fromAmount(this._amount + other._amount);
  }
}

export class HourlyRate {
  private readonly _rate: Money;

  static fromAmount(amount: number): HourlyRate {
    if (amount > 10_000) {
      throw new Error('Rate cannot exceed $10,000/hour');
    }
    return new HourlyRate(Money.fromAmount(amount));
  }

  calculateCost(hours: number): Money {
    return this._rate.multiply(hours);
  }
}

// Step 2: Create feature-specific value objects
export class ProjectSlug {
  private readonly _value: string;

  static fromString(value: string): ProjectSlug {
    const slugRegex = /^[a-z0-9]+(?:-[a-z0-9]+)*$/;
    if (!slugRegex.test(value)) {
      throw new Error('Invalid slug format');
    }
    return new ProjectSlug(value);
  }

  static fromProjectName(name: string): ProjectSlug {
    const slug = name
      .toLowerCase()
      .replace(/[\s_]+/g, '-')
      .replace(/[^a-z0-9-]/g, '');
    return new ProjectSlug(slug);
  }
}

// Step 3: Use value objects in entities
export class Project {
  private constructor(
    private readonly id: string,
    private name: string,
    private slug: ProjectSlug, // Type-safe, validated
    private budget: Money | null, // Encapsulated money logic
    private defaultRate: HourlyRate | null, // Business rules enforced
  ) {}

  calculateBudgetUtilization(totalCost: Money): number | null {
    if (!this.budget) return null;
    return totalCost.percentageOf(this.budget);
  }
}
```

**Benefits:**

- Value objects can be tested in isolation (fast, simple unit tests)
- Entities get clean, validated data automatically
- Business rules are centralized in value objects
- Compile-time type safety prevents passing raw primitives

#### 2. Shared vs. Bounded Context Value Objects

**Shared Value Objects** (`domain/shared/`):
Value objects used across multiple aggregates or bounded contexts.

```typescript
// domain/shared/money.vo.ts
export class Money {}

// domain/shared/hourly-rate.vo.ts
export class HourlyRate {}

// domain/shared/email.vo.ts
export class Email {}
```

**Used by:** Project, TimeEntry, Invoice, User, Organization

**Bounded Context Value Objects** (`domain/{aggregate}/`):
Value objects specific to a single aggregate or bounded context.

```typescript
// domain/project/project-slug.vo.ts
export class ProjectSlug {}

// domain/time-entry/duration.vo.ts
export class Duration {}

// domain/user/password-hash.vo.ts
export class PasswordHash {}
```

**Decision Criteria:**

- **Shared**: Used by 2+ aggregates, universal business concept (Money, Email)
- **Bounded**: Specific to one aggregate, specialized concept (ProjectSlug, Duration)

#### 3. Slug Pattern for Human-Readable Identifiers

**Problem:** UUIDs are database-friendly but not user-friendly in URLs:

```
❌ /projects/f47ac10b-58cc-4372-a567-0e02b2c3d479
✅ /projects/acme-website-redesign
```

**Solution:** Use Slug value object for URL-safe, human-readable identifiers:

```typescript
export class ProjectSlug {
  private readonly _value: string;

  static fromString(value: string): ProjectSlug {
    // Validates: lowercase, hyphens, 3-100 chars
    return new ProjectSlug(value);
  }

  static fromProjectName(name: string): ProjectSlug {
    // "ACME Website Redesign!" → "acme-website-redesign"
    const slug = name
      .toLowerCase()
      .trim()
      .replace(/[\s_]+/g, '-')
      .replace(/[^a-z0-9-]/g, '')
      .replace(/-+/g, '-')
      .replace(/^-+|-+$/g, '');

    return new ProjectSlug(slug);
  }

  withSuffix(suffix: number): ProjectSlug {
    // Handle uniqueness: "acme-website-2"
    return new ProjectSlug(`${this._value}-${suffix}`);
  }

  toUrlPath(): string {
    return `/projects/${this._value}`;
  }
}
```

**Database Schema:**

```sql
CREATE TABLE projects (
  id UUID PRIMARY KEY,          -- Internal ID
  slug VARCHAR(100) NOT NULL,   -- Human-readable ID
  UNIQUE (organization_id, slug) -- Unique within tenant
);
```

**Benefits:**

- Better SEO for public-facing routes
- Easier debugging (recognizable URLs in logs)
- Better UX (users can guess/remember URLs)
- Still use UUID for internal references (FK, API responses)

#### 4. State Machine Enums

Encapsulate valid state transitions in enums:

```typescript
export enum ProjectStatus {
  ACTIVE = 'ACTIVE',
  ARCHIVED = 'ARCHIVED',
}

export const VALID_TRANSITIONS: Record<ProjectStatus, ProjectStatus[]> = {
  [ProjectStatus.ACTIVE]: [ProjectStatus.ARCHIVED],
  [ProjectStatus.ARCHIVED]: [], // Terminal state
};

export function isValidTransition(from: ProjectStatus, to: ProjectStatus): boolean {
  return VALID_TRANSITIONS[from]?.includes(to) ?? false;
}

// Use in entity
export class Project {
  archive(): void {
    if (!isValidTransition(this.status, ProjectStatus.ARCHIVED)) {
      throw new Error('Invalid state transition');
    }
    this.status = ProjectStatus.ARCHIVED;
  }
}
```

#### 5. Factory Methods for Complex Creation

Use static factory methods instead of multiple constructors:

```typescript
export class TimeEntry {
  private constructor(private props: TimeEntryProps) {
    this.validateInvariants(props);
  }

  // Factory for draft entry
  static createDraft(params: {
    userId: string;
    projectId: string;
    startTime: Date;
    endTime: Date | null;
    description: string;
  }): TimeEntry {
    const duration = params.endTime
      ? Duration.fromTimeRange(params.startTime, params.endTime)
      : null;

    return new TimeEntry({
      ...params,
      status: TimeEntryStatus.DRAFT,
      duration,
      submissionCount: 0,
      // ... defaults
    });
  }

  // Factory for timer (running entry)
  static startTimer(params: { userId: string; projectId: string; description: string }): TimeEntry {
    return new TimeEntry({
      ...params,
      startTime: new Date(),
      endTime: null,
      duration: null,
      status: TimeEntryStatus.RUNNING,
      // ... defaults
    });
  }

  // Factory for persistence
  static fromPersistence(props: TimeEntryProps): TimeEntry {
    return new TimeEntry(props);
  }
}
```

**Usage:**

```typescript
// Clear intent, type-safe defaults
const draft = TimeEntry.createDraft({ ... });
const running = TimeEntry.startTimer({ ... });
```

## Anti-Patterns to Avoid

### 1. Anemic Domain Model

```typescript
// DON'T: All logic in services
class VendorService {
  activate(vendor: Vendor) {
    vendor.status = 'ACTIVE'; // Just data manipulation
  }
}
```

### 2. Leaky Abstractions

```typescript
// DON'T: Domain depending on infrastructure
class Vendor {
  constructor(private db: Database) {} // Infrastructure leak
}
```

### 3. God Aggregates

```typescript
// DON'T: One aggregate handling everything
class Organization {
  vendors: Vendor[];
  users: User[];
  leases: Lease[];
  contracts: Contract[];
  // Too many responsibilities
}
```

### 4. Technical Ubiquitous Language

```typescript
// DON'T: Technical terms in domain
class VendorEntity {
  pk: string; // Use business terms like 'id' or 'vendorId'
}
```

## Testing Domain Models

### Unit Testing Entities

```typescript
describe('Vendor', () => {
  describe('activate', () => {
    it('should activate pending vendor', () => {
      // Given
      const vendor = createVendorWithStatus(VendorStatus.PENDING);

      // When
      vendor.activate();

      // Then
      expect(vendor.isActive()).toBe(true);
      expect(vendor.getDomainEvents()).toContainEqual(expect.any(VendorActivatedEvent));
    });

    it('should not activate blocked vendor', () => {
      // Given
      const vendor = createVendorWithStatus(VendorStatus.BLOCKED);

      // When/Then
      expect(() => vendor.activate()).toThrow(VendorDomainError);
    });
  });
});
```

### Integration Testing with Repository

```typescript
describe('VendorRepository', () => {
  it('should persist vendor aggregate', async () => {
    // Given
    const vendor = Vendor.create(validVendorData);

    // When
    await vendorRepository.save(vendor);

    // Then
    const savedVendor = await vendorRepository.findById(vendor.getId());
    expect(savedVendor).toEqual(vendor);
  });
});
```

DDD provides a structured approach to building software that truly reflects the
business domain, leading to more maintainable and business-aligned code. In our
oil & gas management system, it helps us model complex business rules around
vendors, lease operating statements, and revenue distribution in a way that
domain experts can understand and validate.
