# Observer Pattern

## Overview

The Observer pattern defines a one-to-many dependency between objects so that
when one object changes state, all its dependents are notified and updated
automatically. It promotes loose coupling between the subject (observable) and
its observers, allowing for flexible event-driven architectures.

## Core Concepts

### Subject (Observable)

The object being watched that maintains a list of observers and notifies them of
state changes.

### Observer

The interface that defines how objects should be notified of changes in the
subject.

### Concrete Observer

Specific implementations that react to notifications from the subject.

### Event-Driven Communication

Decoupled communication where observers respond to events without tight coupling
to the subject.

## Benefits

- **Loose Coupling**: Subjects and observers are loosely coupled
- **Dynamic Relationships**: Observers can be added/removed at runtime
- **Event-Driven Architecture**: Supports reactive programming patterns
- **Single Responsibility**: Each observer handles specific concerns
- **Open/Closed Principle**: New observers can be added without modifying
  existing code
- **Broadcast Communication**: One subject can notify many observers

## Implementation in Our Project

### Before: Tightly Coupled Notifications

```typescript
@Injectable()
export class VendorService {
  constructor(
    private readonly vendorRepository: VendorRepository,
    private readonly notificationService: NotificationService,
    private readonly auditService: AuditService,
    private readonly emailService: EmailService,
    private readonly integrationService: IntegrationService,
  ) {}

  async activateVendor(vendorId: string, userId: string): Promise<void> {
    const vendor = await this.vendorRepository.findById(vendorId);

    if (!vendor) {
      throw new VendorNotFoundError(vendorId);
    }

    vendor.activate();
    await this.vendorRepository.save(vendor);

    // Tightly coupled side effects - all handled directly in service
    try {
      // Send notification
      await this.notificationService.notify('VENDOR_ACTIVATED', {
        vendorId,
        vendorName: vendor.getName(),
      });
    } catch (error) {
      // Log error but don't fail the main operation
      console.error('Failed to send notification', error);
    }

    try {
      // Create audit entry
      await this.auditService.createAuditEntry({
        action: 'VENDOR_ACTIVATED',
        entityId: vendorId,
        entityType: 'Vendor',
        userId,
        timestamp: new Date(),
        details: { vendorName: vendor.getName() },
      });
    } catch (error) {
      console.error('Failed to create audit entry', error);
    }

    try {
      // Send email to vendor
      await this.emailService.sendEmail({
        to: vendor.getContactEmail(),
        subject: 'Your vendor account has been activated',
        template: 'vendor-activated',
        data: { vendorName: vendor.getName() },
      });
    } catch (error) {
      console.error('Failed to send email', error);
    }

    try {
      // Update external systems
      await this.integrationService.updateVendorStatus(vendorId, 'ACTIVE');
    } catch (error) {
      console.error('Failed to update external systems', error);
    }

    // More side effects...
  }

  async updateVendorInsurance(
    vendorId: string,
    insurance: InsuranceData,
    userId: string,
  ): Promise<void> {
    const vendor = await this.vendorRepository.findById(vendorId);

    if (!vendor) {
      throw new VendorNotFoundError(vendorId);
    }

    const oldInsurance = vendor.getInsurance();
    vendor.updateInsurance(new Insurance(insurance));
    await this.vendorRepository.save(vendor);

    // Duplicate notification logic
    try {
      await this.notificationService.notify('VENDOR_INSURANCE_UPDATED', {
        vendorId,
        vendorName: vendor.getName(),
      });
    } catch (error) {
      console.error('Failed to send notification', error);
    }

    try {
      await this.auditService.createAuditEntry({
        action: 'VENDOR_INSURANCE_UPDATED',
        entityId: vendorId,
        entityType: 'Vendor',
        userId,
        timestamp: new Date(),
        details: {
          vendorName: vendor.getName(),
          oldInsurance: oldInsurance?.toPlain(),
          newInsurance: insurance,
        },
      });
    } catch (error) {
      console.error('Failed to create audit entry', error);
    }

    // Insurance-specific notifications
    try {
      if (this.isInsuranceExpiringSoon(vendor.getInsurance())) {
        await this.notificationService.notify('INSURANCE_EXPIRING_SOON', {
          vendorId,
          expiryDate: vendor.getInsurance().getExpiryDate(),
        });
      }
    } catch (error) {
      console.error('Failed to check insurance expiry', error);
    }
  }
}
```

