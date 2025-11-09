# NestJS CQRS Module Organization Pattern

**Category**: Backend Architecture
**Pattern Type**: Structural
**Complexity**: Intermediate
**Status**: ✅ Production-Ready

---

## Overview

A proven pattern for organizing NestJS modules that use CQRS (Command Query Responsibility Segregation) to ensure proper handler registration and avoid common pitfalls with QueryBus/CommandBus scoping.

### Problem

When using NestJS with `@nestjs/cqrs`, improper module organization can lead to runtime errors where CQRS handlers cannot be found by the QueryBus or CommandBus, even though they are properly decorated and appear to be registered.

**Common Error**:
```
Error: No handler found for the query: "GetFooQuery"
```

**Root Cause**: Each NestJS module that imports `CqrsModule` gets its **own instance** of QueryBus and CommandBus. Handlers registered in one module's bus are invisible to controllers using a different module's bus.

---

## Anti-Pattern: Split Application/Presentation Modules

❌ **Don't Do This**:

```typescript
// ❌ ANTI-PATTERN: Separate modules create separate QueryBus instances
// application/scada/scada.module.ts
@Module({
  imports: [CqrsModule, DatabaseModule],
  providers: [
    ...CommandHandlers,
    ...QueryHandlers,  // ← Registered in this module's QueryBus
    ...Repositories,
  ],
  exports: [CqrsModule],
})
export class ScadaModule {}

// presentation/scada/scada-presentation.module.ts
@Module({
  imports: [
    CqrsModule,      // ← Creates NEW QueryBus instance!
    ScadaModule,     // ← Different QueryBus than handlers
  ],
  controllers: [ScadaController],
  providers: [TenantDatabaseService],
})
export class ScadaPresentationModule {}

// app.module.ts
@Module({
  imports: [
    ScadaPresentationModule,  // ← Only this is imported
    // ScadaModule NOT imported at app level
  ],
})
export class AppModule {}
```

**Why This Fails**:
1. `ScadaModule` registers handlers in **its** QueryBus instance
2. `ScadaPresentationModule` imports `CqrsModule` separately, creating a **new** QueryBus instance
3. `ScadaController` injects the **new** QueryBus, which doesn't have any handlers
4. Runtime error: "No handler found for the query"

Even if `ScadaPresentationModule` imports `ScadaModule`, the duplicate `CqrsModule` import creates module scoping issues.

---

## Solution: Unified Module Pattern

✅ **Do This Instead**:

```typescript
// presentation/scada/scada.module.ts
import { Module } from '@nestjs/common';
import { CqrsModule } from '@nestjs/cqrs';
import { ScadaController } from './scada.controller';
import { DatabaseModule } from '../../infrastructure/database/database.module';

// Import CQRS handlers directly
import {
  CreateScadaConnectionHandler,
  UpdateScadaConnectionHandler,
  DeleteScadaConnectionHandler,
} from '../../application/scada/commands';

import {
  GetScadaConnectionsHandler,
  GetScadaConnectionByIdHandler,
  GetScadaReadingsHandler,
} from '../../application/scada/queries';

// Import repositories
import { ScadaConnectionRepository } from '../../infrastructure/database/repositories/scada-connection.repository';
import { TagMappingRepository } from '../../infrastructure/database/repositories/tag-mapping.repository';

const CommandHandlers = [
  CreateScadaConnectionHandler,
  UpdateScadaConnectionHandler,
  DeleteScadaConnectionHandler,
];

const QueryHandlers = [
  GetScadaConnectionsHandler,
  GetScadaConnectionByIdHandler,
  GetScadaReadingsHandler,
];

const Repositories = [
  {
    provide: 'IScadaConnectionRepository',
    useClass: ScadaConnectionRepository,
  },
  {
    provide: 'ITagMappingRepository',
    useClass: TagMappingRepository,
  },
];

@Module({
  imports: [
    CqrsModule,      // ← Single CqrsModule import
    DatabaseModule,
  ],
  controllers: [
    ScadaController,  // ← Controllers in same module
  ],
  providers: [
    ...CommandHandlers,  // ← Handlers in same module
    ...QueryHandlers,    // ← All share the same QueryBus instance
    ...Repositories,
  ],
})
export class ScadaModule {}

// app.module.ts
@Module({
  imports: [
    ScadaModule,  // ← Single module import
  ],
})
export class AppModule {}
```

**Why This Works**:
1. **Single `CqrsModule` import** = single QueryBus/CommandBus instance
2. **Handlers and controllers in same module** = they share the same bus
3. **Handlers are registered** and immediately available to controllers
4. **No scoping issues** - everything is in one cohesive module

---

## File Organization Best Practices

### ✅ Recommended: Directory Structure

Keep query/command handlers in directories with `index.ts` exports:

