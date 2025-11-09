# Unit of Work Pattern

## Overview

The Unit of Work pattern maintains a list of objects affected by a business
transaction and coordinates writing out changes and resolving concurrency
problems. It tracks changes to objects during a business transaction and
orchestrates the writing out of changes as a single unit to ensure data
consistency and integrity.

## Core Concepts

### Transaction Boundary

Defines the scope of a business operation that must be atomic.

### Change Tracking

Monitors modifications to domain objects within the transaction.

### Commit/Rollback

Ensures all changes are either committed together or rolled back as a unit.

### Repository Coordination

Coordinates multiple repositories to work within the same transaction context.

## Benefits

- **Atomicity**: All changes succeed or fail together
- **Consistency**: Maintains data integrity across multiple entities
- **Performance**: Batches database operations to reduce round trips
- **Simplified Error Handling**: Single point for transaction management
- **Concurrency Control**: Manages optimistic/pessimistic locking
- **Clean Architecture**: Separates business logic from transaction concerns

## Implementation in Our Project

### Before: Manual Transaction Management

```typescript
@Injectable()
export class VendorActivationService {
  constructor(
    private readonly vendorRepository: VendorRepository,
    private readonly contractRepository: ContractRepository,
    private readonly auditRepository: AuditRepository,
    private readonly notificationService: NotificationService,
    @Inject('DATABASE_CONNECTION') private readonly db: Database,
  ) {}

  async activateVendor(vendorId: string): Promise<void> {
    // Manual transaction management scattered throughout the service
    const transaction = await this.db.transaction();

    try {
      // Multiple repository operations that must be atomic
      const vendor = await this.vendorRepository.findById(new VendorId(vendorId), transaction);

      if (!vendor) {
        throw new VendorNotFoundError(vendorId);
      }

      // Business logic mixed with transaction management
      vendor.activate();
      await this.vendorRepository.save(vendor, transaction);

      // Related entities must also be updated
      const contracts = await this.contractRepository.findByVendorId(vendor.getId(), transaction);

      for (const contract of contracts) {
        contract.activate();
        await this.contractRepository.save(contract, transaction);
      }

      // Audit trail
      const auditEntry = new AuditEntry('VENDOR_ACTIVATED', vendorId, 'System', new Date());
      await this.auditRepository.save(auditEntry, transaction);

      // If any operation fails, we need to manually rollback
      await transaction.commit();

      // Side effects outside transaction
      await this.notificationService.notifyVendorActivated(vendorId);
    } catch (error) {
      await transaction.rollback();
      throw error;
    }
  }

  async deactivateVendorWithContracts(vendorId: string): Promise<void> {
    // Duplicate transaction management code
    const transaction = await this.db.transaction();

    try {
      const vendor = await this.vendorRepository.findById(new VendorId(vendorId), transaction);

      if (!vendor) {
        throw new VendorNotFoundError(vendorId);
      }

      // Complex business rules
      const activeContracts = await this.contractRepository.findActiveByVendorId(
        vendor.getId(),
        transaction,
      );

      if (activeContracts.length > 0) {
        throw new VendorHasActiveContractsError(vendorId);
      }

      vendor.deactivate();
      await this.vendorRepository.save(vendor, transaction);

      // More operations...
      const auditEntry = new AuditEntry('VENDOR_DEACTIVATED', vendorId, 'System', new Date());
      await this.auditRepository.save(auditEntry, transaction);

      await transaction.commit();
    } catch (error) {
      await transaction.rollback();
      throw error;
    }
  }
}
```

### After: Unit of Work Pattern

