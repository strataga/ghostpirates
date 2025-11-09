# Anti-Corruption Layer Pattern

## Overview

The Anti-Corruption Layer pattern acts as a translation layer between different
bounded contexts or external systems. It prevents the external model from
"corrupting" your domain model by translating between different representations
and protecting your domain from changes in external systems.

## Core Concepts

### Translation Layer

Converts data and behavior between your domain model and external systems.

### Domain Protection

Shields your domain model from external changes and inconsistencies.

### Interface Adaptation

Adapts external interfaces to match your domain's needs.

### Data Transformation

Transforms external data formats to domain-appropriate structures.

## Benefits

- **Domain Integrity**: Protects domain model from external changes
- **Isolation**: Isolates your system from external system complexities
- **Flexibility**: Allows independent evolution of internal and external models
- **Consistency**: Maintains consistent domain language and concepts
- **Testability**: External dependencies can be easily mocked
- **Maintainability**: Changes to external systems are contained

## Implementation in Our Project

### Before: Direct External System Integration

```typescript
@Injectable()
export class VendorService {
  constructor(
    private readonly vendorRepository: VendorRepository,
    private readonly erpApiClient: ErpApiClient,
    private readonly bankingApiClient: BankingApiClient,
  ) {}

  async createVendor(vendorData: CreateVendorDto): Promise<Vendor> {
    // Direct usage of external API models in domain logic
    const vendor = new Vendor();
    vendor.name = vendorData.name;
    vendor.code = vendorData.code;

    await this.vendorRepository.save(vendor);

    // Direct integration with ERP system using their data format
    try {
      await this.erpApiClient.createVendor({
        VendorID: vendor.id, // ERP uses different field names
        CompanyName: vendor.name,
        VendorCode: vendor.code,
        Status: 'ACTIVE', // ERP expects specific status values
        CreatedDate: new Date().toISOString(),
        // ERP requires additional fields we don't track
        TaxID: '',
        PaymentTerms: 'NET30',
        CreditLimit: 0,
      });
    } catch (error) {
      // ERP failures affect our domain logic
      console.error('Failed to sync with ERP', error);
      // Should we rollback our vendor creation?
    }

    // Direct integration with banking system
    if (vendorData.bankingInfo) {
      try {
        await this.bankingApiClient.registerPayee({
          payee_id: vendor.id, // Banking API uses snake_case
          payee_name: vendor.name,
          account_number: vendorData.bankingInfo.accountNumber,
          routing_number: vendorData.bankingInfo.routingNumber,
          bank_name: vendorData.bankingInfo.bankName,
          // Banking system expects different validation format
          account_type: this.mapAccountType(vendorData.bankingInfo.accountType),
        });
      } catch (error) {
        console.error('Failed to register payee with bank', error);
      }
    }

    return vendor;
  }

  async updateVendorInsurance(vendorId: string, insurance: InsuranceData): Promise<void> {
    const vendor = await this.vendorRepository.findById(vendorId);
    vendor.updateInsurance(insurance);
    await this.vendorRepository.save(vendor);

    // Direct integration with compliance system
    try {
      await this.complianceApiClient.updateInsuranceRecord({
        EntityId: vendorId,
        EntityType: 'VENDOR',
        InsuranceProvider: insurance.provider,
        PolicyNumber: insurance.policyNumber,
        EffectiveDate: insurance.effectiveDate,
        ExpirationDate: insurance.expiryDate,
        CoverageAmount: insurance.coverageAmount,
        // Compliance system requires different data structure
        CoverageTypes: this.mapCoverageTypes(insurance.coverageTypes),
        ComplianceStatus: this.calculateComplianceStatus(insurance),
      });
    } catch (error) {
      console.error('Failed to update compliance record', error);
    }
  }

  // Scattered mapping logic throughout the service
  private mapAccountType(accountType: string): string {
    const mapping = {
      CHECKING: 'CHK',
      SAVINGS: 'SAV',
      BUSINESS_CHECKING: 'BUS_CHK',
    };
    return mapping[accountType] || 'CHK';
  }

  private mapCoverageTypes(coverageTypes: string[]): any[] {
    // Complex mapping logic scattered in service
    return coverageTypes.map((type) => ({
      Type: type,
      Required: true,
      MinimumAmount: this.getMinimumCoverage(type),
    }));
  }

  private calculateComplianceStatus(insurance: InsuranceData): string {
    // Business logic mixed with external system requirements
    if (new Date(insurance.expiryDate) <= new Date()) {
      return 'NON_COMPLIANT';
    }
    return 'COMPLIANT';
  }
}
```

