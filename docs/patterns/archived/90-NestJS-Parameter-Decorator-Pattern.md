# 90. NestJS Parameter Decorator Pattern

## Pattern Category
**Backend / Hexagonal Architecture - Presentation Layer**

## Problem Statement

In multi-tenant applications using NestJS, controllers need access to tenant context information extracted from requests (via middleware). Without a parameter decorator, controllers must manually extract this information from the request object, leading to:

1. **Code Duplication**: Every controller method must extract tenant context the same way
2. **Type Safety Issues**: Manual extraction is prone to typos and lacks TypeScript type checking
3. **Coupling**: Controllers become tightly coupled to Express request internals
4. **Inconsistency**: Different developers may extract context differently

## Solution Overview

Use NestJS `createParamDecorator` to create a custom parameter decorator that:

1. **Extracts** tenant context from the request object (populated by middleware)
2. **Supports Property Selection**: Allows extracting specific properties or the entire context object
3. **Type Safety**: Provides full TypeScript type checking
4. **Backward Compatibility**: Supports aliased property names for legacy code

## Implementation

### 1. Define Context Interface

```typescript
/**
 * apps/api/src/infrastructure/decorators/tenant-context.decorator.ts
 */
export interface TenantContextDto {
  id: string;
  subdomain: string;
  slug: string;
  databaseUrl: string;
  databaseName: string;
  databaseType: string;
}
```

### 2. Create Parameter Decorator

```typescript
import { createParamDecorator, ExecutionContext } from '@nestjs/common';

type TenantContextProperty = keyof TenantContextDto | 'tenantId';

export const TenantContext = createParamDecorator(
  (
    data: TenantContextProperty | undefined,
    ctx: ExecutionContext,
  ): string | TenantContextDto | undefined => {
    const request = ctx
      .switchToHttp()
      .getRequest<{ tenant?: TenantContextDto }>();

    const tenant = request.tenant;

    if (!tenant) {
      return undefined;
    }

    // If no property specified, return entire context
    if (!data) {
      return tenant;
    }

    // Handle 'tenantId' alias for backward compatibility
    if (data === 'tenantId') {
      return tenant.id;
    }

    // Extract specific property
    return tenant[data];
  },
);
```

### 3. Middleware Populates Context

```typescript
/**
 * apps/api/src/infrastructure/middleware/tenant-resolver.middleware.ts
 */
@Injectable()
export class TenantResolverMiddleware implements NestMiddleware {
  async use(req: Request, res: Response, next: NextFunction) {
    // Extract subdomain or X-Tenant-ID header
    const subdomain = this.extractSubdomain(req.hostname);

    // Look up tenant in database
    const tenant = await this.tenantRepository.findBySubdomain(subdomain);

    // Attach to request
    const tenantContext: TenantContextDto = {
      id: tenant.id,
      subdomain: tenant.subdomain,
      slug: tenant.slug,
      databaseUrl: tenant.databaseConfig.url,
      databaseName: tenant.databaseConfig.name,
      databaseType: tenant.databaseConfig.type,
    };

    req.tenant = tenantContext;
    next();
  }
}
```

### 4. Usage in Controllers

```typescript
/**
 * Example 1: Extract entire context
 */
@Get()
async getData(
  @TenantContext() tenant: TenantContextDto
) {
  return this.service.getData(tenant.id, tenant.databaseName);
}

/**
 * Example 2: Extract specific property
 */
@Get()
async getData(
  @TenantContext('id') tenantId: string,
  @TenantContext('databaseName') databaseName: string
) {
  return this.service.getData(tenantId, databaseName);
}

/**
 * Example 3: Backward compatibility alias
 */
@Get()
async getData(
  @TenantContext('tenantId') tenantId: string  // 'tenantId' → tenant.id
) {
  return this.service.getData(tenantId);
}
```

## Key Design Decisions

### 1. Property Selection vs. Full Object

**Decision**: Support both modes - entire object AND property extraction

