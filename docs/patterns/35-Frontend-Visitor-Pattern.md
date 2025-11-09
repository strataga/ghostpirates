# Frontend Visitor Pattern

## Overview

The Visitor Pattern in React applications allows you to define operations on a
family of related objects without modifying their classes. This pattern is
particularly valuable for processing different types of data structures,
implementing complex business logic across various entity types, and creating
flexible data transformation pipelines in oil & gas applications.

## Problem Statement

Complex frontend applications often need to:

- **Process different entity types** (wells, leases, equipment) with similar
  operations
- **Apply business rules** across various data structures
- **Transform data** for different output formats (reports, exports,
  visualizations)
- **Validate complex hierarchies** of related objects
- **Calculate metrics** across different entity types

Traditional approaches lead to:

- **Scattered business logic** across multiple classes
- **Tight coupling** between operations and data structures
- **Difficult maintenance** when adding new operations
- **Code duplication** for similar operations on different types
- **Complex conditional logic** based on object types

## Solution

Implement the Visitor Pattern to separate operations from the objects they
operate on, allowing you to add new operations without modifying existing
classes and organize related operations together.

## Implementation

### Base Visitor Interface

```typescript
// lib/visitors/interfaces.ts
export interface Visitor<TResult = any> {
  visitWell(well: Well): TResult;
  visitLease(lease: Lease): TResult;
  visitEquipment(equipment: Equipment): TResult;
  visitProduction(production: Production): TResult;
  visitOperator(operator: Operator): TResult;
}

export interface Visitable {
  accept<TResult>(visitor: Visitor<TResult>): TResult;
}

export interface VisitorContext {
  user: User;
  permissions: string[];
  options: Record<string, any>;
  metadata: Record<string, any>;
}

export interface ProcessingResult<T> {
  success: boolean;
  data?: T;
  errors: ProcessingError[];
  warnings: ProcessingWarning[];
  metadata: Record<string, any>;
}

export interface ProcessingError {
  entityType: string;
  entityId: string;
  message: string;
  code: string;
  severity: 'error' | 'warning';
}

export interface ProcessingWarning {
  entityType: string;
  entityId: string;
  message: string;
  code: string;
}
```

### Visitable Entity Base

```typescript
// lib/visitors/visitable-entity.ts
export abstract class VisitableEntity implements Visitable {
  abstract accept<TResult>(visitor: Visitor<TResult>): TResult;
}

// Enhanced entity classes
export class Well extends VisitableEntity {
  constructor(
    public id: string,
    public name: string,
    public apiNumber: string,
    public operatorId: string,
    public location: Coordinates,
    public wellType: WellType,
    public status: WellStatus,
    public totalDepth: number,
    public spudDate: Date,
    public completionDate?: Date,
    public production?: Production[],
    public equipment?: Equipment[],
  ) {
    super();
  }

  accept<TResult>(visitor: Visitor<TResult>): TResult {
    return visitor.visitWell(this);
  }
}

export class Lease extends VisitableEntity {
  constructor(
    public id: string,
    public name: string,
    public operatorId: string,
    public legalDescription: string,
    public acreage: number,
    public leaseType: LeaseType,
    public effectiveDate: Date,
    public expirationDate: Date,
    public wells?: Well[],
    public royaltyRate?: number,
  ) {
    super();
  }

  accept<TResult>(visitor: Visitor<TResult>): TResult {
    return visitor.visitLease(this);
  }
}

export class Equipment extends VisitableEntity {
  constructor(
    public id: string,
    public name: string,
    public type: EquipmentType,
    public wellId: string,
    public manufacturer: string,
    public model: string,
    public installDate: Date,
    public status: EquipmentStatus,
    public specifications: Record<string, any>,
    public maintenanceHistory?: MaintenanceRecord[],
  ) {
    super();
  }

  accept<TResult>(visitor: Visitor<TResult>): TResult {
    return visitor.visitEquipment(this);
  }
}

export class Production extends VisitableEntity {
  constructor(
    public id: string,
    public wellId: string,
    public date: Date,
    public oil: number,
    public gas: number,
    public water: number,
    public testType: TestType,
    public duration: number,
    public pressure?: number,
    public temperature?: number,
  ) {
    super();
  }

  accept<TResult>(visitor: Visitor<TResult>): TResult {
    return visitor.visitProduction(this);
  }
}

export class Operator extends VisitableEntity {
  constructor(
    public id: string,
    public name: string,
    public operatorNumber: string,
    public address: Address,
    public contactInfo: ContactInfo,
    public licenseNumber: string,
    public licenseExpiration: Date,
    public wells?: Well[],
    public leases?: Lease[],
  ) {
    super();
  }

  accept<TResult>(visitor: Visitor<TResult>): TResult {
    return visitor.visitOperator(this);
  }
}
```

