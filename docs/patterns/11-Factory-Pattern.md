# Factory Pattern

## Overview

The Factory pattern provides an interface for creating objects in a superclass,
but allows subclasses to alter the type of objects that will be created. It
encapsulates object creation logic and provides a way to delegate the
instantiation process to subclasses or specialized factory classes.

## Core Concepts

### Factory Method

A method that returns an instance of a class, encapsulating the creation logic.

### Abstract Factory

An interface for creating families of related objects.

### Concrete Factory

Specific implementations that create concrete objects.

### Product Interface

The common interface for all products created by the factory.

## Benefits

- **Encapsulation**: Object creation logic is centralized and hidden
- **Flexibility**: Easy to add new product types without modifying existing code
- **Loose Coupling**: Clients depend on abstractions, not concrete classes
- **Single Responsibility**: Creation logic is separated from business logic
- **Consistency**: Ensures objects are created in a consistent manner
- **Testability**: Factory methods can be easily mocked for testing

## Implementation in Our Project

### Before: Direct Object Creation

```typescript
@Injectable()
export class VendorCreationService {
  constructor(
    private readonly vendorRepository: VendorRepository,
    private readonly auditService: AuditService,
  ) {}

  async createVendor(dto: CreateVendorDto): Promise<Vendor> {
    // Direct object creation with complex initialization logic scattered
    const vendor = new Vendor();
    vendor.id = generateId();
    vendor.organizationId = dto.organizationId;
    vendor.name = dto.name;
    vendor.code = dto.code;
    vendor.status = VendorStatus.PENDING;
    vendor.createdAt = new Date();
    vendor.updatedAt = new Date();

    // Complex validation and initialization
    if (dto.contactInfo) {
      const contactInfo = new ContactInfo();
      contactInfo.email = dto.contactInfo.email;
      contactInfo.phone = dto.contactInfo.phone;

      if (dto.contactInfo.address) {
        const address = new Address();
        address.street = dto.contactInfo.address.street;
        address.city = dto.contactInfo.address.city;
        address.state = dto.contactInfo.address.state;
        address.zipCode = dto.contactInfo.address.zipCode;
        contactInfo.address = address;
      }

      vendor.contactInfo = contactInfo;
    }

    if (dto.insurance) {
      const insurance = new Insurance();
      insurance.provider = dto.insurance.provider;
      insurance.policyNumber = dto.insurance.policyNumber;
      insurance.expiryDate = new Date(dto.insurance.expiryDate);
      insurance.coverageAmount = dto.insurance.coverageAmount;
      vendor.insurance = insurance;
    }

    // Validation logic mixed with creation
    if (!vendor.name || vendor.name.trim().length === 0) {
      throw new Error('Vendor name is required');
    }

    if (!vendor.code || !this.isValidVendorCode(vendor.code)) {
      throw new Error('Invalid vendor code');
    }

    await this.vendorRepository.save(vendor);

    // Audit creation
    await this.auditService.logVendorCreation(vendor.id, dto.userId);

    return vendor;
  }

  async createContractVendor(contractDto: CreateContractVendorDto): Promise<Vendor> {
    // Similar but slightly different creation logic
    const vendor = new Vendor();
    vendor.id = generateId();
    vendor.organizationId = contractDto.organizationId;
    vendor.name = contractDto.companyName;
    vendor.code = this.generateVendorCode(contractDto.companyName);
    vendor.status = VendorStatus.ACTIVE; // Different default status
    vendor.createdAt = new Date();
    vendor.updatedAt = new Date();

    // Contract-specific initialization
    vendor.contractInfo = {
      contractNumber: contractDto.contractNumber,
      startDate: contractDto.startDate,
      endDate: contractDto.endDate,
    };

    // Duplicate validation logic
    if (!vendor.name || vendor.name.trim().length === 0) {
      throw new Error('Company name is required');
    }

    await this.vendorRepository.save(vendor);
    await this.auditService.logContractVendorCreation(vendor.id, contractDto.userId);

    return vendor;
  }

  private isValidVendorCode(code: string): boolean {
    return /^[A-Z0-9_-]{3,20}$/.test(code);
  }

  private generateVendorCode(companyName: string): string {
    return companyName
      .toUpperCase()
      .replace(/[^A-Z0-9]/g, '')
      .substring(0, 10);
  }
}
```