### After: Observer Pattern Implementation

```rust
use async_trait::async_trait;
use std::sync::Arc;

// Observer trait
#[async_trait]
pub trait VendorObserver: Send + Sync {
    async fn on_vendor_activated(&self, event: &VendorActivatedEvent) -> Result<(), ObserverError> {
        // Default implementation - override if needed
        Ok(())
    }

    async fn on_vendor_deactivated(&self, event: &VendorDeactivatedEvent) -> Result<(), ObserverError> {
        // Default implementation - override if needed
        Ok(())
    }

    async fn on_vendor_insurance_updated(
        &self,
        event: &VendorInsuranceUpdatedEvent,
    ) -> Result<(), ObserverError> {
        // Default implementation - override if needed
        Ok(())
    }

    async fn on_vendor_created(&self, event: &VendorCreatedEvent) -> Result<(), ObserverError> {
        // Default implementation - override if needed
        Ok(())
    }
}

// Concrete observers
pub struct VendorNotificationObserver {
    notification_service: Arc<dyn NotificationService>,
}

#[async_trait]
impl VendorObserver for VendorNotificationObserver {
    async fn on_vendor_activated(&self, event: &VendorActivatedEvent) -> Result<(), ObserverError> {
        self.notification_service
            .notify(
                "VENDOR_ACTIVATED",
                serde_json::json!({
                    "vendor_id": event.vendor_id,
                    "vendor_name": event.vendor_name,
                    "organization_id": event.organization_id,
                }),
            )
            .await
            .map_err(|e| ObserverError::NotificationFailed(e))?;

        Ok(())
    }

    async fn on_vendor_insurance_updated(
        &self,
        event: &VendorInsuranceUpdatedEvent,
    ) -> Result<(), ObserverError> {
        self.notification_service
            .notify(
                "VENDOR_INSURANCE_UPDATED",
                serde_json::json!({
                    "vendor_id": event.vendor_id,
                    "vendor_name": event.vendor_name,
                    "expiry_date": event.new_insurance.expiry_date,
                }),
            )
            .await
            .map_err(|e| ObserverError::NotificationFailed(e))?;

        // Check for expiring insurance
        if Self::is_insurance_expiring_soon(&event.new_insurance) {
            self.notification_service
                .notify(
                    "INSURANCE_EXPIRING_SOON",
                    serde_json::json!({
                        "vendor_id": event.vendor_id,
                        "expiry_date": event.new_insurance.expiry_date,
                    }),
                )
                .await
                .map_err(|e| ObserverError::NotificationFailed(e))?;
        }

        Ok(())
    }
}

impl VendorNotificationObserver {
    fn is_insurance_expiring_soon(insurance: &InsuranceData) -> bool {
        use chrono::Duration;
        let thirty_days_from_now = Utc::now() + Duration::days(30);
        insurance.expiry_date <= thirty_days_from_now
    }
}

pub struct VendorAuditObserver {
    audit_service: Arc<dyn AuditService>,
}

#[async_trait]
impl VendorObserver for VendorAuditObserver {
    async fn on_vendor_activated(&self, event: &VendorActivatedEvent) -> Result<(), ObserverError> {
        self.audit_service
            .create_audit_entry(AuditEntry {
                action: "VENDOR_ACTIVATED".to_string(),
                entity_id: event.vendor_id.clone(),
                entity_type: "Vendor".to_string(),
                user_id: event.user_id.clone(),
                timestamp: event.occurred_at,
                details: serde_json::json!({
                    "vendor_name": event.vendor_name,
                    "organization_id": event.organization_id,
                }),
            })
            .await
            .map_err(|e| ObserverError::AuditFailed(e))?;

        Ok(())
    }

    async fn on_vendor_created(&self, event: &VendorCreatedEvent) -> Result<(), ObserverError> {
        self.audit_service
            .create_audit_entry(AuditEntry {
                action: "VENDOR_CREATED".to_string(),
                entity_id: event.vendor_id.clone(),
                entity_type: "Vendor".to_string(),
                user_id: event.user_id.clone(),
                timestamp: event.occurred_at,
                details: serde_json::json!({
                    "vendor_name": event.vendor_name,
                    "vendor_code": event.vendor_code,
                    "organization_id": event.organization_id,
                }),
            })
            .await
            .map_err(|e| ObserverError::AuditFailed(e))?;

        Ok(())
    }

    async fn on_vendor_insurance_updated(
        &self,
        event: &VendorInsuranceUpdatedEvent,
    ) -> Result<(), ObserverError> {
        self.audit_service
            .create_audit_entry(AuditEntry {
                action: "VENDOR_INSURANCE_UPDATED".to_string(),
                entity_id: event.vendor_id.clone(),
                entity_type: "Vendor".to_string(),
                user_id: event.user_id.clone(),
                timestamp: event.occurred_at,
                details: serde_json::json!({
                    "vendor_name": event.vendor_name,
                    "old_insurance": event.old_insurance,
                    "new_insurance": event.new_insurance,
                }),
            })
            .await
            .map_err(|e| ObserverError::AuditFailed(e))?;

        Ok(())
    }
}

pub struct VendorEmailObserver {
    email_service: Arc<dyn EmailService>,
}

#[async_trait]
impl VendorObserver for VendorEmailObserver {
    async fn on_vendor_activated(&self, event: &VendorActivatedEvent) -> Result<(), ObserverError> {
        if let Some(ref contact_email) = event.contact_email {
            self.email_service
                .send_email(EmailMessage {
                    to: contact_email.clone(),
                    subject: "Your vendor account has been activated".to_string(),
                    template: "vendor-activated".to_string(),
                    data: serde_json::json!({
                        "vendor_name": event.vendor_name,
                        "activation_date": event.occurred_at,
                    }),
                })
                .await
                .map_err(|e| ObserverError::EmailFailed(e))?;
        }

        Ok(())
    }

    async fn on_vendor_created(&self, event: &VendorCreatedEvent) -> Result<(), ObserverError> {
        if let Some(ref contact_email) = event.contact_email {
            self.email_service
                .send_email(EmailMessage {
                    to: contact_email.clone(),
                    subject: "Welcome to our vendor network".to_string(),
                    template: "vendor-welcome".to_string(),
                    data: serde_json::json!({
                        "vendor_name": event.vendor_name,
                        "vendor_code": event.vendor_code,
                    }),
                })
                .await
                .map_err(|e| ObserverError::EmailFailed(e))?;
        }

        Ok(())
    }
}

pub struct VendorIntegrationObserver {
    integration_service: Arc<dyn IntegrationService>,
}

#[async_trait]
impl VendorObserver for VendorIntegrationObserver {
    async fn on_vendor_activated(&self, event: &VendorActivatedEvent) -> Result<(), ObserverError> {
        self.integration_service
            .update_vendor_status(&event.vendor_id, "ACTIVE")
            .await
            .map_err(|e| ObserverError::IntegrationFailed(e))?;

        Ok(())
    }

    async fn on_vendor_deactivated(&self, event: &VendorDeactivatedEvent) -> Result<(), ObserverError> {
        self.integration_service
            .update_vendor_status(&event.vendor_id, "INACTIVE")
            .await
            .map_err(|e| ObserverError::IntegrationFailed(e))?;

        Ok(())
    }

    async fn on_vendor_created(&self, event: &VendorCreatedEvent) -> Result<(), ObserverError> {
        self.integration_service
            .sync_new_vendor(NewVendorSync {
                vendor_id: event.vendor_id.clone(),
                vendor_name: event.vendor_name.clone(),
                vendor_code: event.vendor_code.clone(),
                organization_id: event.organization_id.clone(),
            })
            .await
            .map_err(|e| ObserverError::IntegrationFailed(e))?;

        Ok(())
    }
}

// Subject (Observable) - Event Publisher
use tokio::task::JoinSet;

pub struct VendorEventPublisher {
    observers: Vec<Arc<dyn VendorObserver>>,
}

impl VendorEventPublisher {
    pub fn new() -> Self {
        Self {
            observers: Vec::new(),
        }
    }

    pub fn add_observer(&mut self, observer: Arc<dyn VendorObserver>) {
        self.observers.push(observer);
    }

    pub fn remove_observer(&mut self, observer_id: &str) {
        // Implementation for removing observer by ID
        // Would need to add ID tracking to observers
    }

    pub async fn publish_vendor_activated(&self, event: VendorActivatedEvent) {
        self.notify_observers(|observer| async move {
            observer.on_vendor_activated(&event).await
        })
        .await;
    }

    pub async fn publish_vendor_created(&self, event: VendorCreatedEvent) {
        self.notify_observers(|observer| async move {
            observer.on_vendor_created(&event).await
        })
        .await;
    }

    pub async fn publish_vendor_insurance_updated(&self, event: VendorInsuranceUpdatedEvent) {
        self.notify_observers(|observer| async move {
            observer.on_vendor_insurance_updated(&event).await
        })
        .await;
    }

    async fn notify_observers<F, Fut>(&self, notification: F)
    where
        F: Fn(Arc<dyn VendorObserver>) -> Fut,
        Fut: std::future::Future<Output = Result<(), ObserverError>> + Send + 'static,
    {
        // Notify observers in parallel but handle errors gracefully
        let mut tasks = JoinSet::new();

        for observer in &self.observers {
            let observer_clone = observer.clone();
            let future = notification(observer_clone);

            tasks.spawn(async move {
                if let Err(error) = future.await {
                    tracing::error!(
                        "Observer notification failed: {:?}",
                        error
                    );
                    // Don't let observer failures affect the main flow
                }
            });
        }

        // Wait for all observers to complete
        while let Some(_) = tasks.join_next().await {}
    }
}

// Clean service using Observer pattern
pub struct VendorService {
    vendor_repository: Arc<dyn VendorRepository>,
    event_publisher: Arc<VendorEventPublisher>,
}

impl VendorService {
    pub async fn activate_vendor(&self, vendor_id: &str, user_id: &str) -> Result<(), VendorError> {
        let vendor = self
            .vendor_repository
            .find_by_id(vendor_id)
            .await?
            .ok_or_else(|| VendorError::NotFound(vendor_id.to_string()))?;

        vendor.activate();
        self.vendor_repository.save(&vendor).await?;

        // Single responsibility: just publish the event
        let event = VendorActivatedEvent {
            vendor_id: vendor_id.to_string(),
            vendor_name: vendor.get_name().to_string(),
            organization_id: vendor.get_organization_id().to_string(),
            contact_email: vendor.get_contact_info().get_email().map(|e| e.to_string()),
            user_id: user_id.to_string(),
            occurred_at: Utc::now(),
        };

        self.event_publisher.publish_vendor_activated(event).await;

        Ok(())
    }

    pub async fn update_vendor_insurance(
        &self,
        vendor_id: &str,
        insurance: InsuranceData,
        user_id: &str,
    ) -> Result<(), VendorError> {
        let vendor = self
            .vendor_repository
            .find_by_id(vendor_id)
            .await?
            .ok_or_else(|| VendorError::NotFound(vendor_id.to_string()))?;

        let old_insurance = vendor.get_insurance().map(|i| i.to_plain());
        vendor.update_insurance(Insurance::new(insurance.clone()));
        self.vendor_repository.save(&vendor).await?;

        let event = VendorInsuranceUpdatedEvent {
            vendor_id: vendor_id.to_string(),
            vendor_name: vendor.get_name().to_string(),
            organization_id: vendor.get_organization_id().to_string(),
            old_insurance,
            new_insurance: insurance,
            user_id: user_id.to_string(),
            occurred_at: Utc::now(),
        };

        self.event_publisher
            .publish_vendor_insurance_updated(event)
            .await;

        Ok(())
    }
}

// Event structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorActivatedEvent {
    pub vendor_id: String,
    pub vendor_name: String,
    pub organization_id: String,
    pub contact_email: Option<String>,
    pub user_id: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorCreatedEvent {
    pub vendor_id: String,
    pub vendor_name: String,
    pub vendor_code: String,
    pub organization_id: String,
    pub contact_email: Option<String>,
    pub user_id: String,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorInsuranceUpdatedEvent {
    pub vendor_id: String,
    pub vendor_name: String,
    pub organization_id: String,
    pub old_insurance: Option<InsuranceData>,
    pub new_insurance: InsuranceData,
    pub user_id: String,
    pub occurred_at: DateTime<Utc>,
}
```

