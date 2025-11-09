# Repository Pattern

## Overview

The Repository pattern encapsulates the logic needed to access data sources. It
provides a uniform interface for accessing domain objects while hiding the
complexities of the underlying storage mechanism. This pattern promotes
separation of concerns and makes the code more testable and maintainable.

## Core Concepts

### Repository Interface

Defines the contract for data access operations without exposing implementation
details.

### Repository Implementation

Concrete implementation that handles the actual data storage and retrieval.

### Domain Objects

Business entities that the repository manages.

### Query Abstraction

Methods that express business intent rather than technical data access patterns.

## Benefits

- **Testability**: Easy to mock for unit testing
- **Flexibility**: Can switch between different storage mechanisms
- **Separation of Concerns**: Business logic separated from data access
- **Consistency**: Uniform interface for data operations
- **Caching**: Centralized location for caching strategies
- **Query Optimization**: Repository can optimize queries for specific use cases

## Implementation in Our Project

### Before: Direct Database Access

```rust
// ❌ POOR: Direct database access mixed with business logic
pub struct VendorService {
    pool: PgPool,
}

impl VendorService {
    pub async fn create_vendor(&self, dto: CreateVendorDto) -> Result<VendorDto> {
        // Direct database access mixed with business logic
        let existing = sqlx::query!(
            "SELECT id FROM vendors WHERE code = $1 LIMIT 1",
            dto.code
        )
        .fetch_optional(&self.pool)
        .await?;

        if existing.is_some() {
            return Err(Error::Conflict("Vendor code already exists".into()));
        }

        // Complex mapping logic
        let contact_info = serde_json::to_value(&dto.contact_info)?;
        let insurance = serde_json::to_value(&dto.insurance)?;

        // Raw SQL operations
        let result = sqlx::query!(
            r#"
            INSERT INTO vendors (id, organization_id, name, code, status, contact_info, insurance, created_at, updated_at)
            VALUES ($1, $2, $3, $4, 'PENDING', $5, $6, NOW(), NOW())
            RETURNING id, name, code, status, contact_info, insurance, created_at, updated_at
            "#,
            Uuid::new_v4(),
            dto.organization_id,
            dto.name,
            dto.code,
            contact_info,
            insurance
        )
        .fetch_one(&self.pool)
        .await?;

        // Manual mapping back to DTO
        Ok(VendorDto {
            id: result.id,
            name: result.name,
            code: result.code,
            status: result.status,
            contact_info: serde_json::from_value(result.contact_info)?,
            insurance: serde_json::from_value(result.insurance)?,
            created_at: result.created_at,
            updated_at: result.updated_at,
        })
    }

    pub async fn get_active_vendors_by_organization(&self, org_id: Uuid) -> Result<Vec<VendorDto>> {
        // Complex query logic scattered throughout service layer
        let results = sqlx::query!(
            r#"
            SELECT
                v.id,
                v.name,
                v.code,
                v.status,
                v.contact_info,
                (SELECT MAX(payment_date) FROM payments WHERE vendor_id = v.id) as last_payment_date,
                (SELECT COUNT(*) FROM contracts WHERE vendor_id = v.id AND status = 'ACTIVE') as "contract_count!"
            FROM vendors v
            WHERE v.organization_id = $1 AND v.status = 'ACTIVE'
            "#,
            org_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Manual mapping
        results.into_iter()
            .map(|row| Ok(VendorDto {
                id: row.id,
                name: row.name,
                code: row.code,
                status: row.status,
                contact_info: serde_json::from_value(row.contact_info)?,
                last_payment_date: row.last_payment_date,
                contract_count: row.contract_count,
            }))
            .collect()
    }
}
```

### After: Repository Pattern

