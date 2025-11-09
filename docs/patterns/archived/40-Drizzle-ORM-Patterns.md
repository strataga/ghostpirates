# Drizzle ORM Patterns and Best Practices

## Overview

Drizzle ORM is a lightweight, TypeScript-first ORM that provides type-safe database access with minimal overhead. Unlike traditional ORMs, Drizzle focuses on SQL-like syntax while maintaining full type safety.

## Core Principles

### 1. Type-Safe Schemas

**Key Benefit**: Drizzle generates TypeScript types directly from schema definitions, eliminating ORM/domain type drift.

```typescript
// Schema definition
export const projectsTable = pgTable('projects', {
  id: text('id')
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),
  name: text('name').notNull(),
  budget: decimal('budget', { precision: 12, scale: 2 }),
  status: projectStatusEnum('status').notNull().default('ACTIVE'),
  createdAt: timestamp('created_at', { mode: 'date' }).notNull().defaultNow(),
});

// Types automatically inferred
export type Project = typeof projectsTable.$inferSelect;
export type NewProject = typeof projectsTable.$inferInsert;

// Usage - fully type-safe
const project: Project = {
  id: '123',
  name: 'ACME Project',
  budget: '50000.00',
  status: 'ACTIVE',
  createdAt: new Date(),
};

const newProject: NewProject = {
  name: 'New Project',
  budget: '25000.00',
  // id, createdAt, status have defaults - optional
};
```

**Why This Matters**:

- Compile-time errors if you try to access non-existent columns
- Autocomplete for all database fields
- No manual type definitions needed
- Single source of truth for database structure

### 2. Migration Generation

**Key Benefit**: Drizzle compares current schema to database state and generates SQL migrations automatically.

```bash
# Development workflow
pnpm db:push              # Push schema changes directly (dev only)

# Production workflow
pnpm db:generate          # Generate migration SQL files
pnpm db:migrate           # Apply migrations to database
```

**Migration Files Generated**:

```sql
-- drizzle/0013_add_projects_table.sql
CREATE TABLE IF NOT EXISTS "projects" (
  "id" text PRIMARY KEY NOT NULL,
  "name" text NOT NULL,
  "budget" numeric(12, 2),
  "status" "project_status" DEFAULT 'ACTIVE' NOT NULL,
  "created_at" timestamp DEFAULT now() NOT NULL
);

CREATE INDEX IF NOT EXISTS "projects_status_idx" ON "projects" ("status");
```

**Best Practices**:

- Use `db:push` during rapid development (no migration files)
- Switch to `db:generate` + `db:migrate` before production
- Always review generated SQL before applying
- Name migrations descriptively: `--name add_projects_table`

### 3. Relations

**Key Benefit**: Define relationships in schema for type-safe joins and eager loading.

```typescript
// Define relations
export const projectsRelations = relations(projectsTable, ({ one, many }) => ({
  organization: one(organizationsTable, {
    fields: [projectsTable.organizationId],
    references: [organizationsTable.id],
  }),
  assignments: many(projectAssignmentsTable),
  timeEntries: many(timeEntriesTable),
}));

// Type-safe queries with relations
const projectWithTeam = await db.query.projects.findFirst({
  where: eq(projects.id, projectId),
  with: {
    organization: true,
    assignments: {
      with: {
        user: true, // Nested relations
      },
    },
  },
});

// Result is fully typed
projectWithTeam.organization.name; // ✅ Type-safe
projectWithTeam.assignments[0].user.email; // ✅ Type-safe
```

## Schema Design Patterns

### Pattern 1: Enums for State Machines

Define enums in schema that match domain enums:

```typescript
// Domain enum
export enum ProjectStatus {
  ACTIVE = 'ACTIVE',
  ARCHIVED = 'ARCHIVED',
}

// Database enum (matches exactly)
export const projectStatusEnum = pgEnum('project_status', ['ACTIVE', 'ARCHIVED']);

// Use in schema
export const projectsTable = pgTable('projects', {
  status: projectStatusEnum('status').notNull().default('ACTIVE'),
});
```

**Benefits**:

- Database-level validation
- Type-safe queries: `eq(projects.status, 'ACTIVE')`
- Prevents invalid status values in DB

### Pattern 2: Timestamp Modes

Drizzle supports two timestamp modes:

```typescript
// Mode: 'string' - Returns ISO strings
timestamp('created_at', { mode: 'string' });

// Mode: 'date' - Returns Date objects (recommended)
timestamp('created_at', { mode: 'date' });
```