## Advanced Observer Patterns

### Event-Driven Domain Events

```rust
// Domain Event Observer using Tokio channels
pub struct LosEventObserver {
    notification_service: Arc<dyn NotificationService>,
    report_service: Arc<dyn ReportService>,
}

#[async_trait]
impl EventObserver for LosEventObserver {
    async fn handle_event(&self, event: DomainEvent) -> Result<(), ObserverError> {
        match event {
            DomainEvent::LosFinalized(e) => self.handle_los_finalized(e).await,
            DomainEvent::LosExpenseAdded(e) => self.handle_expense_added(e).await,
            DomainEvent::LosDistributed(e) => self.handle_los_distributed(e).await,
            _ => Ok(()),
        }
    }
}

impl LosEventObserver {
    async fn handle_los_finalized(&self, event: LosFinalizedEvent) -> Result<(), ObserverError> {
        // Generate finalization report
        self.report_service
            .generate_finalization_report(&event.los_id)
            .await
            .map_err(|e| ObserverError::ReportFailed(e))?;

        // Notify stakeholders
        self.notification_service
            .notify_los_finalized(LosFinalizationNotification {
                los_id: event.los_id.clone(),
                lease_id: event.lease_id.clone(),
                total_expenses: event.total_expenses,
                finalized_by: event.user_id.clone(),
            })
            .await
            .map_err(|e| ObserverError::NotificationFailed(e))?;

        Ok(())
    }

    async fn handle_expense_added(&self, event: LosExpenseAddedEvent) -> Result<(), ObserverError> {
        // Check if expense needs approval
        if event.amount > 10000.0 {
            self.notification_service
                .notify_expense_requires_approval(ExpenseApprovalNotification {
                    los_id: event.los_id.clone(),
                    expense_id: event.expense_id.clone(),
                    amount: event.amount,
                    description: event.description.clone(),
                })
                .await
                .map_err(|e| ObserverError::NotificationFailed(e))?;
        }

        Ok(())
    }

    async fn handle_los_distributed(&self, event: LosDistributedEvent) -> Result<(), ObserverError> {
        // Send distribution notifications to partners
        for partner in &event.partners {
            self.notification_service
                .notify_partner_los_distribution(PartnerDistributionNotification {
                    partner_id: partner.id.clone(),
                    los_id: event.los_id.clone(),
                    allocation: partner.allocation,
                    amount: partner.amount,
                })
                .await
                .map_err(|e| ObserverError::NotificationFailed(e))?;
        }

        Ok(())
    }
}

// Domain events in entities
export class LeaseOperatingStatement {
  // ... entity properties

  finalize(userId: string): void {
    if (this.status !== LosStatus.DRAFT) {
      throw new LosDomainError('Only draft LOS can be finalized');
    }

    this.status = LosStatus.FINALIZED;
    this.finalizedAt = new Date();
    this.finalizedBy = userId;

    // Emit domain event
    this.addDomainEvent(
      new LosFinalizedEvent(
        this.id.getValue(),
        this.leaseId,
        this.totalExpenses.getAmount(),
        userId,
        new Date(),
      ),
    );
  }

  addExpense(expense: ExpenseLineItem, userId: string): void {
    this.expenseLineItems.push(expense);
    this.recalculateTotals();

    // Emit domain event
    this.addDomainEvent(
      new LosExpenseAddedEvent(
        this.id.getValue(),
        expense.getId().getValue(),
        expense.getAmount().getAmount(),
        expense.getDescription(),
        userId,
        new Date(),
      ),
    );
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
```