### Abstract Base Visitor

```typescript
// lib/visitors/base-visitor.ts
export abstract class BaseVisitor<TResult> implements Visitor<TResult> {
  protected context: VisitorContext;
  protected errors: ProcessingError[] = [];
  protected warnings: ProcessingWarning[] = [];

  constructor(context: VisitorContext) {
    this.context = context;
  }

  abstract visitWell(well: Well): TResult;
  abstract visitLease(lease: Lease): TResult;
  abstract visitEquipment(equipment: Equipment): TResult;
  abstract visitProduction(production: Production): TResult;
  abstract visitOperator(operator: Operator): TResult;

  // Utility methods
  protected addError(entityType: string, entityId: string, message: string, code: string): void {
    this.errors.push({
      entityType,
      entityId,
      message,
      code,
      severity: 'error',
    });
  }

  protected addWarning(entityType: string, entityId: string, message: string, code: string): void {
    this.warnings.push({
      entityType,
      entityId,
      message,
      code,
    });
  }

  protected hasPermission(permission: string): boolean {
    return this.context.permissions.includes(permission) || this.context.permissions.includes('*');
  }

  protected getOption<T>(key: string, defaultValue?: T): T {
    return this.context.options[key] ?? defaultValue;
  }

  getProcessingResult(data?: TResult): ProcessingResult<TResult> {
    return {
      success: this.errors.length === 0,
      data,
      errors: [...this.errors],
      warnings: [...this.warnings],
      metadata: { ...this.context.metadata },
    };
  }

  clearErrors(): void {
    this.errors = [];
    this.warnings = [];
  }
}
```

### Validation Visitor

