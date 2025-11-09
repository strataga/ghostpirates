# Frontend Chain of Responsibility Pattern

## Overview

The Chain of Responsibility Pattern in frontend applications creates a chain of
validation handlers, processors, or middleware that can handle requests in
sequence. This pattern is particularly valuable for complex validation
scenarios, data processing pipelines, and permission checking in oil & gas
applications.

## Problem Statement

Complex frontend applications often need to:

- **Validate data** through multiple sequential checks
- **Process requests** through different middleware layers
- **Handle permissions** with multiple authorization levels
- **Transform data** through various processing steps
- **Apply business rules** in a specific order

Traditional approaches lead to:

- **Monolithic validation functions** that are hard to maintain
- **Tightly coupled** validation logic
- **Difficult to extend** or modify validation rules
- **Hard to test** individual validation steps
- **Inflexible ordering** of validation rules

## Solution

Implement the Chain of Responsibility Pattern to create a flexible, extensible
chain of handlers that can process requests sequentially, with each handler
deciding whether to process the request and/or pass it to the next handler.

## Implementation

### Base Handler Interface

```typescript
// lib/chain/interfaces.ts
export interface ChainHandler<TRequest, TResult> {
  setNext(handler: ChainHandler<TRequest, TResult>): ChainHandler<TRequest, TResult>;
  handle(request: TRequest): Promise<TResult>;
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  data?: any;
}

export interface ValidationError {
  field: string;
  message: string;
  code: string;
  severity: 'error' | 'warning';
}

export interface ValidationWarning {
  field: string;
  message: string;
  code: string;
}

export interface ProcessingContext {
  user: User;
  permissions: string[];
  metadata: Record<string, any>;
}

export interface ChainRequest<T = any> {
  data: T;
  context: ProcessingContext;
  options?: Record<string, any>;
}
```

### Abstract Base Handler

```typescript
// lib/chain/base-handler.ts
export abstract class BaseHandler<TRequest, TResult> implements ChainHandler<TRequest, TResult> {
  protected nextHandler?: ChainHandler<TRequest, TResult>;

  setNext(handler: ChainHandler<TRequest, TResult>): ChainHandler<TRequest, TResult> {
    this.nextHandler = handler;
    return handler;
  }

  async handle(request: TRequest): Promise<TResult> {
    const result = await this.doHandle(request);

    // If this handler indicates to continue and there's a next handler
    if (this.shouldContinue(result) && this.nextHandler) {
      const nextResult = await this.nextHandler.handle(request);
      return this.combineResults(result, nextResult);
    }

    return result;
  }

  protected abstract doHandle(request: TRequest): Promise<TResult>;

  protected shouldContinue(result: TResult): boolean {
    // Default: continue if result is valid
    if (typeof result === 'object' && result !== null && 'isValid' in result) {
      return (result as any).isValid;
    }
    return true;
  }

  protected combineResults(current: TResult, next: TResult): TResult {
    // Default: return the next result
    // Override in subclasses for custom combination logic
    return next;
  }
}
```

### Validation Chain Implementation