### After: Anti-Corruption Layer Implementation

```typescript
// Anti-Corruption Layer Interfaces
export interface IErpAdapter {
  createVendor(vendor: Vendor): Promise<ErpVendorId>;
  updateVendor(vendor: Vendor): Promise<void>;
  deleteVendor(vendorId: VendorId): Promise<void>;
  syncVendorStatus(vendorId: VendorId, status: VendorStatus): Promise<void>;
}

export interface IBankingAdapter {
  registerPayee(vendor: Vendor, bankingInfo: BankingInfo): Promise<BankingPayeeId>;
  updatePayeeInfo(vendor: Vendor, bankingInfo: BankingInfo): Promise<void>;
  removePayee(vendorId: VendorId): Promise<void>;
}

export interface IComplianceAdapter {
  updateInsuranceRecord(vendor: Vendor): Promise<void>;
  checkComplianceStatus(vendorId: VendorId): Promise<ComplianceStatus>;
  getRequiredDocuments(vendorType: VendorType): Promise<DocumentRequirement[]>;
}

// ERP Anti-Corruption Layer
@Injectable()
export class ErpAdapter implements IErpAdapter {
  constructor(
    private readonly erpApiClient: ErpApiClient,
    private readonly logger: ILogger,
  ) {}

  async createVendor(vendor: Vendor): Promise<ErpVendorId> {
    try {
      const erpVendorRequest = this.translateVendorToErpFormat(vendor);
      const response = await this.erpApiClient.createVendor(erpVendorRequest);
      return new ErpVendorId(response.VendorID);
    } catch (error) {
      this.logger.error('Failed to create vendor in ERP', {
        vendorId: vendor.getId().getValue(),
        error: error.message,
      });
      throw new ErpIntegrationError('Failed to create vendor in ERP', error);
    }
  }

  async updateVendor(vendor: Vendor): Promise<void> {
    try {
      const erpVendorRequest = this.translateVendorToErpFormat(vendor);
      await this.erpApiClient.updateVendor(vendor.getId().getValue(), erpVendorRequest);
    } catch (error) {
      this.logger.error('Failed to update vendor in ERP', {
        vendorId: vendor.getId().getValue(),
        error: error.message,
      });
      throw new ErpIntegrationError('Failed to update vendor in ERP', error);
    }
  }

  async syncVendorStatus(vendorId: VendorId, status: VendorStatus): Promise<void> {
    try {
      const erpStatus = this.translateStatusToErpFormat(status);
      await this.erpApiClient.updateVendorStatus(vendorId.getValue(), erpStatus);
    } catch (error) {
      this.logger.error('Failed to sync vendor status with ERP', {
        vendorId: vendorId.getValue(),
        status: status.getValue(),
        error: error.message,
      });
      throw new ErpIntegrationError('Failed to sync vendor status with ERP', error);
    }
  }

  // Translation methods - encapsulated within the ACL
  private translateVendorToErpFormat(vendor: Vendor): ErpVendorRequest {
    return {
      VendorID: vendor.getId().getValue(),
      CompanyName: vendor.getName().getValue(),
      VendorCode: vendor.getCode().getValue(),
      Status: this.translateStatusToErpFormat(vendor.getStatus()),
      CreatedDate: vendor.getCreatedAt().toISOString(),
      ModifiedDate: vendor.getUpdatedAt().toISOString(),
      // ERP-specific required fields with sensible defaults
      TaxID: vendor.getTaxId()?.getValue() || '',
      PaymentTerms: this.translatePaymentTermsToErp(vendor.getPaymentTerms()),
      CreditLimit: vendor.getCreditLimit()?.getAmount() || 0,
      ContactInfo: this.translateContactInfoToErp(vendor.getContactInfo()),
      // Additional ERP fields that don't exist in our domain
      AccountingCategory: 'VENDOR',
      DefaultExpenseAccount: '2000', // Standard vendor expense account
      RequiresPO: true, // Default ERP setting
    };
  }

  private translateStatusToErpFormat(status: VendorStatus): string {
    const statusMapping = {
      [VendorStatus.PENDING]: 'PENDING_APPROVAL',
      [VendorStatus.ACTIVE]: 'ACTIVE',
      [VendorStatus.INACTIVE]: 'INACTIVE',
      [VendorStatus.BLOCKED]: 'HOLD',
      [VendorStatus.SUSPENDED]: 'SUSPENDED',
    };

    return statusMapping[status.getValue()] || 'PENDING_APPROVAL';
  }

  private translatePaymentTermsToErp(paymentTerms?: PaymentTerms): string {
    if (!paymentTerms) return 'NET30';

    const termsMapping = {
      NET_15: 'NET15',
      NET_30: 'NET30',
      NET_45: 'NET45',
      NET_60: 'NET60',
      IMMEDIATE: 'IMMEDIATE',
      COD: 'COD',
    };

    return termsMapping[paymentTerms.getValue()] || 'NET30';
  }

  private translateContactInfoToErp(contactInfo: ContactInfo): ErpContactInfo {
    return {
      PrimaryContact: contactInfo.getContactPerson() || '',
      EmailAddress: contactInfo.getEmail().getValue(),
      PhoneNumber: contactInfo.getPhone()?.getValue() || '',
      Address: contactInfo.getAddress()
        ? {
            Street1: contactInfo.getAddress()!.getStreet(),
            City: contactInfo.getAddress()!.getCity(),
            State: contactInfo.getAddress()!.getState(),
            ZipCode: contactInfo.getAddress()!.getZipCode(),
            Country: contactInfo.getAddress()!.getCountry(),
          }
        : undefined,
    };
  }
}

// Banking Anti-Corruption Layer
@Injectable()
export class BankingAdapter implements IBankingAdapter {
  constructor(
    private readonly bankingApiClient: BankingApiClient,
    private readonly logger: ILogger,
  ) {}

  async registerPayee(vendor: Vendor, bankingInfo: BankingInfo): Promise<BankingPayeeId> {
    try {
      const bankingRequest = this.translateVendorToBankingFormat(vendor, bankingInfo);
      const response = await this.bankingApiClient.registerPayee(bankingRequest);
      return new BankingPayeeId(response.payee_id);
    } catch (error) {
      this.logger.error('Failed to register payee with banking system', {
        vendorId: vendor.getId().getValue(),
        error: error.message,
      });
      throw new BankingIntegrationError('Failed to register payee', error);
    }
  }

  async updatePayeeInfo(vendor: Vendor, bankingInfo: BankingInfo): Promise<void> {
    try {
      const bankingRequest = this.translateVendorToBankingFormat(vendor, bankingInfo);
      await this.bankingApiClient.updatePayee(vendor.getId().getValue(), bankingRequest);
    } catch (error) {
      this.logger.error('Failed to update payee info', {
        vendorId: vendor.getId().getValue(),
        error: error.message,
      });
      throw new BankingIntegrationError('Failed to update payee info', error);
    }
  }

  private translateVendorToBankingFormat(
    vendor: Vendor,
    bankingInfo: BankingInfo,
  ): BankingPayeeRequest {
    return {
      payee_id: vendor.getId().getValue(),
      payee_name: vendor.getName().getValue(),
      payee_type: 'VENDOR',
      account_number: bankingInfo.getAccountNumber().getValue(),
      routing_number: bankingInfo.getRoutingNumber().getValue(),
      bank_name: bankingInfo.getBankName(),
      account_type: this.translateAccountTypeToBanking(bankingInfo.getAccountType()),
      // Banking-specific requirements
      verification_status: 'PENDING',
      payment_methods: ['ACH', 'WIRE'],
      notification_preferences: {
        email: vendor.getContactInfo().getEmail().getValue(),
        sms: vendor.getContactInfo().getPhone()?.getValue(),
      },
    };
  }

  private translateAccountTypeToBanking(accountType: BankAccountType): string {
    const typeMapping = {
      [BankAccountType.CHECKING]: 'CHK',
      [BankAccountType.SAVINGS]: 'SAV',
      [BankAccountType.BUSINESS_CHECKING]: 'BUS_CHK',
      [BankAccountType.MONEY_MARKET]: 'MM',
    };

    return typeMapping[accountType] || 'CHK';
  }
}

// Compliance Anti-Corruption Layer
@Injectable()
export class ComplianceAdapter implements IComplianceAdapter {
  constructor(
    private readonly complianceApiClient: ComplianceApiClient,
    private readonly logger: ILogger,
  ) {}

  async updateInsuranceRecord(vendor: Vendor): Promise<void> {
    try {
      const complianceRequest = this.translateVendorToComplianceFormat(vendor);
      await this.complianceApiClient.updateEntity(complianceRequest);
    } catch (error) {
      this.logger.error('Failed to update compliance record', {
        vendorId: vendor.getId().getValue(),
        error: error.message,
      });
      throw new ComplianceIntegrationError('Failed to update compliance record', error);
    }
  }

  async checkComplianceStatus(vendorId: VendorId): Promise<ComplianceStatus> {
    try {
      const response = await this.complianceApiClient.getEntityStatus(vendorId.getValue());
      return this.translateComplianceStatusFromExternal(response);
    } catch (error) {
      this.logger.error('Failed to check compliance status', {
        vendorId: vendorId.getValue(),
        error: error.message,
      });
      throw new ComplianceIntegrationError('Failed to check compliance status', error);
    }
  }

  private translateVendorToComplianceFormat(vendor: Vendor): ComplianceEntityRequest {
    const insurance = vendor.getInsurance();

    return {
      EntityId: vendor.getId().getValue(),
      EntityType: 'VENDOR',
      EntityName: vendor.getName().getValue(),
      EntityCode: vendor.getCode().getValue(),
      OrganizationId: vendor.getOrganizationId(),
      InsuranceInfo: insurance
        ? {
            Provider: insurance.getProvider(),
            PolicyNumber: insurance.getPolicyNumber(),
            EffectiveDate: insurance.getEffectiveDate().toISOString(),
            ExpirationDate: insurance.getExpiryDate().toISOString(),
            CoverageAmount: insurance.getCoverageAmount().getAmount(),
            CoverageTypes: this.translateCoverageTypesToCompliance(insurance.getCoverageTypes()),
            IsActive: insurance.isValid() && !insurance.isExpired(),
          }
        : null,
      ComplianceRequirements: this.getComplianceRequirementsForVendor(vendor),
      LastUpdated: vendor.getUpdatedAt().toISOString(),
    };
  }

  private translateCoverageTypesToCompliance(
    coverageTypes: InsuranceCoverageType[],
  ): ComplianceCoverageType[] {
    return coverageTypes.map((type) => ({
      Type: this.mapCoverageTypeToCompliance(type),
      Required: true,
      MinimumAmount: this.getMinimumCoverageAmount(type),
      Status: 'ACTIVE',
    }));
  }

  private translateComplianceStatusFromExternal(externalStatus: any): ComplianceStatus {
    // Translate external compliance status to our domain status
    const statusMapping = {
      COMPLIANT: ComplianceStatus.COMPLIANT,
      NON_COMPLIANT: ComplianceStatus.NON_COMPLIANT,
      PENDING_REVIEW: ComplianceStatus.UNDER_REVIEW,
      EXPIRED: ComplianceStatus.EXPIRED,
      GRACE_PERIOD: ComplianceStatus.GRACE_PERIOD,
    };

    const status = statusMapping[externalStatus.Status] || ComplianceStatus.UNKNOWN;

    return new ComplianceStatus(status, {
      expirationDate: externalStatus.ExpirationDate
        ? new Date(externalStatus.ExpirationDate)
        : undefined,
      issues: externalStatus.Issues || [],
      lastReviewDate: externalStatus.LastReviewDate
        ? new Date(externalStatus.LastReviewDate)
        : undefined,
    });
  }

  private mapCoverageTypeToCompliance(coverageType: InsuranceCoverageType): string {
    const typeMapping = {
      [InsuranceCoverageType.GENERAL_LIABILITY]: 'GL',
      [InsuranceCoverageType.WORKERS_COMPENSATION]: 'WC',
      [InsuranceCoverageType.PROFESSIONAL_LIABILITY]: 'PL',
      [InsuranceCoverageType.COMMERCIAL_AUTO]: 'AUTO',
    };

    return typeMapping[coverageType] || 'OTHER';
  }

  private getMinimumCoverageAmount(coverageType: InsuranceCoverageType): number {
    const minimums = {
      [InsuranceCoverageType.GENERAL_LIABILITY]: 1000000,
      [InsuranceCoverageType.WORKERS_COMPENSATION]: 500000,
      [InsuranceCoverageType.PROFESSIONAL_LIABILITY]: 500000,
      [InsuranceCoverageType.COMMERCIAL_AUTO]: 1000000,
    };

    return minimums[coverageType] || 500000;
  }

  private getComplianceRequirementsForVendor(vendor: Vendor): string[] {
    const requirements = ['INSURANCE_CURRENT'];

    if (vendor.isInternational()) {
      requirements.push('INTERNATIONAL_COMPLIANCE');
    }

    if (vendor.isHighValue()) {
      requirements.push('ENHANCED_DUE_DILIGENCE');
    }

    return requirements;
  }
}

// Clean Domain Service using Anti-Corruption Layers
@Injectable()
export class VendorService {
  constructor(
    private readonly vendorRepository: IVendorRepository,
    private readonly erpAdapter: IErpAdapter,
    private readonly bankingAdapter: IBankingAdapter,
    private readonly complianceAdapter: IComplianceAdapter,
    private readonly eventBus: EventBus,
  ) {}

  async createVendor(command: CreateVendorCommand): Promise<string> {
    // Pure domain logic - no external system concerns
    const vendor = Vendor.create({
      organizationId: command.organizationId,
      name: command.name,
      code: command.code,
      contactInfo: command.contactInfo,
      insurance: command.insurance,
    });

    await this.vendorRepository.save(vendor);

    // Publish domain event - external integrations handled by event listeners
    const event = new VendorCreatedEvent(
      vendor.getId().getValue(),
      vendor.getName().getValue(),
      vendor.getCode().getValue(),
      vendor.getOrganizationId(),
    );

    this.eventBus.publish(event);

    return vendor.getId().getValue();
  }

  async updateVendorInsurance(command: UpdateVendorInsuranceCommand): Promise<void> {
    const vendor = await this.vendorRepository.findById(new VendorId(command.vendorId));

    if (!vendor) {
      throw new VendorNotFoundError(command.vendorId);
    }

    vendor.updateInsurance(new Insurance(command.insurance));
    await this.vendorRepository.save(vendor);

    // Publish domain event
    const event = new VendorInsuranceUpdatedEvent(
      vendor.getId().getValue(),
      vendor.getInsurance()!,
    );

    this.eventBus.publish(event);
  }
}

// Event Handlers for External System Integration
@Injectable()
export class VendorExternalIntegrationHandler {
  constructor(
    private readonly erpAdapter: IErpAdapter,
    private readonly bankingAdapter: IBankingAdapter,
    private readonly complianceAdapter: IComplianceAdapter,
    private readonly vendorRepository: IVendorRepository,
  ) {}

  @EventsHandler(VendorCreatedEvent)
  async handleVendorCreated(event: VendorCreatedEvent): Promise<void> {
    const vendor = await this.vendorRepository.findById(new VendorId(event.vendorId));

    if (!vendor) return;

    // Each integration is isolated and can fail independently
    await Promise.allSettled([
      this.syncWithErp(vendor),
      this.registerWithBanking(vendor),
      this.updateComplianceRecord(vendor),
    ]);
  }

  @EventsHandler(VendorInsuranceUpdatedEvent)
  async handleVendorInsuranceUpdated(event: VendorInsuranceUpdatedEvent): Promise<void> {
    const vendor = await this.vendorRepository.findById(new VendorId(event.vendorId));

    if (!vendor) return;

    // Only compliance system needs to know about insurance updates
    try {
      await this.complianceAdapter.updateInsuranceRecord(vendor);
    } catch (error) {
      // Log error but don't fail the domain operation
      console.error('Failed to update compliance record after insurance update', error);
    }
  }

  private async syncWithErp(vendor: Vendor): Promise<void> {
    try {
      await this.erpAdapter.createVendor(vendor);
    } catch (error) {
      console.error('Failed to sync vendor with ERP', error);
      // Could queue for retry, send alert, etc.
    }
  }

  private async registerWithBanking(vendor: Vendor): Promise<void> {
    const bankingInfo = vendor.getBankingInfo();
    if (!bankingInfo) return;

    try {
      await this.bankingAdapter.registerPayee(vendor, bankingInfo);
    } catch (error) {
      console.error('Failed to register vendor with banking system', error);
    }
  }

  private async updateComplianceRecord(vendor: Vendor): Promise<void> {
    try {
      await this.complianceAdapter.updateInsuranceRecord(vendor);
    } catch (error) {
      console.error('Failed to update compliance record', error);
    }
  }
}
```