```typescript
// lib/visitors/validation-visitor.ts
export interface ValidationResult {
  isValid: boolean;
  entityId: string;
  entityType: string;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export class ValidationVisitor extends BaseVisitor<ValidationResult> {
  visitWell(well: Well): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // API Number validation
    if (!well.apiNumber || !/^\d{14}$/.test(well.apiNumber)) {
      errors.push({
        field: 'apiNumber',
        message: 'API number must be exactly 14 digits',
        code: 'INVALID_API_NUMBER',
        severity: 'error',
      });
    }

    // Well name validation
    if (!well.name || well.name.trim().length === 0) {
      errors.push({
        field: 'name',
        message: 'Well name is required',
        code: 'MISSING_WELL_NAME',
        severity: 'error',
      });
    }

    // Depth validation
    if (well.totalDepth <= 0) {
      errors.push({
        field: 'totalDepth',
        message: 'Total depth must be greater than 0',
        code: 'INVALID_DEPTH',
        severity: 'error',
      });
    } else if (well.totalDepth > 50000) {
      warnings.push({
        field: 'totalDepth',
        message: 'Total depth exceeds typical maximum (50,000 ft)',
        code: 'UNUSUAL_DEPTH',
      });
    }

    // Date validation
    if (well.completionDate && well.completionDate < well.spudDate) {
      errors.push({
        field: 'completionDate',
        message: 'Completion date cannot be before spud date',
        code: 'INVALID_DATE_SEQUENCE',
        severity: 'error',
      });
    }

    // Location validation
    if (!well.location || !this.isValidCoordinates(well.location)) {
      errors.push({
        field: 'location',
        message: 'Valid coordinates are required',
        code: 'INVALID_COORDINATES',
        severity: 'error',
      });
    }

    return {
      isValid: errors.length === 0,
      entityId: well.id,
      entityType: 'Well',
      errors,
      warnings,
    };
  }

  visitLease(lease: Lease): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Lease name validation
    if (!lease.name || lease.name.trim().length === 0) {
      errors.push({
        field: 'name',
        message: 'Lease name is required',
        code: 'MISSING_LEASE_NAME',
        severity: 'error',
      });
    }

    // Legal description validation
    if (!lease.legalDescription || lease.legalDescription.trim().length < 10) {
      errors.push({
        field: 'legalDescription',
        message: 'Legal description must be at least 10 characters',
        code: 'INVALID_LEGAL_DESCRIPTION',
        severity: 'error',
      });
    }

    // Acreage validation
    if (lease.acreage <= 0) {
      errors.push({
        field: 'acreage',
        message: 'Acreage must be greater than 0',
        code: 'INVALID_ACREAGE',
        severity: 'error',
      });
    } else if (lease.acreage > 10000) {
      warnings.push({
        field: 'acreage',
        message: 'Acreage is unusually large (>10,000 acres)',
        code: 'LARGE_ACREAGE',
      });
    }

    // Date validation
    if (lease.expirationDate <= lease.effectiveDate) {
      errors.push({
        field: 'expirationDate',
        message: 'Expiration date must be after effective date',
        code: 'INVALID_LEASE_DATES',
        severity: 'error',
      });
    }

    // Expiration warning
    const daysUntilExpiration = Math.ceil(
      (lease.expirationDate.getTime() - Date.now()) / (1000 * 60 * 60 * 24),
    );
    if (daysUntilExpiration < 90 && daysUntilExpiration > 0) {
      warnings.push({
        field: 'expirationDate',
        message: `Lease expires in ${daysUntilExpiration} days`,
        code: 'LEASE_EXPIRING_SOON',
      });
    }

    return {
      isValid: errors.length === 0,
      entityId: lease.id,
      entityType: 'Lease',
      errors,
      warnings,
    };
  }

  visitEquipment(equipment: Equipment): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Equipment name validation
    if (!equipment.name || equipment.name.trim().length === 0) {
      errors.push({
        field: 'name',
        message: 'Equipment name is required',
        code: 'MISSING_EQUIPMENT_NAME',
        severity: 'error',
      });
    }

    // Manufacturer validation
    if (!equipment.manufacturer || equipment.manufacturer.trim().length === 0) {
      errors.push({
        field: 'manufacturer',
        message: 'Manufacturer is required',
        code: 'MISSING_MANUFACTURER',
        severity: 'error',
      });
    }

    // Install date validation
    if (equipment.installDate > new Date()) {
      errors.push({
        field: 'installDate',
        message: 'Install date cannot be in the future',
        code: 'FUTURE_INSTALL_DATE',
        severity: 'error',
      });
    }

    // Age warning
    const ageInYears = (Date.now() - equipment.installDate.getTime()) / (1000 * 60 * 60 * 24 * 365);
    if (ageInYears > 20) {
      warnings.push({
        field: 'installDate',
        message: `Equipment is ${Math.floor(ageInYears)} years old`,
        code: 'OLD_EQUIPMENT',
      });
    }

    // Status validation
    if (equipment.status === 'failed' && !equipment.maintenanceHistory?.length) {
      warnings.push({
        field: 'status',
        message: 'Failed equipment should have maintenance history',
        code: 'MISSING_MAINTENANCE_HISTORY',
      });
    }

    return {
      isValid: errors.length === 0,
      entityId: equipment.id,
      entityType: 'Equipment',
      errors,
      warnings,
    };
  }

  visitProduction(production: Production): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Production values validation
    if (production.oil < 0 || production.gas < 0 || production.water < 0) {
      errors.push({
        field: 'production',
        message: 'Production values cannot be negative',
        code: 'NEGATIVE_PRODUCTION',
        severity: 'error',
      });
    }

    // Zero production warning
    if (production.oil === 0 && production.gas === 0) {
      warnings.push({
        field: 'production',
        message: 'No oil or gas production recorded',
        code: 'ZERO_PRODUCTION',
      });
    }

    // High water cut warning
    const totalLiquid = production.oil + production.water;
    if (totalLiquid > 0 && production.water / totalLiquid > 0.9) {
      warnings.push({
        field: 'water',
        message: 'High water cut (>90%)',
        code: 'HIGH_WATER_CUT',
      });
    }

    // Date validation
    if (production.date > new Date()) {
      errors.push({
        field: 'date',
        message: 'Production date cannot be in the future',
        code: 'FUTURE_PRODUCTION_DATE',
        severity: 'error',
      });
    }

    // Test duration validation
    if (production.duration <= 0) {
      errors.push({
        field: 'duration',
        message: 'Test duration must be greater than 0',
        code: 'INVALID_DURATION',
        severity: 'error',
      });
    }

    return {
      isValid: errors.length === 0,
      entityId: production.id,
      entityType: 'Production',
      errors,
      warnings,
    };
  }

  visitOperator(operator: Operator): ValidationResult {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Operator name validation
    if (!operator.name || operator.name.trim().length === 0) {
      errors.push({
        field: 'name',
        message: 'Operator name is required',
        code: 'MISSING_OPERATOR_NAME',
        severity: 'error',
      });
    }

    // Operator number validation
    if (!operator.operatorNumber || !/^\d+$/.test(operator.operatorNumber)) {
      errors.push({
        field: 'operatorNumber',
        message: 'Valid operator number is required',
        code: 'INVALID_OPERATOR_NUMBER',
        severity: 'error',
      });
    }

    // License validation
    if (!operator.licenseNumber) {
      errors.push({
        field: 'licenseNumber',
        message: 'License number is required',
        code: 'MISSING_LICENSE',
        severity: 'error',
      });
    }

    // License expiration validation
    if (operator.licenseExpiration <= new Date()) {
      errors.push({
        field: 'licenseExpiration',
        message: 'Operator license has expired',
        code: 'EXPIRED_LICENSE',
        severity: 'error',
      });
    } else {
      const daysUntilExpiration = Math.ceil(
        (operator.licenseExpiration.getTime() - Date.now()) / (1000 * 60 * 60 * 24),
      );
      if (daysUntilExpiration < 90) {
        warnings.push({
          field: 'licenseExpiration',
          message: `License expires in ${daysUntilExpiration} days`,
          code: 'LICENSE_EXPIRING_SOON',
        });
      }
    }

    return {
      isValid: errors.length === 0,
      entityId: operator.id,
      entityType: 'Operator',
      errors,
      warnings,
    };
  }

  private isValidCoordinates(location: Coordinates): boolean {
    return (
      location.latitude >= -90 &&
      location.latitude <= 90 &&
      location.longitude >= -180 &&
      location.longitude <= 180
    );
  }
}
```

