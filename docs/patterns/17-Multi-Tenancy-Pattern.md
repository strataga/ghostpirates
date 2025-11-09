# Multi-Tenancy Pattern

## Overview

Multi-tenancy is an architecture pattern where a single instance of software serves multiple tenants (customers/organizations). Each tenant's data is isolated and invisible to other tenants. This pattern is essential for SaaS applications where multiple organizations share the same infrastructure while maintaining complete data separation.

## Core Concepts

### Tenant

An isolated customer or organization using the application. In WellOS, this is the **Organization** entity.

### Tenant Isolation

Ensuring that one tenant's data is completely separated from another tenant's data.

### Tenant Context

The current tenant scope for a request, determining which data can be accessed.

### Multi-Tenancy Models

1. **Database per Tenant**: Separate database for each tenant (highest isolation, highest cost)
2. **Schema per Tenant**: Separate schema for each tenant in shared database (good isolation, moderate cost)
3. **Row-Level Security**: Shared tables with tenant ID column (lowest cost, requires careful implementation)

## Benefits

- **Cost Efficiency**: Shared infrastructure reduces per-tenant costs
- **Scalability**: Easy to onboard new tenants
- **Maintenance**: Single codebase, easier updates
- **Resource Optimization**: Efficient use of computing resources
- **Rapid Deployment**: Quick tenant provisioning

## Implementation Strategy for WellOS

WellOS uses **Row-Level Security (RLS)** with **Organization-based tenancy**:

- Shared PostgreSQL database
- `organizationId` column on all tenant-scoped tables
- PostgreSQL Row-Level Security policies
- Domain-based organization assignment
- First user creates organization

## Before: No Multi-Tenancy

```typescript
// ❌ POOR: Global data access, no tenant isolation

@Injectable()
export class ProjectService {
  constructor(private readonly db: Database) {}

  async getAllProjects(): Promise<Project[]> {
    // Returns ALL projects from ALL customers - security breach!
    return await this.db.select().from(projects);
  }

  async getProject(id: string): Promise<Project> {
    // No tenant check - user can access any project
    return await this.db.select().from(projects).where(eq(projects.id, id)).limit(1);
  }

  async createProject(data: CreateProjectDto): Promise<Project> {
    // No organization context
    return await this.db.insert(projects).values(data).returning();
  }
}

// Schema without tenant isolation
export const projects = pgTable('projects', {
  id: varchar('id', { length: 255 }).primaryKey(),
  name: varchar('name', { length: 255 }).notNull(),
  budget: decimal('budget'),
  // No organizationId - all data is global!
});
```

## After: Multi-Tenancy with Row-Level Security

### 1. Domain Layer - Organization Aggregate

