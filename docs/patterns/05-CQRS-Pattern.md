# Command Query Responsibility Segregation (CQRS) Pattern

## Overview

CQRS is a pattern that separates read and write operations for a data store.
Commands handle updates to data, while queries handle reading data. This
separation allows for optimized data models and improved performance,
scalability, and security.

## Core Concepts

### Commands

Operations that change the state of the system but don't return data.

### Queries

Operations that return data but don't change the state of the system.

### Command Handlers

Process commands and coordinate business operations.

### Query Handlers

Process queries and return read-optimized data.

### Separation of Models

Different models optimized for reading vs writing operations.

## Benefits

- **Performance Optimization**: Read and write operations can be optimized
  independently
- **Scalability**: Read and write databases can scale differently
- **Security**: Different permissions for read vs write operations
- **Flexibility**: Different technologies for reads vs writes
- **Complex Business Logic**: Commands can handle complex business rules
- **Simplified Queries**: Queries can be optimized for specific use cases

## Implementation in Our Project

### Before: Mixed Read/Write Operations

```rust
// Mixed concerns - both command and query logic in handlers
pub struct VendorHandler {
    vendor_service: Arc<VendorService>,
}

async fn create_vendor(
    State(app): State<AppState>,
    Json(dto): Json<CreateVendorDto>,
) -> Result<Json<VendorDetailDto>, AppError> {
    // Command operation mixed with query response
    let vendor = app.vendor_service.create(dto).await?;

    // Immediate query to return full data
    let details = app.vendor_service.get_vendor_with_details(vendor.id).await?;
    Ok(Json(details))
}

async fn get_vendor(
    State(app): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<VendorDto>, AppError> {
    // Query operation
    let vendor = app.vendor_service.find_by_id(id).await?;
    Ok(Json(vendor))
}

async fn update_vendor(
    State(app): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateVendorDto>,
) -> Result<Json<VendorDto>, AppError> {
    // Update operation mixed with return
    app.vendor_service.update(id, dto).await?;
    let vendor = app.vendor_service.find_by_id(id).await?; // Immediate read
    Ok(Json(vendor))
}

// Service with mixed responsibilities
pub struct VendorService {
    repository: Arc<dyn VendorRepository>,
}

impl VendorService {
    // Mixed read/write operations
    async fn create(&self, dto: CreateVendorDto) -> Result<VendorEntity, AppError> {
        let vendor = VendorEntity::new(/* ... */);
        let saved = self.repository.save(vendor).await?;

        // Mixed with business logic and queries
        self.audit_service.log_creation(saved.id).await?;
        self.notification_service.notify_created(saved.id).await?;

        Ok(saved)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<VendorDto, AppError> {
        let vendor = self.repository.find_by_id(id).await?;
        // ... mapping to DTO
        Ok(dto)
    }
}
```

### After: CQRS Implementation