**Rationale**:
- **Full Object**: Useful when multiple properties are needed
- **Property Extraction**: Cleaner when only one property is needed (e.g., just `tenantId`)
- **Flexibility**: Different use cases require different approaches

### 2. Backward Compatibility Alias

**Decision**: Map `'tenantId'` to `tenant.id` for backward compatibility

**Rationale**:
- Existing code used `@TenantContext('tenantId')` before decorator fix
- Changing all controllers to use `'id'` would be disruptive
- Alias provides smooth migration path

### 3. Return Type Union

**Decision**: Return `string | TenantContextDto | undefined`

**Rationale**:
- When property selected: Returns `string` (e.g., `tenant.id`)
- When no property: Returns `TenantContextDto` (entire object)
- When tenant not found: Returns `undefined`
- TypeScript enforces correct usage based on parameter type

### 4. Middleware Separation of Concerns

**Decision**: Middleware populates context, decorator extracts it

**Rationale**:
- **Middleware**: Handles complex logic (DB lookup, validation, error handling)
- **Decorator**: Simple extraction layer with zero business logic
- **Testability**: Middleware and decorator can be tested independently

## Common Pitfalls

### ❌ Anti-Pattern: Using Decorator Without Parameter

```typescript
// WRONG: Will return entire TenantContextDto object
@Get()
async getData(
  @TenantContext() tenantId: string  // Type mismatch!
) {
  return this.service.getData(tenantId);  // Runtime error: tenantId is an object
}
```

**Fix**: Specify property to extract

```typescript
// CORRECT: Extract specific property
@Get()
async getData(
  @TenantContext('id') tenantId: string
) {
  return this.service.getData(tenantId);
}
```

### ❌ Anti-Pattern: Non-Existent Property

```typescript
// WRONG: 'tenantUuid' doesn't exist in TenantContextDto
@Get()
async getData(
  @TenantContext('tenantUuid') tenantId: string  // TypeScript error
) {
  return this.service.getData(tenantId);
}
```

**Fix**: Use correct property name from interface

```typescript
// CORRECT: Use 'id' or alias 'tenantId'
@Get()
async getData(
  @TenantContext('id') tenantId: string
) {
  return this.service.getData(tenantId);
}
```

### ❌ Anti-Pattern: Ignoring Undefined Case

```typescript
// WRONG: Not handling undefined case
@Get()
async getData(
  @TenantContext('id') tenantId: string
) {
  return this.service.getData(tenantId);  // tenantId could be undefined
}
```

**Fix**: Add guard or use middleware to ensure tenant exists

```typescript
// CORRECT: Middleware throws error if tenant not found
@Injectable()
export class TenantResolverMiddleware implements NestMiddleware {
  async use(req: Request, res: Response, next: NextFunction) {
    const tenant = await this.tenantRepository.findBySubdomain(subdomain);

    if (!tenant) {
      throw new NotFoundException('Tenant not found');  // Prevents undefined
    }

    req.tenant = tenantContext;
    next();
  }
}
```

## Testing

### Unit Test for Decorator

```typescript
describe('TenantContext Decorator', () => {
  let executionContext: ExecutionContext;

  beforeEach(() => {
    const mockRequest = {
      tenant: {
        id: '123',
        subdomain: 'acme',
        slug: 'acme-corp',
        databaseUrl: 'postgresql://...',
        databaseName: 'acme_db',
        databaseType: 'postgresql',
      },
    };

    executionContext = {
      switchToHttp: () => ({
        getRequest: () => mockRequest,
      }),
    } as ExecutionContext;
  });

  it('should return entire context when no property specified', () => {
    const factory = TenantContext(undefined, executionContext);
    expect(factory).toEqual(mockRequest.tenant);
  });

  it('should extract specific property', () => {
    const factory = TenantContext('id', executionContext);
    expect(factory).toBe('123');
  });

  it('should handle tenantId alias', () => {
    const factory = TenantContext('tenantId', executionContext);
    expect(factory).toBe('123');  // Maps to tenant.id
  });

  it('should return undefined when tenant not in request', () => {
    const mockRequest = { tenant: undefined };
    const ctx = {
      switchToHttp: () => ({ getRequest: () => mockRequest }),
    } as ExecutionContext;

    const factory = TenantContext('id', ctx);
    expect(factory).toBeUndefined();
  });
});
```