### After: Factory Pattern Implementation

```typescript
// Product interface
export interface IVendor {
  getId(): VendorId;
  getName(): VendorName;
  getCode(): VendorCode;
  getStatus(): VendorStatus;
  // ... other vendor methods
}

// Abstract Factory interface
export interface IVendorFactory {
  createVendor(data: VendorCreationData): Promise<Vendor>;
  createFromDto(dto: any): Promise<Vendor>;
  supports(type: VendorType): boolean;
}

// Concrete Factory for standard vendors
@Injectable()
export class StandardVendorFactory implements IVendorFactory {
  constructor(
    private readonly idGenerator: IIdGenerator,
    private readonly codeGenerator: IVendorCodeGenerator,
    private readonly validator: IVendorValidator,
  ) {}

  async createVendor(data: VendorCreationData): Promise<Vendor> {
    // Encapsulated creation logic
    await this.validateCreationData(data);

    const vendor = Vendor.create({
      id: this.idGenerator.generate(),
      organizationId: data.organizationId,
      name: new VendorName(data.name),
      code: await this.generateVendorCode(data.name, data.organizationId),
      status: VendorStatus.PENDING,
      contactInfo: this.createContactInfo(data.contactInfo),
      insurance: this.createInsurance(data.insurance),
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    return vendor;
  }

  async createFromDto(dto: CreateVendorDto): Promise<Vendor> {
    const creationData: VendorCreationData = {
      organizationId: dto.organizationId,
      name: dto.name,
      code: dto.code,
      contactInfo: dto.contactInfo,
      insurance: dto.insurance,
      type: VendorType.STANDARD,
    };

    return await this.createVendor(creationData);
  }

  supports(type: VendorType): boolean {
    return type === VendorType.STANDARD;
  }

  private async validateCreationData(data: VendorCreationData): Promise<void> {
    const errors = await this.validator.validateCreationData(data);
    if (errors.length > 0) {
      throw new VendorValidationError(errors);
    }
  }

  private async generateVendorCode(name: string, organizationId: string): Promise<VendorCode> {
    return await this.codeGenerator.generateUniqueCode(name, organizationId);
  }

  private createContactInfo(data?: ContactInfoData): ContactInfo | undefined {
    if (!data) return undefined;

    return ContactInfo.create({
      email: new Email(data.email),
      phone: data.phone ? new PhoneNumber(data.phone) : undefined,
      address: this.createAddress(data.address),
      contactPerson: data.contactPerson,
    });
  }

  private createAddress(data?: AddressData): Address | undefined {
    if (!data) return undefined;

    return Address.create({
      street: data.street,
      city: data.city,
      state: data.state,
      zipCode: data.zipCode,
      country: data.country || 'USA',
    });
  }

  private createInsurance(data?: InsuranceData): Insurance | undefined {
    if (!data) return undefined;

    return Insurance.create({
      provider: data.provider,
      policyNumber: data.policyNumber,
      expiryDate: new Date(data.expiryDate),
      coverageAmount: Money.fromAmount(data.coverageAmount, 'USD'),
    });
  }
}

// Concrete Factory for contract vendors
@Injectable()
export class ContractVendorFactory implements IVendorFactory {
  constructor(
    private readonly idGenerator: IIdGenerator,
    private readonly codeGenerator: IVendorCodeGenerator,
    private readonly contractService: IContractService,
  ) {}

  async createVendor(data: VendorCreationData): Promise<Vendor> {
    await this.validateContractData(data);

    const vendor = Vendor.create({
      id: this.idGenerator.generate(),
      organizationId: data.organizationId,
      name: new VendorName(data.name),
      code: await this.generateVendorCode(data.name, data.organizationId),
      status: VendorStatus.ACTIVE, // Contract vendors start as active
      contactInfo: this.createContactInfo(data.contactInfo),
      insurance: this.createInsurance(data.insurance),
      contractInfo: this.createContractInfo(data.contractInfo),
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    return vendor;
  }

  async createFromDto(dto: CreateContractVendorDto): Promise<Vendor> {
    const creationData: VendorCreationData = {
      organizationId: dto.organizationId,
      name: dto.companyName,
      contactInfo: dto.contactInfo,
      insurance: dto.insurance,
      contractInfo: {
        contractNumber: dto.contractNumber,
        startDate: dto.startDate,
        endDate: dto.endDate,
        terms: dto.terms,
      },
      type: VendorType.CONTRACT,
    };

    return await this.createVendor(creationData);
  }

  supports(type: VendorType): boolean {
    return type === VendorType.CONTRACT;
  }

  private async validateContractData(data: VendorCreationData): Promise<void> {
    if (!data.contractInfo) {
      throw new VendorValidationError(['Contract information is required']);
    }

    // Contract-specific validation
    const existingContract = await this.contractService.findByNumber(
      data.contractInfo.contractNumber,
    );

    if (existingContract) {
      throw new VendorValidationError(['Contract number already exists']);
    }
  }

  private async generateVendorCode(name: string, organizationId: string): Promise<VendorCode> {
    // Contract vendors use a different code generation strategy
    const baseCode = VendorCode.generateFromCompanyName(name);
    return await this.codeGenerator.generateUniqueCodeWithPrefix('CTR', baseCode, organizationId);
  }

  private createContractInfo(data?: ContractInfoData): ContractInfo | undefined {
    if (!data) return undefined;

    return ContractInfo.create({
      contractNumber: data.contractNumber,
      startDate: new Date(data.startDate),
      endDate: new Date(data.endDate),
      terms: data.terms,
    });
  }
}

// Abstract Factory Registry
@Injectable()
export class VendorFactoryRegistry {
  private factories: Map<VendorType, IVendorFactory> = new Map();

  constructor(
    standardFactory: StandardVendorFactory,
    contractFactory: ContractVendorFactory,
    internationalFactory: InternationalVendorFactory,
  ) {
    this.register(VendorType.STANDARD, standardFactory);
    this.register(VendorType.CONTRACT, contractFactory);
    this.register(VendorType.INTERNATIONAL, internationalFactory);
  }

  register(type: VendorType, factory: IVendorFactory): void {
    this.factories.set(type, factory);
  }

  getFactory(type: VendorType): IVendorFactory {
    const factory = this.factories.get(type);

    if (!factory) {
      throw new UnsupportedVendorTypeError(type);
    }

    return factory;
  }

  getAllSupportedTypes(): VendorType[] {
    return Array.from(this.factories.keys());
  }
}

// Clean service using factories
@Injectable()
export class VendorCreationService {
  constructor(
    private readonly factoryRegistry: VendorFactoryRegistry,
    private readonly vendorRepository: IVendorRepository,
    private readonly auditService: IAuditService,
    private readonly eventBus: EventBus,
  ) {}

  async createVendor(dto: CreateVendorDto, vendorType: VendorType): Promise<string> {
    const factory = this.factoryRegistry.getFactory(vendorType);

    const vendor = await factory.createFromDto(dto);

    await this.vendorRepository.save(vendor);

    // Audit and events
    await this.auditService.logVendorCreation(vendor.getId(), dto.userId);

    const domainEvents = vendor.getDomainEvents();
    for (const event of domainEvents) {
      this.eventBus.publish(event);
    }

    return vendor.getId().getValue();
  }

  async createContractVendor(dto: CreateContractVendorDto): Promise<string> {
    const factory = this.factoryRegistry.getFactory(VendorType.CONTRACT);

    const vendor = await factory.createFromDto(dto);

    await this.vendorRepository.save(vendor);

    await this.auditService.logContractVendorCreation(vendor.getId(), dto.userId);

    return vendor.getId().getValue();
  }
}
```

