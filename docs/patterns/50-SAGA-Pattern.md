# Pattern 50: SAGA Pattern

**Version**: 1.0
**Last Updated**: October 8, 2025
**Category**: Architecture & Transactions

---

## Table of Contents

1. [Overview](#overview)
2. [When to Use](#when-to-use)
3. [SAGA Types](#saga-types)
4. [Choreography-Based SAGA](#choreography-based-saga)
5. [Orchestration-Based SAGA](#orchestration-based-saga)
6. [Compensation](#compensation)
7. [State Management](#state-management)
8. [Rust Implementation](#rust-implementation)
9. [Error Handling](#error-handling)
10. [Testing SAGAs](#testing-sagas)
11. [Best Practices](#best-practices)
12. [Anti-Patterns](#anti-patterns)
13. [Related Patterns](#related-patterns)
14. [References](#references)

---

## Overview

**SAGA Pattern** is a design pattern for managing distributed transactions across multiple services. Instead of a single ACID transaction, a SAGA is a sequence of local transactions where each transaction updates data within a single service.

**Problem**: Traditional ACID transactions don't work across microservices:

```rust
// ❌ Can't do this across services
// BEGIN TRANSACTION
//   INSERT INTO invoices ...          // Invoice Service
//   UPDATE time_entries ...           // Time Tracking Service
//   POST to QuickBooks API            // QuickBooks Service
// COMMIT
```

**SAGA Solution**: Chain of local transactions with compensating actions:

```rust
// ✅ SAGA with compensation
// 1. Create invoice (Invoice Service)
//    ↓ Success
// 2. Mark time entries as billed (Time Service)
//    ↓ Success
// 3. Sync to QuickBooks (Integration Service)
//    ↓ Failure!
// 4. Compensate: Unmark time entries (Time Service)
// 5. Compensate: Delete invoice (Invoice Service)
```

**Use Cases in WellOS**:

- **Invoice generation** - Create invoice → Mark time as billed → Sync to QuickBooks
- **Project creation** - Create project → Create default tasks → Send welcome email
- **User registration** - Create user → Send email → Create trial organization
- **Payment processing** - Charge card → Update invoice → Send receipt

---

## When to Use

### ✅ Use SAGA When:

1. **Multi-service transactions** - Operation spans multiple microservices
   - Invoice creation + QuickBooks sync
   - Order placement + inventory update + payment processing

2. **Long-lived transactions** - Operations take seconds/minutes
   - File processing workflows
   - Report generation
   - External API integrations

3. **Eventual consistency is acceptable** - Not critical to be instantly consistent
   - Invoice appears before QuickBooks sync completes
   - User registration before welcome email sends

### ❌ Don't Use SAGA When:

1. **Single service transactions** - Use local ACID transactions
2. **Strict consistency required** - Use distributed locks or 2PC
3. **No compensating actions possible** - Some operations can't be undone

---

## SAGA Types

### 1. Choreography-Based SAGA

Services communicate via events. No central coordinator.

**Flow**:

```
InvoiceCreated → TimeEntryService marks time as billed → TimeBilled event
                                                        ↓
                                            QuickBooksService syncs invoice
```

**Pros**:

- ✅ Loose coupling
- ✅ Simple for small SAGAs
- ✅ No single point of failure

**Cons**:

- ❌ Hard to understand flow
- ❌ Difficult to add new steps
- ❌ Cyclic dependencies risk

---

### 2. Orchestration-Based SAGA

Central orchestrator coordinates the SAGA steps.

**Flow**:

```
InvoiceSagaOrchestrator
  ├─ Step 1: Create invoice
  ├─ Step 2: Mark time as billed
  └─ Step 3: Sync to QuickBooks
```

**Pros**:

- ✅ Clear business logic
- ✅ Easy to add/remove steps
- ✅ Centralized monitoring

**Cons**:

- ❌ Orchestrator is single point of failure
- ❌ Tighter coupling

**Recommendation**: Use **Orchestration** for complex SAGAs (>3 steps) in WellOS.

---

## Choreography-Based SAGA

### Event-Driven Invoice Creation

```typescript
// Event-driven invoice creation
export class CreateInvoiceHandler {
  constructor(
    private readonly invoiceRepository: InvoiceRepository,
    private readonly eventBus: EventBus,
  ) {}

  async execute(command: CreateInvoiceCommand) {
    const { organizationId, timeEntryIds, dueDate } = command;

    // Step 1: Create invoice locally
    const invoice = await this.invoiceRepository.create({
      organizationId,
      timeEntryIds,
      dueDate,
      status: 'draft',
    });

    // Publish event (other services react)
    this.eventBus.publish(new InvoiceCreatedEvent(invoice.id, organizationId, timeEntryIds));

    return invoice;
  }
}
```

```typescript
// Time entry event handler
export class InvoiceCreatedHandler {
  constructor(
    private readonly timeEntryRepository: TimeEntryRepository,
    private readonly eventBus: EventBus,
  ) {}

  async handle(event: InvoiceCreatedEvent) {
    try {
      // Step 2: Mark time entries as billed
      await this.timeEntryRepository.markAsBilled(event.timeEntryIds, event.invoiceId);

      // Publish success event
      this.eventBus.publish(new TimeEntriesBilledEvent(event.invoiceId, event.timeEntryIds));
    } catch (error) {
      // Publish failure event to trigger compensation
      this.eventBus.publish(new TimeEntryBillingFailedEvent(event.invoiceId, error.message));
    }
  }
}
```

```typescript
// QuickBooks integration event handler
export class TimeEntriesBilledHandler {
  constructor(private readonly quickbooksService: QuickBooksService) {}

  async handle(event: TimeEntriesBilledEvent) {
    try {
      // Step 3: Sync invoice to QuickBooks
      await this.quickbooksService.syncInvoice(event.invoiceId);
    } catch (error) {
      // Publish failure event to trigger compensation
      this.eventBus.publish(new QuickBooksSyncFailedEvent(event.invoiceId, error.message));
    }
  }
}
```

---

## Orchestration-Based SAGA

### SAGA Orchestrator

```typescript
// SAGA orchestrator for invoice creation
export interface CreateInvoiceSagaContext extends SagaContext {
  organizationId: string;
  timeEntryIds: string[];
  dueDate: Date;
  invoiceId?: string;
}

export class CreateInvoiceSaga {
  constructor(
    private readonly invoiceRepository: InvoiceRepository,
    private readonly timeEntryRepository: TimeEntryRepository,
    private readonly quickbooksService: QuickBooksService,
  ) {}

  private steps: SagaStep<CreateInvoiceSagaContext>[] = [
    {
      name: 'create-invoice',
      execute: this.createInvoice.bind(this),
      compensate: this.deleteInvoice.bind(this),
    },
    {
      name: 'mark-time-as-billed',
      execute: this.markTimeAsBilled.bind(this),
      compensate: this.unmarkTimeAsBilled.bind(this),
    },
    {
      name: 'sync-to-quickbooks',
      execute: this.syncToQuickBooks.bind(this),
      compensate: this.deleteFromQuickBooks.bind(this),
    },
  ];

  async execute(context: CreateInvoiceSagaContext): Promise<void> {
    const completedSteps: number[] = [];

    try {
      // Execute each step
      for (let i = 0; i < this.steps.length; i++) {
        const step = this.steps[i];
        console.log(`Executing step: ${step.name}`);

        await step.execute(context);
        completedSteps.push(i);
      }

      console.log('SAGA completed successfully');
    } catch (error) {
      console.error('SAGA failed, compensating...', error);

      // Compensate in reverse order
      for (let i = completedSteps.length - 1; i >= 0; i--) {
        const stepIndex = completedSteps[i];
        const step = this.steps[stepIndex];

        try {
          console.log(`Compensating step: ${step.name}`);
          await step.compensate(context);
        } catch (compensationError) {
          console.error(`Compensation failed for step: ${step.name}`, compensationError);
          // Log to monitoring system, manual intervention may be needed
        }
      }

      throw error;
    }
  }

  // Step 1: Create invoice
  private async createInvoice(context: CreateInvoiceSagaContext): Promise<void> {
    const invoice = await this.invoiceRepository.create({
      organizationId: context.organizationId,
      timeEntryIds: context.timeEntryIds,
      dueDate: context.dueDate,
      status: 'draft',
    });

    context.invoiceId = invoice.id;
  }

  // Compensate Step 1: Delete invoice
  private async deleteInvoice(context: CreateInvoiceSagaContext): Promise<void> {
    if (context.invoiceId) {
      await this.invoiceRepository.delete(context.invoiceId);
    }
  }

  // Step 2: Mark time entries as billed
  private async markTimeAsBilled(context: CreateInvoiceSagaContext): Promise<void> {
    await this.timeEntryRepository.markAsBilled(context.timeEntryIds, context.invoiceId!);
  }

  // Compensate Step 2: Unmark time entries
  private async unmarkTimeAsBilled(context: CreateInvoiceSagaContext): Promise<void> {
    await this.timeEntryRepository.unmarkAsBilled(context.timeEntryIds);
  }

  // Step 3: Sync to QuickBooks
  private async syncToQuickBooks(context: CreateInvoiceSagaContext): Promise<void> {
    await this.quickbooksService.syncInvoice(context.invoiceId!);
  }

  // Compensate Step 3: Delete from QuickBooks
  private async deleteFromQuickBooks(context: CreateInvoiceSagaContext): Promise<void> {
    await this.quickbooksService.deleteInvoice(context.invoiceId!);
  }
}
```

### SAGA Types

```typescript
// apps/api/src/application/shared/sagas/saga.types.ts
export interface SagaContext {
  sagaId?: string;
  correlationId?: string;
  [key: string]: any;
}

export interface SagaStep<T extends SagaContext> {
  name: string;
  execute: (context: T) => Promise<void>;
  compensate: (context: T) => Promise<void>;
}

export enum SagaStatus {
  PENDING = 'pending',
  IN_PROGRESS = 'in_progress',
  COMPLETED = 'completed',
  COMPENSATING = 'compensating',
  FAILED = 'failed',
}
```

---

## Compensation

**Compensation** is the inverse operation that undoes a step's effects.

### Compensation Strategies

#### 1. Semantic Compensation

Logically undo the operation (doesn't restore exact state).

```typescript
// Forward: Create invoice
await invoiceRepository.create(invoice);

// Compensate: Mark invoice as cancelled (not delete)
await invoiceRepository.update(invoice.id, { status: 'cancelled' });
```

#### 2. Backward Recovery

Restore exact previous state.

```typescript
// Forward: Update time entry
const previousState = await timeEntryRepository.findById(id);
await timeEntryRepository.update(id, { billable: true });

// Compensate: Restore previous state
await timeEntryRepository.update(id, previousState);
```

#### 3. No Compensation

Some operations can't be compensated (e.g., sending email).

```typescript
// Forward: Send email
await emailService.send(email);

// Compensate: Send "Ignore previous email" message
await emailService.send({
  to: email.to,
  subject: 'Please disregard previous message',
  body: '...',
});
```

---

## State Management

### Persist SAGA State

```sql
-- SAGA state persistence schema
CREATE TABLE saga_state (
  saga_id VARCHAR(255) PRIMARY KEY,
  saga_type VARCHAR(100) NOT NULL,
  status VARCHAR(50) NOT NULL,
  context JSONB NOT NULL,
  current_step VARCHAR(100),
  completed_steps JSONB NOT NULL DEFAULT '[]',
  error TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_saga_state_status ON saga_state(status);
CREATE INDEX idx_saga_state_type ON saga_state(saga_type);
```

### SAGA State Manager

```typescript
// SAGA state manager
export class SagaStateManager {
  constructor(private readonly db: DatabaseConnection) {}

  async create(sagaType: string, context: SagaContext): Promise<string> {
    const sagaId = uuidv4();

    await this.db.query(
      `INSERT INTO saga_state (saga_id, saga_type, status, context, completed_steps)
       VALUES ($1, $2, $3, $4, $5)`,
      [sagaId, sagaType, SagaStatus.PENDING, JSON.stringify(context), JSON.stringify([])]
    );

    return sagaId;
  }

  async updateStatus(sagaId: string, status: SagaStatus): Promise<void> {
    await this.db.query(
      `UPDATE saga_state SET status = $1, updated_at = NOW() WHERE saga_id = $2`,
      [status, sagaId]
    );
  }

  async addCompletedStep(sagaId: string, stepName: string): Promise<void> {
    await this.db.query(
      `UPDATE saga_state
       SET current_step = $1,
           completed_steps = jsonb_insert(completed_steps, '{-1}', $2),
           updated_at = NOW()
       WHERE saga_id = $3`,
      [stepName, JSON.stringify(stepName), sagaId]
    );
  }

  async recordError(sagaId: string, error: string): Promise<void> {
    await this.db.query(
      `UPDATE saga_state
       SET status = $1, error = $2, updated_at = NOW()
       WHERE saga_id = $3`,
      [SagaStatus.FAILED, error, sagaId]
    );
  }

  async get(sagaId: string): Promise<any> {
    const result = await this.db.query(
      `SELECT * FROM saga_state WHERE saga_id = $1`,
      [sagaId]
    );

    return result.rows[0];
  }
}
```

---

## Implementation Example

### SAGA Command Handler

```typescript
// Command handler for invoice creation with sync
export class CreateInvoiceWithSyncHandler {
  constructor(
    private readonly saga: CreateInvoiceSaga,
    private readonly stateManager: SagaStateManager,
  ) {}

  async execute(command: CreateInvoiceWithSyncCommand) {
    const context: CreateInvoiceSagaContext = {
      organizationId: command.organizationId,
      timeEntryIds: command.timeEntryIds,
      dueDate: command.dueDate,
    };

    // Create SAGA state
    const sagaId = await this.stateManager.create('create-invoice', context);
    context.sagaId = sagaId;

    try {
      await this.stateManager.updateStatus(sagaId, SagaStatus.IN_PROGRESS);

      // Execute SAGA
      await this.saga.execute(context);

      await this.stateManager.updateStatus(sagaId, SagaStatus.COMPLETED);

      return { invoiceId: context.invoiceId, sagaId };
    } catch (error) {
      await this.stateManager.recordError(sagaId, error.message);
      throw error;
    }
  }
}
```

---

## Error Handling

### Retry with Exponential Backoff

```typescript
// SAGA step executor with retry logic
export class SagaStepExecutor {
  async executeWithRetry<T extends SagaContext>(
    step: SagaStep<T>,
    context: T,
    maxRetries: number = 3,
  ): Promise<void> {
    let lastError: Error;

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        await step.execute(context);
        return; // Success
      } catch (error) {
        lastError = error;

        if (attempt < maxRetries) {
          const delay = Math.pow(2, attempt) * 1000; // 1s, 2s, 4s
          console.log(`Step ${step.name} failed, retrying in ${delay}ms...`);
          await this.sleep(delay);
        }
      }
    }

    throw new Error(
      `Step ${step.name} failed after ${maxRetries + 1} attempts: ${lastError.message}`,
    );
  }

  private sleep(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}
```

### Idempotent Steps

Ensure steps can be safely retried.

```typescript
// Time entry repository with idempotent operations
export interface TimeEntryRepository {
  // Idempotent: Multiple calls with same invoiceId have no additional effect
  async markAsBilled(timeEntryIds: string[], invoiceId: string): Promise<void> {
    await this.db.query(
      `UPDATE time_entries
       SET billable = true, invoice_id = $1, billed_at = NOW()
       WHERE id = ANY($2)
       AND (invoice_id IS NULL OR invoice_id != $1)`,
      [invoiceId, timeEntryIds]
    );
  }
}
```

---

## Testing SAGAs

### Unit Test

```typescript
// SAGA unit tests
describe('CreateInvoiceSaga', () => {
  let saga: CreateInvoiceSaga;
  let invoiceRepository: MockInvoiceRepository;
  let timeEntryRepository: MockTimeEntryRepository;
  let quickbooksService: MockQuickBooksService;

  beforeEach(() => {
    invoiceRepository = {
      create: jest.fn(),
      delete: jest.fn(),
    };

    timeEntryRepository = {
      markAsBilled: jest.fn(),
      unmarkAsBilled: jest.fn(),
    };

    quickbooksService = {
      syncInvoice: jest.fn(),
      deleteInvoice: jest.fn(),
    };

    saga = new CreateInvoiceSaga(
      invoiceRepository,
      timeEntryRepository,
      quickbooksService
    );
  });

  it('should execute all steps successfully', async () => {
    invoiceRepository.create.mockResolvedValue({ id: 'invoice-123' });

    const context = {
      organizationId: 'org-1',
      timeEntryIds: ['te-1', 'te-2'],
      dueDate: new Date('2025-10-31'),
    };

    await saga.execute(context);

    expect(invoiceRepository.create).toHaveBeenCalled();
    expect(timeEntryRepository.markAsBilled).toHaveBeenCalledWith(['te-1', 'te-2'], 'invoice-123');
    expect(quickbooksService.syncInvoice).toHaveBeenCalledWith('invoice-123');
  });

  it('should compensate when QuickBooks sync fails', async () => {
    invoiceRepository.create.mockResolvedValue({ id: 'invoice-123' });
    quickbooksService.syncInvoice.mockRejectedValue(new Error('QuickBooks API error'));

    const context = {
      organizationId: 'org-1',
      timeEntryIds: ['te-1', 'te-2'],
      dueDate: new Date('2025-10-31'),
    };

    await expect(saga.execute(context)).rejects.toThrow('QuickBooks API error');

    // Verify compensation (in reverse order)
    expect(timeEntryRepository.unmarkAsBilled).toHaveBeenCalledWith(['te-1', 'te-2']);
    expect(invoiceRepository.delete).toHaveBeenCalledWith('invoice-123');
  });
});
```

---

## Best Practices

### ✅ DO

1. **Make steps idempotent** - Safe to retry

   ```typescript
   // Check if already done before executing
   if (await alreadyExecuted(step)) return;
   ```

2. **Persist SAGA state** - Survive crashes

   ```typescript
   await sagaStateManager.create(sagaType, context);
   ```

3. **Use timeouts** - Don't wait forever

   ```typescript
   await Promise.race([
     step.execute(context),
     timeout(30000), // 30 seconds
   ]);
   ```

4. **Log every step** - Observability

   ```typescript
   logger.info('Executing step', { sagaId, step: step.name });
   ```

5. **Design compensations carefully** - Test rollback scenarios
   ```typescript
   // Compensate: Don't just delete, mark as cancelled
   ```

---

### ❌ DON'T

1. **Don't use SAGAs for single-service transactions** - Use local ACID
2. **Don't assume all steps will succeed** - Always plan for failures
3. **Don't ignore compensation failures** - Log and alert for manual intervention
4. **Don't make compensations complex** - Keep them simple and reliable

---

## Anti-Patterns

### 1. God SAGA

```typescript
// ❌ Anti-pattern: Too many steps in one SAGA
const steps = [
  createInvoice,
  markTimeAsBilled,
  syncToQuickBooks,
  sendEmail,
  updateAnalytics,
  notifySlack,
  archiveOldInvoices,
  generateReport,
  // 20 more steps...
];

// ✅ Solution: Break into multiple SAGAs
// SAGA 1: Invoice creation + QuickBooks sync
// SAGA 2: Notifications (email, Slack)
// SAGA 3: Analytics and reporting
```

### 2. Missing Compensation

```typescript
// ❌ Anti-pattern: No compensation logic
compensate: async (context) => {
  // TODO: Implement compensation
};

// ✅ Solution: Always implement compensation
compensate: async (context) => {
  await invoiceRepository.delete(context.invoiceId);
};
```

---

## Related Patterns

- **Pattern 05: CQRS Pattern** - Commands initiate SAGAs
- **Pattern 11: Domain Events Pattern** - Choreography-based SAGAs use events
- **Pattern 45: Background Job Patterns** - Long-running SAGA steps as jobs
- **Pattern 49: Event Sourcing Pattern** - SAGA state as event stream

---

## References

### Books & Articles

- **"Microservices Patterns"** by Chris Richardson - SAGA pattern chapter
- **"Designing Data-Intensive Applications"** by Martin Kleppmann - Distributed transactions
- **"Enterprise Integration Patterns"** - Process Manager pattern

### Libraries

- **Camunda** - Workflow and SAGA orchestration
- **Temporal** - Durable execution for SAGAs
- **NestJS CQRS** - Event-driven SAGAs

---

## Summary

**SAGA Pattern** manages distributed transactions across services:

✅ **Sequence of local transactions** - Each service commits locally
✅ **Compensation for failures** - Undo completed steps if later steps fail
✅ **Orchestration vs Choreography** - Centralized vs event-driven coordination
✅ **Idempotent steps** - Safe to retry
✅ **Persistent state** - Survive crashes and resume
✅ **Timeouts and retries** - Handle transient failures

**Remember**: SAGAs are complex. Only use for multi-service transactions. For single-service, use local ACID transactions.

---

**Next Steps**:

1. Identify multi-service transactions (invoice creation, payment processing)
2. Choose orchestration vs choreography
3. Design compensation logic for each step
4. Implement SAGA state persistence
5. Add retry logic with exponential backoff
6. Test failure scenarios and compensation
