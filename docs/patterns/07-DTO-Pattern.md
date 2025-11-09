# Data Transfer Object (DTO) Pattern

## Overview

The Data Transfer Object (DTO) pattern is used to transfer data between software
application subsystems. DTOs are simple objects that should not contain any
business logic but may contain serialization and deserialization mechanisms for
transferring data over the wire. They help in reducing the number of method
calls and provide a clear contract for data exchange.

## Core Concepts

### Data Transfer Object

A simple object that carries data between processes to reduce the number of
method calls.

### Boundary Objects

Objects specifically designed for crossing architectural boundaries.

### Serialization/Deserialization

Converting objects to and from formats suitable for transmission or storage.

### Validation

Input validation and data integrity checks at the boundary.

## Benefits

- **Reduced Method Calls**: Aggregate data to minimize network round trips
- **Boundary Definition**: Clear contracts between layers
- **Validation**: Centralized input validation
- **Versioning**: API version control and backward compatibility
- **Security**: Control what data is exposed externally
- **Performance**: Optimize data transfer for specific use cases

## Implementation in Our Project

### Before: Domain Objects Crossing Boundaries

```typescript
// Exposing domain entities directly
@Controller('vendors')
export class VendorController {
  constructor(private readonly vendorService: VendorService) {}

  @Post()
  async createVendor(@Body() vendor: Vendor): Promise<Vendor> {
    // Domain entity used directly in API
    // Problems:
    // - Exposes internal structure
    // - Breaks encapsulation
    // - Tight coupling with domain
    // - Security risks (mass assignment)
    // - Validation mixed with domain logic
    return await this.vendorService.create(vendor);
  }

  @Get(':id')
  async getVendor(@Param('id') id: string): Promise<Vendor> {
    // Returns full domain entity
    // Problems:
    // - May expose sensitive data
    // - Includes behavior methods in JSON
    // - Client depends on domain structure
    return await this.vendorService.findById(id);
  }

  @Put(':id')
  async updateVendor(
    @Param('id') id: string,
    @Body() updateData: Partial<Vendor>,
  ): Promise<Vendor> {
    // Partial domain entity for updates
    // Problems:
    // - Unclear what can be updated
    // - No validation on partial data
    // - May break domain invariants
    return await this.vendorService.update(id, updateData);
  }
}
```

### After: DTO Pattern Implementation