## Advanced Factory Patterns

### Abstract Factory for Related Objects

```typescript
// Abstract factory for creating families of related objects
export interface ILeaseOperatingStatementFactory {
  createLos(data: LosCreationData): Promise<LeaseOperatingStatement>;
  createExpenseLineItem(data: ExpenseItemData): ExpenseLineItem;
  createAllocation(data: AllocationData): Allocation;
  createReport(data: ReportData): LosReport;
}

// Concrete factory for standard LOS processing
@Injectable()
export class StandardLosFactory implements ILeaseOperatingStatementFactory {
  constructor(
    private readonly idGenerator: IIdGenerator,
    private readonly allocationCalculator: IStandardAllocationCalculator,
  ) {}

  async createLos(data: LosCreationData): Promise<LeaseOperatingStatement> {
    return LeaseOperatingStatement.create({
      id: this.idGenerator.generate(),
      organizationId: data.organizationId,
      leaseId: data.leaseId,
      statementMonth: new StatementMonth(data.year, data.month),
      status: LosStatus.DRAFT,
      processingType: LosProcessingType.STANDARD,
      createdAt: new Date(),
      updatedAt: new Date(),
    });
  }

  createExpenseLineItem(data: ExpenseItemData): ExpenseLineItem {
    return ExpenseLineItem.create({
      id: this.idGenerator.generate(),
      description: data.description,
      category: ExpenseCategory.fromString(data.category),
      type: ExpenseType.fromString(data.type),
      amount: Money.fromAmount(data.amount, data.currency || 'USD'),
      vendorName: data.vendorName,
      invoiceNumber: data.invoiceNumber,
      invoiceDate: data.invoiceDate ? new Date(data.invoiceDate) : undefined,
    });
  }

  createAllocation(data: AllocationData): Allocation {
    return this.allocationCalculator.calculateStandardAllocation(data);
  }

  createReport(data: ReportData): LosReport {
    return new StandardLosReport(data);
  }
}

// Concrete factory for joint venture LOS processing
@Injectable()
export class JointVentureLosFactory implements ILeaseOperatingStatementFactory {
  constructor(
    private readonly idGenerator: IIdGenerator,
    private readonly jvAllocationCalculator: IJointVentureAllocationCalculator,
  ) {}

  async createLos(data: LosCreationData): Promise<LeaseOperatingStatement> {
    const los = LeaseOperatingStatement.create({
      id: this.idGenerator.generate(),
      organizationId: data.organizationId,
      leaseId: data.leaseId,
      statementMonth: new StatementMonth(data.year, data.month),
      status: LosStatus.DRAFT,
      processingType: LosProcessingType.JOINT_VENTURE,
      createdAt: new Date(),
      updatedAt: new Date(),
    });

    // Joint venture specific initialization
    los.enableJointVentureTracking();
    return los;
  }

  createExpenseLineItem(data: ExpenseItemData): ExpenseLineItem {
    const expenseItem = ExpenseLineItem.create({
      id: this.idGenerator.generate(),
      description: data.description,
      category: ExpenseCategory.fromString(data.category),
      type: ExpenseType.fromString(data.type),
      amount: Money.fromAmount(data.amount, data.currency || 'USD'),
      vendorName: data.vendorName,
      invoiceNumber: data.invoiceNumber,
      invoiceDate: data.invoiceDate ? new Date(data.invoiceDate) : undefined,
    });

    // Joint venture specific features
    expenseItem.enablePartnerAllocation();
    return expenseItem;
  }

  createAllocation(data: AllocationData): Allocation {
    return this.jvAllocationCalculator.calculateJointVentureAllocation(data);
  }

  createReport(data: ReportData): LosReport {
    return new JointVentureLosReport(data);
  }
}
```

