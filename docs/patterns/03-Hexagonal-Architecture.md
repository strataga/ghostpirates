# Hexagonal Architecture Pattern (Ports and Adapters)

## Overview

Hexagonal Architecture isolates the core business logic from external systems
(databases, APIs, UI) through ports and adapters. In oil & gas operations, this
is crucial for integrating with various state regulatory APIs, equipment
sensors, and partner systems while keeping business rules independent.

## Purpose

- Isolate business logic from infrastructure
- Enable testing without external dependencies
- Support multiple interfaces (Web, Mobile, API, CLI)
- Swap implementations without changing business logic
- Integrate with various state regulatory systems

## Oil & Gas Use Cases

- Integrate with different state regulatory APIs (Texas RRC, Oklahoma CC, New
  Mexico OCD)
- Support multiple data sources (SCADA systems, manual entry, IoT sensors)
- Switch between cloud providers (AWS, Azure) without changing business logic
- Support different payment processors for royalty distributions
- Connect to various equipment monitoring systems

## Before Implementation

```typescript
// ❌ POOR: Business logic tightly coupled to infrastructure

class WellProductionService {
  constructor() {
    // Direct dependency on specific implementations
    this.db = new PostgreSQLConnection();
    this.texasApi = new TexasRRCApi();
    this.emailService = new SendGridService();
    this.scadaSystem = new SchneiderElectricScada();
  }

  async recordDailyProduction(wellId: string, data: any) {
    // Business logic mixed with infrastructure

    // Direct SCADA system call
    const scadaData = await this.scadaSystem.getData(wellId);

    // Direct database access
    const well = await this.db.query('SELECT * FROM wells WHERE id = $1', [wellId]);

    // Business calculation mixed with data access
    const production = {
      oil: scadaData.oilMeters * 0.98, // Loss factor hard-coded
      gas: scadaData.gasMeter * 1000, // MCF conversion
      water: scadaData.waterMeter,
    };

    // Direct database insert
    await this.db.query('INSERT INTO production VALUES ($1, $2, $3, $4, $5)', [
      wellId,
      production.oil,
      production.gas,
      production.water,
      new Date(),
    ]);

    // State-specific API call embedded in service
    if (well.state === 'TX') {
      await this.texasApi.post('/production', {
        apiNumber: well.api_number,
        oil: production.oil,
        gas: production.gas,
        date: new Date(),
      });
    } else if (well.state === 'OK') {
      // Different API structure for Oklahoma
      const okApi = new OklahomaOCCApi();
      await okApi.submitProduction({
        wellCode: well.api_number,
        oilBbls: production.oil,
        gasMcf: production.gas,
      });
    }

    // Direct email service call
    await this.emailService.send({
      to: well.operator_email,
      subject: 'Daily Production Recorded',
      body: `Production: ${production.oil} bbl oil`,
    });
  }

  async generateMonthlyReport(wellId: string, month: string) {
    // More infrastructure coupling
    const data = await this.db.query(
      'SELECT SUM(oil), SUM(gas) FROM production WHERE well_id = $1 AND month = $2',
      [wellId, month],
    );

    // PDF generation tied to specific library
    const pdf = new PDFKit();
    pdf.text(`Monthly Production: ${data.oil} bbl`);

    // Direct file system access
    const fs = require('fs');
    fs.writeFileSync(`/reports/${wellId}-${month}.pdf`, pdf);

    return `/reports/${wellId}-${month}.pdf`;
  }
}

// Problems:
// - Can't test without database and external APIs
// - Can't switch SCADA providers easily
// - Business logic tied to infrastructure
// - Adding new states requires modifying core service
// - Can't reuse business logic in different contexts
```

## After Implementation