```
application/scada/
├── commands/
│   ├── create-scada-connection/
│   │   ├── create-scada-connection.command.ts
│   │   ├── create-scada-connection.handler.ts
│   │   └── index.ts  ← export * from './create-scada-connection.command'
│   │                    export * from './create-scada-connection.handler'
│   └── index.ts      ← export * from './create-scada-connection'
│
├── queries/
│   ├── get-scada-connections/
│   │   ├── get-scada-connections.query.ts
│   │   ├── get-scada-connections.handler.ts
│   │   └── index.ts
│   └── index.ts
│
└── dto/
    └── scada-connection.dto.ts
```

**Import Pattern**:
```typescript
// ✅ Good: Import from directory
import { GetScadaConnectionsQuery } from '../../application/scada/queries/get-scada-connections';

// ✅ Also good: Import from barrel
import { GetScadaConnectionsQuery } from '../../application/scada/queries';
```

### ❌ Anti-Pattern: Mixed File/Directory Organization

**Never have both a file AND a directory with the same name**:

```
queries/
├── get-scada-connections.query.ts    ← File (OLD version)
├── get-scada-connections/             ← Directory (NEW version)
│   ├── get-scada-connections.query.ts
│   └── get-scada-connections.handler.ts
└── index.ts
```

**Why This Fails**:
- TypeScript resolves `import from './get-scada-connections'` ambiguously
- File version vs directory version creates **two different class instances**
- `@QueryHandler(GetScadaConnectionsQuery)` decorator can't match if controller uses different class
- Result: "No handler found" error

**Fix**: Delete the standalone file or the directory - keep only one version.

---

## Migration Guide

If you have split modules (application + presentation), migrate to unified modules:

### Step 1: Identify Split Modules

```bash
# Find modules that might be split
find src/application -name "*.module.ts"
find src/presentation -name "*.module.ts"

# Look for matching pairs like:
# - application/scada/scada.module.ts
# - presentation/scada/scada-presentation.module.ts
```

### Step 2: Merge into Unified Module

1. **Open the presentation module** (e.g., `scada-presentation.module.ts`)
2. **Import handlers directly**:
   ```typescript
   import { CommandHandlers } from '../../application/scada/commands';
   import { QueryHandlers } from '../../application/scada/queries';
   ```
3. **Add handlers to providers**:
   ```typescript
   @Module({
     imports: [CqrsModule, DatabaseModule],
     controllers: [...],
     providers: [
       ...CommandHandlers,  // ← Add this
       ...QueryHandlers,    // ← Add this
       ...Repositories,
     ],
   })
   ```
4. **Remove duplicate `CqrsModule` import** if present
5. **Delete the separate application module** (e.g., `scada.module.ts`)
6. **Update app.module.ts** to import only the unified module

### Step 3: Fix Import Paths

Update controller imports to use correct paths:

```typescript
// Before (might reference old file)
import { GetFooQuery } from '../../application/foo/queries/get-foo.query';

// After (reference directory)
import { GetFooQuery } from '../../application/foo/queries/get-foo';
```

### Step 4: Clean Up Duplicate Files

```bash
# Find potential file/directory conflicts
find src/application -type f -name "*.query.ts" | while read file; do
  dir="${file%.query.ts}"
  if [ -d "$dir" ]; then
    echo "Conflict: $file and $dir/"
  fi
done
```

### Step 5: Test

```bash
# Verify handlers are found
pnpm --filter=api dev
# Make API request to trigger query handler
curl http://localhost:4000/api/scada/connections

# Should NOT see "No handler found" error
```

---

## Real-World Example: WellOS SCADA Module

### Before (Split Modules - Broken)

```typescript
// application/scada/scada.module.ts
@Module({
  imports: [CqrsModule, DatabaseModule],
  providers: [
    ...CommandHandlers,
    ...QueryHandlers,
    ...Repositories,
  ],
  exports: [CqrsModule],  // ← Export doesn't help!
})
export class ScadaModule {}

// presentation/scada/scada-presentation.module.ts
@Module({
  imports: [
    CqrsModule,           // ← New QueryBus instance
    ScadaModule,          // ← Different QueryBus
  ],
  controllers: [ScadaController],
})
export class ScadaPresentationModule {}
```

**Error**:
```
Error: No handler found for the query: "GetScadaConnectionsQuery"
```

### After (Unified Module - Working)

```typescript
// presentation/scada/scada.module.ts (merged)
import { GetScadaConnectionsHandler } from '../../application/scada/queries';

@Module({
  imports: [
    CqrsModule,      // ← Single import
    DatabaseModule,
  ],
  controllers: [ScadaController],  // ← Same module
  providers: [
    ...QueryHandlers,  // ← Same QueryBus instance
    ...Repositories,
  ],
})
export class ScadaModule {}  // ← Renamed from ScadaPresentationModule
```

**Result**: ✅ Handlers found, queries execute successfully

---

## When to Use This Pattern

✅ **Use Unified Module Pattern when**:
- Building any NestJS feature that uses CQRS
- Using `@nestjs/cqrs` CommandBus or QueryBus
- Creating new modules from scratch
- Migrating from split modules after handler registration errors