### Observer with Filtering

```typescript
// Observer interface with filtering capability
export interface IFilterableVendorObserver extends IVendorObserver {
  shouldHandle(event: VendorEvent): boolean;
}

// Filtered observer
@Injectable()
export class HighValueVendorObserver implements IFilterableVendorObserver {
  constructor(private readonly specialHandlingService: ISpecialHandlingService) {}

  shouldHandle(event: VendorEvent): boolean {
    // Only handle events for high-value vendors
    return event.vendorValue > 100000;
  }

  async onVendorActivated(event: VendorActivatedEvent): Promise<void> {
    if (!this.shouldHandle(event)) return;

    await this.specialHandlingService.setupHighValueVendorMonitoring(event.vendorId);
  }

  async onVendorCreated(event: VendorCreatedEvent): Promise<void> {
    if (!this.shouldHandle(event)) return;

    await this.specialHandlingService.assignSpecialAccountManager(event.vendorId);
  }

  // ... other methods
}

// Enhanced event publisher with filtering
@Injectable()
export class FilteringVendorEventPublisher {
  private observers: IFilterableVendorObserver[] = [];

  addObserver(observer: IFilterableVendorObserver): void {
    this.observers.push(observer);
  }

  async publishVendorActivated(event: VendorActivatedEvent): Promise<void> {
    const applicableObservers = this.observers.filter((observer) => observer.shouldHandle(event));

    await this.notifyObservers(applicableObservers, (observer) =>
      observer.onVendorActivated(event),
    );
  }

  private async notifyObservers(
    observers: IFilterableVendorObserver[],
    notification: (observer: IFilterableVendorObserver) => Promise<void>,
  ): Promise<void> {
    const promises = observers.map(async (observer) => {
      try {
        await notification(observer);
      } catch (error) {
        console.error('Observer notification failed', {
          observer: observer.constructor.name,
          error: error.message,
        });
      }
    });

    await Promise.allSettled(promises);
  }
}
```