### Factory with Builder Pattern

```typescript
// Factory that uses builder pattern for complex object creation
export interface IVendorBuilder {
  setOrganizationId(organizationId: string): IVendorBuilder;
  setName(name: string): IVendorBuilder;
  setCode(code: string): IVendorBuilder;
  setContactInfo(contactInfo: ContactInfoData): IVendorBuilder;
  setInsurance(insurance: InsuranceData): IVendorBuilder;
  setContractInfo(contractInfo: ContractInfoData): IVendorBuilder;
  build(): Promise<Vendor>;
}

@Injectable()
export class VendorBuilder implements IVendorBuilder {
  private vendorData: Partial<VendorCreationData> = {};

  constructor(
    private readonly idGenerator: IIdGenerator,
    private readonly validator: IVendorValidator,
  ) {}

  setOrganizationId(organizationId: string): IVendorBuilder {
    this.vendorData.organizationId = organizationId;
    return this;
  }

  setName(name: string): IVendorBuilder {
    this.vendorData.name = name;
    return this;
  }

  setCode(code: string): IVendorBuilder {
    this.vendorData.code = code;
    return this;
  }

  setContactInfo(contactInfo: ContactInfoData): IVendorBuilder {
    this.vendorData.contactInfo = contactInfo;
    return this;
  }

  setInsurance(insurance: InsuranceData): IVendorBuilder {
    this.vendorData.insurance = insurance;
    return this;
  }

  setContractInfo(contractInfo: ContractInfoData): IVendorBuilder {
    this.vendorData.contractInfo = contractInfo;
    return this;
  }

  async build(): Promise<Vendor> {
    await this.validateData();

    return Vendor.create({
      id: this.idGenerator.generate(),
      organizationId: this.vendorData.organizationId!,
      name: new VendorName(this.vendorData.name!),
      code: new VendorCode(this.vendorData.code!),
      status: this.determineInitialStatus(),
      contactInfo: this.buildContactInfo(),
      insurance: this.buildInsurance(),
      contractInfo: this.buildContractInfo(),
      createdAt: new Date(),
      updatedAt: new Date(),
    });
  }

  private async validateData(): Promise<void> {
    const required = ['organizationId', 'name', 'code'];
    const missing = required.filter((field) => !this.vendorData[field]);

    if (missing.length > 0) {
      throw new VendorValidationError([`Missing required fields: ${missing.join(', ')}`]);
    }

    const errors = await this.validator.validateCreationData(this.vendorData as VendorCreationData);
    if (errors.length > 0) {
      throw new VendorValidationError(errors);
    }
  }

  private determineInitialStatus(): VendorStatus {
    return this.vendorData.contractInfo ? VendorStatus.ACTIVE : VendorStatus.PENDING;
  }

  private buildContactInfo(): ContactInfo | undefined {
    if (!this.vendorData.contactInfo) return undefined;

    return ContactInfo.create({
      email: new Email(this.vendorData.contactInfo.email),
      phone: this.vendorData.contactInfo.phone
        ? new PhoneNumber(this.vendorData.contactInfo.phone)
        : undefined,
      address: this.vendorData.contactInfo.address
        ? Address.create(this.vendorData.contactInfo.address)
        : undefined,
      contactPerson: this.vendorData.contactInfo.contactPerson,
    });
  }

  private buildInsurance(): Insurance | undefined {
    if (!this.vendorData.insurance) return undefined;

    return Insurance.create({
      provider: this.vendorData.insurance.provider,
      policyNumber: this.vendorData.insurance.policyNumber,
      expiryDate: new Date(this.vendorData.insurance.expiryDate),
      coverageAmount: Money.fromAmount(this.vendorData.insurance.coverageAmount, 'USD'),
    });
  }

  private buildContractInfo(): ContractInfo | undefined {
    if (!this.vendorData.contractInfo) return undefined;

    return ContractInfo.create(this.vendorData.contractInfo);
  }
}

// Factory that uses the builder
@Injectable()
export class BuilderBasedVendorFactory implements IVendorFactory {
  constructor(private readonly builderProvider: () => IVendorBuilder) {}

  async createVendor(data: VendorCreationData): Promise<Vendor> {
    const builder = this.builderProvider()
      .setOrganizationId(data.organizationId)
      .setName(data.name)
      .setCode(data.code!);

    if (data.contactInfo) {
      builder.setContactInfo(data.contactInfo);
    }

    if (data.insurance) {
      builder.setInsurance(data.insurance);
    }

    if (data.contractInfo) {
      builder.setContractInfo(data.contractInfo);
    }

    return await builder.build();
  }

  async createFromDto(dto: any): Promise<Vendor> {
    const creationData: VendorCreationData = {
      organizationId: dto.organizationId,
      name: dto.name,
      code: dto.code,
      contactInfo: dto.contactInfo,
      insurance: dto.insurance,
      contractInfo: dto.contractInfo,
      type: dto.type || VendorType.STANDARD,
    };

    return await this.createVendor(creationData);
  }

  supports(type: VendorType): boolean {
    return true; // Builder can handle any type
  }
}
```