### Export Visitor

```typescript
// lib/visitors/export-visitor.ts
export interface ExportData {
  type: string;
  id: string;
  data: Record<string, any>;
  metadata: Record<string, any>;
}

export class ExportVisitor extends BaseVisitor<ExportData> {
  private exportFormat: 'json' | 'csv' | 'xml';

  constructor(context: VisitorContext, exportFormat: 'json' | 'csv' | 'xml' = 'json') {
    super(context);
    this.exportFormat = exportFormat;
  }

  visitWell(well: Well): ExportData {
    const data = {
      id: well.id,
      name: well.name,
      apiNumber: well.apiNumber,
      operatorId: well.operatorId,
      location: {
        latitude: well.location.latitude,
        longitude: well.location.longitude,
      },
      wellType: well.wellType,
      status: well.status,
      totalDepth: well.totalDepth,
      spudDate: well.spudDate.toISOString(),
      completionDate: well.completionDate?.toISOString(),
      productionCount: well.production?.length || 0,
      equipmentCount: well.equipment?.length || 0,
    };

    return {
      type: 'Well',
      id: well.id,
      data,
      metadata: {
        exportFormat: this.exportFormat,
        exportedAt: new Date().toISOString(),
        exportedBy: this.context.user.id,
      },
    };
  }

  visitLease(lease: Lease): ExportData {
    const data = {
      id: lease.id,
      name: lease.name,
      operatorId: lease.operatorId,
      legalDescription: lease.legalDescription,
      acreage: lease.acreage,
      leaseType: lease.leaseType,
      effectiveDate: lease.effectiveDate.toISOString(),
      expirationDate: lease.expirationDate.toISOString(),
      royaltyRate: lease.royaltyRate,
      wellCount: lease.wells?.length || 0,
    };

    return {
      type: 'Lease',
      id: lease.id,
      data,
      metadata: {
        exportFormat: this.exportFormat,
        exportedAt: new Date().toISOString(),
        exportedBy: this.context.user.id,
      },
    };
  }

  visitEquipment(equipment: Equipment): ExportData {
    const data = {
      id: equipment.id,
      name: equipment.name,
      type: equipment.type,
      wellId: equipment.wellId,
      manufacturer: equipment.manufacturer,
      model: equipment.model,
      installDate: equipment.installDate.toISOString(),
      status: equipment.status,
      specifications: equipment.specifications,
      maintenanceCount: equipment.maintenanceHistory?.length || 0,
    };

    return {
      type: 'Equipment',
      id: equipment.id,
      data,
      metadata: {
        exportFormat: this.exportFormat,
        exportedAt: new Date().toISOString(),
        exportedBy: this.context.user.id,
      },
    };
  }

  visitProduction(production: Production): ExportData {
    const data = {
      id: production.id,
      wellId: production.wellId,
      date: production.date.toISOString(),
      oil: production.oil,
      gas: production.gas,
      water: production.water,
      testType: production.testType,
      duration: production.duration,
      pressure: production.pressure,
      temperature: production.temperature,
    };

    return {
      type: 'Production',
      id: production.id,
      data,
      metadata: {
        exportFormat: this.exportFormat,
        exportedAt: new Date().toISOString(),
        exportedBy: this.context.user.id,
      },
    };
  }

  visitOperator(operator: Operator): ExportData {
    const data = {
      id: operator.id,
      name: operator.name,
      operatorNumber: operator.operatorNumber,
      address: operator.address,
      contactInfo: operator.contactInfo,
      licenseNumber: operator.licenseNumber,
      licenseExpiration: operator.licenseExpiration.toISOString(),
      wellCount: operator.wells?.length || 0,
      leaseCount: operator.leases?.length || 0,
    };

    return {
      type: 'Operator',
      id: operator.id,
      data,
      metadata: {
        exportFormat: this.exportFormat,
        exportedAt: new Date().toISOString(),
        exportedBy: this.context.user.id,
      },
    };
  }
}
```