```rust
use sqlx::{PgPool, FromRow, types::Json};
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Domain Repository Trait (Interface)
#[async_trait]
pub trait VendorRepository: Send + Sync {
    // Basic CRUD operations
    async fn find_by_id(&self, id: VendorId) -> Result<Option<Vendor>>;
    async fn find_by_code(&self, code: &VendorCode) -> Result<Option<Vendor>>;
    async fn save(&self, vendor: &Vendor) -> Result<()>;
    async fn delete(&self, id: VendorId) -> Result<()>;

    // Business-specific queries
    async fn find_active_vendors_by_organization(&self, organization_id: Uuid) -> Result<Vec<Vendor>>;
    async fn find_vendors_with_expired_insurance(&self, organization_id: Uuid) -> Result<Vec<Vendor>>;
    async fn find_vendors_by_status(&self, organization_id: Uuid, status: VendorStatus) -> Result<Vec<Vendor>>;

    // Complex domain queries
    async fn exists_by_code_in_organization(&self, code: &VendorCode, organization_id: Uuid) -> Result<bool>;
    async fn find_vendors_for_payment(&self, organization_id: Uuid, min_amount: Money) -> Result<Vec<Vendor>>;

    // Pagination support
    async fn find_by_organization_paginated(
        &self,
        organization_id: Uuid,
        options: PaginationOptions,
    ) -> Result<PaginatedResult<Vendor>>;
}

// Database row struct with #[derive(FromRow)]
#[derive(FromRow)]
struct VendorRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    code: String,
    status: String,
    contact_info: Json<ContactInfo>,
    insurance: Json<Insurance>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

// Infrastructure Implementation
pub struct PostgresVendorRepository {
    pool: PgPool,
}

impl PostgresVendorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // Private mapping methods
    fn map_to_domain(&self, row: VendorRow) -> Result<Vendor> {
        Vendor::from_persistence(VendorPersistence {
            id: row.id,
            organization_id: row.organization_id,
            name: row.name,
            code: row.code,
            status: VendorStatus::from_str(&row.status)?,
            contact_info: row.contact_info.0,
            insurance: row.insurance.0,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl VendorRepository for PostgresVendorRepository {
    async fn find_by_id(&self, id: VendorId) -> Result<Option<Vendor>> {
        let row = sqlx::query_as::<_, VendorRow>(
            "SELECT * FROM vendors WHERE id = $1 LIMIT 1"
        )
        .bind(id.value())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| self.map_to_domain(r)).transpose()
    }

    async fn find_by_code(&self, code: &VendorCode) -> Result<Option<Vendor>> {
        let row = sqlx::query_as::<_, VendorRow>(
            "SELECT * FROM vendors WHERE code = $1 LIMIT 1"
        )
        .bind(code.value())
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| self.map_to_domain(r)).transpose()
    }

    async fn save(&self, vendor: &Vendor) -> Result<()> {
        let persistence = vendor.to_persistence();

        // Check if exists
        let existing = self.find_by_id(vendor.id()).await?;

        if existing.is_some() {
            // Update
            sqlx::query!(
                r#"
                UPDATE vendors
                SET name = $2, contact_info = $3, insurance = $4,
                    status = $5, updated_at = $6
                WHERE id = $1
                "#,
                persistence.id,
                persistence.name,
                Json(persistence.contact_info) as _,
                Json(persistence.insurance) as _,
                persistence.status.to_string(),
                persistence.updated_at
            )
            .execute(&self.pool)
            .await?;
        } else {
            // Insert
            sqlx::query!(
                r#"
                INSERT INTO vendors (id, organization_id, name, code, status, contact_info, insurance, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                persistence.id,
                persistence.organization_id,
                persistence.name,
                persistence.code,
                persistence.status.to_string(),
                Json(persistence.contact_info) as _,
                Json(persistence.insurance) as _,
                persistence.created_at,
                persistence.updated_at
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    async fn find_active_vendors_by_organization(&self, organization_id: Uuid) -> Result<Vec<Vendor>> {
        let rows = sqlx::query_as::<_, VendorRow>(
            r#"
            SELECT * FROM vendors
            WHERE organization_id = $1 AND status = 'ACTIVE'
            ORDER BY name
            "#
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.map_to_domain(row))
            .collect()
    }

    async fn find_vendors_with_expired_insurance(&self, organization_id: Uuid) -> Result<Vec<Vendor>> {
        let rows = sqlx::query_as::<_, VendorRow>(
            r#"
            SELECT * FROM vendors
            WHERE organization_id = $1
              AND (insurance->>'expiryDate')::date <= CURRENT_DATE
            "#
        )
        .bind(organization_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.map_to_domain(row))
            .collect()
    }

    async fn exists_by_code_in_organization(&self, code: &VendorCode, organization_id: Uuid) -> Result<bool> {
        let result = sqlx::query!(
            "SELECT COUNT(*) as count FROM vendors WHERE code = $1 AND organization_id = $2",
            code.value(),
            organization_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.count.unwrap_or(0) > 0)
    }

    async fn find_by_organization_paginated(
        &self,
        organization_id: Uuid,
        options: PaginationOptions,
    ) -> Result<PaginatedResult<Vendor>> {
        // Count total
        let count_result = sqlx::query!(
            "SELECT COUNT(*) as count FROM vendors WHERE organization_id = $1",
            organization_id
        )
        .fetch_one(&self.pool)
        .await?;

        let total = count_result.count.unwrap_or(0) as usize;

        // Fetch page
        let rows = sqlx::query_as::<_, VendorRow>(
            r#"
            SELECT * FROM vendors
            WHERE organization_id = $1
            ORDER BY name
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(organization_id)
        .bind(options.limit as i64)
        .bind(options.offset as i64)
        .fetch_all(&self.pool)
        .await?;

        let items: Result<Vec<Vendor>> = rows
            .into_iter()
            .map(|row| self.map_to_domain(row))
            .collect();

        Ok(PaginatedResult {
            items: items?,
            total,
            limit: options.limit,
            offset: options.offset,
        })
    }
}

// Clean Service Layer
pub struct CreateVendorHandler {
    vendor_repository: Arc<dyn VendorRepository>,
    event_bus: Arc<EventBus>,
}

impl CreateVendorHandler {
    pub async fn execute(&self, command: CreateVendorCommand) -> Result<Uuid> {
        // Business logic focused on domain concepts
        let vendor_code = VendorCode::new(command.code)?;

        // Repository handles the complex query logic
        if self.vendor_repository
            .exists_by_code_in_organization(&vendor_code, command.organization_id)
            .await?
        {
            return Err(Error::VendorCodeAlreadyExists(vendor_code.value().to_string()));
        }

        // Create domain entity
        let vendor = Vendor::create(CreateVendorData {
            organization_id: command.organization_id,
            name: command.name,
            code: command.code,
            contact_info: command.contact_info,
            insurance: command.insurance,
        })?;

        // Repository handles persistence complexity
        self.vendor_repository.save(&vendor).await?;

        // Domain events
        for event in vendor.domain_events() {
            self.event_bus.publish(event).await?;
        }

        Ok(vendor.id().value())
    }
}
```