```typescript
// Organization aggregate root
export class Organization {
  private constructor(
    private readonly id: OrganizationId,
    private name: OrganizationName,
    private slug: OrganizationSlug,
    private domain: OrganizationDomain,
    private billingTier: BillingTier,
    private settings: OrganizationSettings,
    private readonly createdAt: Date,
    private updatedAt: Date,
    private readonly domainEvents: DomainEvent[] = [],
  ) {}

  // Factory method - first user creates organization
  static createFromFirstUser(data: CreateOrganizationData): Organization {
    const org = new Organization(
      OrganizationId.generate(),
      new OrganizationName(data.name),
      OrganizationSlug.fromEmail(data.ownerEmail),
      OrganizationDomain.fromEmail(data.ownerEmail),
      BillingTier.FREE, // Start with free tier
      OrganizationSettings.default(),
      new Date(),
      new Date(),
    );

    org.addDomainEvent(
      new OrganizationCreatedEvent(
        org.id,
        org.name.getValue(),
        org.domain.getValue(),
        data.ownerEmail,
      ),
    );

    return org;
  }

  // Business logic - upgrade billing tier
  upgradeTier(newTier: BillingTier, currentUserCount: number): void {
    if (this.billingTier.isHigherThan(newTier)) {
      throw new OrganizationDomainError('Cannot downgrade tier, use downgradeTier()');
    }

    if (!newTier.supportsUserCount(currentUserCount)) {
      throw new OrganizationDomainError(
        `Tier ${newTier.name} only supports ${newTier.maxUsers} users`,
      );
    }

    this.billingTier = newTier;
    this.updatedAt = new Date();

    this.addDomainEvent(new BillingTierChangedEvent(this.id, newTier));
  }

  // Domain validation - can user join organization?
  canUserJoin(userEmail: Email): boolean {
    return this.domain.matches(userEmail);
  }

  getId(): OrganizationId {
    return this.id;
  }

  getDomain(): OrganizationDomain {
    return this.domain;
  }

  private addDomainEvent(event: DomainEvent): void {
    this.domainEvents.push(event);
  }

  getDomainEvents(): readonly DomainEvent[] {
    return this.domainEvents;
  }

  clearDomainEvents(): void {
    this.domainEvents.length = 0;
  }
}

// Value Objects
export class OrganizationDomain {
  private constructor(private readonly value: string) {}

  static fromEmail(email: string): OrganizationDomain {
    const domain = email.split('@')[1].toLowerCase();

    // Domain blacklist - prevent public email providers
    const blacklist = [
      'gmail.com',
      'yahoo.com',
      'hotmail.com',
      'outlook.com',
      'icloud.com',
      'protonmail.com',
      'aol.com',
    ];

    if (blacklist.includes(domain)) {
      throw new OrganizationDomainError(
        `Public email domain ${domain} not allowed. Use company email.`,
      );
    }

    return new OrganizationDomain(domain);
  }

  matches(email: Email): boolean {
    const emailDomain = email.getValue().split('@')[1].toLowerCase();
    return emailDomain === this.value;
  }

  getValue(): string {
    return this.value;
  }
}

export class BillingTier {
  static readonly FREE = new BillingTier('FREE', 0, 5, 3, 10);
  static readonly STARTER = new BillingTier('STARTER', 10, 20, 10, 50);
  static readonly PRO = new BillingTier('PRO', 25, 100, 50, 500);
  static readonly ENTERPRISE = new BillingTier('ENTERPRISE', 50, Infinity, Infinity, Infinity);

  private constructor(
    public readonly name: string,
    public readonly pricePerUser: number,
    public readonly maxUsers: number,
    public readonly maxProjects: number,
    public readonly maxClients: number,
  ) {}

  supportsUserCount(count: number): boolean {
    return count <= this.maxUsers;
  }

  isHigherThan(other: BillingTier): boolean {
    const tiers = [BillingTier.FREE, BillingTier.STARTER, BillingTier.PRO, BillingTier.ENTERPRISE];
    return tiers.indexOf(this) > tiers.indexOf(other);
  }
}
```

### 2. Infrastructure Layer - Tenant-Aware Repository