```typescript
// Unit of Work interface
export interface IUnitOfWork {
  // Repository access
  vendorRepository: IVendorRepository;
  contractRepository: IContractRepository;
  auditRepository: IAuditRepository;
  paymentRepository: IPaymentRepository;
  losRepository: ILosRepository;

  // Transaction control
  commit(): Promise<void>;
  rollback(): Promise<void>;
  isActive(): boolean;

  // Change tracking
  registerNew<T extends Entity>(entity: T): void;
  registerDirty<T extends Entity>(entity: T): void;
  registerDeleted<T extends Entity>(entity: T): void;
  registerClean<T extends Entity>(entity: T): void;
}

// Unit of Work implementation
@Injectable()
export class UnitOfWork implements IUnitOfWork {
  private transaction: any = null;
  private isTransactionActive = false;

  // Change tracking collections
  private newObjects: Map<string, Entity> = new Map();
  private dirtyObjects: Map<string, Entity> = new Map();
  private deletedObjects: Map<string, Entity> = new Map();
  private cleanObjects: Set<string> = new Set();

  // Repository instances
  private _vendorRepository: IVendorRepository | null = null;
  private _contractRepository: IContractRepository | null = null;
  private _auditRepository: IAuditRepository | null = null;
  private _paymentRepository: IPaymentRepository | null = null;
  private _losRepository: ILosRepository | null = null;

  constructor(
    @Inject('DATABASE_CONNECTION') private readonly db: Database,
    @Inject('VENDOR_REPOSITORY')
    private readonly vendorRepoFactory: RepositoryFactory<IVendorRepository>,
    @Inject('CONTRACT_REPOSITORY')
    private readonly contractRepoFactory: RepositoryFactory<IContractRepository>,
    @Inject('AUDIT_REPOSITORY')
    private readonly auditRepoFactory: RepositoryFactory<IAuditRepository>,
    @Inject('PAYMENT_REPOSITORY')
    private readonly paymentRepoFactory: RepositoryFactory<IPaymentRepository>,
    @Inject('LOS_REPOSITORY')
    private readonly losRepoFactory: RepositoryFactory<ILosRepository>,
  ) {}

  // Lazy loading of repositories with transaction context
  get vendorRepository(): IVendorRepository {
    if (!this._vendorRepository) {
      this._vendorRepository = this.vendorRepoFactory.create(this.transaction || this.db);
    }
    return this._vendorRepository;
  }

  get contractRepository(): IContractRepository {
    if (!this._contractRepository) {
      this._contractRepository = this.contractRepoFactory.create(this.transaction || this.db);
    }
    return this._contractRepository;
  }

  get auditRepository(): IAuditRepository {
    if (!this._auditRepository) {
      this._auditRepository = this.auditRepoFactory.create(this.transaction || this.db);
    }
    return this._auditRepository;
  }

  get paymentRepository(): IPaymentRepository {
    if (!this._paymentRepository) {
      this._paymentRepository = this.paymentRepoFactory.create(this.transaction || this.db);
    }
    return this._paymentRepository;
  }

  get losRepository(): ILosRepository {
    if (!this._losRepository) {
      this._losRepository = this.losRepoFactory.create(this.transaction || this.db);
    }
    return this._losRepository;
  }

  isActive(): boolean {
    return this.isTransactionActive;
  }

  // Change tracking methods
  registerNew<T extends Entity>(entity: T): void {
    const key = this.getEntityKey(entity);

    if (this.deletedObjects.has(key)) {
      this.deletedObjects.delete(key);
      this.dirtyObjects.set(key, entity);
    } else if (!this.dirtyObjects.has(key) && !this.cleanObjects.has(key)) {
      this.newObjects.set(key, entity);
    }
  }

  registerDirty<T extends Entity>(entity: T): void {
    const key = this.getEntityKey(entity);

    if (!this.newObjects.has(key) && !this.deletedObjects.has(key)) {
      this.dirtyObjects.set(key, entity);
    }
  }

  registerDeleted<T extends Entity>(entity: T): void {
    const key = this.getEntityKey(entity);

    if (this.newObjects.has(key)) {
      this.newObjects.delete(key);
    } else {
      this.deletedObjects.set(key, entity);
      this.dirtyObjects.delete(key);
    }
  }

  registerClean<T extends Entity>(entity: T): void {
    const key = this.getEntityKey(entity);
    this.cleanObjects.add(key);
  }

  async commit(): Promise<void> {
    if (!this.isTransactionActive) {
      this.transaction = await this.db.transaction();
      this.isTransactionActive = true;
    }

    try {
      // Process changes in order: new, dirty, deleted
      await this.commitNew();
      await this.commitDirty();
      await this.commitDeleted();

      await this.transaction.commit();
      this.clearChanges();
    } catch (error) {
      await this.rollback();
      throw error;
    } finally {
      this.isTransactionActive = false;
      this.transaction = null;
    }
  }

  async rollback(): Promise<void> {
    if (this.isTransactionActive && this.transaction) {
      await this.transaction.rollback();
      this.clearChanges();
      this.isTransactionActive = false;
      this.transaction = null;
    }
  }

  private async commitNew(): Promise<void> {
    for (const [key, entity] of this.newObjects) {
      await this.getRepositoryForEntity(entity).save(entity);
    }
  }

  private async commitDirty(): Promise<void> {
    for (const [key, entity] of this.dirtyObjects) {
      await this.getRepositoryForEntity(entity).save(entity);
    }
  }

  private async commitDeleted(): Promise<void> {
    for (const [key, entity] of this.deletedObjects) {
      await this.getRepositoryForEntity(entity).delete(entity.getId());
    }
  }

  private clearChanges(): void {
    this.newObjects.clear();
    this.dirtyObjects.clear();
    this.deletedObjects.clear();
    this.cleanObjects.clear();
  }

  private getEntityKey<T extends Entity>(entity: T): string {
    return `${entity.constructor.name}:${entity.getId().getValue()}`;
  }

  private getRepositoryForEntity(entity: Entity): any {
    if (entity instanceof Vendor) {
      return this.vendorRepository;
    }
    if (entity instanceof Contract) {
      return this.contractRepository;
    }
    if (entity instanceof AuditEntry) {
      return this.auditRepository;
    }
    if (entity instanceof Payment) {
      return this.paymentRepository;
    }
    if (entity instanceof LeaseOperatingStatement) {
      return this.losRepository;
    }

    throw new Error(`No repository found for entity type: ${entity.constructor.name}`);
  }
}

// Clean application service using Unit of Work
@Injectable()
export class VendorActivationHandler implements ICommandHandler<ActivateVendorCommand> {
  constructor(
    private readonly unitOfWork: IUnitOfWork,
    private readonly eventBus: EventBus,
  ) {}

  async execute(command: ActivateVendorCommand): Promise<void> {
    // Business logic focused, no transaction management
    const vendor = await this.unitOfWork.vendorRepository.findById(new VendorId(command.vendorId));

    if (!vendor) {
      throw new VendorNotFoundError(command.vendorId);
    }

    // Domain logic
    vendor.activate();
    this.unitOfWork.registerDirty(vendor);

    // Related entities
    const contracts = await this.unitOfWork.contractRepository.findByVendorId(vendor.getId());

    for (const contract of contracts) {
      contract.activate();
      this.unitOfWork.registerDirty(contract);
    }

    // Audit trail
    const auditEntry = AuditEntry.create(
      'VENDOR_ACTIVATED',
      command.vendorId,
      command.userId,
      new Date(),
    );
    this.unitOfWork.registerNew(auditEntry);

    // Single commit for all changes
    await this.unitOfWork.commit();

    // Publish domain events after successful commit
    const events = vendor.getDomainEvents();
    for (const event of events) {
      this.eventBus.publish(event);
    }
  }
}
```