```typescript
// Input DTOs for API requests
export class CreateVendorDto {
  @IsNotEmpty()
  @IsString()
  @MaxLength(255)
  readonly name: string;

  @IsNotEmpty()
  @IsString()
  @Matches(/^[A-Za-z0-9_-]{3,20}$/)
  readonly code: string;

  @IsNotEmpty()
  @IsString()
  readonly organizationId: string;

  @ValidateNested()
  @Type(() => ContactInfoDto)
  readonly contactInfo: ContactInfoDto;

  @ValidateNested()
  @Type(() => InsuranceDto)
  readonly insurance: InsuranceDto;

  @IsOptional()
  @IsString()
  @MaxLength(1000)
  readonly notes?: string;
}

export class UpdateVendorDto {
  @IsOptional()
  @IsString()
  @MaxLength(255)
  readonly name?: string;

  @IsOptional()
  @ValidateNested()
  @Type(() => ContactInfoDto)
  readonly contactInfo?: ContactInfoDto;

  @IsOptional()
  @ValidateNested()
  @Type(() => InsuranceDto)
  readonly insurance?: InsuranceDto;

  @IsOptional()
  @IsString()
  @MaxLength(1000)
  readonly notes?: string;
}

// Output DTOs for API responses
export class VendorDto {
  readonly id: string;
  readonly organizationId: string;
  readonly name: string;
  readonly code: string;
  readonly status: string;
  readonly contactInfo: ContactInfoDto;
  readonly insurance: InsuranceDto;
  readonly createdAt: Date;
  readonly updatedAt: Date;
  readonly notes?: string;

  constructor(data: {
    id: string;
    organizationId: string;
    name: string;
    code: string;
    status: string;
    contactInfo: ContactInfoDto;
    insurance: InsuranceDto;
    createdAt: Date;
    updatedAt: Date;
    notes?: string;
  }) {
    Object.assign(this, data);
  }
}

// Specialized DTOs for different use cases
export class VendorListDto {
  readonly id: string;
  readonly name: string;
  readonly code: string;
  readonly status: string;
  readonly lastActivity: Date;
  readonly activeContracts: number;

  constructor(data: VendorListDto) {
    Object.assign(this, data);
  }
}

export class VendorDetailDto extends VendorDto {
  readonly paymentHistory: PaymentSummaryDto[];
  readonly contracts: ContractSummaryDto[];
  readonly documents: DocumentSummaryDto[];
  readonly complianceStatus: ComplianceStatusDto;

  constructor(data: VendorDetailDto) {
    super(data);
    this.paymentHistory = data.paymentHistory;
    this.contracts = data.contracts;
    this.documents = data.documents;
    this.complianceStatus = data.complianceStatus;
  }
}

// Nested DTOs
export class ContactInfoDto {
  @IsNotEmpty()
  @IsEmail()
  readonly email: string;

  @IsOptional()
  @IsPhoneNumber()
  readonly phone?: string;

  @IsOptional()
  @ValidateNested()
  @Type(() => AddressDto)
  readonly address?: AddressDto;

  @IsOptional()
  @IsString()
  readonly contactPerson?: string;
}

export class InsuranceDto {
  @IsNotEmpty()
  @IsString()
  readonly provider: string;

  @IsNotEmpty()
  @IsString()
  readonly policyNumber: string;

  @IsDateString()
  readonly expiryDate: string;

  @IsNumber()
  @Min(0)
  readonly coverageAmount: number;

  @IsOptional()
  @IsString()
  readonly notes?: string;
}

// Clean controller with DTOs
@Controller('vendors')
export class VendorController {
  constructor(
    private readonly commandBus: CommandBus,
    private readonly queryBus: QueryBus,
  ) {}

  @Post()
  @HttpCode(HttpStatus.CREATED)
  async createVendor(@Body() dto: CreateVendorDto): Promise<{ id: string; message: string }> {
    const command = new CreateVendorCommand(
      dto.organizationId,
      dto.name,
      dto.code,
      dto.contactInfo,
      dto.insurance,
      dto.notes,
    );

    const vendorId = await this.commandBus.execute(command);

    return {
      id: vendorId,
      message: 'Vendor created successfully',
    };
  }

  @Get(':id')
  async getVendor(@Param('id') id: string): Promise<VendorDetailDto> {
    const query = new GetVendorDetailQuery(id);
    return await this.queryBus.execute(query);
  }

  @Get()
  async getVendors(@Query() filters: VendorFiltersDto): Promise<VendorListDto[]> {
    const query = new GetVendorsQuery(filters);
    return await this.queryBus.execute(query);
  }

  @Put(':id')
  async updateVendor(
    @Param('id') id: string,
    @Body() dto: UpdateVendorDto,
  ): Promise<{ message: string }> {
    const command = new UpdateVendorCommand(
      id,
      dto.name,
      dto.contactInfo,
      dto.insurance,
      dto.notes,
    );

    await this.commandBus.execute(command);

    return { message: 'Vendor updated successfully' };
  }
}
```

## DTO Mapping and Transformation

### Manual Mapping