```typescript
// Base repository with tenant isolation
export abstract class BaseTenantRepository<T> {
  constructor(protected readonly db: NodePgDatabase<typeof schema>) {}

  // Enforce tenant context on all queries
  protected withTenantFilter(query: any, organizationId: string) {
    return query.where(eq(this.getTable().organizationId, organizationId));
  }

  protected abstract getTable(): any;

  // Tenant-scoped find
  protected async findByIdInTenant(id: string, organizationId: string): Promise<T | null> {
    const result = await this.db
      .select()
      .from(this.getTable())
      .where(and(eq(this.getTable().id, id), eq(this.getTable().organizationId, organizationId)))
      .limit(1);

    return result[0] ? this.mapToDomain(result[0]) : null;
  }

  // Tenant-scoped list
  protected async findAllInTenant(organizationId: string): Promise<T[]> {
    const results = await this.db
      .select()
      .from(this.getTable())
      .where(eq(this.getTable().organizationId, organizationId));

    return results.map((row) => this.mapToDomain(row));
  }

  protected abstract mapToDomain(row: any): T;
}

// Organization Repository Implementation
@Injectable()
export class DrizzleOrganizationRepository implements IOrganizationRepository {
  constructor(
    @Inject('DATABASE_CONNECTION')
    private readonly db: NodePgDatabase<typeof schema>,
  ) {}

  async findById(id: OrganizationId): Promise<Organization | null> {
    const result = await this.db
      .select()
      .from(organizations)
      .where(eq(organizations.id, id.getValue()))
      .limit(1);

    return result[0] ? this.mapToDomain(result[0]) : null;
  }

  async findByDomain(domain: OrganizationDomain): Promise<Organization | null> {
    const result = await this.db
      .select()
      .from(organizations)
      .where(eq(organizations.domain, domain.getValue()))
      .limit(1);

    return result[0] ? this.mapToDomain(result[0]) : null;
  }

  async save(organization: Organization): Promise<void> {
    const data = organization.toPersistence();

    const existing = await this.findById(organization.getId());

    if (existing) {
      await this.db
        .update(organizations)
        .set({
          name: data.name,
          billingTier: data.billingTier,
          settings: data.settings,
          updatedAt: data.updatedAt,
        })
        .where(eq(organizations.id, data.id));
    } else {
      await this.db.insert(organizations).values(data);
    }
  }

  private mapToDomain(row: any): Organization {
    return Organization.fromPersistence({
      id: row.id,
      name: row.name,
      slug: row.slug,
      domain: row.domain,
      billingTier: row.billingTier,
      settings: row.settings,
      createdAt: row.createdAt,
      updatedAt: row.updatedAt,
    });
  }
}

// Tenant-aware Project Repository
@Injectable()
export class ProjectRepository extends BaseTenantRepository<Project> {
  protected getTable() {
    return projects;
  }

  async findById(id: string, organizationId: string): Promise<Project | null> {
    return this.findByIdInTenant(id, organizationId);
  }

  async findByOrganization(organizationId: string): Promise<Project[]> {
    return this.findAllInTenant(organizationId);
  }

  protected mapToDomain(row: any): Project {
    return Project.fromPersistence(row);
  }
}
```

### 3. Database Schema - Row-Level Security

```sql
-- SQL migrations with tenant isolation
CREATE TABLE organizations (
  id VARCHAR(255) PRIMARY KEY,
  name VARCHAR(255) NOT NULL,
  slug VARCHAR(255) NOT NULL UNIQUE,
  domain VARCHAR(255) NOT NULL UNIQUE,
  billing_tier VARCHAR(50) NOT NULL DEFAULT 'FREE',
  settings JSONB NOT NULL DEFAULT '{}',
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMP,
  deleted_by VARCHAR(255)
);

-- All tenant-scoped tables have organization_id
CREATE TABLE projects (
  id VARCHAR(255) PRIMARY KEY,
  organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id),
  name VARCHAR(255) NOT NULL,
  budget DECIMAL,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
  deleted_at TIMESTAMP,
  deleted_by VARCHAR(255)
);

-- Index for tenant queries
CREATE INDEX projects_org_id_idx ON projects(organization_id);

-- Users belong to organizations
CREATE TABLE users (
  id VARCHAR(255) PRIMARY KEY,
  organization_id VARCHAR(255) NOT NULL REFERENCES organizations(id),
  email VARCHAR(255) NOT NULL UNIQUE
  -- ... other fields
);
```

### 4. PostgreSQL Row-Level Security Policies