## Advanced Repository Patterns

### Specification Pattern Integration

```typescript
// Specification for complex queries
export abstract class VendorSpecification {
  abstract isSatisfiedBy(vendor: Vendor): boolean;
  abstract toQuery(): QueryCondition;
}

export class ActiveVendorWithValidInsuranceSpecification extends VendorSpecification {
  isSatisfiedBy(vendor: Vendor): boolean {
    return vendor.isActive() && vendor.hasValidInsurance();
  }

  toQuery(): QueryCondition {
    return and(
      eq(vendors.status, VendorStatus.ACTIVE),
      gte(sql`(${vendors.insurance}->>'expiryDate')::date`, new Date()),
    );
  }
}

// Repository with specification support
export interface IVendorRepository {
  findBySpecification(
    specification: VendorSpecification,
    organizationId: string,
  ): Promise<Vendor[]>;
}

@Injectable()
export class VendorRepository implements IVendorRepository {
  async findBySpecification(
    specification: VendorSpecification,
    organizationId: string,
  ): Promise<Vendor[]> {
    const queryCondition = specification.toQuery();

    const results = await this.db
      .select()
      .from(vendors)
      .where(and(eq(vendors.organizationId, organizationId), queryCondition));

    return results.map((row) => this.mapToDomainEntity(row));
  }
}
```

### Unit of Work Integration