### Calculation Visitor

```typescript
// lib/visitors/calculation-visitor.ts
export interface CalculationResult {
  entityType: string;
  entityId: string;
  calculations: Record<string, number>;
  metadata: Record<string, any>;
}

export class CalculationVisitor extends BaseVisitor<CalculationResult> {
  visitWell(well: Well): CalculationResult {
    const calculations: Record<string, number> = {};

    // Calculate total production if available
    if (well.production && well.production.length > 0) {
      calculations.totalOilProduction = well.production.reduce((sum, p) => sum + p.oil, 0);
      calculations.totalGasProduction = well.production.reduce((sum, p) => sum + p.gas, 0);
      calculations.totalWaterProduction = well.production.reduce((sum, p) => sum + p.water, 0);
      calculations.averageOilRate = calculations.totalOilProduction / well.production.length;
      calculations.averageGasRate = calculations.totalGasProduction / well.production.length;
      calculations.waterCut =
        calculations.totalWaterProduction /
        (calculations.totalOilProduction + calculations.totalWaterProduction);
    }

    // Calculate well age
    const ageInDays = (Date.now() - well.spudDate.getTime()) / (1000 * 60 * 60 * 24);
    calculations.ageInDays = Math.floor(ageInDays);
    calculations.ageInYears = Math.floor(ageInDays / 365);

    // Calculate equipment count
    calculations.equipmentCount = well.equipment?.length || 0;

    return {
      entityType: 'Well',
      entityId: well.id,
      calculations,
      metadata: {
        calculatedAt: new Date().toISOString(),
        calculatedBy: this.context.user.id,
      },
    };
  }

  visitLease(lease: Lease): CalculationResult {
    const calculations: Record<string, number> = {};

    // Calculate lease duration
    const durationMs = lease.expirationDate.getTime() - lease.effectiveDate.getTime();
    calculations.durationInDays = Math.floor(durationMs / (1000 * 60 * 60 * 24));
    calculations.durationInYears = Math.floor(calculations.durationInDays / 365);

    // Calculate remaining time
    const remainingMs = lease.expirationDate.getTime() - Date.now();
    calculations.remainingDays = Math.max(0, Math.floor(remainingMs / (1000 * 60 * 60 * 24)));

    // Calculate well density
    calculations.wellCount = lease.wells?.length || 0;
    calculations.wellDensity = calculations.wellCount / lease.acreage; // wells per acre

    // Calculate total production from all wells
    if (lease.wells && lease.wells.length > 0) {
      let totalOil = 0;
      let totalGas = 0;
      let totalWater = 0;

      lease.wells.forEach((well) => {
        if (well.production) {
          totalOil += well.production.reduce((sum, p) => sum + p.oil, 0);
          totalGas += well.production.reduce((sum, p) => sum + p.gas, 0);
          totalWater += well.production.reduce((sum, p) => sum + p.water, 0);
        }
      });

      calculations.totalOilProduction = totalOil;
      calculations.totalGasProduction = totalGas;
      calculations.totalWaterProduction = totalWater;
      calculations.productionPerAcre = totalOil / lease.acreage;
    }

    return {
      entityType: 'Lease',
      entityId: lease.id,
      calculations,
      metadata: {
        calculatedAt: new Date().toISOString(),
        calculatedBy: this.context.user.id,
      },
    };
  }

  visitEquipment(equipment: Equipment): CalculationResult {
    const calculations: Record<string, number> = {};

    // Calculate equipment age
    const ageInDays = (Date.now() - equipment.installDate.getTime()) / (1000 * 60 * 60 * 24);
    calculations.ageInDays = Math.floor(ageInDays);
    calculations.ageInYears = Math.floor(ageInDays / 365);

    // Calculate maintenance frequency
    if (equipment.maintenanceHistory && equipment.maintenanceHistory.length > 0) {
      calculations.maintenanceCount = equipment.maintenanceHistory.length;
      calculations.averageMaintenanceInterval = ageInDays / equipment.maintenanceHistory.length;

      // Calculate time since last maintenance
      const lastMaintenance = equipment.maintenanceHistory.sort(
        (a, b) => b.date.getTime() - a.date.getTime(),
      )[0];
      const daysSinceLastMaintenance =
        (Date.now() - lastMaintenance.date.getTime()) / (1000 * 60 * 60 * 24);
      calculations.daysSinceLastMaintenance = Math.floor(daysSinceLastMaintenance);
    } else {
      calculations.maintenanceCount = 0;
      calculations.daysSinceLastMaintenance = ageInDays;
    }

    return {
      entityType: 'Equipment',
      entityId: equipment.id,
      calculations,
      metadata: {
        calculatedAt: new Date().toISOString(),
        calculatedBy: this.context.user.id,
      },
    };
  }

  visitProduction(production: Production): CalculationResult {
    const calculations: Record<string, number> = {};

    // Calculate rates (per day)
    calculations.oilRate = production.oil / (production.duration / 24);
    calculations.gasRate = production.gas / (production.duration / 24);
    calculations.waterRate = production.water / (production.duration / 24);

    // Calculate ratios
    const totalLiquid = production.oil + production.water;
    if (totalLiquid > 0) {
      calculations.waterCut = production.water / totalLiquid;
      calculations.oilCut = production.oil / totalLiquid;
    }

    // Calculate gas-oil ratio
    if (production.oil > 0) {
      calculations.gasOilRatio = production.gas / production.oil;
    }

    // Calculate productivity index if pressure is available
    if (production.pressure && production.pressure > 0) {
      calculations.productivityIndex = calculations.oilRate / production.pressure;
    }

    return {
      entityType: 'Production',
      entityId: production.id,
      calculations,
      metadata: {
        calculatedAt: new Date().toISOString(),
        calculatedBy: this.context.user.id,
      },
    };
  }

  visitOperator(operator: Operator): CalculationResult {
    const calculations: Record<string, number> = {};

    // Calculate license remaining time
    const remainingMs = operator.licenseExpiration.getTime() - Date.now();
    calculations.licenseRemainingDays = Math.max(
      0,
      Math.floor(remainingMs / (1000 * 60 * 60 * 24)),
    );

    // Calculate asset counts
    calculations.wellCount = operator.wells?.length || 0;
    calculations.leaseCount = operator.leases?.length || 0;

    // Calculate total acreage
    if (operator.leases && operator.leases.length > 0) {
      calculations.totalAcreage = operator.leases.reduce((sum, lease) => sum + lease.acreage, 0);
      calculations.averageLeaseSize = calculations.totalAcreage / operator.leases.length;
    }

    // Calculate total production across all wells
    if (operator.wells && operator.wells.length > 0) {
      let totalOil = 0;
      let totalGas = 0;
      let totalWater = 0;

      operator.wells.forEach((well) => {
        if (well.production) {
          totalOil += well.production.reduce((sum, p) => sum + p.oil, 0);
          totalGas += well.production.reduce((sum, p) => sum + p.gas, 0);
          totalWater += well.production.reduce((sum, p) => sum + p.water, 0);
        }
      });

      calculations.totalOilProduction = totalOil;
      calculations.totalGasProduction = totalGas;
      calculations.totalWaterProduction = totalWater;
      calculations.averageOilPerWell = totalOil / operator.wells.length;
      calculations.averageGasPerWell = totalGas / operator.wells.length;
    }

    return {
      entityType: 'Operator',
      entityId: operator.id,
      calculations,
      metadata: {
        calculatedAt: new Date().toISOString(),
        calculatedBy: this.context.user.id,
      },
    };
  }
}
```