**Best Practice**: Use `mode: 'date'` for consistency with domain entities:

```typescript
export const projectsTable = pgTable('projects', {
  createdAt: timestamp('created_at', { mode: 'date' }).notNull().defaultNow(),
  updatedAt: timestamp('updated_at', { mode: 'date' }).notNull().defaultNow(),
});

// Domain entity receives Date objects
const project = Project.fromPersistence({
  createdAt: row.createdAt, // Already a Date object
});
```

### Pattern 3: Default Values

Use `$defaultFn()` for dynamic defaults:

```typescript
export const projectsTable = pgTable('projects', {
  // UUID generation
  id: text('id')
    .primaryKey()
    .$defaultFn(() => crypto.randomUUID()),

  // Timestamp defaults
  createdAt: timestamp('created_at', { mode: 'date' }).notNull().defaultNow(),

  // Static defaults
  status: projectStatusEnum('status').notNull().default('ACTIVE'),
});
```

### Pattern 4: Soft Delete Pattern

Standard soft delete implementation:

```typescript
export const projectsTable = pgTable('projects', {
  // ... other fields
  deletedAt: timestamp('deleted_at', { mode: 'date' }),
  deletedBy: text('deleted_by').references((): any => usersTable.id),
});

// Query active records only
const activeProjects = await db.select().from(projects).where(isNull(projects.deletedAt));

// Soft delete
await db
  .update(projects)
  .set({
    deletedAt: new Date(),
    deletedBy: userId,
  })
  .where(eq(projects.id, projectId));
```

### Pattern 5: Indexes and Constraints

Define indexes and constraints inline:

```typescript
export const projectsTable = pgTable(
  'projects',
  {
    id: text('id').primaryKey(),
    organizationId: text('organization_id').notNull(),
    slug: text('slug').notNull(),
    name: text('name').notNull(),
    status: projectStatusEnum('status').notNull(),
  },
  (table) => ({
    // Simple indexes
    orgIdIdx: index('projects_org_id_idx').on(table.organizationId),
    statusIdx: index('projects_status_idx').on(table.status),

    // Composite indexes
    orgStatusIdx: index('projects_org_status_idx').on(table.organizationId, table.status),

    // Unique constraints
    uniqueSlug: unique('unique_project_slug').on(table.organizationId, table.slug),
    uniqueName: unique('unique_project_name').on(table.organizationId, table.name),
  }),
);
```

**Index Naming Convention**:

- Simple index: `{table}_{column}_idx`
- Composite index: `{table}_{col1}_{col2}_idx`
- Unique constraint: `unique_{description}`

### Pattern 6: Foreign Keys with Cascade

```typescript
export const timeEntriesTable = pgTable('time_entries', {
  userId: text('user_id')
    .notNull()
    .references(() => usersTable.id, { onDelete: 'cascade' }),

  organizationId: text('organization_id')
    .notNull()
    .references(() => organizationsTable.id, { onDelete: 'cascade' }),

  projectId: text('project_id').references(() => projectsTable.id, { onDelete: 'restrict' }),
});
```

**Cascade Options**:

- `cascade` - Delete child records when parent is deleted
- `restrict` - Prevent parent deletion if children exist
- `set null` - Set foreign key to null when parent is deleted
- `set default` - Set foreign key to default when parent is deleted

**Best Practices**:

- Use `cascade` for owned relationships (user → tokens)
- Use `restrict` for important data (project → time entries)
- Use `set null` for optional references

### Pattern 7: Self-Referencing Tables

Handle self-references with type assertion:

```typescript
export const usersTable = pgTable('users', {
  id: text('id').primaryKey(),
  deletedBy: text('deleted_by').references((): any => usersTable.id),
  // `: any` needed because TypeScript can't infer self-reference
});
```

## Query Patterns

### Pattern 1: Repository Query Methods

Encapsulate common queries in repository methods:

```typescript
export class DrizzleProjectRepository implements IProjectRepository {
  async findActiveProjects(
    organizationId: string,
    pagination?: PaginationOptions,
  ): Promise<PaginatedResult<Project>> {
    const query = this.db
      .select()
      .from(projects)
      .where(
        and(
          eq(projects.organizationId, organizationId),
          eq(projects.status, 'ACTIVE'),
          isNull(projects.deletedAt),
        ),
      );

    // Apply pagination
    if (pagination) {
      query.limit(pagination.limit).offset(pagination.page * pagination.limit);
    }

    const rows = await query;

    return {
      data: rows.map((row) => this.toDomain(row)),
      total: await this.countActiveProjects(organizationId),
      page: pagination?.page || 0,
      limit: pagination?.limit || rows.length,
    };
  }
}
```