## Advanced Anti-Corruption Layer Patterns

### Generic Anti-Corruption Layer

```typescript
// Generic ACL interface for reusability
export interface IAntiCorruptionLayer<TDomain, TExternal> {
  toDomain(external: TExternal): TDomain;
  toExternal(domain: TDomain): TExternal;
}

// Generic ACL implementation
export abstract class BaseAntiCorruptionLayer<TDomain, TExternal>
  implements IAntiCorruptionLayer<TDomain, TExternal>
{
  abstract toDomain(external: TExternal): TDomain;
  abstract toExternal(domain: TDomain): TExternal;

  protected handleTranslationError(error: Error, context: string): never {
    throw new TranslationError(`Failed to translate ${context}: ${error.message}`, error);
  }

  protected validateDomainObject(obj: TDomain): void {
    if (!obj) {
      throw new ValidationError('Domain object is required');
    }
  }

  protected validateExternalObject(obj: TExternal): void {
    if (!obj) {
      throw new ValidationError('External object is required');
    }
  }
}

// Specific implementation
@Injectable()
export class VendorErpAntiCorruptionLayer extends BaseAntiCorruptionLayer<Vendor, ErpVendor> {
  toDomain(erpVendor: ErpVendor): Vendor {
    this.validateExternalObject(erpVendor);

    try {
      return Vendor.fromErpData({
        id: erpVendor.VendorID,
        name: erpVendor.CompanyName,
        code: erpVendor.VendorCode,
        status: this.mapErpStatusToDomain(erpVendor.Status),
        contactInfo: this.mapErpContactInfoToDomain(erpVendor.ContactInfo),
        createdAt: new Date(erpVendor.CreatedDate),
        updatedAt: new Date(erpVendor.ModifiedDate),
      });
    } catch (error) {
      this.handleTranslationError(error, 'ERP vendor to domain');
    }
  }

  toExternal(vendor: Vendor): ErpVendor {
    this.validateDomainObject(vendor);

    try {
      return {
        VendorID: vendor.getId().getValue(),
        CompanyName: vendor.getName().getValue(),
        VendorCode: vendor.getCode().getValue(),
        Status: this.mapDomainStatusToErp(vendor.getStatus()),
        ContactInfo: this.mapDomainContactInfoToErp(vendor.getContactInfo()),
        CreatedDate: vendor.getCreatedAt().toISOString(),
        ModifiedDate: vendor.getUpdatedAt().toISOString(),
        // ERP-specific fields with defaults
        TaxID: vendor.getTaxId()?.getValue() || '',
        PaymentTerms: 'NET30',
        CreditLimit: 0,
      };
    } catch (error) {
      this.handleTranslationError(error, 'domain vendor to ERP');
    }
  }

  private mapErpStatusToDomain(erpStatus: string): VendorStatus {
    const statusMapping = {
      PENDING_APPROVAL: VendorStatus.PENDING,
      ACTIVE: VendorStatus.ACTIVE,
      INACTIVE: VendorStatus.INACTIVE,
      HOLD: VendorStatus.BLOCKED,
      SUSPENDED: VendorStatus.SUSPENDED,
    };

    const domainStatus = statusMapping[erpStatus];
    if (!domainStatus) {
      throw new TranslationError(`Unknown ERP status: ${erpStatus}`);
    }

    return domainStatus;
  }

  private mapDomainStatusToErp(status: VendorStatus): string {
    const statusMapping = {
      [VendorStatus.PENDING]: 'PENDING_APPROVAL',
      [VendorStatus.ACTIVE]: 'ACTIVE',
      [VendorStatus.INACTIVE]: 'INACTIVE',
      [VendorStatus.BLOCKED]: 'HOLD',
      [VendorStatus.SUSPENDED]: 'SUSPENDED',
    };

    return statusMapping[status.getValue()] || 'PENDING_APPROVAL';
  }
}
```