```rust
// Commands (Write Side)
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CreateVendorCommand {
    pub organization_id: String,
    pub name: String,
    pub code: String,
    pub contact_info: ContactInfoData,
    pub insurance: InsuranceData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateVendorCommand {
    pub vendor_id: String,
    pub name: Option<String>,
    pub contact_info: Option<ContactInfoData>,
}

// Queries (Read Side)
#[derive(Debug, Clone, Deserialize)]
pub struct GetVendorByIdQuery {
    pub vendor_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetVendorsByOrganizationQuery {
    pub organization_id: String,
    pub filters: Option<VendorFilters>,
    pub pagination: Option<PaginationOptions>,
}

// Command Handlers
pub struct CreateVendorHandler {
    vendor_repository: Arc<dyn VendorRepository>,
    event_bus: Arc<EventBus>,
}

impl CreateVendorHandler {
    pub async fn execute(&self, command: CreateVendorCommand) -> Result<String, AppError> {
        // Focus only on business logic and persistence
        let vendor = Vendor::create(
            command.organization_id,
            command.name,
            command.code,
            command.contact_info,
            command.insurance,
        )?;

        self.vendor_repository.save(&vendor).await?;

        // Publish events for side effects
        let events = vendor.get_domain_events();
        for event in events {
            self.event_bus.publish(event).await?;
        }

        Ok(vendor.get_id().value())
    }
}

pub struct UpdateVendorHandler {
    vendor_repository: Arc<dyn VendorRepository>,
}

impl UpdateVendorHandler {
    pub async fn execute(&self, command: UpdateVendorCommand) -> Result<(), AppError> {
        let mut vendor = self
            .vendor_repository
            .find_by_id(&VendorId::new(&command.vendor_id))
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Vendor {} not found", command.vendor_id)))?;

        // Business logic focused on state changes
        if let Some(name) = command.name {
            vendor.update_name(VendorName::new(name)?)?;
        }

        if let Some(contact_info) = command.contact_info {
            vendor.update_contact_info(ContactInfo::new(contact_info)?)?;
        }

        self.vendor_repository.save(&vendor).await?;
        Ok(())
    }
}

// Query Handlers
pub struct GetVendorByIdHandler {
    vendor_read_repository: Arc<dyn VendorReadRepository>,
}

impl GetVendorByIdHandler {
    pub async fn execute(&self, query: GetVendorByIdQuery) -> Result<VendorDetailDto, AppError> {
        let vendor = self
            .vendor_read_repository
            .find_by_id_with_details(&query.vendor_id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Vendor {} not found", query.vendor_id)))?;

        // Return optimized read model
        Ok(VendorDetailDto {
            id: vendor.id,
            organization_id: vendor.organization_id,
            name: vendor.name,
            code: vendor.code,
            status: vendor.status,
            contact_info: vendor.contact_info,
            insurance: vendor.insurance,
            created_at: vendor.created_at,
            updated_at: vendor.updated_at,
            // Joined data for efficient reading
            active_contracts: vendor.active_contract_count,
            total_paid: vendor.total_amount_paid,
            last_payment_date: vendor.last_payment_date,
        })
    }
}

pub struct GetVendorsByOrganizationHandler {
    vendor_read_repository: Arc<dyn VendorReadRepository>,
}

impl GetVendorsByOrganizationHandler {
    pub async fn execute(
        &self,
        query: GetVendorsByOrganizationQuery,
    ) -> Result<Vec<VendorListDto>, AppError> {
        let vendors = self
            .vendor_read_repository
            .find_by_organization(&query.organization_id, query.filters, query.pagination)
            .await?;

        // Return list-optimized read models
        Ok(vendors
            .into_iter()
            .map(|vendor| VendorListDto {
                id: vendor.id,
                name: vendor.name,
                code: vendor.code,
                status: vendor.status,
                last_activity: vendor.last_activity,
                active_contract_count: vendor.active_contract_count,
            })
            .collect())
    }
}

// Axum Handlers with CQRS
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};

#[derive(Clone)]
pub struct AppState {
    command_bus: Arc<CommandBus>,
    query_bus: Arc<QueryBus>,
}

pub async fn create_vendor(
    State(state): State<AppState>,
    Json(dto): Json<CreateVendorDto>,
) -> Result<Json<CreateVendorResponse>, AppError> {
    // Execute command - only returns ID
    let vendor_id = state
        .command_bus
        .execute_create_vendor(CreateVendorCommand {
            organization_id: dto.organization_id,
            name: dto.name,
            code: dto.code,
            contact_info: dto.contact_info,
            insurance: dto.insurance,
        })
        .await?;

    Ok(Json(CreateVendorResponse {
        id: vendor_id,
        message: "Vendor created successfully".to_string(),
    }))
}

pub async fn get_vendor(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<VendorDetailDto>, AppError> {
    // Execute query
    let vendor = state
        .query_bus
        .execute_get_vendor_by_id(GetVendorByIdQuery { vendor_id: id })
        .await?;

    Ok(Json(vendor))
}

pub async fn get_vendors(
    State(state): State<AppState>,
    Query(params): Query<GetVendorsParams>,
) -> Result<Json<Vec<VendorListDto>>, AppError> {
    let vendors = state
        .query_bus
        .execute_get_vendors(GetVendorsByOrganizationQuery {
            organization_id: params.organization_id,
            filters: params.filters,
            pagination: None,
        })
        .await?;

    Ok(Json(vendors))
}

pub async fn update_vendor(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateVendorDto>,
) -> Result<Json<UpdateVendorResponse>, AppError> {
    // Execute command - no return data
    state
        .command_bus
        .execute_update_vendor(UpdateVendorCommand {
            vendor_id: id,
            name: dto.name,
            contact_info: dto.contact_info,
        })
        .await?;

    Ok(Json(UpdateVendorResponse {
        message: "Vendor updated successfully".to_string(),
    }))
}
```

## Read and Write Models

### Write Model (Domain-Focused)