```typescript
export interface IUnitOfWork {
  vendorRepository: IVendorRepository;
  contractRepository: IContractRepository;
  paymentRepository: IPaymentRepository;

  commit(): Promise<void>;
  rollback(): Promise<void>;
}

@Injectable()
export class UnitOfWork implements IUnitOfWork {
  private transaction: any;

  constructor(@Inject('DATABASE_CONNECTION') private readonly db: Database) {}

  get vendorRepository(): IVendorRepository {
    return new TransactionalVendorRepository(this.db, this.transaction);
  }

  get contractRepository(): IContractRepository {
    return new TransactionalContractRepository(this.db, this.transaction);
  }

  async commit(): Promise<void> {
    if (this.transaction) {
      await this.transaction.commit();
      this.transaction = null;
    }
  }

  async rollback(): Promise<void> {
    if (this.transaction) {
      await this.transaction.rollback();
      this.transaction = null;
    }
  }

  async begin(): Promise<void> {
    this.transaction = await this.db.transaction();
  }
}

// Usage in application service
@Injectable()
export class ActivateVendorHandler {
  constructor(private readonly unitOfWork: IUnitOfWork) {}

  async execute(command: ActivateVendorCommand): Promise<void> {
    await this.unitOfWork.begin();

    try {
      const vendor = await this.unitOfWork.vendorRepository.findById(
        new VendorId(command.vendorId),
      );

      if (!vendor) {
        throw new VendorNotFoundError(command.vendorId);
      }

      vendor.activate();

      // Update vendor
      await this.unitOfWork.vendorRepository.save(vendor);

      // Activate related contracts
      const contracts = await this.unitOfWork.contractRepository.findByVendorId(vendor.getId());

      for (const contract of contracts) {
        contract.activate();
        await this.unitOfWork.contractRepository.save(contract);
      }

      await this.unitOfWork.commit();
    } catch (error) {
      await this.unitOfWork.rollback();
      throw error;
    }
  }
}
```

### Read/Write Repository Separation

```typescript
// Write-optimized repository
export interface IVendorWriteRepository {
  save(vendor: Vendor): Promise<void>;
  delete(id: VendorId): Promise<void>;
}

// Read-optimized repository
export interface IVendorReadRepository {
  findById(id: string): Promise<VendorReadModel | null>;
  findByOrganization(organizationId: string, filters?: VendorFilters): Promise<VendorReadModel[]>;

  // Specialized read operations
  getVendorSummary(organizationId: string): Promise<VendorSummaryDto>;
  getVendorPaymentHistory(vendorId: string): Promise<PaymentHistoryDto[]>;
  searchVendors(organizationId: string, searchTerm: string): Promise<VendorSearchResultDto[]>;
}

@Injectable()
export class VendorReadRepository implements IVendorReadRepository {
  constructor(@Inject('DATABASE_CONNECTION') private readonly db: Database) {}

  async findByOrganization(
    organizationId: string,
    filters?: VendorFilters,
  ): Promise<VendorReadModel[]> {
    let query = this.db
      .select({
        id: vendors.id,
        name: vendors.name,
        code: vendors.code,
        status: vendors.status,
        contactEmail: sql`${vendors.contactInfo}->>'email'`,
        lastPaymentDate: sql`(
          SELECT MAX(payment_date)
          FROM payments
          WHERE vendor_id = ${vendors.id}
        )`,
        totalPaid: sql`(
          SELECT COALESCE(SUM(amount), 0)
          FROM payments
          WHERE vendor_id = ${vendors.id}
        )`,
        activeContractCount: sql`(
          SELECT COUNT(*)
          FROM contracts
          WHERE vendor_id = ${vendors.id} AND status = 'ACTIVE'
        )`,
      })
      .from(vendors)
      .where(eq(vendors.organizationId, organizationId));

    // Apply filters
    if (filters?.status) {
      query = query.where(eq(vendors.status, filters.status));
    }

    if (filters?.searchTerm) {
      query = query.where(
        or(
          ilike(vendors.name, `%${filters.searchTerm}%`),
          ilike(vendors.code, `%${filters.searchTerm}%`),
        ),
      );
    }

    return await query;
  }

  async getVendorSummary(organizationId: string): Promise<VendorSummaryDto> {
    const [statusCounts, paymentStats] = await Promise.all([
      this.getVendorStatusCounts(organizationId),
      this.getPaymentStatistics(organizationId),
    ]);

    return {
      totalVendors: statusCounts.total,
      activeVendors: statusCounts.active,
      pendingVendors: statusCounts.pending,
      totalAmountPaid: paymentStats.totalPaid,
      averagePaymentAmount: paymentStats.averageAmount,
    };
  }

  private async getVendorStatusCounts(organizationId: string) {
    const result = await this.db
      .select({
        status: vendors.status,
        count: count(),
      })
      .from(vendors)
      .where(eq(vendors.organizationId, organizationId))
      .groupBy(vendors.status);

    return result.reduce(
      (acc, row) => {
        acc[row.status] = row.count;
        acc.total += row.count;
        return acc;
      },
      { total: 0, active: 0, pending: 0, blocked: 0 },
    );
  }
}
```