```sql
-- Enable RLS on tenant-scoped tables
ALTER TABLE projects ENABLE ROW LEVEL SECURITY;
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE time_entries ENABLE ROW LEVEL SECURITY;

-- RLS Policy: Users can only see data from their organization
CREATE POLICY projects_tenant_isolation ON projects
  FOR ALL
  USING (organization_id = current_setting('app.current_organization_id')::varchar);

CREATE POLICY users_tenant_isolation ON users
  FOR ALL
  USING (organization_id = current_setting('app.current_organization_id')::varchar);

-- Super admin bypass (for support)
CREATE POLICY projects_superadmin_access ON projects
  FOR ALL
  USING (
    EXISTS (
      SELECT 1 FROM users
      WHERE users.id = current_setting('app.current_user_id')::varchar
      AND users.role = 'SUPER_ADMIN'
    )
  );
```

### 5. Tenant Context Middleware

```typescript
// Extract tenant context from JWT
@Injectable()
export class TenantContextMiddleware implements NestMiddleware {
  constructor(private readonly jwtService: JwtService) {}

  async use(req: Request, res: Response, next: NextFunction) {
    try {
      const token = this.extractTokenFromHeader(req);

      if (token) {
        const payload = await this.jwtService.verifyAsync(token);

        // Set tenant context from JWT payload
        req['organizationId'] = payload.organizationId;
        req['userId'] = payload.userId;

        // Set PostgreSQL session variables for RLS
        if (req['db']) {
          await req['db'].execute(sql`SET app.current_organization_id = ${payload.organizationId}`);
          await req['db'].execute(sql`SET app.current_user_id = ${payload.userId}`);
        }
      }
    } catch (error) {
      // Invalid token - proceed without context
    }

    next();
  }

  private extractTokenFromHeader(request: Request): string | undefined {
    const [type, token] = request.headers.authorization?.split(' ') ?? [];
    return type === 'Bearer' ? token : undefined;
  }
}

// Tenant context decorator
export const TenantContext = createParamDecorator(
  (data: unknown, ctx: ExecutionContext): string => {
    const request = ctx.switchToHttp().getRequest();
    const organizationId = request['organizationId'];

    if (!organizationId) {
      throw new UnauthorizedException('No tenant context');
    }

    return organizationId;
  },
);

// Usage in controllers
@Controller('projects')
export class ProjectController {
  constructor(private readonly commandBus: CommandBus) {}

  @Get()
  async getProjects(@TenantContext() organizationId: string) {
    // organizationId automatically extracted from JWT
    return this.commandBus.execute(new GetProjectsQuery(organizationId));
  }

  @Post()
  async createProject(@TenantContext() organizationId: string, @Body() dto: CreateProjectDto) {
    // Automatically scoped to tenant
    return this.commandBus.execute(new CreateProjectCommand(organizationId, dto.name, dto.budget));
  }
}
```

### 6. Domain-Based Organization Assignment

```typescript
// Command: Register user with organization detection
export class RegisterUserCommand {
  constructor(
    public readonly email: string,
    public readonly password: string,
    public readonly firstName: string,
    public readonly lastName: string,
  ) {}
}

@CommandHandler(RegisterUserCommand)
export class RegisterUserHandler implements ICommandHandler<RegisterUserCommand> {
  constructor(
    private readonly userRepository: IUserRepository,
    private readonly organizationRepository: IOrganizationRepository,
    private readonly eventBus: EventBus,
  ) {}

  async execute(command: RegisterUserCommand): Promise<string> {
    const email = new Email(command.email);
    const domain = OrganizationDomain.fromEmail(command.email);

    // Check if organization exists for this domain
    let organization = await this.organizationRepository.findByDomain(domain);

    if (!organization) {
      // First user with this domain - create organization
      organization = Organization.createFromFirstUser({
        name: domain.getValue(), // e.g., "acmecorp.com" -> "Acmecorp"
        ownerEmail: command.email,
      });

      await this.organizationRepository.save(organization);
    }

    // Create user in organization
    const user = User.register({
      email: command.email,
      password: command.password,
      firstName: command.firstName,
      lastName: command.lastName,
      organizationId: organization.getId().getValue(),
      role: organization.isFirstUser() ? UserRole.OWNER : UserRole.MEMBER,
    });

    await this.userRepository.save(user);

    // Publish events
    const events = [...organization.getDomainEvents(), ...user.getDomainEvents()];
    for (const event of events) {
      this.eventBus.publish(event);
    }

    return user.getId().getValue();
  }
}
```

