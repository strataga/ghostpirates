# SOLID Principles in Oil & Gas Software

## Overview

SOLID is a set of five design principles that make software designs more
understandable, flexible, and maintainable. In oil & gas operations, where
regulations change frequently and systems must integrate with various state
agencies and equipment vendors, SOLID principles are crucial.

## The Five Principles

### 1. Single Responsibility Principle (SRP)

_A class should have only one reason to change_

### 2. Open/Closed Principle (OCP)

_Software entities should be open for extension but closed for modification_

### 3. Liskov Substitution Principle (LSP)

_Objects of a superclass should be replaceable with objects of its subclasses_

### 4. Interface Segregation Principle (ISP)

_Clients should not be forced to depend on interfaces they don't use_

### 5. Dependency Inversion Principle (DIP)

_High-level modules should not depend on low-level modules; both should depend
on abstractions_

## Before Implementation (Violating SOLID)

```typescript
// ❌ POOR: Violating all SOLID principles

// 1. SRP Violation: WellService does everything
class WellService {
  constructor(
    private db: PostgresDatabase,
    private emailService: EmailService,
  ) {}

  // Database operations
  async createWell(data: any) {
    // Validation logic mixed with business logic
    if (!data.apiNumber || data.apiNumber.length !== 14) {
      throw new Error('Invalid API number');
    }

    // Direct database access
    const result = await this.db.query('INSERT INTO wells VALUES ($1, $2, $3)', [
      data.apiNumber,
      data.name,
      data.operator,
    ]);

    // Email notification mixed in
    await this.emailService.send({
      to: 'regulatory@state.gov',
      subject: 'New Well Created',
      body: `Well ${data.name} created`,
    });

    // PDF generation mixed in
    const pdf = await this.generateWellReport(result.id);

    // State reporting mixed in
    await this.submitToTexasRRC(result.id);

    return result;
  }

  // Production calculations mixed with service
  async calculateRoyalties(wellId: string, month: string) {
    const production = await this.db.query(
      'SELECT * FROM production WHERE well_id = $1 AND month = $2',
      [wellId, month],
    );

    const lease = await this.db.query('SELECT * FROM leases WHERE well_id = $1', [wellId]);

    // Complex Texas-specific calculation logic
    let royalty = 0;
    if (lease.state === 'TX') {
      royalty = production.oil * 0.125 * lease.royaltyRate;
      // Texas severance tax
      royalty -= production.oil * 0.046;
    } else if (lease.state === 'OK') {
      royalty = production.oil * 0.125 * lease.royaltyRate;
      // Oklahoma gross production tax
      royalty -= production.oil * 0.05;
    }

    return royalty;
  }

  // Report generation mixed in
  async generateWellReport(wellId: string) {
    // PDF generation logic
  }

  // External API calls mixed in
  async submitToTexasRRC(wellId: string) {
    // Texas Railroad Commission submission
  }
}

// 2. OCP Violation: Adding new states requires modifying existing code
class ProductionTaxCalculator {
  calculateTax(state: string, production: number): number {
    // Must modify this method for each new state
    switch (state) {
      case 'TX':
        return production * 0.046;
      case 'OK':
        return production * 0.05;
      case 'NM':
        return production * 0.038;
      // Adding new state requires changing this method
      default:
        return 0;
    }
  }
}

// 3. LSP Violation: Subclass breaks parent contract
class Well {
  status: string;

  updateStatus(newStatus: string) {
    this.status = newStatus;
  }
}

class PluggedWell extends Well {
  // Violates LSP - throws error instead of updating
  updateStatus(newStatus: string) {
    throw new Error('Cannot update status of plugged well');
  }
}

// 4. ISP Violation: Massive interface forcing unnecessary implementations
interface IWellOperations {
  drill(): void;
  complete(): void;
  produce(): void;
  workover(): void;
  plug(): void;
  abandon(): void;
  submitToTexasRRC(): void;
  submitToOklahomaOCC(): void;
  submitToNewMexicoOCD(): void;
  calculateRoyalties(): void;
  distributeFunds(): void;
  generateAFE(): void;
  approveAFE(): void;
}

// Poor pumper class forced to implement everything
class Pumper implements IWellOperations {
  drill() {
    throw new Error('Pumpers dont drill');
  }
  complete() {
    throw new Error('Pumpers dont complete');
  }
  produce() {
    /* actual implementation */
  }
  workover() {
    throw new Error('Pumpers dont workover');
  }
  // ... forced to implement 10+ methods they don't use
}

// 5. DIP Violation: High-level module depends on concrete implementation
class ComplianceReportService {
  private database: PostgresDatabase; // Depends on concrete class

  constructor() {
    this.database = new PostgresDatabase(); // Creating concrete instance
  }

  async submitReport(report: any) {
    // Directly using concrete database
    await this.database.query('INSERT INTO reports...');
  }
}
```

## After Implementation (Following SOLID)