## Repository Testing

### Unit Testing Repository Interface

```typescript
describe('VendorRepository', () => {
  let repository: VendorRepository;
  let mockDb: jest.Mocked<Database>;

  beforeEach(() => {
    mockDb = createMockDatabase();
    repository = new VendorRepository(mockDb);
  });

  describe('findById', () => {
    it('should return vendor when found', async () => {
      // Given
      const vendorId = new VendorId('vendor-123');
      const mockDbResult = [createMockVendorDbRow()];
      mockDb.select.mockReturnValue({
        from: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        limit: jest.fn().mockResolvedValue(mockDbResult),
      } as any);

      // When
      const result = await repository.findById(vendorId);

      // Then
      expect(result).toBeInstanceOf(Vendor);
      expect(result?.getId()).toEqual(vendorId);
      expect(mockDb.select).toHaveBeenCalled();
    });

    it('should return null when not found', async () => {
      // Given
      const vendorId = new VendorId('non-existent');
      mockDb.select.mockReturnValue({
        from: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        limit: jest.fn().mockResolvedValue([]),
      } as any);

      // When
      const result = await repository.findById(vendorId);

      // Then
      expect(result).toBeNull();
    });
  });

  describe('save', () => {
    it('should create new vendor when not exists', async () => {
      // Given
      const vendor = createMockVendor();
      jest.spyOn(repository, 'findById').mockResolvedValue(null);
      mockDb.insert.mockReturnValue({
        values: jest.fn().mockResolvedValue(undefined),
      } as any);

      // When
      await repository.save(vendor);

      // Then
      expect(mockDb.insert).toHaveBeenCalled();
    });

    it('should update existing vendor', async () => {
      // Given
      const vendor = createMockVendor();
      jest.spyOn(repository, 'findById').mockResolvedValue(vendor);
      mockDb.update.mockReturnValue({
        set: jest.fn().mockReturnThis(),
        where: jest.fn().mockResolvedValue(undefined),
      } as any);

      // When
      await repository.save(vendor);

      // Then
      expect(mockDb.update).toHaveBeenCalled();
    });
  });

  describe('findActiveVendorsByOrganization', () => {
    it('should return active vendors for organization', async () => {
      // Given
      const organizationId = 'org-123';
      const mockDbResults = [
        createMockVendorDbRow({ status: VendorStatus.ACTIVE }),
        createMockVendorDbRow({ status: VendorStatus.ACTIVE }),
      ];

      mockDb.select.mockReturnValue({
        from: jest.fn().mockReturnThis(),
        where: jest.fn().mockReturnThis(),
        orderBy: jest.fn().mockResolvedValue(mockDbResults),
      } as any);

      // When
      const results = await repository.findActiveVendorsByOrganization(organizationId);

      // Then
      expect(results).toHaveLength(2);
      expect(results.every((v) => v.isActive())).toBe(true);
    });
  });
});
```

### Integration Testing with Real Database

```typescript
describe('VendorRepository Integration', () => {
  let repository: VendorRepository;
  let testDb: Database;

  beforeAll(async () => {
    testDb = await createTestDatabase();
    repository = new VendorRepository(testDb);
  });

  afterAll(async () => {
    await testDb.close();
  });

  beforeEach(async () => {
    await cleanDatabase(testDb);
  });

  it('should persist and retrieve vendor', async () => {
    // Given
    const vendor = Vendor.create({
      organizationId: 'test-org',
      name: 'Test Vendor',
      code: 'TEST-01',
      contactInfo: createValidContactInfo(),
      insurance: createValidInsurance(),
    });

    // When
    await repository.save(vendor);
    const retrieved = await repository.findById(vendor.getId());

    // Then
    expect(retrieved).toBeTruthy();
    expect(retrieved?.getName().getValue()).toBe('Test Vendor');
    expect(retrieved?.getCode().getValue()).toBe('TEST-01');
  });

  it('should handle concurrent saves correctly', async () => {
    // Given
    const vendor = createTestVendor();
    await repository.save(vendor);

    // When - Simulate concurrent updates
    const [vendor1, vendor2] = await Promise.all([
      repository.findById(vendor.getId()),
      repository.findById(vendor.getId()),
    ]);

    vendor1?.updateName(new VendorName('Updated Name 1'));
    vendor2?.updateName(new VendorName('Updated Name 2'));

    // Then - Should handle optimistic concurrency appropriately
    await repository.save(vendor1!);
    await expect(repository.save(vendor2!)).rejects.toThrow();
  });
});
```