### 7. Multi-Tenant Testing

```typescript
// Test helpers for multi-tenancy
export class TenantTestHelper {
  static async createTestOrganization(
    db: Database,
    data?: Partial<OrganizationData>,
  ): Promise<Organization> {
    const org = Organization.createFromFirstUser({
      name: data?.name || 'Test Org',
      ownerEmail: data?.ownerEmail || 'owner@test.com',
    });

    await db.insert(organizations).values(org.toPersistence());
    return org;
  }

  static async createTestUser(
    db: Database,
    organizationId: string,
    data?: Partial<UserData>,
  ): Promise<User> {
    const user = User.create({
      email: data?.email || 'test@test.com',
      organizationId,
      ...data,
    });

    await db.insert(users).values(user.toPersistence());
    return user;
  }

  static async setTenantContext(db: Database, organizationId: string, userId: string) {
    await db.execute(sql`SET app.current_organization_id = ${organizationId}`);
    await db.execute(sql`SET app.current_user_id = ${userId}`);
  }
}

// Test with tenant isolation
describe('Project Repository - Tenant Isolation', () => {
  let db: Database;
  let org1: Organization;
  let org2: Organization;
  let user1: User;
  let user2: User;

  beforeEach(async () => {
    db = await createTestDatabase();

    // Create two separate organizations
    org1 = await TenantTestHelper.createTestOrganization(db, {
      ownerEmail: 'owner@org1.com',
    });
    org2 = await TenantTestHelper.createTestOrganization(db, {
      ownerEmail: 'owner@org2.com',
    });

    user1 = await TenantTestHelper.createTestUser(db, org1.getId().getValue());
    user2 = await TenantTestHelper.createTestUser(db, org2.getId().getValue());
  });

  it('should isolate projects by organization', async () => {
    const repository = new ProjectRepository(db);

    // Create project in org1
    const project1 = Project.create({
      organizationId: org1.getId().getValue(),
      name: 'Org 1 Project',
    });
    await repository.save(project1, org1.getId().getValue());

    // Create project in org2
    const project2 = Project.create({
      organizationId: org2.getId().getValue(),
      name: 'Org 2 Project',
    });
    await repository.save(project2, org2.getId().getValue());

    // Verify: Org1 user can only see org1 projects
    await TenantTestHelper.setTenantContext(db, org1.getId().getValue(), user1.getId());
    const org1Projects = await repository.findByOrganization(org1.getId().getValue());
    expect(org1Projects).toHaveLength(1);
    expect(org1Projects[0].getName()).toBe('Org 1 Project');

    // Verify: Org2 user can only see org2 projects
    await TenantTestHelper.setTenantContext(db, org2.getId().getValue(), user2.getId());
    const org2Projects = await repository.findByOrganization(org2.getId().getValue());
    expect(org2Projects).toHaveLength(1);
    expect(org2Projects[0].getName()).toBe('Org 2 Project');
  });

  it('should prevent cross-tenant data access', async () => {
    const repository = new ProjectRepository(db);

    // Org1 creates a project
    const project = Project.create({
      organizationId: org1.getId().getValue(),
      name: 'Secret Project',
    });
    await repository.save(project, org1.getId().getValue());

    // Org2 user tries to access org1's project
    await TenantTestHelper.setTenantContext(db, org2.getId().getValue(), user2.getId());

    const result = await repository.findById(
      project.getId().getValue(),
      org2.getId().getValue(), // Wrong org ID
    );

    // Should return null - RLS prevents access
    expect(result).toBeNull();
  });
});
```

## Multi-Tenancy Best Practices

### 1. Always Scope Queries by Tenant