```typescript
export class VendorDtoMapper {
  static toDto(vendor: Vendor): VendorDto {
    return new VendorDto({
      id: vendor.getId().getValue(),
      organizationId: vendor.getOrganizationId(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      contactInfo: ContactInfoDtoMapper.toDto(vendor.getContactInfo()),
      insurance: InsuranceDtoMapper.toDto(vendor.getInsurance()),
      createdAt: vendor.getCreatedAt(),
      updatedAt: vendor.getUpdatedAt(),
      notes: vendor.getNotes(),
    });
  }

  static toListDto(
    vendor: Vendor,
    additionalData?: {
      lastActivity?: Date;
      activeContracts?: number;
    },
  ): VendorListDto {
    return new VendorListDto({
      id: vendor.getId().getValue(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      lastActivity: additionalData?.lastActivity ?? vendor.getUpdatedAt(),
      activeContracts: additionalData?.activeContracts ?? 0,
    });
  }

  static toDetailDto(
    vendor: Vendor,
    additionalData: {
      paymentHistory: PaymentSummaryDto[];
      contracts: ContractSummaryDto[];
      documents: DocumentSummaryDto[];
      complianceStatus: ComplianceStatusDto;
    },
  ): VendorDetailDto {
    const baseDto = this.toDto(vendor);

    return new VendorDetailDto({
      ...baseDto,
      ...additionalData,
    });
  }

  static fromCreateDto(dto: CreateVendorDto): CreateVendorCommand {
    return new CreateVendorCommand(
      dto.organizationId,
      dto.name,
      dto.code,
      ContactInfoDtoMapper.fromDto(dto.contactInfo),
      InsuranceDtoMapper.fromDto(dto.insurance),
      dto.notes,
    );
  }
}

export class ContactInfoDtoMapper {
  static toDto(contactInfo: ContactInfo): ContactInfoDto {
    return {
      email: contactInfo.getEmail().getValue(),
      phone: contactInfo.getPhone()?.getValue(),
      address: contactInfo.getAddress()
        ? AddressDtoMapper.toDto(contactInfo.getAddress()!)
        : undefined,
      contactPerson: contactInfo.getContactPerson(),
    };
  }

  static fromDto(dto: ContactInfoDto): ContactInfoData {
    return {
      email: dto.email,
      phone: dto.phone,
      address: dto.address ? AddressDtoMapper.fromDto(dto.address) : undefined,
      contactPerson: dto.contactPerson,
    };
  }
}
```

### Automated Mapping with Libraries

```typescript
// Using class-transformer for automatic mapping
import { plainToInstance, instanceToPlain, Transform, Type } from 'class-transformer';

export class VendorDto {
  id: string;
  organizationId: string;
  name: string;
  code: string;
  status: string;

  @Type(() => ContactInfoDto)
  contactInfo: ContactInfoDto;

  @Type(() => InsuranceDto)
  insurance: InsuranceDto;

  @Transform(({ value }) => value?.toISOString())
  createdAt: Date;

  @Transform(({ value }) => value?.toISOString())
  updatedAt: Date;

  notes?: string;

  // Factory method for creating from domain entity
  static fromEntity(vendor: Vendor): VendorDto {
    return plainToInstance(VendorDto, {
      id: vendor.getId().getValue(),
      organizationId: vendor.getOrganizationId(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      contactInfo: vendor.getContactInfo().toPlain(),
      insurance: vendor.getInsurance().toPlain(),
      createdAt: vendor.getCreatedAt(),
      updatedAt: vendor.getUpdatedAt(),
      notes: vendor.getNotes(),
    });
  }
}

// Usage in query handler
@QueryHandler(GetVendorByIdQuery)
export class GetVendorByIdHandler implements IQueryHandler<GetVendorByIdQuery> {
  constructor(private readonly vendorRepository: IVendorRepository) {}

  async execute(query: GetVendorByIdQuery): Promise<VendorDto> {
    const vendor = await this.vendorRepository.findById(new VendorId(query.vendorId));

    if (!vendor) {
      throw new VendorNotFoundError(query.vendorId);
    }

    return VendorDto.fromEntity(vendor);
  }
}
```

## Specialized DTOs for Different Contexts

### API Version-Specific DTOs

```typescript
// V1 API DTO
export class VendorDtoV1 {
  readonly id: string;
  readonly name: string;
  readonly code: string;
  readonly status: string;
  readonly email: string;
  readonly phone?: string;

  static fromEntity(vendor: Vendor): VendorDtoV1 {
    return {
      id: vendor.getId().getValue(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      email: vendor.getContactInfo().getEmail().getValue(),
      phone: vendor.getContactInfo().getPhone()?.getValue(),
    };
  }
}

// V2 API DTO with enhanced structure
export class VendorDtoV2 {
  readonly id: string;
  readonly name: string;
  readonly code: string;
  readonly status: string;
  readonly contactInfo: ContactInfoDto;
  readonly insurance: InsuranceDto;
  readonly metadata: VendorMetadataDto;

  static fromEntity(vendor: Vendor): VendorDtoV2 {
    return {
      id: vendor.getId().getValue(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      contactInfo: ContactInfoDtoMapper.toDto(vendor.getContactInfo()),
      insurance: InsuranceDtoMapper.toDto(vendor.getInsurance()),
      metadata: {
        createdAt: vendor.getCreatedAt(),
        updatedAt: vendor.getUpdatedAt(),
        version: vendor.getVersion(),
      },
    };
  }
}
```