## Best Practices

### Implementation Strategy (SQLx + Rust)

When implementing repositories with SQLx in Rust (not Drizzle ORM), follow these proven patterns:

#### 1. Mapper Functions

**Use `#[derive(FromRow)]` for automatic row mapping, with manual conversion for value objects.**

Value objects (Money, Duration, ProjectSlug) need explicit conversion between domain and database representations:

```rust
use sqlx::{FromRow, PgPool, types::BigDecimal};
use uuid::Uuid;

// Database row struct - SQLx automatically maps columns to fields
#[derive(FromRow)]
struct ProjectRow {
    id: Uuid,
    name: String,
    slug: String,
    budget: Option<BigDecimal>,  // PostgreSQL DECIMAL mapped to BigDecimal
    default_hourly_rate: Option<BigDecimal>,
    status: String,
    // ... other fields
}

pub struct PostgresProjectRepository {
    pool: PgPool,
}

impl PostgresProjectRepository {
    /**
     * Convert database row to domain entity
     */
    fn map_to_domain(&self, row: ProjectRow) -> Result<Project> {
        Project::from_persistence(ProjectPersistence {
            id: row.id,
            name: row.name,
            slug: ProjectSlug::from_str(&row.slug)?, // String → Value Object
            budget: row.budget.map(|b| Money::from_decimal(b)).transpose()?, // BigDecimal → Money
            default_hourly_rate: row.default_hourly_rate
                .map(|rate| HourlyRate::from_decimal(rate))
                .transpose()?, // BigDecimal → HourlyRate
            status: ProjectStatus::from_str(&row.status)?,
            // ... other fields
        })
    }

    /**
     * Convert domain entity to database values
     */
    fn to_persistence(&self, project: &Project) -> ProjectPersistence {
        ProjectPersistence {
            id: project.id(),
            name: project.name().to_string(),
            slug: project.slug().value().to_string(), // Value Object → String
            budget: project.budget().map(|m| m.to_decimal()), // Money → BigDecimal
            default_hourly_rate: project.default_hourly_rate()
                .map(|rate| rate.to_decimal()), // HourlyRate → BigDecimal
            status: project.status().to_string(),
            // ... other fields
        }
    }

    pub async fn save(&self, project: &Project) -> Result<Project> {
        let data = self.to_persistence(project); // Domain → DB

        let row = sqlx::query_as::<_, ProjectRow>(
            r#"
            INSERT INTO projects (id, name, slug, budget, default_hourly_rate, status)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                slug = EXCLUDED.slug,
                budget = EXCLUDED.budget,
                default_hourly_rate = EXCLUDED.default_hourly_rate,
                status = EXCLUDED.status
            RETURNING *
            "#
        )
        .bind(data.id)
        .bind(&data.name)
        .bind(&data.slug)
        .bind(data.budget)
        .bind(data.default_hourly_rate)
        .bind(&data.status)
        .fetch_one(&self.pool)
        .await?;

        self.map_to_domain(row) // DB → Domain
    }
}
```

**Why This Matters**:

- Domain entities remain pure (no database concerns)
- Value objects enforce business rules during conversion
- Type safety at boundaries (compile-time errors if fields mismatch)
- Single responsibility (mappers only handle translation)

#### 2. Type Conversions

**SQLx returns PostgreSQL DECIMAL as BigDecimal in Rust; convert to domain types (Money, HourlyRate).**

PostgreSQL `DECIMAL` and `NUMERIC` columns require conversion from `BigDecimal` to domain value objects:

```rust
use sqlx::types::BigDecimal;

// Database schema (SQL)
// CREATE TABLE projects (
//   budget DECIMAL(12,2),
//   default_hourly_rate DECIMAL(10,2)
// );

// ❌ Common mistake - Using BigDecimal directly in domain
let budget: Option<BigDecimal> = row.budget;

// ✅ Correct - Convert to domain type
let budget: Option<Money> = row.budget
    .map(|b| Money::from_decimal(b))
    .transpose()?;

// ✅ Correct - Convert to value object
let rate: Option<HourlyRate> = row.default_hourly_rate
    .map(|r| HourlyRate::from_decimal(r))
    .transpose()?;
```