## Configuration-Driven Factory

```typescript
// Factory configuration
export interface FactoryConfiguration {
  vendorTypes: {
    [key: string]: {
      factoryClass: string;
      defaultStatus: VendorStatus;
      requiredFields: string[];
      validationRules: ValidationRule[];
    };
  };
}

// Configuration-driven factory
@Injectable()
export class ConfigurableVendorFactory implements IVendorFactory {
  private configuration: FactoryConfiguration;

  constructor(
    @Inject('FACTORY_CONFIGURATION')
    configuration: FactoryConfiguration,
    private readonly moduleRef: ModuleRef,
  ) {
    this.configuration = configuration;
  }

  async createVendor(data: VendorCreationData): Promise<Vendor> {
    const config = this.configuration.vendorTypes[data.type];

    if (!config) {
      throw new UnsupportedVendorTypeError(data.type);
    }

    // Validate based on configuration
    await this.validateUsingConfiguration(data, config);

    // Create using configured factory class
    const factoryClass = await this.moduleRef.get(config.factoryClass);
    return await factoryClass.createVendor(data);
  }

  async createFromDto(dto: any): Promise<Vendor> {
    // Implementation using configuration
    throw new Error('Method not implemented');
  }

  supports(type: VendorType): boolean {
    return !!this.configuration.vendorTypes[type];
  }

  private async validateUsingConfiguration(data: VendorCreationData, config: any): Promise<void> {
    // Validate required fields
    const missing = config.requiredFields.filter((field) => !data[field]);
    if (missing.length > 0) {
      throw new VendorValidationError([`Missing required fields: ${missing.join(', ')}`]);
    }

    // Apply validation rules
    for (const rule of config.validationRules) {
      const isValid = await this.comlyValidationRule(data, rule);
      if (!isValid) {
        throw new VendorValidationError([rule.errorMessage]);
      }
    }
  }

  private async applyValidationRule(
    data: VendorCreationData,
    rule: ValidationRule,
  ): Promise<boolean> {
    // Implementation of rule validation
    return true;
  }
}
```