### Prioritized Observer Execution

```typescript
// Observer with priority
export interface IPriorityVendorObserver extends IVendorObserver {
  getPriority(): number; // Lower numbers = higher priority
}

@Injectable()
export class CriticalVendorAuditObserver implements IPriorityVendorObserver {
  constructor(private readonly auditService: IAuditService) {}

  getPriority(): number {
    return 1; // Highest priority - audit first
  }

  async onVendorActivated(event: VendorActivatedEvent): Promise<void> {
    await this.auditService.createCriticalAuditEntry({
      action: 'VENDOR_ACTIVATED',
      entityId: event.vendorId,
      priority: 'HIGH',
      timestamp: event.occurredAt,
    });
  }

  // ... other methods
}

@Injectable()
export class PriorityVendorEventPublisher {
  private observers: IPriorityVendorObserver[] = [];

  addObserver(observer: IPriorityVendorObserver): void {
    this.observers.push(observer);
    // Keep observers sorted by priority
    this.observers.sort((a, b) => a.getPriority() - b.getPriority());
  }

  async publishVendorActivated(event: VendorActivatedEvent): Promise<void> {
    // Execute observers in priority order
    for (const observer of this.observers) {
      try {
        await observer.onVendorActivated(event);
      } catch (error) {
        console.error('Priority observer failed', {
          observer: observer.constructor.name,
          priority: observer.getPriority(),
          error: error.message,
        });

        // For high-priority observers, we might want to fail fast
        if (observer.getPriority() === 1) {
          throw error;
        }
      }
    }
  }
}
```