```rust
// Optimized for business logic and consistency
use chrono::{DateTime, Utc};

pub struct Vendor {
    id: VendorId,
    organization_id: String,
    name: VendorName,
    code: VendorCode,
    status: VendorStatus,
    contact_info: ContactInfo,
    insurance: Insurance,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Vendor {
    // Rich behavior for business operations
    pub fn activate(&mut self) -> Result<(), VendorDomainError> {
        if self.status.is_blocked() {
            return Err(VendorDomainError::CannotActivateBlocked);
        }
        self.status = VendorStatus::Active;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn update_insurance(&mut self, insurance: Insurance) -> Result<(), VendorDomainError> {
        self.validate_insurance_requirements(&insurance)?;
        self.insurance = insurance;
        self.updated_at = Utc::now();
        Ok(())
    }

    fn validate_insurance_requirements(&self, insurance: &Insurance) -> Result<(), VendorDomainError> {
        if self.status.is_active() && !insurance.is_valid() {
            return Err(VendorDomainError::ActiveVendorRequiresValidInsurance);
        }
        Ok(())
    }
}
```

### Read Model (Query-Optimized)

```rust
// Optimized for queries and display
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VendorReadModel {
    pub id: String,
    pub organization_id: String,
    pub name: String,
    pub code: String,
    pub status: String,
    pub contact_email: String,
    pub contact_phone: String,
    pub insurance_expiry: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // Denormalized data for efficient reading
    pub active_contract_count: i32,
    pub total_amount_paid: rust_decimal::Decimal,
    pub last_payment_date: Option<DateTime<Utc>>,
    pub last_activity: DateTime<Utc>,
    pub risk_score: f64,
    pub compliance_status: ComplianceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComplianceStatus {
    Compliant,
    Warning,
    NonCompliant,
}

// Separate repository for reads
#[async_trait]
pub trait VendorReadRepository: Send + Sync {
    async fn find_by_id_with_details(&self, id: &str) -> Result<Option<VendorReadModel>, RepositoryError>;

    async fn find_by_organization(
        &self,
        organization_id: &str,
        filters: Option<VendorFilters>,
        pagination: Option<PaginationOptions>,
    ) -> Result<Vec<VendorReadModel>, RepositoryError>;

    // Specialized queries
    async fn find_expiring_insurance(&self, days: i32) -> Result<Vec<VendorReadModel>, RepositoryError>;
    async fn find_high_risk_vendors(&self, organization_id: &str) -> Result<Vec<VendorReadModel>, RepositoryError>;
    async fn get_vendor_summary_stats(&self, organization_id: &str) -> Result<VendorStatsDto, RepositoryError>;
}
```

## Event-Driven Updates