**Decimal Handling Checklist**:

- ✅ Always convert `BigDecimal` columns to domain value objects (Money, HourlyRate)
- ✅ Always convert domain value objects to `BigDecimal` before saving with SQLx
- ✅ Use proper formatting when displaying to users (2 decimal places for money)
- ✅ Never do math on `BigDecimal` directly (convert to domain value objects first)

**Example: Time Entry with Duration (SQLx + Rust)**:

```rust
use sqlx::FromRow;

#[derive(FromRow)]
struct TimeEntryRow {
    id: Uuid,
    duration: Option<i32>,  // Seconds as integer
    hourly_rate: Option<BigDecimal>,  // PostgreSQL DECIMAL
    // ... other fields
}

impl TimeEntryRepository {
    fn map_to_domain(&self, row: TimeEntryRow) -> Result<TimeEntry> {
        TimeEntry::from_persistence(TimeEntryPersistence {
            id: row.id,
            duration: row.duration
                .map(|seconds| Duration::from_seconds(seconds))  // i32 → Value Object
                .transpose()?,
            hourly_rate: row.hourly_rate
                .map(|rate| HourlyRate::from_decimal(rate))  // BigDecimal → Value Object
                .transpose()?,
            // ...
        })
    }

    fn to_persistence(&self, entry: &TimeEntry) -> TimeEntryPersistence {
        TimeEntryPersistence {
            id: entry.id(),
            duration: entry.duration().map(|d| d.seconds()),  // Value Object → i32
            hourly_rate: entry.hourly_rate().map(|r| r.to_decimal()),  // Value Object → BigDecimal
            // ...
        }
    }
}
```

#### 3. Soft Delete Filtering

**Always filter `deleted_at IS NULL` in SQLx queries unless explicitly including deleted records.**

Soft delete is a cross-cutting concern that must be consistently applied:

```rust
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

// ✅ Correct - Always filter soft-deleted records
pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>> {
    let row = sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT * FROM projects
        WHERE id = $1 AND deleted_at IS NULL
        LIMIT 1
        "#
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    row.map(|r| self.map_to_domain(r)).transpose()
}

// ✅ Correct - Conditional soft delete filtering
pub async fn find_all(&self, filters: &ProjectFilters) -> Result<Vec<Project>> {
    let mut query = String::from("SELECT * FROM projects WHERE 1=1");

    if !filters.include_deleted {
        query.push_str(" AND deleted_at IS NULL");  // Apply by default
    }

    let rows = sqlx::query_as::<_, ProjectRow>(&query)
        .fetch_all(&self.pool)
        .await?;

    rows.into_iter()
        .map(|row| self.map_to_domain(row))
        .collect()
}

// ❌ Common mistake - Forgetting soft delete filter
pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>> {
    let row = sqlx::query_as::<_, ProjectRow>(
        "SELECT * FROM projects WHERE id = $1 LIMIT 1"  // BUG: Returns deleted records!
    )
    .bind(id)
    .fetch_optional(&self.pool)
    .await?;

    row.map(|r| self.map_to_domain(r)).transpose()
}
```

**Soft Delete Best Practices**:

1. **Always filter** `deleted_at IS NULL` unless `include_deleted` flag is true
2. **Update soft delete** using `UPDATE SET deleted_at = NOW(), deleted_by = $1`
3. **Never hard delete** (use soft delete for audit trail)
4. **Cascade consideration**: Soft-deleting parent should cascade to children (handled at application layer)

**Example: Soft Delete Implementation (SQLx + Rust)**:

```rust
pub async fn delete(&self, id: Uuid, deleted_by: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE projects
        SET deleted_at = NOW(),
            deleted_by = $2,
            updated_at = NOW()
        WHERE id = $1
        "#,
        id,
        deleted_by
    )
    .execute(&self.pool)
    .await?;

    // No hard delete - preserve for audit trail
    Ok(())
}

// Restore from soft delete (if needed)
pub async fn restore(&self, id: Uuid) -> Result<()> {
    sqlx::query!(
        r#"
        UPDATE projects
        SET deleted_at = NULL,
            deleted_by = NULL,
            updated_at = NOW()
        WHERE id = $1
        "#,
        id
    )
    .execute(&self.pool)
    .await?;

    Ok(())
}
```