## Observer Configuration and Registration

### Module Configuration

```rust
// Dependency injection setup using constructor pattern
pub fn create_vendor_module(
    vendor_repository: Arc<dyn VendorRepository>,
    notification_service: Arc<dyn NotificationService>,
    audit_service: Arc<dyn AuditService>,
    email_service: Arc<dyn EmailService>,
    integration_service: Arc<dyn IntegrationService>,
) -> VendorModule {
    // Create observers
    let notification_observer = Arc::new(VendorNotificationObserver {
        notification_service: notification_service.clone(),
    });

    let audit_observer = Arc::new(VendorAuditObserver {
        audit_service: audit_service.clone(),
    });

    let email_observer = Arc::new(VendorEmailObserver {
        email_service: email_service.clone(),
    });

    let integration_observer = Arc::new(VendorIntegrationObserver {
        integration_service: integration_service.clone(),
    });

    // Create event publisher and register observers
    let mut event_publisher = VendorEventPublisher::new();
    event_publisher.add_observer(notification_observer);
    event_publisher.add_observer(audit_observer);
    event_publisher.add_observer(email_observer);
    event_publisher.add_observer(integration_observer);

    let event_publisher = Arc::new(event_publisher);

    // Create vendor service
    let vendor_service = VendorService {
        vendor_repository,
        event_publisher,
    };

    VendorModule {
        vendor_service: Arc::new(vendor_service),
    }
}

pub struct VendorModule {
    pub vendor_service: Arc<VendorService>,
}
```