### Configuration-Driven ACL

```typescript
// Configuration for field mappings
export interface FieldMapping {
  domainField: string;
  externalField: string;
  transformer?: (value: any) => any;
  required?: boolean;
  defaultValue?: any;
}

export interface AclConfiguration {
  entityName: string;
  fieldMappings: FieldMapping[];
  customTransformations: { [key: string]: (obj: any) => any };
}

// Configuration-driven ACL
@Injectable()
export class ConfigurableAntiCorruptionLayer {
  constructor(
    @Inject('ACL_CONFIGURATIONS')
    private readonly configurations: Map<string, AclConfiguration>,
  ) {}

  toDomain<T>(entityType: string, externalData: any): T {
    const config = this.configurations.get(entityType);
    if (!config) {
      throw new Error(`No ACL configuration found for entity type: ${entityType}`);
    }

    const domainData: any = {};

    for (const mapping of config.fieldMappings) {
      const externalValue = this.getNestedValue(externalData, mapping.externalField);

      if (externalValue !== undefined && externalValue !== null) {
        const transformedValue = mapping.transformer
          ? mapping.transformer(externalValue)
          : externalValue;

        this.setNestedValue(domainData, mapping.domainField, transformedValue);
      } else if (mapping.required) {
        if (mapping.defaultValue !== undefined) {
          this.setNestedValue(domainData, mapping.domainField, mapping.defaultValue);
        } else {
          throw new TranslationError(`Required field ${mapping.externalField} is missing`);
        }
      }
    }

    // Apply custom transformations
    for (const [transformName, transform] of Object.entries(config.customTransformations)) {
      domainData[transformName] = transform(externalData);
    }

    return domainData as T;
  }

  toExternal<T>(entityType: string, domainData: any): T {
    const config = this.configurations.get(entityType);
    if (!config) {
      throw new Error(`No ACL configuration found for entity type: ${entityType}`);
    }

    const externalData: any = {};

    for (const mapping of config.fieldMappings) {
      const domainValue = this.getNestedValue(domainData, mapping.domainField);

      if (domainValue !== undefined && domainValue !== null) {
        const transformedValue = mapping.transformer
          ? this.reverseTransform(mapping.transformer, domainValue)
          : domainValue;

        this.setNestedValue(externalData, mapping.externalField, transformedValue);
      }
    }

    return externalData as T;
  }

  private getNestedValue(obj: any, path: string): any {
    return path.split('.').reduce((current, key) => current?.[key], obj);
  }

  private setNestedValue(obj: any, path: string, value: any): void {
    const keys = path.split('.');
    const lastKey = keys.pop()!;
    const target = keys.reduce((current, key) => {
      if (!(key in current)) {
        current[key] = {};
      }
      return current[key];
    }, obj);

    target[lastKey] = value;
  }

  private reverseTransform(transformer: (value: any) => any, value: any): any {
    // This is a simplified reverse transformation
    // In practice, you'd need proper reverse transformers
    return value;
  }
}

// Configuration setup
const vendorAclConfig: AclConfiguration = {
  entityName: 'Vendor',
  fieldMappings: [
    {
      domainField: 'id',
      externalField: 'VendorID',
      required: true,
    },
    {
      domainField: 'name',
      externalField: 'CompanyName',
      required: true,
    },
    {
      domainField: 'code',
      externalField: 'VendorCode',
      required: true,
    },
    {
      domainField: 'status',
      externalField: 'Status',
      transformer: (value: string) => {
        const mapping = {
          PENDING_APPROVAL: 'PENDING',
          ACTIVE: 'ACTIVE',
          INACTIVE: 'INACTIVE',
        };
        return mapping[value] || 'PENDING';
      },
    },
    {
      domainField: 'contactInfo.email',
      externalField: 'ContactInfo.EmailAddress',
      required: false,
    },
  ],
  customTransformations: {
    createdAt: (obj: any) => new Date(obj.CreatedDate),
    updatedAt: (obj: any) => new Date(obj.ModifiedDate),
  },
};
```