### Pattern 2: Conditional Queries

Build dynamic queries with conditional logic:

```typescript
async findAll(filters: TimeEntryFilters): Promise<TimeEntry[]> {
  const conditions = [];

  if (filters.userId) {
    conditions.push(eq(timeEntries.userId, filters.userId));
  }

  if (filters.projectId) {
    conditions.push(eq(timeEntries.projectId, filters.projectId));
  }

  if (filters.statuses && filters.statuses.length > 0) {
    conditions.push(inArray(timeEntries.status, filters.statuses));
  }

  if (filters.startDateFrom) {
    conditions.push(gte(timeEntries.startTime, filters.startDateFrom));
  }

  if (!filters.includeDeleted) {
    conditions.push(isNull(timeEntries.deletedAt));
  }

  const rows = await this.db
    .select()
    .from(timeEntries)
    .where(and(...conditions));

  return rows.map(row => this.toDomain(row));
}
```

### Pattern 3: Transactions

Use transactions for multi-table operations:

```typescript
async assignUserToProject(
  projectId: string,
  userId: string,
  hourlyRate: number | null,
): Promise<void> {
  await this.db.transaction(async (tx) => {
    // Create assignment
    await tx.insert(projectAssignments).values({
      id: crypto.randomUUID(),
      projectId,
      userId,
      hourlyRate,
      assignedAt: new Date(),
      assignedBy: currentUserId,
    });

    // Update project metadata
    await tx
      .update(projects)
      .set({ updatedAt: new Date() })
      .where(eq(projects.id, projectId));
  });
}
```

### Pattern 4: Aggregations

```typescript
async calculateStats(
  organizationId: string,
): Promise<ProjectStats> {
  const result = await this.db
    .select({
      totalProjects: count(),
      activeProjects: count(
        sql`CASE WHEN ${projects.status} = 'ACTIVE' THEN 1 END`,
      ),
      totalBudget: sum(projects.budget),
      averageBudget: avg(projects.budget),
    })
    .from(projects)
    .where(
      and(
        eq(projects.organizationId, organizationId),
        isNull(projects.deletedAt),
      ),
    );

  return {
    totalProjects: result[0].totalProjects || 0,
    activeProjects: result[0].activeProjects || 0,
    totalBudget: Number(result[0].totalBudget || 0),
    averageBudget: Number(result[0].averageBudget || 0),
  };
}
```

## Domain/Infrastructure Mapping

### Pattern: Mapper Functions

Separate domain and infrastructure concerns with mapper functions:

```typescript
export class DrizzleProjectRepository {
  // Database → Domain
  private toDomain(row: typeof projectsTable.$inferSelect): Project {
    return Project.fromPersistence({
      id: row.id,
      organizationId: row.organizationId,
      clientId: row.clientId,
      name: row.name,
      slug: ProjectSlug.fromString(row.slug),
      description: row.description,
      budget: row.budget ? Money.fromAmount(Number(row.budget)) : null,
      defaultHourlyRate: row.defaultHourlyRate
        ? HourlyRate.fromAmount(Number(row.defaultHourlyRate))
        : null,
      status: row.status as ProjectStatus,
      startDate: row.startDate,
      endDate: row.endDate,
      createdAt: row.createdAt,
      updatedAt: row.updatedAt,
      createdBy: row.createdBy,
      updatedBy: row.updatedBy,
      deletedAt: row.deletedAt,
      deletedBy: row.deletedBy,
    });
  }

  // Domain → Database
  private toDatabase(project: Project): typeof projectsTable.$inferInsert {
    return {
      id: project.id,
      organizationId: project.organizationId,
      clientId: project.clientId,
      name: project.name,
      slug: project.slug.value,
      description: project.description,
      budget: project.budget?.amount.toString() || null,
      defaultHourlyRate: project.defaultHourlyRate?.amount.toString() || null,
      status: project.status,
      startDate: project.startDate,
      endDate: project.endDate,
      createdAt: project.createdAt,
      updatedAt: project.updatedAt,
      createdBy: project.createdBy,
      updatedBy: project.updatedBy,
      deletedAt: project.deletedAt,
      deletedBy: project.deletedBy,
    };
  }

  async save(project: Project): Promise<Project> {
    const data = this.toDatabase(project);

    const [row] = await this.db
      .insert(projects)
      .values(data)
      .onConflictDoUpdate({
        target: projects.id,
        set: data,
      })
      .returning();

    return this.toDomain(row);
  }
}
```