## Testing Observer Pattern

### Observer Testing

```typescript
describe('VendorObservers', () => {
  describe('VendorNotificationObserver', () => {
    let observer: VendorNotificationObserver;
    let mockNotificationService: jest.Mocked<INotificationService>;

    beforeEach(() => {
      mockNotificationService = {
        notify: jest.fn().mockResolvedValue(undefined),
      };

      observer = new VendorNotificationObserver(mockNotificationService);
    });

    describe('onVendorActivated', () => {
      it('should send activation notification', async () => {
        const event = new VendorActivatedEvent(
          'vendor-123',
          'Test Vendor',
          'org-456',
          'test@vendor.com',
          'user-789',
          new Date(),
        );

        await observer.onVendorActivated(event);

        expect(mockNotificationService.notify).toHaveBeenCalledWith('VENDOR_ACTIVATED', {
          vendorId: 'vendor-123',
          vendorName: 'Test Vendor',
          organizationId: 'org-456',
        });
      });
    });

    describe('onVendorInsuranceUpdated', () => {
      it('should send insurance update notification', async () => {
        const event = new VendorInsuranceUpdatedEvent(
          'vendor-123',
          'Test Vendor',
          'org-456',
          undefined,
          {
            provider: 'New Insurance',
            policyNumber: 'POL-456',
            expiryDate: '2025-12-31',
            coverageAmount: 1000000,
          },
          'user-789',
          new Date(),
        );

        await observer.onVendorInsuranceUpdated(event);

        expect(mockNotificationService.notify).toHaveBeenCalledWith(
          'VENDOR_INSURANCE_UPDATED',
          expect.objectContaining({
            vendorId: 'vendor-123',
            vendorName: 'Test Vendor',
          }),
        );
      });

      it('should send expiry warning for expiring insurance', async () => {
        const soonToExpire = new Date();
        soonToExpire.setDate(soonToExpire.getDate() + 15); // 15 days from now

        const event = new VendorInsuranceUpdatedEvent(
          'vendor-123',
          'Test Vendor',
          'org-456',
          undefined,
          {
            provider: 'New Insurance',
            policyNumber: 'POL-456',
            expiryDate: soonToExpire.toISOString(),
            coverageAmount: 1000000,
          },
          'user-789',
          new Date(),
        );

        await observer.onVendorInsuranceUpdated(event);

        expect(mockNotificationService.notify).toHaveBeenCalledWith(
          'INSURANCE_EXPIRING_SOON',
          expect.objectContaining({
            vendorId: 'vendor-123',
            expiryDate: soonToExpire.toISOString(),
          }),
        );
      });
    });
  });
});

describe('VendorEventPublisher', () => {
  let publisher: VendorEventPublisher;
  let mockObserver1: jest.Mocked<IVendorObserver>;
  let mockObserver2: jest.Mocked<IVendorObserver>;

  beforeEach(() => {
    publisher = new VendorEventPublisher();

    mockObserver1 = {
      onVendorActivated: jest.fn().mockResolvedValue(undefined),
      onVendorCreated: jest.fn().mockResolvedValue(undefined),
      onVendorDeactivated: jest.fn().mockResolvedValue(undefined),
      onVendorInsuranceUpdated: jest.fn().mockResolvedValue(undefined),
    };

    mockObserver2 = {
      onVendorActivated: jest.fn().mockResolvedValue(undefined),
      onVendorCreated: jest.fn().mockResolvedValue(undefined),
      onVendorDeactivated: jest.fn().mockResolvedValue(undefined),
      onVendorInsuranceUpdated: jest.fn().mockResolvedValue(undefined),
    };

    publisher.addObserver(mockObserver1);
    publisher.addObserver(mockObserver2);
  });

  describe('publishVendorActivated', () => {
    it('should notify all observers', async () => {
      const event = new VendorActivatedEvent(
        'vendor-123',
        'Test Vendor',
        'org-456',
        'test@vendor.com',
        'user-789',
        new Date(),
      );

      await publisher.publishVendorActivated(event);

      expect(mockObserver1.onVendorActivated).toHaveBeenCalledWith(event);
      expect(mockObserver2.onVendorActivated).toHaveBeenCalledWith(event);
    });

    it('should handle observer failures gracefully', async () => {
      const event = new VendorActivatedEvent(
        'vendor-123',
        'Test Vendor',
        'org-456',
        'test@vendor.com',
        'user-789',
        new Date(),
      );

      mockObserver1.onVendorActivated.mockRejectedValue(new Error('Observer 1 failed'));

      // Should not throw despite observer failure
      await expect(publisher.publishVendorActivated(event)).resolves.not.toThrow();

      // Other observers should still be called
      expect(mockObserver2.onVendorActivated).toHaveBeenCalledWith(event);
    });
  });

  describe('observer management', () => {
    it('should add observers', () => {
      const newObserver = mockObserver1;
      publisher.addObserver(newObserver);

      expect(publisher['observers']).toContain(newObserver);
    });

    it('should remove observers', () => {
      publisher.removeObserver(mockObserver1);

      expect(publisher['observers']).not.toContain(mockObserver1);
      expect(publisher['observers']).toContain(mockObserver2);
    });
  });
});
```