### Context-Specific DTOs

```typescript
// Dashboard summary DTO
export class VendorDashboardDto {
  readonly totalVendors: number;
  readonly activeVendors: number;
  readonly pendingVendors: number;
  readonly expiringInsurance: VendorInsuranceExpiryDto[];
  readonly recentActivity: VendorActivityDto[];
  readonly topVendorsByPayments: VendorPaymentSummaryDto[];
}

// Export/Reporting DTO
export class VendorExportDto {
  readonly id: string;
  readonly organizationId: string;
  readonly name: string;
  readonly code: string;
  readonly status: string;
  readonly email: string;
  readonly phone: string;
  readonly address: string;
  readonly insuranceProvider: string;
  readonly insuranceExpiry: string;
  readonly totalPaid: number;
  readonly lastPaymentDate: string;
  readonly contractCount: number;
  readonly complianceScore: number;

  // Flattened structure for CSV/Excel export
  static fromEntity(vendor: Vendor, aggregateData: VendorAggregateData): VendorExportDto {
    const contactInfo = vendor.getContactInfo();
    const insurance = vendor.getInsurance();
    const address = contactInfo.getAddress();

    return {
      id: vendor.getId().getValue(),
      organizationId: vendor.getOrganizationId(),
      name: vendor.getName().getValue(),
      code: vendor.getCode().getValue(),
      status: vendor.getStatus().getValue(),
      email: contactInfo.getEmail().getValue(),
      phone: contactInfo.getPhone()?.getValue() || '',
      address: address
        ? `${address.getStreet()}, ${address.getCity()}, ${address.getState()} ${address.getZipCode()}`
        : '',
      insuranceProvider: insurance.getProvider(),
      insuranceExpiry: insurance.getExpiryDate().toISOString(),
      totalPaid: aggregateData.totalPaid,
      lastPaymentDate: aggregateData.lastPaymentDate?.toISOString() || '',
      contractCount: aggregateData.contractCount,
      complianceScore: aggregateData.complianceScore,
    };
  }
}

// Integration DTO for external systems
export class VendorIntegrationDto {
  readonly vendorId: string;
  readonly organizationCode: string;
  readonly vendorCode: string;
  readonly companyName: string;
  readonly taxId?: string;
  readonly paymentTerms: string;
  readonly bankingInfo?: BankingInfoDto;
  readonly preferredCommunication: 'EMAIL' | 'PHONE' | 'POSTAL';

  // Structure optimized for ERP integration
  static fromEntity(vendor: Vendor): VendorIntegrationDto {
    return {
      vendorId: vendor.getId().getValue(),
      organizationCode: vendor.getOrganizationId(),
      vendorCode: vendor.getCode().getValue(),
      companyName: vendor.getName().getValue(),
      taxId: vendor.getTaxId()?.getValue(),
      paymentTerms: vendor.getPaymentTerms().getValue(),
      bankingInfo: vendor.getBankingInfo()
        ? BankingInfoDtoMapper.toDto(vendor.getBankingInfo()!)
        : undefined,
      preferredCommunication: vendor.getPreferredCommunication(),
    };
  }
}
```

## DTO Validation and Error Handling

### Custom Validation Decorators