## Advanced Unit of Work Patterns

### Optimistic Concurrency Control

```typescript
export class OptimisticUnitOfWork implements IUnitOfWork {
  private entityVersions: Map<string, number> = new Map();

  registerDirty<T extends Entity>(entity: T): void {
    const key = this.getEntityKey(entity);
    const currentVersion = entity.getVersion();

    // Store the version when entity was loaded
    if (!this.entityVersions.has(key)) {
      this.entityVersions.set(key, currentVersion);
    }

    super.registerDirty(entity);
  }

  private async commitDirty(): Promise<void> {
    for (const [key, entity] of this.dirtyObjects) {
      const expectedVersion = this.entityVersions.get(key);
      const currentVersion = entity.getVersion();

      // Check for concurrent modifications
      const actualVersion = await this.getActualVersionFromDb(entity);

      if (expectedVersion !== actualVersion) {
        throw new OptimisticConcurrencyError(
          `Entity ${key} was modified by another process. Expected version: ${expectedVersion}, actual: ${actualVersion}`,
        );
      }

      // Increment version before saving
      entity.incrementVersion();
      await this.getRepositoryForEntity(entity).save(entity);
    }
  }

  private async getActualVersionFromDb(entity: Entity): Promise<number> {
    const repository = this.getRepositoryForEntity(entity);
    const current = await repository.findById(entity.getId());
    return current?.getVersion() ?? 0;
  }
}
```

### Nested Unit of Work