```typescript
// lib/chain/validation-chain.ts
export abstract class ValidationHandler extends BaseHandler<ChainRequest<any>, ValidationResult> {
  protected combineResults(current: ValidationResult, next: ValidationResult): ValidationResult {
    return {
      isValid: current.isValid && next.isValid,
      errors: [...current.errors, ...next.errors],
      warnings: [...current.warnings, ...next.warnings],
      data: { ...current.data, ...next.data },
    };
  }

  protected shouldContinue(result: ValidationResult): boolean {
    // Continue even if there are errors, but stop on critical errors
    return !result.errors.some(
      (error) => error.severity === 'error' && error.code.startsWith('CRITICAL_'),
    );
  }
}

// Format validation handler
export class FormatValidationHandler extends ValidationHandler {
  protected async doHandle(request: ChainRequest<any>): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // API Number format validation
    if (request.data.apiNumber) {
      if (!/^\d{14}$/.test(request.data.apiNumber)) {
        errors.push({
          field: 'apiNumber',
          message: 'API number must be exactly 14 digits',
          code: 'INVALID_FORMAT',
          severity: 'error',
        });
      }
    }

    // Email format validation
    if (request.data.email) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(request.data.email)) {
        errors.push({
          field: 'email',
          message: 'Invalid email format',
          code: 'INVALID_EMAIL_FORMAT',
          severity: 'error',
        });
      }
    }

    // Phone number format validation
    if (request.data.phone) {
      const phoneRegex = /^\+?[\d\s\-\(\)]+$/;
      if (!phoneRegex.test(request.data.phone)) {
        warnings.push({
          field: 'phone',
          message: 'Phone number format may be invalid',
          code: 'PHONE_FORMAT_WARNING',
        });
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      data: request.data,
    };
  }
}

// Business rules validation handler
export class BusinessRulesValidationHandler extends ValidationHandler {
  protected async doHandle(request: ChainRequest<any>): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Well depth validation
    if (request.data.totalDepth !== undefined) {
      if (request.data.totalDepth < 0) {
        errors.push({
          field: 'totalDepth',
          message: 'Well depth cannot be negative',
          code: 'INVALID_DEPTH',
          severity: 'error',
        });
      } else if (request.data.totalDepth > 50000) {
        warnings.push({
          field: 'totalDepth',
          message: 'Well depth exceeds typical maximum (50,000 ft)',
          code: 'UNUSUAL_DEPTH',
        });
      }
    }

    // Production rate validation
    if (request.data.oilRate !== undefined && request.data.oilRate < 0) {
      errors.push({
        field: 'oilRate',
        message: 'Oil production rate cannot be negative',
        code: 'INVALID_PRODUCTION_RATE',
        severity: 'error',
      });
    }

    // Date validation
    if (request.data.spudDate && request.data.completionDate) {
      const spudDate = new Date(request.data.spudDate);
      const completionDate = new Date(request.data.completionDate);

      if (completionDate < spudDate) {
        errors.push({
          field: 'completionDate',
          message: 'Completion date cannot be before spud date',
          code: 'INVALID_DATE_SEQUENCE',
          severity: 'error',
        });
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      data: request.data,
    };
  }
}

// Database validation handler
export class DatabaseValidationHandler extends ValidationHandler {
  constructor(private apiService: ApiService) {
    super();
  }

  protected async doHandle(request: ChainRequest<any>): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Check API number uniqueness
    if (request.data.apiNumber) {
      try {
        const exists = await this.apiService.checkApiNumberExists(request.data.apiNumber);
        if (exists && !request.options?.isUpdate) {
          errors.push({
            field: 'apiNumber',
            message: 'API number already exists in the system',
            code: 'DUPLICATE_API_NUMBER',
            severity: 'error',
          });
        }
      } catch (error) {
        warnings.push({
          field: 'apiNumber',
          message: 'Could not verify API number uniqueness',
          code: 'VALIDATION_SERVICE_ERROR',
        });
      }
    }

    // Check operator exists
    if (request.data.operatorId) {
      try {
        const operator = await this.apiService.getOperator(request.data.operatorId);
        if (!operator) {
          errors.push({
            field: 'operatorId',
            message: 'Selected operator does not exist',
            code: 'INVALID_OPERATOR',
            severity: 'error',
          });
        } else if (!operator.active) {
          warnings.push({
            field: 'operatorId',
            message: 'Selected operator is inactive',
            code: 'INACTIVE_OPERATOR',
          });
        }
      } catch (error) {
        warnings.push({
          field: 'operatorId',
          message: 'Could not verify operator information',
          code: 'OPERATOR_SERVICE_ERROR',
        });
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      data: request.data,
    };
  }
}

// Regulatory compliance validation handler
export class RegulatoryValidationHandler extends ValidationHandler {
  constructor(private regulatoryService: RegulatoryService) {
    super();
  }

  protected async doHandle(request: ChainRequest<any>): Promise<ValidationResult> {
    const errors: ValidationError[] = [];
    const warnings: ValidationWarning[] = [];

    // Check regulatory compliance for API number
    if (request.data.apiNumber && request.data.state) {
      try {
        const compliance = await this.regulatoryService.checkCompliance(
          request.data.apiNumber,
          request.data.state,
        );

        if (!compliance.isValid) {
          errors.push({
            field: 'apiNumber',
            message: `Regulatory compliance check failed: ${compliance.reason}`,
            code: 'REGULATORY_COMPLIANCE_FAILED',
            severity: 'error',
          });
        }

        if (compliance.warnings?.length > 0) {
          compliance.warnings.forEach((warning) => {
            warnings.push({
              field: 'apiNumber',
              message: warning,
              code: 'REGULATORY_WARNING',
            });
          });
        }
      } catch (error) {
        warnings.push({
          field: 'apiNumber',
          message: 'Could not verify regulatory compliance',
          code: 'REGULATORY_SERVICE_ERROR',
        });
      }
    }

    // Check permit requirements
    if (request.data.wellType && request.data.state) {
      const requiredPermits = this.getRequiredPermits(request.data.wellType, request.data.state);
      const providedPermits = request.data.permits || [];

      requiredPermits.forEach((requiredPermit) => {
        const hasPermit = providedPermits.some((permit: any) => permit.type === requiredPermit);
        if (!hasPermit) {
          errors.push({
            field: 'permits',
            message: `Missing required permit: ${requiredPermit}`,
            code: 'MISSING_PERMIT',
            severity: 'error',
          });
        }
      });
    }

    return {
      isValid: errors.length === 0,
      errors,
      warnings,
      data: request.data,
    };
  }

  private getRequiredPermits(wellType: string, state: string): string[] {
    // Simplified permit requirements logic
    const permits: string[] = ['drilling_permit'];

    if (wellType === 'horizontal') {
      permits.push('horizontal_drilling_permit');
    }

    if (state === 'TX') {
      permits.push('texas_rrc_permit');
    }

    return permits;
  }
}
```