### React Hook Integration

```typescript
// hooks/use-visitor.ts
export function useVisitor<TResult>(
  visitorClass: new (context: VisitorContext, ...args: any[]) => Visitor<TResult>,
  ...args: any[]
) {
  const { user } = useAuth();
  const { permissions } = usePermissions();

  const [visitor] = useState(() => {
    const context: VisitorContext = {
      user,
      permissions,
      options: {},
      metadata: {},
    };

    return new visitorClass(context, ...args);
  });

  const visitEntity = useCallback(
    (entity: Visitable): TResult => {
      return entity.accept(visitor);
    },
    [visitor],
  );

  const visitEntities = useCallback(
    (entities: Visitable[]): TResult[] => {
      return entities.map((entity) => entity.accept(visitor));
    },
    [visitor],
  );

  return {
    visitor,
    visitEntity,
    visitEntities,
  };
}

// hooks/use-validation.ts
export function useValidation() {
  const { visitEntity, visitEntities } = useVisitor(ValidationVisitor);

  const validateEntity = useCallback(
    (entity: Visitable): ValidationResult => {
      return visitEntity(entity);
    },
    [visitEntity],
  );

  const validateEntities = useCallback(
    (entities: Visitable[]): ValidationResult[] => {
      return visitEntities(entities);
    },
    [visitEntities],
  );

  const getValidationSummary = useCallback((results: ValidationResult[]) => {
    return {
      totalEntities: results.length,
      validEntities: results.filter((r) => r.isValid).length,
      invalidEntities: results.filter((r) => !r.isValid).length,
      totalErrors: results.reduce((sum, r) => sum + r.errors.length, 0),
      totalWarnings: results.reduce((sum, r) => sum + r.warnings.length, 0),
    };
  }, []);

  return {
    validateEntity,
    validateEntities,
    getValidationSummary,
  };
}
```