```typescript
@Injectable()
export class NestedUnitOfWork implements IUnitOfWork {
  private parentUnitOfWork: IUnitOfWork | null = null;
  private isNested = false;

  constructor(parent?: IUnitOfWork) {
    if (parent) {
      this.parentUnitOfWork = parent;
      this.isNested = true;
    }
  }

  async commit(): Promise<void> {
    if (this.isNested) {
      // For nested UoW, just register changes with parent
      this.mergeChangesWithParent();
    } else {
      // Root UoW commits to database
      await super.commit();
    }
  }

  async rollback(): Promise<void> {
    if (this.isNested) {
      // Clear local changes but don't affect parent
      this.clearChanges();
    } else {
      await super.rollback();
    }
  }

  private mergeChangesWithParent(): void {
    if (!this.parentUnitOfWork) return;

    // Merge new objects
    for (const [key, entity] of this.newObjects) {
      this.parentUnitOfWork.registerNew(entity);
    }

    // Merge dirty objects
    for (const [key, entity] of this.dirtyObjects) {
      this.parentUnitOfWork.registerDirty(entity);
    }

    // Merge deleted objects
    for (const [key, entity] of this.deletedObjects) {
      this.parentUnitOfWork.registerDeleted(entity);
    }

    this.clearChanges();
  }
}

// Usage of nested Unit of Work
@Injectable()
export class ComplexBusinessOperationHandler {
  constructor(private readonly unitOfWorkFactory: UnitOfWorkFactory) {}

  async execute(command: ComplexOperationCommand): Promise<void> {
    const rootUoW = this.unitOfWorkFactory.create();

    try {
      // Main operation
      await this.performMainOperation(command, rootUoW);

      // Sub-operation that might fail
      const nestedUoW = this.unitOfWorkFactory.createNested(rootUoW);
      try {
        await this.performRiskyOperation(command, nestedUoW);
        await nestedUoW.commit(); // Merge with parent
      } catch (error) {
        await nestedUoW.rollback(); // Rollback only nested changes
        // Continue with main operation
      }

      await rootUoW.commit(); // Commit all successful changes
    } catch (error) {
      await rootUoW.rollback();
      throw error;
    }
  }
}
```

## Complex Business Scenarios

### Lease Operating Statement Finalization

```typescript
@Injectable()
export class FinalizeLosHandler implements ICommandHandler<FinalizeLosCommand> {
  constructor(
    private readonly unitOfWork: IUnitOfWork,
    private readonly domainEventPublisher: IDomainEventPublisher,
  ) {}

  async execute(command: FinalizeLosCommand): Promise<void> {
    const los = await this.unitOfWork.losRepository.findById(new LosId(command.losId));

    if (!los) {
      throw new LosNotFoundError(command.losId);
    }

    // Business rule validation
    if (!los.canBeFinalized()) {
      throw new LosCannotBeFinalizedError(command.losId);
    }

    // Finalize the LOS
    los.finalize();
    this.unitOfWork.registerDirty(los);

    // Update related entities
    await this.processExpenseAllocations(los);
    await this.updateVendorPayments(los);
    await this.createAuditTrail(command, los);

    // Commit all changes atomically
    await this.unitOfWork.commit();

    // Publish events after successful commit
    await this.publishDomainEvents(los);
  }

  private async processExpenseAllocations(los: LeaseOperatingStatement): Promise<void> {
    const expenseItems = los.getExpenseLineItems();

    for (const expense of expenseItems) {
      if (expense.requiresAllocation()) {
        const allocation = ExpenseAllocation.create(
          expense.getId(),
          los.getLeaseId(),
          expense.getAmount(),
          los.getAllocationRules(),
        );

        this.unitOfWork.registerNew(allocation);
      }
    }
  }

  private async updateVendorPayments(los: LeaseOperatingStatement): Promise<void> {
    const vendorExpenses = los.getExpensesByVendor();

    for (const [vendorId, expenses] of vendorExpenses.entries()) {
      const totalAmount = Money.sum(...expenses.map((e) => e.getAmount()));

      const payment = Payment.create(vendorId, los.getId(), totalAmount, PaymentStatus.PENDING);

      this.unitOfWork.registerNew(payment);
    }
  }

  private async createAuditTrail(
    command: FinalizeLosCommand,
    los: LeaseOperatingStatement,
  ): Promise<void> {
    const auditEntry = AuditEntry.create(
      'LOS_FINALIZED',
      los.getId().getValue(),
      command.userId,
      new Date(),
      {
        losId: los.getId().getValue(),
        leaseId: los.getLeaseId(),
        totalExpenses: los.getTotalExpenses().getAmount(),
        statementMonth: los.getStatementMonth().toString(),
      },
    );

    this.unitOfWork.registerNew(auditEntry);
  }

  private async publishDomainEvents(los: LeaseOperatingStatement): Promise<void> {
    const events = los.getDomainEvents();

    for (const event of events) {
      await this.domainEventPublisher.publish(event);
    }

    los.clearDomainEvents();
  }
}
```