### Permission Chain Implementation

```typescript
// lib/chain/permission-chain.ts
export abstract class PermissionHandler extends BaseHandler<ChainRequest<any>, boolean> {
  protected shouldContinue(result: boolean): boolean {
    // Continue only if permission is granted
    return result;
  }

  protected combineResults(current: boolean, next: boolean): boolean {
    // All permissions must be granted
    return current && next;
  }
}

export class RolePermissionHandler extends PermissionHandler {
  constructor(private requiredRoles: string[]) {
    super();
  }

  protected async doHandle(request: ChainRequest<any>): Promise<boolean> {
    const userRoles = request.context.user.roles || [];
    return this.requiredRoles.some((role) => userRoles.includes(role));
  }
}

export class ResourcePermissionHandler extends PermissionHandler {
  constructor(
    private action: string,
    private resource: string,
  ) {
    super();
  }

  protected async doHandle(request: ChainRequest<any>): Promise<boolean> {
    const permissions = request.context.permissions || [];
    const requiredPermission = `${this.action}:${this.resource}`;
    return permissions.includes(requiredPermission) || permissions.includes('*');
  }
}

export class OwnershipPermissionHandler extends PermissionHandler {
  protected async doHandle(request: ChainRequest<any>): Promise<boolean> {
    // Check if user owns the resource or has admin privileges
    const userId = request.context.user.id;
    const resourceOwnerId = request.data.ownerId || request.data.createdBy;
    const isAdmin = request.context.user.roles?.includes('admin');

    return isAdmin || userId === resourceOwnerId;
  }
}
```

### Chain Factory