```typescript
// ✅ Good: Explicit tenant scoping
async findProjects(organizationId: string): Promise<Project[]> {
  return await this.db
    .select()
    .from(projects)
    .where(eq(projects.organizationId, organizationId));
}

// ❌ Bad: No tenant scoping
async findProjects(): Promise<Project[]> {
  return await this.db.select().from(projects); // Returns ALL tenants' data!
}
```

### 2. Use Base Repository for Consistency

```typescript
// ✅ Good: Consistent tenant filtering
export abstract class BaseTenantRepository<T> {
  protected async findAllInTenant(organizationId: string): Promise<T[]> {
    return this.withTenantFilter(this.db.select().from(this.getTable()), organizationId);
  }
}

// All repositories inherit tenant isolation
export class ProjectRepository extends BaseTenantRepository<Project> {}
export class ClientRepository extends BaseTenantRepository<Client> {}
```

### 3. Validate Tenant Context

```typescript
// ✅ Good: Validate before operations
@Injectable()
export class TenantGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest();

    if (!request.organizationId) {
      throw new UnauthorizedException('No tenant context');
    }

    return true;
  }
}

@Controller('projects')
@UseGuards(TenantGuard)
export class ProjectController {
  // All routes require valid tenant context
}
```

### 4. Use Domain Events for Cross-Tenant Operations

```typescript
// ✅ Good: Event-driven cross-tenant logic
@EventsHandler(UserInvitedToOrganizationEvent)
export class UserInvitedEventHandler {
  async handle(event: UserInvitedToOrganizationEvent) {
    // Send email to user in invited organization's context
    await this.emailService.sendInvitation(event.invitedUserEmail, event.organizationId);
  }
}
```

## Performance Considerations

### 1. Index Organization ID

```typescript
// Always index organizationId for fast queries
export const projectsOrgIdIndex = index('projects_org_id_idx').on(projects.organizationId);
export const usersOrgIdIndex = index('users_org_id_idx').on(users.organizationId);
```

### 2. Composite Indexes

```typescript
// Composite index for common queries
export const projectsOrgStatusIndex = index('projects_org_status_idx').on(
  projects.organizationId,
  projects.status,
);
```

### 3. Partition Large Tables

```sql
-- Partition by organization for very large datasets
CREATE TABLE projects_partitioned (
  id VARCHAR(255) PRIMARY KEY,
  organization_id VARCHAR(255) NOT NULL,
  -- ... other fields
) PARTITION BY LIST (organization_id);

-- Create partition per tenant (for very large customers)
CREATE TABLE projects_org_1 PARTITION OF projects_partitioned
  FOR VALUES IN ('org-1-id');
```

## Security Considerations

### 1. Defense in Depth

- **Application Layer**: Validate tenant in repositories
- **Database Layer**: RLS policies as last line of defense
- **Network Layer**: JWT with tenant context

### 2. Audit Logging

```typescript
// Log all cross-tenant access attempts
@Injectable()
export class TenantAuditInterceptor implements NestInterceptor {
  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const request = context.switchToHttp().getRequest();
    const organizationId = request.organizationId;

    return next.handle().pipe(
      tap(() => {
        this.auditLog.log({
          userId: request.userId,
          organizationId,
          action: request.method,
          resource: request.url,
          timestamp: new Date(),
        });
      }),
    );
  }
}
```

## When to Use Multi-Tenancy

### Good Fit

- **SaaS Applications**: Multiple customers sharing infrastructure
- **B2B Platforms**: Each business is a separate tenant
- **Cost Optimization**: Need to minimize per-customer infrastructure
- **Rapid Scaling**: Quick onboarding of new customers

### Not Recommended

- **High Regulatory Requirements**: Banking, healthcare (may need database-per-tenant)
- **Massive Scale per Tenant**: If one tenant needs dedicated resources
- **Custom Per-Tenant Logic**: Significant customization per tenant

Multi-tenancy in WellOS enables cost-effective SaaS operation while maintaining complete data isolation between consulting firms using organization-based tenancy with Row-Level Security.