```typescript
import { registerDecorator, ValidationOptions, ValidationArguments } from 'class-validator';

// Custom validator for vendor codes
export function IsValidVendorCode(validationOptions?: ValidationOptions) {
  return function (object: Object, propertyName: string) {
    registerDecorator({
      name: 'isValidVendorCode',
      target: object.constructor,
      propertyName: propertyName,
      options: validationOptions,
      validator: {
        validate(value: any, args: ValidationArguments) {
          if (typeof value !== 'string') return false;
          return VendorCode.isValidFormat(value);
        },
        defaultMessage(args: ValidationArguments) {
          return `${args.property} must be a valid vendor code format`;
        },
      },
    });
  };
}

// Custom validator for insurance dates
export function IsInsuranceValid(validationOptions?: ValidationOptions) {
  return function (object: Object, propertyName: string) {
    registerDecorator({
      name: 'isInsuranceValid',
      target: object.constructor,
      propertyName: propertyName,
      options: validationOptions,
      validator: {
        validate(value: InsuranceDto, args: ValidationArguments) {
          const expiryDate = new Date(value.expiryDate);
          const now = new Date();

          // Insurance must not be expired
          if (expiryDate <= now) {
            return false;
          }

          // Coverage amount must be reasonable
          if (value.coverageAmount < 100000) {
            return false;
          }

          return true;
        },
        defaultMessage(args: ValidationArguments) {
          return `Insurance must be valid with future expiry date and adequate coverage`;
        },
      },
    });
  };
}

// Usage in DTO
export class CreateVendorDto {
  @IsNotEmpty()
  @IsString()
  @MaxLength(255)
  readonly name: string;

  @IsNotEmpty()
  @IsValidVendorCode()
  readonly code: string;

  @ValidateNested()
  @Type(() => InsuranceDto)
  @IsInsuranceValid()
  readonly insurance: InsuranceDto;
}
```

### Validation Error Handling

```typescript
@Injectable()
export class DtoValidationPipe implements PipeTransform {
  async transform(value: any, metadata: ArgumentMetadata): Promise<any> {
    if (!metadata.metatype || !this.toValidate(metadata.metatype)) {
      return value;
    }

    const object = plainToInstance(metadata.metatype, value);
    const errors = await validate(object, {
      whitelist: true, // Strip unknown properties
      forbidNonWhitelisted: true, // Throw error for unknown properties
      transform: true, // Transform types
      validateCustomDecorators: true, // Validate custom decorators
    });

    if (errors.length > 0) {
      throw new BadRequestException(this.formatErrors(errors));
    }

    return object;
  }

  private toValidate(metatype: any): boolean {
    const types = [String, Boolean, Number, Array, Object];
    return !types.includes(metatype);
  }

  private formatErrors(errors: ValidationError[]): any {
    return {
      message: 'Validation failed',
      errors: this.flattenErrors(errors),
    };
  }

  private flattenErrors(errors: ValidationError[]): any {
    return errors.reduce((acc, error) => {
      if (error.children && error.children.length > 0) {
        acc[error.property] = this.flattenErrors(error.children);
      } else {
        acc[error.property] = Object.values(error.constraints || {});
      }
      return acc;
    }, {} as any);
  }
}
```

## Complex DTO Scenarios

### Lease Operating Statement DTOs