```typescript
// lib/chain/chain-factory.ts
export class ValidationChainFactory {
  static createWellValidationChain(apiService: ApiService, regulatoryService: RegulatoryService) {
    const formatHandler = new FormatValidationHandler();
    const businessRulesHandler = new BusinessRulesValidationHandler();
    const databaseHandler = new DatabaseValidationHandler(apiService);
    const regulatoryHandler = new RegulatoryValidationHandler(regulatoryService);

    // Chain the handlers
    formatHandler.setNext(businessRulesHandler).setNext(databaseHandler).setNext(regulatoryHandler);

    return formatHandler;
  }

  static createUserValidationChain(apiService: ApiService) {
    const formatHandler = new FormatValidationHandler();
    const businessRulesHandler = new BusinessRulesValidationHandler();
    const databaseHandler = new DatabaseValidationHandler(apiService);

    formatHandler.setNext(businessRulesHandler).setNext(databaseHandler);

    return formatHandler;
  }

  static createProductionDataValidationChain() {
    const formatHandler = new FormatValidationHandler();
    const businessRulesHandler = new BusinessRulesValidationHandler();

    formatHandler.setNext(businessRulesHandler);

    return formatHandler;
  }
}

export class PermissionChainFactory {
  static createWellManagementPermissionChain() {
    const roleHandler = new RolePermissionHandler(['admin', 'manager', 'operator']);
    const resourceHandler = new ResourcePermissionHandler('manage', 'wells');
    const ownershipHandler = new OwnershipPermissionHandler();

    roleHandler.setNext(resourceHandler).setNext(ownershipHandler);

    return roleHandler;
  }

  static createUserManagementPermissionChain() {
    const roleHandler = new RolePermissionHandler(['admin', 'manager']);
    const resourceHandler = new ResourcePermissionHandler('manage', 'users');

    roleHandler.setNext(resourceHandler);

    return roleHandler;
  }
}
```

### React Hook Integration

```typescript
// hooks/use-validation-chain.ts
export function useValidationChain<T>(
  chainFactory: () => ChainHandler<ChainRequest<T>, ValidationResult>,
) {
  const [chain] = useState(chainFactory);
  const [isValidating, setIsValidating] = useState(false);

  const validate = useCallback(
    async (
      data: T,
      context: ProcessingContext,
      options?: Record<string, any>,
    ): Promise<ValidationResult> => {
      setIsValidating(true);

      try {
        const request: ChainRequest<T> = { data, context, options };
        const result = await chain.handle(request);
        return result;
      } finally {
        setIsValidating(false);
      }
    },
    [chain],
  );

  return { validate, isValidating };
}

// hooks/use-permission-chain.ts
export function usePermissionChain(chainFactory: () => ChainHandler<ChainRequest<any>, boolean>) {
  const [chain] = useState(chainFactory);
  const { user } = useAuth();
  const { permissions } = usePermissions();

  const checkPermission = useCallback(
    async (data: any, options?: Record<string, any>): Promise<boolean> => {
      const context: ProcessingContext = {
        user,
        permissions,
        metadata: {},
      };

      const request: ChainRequest<any> = { data, context, options };
      return chain.handle(request);
    },
    [chain, user, permissions],
  );

  return { checkPermission };
}
```

### Component Usage