## Testing Factory Pattern

### Factory Testing

```typescript
describe('StandardVendorFactory', () => {
  let factory: StandardVendorFactory;
  let mockIdGenerator: jest.Mocked<IIdGenerator>;
  let mockCodeGenerator: jest.Mocked<IVendorCodeGenerator>;
  let mockValidator: jest.Mocked<IVendorValidator>;

  beforeEach(() => {
    mockIdGenerator = {
      generate: jest.fn().mockReturnValue('vendor-123'),
    };

    mockCodeGenerator = {
      generateUniqueCode: jest.fn().mockResolvedValue(new VendorCode('TEST-01')),
    };

    mockValidator = {
      validateCreationData: jest.fn().mockResolvedValue([]),
    };

    factory = new StandardVendorFactory(mockIdGenerator, mockCodeGenerator, mockValidator);
  });

  describe('createVendor', () => {
    it('should create vendor with valid data', async () => {
      const data: VendorCreationData = {
        organizationId: 'org-123',
        name: 'Test Vendor',
        contactInfo: {
          email: 'test@vendor.com',
          phone: '+1234567890',
        },
        insurance: {
          provider: 'Test Insurance',
          policyNumber: 'POL-123',
          expiryDate: '2025-12-31',
          coverageAmount: 1000000,
        },
        type: VendorType.STANDARD,
      };

      const vendor = await factory.createVendor(data);

      expect(vendor).toBeInstanceOf(Vendor);
      expect(vendor.getName().getValue()).toBe('Test Vendor');
      expect(vendor.getStatus()).toBe(VendorStatus.PENDING);
      expect(mockIdGenerator.generate).toHaveBeenCalled();
      expect(mockValidator.validateCreationData).toHaveBeenCalledWith(data);
    });

    it('should throw error for invalid data', async () => {
      const data: VendorCreationData = {
        organizationId: 'org-123',
        name: '', // Invalid name
        type: VendorType.STANDARD,
      };

      mockValidator.validateCreationData.mockResolvedValue(['Vendor name is required']);

      await expect(factory.createVendor(data)).rejects.toThrow(VendorValidationError);
    });
  });

  describe('createFromDto', () => {
    it('should create vendor from DTO', async () => {
      const dto: CreateVendorDto = {
        organizationId: 'org-123',
        name: 'Test Vendor',
        code: 'TEST-01',
        contactInfo: {
          email: 'test@vendor.com',
        },
      };

      const vendor = await factory.createFromDto(dto);

      expect(vendor).toBeInstanceOf(Vendor);
      expect(vendor.getName().getValue()).toBe('Test Vendor');
    });
  });

  describe('supports', () => {
    it('should support standard vendor type', () => {
      expect(factory.supports(VendorType.STANDARD)).toBe(true);
      expect(factory.supports(VendorType.CONTRACT)).toBe(false);
    });
  });
});

describe('VendorFactoryRegistry', () => {
  let registry: VendorFactoryRegistry;
  let standardFactory: jest.Mocked<IVendorFactory>;
  let contractFactory: jest.Mocked<IVendorFactory>;

  beforeEach(() => {
    standardFactory = {
      createVendor: jest.fn(),
      createFromDto: jest.fn(),
      supports: jest.fn().mockReturnValue(true),
    };

    contractFactory = {
      createVendor: jest.fn(),
      createFromDto: jest.fn(),
      supports: jest.fn().mockReturnValue(true),
    };

    registry = new VendorFactoryRegistry(
      standardFactory as any,
      contractFactory as any,
      null as any,
    );
  });

  describe('getFactory', () => {
    it('should return correct factory for vendor type', () => {
      const factory = registry.getFactory(VendorType.STANDARD);
      expect(factory).toBe(standardFactory);
    });

    it('should throw error for unsupported type', () => {
      expect(() => registry.getFactory('UNSUPPORTED' as VendorType)).toThrow(
        UnsupportedVendorTypeError,
      );
    });
  });

  describe('getAllSupportedTypes', () => {
    it('should return all registered types', () => {
      const types = registry.getAllSupportedTypes();
      expect(types).toContain(VendorType.STANDARD);
      expect(types).toContain(VendorType.CONTRACT);
    });
  });
});
```