## Testing Anti-Corruption Layers

### ACL Testing

```typescript
describe('ErpAdapter', () => {
  let erpAdapter: ErpAdapter;
  let mockErpApiClient: jest.Mocked<ErpApiClient>;

  beforeEach(() => {
    mockErpApiClient = {
      createVendor: jest.fn(),
      updateVendor: jest.fn(),
      updateVendorStatus: jest.fn(),
    };

    erpAdapter = new ErpAdapter(mockErpApiClient, mockLogger);
  });

  describe('createVendor', () => {
    it('should translate domain vendor to ERP format correctly', async () => {
      // Given
      const vendor = createTestVendor({
        name: 'Test Vendor',
        code: 'TEST-01',
        status: VendorStatus.ACTIVE,
      });

      mockErpApiClient.createVendor.mockResolvedValue({
        VendorID: 'ERP-123',
        Status: 'SUCCESS',
      });

      // When
      await erpAdapter.createVendor(vendor);

      // Then
      expect(mockErpApiClient.createVendor).toHaveBeenCalledWith({
        VendorID: vendor.getId().getValue(),
        CompanyName: 'Test Vendor',
        VendorCode: 'TEST-01',
        Status: 'ACTIVE',
        CreatedDate: expect.any(String),
        ModifiedDate: expect.any(String),
        TaxID: '',
        PaymentTerms: 'NET30',
        CreditLimit: 0,
        ContactInfo: expect.any(Object),
        AccountingCategory: 'VENDOR',
        DefaultExpenseAccount: '2000',
        RequiresPO: true,
      });
    });

    it('should handle ERP API errors gracefully', async () => {
      // Given
      const vendor = createTestVendor();
      mockErpApiClient.createVendor.mockRejectedValue(new Error('ERP service unavailable'));

      // When/Then
      await expect(erpAdapter.createVendor(vendor)).rejects.toThrow(ErpIntegrationError);
    });
  });

  describe('translation methods', () => {
    it('should translate vendor status correctly', () => {
      expect(erpAdapter['translateStatusToErpFormat'](VendorStatus.PENDING)).toBe(
        'PENDING_APPROVAL',
      );
      expect(erpAdapter['translateStatusToErpFormat'](VendorStatus.ACTIVE)).toBe('ACTIVE');
      expect(erpAdapter['translateStatusToErpFormat'](VendorStatus.BLOCKED)).toBe('HOLD');
    });

    it('should translate payment terms correctly', () => {
      expect(erpAdapter['translatePaymentTermsToErp'](PaymentTerms.NET_15)).toBe('NET15');
      expect(erpAdapter['translatePaymentTermsToErp'](PaymentTerms.NET_30)).toBe('NET30');
      expect(erpAdapter['translatePaymentTermsToErp'](undefined)).toBe('NET30');
    });
  });
});

describe('BankingAdapter', () => {
  let bankingAdapter: BankingAdapter;
  let mockBankingApiClient: jest.Mocked<BankingApiClient>;

  beforeEach(() => {
    mockBankingApiClient = {
      registerPayee: jest.fn(),
      updatePayee: jest.fn(),
    };

    bankingAdapter = new BankingAdapter(mockBankingApiClient, mockLogger);
  });

  describe('registerPayee', () => {
    it('should translate vendor to banking format correctly', async () => {
      // Given
      const vendor = createTestVendor();
      const bankingInfo = createTestBankingInfo();

      mockBankingApiClient.registerPayee.mockResolvedValue({
        payee_id: 'BANK-123',
        status: 'REGISTERED',
      });

      // When
      await bankingAdapter.registerPayee(vendor, bankingInfo);

      // Then
      expect(mockBankingApiClient.registerPayee).toHaveBeenCalledWith({
        payee_id: vendor.getId().getValue(),
        payee_name: vendor.getName().getValue(),
        payee_type: 'VENDOR',
        account_number: bankingInfo.getAccountNumber().getValue(),
        routing_number: bankingInfo.getRoutingNumber().getValue(),
        bank_name: bankingInfo.getBankName(),
        account_type: 'CHK',
        verification_status: 'PENDING',
        payment_methods: ['ACH', 'WIRE'],
        notification_preferences: expect.any(Object),
      });
    });

    it('should handle banking API errors', async () => {
      // Given
      const vendor = createTestVendor();
      const bankingInfo = createTestBankingInfo();
      mockBankingApiClient.registerPayee.mockRejectedValue(new Error('Banking service error'));

      // When/Then
      await expect(bankingAdapter.registerPayee(vendor, bankingInfo)).rejects.toThrow(
        BankingIntegrationError,
      );
    });
  });
});
```