### Bulk Operations with Unit of Work

```typescript
@Injectable()
export class BulkVendorUpdateHandler implements ICommandHandler<BulkUpdateVendorsCommand> {
  constructor(private readonly unitOfWork: IUnitOfWork) {}

  async execute(command: BulkUpdateVendorsCommand): Promise<BulkOperationResult> {
    const results = new BulkOperationResult();

    try {
      for (const update of command.updates) {
        try {
          await this.processVendorUpdate(update);
          results.addSuccess(update.vendorId);
        } catch (error) {
          results.addFailure(update.vendorId, error.message);
          // Continue processing other vendors
        }
      }

      // Commit all successful changes
      await this.unitOfWork.commit();
    } catch (error) {
      // Rollback everything if commit fails
      await this.unitOfWork.rollback();
      throw error;
    }

    return results;
  }

  private async processVendorUpdate(update: VendorUpdate): Promise<void> {
    const vendor = await this.unitOfWork.vendorRepository.findById(new VendorId(update.vendorId));

    if (!vendor) {
      throw new VendorNotFoundError(update.vendorId);
    }

    // Apply updates
    if (update.name) {
      vendor.updateName(new VendorName(update.name));
    }

    if (update.contactInfo) {
      vendor.updateContactInfo(new ContactInfo(update.contactInfo));
    }

    if (update.insurance) {
      vendor.updateInsurance(new Insurance(update.insurance));
    }

    this.unitOfWork.registerDirty(vendor);

    // Create audit entry for each update
    const auditEntry = AuditEntry.create(
      'VENDOR_BULK_UPDATED',
      update.vendorId,
      update.userId,
      new Date(),
      { changes: update.changes },
    );

    this.unitOfWork.registerNew(auditEntry);
  }
}
```

## Testing Unit of Work

### Unit Testing

```typescript
describe('UnitOfWork', () => {
  let unitOfWork: UnitOfWork;
  let mockDb: jest.Mocked<Database>;
  let mockTransaction: jest.Mocked<Transaction>;

  beforeEach(() => {
    mockTransaction = {
      commit: jest.fn(),
      rollback: jest.fn(),
    } as any;

    mockDb = {
      transaction: jest.fn().mockResolvedValue(mockTransaction),
    } as any;

    unitOfWork = new UnitOfWork(
      mockDb,
      mockVendorRepoFactory,
      mockContractRepoFactory,
      mockAuditRepoFactory,
      mockPaymentRepoFactory,
      mockLosRepoFactory,
    );
  });

  describe('change tracking', () => {
    it('should track new entities', () => {
      const vendor = createTestVendor();

      unitOfWork.registerNew(vendor);

      expect(unitOfWork['newObjects'].has(`Vendor:${vendor.getId().getValue()}`)).toBe(true);
    });

    it('should track dirty entities', () => {
      const vendor = createTestVendor();

      unitOfWork.registerDirty(vendor);

      expect(unitOfWork['dirtyObjects'].has(`Vendor:${vendor.getId().getValue()}`)).toBe(true);
    });

    it('should handle entity state transitions', () => {
      const vendor = createTestVendor();

      // Register as new
      unitOfWork.registerNew(vendor);
      expect(unitOfWork['newObjects'].size).toBe(1);

      // Mark as deleted
      unitOfWork.registerDeleted(vendor);
      expect(unitOfWork['newObjects'].size).toBe(0);
      expect(unitOfWork['deletedObjects'].size).toBe(0); // New entity deleted = no action
    });
  });

  describe('commit', () => {
    it('should commit all changes in order', async () => {
      const vendor = createTestVendor();
      const contract = createTestContract();
      const auditEntry = createTestAuditEntry();

      unitOfWork.registerNew(vendor);
      unitOfWork.registerDirty(contract);
      unitOfWork.registerDeleted(auditEntry);

      await unitOfWork.commit();

      expect(mockTransaction.commit).toHaveBeenCalled();
      expect(unitOfWork['newObjects'].size).toBe(0);
      expect(unitOfWork['dirtyObjects'].size).toBe(0);
      expect(unitOfWork['deletedObjects'].size).toBe(0);
    });

    it('should rollback on error', async () => {
      const vendor = createTestVendor();
      unitOfWork.registerNew(vendor);

      // Mock repository save to throw error
      const mockVendorRepo = {
        save: jest.fn().mockRejectedValue(new Error('Save failed')),
      };
      jest.spyOn(unitOfWork, 'vendorRepository', 'get').mockReturnValue(mockVendorRepo as any);

      await expect(unitOfWork.commit()).rejects.toThrow('Save failed');
      expect(mockTransaction.rollback).toHaveBeenCalled();
    });
  });
});
```