## Best Practices

### 1. Schema Organization

```
infrastructure/database/schema/
├── index.ts                    # Export all schemas
├── users.schema.ts             # User-related tables
├── organizations.schema.ts     # Organization tables
├── projects.schema.ts          # Projects + assignments
├── time-entries.schema.ts      # Time tracking
└── invoices.schema.ts          # Billing (Sprint 5)
```

### 2. Enum Consistency

Keep database enums in sync with domain enums:

```typescript
// Domain
export enum ProjectStatus {
  ACTIVE = 'ACTIVE',
  ARCHIVED = 'ARCHIVED',
}

// Database (must match exactly)
export const projectStatusEnum = pgEnum('project_status', ['ACTIVE', 'ARCHIVED']);
```

### 3. Migration Naming

Use descriptive migration names:

```bash
# Good
pnpm db:generate --name add_projects_table
pnpm db:generate --name add_time_entry_status_column
pnpm db:generate --name create_project_assignments_unique_index

# Bad
pnpm db:generate --name update
pnpm db:generate --name fix
```

### 4. Decimal Precision

Always specify precision for monetary values:

```typescript
// ✅ Good - Explicit precision
budget: decimal('budget', { precision: 12, scale: 2 });

// ❌ Bad - Default precision may vary
budget: decimal('budget');
```

### 5. Timestamp Zones

Always use timezone-aware timestamps:

```typescript
// ✅ Good - Timezone aware
timestamp('created_at', { mode: 'date', withTimezone: true });

// ❌ Bad - No timezone (ambiguous)
timestamp('created_at', { mode: 'date' });
```

## Common Pitfalls

### Pitfall 1: Partial Unique Indexes

Drizzle doesn't support `.where()` on unique constraints:

```typescript
// ❌ Not supported
unique('unique_slug').on(table.slug).where(`deleted_at IS NULL`)

// ✅ Solution: Add manually in migration
// Migration SQL:
CREATE UNIQUE INDEX unique_project_slug
ON projects (organization_id, slug)
WHERE deleted_at IS NULL;
```

### Pitfall 2: Type Mismatch with Decimals

Drizzle returns decimals as strings:

```typescript
// Database returns string
const row = await db.select().from(projects).where(eq(projects.id, id));
row.budget; // Type: string | null

// Convert to domain type
const budget = row.budget ? Money.fromAmount(Number(row.budget)) : null;
```

### Pitfall 3: Self-Reference Type Errors

Self-referencing foreign keys need type assertion:

```typescript
// ❌ Type error
deletedBy: text('deleted_by').references(() => usersTable.id);

// ✅ Works
deletedBy: text('deleted_by').references((): any => usersTable.id);
```

### Pitfall 4: Migration Drift

Always generate migrations before pushing to production:

```bash
# ❌ Development workflow (no migrations)
pnpm db:push

# ✅ Production workflow (migrations tracked)
pnpm db:generate --name descriptive_name
pnpm db:migrate
git add drizzle/
git commit -m "feat: add projects table migration"
```

## Testing with Drizzle

### Pattern: Test Database Setup

```typescript
import { drizzle } from 'drizzle-orm/postgres-js';
import postgres from 'postgres';
import * as schema from './schema';

describe('ProjectRepository', () => {
  let db: ReturnType<typeof drizzle>;
  let connection: ReturnType<typeof postgres>;

  beforeAll(async () => {
    connection = postgres(process.env.DATABASE_URL_TEST);
    db = drizzle(connection, { schema });

    // Run migrations
    await migrate(db, { migrationsFolder: './drizzle' });
  });

  afterAll(async () => {
    await connection.end();
  });

  beforeEach(async () => {
    // Clean database between tests
    await db.delete(projects);
  });

  it('should save project', async () => {
    const repository = new DrizzleProjectRepository(db);
    const project = Project.create({
      /* ... */
    });

    const saved = await repository.save(project);

    expect(saved.id).toBe(project.id);
  });
});
```

## Resources

- [Drizzle ORM Documentation](https://orm.drizzle.team/docs/overview)
- [Drizzle Kit Documentation](https://orm.drizzle.team/kit-docs/overview)
- [PostgreSQL Data Types](https://www.postgresql.org/docs/current/datatype.html)

---

**Summary**: Drizzle ORM provides type-safe database access with minimal overhead. By following these patterns, you can build maintainable, performant data access layers that stay in sync with your domain models.