## Best Practices

### 1. Single Responsibility per ACL

```typescript
// Good: Separate ACLs for different external systems
class ErpAntiCorruptionLayer {
  // Only handles ERP translation
}

class BankingAntiCorruptionLayer {
  // Only handles banking translation
}

// Avoid: One ACL handling multiple systems
class ExternalSystemAntiCorruptionLayer {
  translateToErp() {
    /* ... */
  }
  translateToBanking() {
    /* ... */
  }
  translateToCompliance() {
    /* ... */
  }
}
```

### 2. Error Handling and Logging

```typescript
// Good: Proper error handling in ACL
async createVendor(vendor: Vendor): Promise<ErpVendorId> {
  try {
    const erpRequest = this.translateToErp(vendor);
    const response = await this.erpClient.create(erpRequest);
    return new ErpVendorId(response.id);
  } catch (error) {
    this.logger.error('ERP vendor creation failed', {
      vendorId: vendor.getId().getValue(),
      error: error.message,
      stackTrace: error.stack
    });
    throw new ErpIntegrationError('Failed to create vendor in ERP', error);
  }
}
```

### 3. Bidirectional Translation

```typescript
// Good: Support both directions
interface IAntiCorruptionLayer<TDomain, TExternal> {
  toDomain(external: TExternal): TDomain;
  toExternal(domain: TDomain): TExternal;

  // Optional: bulk operations
  toDomainBulk?(external: TExternal[]): TDomain[];
  toExternalBulk?(domain: TDomain[]): TExternal[];
}
```

The Anti-Corruption Layer pattern in our oil & gas management system protects
our domain model from external system complexities, provides translation between
different data formats, and allows independent evolution of internal and
external models while maintaining clean separation of concerns.