```typescript
export class CreateLosDto {
  @IsNotEmpty()
  @IsUUID()
  readonly leaseId: string;

  @IsNumber()
  @Min(2000)
  @Max(new Date().getFullYear())
  readonly year: number;

  @IsNumber()
  @Min(1)
  @Max(12)
  readonly month: number;

  @IsOptional()
  @IsString()
  @MaxLength(1000)
  readonly notes?: string;
}

export class AddLosExpenseDto {
  @IsNotEmpty()
  @IsString()
  @MaxLength(255)
  readonly description: string;

  @IsEnum(ExpenseCategory)
  readonly category: ExpenseCategory;

  @IsEnum(ExpenseType)
  readonly type: ExpenseType;

  @IsNumber()
  @Min(0.01)
  @Transform(({ value }) => Math.round(value * 100) / 100) // Round to 2 decimal places
  readonly amount: number;

  @IsOptional()
  @IsString()
  @Length(3, 3)
  readonly currency?: string = 'USD';

  @IsOptional()
  @IsString()
  @MaxLength(255)
  readonly vendorName?: string;

  @IsOptional()
  @IsString()
  @MaxLength(100)
  readonly invoiceNumber?: string;

  @IsOptional()
  @IsDateString()
  readonly invoiceDate?: string;

  @IsOptional()
  @IsString()
  @MaxLength(500)
  readonly notes?: string;
}

export class LosDetailDto {
  readonly id: string;
  readonly organizationId: string;
  readonly leaseId: string;
  readonly statementMonth: string;
  readonly displayMonth: string;
  readonly totalExpenses: number;
  readonly operatingExpenses: number;
  readonly capitalExpenses: number;
  readonly status: string;
  readonly notes?: string;
  readonly expenseLineItems: ExpenseLineItemDto[];
  readonly createdAt: Date;
  readonly updatedAt: Date;
  readonly version: number;

  static fromEntity(los: LeaseOperatingStatement): LosDetailDto {
    return {
      id: los.getId().getValue(),
      organizationId: los.getOrganizationId(),
      leaseId: los.getLeaseId(),
      statementMonth: los.getStatementMonth().toString(),
      displayMonth: los.getStatementMonth().toDisplayString(),
      totalExpenses: los.getTotalExpenses().getAmount(),
      operatingExpenses: los.getOperatingExpenses().getAmount(),
      capitalExpenses: los.getCapitalExpenses().getAmount(),
      status: los.getStatus().getValue(),
      notes: los.getNotes(),
      expenseLineItems: los
        .getExpenseLineItems()
        .map((item) => ExpenseLineItemDto.fromEntity(item)),
      createdAt: los.getCreatedAt(),
      updatedAt: los.getUpdatedAt(),
      version: los.getVersion(),
    };
  }
}

export class ExpenseLineItemDto {
  readonly id: string;
  readonly description: string;
  readonly category: string;
  readonly type: string;
  readonly amount: number;
  readonly currency: string;
  readonly vendorName?: string;
  readonly invoiceNumber?: string;
  readonly invoiceDate?: Date;
  readonly notes?: string;

  static fromEntity(item: ExpenseLineItem): ExpenseLineItemDto {
    return {
      id: item.getId().getValue(),
      description: item.getDescription(),
      category: item.getCategory().getValue(),
      type: item.getType().getValue(),
      amount: item.getAmount().getAmount(),
      currency: item.getAmount().getCurrency(),
      vendorName: item.getVendorName(),
      invoiceNumber: item.getInvoiceNumber(),
      invoiceDate: item.getInvoiceDate(),
      notes: item.getNotes(),
    };
  }
}
```

## Testing DTOs

### DTO Validation Testing

```typescript
describe('CreateVendorDto', () => {
  let validDto: CreateVendorDto;

  beforeEach(() => {
    validDto = {
      name: 'Test Vendor',
      code: 'TEST-01',
      organizationId: 'org-123',
      contactInfo: {
        email: 'test@vendor.com',
        phone: '+1234567890',
      },
      insurance: {
        provider: 'Test Insurance Co',
        policyNumber: 'POL-123456',
        expiryDate: '2025-12-31',
        coverageAmount: 1000000,
      },
    };
  });

  describe('validation', () => {
    it('should pass with valid data', async () => {
      const dto = plainToInstance(CreateVendorDto, validDto);
      const errors = await validate(dto);
      expect(errors).toHaveLength(0);
    });

    it('should fail with empty name', async () => {
      const invalidDto = { ...validDto, name: '' };
      const dto = plainToInstance(CreateVendorDto, invalidDto);
      const errors = await validate(dto);

      expect(errors).toHaveLength(1);
      expect(errors[0].property).toBe('name');
      expect(errors[0].constraints).toHaveProperty('isNotEmpty');
    });

    it('should fail with invalid vendor code format', async () => {
      const invalidDto = { ...validDto, code: 'invalid code!' };
      const dto = plainToInstance(CreateVendorDto, invalidDto);
      const errors = await validate(dto);

      expect(errors).toHaveLength(1);
      expect(errors[0].property).toBe('code');
    });

    it('should validate nested objects', async () => {
      const invalidDto = {
        ...validDto,
        contactInfo: { ...validDto.contactInfo, email: 'invalid-email' },
      };
      const dto = plainToInstance(CreateVendorDto, invalidDto);
      const errors = await validate(dto);

      expect(errors).toHaveLength(1);
      expect(errors[0].property).toBe('contactInfo');
      expect(errors[0].children).toBeDefined();
    });
  });

  describe('transformation', () => {
    it('should transform string numbers to numbers', async () => {
      const inputData = {
        ...validDto,
        insurance: {
          ...validDto.insurance,
          coverageAmount: '1000000', // String input
        },
      };

      const dto = plainToInstance(CreateVendorDto, inputData, {
        transform: true,
      });

      expect(typeof dto.insurance.coverageAmount).toBe('number');
      expect(dto.insurance.coverageAmount).toBe(1000000);
    });

    it('should strip unknown properties', async () => {
      const inputData = {
        ...validDto,
        unknownProperty: 'should be removed',
      };

      const dto = plainToInstance(CreateVendorDto, inputData, {
        excludeExtraneousValues: true,
      });

      expect((dto as any).unknownProperty).toBeUndefined();
    });
  });
});
```