```typescript
// ✅ GOOD: Following all SOLID principles

// 1. SRP: Each class has a single responsibility

// Domain entity only handles well business rules
class Well {
  constructor(
    private id: string,
    private apiNumber: ApiNumber,
    private status: WellStatus,
    private location: Location,
  ) {}

  // Only well-specific business logic
  canTransitionTo(newStatus: WellStatus): boolean {
    return this.status.canTransitionTo(newStatus);
  }

  updateStatus(newStatus: WellStatus): DomainEvent[] {
    if (!this.canTransitionTo(newStatus)) {
      throw new InvalidStatusTransitionError(this.status, newStatus);
    }

    const oldStatus = this.status;
    this.status = newStatus;

    return [new WellStatusChangedEvent(this.id, oldStatus, newStatus)];
  }
}

// Repository only handles data access
interface IWellRepository {
  save(well: Well): Promise<void>;
  findById(id: string): Promise<Well>;
  findByApiNumber(apiNumber: ApiNumber): Promise<Well>;
}

class WellRepository implements IWellRepository {
  constructor(private db: IDatabase) {}

  async save(well: Well): Promise<void> {
    await this.db.wells.upsert(well.toPersistence());
  }

  async findById(id: string): Promise<Well> {
    const data = await this.db.wells.findOne({ id });
    return Well.fromPersistence(data);
  }

  async findByApiNumber(apiNumber: ApiNumber): Promise<Well> {
    const data = await this.db.wells.findOne({
      apiNumber: apiNumber.getValue(),
    });
    return Well.fromPersistence(data);
  }
}

// Notification service only handles notifications
interface INotificationService {
  notify(event: DomainEvent): Promise<void>;
}

class NotificationService implements INotificationService {
  constructor(
    private emailAdapter: IEmailAdapter,
    private smsAdapter: ISmsAdapter,
  ) {}

  async notify(event: DomainEvent): Promise<void> {
    const subscribers = await this.getSubscribers(event);

    for (const subscriber of subscribers) {
      if (subscriber.preferEmail) {
        await this.emailAdapter.send(this.buildEmail(event, subscriber));
      }
      if (subscriber.preferSms) {
        await this.smsAdapter.send(this.buildSms(event, subscriber));
      }
    }
  }

  private getSubscribers(event: DomainEvent): Promise<Subscriber[]> {
    // Get subscribers for this event type
  }

  private buildEmail(event: DomainEvent, subscriber: Subscriber): Email {
    // Build email from event
  }

  private buildSms(event: DomainEvent, subscriber: Subscriber): Sms {
    // Build SMS from event
  }
}

// 2. OCP: Open for extension, closed for modification

// Tax calculation using strategy pattern
interface ITaxStrategy {
  calculateTax(production: Production): Money;
}

// Base implementations
class TexasTaxStrategy implements ITaxStrategy {
  calculateTax(production: Production): Money {
    const severanceTax = production.oilVolume.multiply(0.046);
    const regulatoryFee = production.gasVolume.multiply(0.00667);
    return severanceTax.add(regulatoryFee);
  }
}

class OklahomaTaxStrategy implements ITaxStrategy {
  calculateTax(production: Production): Money {
    return production.totalValue.multiply(0.05);
  }
}

// Easy to add new state without modifying existing code
class NewMexicoTaxStrategy implements ITaxStrategy {
  calculateTax(production: Production): Money {
    const oilTax = production.oilVolume.multiply(0.038);
    const gasTax = production.gasVolume.multiply(0.04);
    return oilTax.add(gasTax);
  }
}

// Tax calculator doesn't need modification for new states
class TaxCalculator {
  private strategies = new Map<State, ITaxStrategy>();

  registerStrategy(state: State, strategy: ITaxStrategy) {
    this.strategies.set(state, strategy);
  }

  calculateTax(state: State, production: Production): Money {
    const strategy = this.strategies.get(state);
    if (!strategy) {
      throw new UnsupportedStateError(state);
    }
    return strategy.calculateTax(production);
  }
}

// 3. LSP: Subclasses properly extend parent behavior

abstract class WellState {
  abstract canTransitionTo(newState: WellState): boolean;
  abstract getAllowedOperations(): Operation[];
}

class ProducingWellState extends WellState {
  canTransitionTo(newState: WellState): boolean {
    return newState instanceof ShutInWellState || newState instanceof WorkoverWellState;
  }

  getAllowedOperations(): Operation[] {
    return [Operation.UPDATE_PRODUCTION, Operation.REPORT_ISSUES];
  }
}

class PluggedWellState extends WellState {
  canTransitionTo(newState: WellState): boolean {
    return false; // Plugged is terminal state
  }

  getAllowedOperations(): Operation[] {
    return [Operation.VIEW_HISTORY]; // Limited but valid operations
  }
}

// 4. ISP: Segregated interfaces for different roles

// Separate interfaces for different operations
interface IProductionOperations {
  recordProduction(data: ProductionData): Promise<void>;
  viewProduction(wellId: string, period: DateRange): Promise<Production[]>;
}

interface IDrillingOperations {
  spudWell(wellId: string): Promise<void>;
  updateDrillingProgress(wellId: string, depth: Depth): Promise<void>;
  reachTotalDepth(wellId: string): Promise<void>;
}

interface IComplianceOperations {
  submitFormPR(wellId: string, data: FormPRData): Promise<void>;
  submitW10(wellId: string, data: W10Data): Promise<void>;
}

interface IFinancialOperations {
  calculateRoyalties(wellId: string, period: Period): Promise<Money>;
  distributeRevenue(wellId: string, revenue: Money): Promise<void>;
}

// Pumper only implements what they need
class PumperService implements IProductionOperations {
  async recordProduction(data: ProductionData): Promise<void> {
    // Pumper-specific production recording
  }

  async viewProduction(wellId: string, period: DateRange): Promise<Production[]> {
    // View production history
  }
}

// Drilling contractor implements drilling operations
class DrillingService implements IDrillingOperations {
  async spudWell(wellId: string): Promise<void> {
    // Start drilling
  }

  async updateDrillingProgress(wellId: string, depth: Depth): Promise<void> {
    // Update depth
  }

  async reachTotalDepth(wellId: string): Promise<void> {
    // Complete drilling
  }
}

// 5. DIP: Depend on abstractions, not concretions

// Define abstractions
interface IDatabase {
  wells: IWellCollection;
  production: IProductionCollection;
}

interface IEventBus {
  publish(event: DomainEvent): Promise<void>;
  subscribe(eventType: string, handler: EventHandler): void;
}

interface IStateReportingService {
  submitReport(report: ComplianceReport): Promise<SubmissionResult>;
}

// High-level module depends on abstractions
class ComplianceReportingService {
  constructor(
    private database: IDatabase, // Depends on interface
    private eventBus: IEventBus, // Depends on interface
    private reportingServices: Map<State, IStateReportingService>, // Abstractions
  ) {}

  async submitMonthlyReport(wellId: string, month: Month): Promise<void> {
    // Get well and production data
    const well = await this.database.wells.findById(wellId);
    const production = await this.database.production.findByWellAndMonth(wellId, month);

    // Create report
    const report = new ComplianceReport(well, production, month);

    // Get appropriate reporting service
    const reportingService = this.reportingServices.get(well.location.state);
    if (!reportingService) {
      throw new UnsupportedStateError(well.location.state);
    }

    // Submit report
    const result = await reportingService.submitReport(report);

    // Publish event
    await this.eventBus.publish(new ComplianceReportSubmittedEvent(wellId, month, result));
  }
}

// Concrete implementations injected at runtime
class TexasRRCReportingService implements IStateReportingService {
  async submitReport(report: ComplianceReport): Promise<SubmissionResult> {
    // Texas-specific submission logic
  }
}

class OklahomaCCReportingService implements IStateReportingService {
  async submitReport(report: ComplianceReport): Promise<SubmissionResult> {
    // Oklahoma-specific submission logic
  }
}

// Dependency injection container
class DIContainer {
  register() {
    // Register abstractions with concrete implementations
    container.bind<IDatabase>(TYPES.Database).to(PostgresDatabase);
    container.bind<IEventBus>(TYPES.EventBus).to(RabbitMQEventBus);
    container.bind<IStateReportingService>(TYPES.TexasReporting).to(TexasRRCReportingService);
    container.bind<IStateReportingService>(TYPES.OklahomaReporting).to(OklahomaCCReportingService);
  }
}
```

## Benefits

1. **Single Responsibility**
   - Easier to test individual components
   - Changes in regulations only affect specific modules
   - Clearer code organization

2. **Open/Closed**
   - Add new states without modifying existing code
   - New compliance requirements don't break existing functionality
   - Plugin architecture for vendor integrations

3. **Liskov Substitution**
   - Polymorphism works correctly
   - No surprises when using derived classes
   - Consistent behavior across well states

4. **Interface Segregation**
   - Role-based implementations
   - No unnecessary dependencies
   - Cleaner API surface

5. **Dependency Inversion**
   - Testable with mock implementations
   - Swappable infrastructure components
   - Better separation of concerns

## Implementation Checklist

- [ ] Identify classes with multiple responsibilities
- [ ] Extract single responsibilities into separate classes
- [ ] Replace conditionals with polymorphism (OCP)
- [ ] Ensure derived classes honor base class contracts (LSP)
- [ ] Break large interfaces into role-specific ones (ISP)
- [ ] Introduce abstractions between high and low-level modules (DIP)
- [ ] Set up dependency injection container
- [ ] Create unit tests for each SOLID principle
- [ ] Document architectural decisions
- [ ] Review code for SOLID violations in PRs