### Integration Test

```typescript
describe('Analytics Controller (e2e)', () => {
  it('should extract tenant context correctly', async () => {
    return request(app.getHttpServer())
      .get('/api/analytics/trends/well-123')
      .set('Host', 'acme.onwellos.com')  // Subdomain triggers middleware
      .expect(200)
      .expect((res) => {
        // Verify decorator extracted tenantId correctly
        expect(res.body.tenantId).toBe('123');
      });
  });
});
```

## Related Patterns

- **[69. Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)**: Uses this decorator to extract tenant database connection info
- **[76. Triple-Credential Multi-Tenant Authentication Pattern](./76-Triple-Credential-Multi-Tenant-Authentication-Pattern.md)**: Middleware that populates tenant context
- **[58. JWT Authentication Session Management Pattern](./58-JWT-Authentication-Session-Management-Pattern.md)**: Similar decorator pattern for extracting current user

## Benefits

1. **Type Safety**: Full TypeScript support with autocomplete and compile-time checking
2. **Code Reuse**: Eliminates duplicated tenant context extraction code
3. **Consistency**: Enforces uniform approach across all controllers
4. **Testability**: Decorator can be tested independently of controllers
5. **Maintainability**: Single source of truth for context extraction logic

## When to Use

- ✅ Multi-tenant applications where tenant context is needed in controllers
- ✅ Request-scoped data that needs to be accessed across many controller methods
- ✅ Custom authentication/authorization that attaches user/tenant data to requests
- ✅ Logging/auditing where context information needs to be extracted consistently

## When NOT to Use

- ❌ Global configuration that doesn't change per request
- ❌ Simple single-tenant applications
- ❌ Data that can be passed as route parameters instead

## Real-World Example: Analytics Endpoint

```typescript
/**
 * Before: Manual extraction, error-prone
 */
@Get('trends/:wellId')
async getTrend(
  @Req() req: Request,
  @Param('wellId') wellId: string,
) {
  const tenant = req['tenant'];  // No type safety
  const tenantId = tenant?.id;   // Could be undefined
  const dbName = tenant?.databaseName;  // Typo-prone

  return this.service.getTrend(tenantId, dbName, wellId);
}

/**
 * After: Decorator-based extraction, type-safe
 */
@Get('trends/:wellId')
async getTrend(
  @TenantContext('id') tenantId: string,
  @TenantContext('databaseName') databaseName: string,
  @Param('wellId') wellId: string,
) {
  return this.service.getTrend(tenantId, databaseName, wellId);
}
```

## Migration Guide

### Step 1: Update Decorator Implementation

Add property selection and alias support to existing decorator.

### Step 2: Identify Incorrect Usage

Search for `@TenantContext() tenantId: string` pattern (should be `@TenantContext('id')`).

### Step 3: Fix Controllers

Replace:
```typescript
@TenantContext() tenantId: string
```

With:
```typescript
@TenantContext('id') tenantId: string
// OR use alias for backward compatibility
@TenantContext('tenantId') tenantId: string
```

### Step 4: Run Tests

Ensure all controller tests pass and verify runtime behavior.

## Summary

The NestJS Parameter Decorator Pattern provides a clean, type-safe way to extract request-scoped context in multi-tenant applications. By combining middleware (for context population) with decorators (for extraction), we achieve:

- **Separation of Concerns**: Middleware handles complexity, decorator handles extraction
- **Type Safety**: Full TypeScript support with compile-time checking
- **Consistency**: Uniform approach across all controllers
- **Backward Compatibility**: Alias support for smooth migrations

This pattern is essential for any multi-tenant NestJS application where tenant context needs to be accessed across many controller methods.