### Integration Testing

```typescript
describe('UnitOfWork Integration', () => {
  let unitOfWork: UnitOfWork;
  let testDb: Database;

  beforeAll(async () => {
    testDb = await createTestDatabase();
    unitOfWork = new UnitOfWork(testDb, ...repositoryFactories);
  });

  afterEach(async () => {
    await cleanDatabase(testDb);
  });

  it('should handle complex business transaction', async () => {
    // Given
    const vendor = Vendor.create(createValidVendorData());
    const contract = Contract.create(createValidContractData(vendor.getId()));

    // When
    unitOfWork.registerNew(vendor);
    unitOfWork.registerNew(contract);
    await unitOfWork.commit();

    // Then
    const savedVendor = await unitOfWork.vendorRepository.findById(vendor.getId());
    const savedContract = await unitOfWork.contractRepository.findById(contract.getId());

    expect(savedVendor).toBeTruthy();
    expect(savedContract).toBeTruthy();
    expect(savedContract?.getVendorId()).toEqual(vendor.getId());
  });

  it('should rollback on constraint violation', async () => {
    // Given
    const vendor1 = Vendor.create({
      ...createValidVendorData(),
      code: 'DUPLICATE',
    });
    const vendor2 = Vendor.create({
      ...createValidVendorData(),
      code: 'DUPLICATE',
    });

    // When/Then
    unitOfWork.registerNew(vendor1);
    unitOfWork.registerNew(vendor2);

    await expect(unitOfWork.commit()).rejects.toThrow();

    // Verify rollback
    const foundVendor = await unitOfWork.vendorRepository.findByCode(new VendorCode('DUPLICATE'));
    expect(foundVendor).toBeNull();
  });
});
```

## Best Practices

### 1. Single Responsibility

```typescript
// Good: UoW focuses only on transaction management
export class UnitOfWork implements IUnitOfWork {
  async commit(): Promise<void> {
    // Only transaction and change tracking logic
  }
}

// Keep business logic in handlers
export class BusinessOperationHandler {
  constructor(private readonly unitOfWork: IUnitOfWork) {}

  async execute(command: Command): Promise<void> {
    // Business logic here
    // Use UoW for transaction management
  }
}

// Avoid: Business logic mixed with UoW
export class UnitOfWork {
  async commitVendorActivation(vendorId: string): Promise<void> {
    // Don't put business logic here
    const vendor = await this.vendorRepository.findById(vendorId);
    vendor.activate(); // Business logic doesn't belong here
    await this.commit();
  }
}
```

### 2. Proper Error Handling

```typescript
export class UnitOfWork {
  async commit(): Promise<void> {
    if (!this.isTransactionActive) {
      this.transaction = await this.db.transaction();
      this.isTransactionActive = true;
    }

    try {
      await this.commitChanges();
      await this.transaction.commit();
      this.clearChanges();
    } catch (error) {
      await this.rollback();
      throw new UnitOfWorkError('Failed to commit transaction', error);
    } finally {
      this.isTransactionActive = false;
      this.transaction = null;
    }
  }
}
```

### 3. Resource Management

```typescript
export class UnitOfWork implements IDisposable {
  private disposed = false;

  async dispose(): Promise<void> {
    if (this.disposed) return;

    if (this.isTransactionActive) {
      await this.rollback();
    }

    this.clearChanges();
    this.disposed = true;
  }

  private ensureNotDisposed(): void {
    if (this.disposed) {
      throw new Error('Unit of Work has been disposed');
    }
  }
}

// Usage with proper cleanup
export class Handler {
  async execute(command: Command): Promise<void> {
    const unitOfWork = this.unitOfWorkFactory.create();

    try {
      await this.performBusinessOperation(unitOfWork);
      await unitOfWork.commit();
    } finally {
      await unitOfWork.dispose();
    }
  }
}
```

The Unit of Work pattern in our oil & gas management system ensures that complex
business operations involving multiple entities are handled atomically,
maintaining data consistency and simplifying error handling across repository
operations.