## Best Practices

### 1. Clear Factory Interface

```typescript
// Good: Clear, specific interface
export interface IVendorFactory {
  createVendor(data: VendorCreationData): Promise<Vendor>;
  createFromDto(dto: any): Promise<Vendor>;
  supports(type: VendorType): boolean;
}

// Avoid: Generic factory
export interface IFactory<T> {
  create(data: any): T; // Too generic, loses type safety
}
```

### 2. Factory Registration

```typescript
// Good: Registry pattern for managing factories
@Injectable()
export class FactoryRegistry<T> {
  private factories: Map<string, T> = new Map();

  register(type: string, factory: T): void {
    this.factories.set(type, factory);
  }

  get(type: string): T {
    const factory = this.factories.get(type);
    if (!factory) {
      throw new Error(`No factory registered for type: ${type}`);
    }
    return factory;
  }
}
```

### 3. Factory Validation

```typescript
// Good: Validate factory capabilities
export abstract class BaseFactory<T> {
  abstract create(data: any): Promise<T>;
  abstract supports(type: string): boolean;

  protected validateSupport(type: string): void {
    if (!this.supports(type)) {
      throw new Error(`${this.constructor.name} does not support type: ${type}`);
    }
  }

  protected validateData(data: any): void {
    if (!data) {
      throw new Error('Creation data is required');
    }
  }
}
```

The Factory pattern in our oil & gas management system provides a clean way to
encapsulate object creation logic, making it easy to support different types of
vendors, lease operating statements, and other domain objects while maintaining
consistency and testability.