## Best Practices

### 1. Error Isolation

```typescript
// Good: Isolate observer failures
async function notifyObservers(observers: Observer[], event: Event): Promise<void> {
  const promises = observers.map(async (observer) => {
    try {
      await observer.handle(event);
    } catch (error) {
      console.error('Observer failed', {
        observer: observer.constructor.name,
        error,
      });
      // Don't let one observer failure affect others
    }
  });

  await Promise.allSettled(promises);
}

// Avoid: Letting observer failures bubble up
async function notifyObservers(observers: Observer[], event: Event): Promise<void> {
  for (const observer of observers) {
    await observer.handle(event); // If one fails, others won't be called
  }
}
```

### 2. Selective Observer Registration

```typescript
// Good: Observers can opt-in to specific events
export interface ISelectiveObserver {
  getInterestedEvents(): string[];
  handle(event: Event): Promise<void>;
}

class EventPublisher {
  async publish(event: Event): Promise<void> {
    const interestedObservers = this.observers.filter((observer) =>
      observer.getInterestedEvents().includes(event.type),
    );

    await this.notifyObservers(interestedObservers, event);
  }
}
```

### 3. Observer Performance Monitoring

```typescript
// Good: Monitor observer performance
class PerformanceMonitoringObserver implements IVendorObserver {
  constructor(
    private readonly wrappedObserver: IVendorObserver,
    private readonly metricsService: IMetricsService,
  ) {}

  async onVendorActivated(event: VendorActivatedEvent): Promise<void> {
    const start = Date.now();
    try {
      await this.wrappedObserver.onVendorActivated(event);
      this.recordSuccess(start, 'onVendorActivated');
    } catch (error) {
      this.recordError(start, 'onVendorActivated', error);
      throw error;
    }
  }

  private recordSuccess(start: number, method: string): void {
    const duration = Date.now() - start;
    this.metricsService.recordObserverExecutionTime(
      this.wrappedObserver.constructor.name,
      method,
      duration,
    );
  }
}
```

The Observer pattern in our oil & gas management system enables loose coupling
between business operations and their side effects, allowing for flexible
event-driven architectures that can easily accommodate new requirements without
modifying existing code.