❌ **Don't Use Split Modules when**:
- You need controllers and handlers in the same NestJS application
- You're using `@nestjs/cqrs` (stick to unified modules)

⚠️ **Split Modules MAY Be Acceptable**:
- For microservices where presentation and application are in **completely separate deployments**
- When using raw message buses (not `@nestjs/cqrs`)
- For pure domain modules with no CQRS handlers

---

## Benefits

1. **✅ Predictable Handler Registration**: Handlers and controllers share the same QueryBus/CommandBus
2. **✅ Fewer Runtime Errors**: Eliminates "No handler found" errors
3. **✅ Simpler Debugging**: Single module to check for handler registration
4. **✅ Easier Testing**: Mock single module instead of module hierarchy
5. **✅ Better Developer Experience**: Less confusion about which module to import
6. **✅ Follows NestJS Best Practices**: Aligns with official NestJS documentation examples

---

## Common Mistakes

### Mistake 1: Importing CqrsModule Multiple Times

```typescript
// ❌ Bad: Multiple CqrsModule imports
@Module({
  imports: [CqrsModule, SomeOtherModule],  // ← CqrsModule here
})
export class FeatureModule {}

@Module({
  imports: [CqrsModule, FeatureModule],  // ← CqrsModule again!
})
export class PresentationModule {}
```

**Fix**: Import `CqrsModule` only once in the unified module.

### Mistake 2: Expecting Exported Handlers to Work Across Modules

```typescript
// ❌ Bad: Exporting handlers doesn't make them available across modules
@Module({
  imports: [CqrsModule],
  providers: [MyQueryHandler],
  exports: [MyQueryHandler],  // ← Won't help with QueryBus
})
export class AppModule {}
```

**Why**: QueryBus uses internal registry tied to the CqrsModule instance, not NestJS dependency injection exports.

### Mistake 3: File/Directory Name Conflicts

```typescript
// ❌ Bad: Both file and directory exist
queries/
├── get-foo.query.ts           // ← Class A: GetFooQuery
└── get-foo/
    └── get-foo.query.ts       // ← Class B: GetFooQuery (different instance!)

// Controller imports Class A
import { GetFooQuery } from './queries/get-foo.query';

// Module registers handler for Class B
import { GetFooHandler } from './queries/get-foo';

// @QueryHandler(GetFooQuery) in handler uses Class B
// Controller executes query with Class A
// QueryBus can't match Class A to Class B → Error!
```

**Fix**: Delete one version. Keep either all files OR all directories, never both.

---

## Testing

### Unit Test: Verify Handler Registration

```typescript
import { Test } from '@nestjs/testing';
import { CqrsModule, QueryBus } from '@nestjs/cqrs';
import { ScadaModule } from './scada.module';
import { GetScadaConnectionsQuery } from '../../application/scada/queries';

describe('ScadaModule Handler Registration', () => {
  it('should register GetScadaConnectionsQuery handler', async () => {
    const moduleRef = await Test.createTestingModule({
      imports: [ScadaModule],
    }).compile();

    const queryBus = moduleRef.get<QueryBus>(QueryBus);
    const query = new GetScadaConnectionsQuery('tenant-123');

    // Should not throw "No handler found" error
    await expect(queryBus.execute(query)).resolves.toBeDefined();
  });
});
```

### Integration Test: Verify Controller → Handler Flow

```typescript
describe('SCADA API (e2e)', () => {
  it('GET /api/scada/connections should return connections', async () => {
    const response = await request(app.getHttpServer())
      .get('/api/scada/connections')
      .set('Authorization', `Bearer ${token}`)
      .set('X-Tenant-Subdomain', 'demo')
      .expect(200);

    expect(response.body).toBeInstanceOf(Array);
  });
});
```

---

## Related Patterns

- **[05 - CQRS Pattern](./05-CQRS-Pattern.md)** - Command Query Responsibility Segregation
- **[03 - Hexagonal Architecture](./03-Hexagonal-Architecture.md)** - Ports and adapters architecture
- **[37 - Backend Patterns Integration Architecture](./37-Backend-Patterns-Integration-Architecture.md)** - Backend pattern combination guide
- **[16 - Pattern Integration Guide](./16-Pattern-Integration-Guide.md)** - How to combine patterns

---

## References

- [NestJS CQRS Documentation](https://docs.nestjs.com/recipes/cqrs)
- [NestJS Module Documentation](https://docs.nestjs.com/modules)
- [WellOS SCADA Implementation](../../apps/api/src/presentation/scada/)

---

## Changelog

- **2025-10-31**: Initial pattern documentation based on real-world debugging session
- **Issue**: GetScadaConnectionsQuery handler not found despite proper registration
- **Root Cause**: Split modules created separate QueryBus instances
- **Resolution**: Merged into unified module pattern

---

## Contributors

- Pattern documented after debugging WellOS SCADA module handler registration failure
- Identified through systematic analysis of NestJS CQRS module scoping behavior