```typescript
// components/forms/well-form.tsx
export function WellForm({ well, onSubmit }: WellFormProps) {
  const { validate, isValidating } = useValidationChain(() =>
    ValidationChainFactory.createWellValidationChain(apiService, regulatoryService)
  );

  const { checkPermission } = usePermissionChain(() =>
    PermissionChainFactory.createWellManagementPermissionChain()
  );

  const [formData, setFormData] = useState(well || {});
  const [validationResult, setValidationResult] = useState<ValidationResult | null>(null);
  const [canEdit, setCanEdit] = useState(false);

  useEffect(() => {
    checkPermission(well).then(setCanEdit);
  }, [well, checkPermission]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const context: ProcessingContext = {
      user: getCurrentUser(),
      permissions: getUserPermissions(),
      metadata: {},
    };

    const result = await validate(formData, context, { isUpdate: !!well?.id });
    setValidationResult(result);

    if (result.isValid) {
      onSubmit(formData);
    } else {
      toast.error('Please fix validation errors before submitting');
    }
  };

  if (!canEdit) {
    return <div>You don't have permission to edit this well.</div>;
  }

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div>
        <Label htmlFor="apiNumber">API Number</Label>
        <Input
          id="apiNumber"
          value={formData.apiNumber || ''}
          onChange={(e) => setFormData({ ...formData, apiNumber: e.target.value })}
          className={validationResult?.errors.some(e => e.field === 'apiNumber') ? 'border-red-500' : ''}
        />
        {validationResult?.errors
          .filter(error => error.field === 'apiNumber')
          .map((error, index) => (
            <p key={index} className="text-sm text-red-600 mt-1">
              {error.message}
            </p>
          ))}
        {validationResult?.warnings
          .filter(warning => warning.field === 'apiNumber')
          .map((warning, index) => (
            <p key={index} className="text-sm text-yellow-600 mt-1">
              {warning.message}
            </p>
          ))}
      </div>

      {/* Other form fields */}

      <Button type="submit" disabled={isValidating}>
        {isValidating ? 'Validating...' : 'Submit'}
      </Button>
    </form>
  );
}
```

## Benefits

### 1. **Flexible Validation**

- Easy to add, remove, or reorder validation steps
- Each handler has a single responsibility
- Validation logic is reusable across different forms

### 2. **Extensibility**

- New validation rules can be added without modifying existing code
- Handlers can be composed in different ways for different scenarios
- Easy to create specialized validation chains

### 3. **Testability**

- Each handler can be tested independently
- Easy to mock dependencies for testing
- Clear separation of concerns

### 4. **Performance**

- Validation can stop early on critical errors
- Async validation steps can be optimized
- Caching can be implemented at the handler level

## Best Practices

### 1. **Handler Responsibility**

```typescript
// ✅ Good: Single responsibility
class EmailFormatValidationHandler extends ValidationHandler {
  // Only validates email format
}

// ❌ Bad: Multiple responsibilities
class EmailValidationHandler extends ValidationHandler {
  // Validates format, uniqueness, and domain restrictions
}
```

### 2. **Error Handling**

```typescript
// ✅ Good: Graceful error handling
protected async doHandle(request: ChainRequest<any>): Promise<ValidationResult> {
  try {
    // Validation logic
  } catch (error) {
    return {
      isValid: false,
      errors: [{
        field: 'general',
        message: 'Validation service unavailable',
        code: 'SERVICE_ERROR',
        severity: 'warning',
      }],
      warnings: [],
    };
  }
}
```

### 3. **Chain Configuration**

```typescript
// ✅ Good: Factory pattern for chain creation
ValidationChainFactory.createWellValidationChain();

// ❌ Bad: Manual chain construction everywhere
const handler1 = new FormatValidationHandler();
const handler2 = new BusinessRulesValidationHandler();
handler1.setNext(handler2);
```

## Testing

```typescript
// __tests__/chain/format-validation-handler.test.ts
describe('FormatValidationHandler', () => {
  let handler: FormatValidationHandler;

  beforeEach(() => {
    handler = new FormatValidationHandler();
  });

  it('should validate API number format', async () => {
    const request: ChainRequest<any> = {
      data: { apiNumber: '12345678901234' },
      context: mockContext,
    };

    const result = await handler.handle(request);

    expect(result.isValid).toBe(true);
    expect(result.errors).toHaveLength(0);
  });

  it('should reject invalid API number format', async () => {
    const request: ChainRequest<any> = {
      data: { apiNumber: '123' },
      context: mockContext,
    };

    const result = await handler.handle(request);

    expect(result.isValid).toBe(false);
    expect(result.errors).toHaveLength(1);
    expect(result.errors[0].code).toBe('INVALID_FORMAT');
  });
});
```

The Chain of Responsibility Pattern provides a flexible, maintainable way to
handle complex validation and processing scenarios while keeping individual
handlers focused and testable.