### Domain Events

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorCreatedEvent {
    pub vendor_id: String,
    pub organization_id: String,
    pub name: String,
    pub code: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorActivatedEvent {
    pub vendor_id: String,
    pub occurred_at: DateTime<Utc>,
}

// Event handlers update read models
pub struct VendorCreatedEventHandler {
    vendor_read_repository: Arc<dyn VendorReadRepository>,
}

impl VendorCreatedEventHandler {
    pub async fn handle(&self, event: VendorCreatedEvent) -> Result<(), EventError> {
        // Update read model
        self.vendor_read_repository
            .create(VendorReadModel {
                id: event.vendor_id,
                organization_id: event.organization_id,
                name: event.name,
                code: event.code,
                status: "PENDING".to_string(),
                active_contract_count: 0,
                total_amount_paid: rust_decimal::Decimal::ZERO,
                last_activity: event.occurred_at,
                compliance_status: ComplianceStatus::Compliant,
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}

pub struct VendorActivatedEventHandler {
    vendor_read_repository: Arc<dyn VendorReadRepository>,
    notification_service: Arc<dyn NotificationService>,
}

impl VendorActivatedEventHandler {
    pub async fn handle(&self, event: VendorActivatedEvent) -> Result<(), EventError> {
        // Update read model
        self.vendor_read_repository
            .update_status(&event.vendor_id, "ACTIVE")
            .await?;

        // Side effects
        self.notification_service
            .notify_vendor_activated(&event.vendor_id)
            .await?;

        Ok(())
    }
}
```

## Complex Query Examples

### Lease Operating Statement Queries

```typescript
// Query for LOS dashboard
export class GetLosDashboardQuery {
  constructor(
    public readonly organizationId: string,
    public readonly year: number,
  ) {}
}

@QueryHandler(GetLosDashboardQuery)
export class GetLosDashboardHandler implements IQueryHandler<GetLosDashboardQuery> {
  constructor(private readonly losReadRepository: ILosReadRepository) {}

  async execute(query: GetLosDashboardQuery): Promise<LosDashboardDto> {
    // Complex aggregated query optimized for dashboard
    const [monthlyTotals, statusCounts, topExpenseCategories, trendData] = await Promise.all([
      this.losReadRepository.getMonthlyTotals(query.organizationId, query.year),
      this.losReadRepository.getStatusCounts(query.organizationId, query.year),
      this.losReadRepository.getTopExpenseCategories(query.organizationId, query.year),
      this.losReadRepository.getTrendData(query.organizationId, 12),
    ]);

    return {
      year: query.year,
      monthlyTotals,
      statusCounts,
      topExpenseCategories,
      trendData,
    };
  }
}

// Specialized read repository
pub struct LosReadRepository {
    db_pool: sqlx::PgPool,
}

impl LosReadRepository {
    pub async fn get_monthly_totals(
        &self,
        organization_id: &str,
        year: i32,
    ) -> Result<Vec<MonthlyTotal>, sqlx::Error> {
        // Optimized query with precomputed aggregations
        sqlx::query_as!(
            MonthlyTotal,
            r#"
            SELECT
                EXTRACT(MONTH FROM statement_month) as "month!",
                SUM(operating_expenses) as "operating_expenses!",
                SUM(capital_expenses) as "capital_expenses!",
                SUM(total_expenses) as "total_expenses!"
            FROM lease_operating_statements
            WHERE organization_id = $1
              AND EXTRACT(YEAR FROM statement_month) = $2
            GROUP BY EXTRACT(MONTH FROM statement_month)
            ORDER BY EXTRACT(MONTH FROM statement_month)
            "#,
            organization_id,
            year
        )
        .fetch_all(&self.db_pool)
        .await
    }
}
```

## CQRS Module Setup

### Axum Router Configuration

```rust
use axum::{routing::{get, post, put}, Router};
use std::sync::Arc;

pub fn vendor_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/vendors", post(create_vendor))
        .route("/vendors/:id", get(get_vendor))
        .route("/vendors", get(get_vendors))
        .route("/vendors/:id", put(update_vendor))
        .with_state(state)
}

// Application state with handlers
pub struct AppState {
    // Command Handlers
    create_vendor_handler: Arc<CreateVendorHandler>,
    update_vendor_handler: Arc<UpdateVendorHandler>,
    finalize_los_handler: Arc<FinalizeLosHandler>,

    // Query Handlers
    get_vendor_by_id_handler: Arc<GetVendorByIdHandler>,
    get_vendors_by_org_handler: Arc<GetVendorsByOrganizationHandler>,
    get_los_by_id_handler: Arc<GetLosByIdHandler>,

    // Event Handlers
    vendor_created_event_handler: Arc<VendorCreatedEventHandler>,
    vendor_activated_event_handler: Arc<VendorActivatedEventHandler>,

    // Repositories
    vendor_repository: Arc<dyn VendorRepository>,
    vendor_read_repository: Arc<dyn VendorReadRepository>,
    los_repository: Arc<dyn LosRepository>,
    los_read_repository: Arc<dyn LosReadRepository>,
}

impl AppState {
    pub async fn new(db_pool: sqlx::PgPool) -> Arc<Self> {
        // Initialize repositories
        let vendor_repository = Arc::new(VendorRepositoryImpl::new(db_pool.clone()));
        let vendor_read_repository = Arc::new(VendorReadRepositoryImpl::new(db_pool.clone()));

        // Initialize handlers
        let create_vendor_handler = Arc::new(CreateVendorHandler::new(
            vendor_repository.clone(),
            Arc::new(EventBus::new()),
        ));

        Arc::new(Self {
            create_vendor_handler,
            // ... initialize other handlers
            vendor_repository,
            vendor_read_repository,
            // ... other repositories
        })
    }
}
```

## Testing CQRS

### Command Handler Testing

```typescript
describe('CreateVendorHandler', () => {
  let handler: CreateVendorHandler;
  let mockRepository: jest.Mocked<IVendorRepository>;
  let mockEventBus: jest.Mocked<EventBus>;

  beforeEach(() => {
    mockRepository = createMockRepository();
    mockEventBus = createMockEventBus();
    handler = new CreateVendorHandler(mockRepository, mockEventBus);
  });

  it('should create vendor and publish events', async () => {
    // Given
    const command = new CreateVendorCommand(
      'org-123',
      'Test Vendor',
      'TEST-01',
      validContactInfo,
      validInsurance,
    );

    // When
    const vendorId = await handler.execute(command);

    // Then
    expect(mockRepository.save).toHaveBeenCalledWith(
      expect.objectContaining({
        name: expect.objectContaining({ value: 'Test Vendor' }),
      }),
    );
    expect(mockEventBus.publish).toHaveBeenCalledWith(expect.any(VendorCreatedEvent));
    expect(vendorId).toBeDefined();
  });
});
```

### Query Handler Testing

```typescript
describe('GetVendorByIdHandler', () => {
  let handler: GetVendorByIdHandler;
  let mockReadRepository: jest.Mocked<IVendorReadRepository>;

  beforeEach(() => {
    mockReadRepository = createMockReadRepository();
    handler = new GetVendorByIdHandler(mockReadRepository);
  });

  it('should return vendor details', async () => {
    // Given
    const vendorId = 'vendor-123';
    const mockVendorData = createMockVendorReadModel();
    mockReadRepository.findByIdWithDetails.mockResolvedValue(mockVendorData);

    // When
    const result = await handler.execute(new GetVendorByIdQuery(vendorId));

    // Then
    expect(result).toEqual({
      id: mockVendorData.id,
      name: mockVendorData.name,
      // ... other fields
    });
    expect(mockReadRepository.findByIdWithDetails).toHaveBeenCalledWith(vendorId);
  });
});
```

## Performance Considerations

### Read Model Optimization

- **Denormalization**: Store computed values to avoid complex joins
- **Indexing**: Create indexes optimized for query patterns
- **Caching**: Cache frequently accessed read models
- **Materialized Views**: Use database views for complex aggregations

### Command Processing

- **Async Processing**: Use message queues for time-consuming operations
- **Event Sourcing**: Consider event sourcing for audit trails
- **Optimistic Concurrency**: Handle concurrent updates appropriately

## Command vs Query Return Values

### Commands Return Minimal Data

Commands should return only what's necessary for the client to take the next action, typically just an ID:

```typescript
// ✅ Good - Returns minimal data
async execute(command: StartTimerCommand): Promise<{ id: string }> {
  // ... business logic
  const saved = await this.timeEntryRepository.save(timeEntry);
  return { id: saved.id };
}

// ❌ Bad - Returns full entity
async execute(command: StartTimerCommand): Promise<TimeEntry> {
  const saved = await this.timeEntryRepository.save(timeEntry);
  return saved; // Don't return full entity from command
}
```

**Why?** Commands are about _doing something_, not _retrieving data_. If the client needs full data after a command, they should issue a separate query with the returned ID.

### Queries Return Full Data or DTOs

Queries should return complete data structures optimized for the client's needs:

```typescript
// ✅ Good - Returns full entity or DTO
async execute(query: GetRunningTimerQuery): Promise<TimeEntry | null> {
  return await this.timeEntryRepository.findRunningTimerByUser(query.userId);
}

// ✅ Also good - Returns paginated result
async execute(query: GetTimeEntriesQuery): Promise<PaginatedResult<TimeEntry>> {
  return await this.timeEntryRepository.findAll(filters, pagination);
}
```

### Exception Handling Differences

**Commands** should throw exceptions when business rules are violated:

```typescript
// Commands throw exceptions
if (existingTimer) {
  throw new ConflictException('A timer is already running');
}

if (!project) {
  throw new NotFoundException('Project not found');
}
```

**Queries** should return null or empty results instead of throwing NotFoundException:

```typescript
// ✅ Queries return null for missing data
async execute(query: GetRunningTimerQuery): Promise<TimeEntry | null> {
  const timeEntry = await this.repository.findRunningTimerByUser(query.userId);
  return timeEntry; // Returns null if not found
}

// ❌ Don't throw NotFoundException in queries
async execute(query: GetRunningTimerQuery): Promise<TimeEntry> {
  const timeEntry = await this.repository.findRunningTimerByUser(query.userId);
  if (!timeEntry) {
    throw new NotFoundException(); // Don't do this in queries
  }
  return timeEntry;
}
```

**Why?** Queries are about reading - missing data is a valid query result. Commands are about state changes - missing data is an error condition that prevents the command from executing.

## When to Use CQRS

### Good Fit

- **Complex Business Logic**: Commands with rich business rules
- **Read-Heavy Applications**: Many more reads than writes
- **Different Scaling Requirements**: Reads and writes need different scaling
- **Complex Reporting**: Advanced analytics and reporting requirements
- **Event-Driven Architecture**: When using domain events extensively

### Not Recommended

- **Simple CRUD Applications**: Basic create, read, update, delete operations
- **Small Applications**: Overhead not justified for simple use cases
- **Tight Consistency Requirements**: When immediate consistency is critical
- **Limited Team Experience**: Requires understanding of event-driven patterns

CQRS in our PSA system helps us handle complex business operations (like time entry approval workflows) separately from optimized queries (like timesheet views and project analytics), leading to better performance and maintainability.