### DTO Mapping Testing

```typescript
describe('VendorDtoMapper', () => {
  let vendor: Vendor;

  beforeEach(() => {
    vendor = createTestVendor();
  });

  describe('toDto', () => {
    it('should map vendor entity to DTO correctly', () => {
      const dto = VendorDtoMapper.toDto(vendor);

      expect(dto.id).toBe(vendor.getId().getValue());
      expect(dto.name).toBe(vendor.getName().getValue());
      expect(dto.code).toBe(vendor.getCode().getValue());
      expect(dto.status).toBe(vendor.getStatus().getValue());
      expect(dto.contactInfo.email).toBe(vendor.getContactInfo().getEmail().getValue());
    });

    it('should handle optional fields correctly', () => {
      const vendorWithoutNotes = createTestVendorWithoutNotes();
      const dto = VendorDtoMapper.toDto(vendorWithoutNotes);

      expect(dto.notes).toBeUndefined();
    });
  });

  describe('toListDto', () => {
    it('should create list DTO with additional data', () => {
      const additionalData = {
        lastActivity: new Date(),
        activeContracts: 5,
      };

      const dto = VendorDtoMapper.toListDto(vendor, additionalData);

      expect(dto.id).toBe(vendor.getId().getValue());
      expect(dto.activeContracts).toBe(5);
      expect(dto.lastActivity).toBe(additionalData.lastActivity);
    });
  });
});
```

## Best Practices

### 1. Immutability

```typescript
// Good: Immutable DTO
export class VendorDto {
  readonly id: string;
  readonly name: string;

  constructor(data: { id: string; name: string }) {
    this.id = data.id;
    this.name = data.name;
  }
}

// Avoid: Mutable DTO
export class VendorDto {
  id: string;
  name: string;
}
```

### 2. Specific DTOs for Different Operations

```typescript
// Good: Operation-specific DTOs
export class CreateVendorDto {
  readonly name: string;
  readonly code: string;
  // Only fields needed for creation
}

export class UpdateVendorDto {
  readonly name?: string;
  readonly contactInfo?: ContactInfoDto;
  // Only fields that can be updated
}

// Avoid: Generic DTO for all operations
export class VendorDto {
  id?: string; // Sometimes present, sometimes not
  name?: string; // All fields optional
  code?: string; // Confusing for consumers
}
```

### 3. Clear Validation Messages

```typescript
export class CreateVendorDto {
  @IsNotEmpty({ message: 'Vendor name is required' })
  @MaxLength(255, { message: 'Vendor name cannot exceed 255 characters' })
  readonly name: string;

  @IsNotEmpty({ message: 'Vendor code is required' })
  @Matches(/^[A-Za-z0-9_-]{3,20}$/, {
    message: 'Vendor code must be 3-20 characters, alphanumeric with hyphens or underscores only',
  })
  readonly code: string;
}
```

The DTO pattern in our oil & gas management system provides clean contracts
between our API and clients, ensuring data integrity and security while
maintaining flexibility for different use cases and API versions.