### 1. Interface Segregation

```typescript
// Separate interfaces for different needs
export interface IVendorReadRepository {
  findById(id: string): Promise<VendorReadModel | null>;
  search(criteria: SearchCriteria): Promise<VendorReadModel[]>;
}

export interface IVendorWriteRepository {
  save(vendor: Vendor): Promise<void>;
  delete(id: VendorId): Promise<void>;
}

// Combined interface when both are needed
export interface IVendorRepository extends IVendorReadRepository, IVendorWriteRepository {}
```

### 2. Business-Focused Methods

```typescript
// Good: Business intent is clear
interface IVendorRepository {
  findVendorsRequiringInsuranceRenewal(days: number): Promise<Vendor[]>;
  findHighRiskVendors(organizationId: string): Promise<Vendor[]>;
  findVendorsEligibleForPayment(minAmount: Money): Promise<Vendor[]>;
}

// Avoid: Technical operations exposed
interface IVendorRepository {
  selectWithJoins(tables: string[], conditions: any[]): Promise<any[]>;
  executeCustomQuery(sql: string, params: any[]): Promise<any[]>;
}
```

### 3. Consistent Error Handling

```typescript
export abstract class RepositoryError extends Error {
  constructor(
    message: string,
    public readonly cause?: Error,
  ) {
    super(message);
    this.name = this.constructor.name;
  }
}

export class VendorNotFoundError extends RepositoryError {
  constructor(vendorId: string) {
    super(`Vendor with ID ${vendorId} not found`);
  }
}

export class VendorPersistenceError extends RepositoryError {
  constructor(operation: string, cause?: Error) {
    super(`Failed to ${operation} vendor`, cause);
  }
}
```

### 4. Caching Strategy

```typescript
@Injectable()
export class CachedVendorRepository implements IVendorRepository {
  constructor(
    private readonly baseRepository: IVendorRepository,
    private readonly cache: ICacheService,
  ) {}

  async findById(id: VendorId): Promise<Vendor | null> {
    const cacheKey = `vendor:${id.getValue()}`;

    // Try cache first
    const cached = await this.cache.get(cacheKey);
    if (cached) {
      return Vendor.fromCached(cached);
    }

    // Fall back to repository
    const vendor = await this.baseRepository.findById(id);

    // Cache for future use
    if (vendor) {
      await this.cache.set(cacheKey, vendor.toCached(), { ttl: 300 });
    }

    return vendor;
  }

  async save(vendor: Vendor): Promise<void> {
    await this.baseRepository.save(vendor);

    // Invalidate cache
    const cacheKey = `vendor:${vendor.getId().getValue()}`;
    await this.cache.delete(cacheKey);
  }
}
```

## Anti-Patterns to Avoid

### 1. Leaky Abstraction

```typescript
// DON'T: Exposing database-specific details
interface IVendorRepository {
  findByQuery(drizzleQuery: SelectQuery): Promise<Vendor[]>;
  executeTransaction(callback: (tx: Transaction) => Promise<void>): Promise<void>;
}
```

### 2. Generic Repository Anti-Pattern

```typescript
// DON'T: Over-generic repository
interface IGenericRepository<T> {
  find(id: string): Promise<T | null>;
  save(entity: T): Promise<void>;
  delete(id: string): Promise<void>;
  findAll(): Promise<T[]>;
}

// This loses business meaning and forces all entities into same mold
```

### 3. Repository as Data Access Layer

```typescript
// DON'T: Repository doing business logic
class VendorRepository {
  async activateVendor(id: string): Promise<void> {
    const vendor = await this.findById(id);

    // Business logic in repository
    if (vendor.contracts.some((c) => c.isPending())) {
      throw new Error('Cannot activate vendor with pending contracts');
    }

    vendor.status = 'ACTIVE';
    await this.save(vendor);
  }
}
```

The Repository pattern in our oil & gas management system provides a clean
abstraction over complex data access operations, allowing our domain logic to
focus on business rules while keeping persistence concerns properly separated.