### Component Usage

```typescript
// components/data-processor.tsx
export function DataProcessor({ entities }: { entities: Visitable[] }) {
  const { validateEntities, getValidationSummary } = useValidation();
  const { visitEntities: calculateEntities } = useVisitor(CalculationVisitor);
  const { visitEntities: exportEntities } = useVisitor(ExportVisitor, 'json');

  const [validationResults, setValidationResults] = useState<ValidationResult[]>([]);
  const [calculations, setCalculations] = useState<CalculationResult[]>([]);
  const [processing, setProcessing] = useState(false);

  const processEntities = async () => {
    setProcessing(true);

    try {
      // Validate all entities
      const validation = validateEntities(entities);
      setValidationResults(validation);

      // Calculate metrics for valid entities
      const validEntities = entities.filter((_, index) => validation[index].isValid);
      const calculationResults = calculateEntities(validEntities);
      setCalculations(calculationResults);

      toast.success('Data processing completed');
    } catch (error) {
      toast.error('Data processing failed');
    } finally {
      setProcessing(false);
    }
  };

  const exportData = () => {
    const validEntities = entities.filter((_, index) => validationResults[index]?.isValid);
    const exportData = exportEntities(validEntities);

    const blob = new Blob([JSON.stringify(exportData, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `export-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const validationSummary = getValidationSummary(validationResults);

  return (
    <div className="space-y-6">
      <div className="flex gap-4">
        <Button onClick={processEntities} disabled={processing}>
          {processing ? 'Processing...' : 'Process Data'}
        </Button>

        <Button
          onClick={exportData}
          disabled={validationResults.length === 0}
          variant="outline"
        >
          Export Valid Data
        </Button>
      </div>

      {validationResults.length > 0 && (
        <div className="grid grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle>Validation Summary</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <p>Total Entities: {validationSummary.totalEntities}</p>
                <p className="text-green-600">Valid: {validationSummary.validEntities}</p>
                <p className="text-red-600">Invalid: {validationSummary.invalidEntities}</p>
                <p className="text-yellow-600">Warnings: {validationSummary.totalWarnings}</p>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Processing Results</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                <p>Calculations: {calculations.length}</p>
                <p>Success Rate: {((validationSummary.validEntities / validationSummary.totalEntities) * 100).toFixed(1)}%</p>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Detailed results */}
      {validationResults.length > 0 && (
        <div className="space-y-4">
          <h3 className="text-lg font-semibold">Detailed Results</h3>
          {validationResults.map((result, index) => (
            <Card key={index} className={result.isValid ? 'border-green-200' : 'border-red-200'}>
              <CardHeader>
                <CardTitle className="text-sm">
                  {result.entityType} - {result.entityId}
                </CardTitle>
              </CardHeader>
              <CardContent>
                {result.errors.length > 0 && (
                  <div className="mb-2">
                    <h4 className="font-medium text-red-600">Errors:</h4>
                    <ul className="text-sm text-red-600">
                      {result.errors.map((error, i) => (
                        <li key={i}>• {error.message}</li>
                      ))}
                    </ul>
                  </div>
                )}

                {result.warnings.length > 0 && (
                  <div>
                    <h4 className="font-medium text-yellow-600">Warnings:</h4>
                    <ul className="text-sm text-yellow-600">
                      {result.warnings.map((warning, i) => (
                        <li key={i}>• {warning.message}</li>
                      ))}
                    </ul>
                  </div>
                )}

                {calculations[index] && (
                  <div className="mt-2">
                    <h4 className="font-medium">Calculations:</h4>
                    <div className="text-sm">
                      {Object.entries(calculations[index].calculations).map(([key, value]) => (
                        <p key={key}>{key}: {typeof value === 'number' ? value.toFixed(2) : value}</p>
                      ))}
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
```

## Benefits

### 1. **Separation of Concerns**

- Operations are separated from data structures
- Easy to add new operations without modifying existing classes
- Related operations are grouped together in visitor classes

### 2. **Extensibility**

- New visitor types can be added easily
- Existing visitors can be extended or modified
- Operations can be composed and chained

### 3. **Type Safety**

- Strong typing ensures correct method calls
- Compile-time checking of visitor implementations
- Clear interfaces for all operations

### 4. **Reusability**

- Visitors can be reused across different contexts
- Common operations can be shared between different parts of the application
- Easy to create specialized visitors for specific use cases

## Best Practices

### 1. **Visitor Interface Design**

```typescript
// ✅ Good: Specific, well-defined visitor methods
interface Visitor<TResult> {
  visitWell(well: Well): TResult;
  visitLease(lease: Lease): TResult;
  visitEquipment(equipment: Equipment): TResult;
}

// ❌ Bad: Generic, unclear visitor methods
interface Visitor<TResult> {
  visit(entity: any): TResult;
}
```

### 2. **Error Handling**

```typescript
// ✅ Good: Comprehensive error handling
visitWell(well: Well): ValidationResult {
  try {
    // Validation logic
  } catch (error) {
    return {
      isValid: false,
      entityId: well.id,
      entityType: 'Well',
      errors: [{
        field: 'general',
        message: 'Validation failed',
        code: 'VALIDATION_ERROR',
        severity: 'error'
      }],
      warnings: [],
    };
  }
}
```

### 3. **Context Usage**

```typescript
// ✅ Good: Use context for permissions and options
visitWell(well: Well): ExportData {
  if (!this.hasPermission('export:wells')) {
    throw new Error('Insufficient permissions');
  }

  const includeProduction = this.getOption('includeProduction', false);
  // ... rest of implementation
}
```

## Testing

```typescript
// __tests__/visitors/validation-visitor.test.ts
describe('ValidationVisitor', () => {
  let visitor: ValidationVisitor;
  let mockContext: VisitorContext;

  beforeEach(() => {
    mockContext = {
      user: { id: 'user1', name: 'Test User' },
      permissions: ['validate:wells'],
      options: {},
      metadata: {},
    };

    visitor = new ValidationVisitor(mockContext);
  });

  it('should validate well successfully', () => {
    const well = new Well(
      '1',
      'Test Well',
      '12345678901234',
      'op1',
      { latitude: 32.0, longitude: -97.0 },
      'vertical',
      'active',
      5000,
      new Date('2024-01-01'),
    );

    const result = visitor.visitWell(well);

    expect(result.isValid).toBe(true);
    expect(result.errors).toHaveLength(0);
    expect(result.entityType).toBe('Well');
  });

  it('should return validation errors for invalid well', () => {
    const well = new Well(
      '1',
      '', // Invalid - empty name
      '123', // Invalid - wrong API number format
      'op1',
      { latitude: 32.0, longitude: -97.0 },
      'vertical',
      'active',
      -100, // Invalid - negative depth
      new Date('2024-01-01'),
    );

    const result = visitor.visitWell(well);

    expect(result.isValid).toBe(false);
    expect(result.errors.length).toBeGreaterThan(0);
    expect(result.errors.some((e) => e.code === 'MISSING_WELL_NAME')).toBe(true);
    expect(result.errors.some((e) => e.code === 'INVALID_API_NUMBER')).toBe(true);
    expect(result.errors.some((e) => e.code === 'INVALID_DEPTH')).toBe(true);
  });
});
```

The Visitor Pattern provides a powerful way to organize operations on complex
data structures while maintaining clean separation of concerns and enabling easy
extensibility for new operations.