```typescript
// ✅ GOOD: Hexagonal Architecture with Ports and Adapters

// ============= DOMAIN LAYER (Hexagon Core) =============

// Domain Entities (Pure business logic)
class Well {
  constructor(
    private id: WellId,
    private apiNumber: ApiNumber,
    private location: Location,
    private operator: Operator,
  ) {}

  calculateNetProduction(grossProduction: Production): Production {
    // Pure business logic - no infrastructure dependencies
    const lossFactor = this.location.isOffshore() ? 0.02 : 0.015;
    return grossProduction.comlyLossFactor(lossFactor);
  }

  requiresStateReporting(): boolean {
    return this.location.state.hasReportingRequirement();
  }
}

class Production {
  constructor(
    private oil: Volume,
    private gas: Volume,
    private water: Volume,
    private recordedAt: Date,
  ) {}

  applyLossFactor(factor: number): Production {
    return new Production(
      this.oil.multiply(1 - factor),
      this.gas.multiply(1 - factor),
      this.water,
      this.recordedAt,
    );
  }

  toBarrels(): { oil: number; gas: number; water: number } {
    return {
      oil: this.oil.toBarrels(),
      gas: this.gas.toMCF(),
      water: this.water.toBarrels(),
    };
  }
}

// ============= APPLICATION LAYER (Use Cases) =============

class RecordProductionUseCase {
  constructor(
    // Depend on ports (interfaces), not implementations
    private wellRepository: IWellRepository,
    private productionRepository: IProductionRepository,
    private scadaPort: IScadaPort,
    private compliancePort: ICompliancePort,
    private notificationPort: INotificationPort,
    private eventBus: IEventBus,
  ) {}

  async execute(command: RecordProductionCommand): Promise<void> {
    // Pure business logic using ports

    // Get well through port
    const well = await this.wellRepository.findById(command.wellId);
    if (!well) {
      throw new WellNotFoundError(command.wellId);
    }

    // Get sensor data through port
    const sensorData = await this.scadaPort.getCurrentReadings(well.id);

    // Business logic - calculate production
    const grossProduction = this.mapSensorDataToProduction(sensorData);
    const netProduction = well.calculateNetProduction(grossProduction);

    // Validate business rules
    this.validateProduction(netProduction);

    // Save through port
    await this.productionRepository.save(well.id, netProduction);

    // Report to state if required
    if (well.requiresStateReporting()) {
      await this.compliancePort.reportProduction(well, netProduction);
    }

    // Notify through port
    await this.notificationPort.notifyProductionRecorded(well, netProduction);

    // Publish domain event
    await this.eventBus.publish(new ProductionRecordedEvent(well.id, netProduction));
  }

  private mapSensorDataToProduction(data: SensorData): Production {
    return new Production(
      new Volume(data.oilReading, VolumeUnit.BARRELS),
      new Volume(data.gasReading, VolumeUnit.MCF),
      new Volume(data.waterReading, VolumeUnit.BARRELS),
      new Date(),
    );
  }

  private validateProduction(production: Production): void {
    if (production.oil.isNegative()) {
      throw new InvalidProductionError('Oil volume cannot be negative');
    }
    // More validation rules
  }
}

// ============= PORTS (Interfaces) =============

// Inbound Port (Driving Port) - How the application is accessed
interface IProductionService {
  recordProduction(wellId: string, data: ProductionData): Promise<void>;
  getMonthlyProduction(wellId: string, month: Month): Promise<Production[]>;
}

// Outbound Ports (Driven Ports) - What the application needs

interface IWellRepository {
  findById(id: WellId): Promise<Well | null>;
  findByApiNumber(apiNumber: ApiNumber): Promise<Well | null>;
  save(well: Well): Promise<void>;
}

interface IProductionRepository {
  save(wellId: WellId, production: Production): Promise<void>;
  findByMonth(wellId: WellId, month: Month): Promise<Production[]>;
}

interface IScadaPort {
  getCurrentReadings(wellId: WellId): Promise<SensorData>;
  getHistoricalData(wellId: WellId, range: DateRange): Promise<SensorData[]>;
}

interface ICompliancePort {
  reportProduction(well: Well, production: Production): Promise<void>;
  submitMonthlyReport(report: ComplianceReport): Promise<SubmissionResult>;
}

interface INotificationPort {
  notifyProductionRecorded(well: Well, production: Production): Promise<void>;
  sendAlert(alert: Alert): Promise<void>;
}

interface IEventBus {
  publish(event: DomainEvent): Promise<void>;
  subscribe(eventType: string, handler: EventHandler): void;
}

// ============= ADAPTERS (Infrastructure) =============

// Primary Adapters (Driving) - Entry points to the application

// REST API Adapter
@Controller('production')
class ProductionRestAdapter {
  constructor(private useCase: RecordProductionUseCase) {}

  @Post(':wellId')
  async recordProduction(
    @Param('wellId') wellId: string,
    @Body() dto: RecordProductionDto,
  ): Promise<void> {
    const command = new RecordProductionCommand(new WellId(wellId), dto.toProductionData());
    await this.useCase.execute(command);
  }
}

// GraphQL Adapter
@Resolver('Production')
class ProductionGraphQLAdapter {
  constructor(private useCase: RecordProductionUseCase) {}

  @Mutation()
  async recordProduction(
    @Args('wellId') wellId: string,
    @Args('data') data: ProductionInput,
  ): Promise<boolean> {
    const command = new RecordProductionCommand(new WellId(wellId), data.toProductionData());
    await this.useCase.execute(command);
    return true;
  }
}

// CLI Adapter
class ProductionCliAdapter {
  constructor(private useCase: RecordProductionUseCase) {}

  async run(args: string[]): Promise<void> {
    const wellId = args[0];
    const oil = parseFloat(args[1]);
    const gas = parseFloat(args[2]);

    const command = new RecordProductionCommand(
      new WellId(wellId),
      new ProductionData(oil, gas, 0),
    );
    await this.useCase.execute(command);
  }
}

// Secondary Adapters (Driven) - Implementations of ports

// Database Adapter
class PostgresWellRepository implements IWellRepository {
  constructor(private db: Database) {}

  async findById(id: WellId): Promise<Well | null> {
    const data = await this.db.wells.findOne({ id: id.getValue() });
    if (!data) return null;

    return new Well(
      new WellId(data.id),
      new ApiNumber(data.apiNumber),
      new Location(data.latitude, data.longitude, data.state),
      new Operator(data.operatorId, data.operatorName),
    );
  }

  async save(well: Well): Promise<void> {
    await this.db.wells.upsert(well.toPersistence());
  }
}

// SCADA System Adapters - Easily swappable
class SchneiderScadaAdapter implements IScadaPort {
  constructor(private schneiderApi: SchneiderAPI) {}

  async getCurrentReadings(wellId: WellId): Promise<SensorData> {
    const data = await this.schneiderApi.getLatestData(wellId.getValue());
    return new SensorData(data.oil_meter, data.gas_meter, data.water_meter);
  }
}

class HoneywellScadaAdapter implements IScadaPort {
  constructor(private honeywellClient: HoneywellClient) {}

  async getCurrentReadings(wellId: WellId): Promise<SensorData> {
    const readings = await this.honeywellClient.readSensors(wellId.getValue());
    return new SensorData(readings.oilVolume, readings.gasVolume, readings.waterVolume);
  }
}

// State Compliance Adapters - One per state
class TexasRRCAdapter implements ICompliancePort {
  constructor(private texasApi: TexasRRCApiClient) {}

  async reportProduction(well: Well, production: Production): Promise<void> {
    const texasFormat = this.mapToTexasFormat(well, production);
    await this.texasApi.submitProduction(texasFormat);
  }

  private mapToTexasFormat(well: Well, production: Production): TexasProductionReport {
    return {
      apiNumber: well.apiNumber.toTexasFormat(),
      reportDate: format(new Date(), 'MM/DD/YYYY'),
      oilBBLS: production.oil.toBarrels(),
      gasMCF: production.gas.toMCF(),
      disposition: 'SOLD',
    };
  }
}

class OklahomaOCCAdapter implements ICompliancePort {
  constructor(private oklahomaApi: OklahomaOCCClient) {}

  async reportProduction(well: Well, production: Production): Promise<void> {
    const okFormat = this.mapToOklahomaFormat(well, production);
    await this.oklahomaApi.reportMonthlyProduction(okFormat);
  }

  private mapToOklahomaFormat(well: Well, production: Production): OKProductionData {
    return {
      wellCode: well.apiNumber.getValue(),
      productionMonth: format(new Date(), 'YYYY-MM'),
      oilProduced: production.oil.toBarrels(),
      gasProduced: production.gas.toMCF(),
      operatorLicense: well.operator.licenseNumber,
    };
  }
}

// Notification Adapters
class EmailNotificationAdapter implements INotificationPort {
  constructor(private emailService: IEmailService) {}

  async notifyProductionRecorded(well: Well, production: Production): Promise<void> {
    const email = this.buildProductionEmail(well, production);
    await this.emailService.send(email);
  }

  private buildProductionEmail(well: Well, production: Production): Email {
    return new Email(
      well.operator.email,
      'Daily Production Recorded',
      `Production for ${well.name}: ${production.oil.toBarrels()} bbl oil`,
    );
  }
}

// ============= DEPENDENCY INJECTION SETUP =============

// Configuration to wire everything together
class ApplicationConfig {
  configure(container: DIContainer): void {
    // Register ports with their adapter implementations

    // Repositories
    container.bind<IWellRepository>(TYPES.WellRepository).to(PostgresWellRepository);

    container
      .bind<IProductionRepository>(TYPES.ProductionRepository)
      .to(PostgresProductionRepository);

    // SCADA - Can easily switch providers
    if (config.scadaProvider === 'schneider') {
      container.bind<IScadaPort>(TYPES.ScadaPort).to(SchneiderScadaAdapter);
    } else {
      container.bind<IScadaPort>(TYPES.ScadaPort).to(HoneywellScadaAdapter);
    }

    // State compliance - Multiple registrations
    container.bind<ICompliancePort>(TYPES.TexasCompliance).to(TexasRRCAdapter);

    container.bind<ICompliancePort>(TYPES.OklahomaCompliance).to(OklahomaOCCAdapter);

    // Notifications
    container.bind<INotificationPort>(TYPES.NotificationPort).to(EmailNotificationAdapter);

    // Use cases
    container
      .bind<RecordProductionUseCase>(TYPES.RecordProductionUseCase)
      .to(RecordProductionUseCase);
  }
}

// ============= TESTING =============

// Easy to test with mock adapters
describe('RecordProductionUseCase', () => {
  it('should record production and notify', async () => {
    // Create mock adapters
    const mockWellRepo = new MockWellRepository();
    const mockProductionRepo = new MockProductionRepository();
    const mockScada = new MockScadaPort();
    const mockCompliance = new MockCompliancePort();
    const mockNotification = new MockNotificationPort();
    const mockEventBus = new MockEventBus();

    // Setup test data
    const well = new Well(/* test data */);
    mockWellRepo.setWell(well);
    mockScada.setReading(new SensorData(100, 500, 50));

    // Create use case with mocks
    const useCase = new RecordProductionUseCase(
      mockWellRepo,
      mockProductionRepo,
      mockScada,
      mockCompliance,
      mockNotification,
      mockEventBus,
    );

    // Execute
    await useCase.execute(new RecordProductionCommand(well.id, {}));

    // Verify
    expect(mockProductionRepo.saved).toBeTruthy();
    expect(mockNotification.notified).toBeTruthy();
    expect(mockEventBus.published).toHaveLength(1);
  });
});
```

## Benefits

1. **Testability**: Test business logic without external dependencies
2. **Flexibility**: Swap implementations without changing core logic
3. **Maintainability**: Clear separation of concerns
4. **Scalability**: Add new adapters without modifying existing code
5. **Technology Independence**: Business logic doesn't depend on frameworks

## Implementation Checklist

- [ ] Identify core business logic
- [ ] Define domain entities and value objects
- [ ] Create use cases for business operations
- [ ] Define inbound ports (how app is accessed)
- [ ] Define outbound ports (what app needs)
- [ ] Implement primary adapters (REST, GraphQL, CLI)
- [ ] Implement secondary adapters (DB, APIs, Services)
- [ ] Set up dependency injection
- [ ] Create mock adapters for testing
- [ ] Document port interfaces
- [ ] Create adapter implementation guide
