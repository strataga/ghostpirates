# PSA Patterns & Best Practices

**Version**: 1.0
**Last Updated**: October 2025
**Status**: ✅ Production Ready

## Overview

This document provides comprehensive patterns, best practices, and implementation strategies specifically for Professional Services Automation (PSA) platforms. Based on the WellOS implementation covering client management, invoicing, time tracking, project management, and resource allocation.

---

## Table of Contents

1. [PSA Domain Fundamentals](#psa-domain-fundamentals)
2. [Client Management Patterns](#client-management-patterns)
3. [Invoice Management Patterns](#invoice-management-patterns)
4. [Time Tracking Patterns](#time-tracking-patterns)
5. [Project & Resource Management](#project--resource-management)
6. [Multi-tenancy in PSA](#multi-tenancy-in-psa)
7. [Financial Operations](#financial-operations)
8. [Reporting & Analytics](#reporting--analytics)
9. [Integration Patterns](#integration-patterns)
10. [Performance Optimization](#performance-optimization)

---

## PSA Domain Fundamentals

### Core PSA Entities

```typescript
// Central entities in any PSA platform
interface PSACore {
  Organization; // Tenant boundary
  Client; // Service recipients
  Project; // Billable work containers
  TimeEntry; // Billable time tracking
  Invoice; // Revenue documents
  Resource; // Staff/consultants
  Assignment; // Resource → Project mapping
}
```

### PSA Entity Relationships

```
Organization (1:N)
├── Clients (1:N)
│   └── Invoices (1:N)
│       └── InvoiceLineItems (1:N)
├── Projects (1:N)
│   └── Assignments (1:N)
│       └── Resources (Users)
└── TimeEntries (1:N)
    ├── Project (N:1)
    ├── User (N:1)
    └── Client (N:1 optional)
```

### Domain-Driven Design for PSA

```typescript
// Domain layer with rich business logic
export class Invoice {
  // Status transitions enforce business rules
  send(userId: string): void {
    if (this.status !== InvoiceStatus.DRAFT) {
      throw new Error('Only draft invoices can be sent');
    }

    if (this.lineItems.length === 0) {
      throw new Error('Cannot send invoice without line items');
    }

    this.status = InvoiceStatus.SENT;
    this.sentDate = new Date();
    this.updatedBy = userId;

    this.addDomainEvent(new InvoiceSentEvent(this.id, this.clientId));
  }

  markPaid(paymentDate: Date, userId: string, paymentMethod?: string): Invoice {
    if (this.status === InvoiceStatus.DRAFT) {
      throw new Error('Cannot mark draft invoice as paid');
    }

    if (this.status === InvoiceStatus.PAID) {
      throw new Error('Invoice is already paid');
    }

    this.status = InvoiceStatus.PAID;
    this.paidDate = paymentDate;
    this.paymentMethod = paymentMethod;
    this.updatedBy = userId;

    this.addDomainEvent(new InvoicePaidEvent(this.id, this.clientId, this.total));
    return this;
  }

  // Business rule: overdue after 30 days
  markOverdue(userId: string): Invoice {
    if (this.status !== InvoiceStatus.SENT) {
      throw new Error('Only sent invoices can be marked overdue');
    }

    if (!this.dueDate) {
      throw new Error('Cannot mark invoice overdue without due date');
    }

    const now = new Date();
    if (now <= this.dueDate) {
      throw new Error('Invoice is not yet overdue');
    }

    this.status = InvoiceStatus.OVERDUE;
    this.updatedBy = userId;

    this.addDomainEvent(new InvoiceOverdueEvent(this.id, this.clientId));
    return this;
  }
}
```

**Key Principles:**

- Business logic lives in domain entities
- State transitions are explicit and validated
- Domain events capture business-significant occurrences
- Immutability for value objects (Email, Money, Duration)

---

## Client Management Patterns

### 1. Client Value Objects Pattern

```typescript
// Email value object with validation
export class Email {
  private readonly _value: string;

  private constructor(value: string) {
    this._value = value;
  }

  static create(email: string): Email {
    if (!email || typeof email !== 'string') {
      throw new Error('Email is required and must be a string');
    }

    const trimmed = email.trim().toLowerCase();

    if (trimmed.length === 0) {
      throw new Error('Email cannot be empty');
    }

    // Basic email format validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(trimmed)) {
      throw new Error('Invalid email format');
    }

    return new Email(trimmed);
  }

  get value(): string {
    return this._value;
  }
}

// BillingAddress value object
export class BillingAddress {
  private constructor(
    private readonly _street?: string,
    private readonly _city?: string,
    private readonly _state?: string,
    private readonly _postalCode?: string,
    private readonly _country?: string,
  ) {}

  static create(data: {
    street?: string;
    city?: string;
    state?: string;
    postalCode?: string;
    country?: string;
  }): BillingAddress {
    return new BillingAddress(data.street, data.city, data.state, data.postalCode, data.country);
  }

  static createEmpty(): BillingAddress {
    return new BillingAddress();
  }

  isEmpty(): boolean {
    return !this._street && !this._city && !this._state && !this._postalCode && !this._country;
  }

  // Getters for immutable access
  get street(): string | undefined {
    return this._street;
  }
  get city(): string | undefined {
    return this._city;
  }
  get state(): string | undefined {
    return this._state;
  }
  get postalCode(): string | undefined {
    return this._postalCode;
  }
  get country(): string | undefined {
    return this._country;
  }
}
```

**Benefits:**

- Encapsulation of validation logic
- Type safety with domain concepts
- Immutability prevents accidental mutations
- Reusable across application layers

### 2. Client Repository Pattern

```rust
// Repository trait definition
pub trait ClientRepository {
    async fn find_by_id(&self, pool: &sqlx::PgPool, id: &str) -> Result<Option<Client>, sqlx::Error>;
    async fn find_by_organization(
        &self,
        pool: &sqlx::PgPool,
        organization_id: &str,
        search: Option<&str>,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Client>, sqlx::Error>;
    async fn save(&self, pool: &sqlx::PgPool, client: &Client) -> Result<(), sqlx::Error>;
    async fn is_email_unique(
        &self,
        pool: &sqlx::PgPool,
        email: &str,
        organization_id: &str,
        exclude_id: Option<&str>,
    ) -> Result<bool, sqlx::Error>;
    async fn delete(&self, pool: &sqlx::PgPool, id: &str, deleted_by: &str) -> Result<(), sqlx::Error>;
}

// SQLx implementation with optimizations
pub struct ClientRepositoryImpl;

impl ClientRepository for ClientRepositoryImpl {
    async fn find_by_organization(
        &self,
        pool: &sqlx::PgPool,
        organization_id: &str,
        search: Option<&str>,
        pagination: Pagination,
    ) -> Result<PaginatedResult<Client>, sqlx::Error> {
        let offset = (pagination.page - 1) * pagination.limit;

        // Build base query
        let mut query_builder = sqlx::QueryBuilder::new(
            "SELECT * FROM clients WHERE organization_id = "
        );
        query_builder.push_bind(organization_id);
        query_builder.push(" AND deleted_at IS NULL");

        // Add search filter if provided
        if let Some(search_term) = search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            query_builder.push(" AND (");
            query_builder.push("LOWER(name) LIKE ");
            query_builder.push_bind(&search_pattern);
            query_builder.push(" OR LOWER(email) LIKE ");
            query_builder.push_bind(&search_pattern);
            query_builder.push(" OR LOWER(company_name) LIKE ");
            query_builder.push_bind(&search_pattern);
            query_builder.push(")");
        }

        // Execute query with pagination in parallel with count query
        let query_sql = query_builder.sql();

        let (clients, total) = tokio::try_join!(
            async {
                sqlx::query_as::<_, ClientRow>(
                    &format!("{} ORDER BY created_at DESC LIMIT $1 OFFSET $2", query_sql)
                )
                .bind(pagination.limit)
                .bind(offset)
                .fetch_all(pool)
                .await
            },
            async {
                sqlx::query_scalar::<_, i64>(&format!("SELECT COUNT(*) FROM ({})", query_sql))
                    .fetch_one(pool)
                    .await
            }
        )?;

        Ok(PaginatedResult {
            data: clients.into_iter().map(|row| self.to_domain(row)).collect(),
            total: total as usize,
            page: pagination.page,
            limit: pagination.limit,
            total_pages: ((total as usize + pagination.limit - 1) / pagination.limit),
        })
    }

    async fn is_email_unique(
        &self,
        pool: &sqlx::PgPool,
        email: &str,
        organization_id: &str,
        exclude_id: Option<&str>,
    ) -> Result<bool, sqlx::Error> {
        let email_lower = email.to_lowercase();

        let result = if let Some(id_to_exclude) = exclude_id {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM clients
                 WHERE LOWER(email) = $1
                 AND organization_id = $2
                 AND deleted_at IS NULL
                 AND id != $3",
                email_lower,
                organization_id,
                id_to_exclude
            )
            .fetch_one(pool)
            .await?
        } else {
            sqlx::query_scalar!(
                "SELECT COUNT(*) FROM clients
                 WHERE LOWER(email) = $1
                 AND organization_id = $2
                 AND deleted_at IS NULL",
                email_lower,
                organization_id
            )
            .fetch_one(pool)
            .await?
        };

        Ok(result.unwrap_or(0) == 0)
    }
}
```

**Best Practices:**

- Parallel queries for data + count (better performance)
- Indexed search columns for fast filtering
- Case-insensitive email storage and comparison
- Soft delete filtering in all queries
- Efficient pagination with offset/limit

### 3. Client CRUD with Audit Trail

```typescript
@CommandHandler(UpdateClientCommand)
export class UpdateClientHandler implements ICommandHandler<UpdateClientCommand> {
  async execute(command: UpdateClientCommand): Promise<Client> {
    // 1. Fetch existing client
    const client = await this.clientRepository.findById(command.id);
    if (!client) {
      throw new NotFoundException('Client not found');
    }

    // 2. Check email uniqueness if changed
    if (command.email && command.email !== client.email.value) {
      const emailUnique = await this.clientRepository.isEmailUnique(
        command.email,
        command.organizationId,
        command.id,
      );

      if (!emailUnique) {
        throw new ConflictException('A client with this email already exists in your organization');
      }
    }

    // 3. Update client using domain method
    const updatedClient = client.update({
      name: command.name,
      email: command.email ? Email.create(command.email) : undefined,
      companyName: command.companyName,
      billingAddress: command.billingAddress
        ? BillingAddress.create(command.billingAddress)
        : undefined,
      phone: command.phone,
      website: command.website,
      notes: command.notes,
      taxId: command.taxId,
      updatedBy: command.userId,
    });

    // 4. Persist changes
    await this.clientRepository.save(updatedClient);

    // 5. Publish domain events
    updatedClient.getDomainEvents().forEach((event) => {
      this.eventBus.publish(event);
    });
    updatedClient.clearDomainEvents();

    // 6. Log audit trail
    await this.auditLogService.log({
      entityType: 'Client',
      entityId: updatedClient.id,
      action: 'UPDATE',
      changes: this.calculateChanges(client, updatedClient),
      userId: command.userId,
      organizationId: command.organizationId,
    });

    return updatedClient;
  }
}
```

**Audit Trail Best Practices:**

- Track all mutations (create, update, delete)
- Store user ID and timestamp
- Capture before/after values for updates
- Separate audit log table for performance
- Include organization context for multi-tenancy

---

## Invoice Management Patterns

### 1. Invoice Generation from Time Entries

```typescript
@CommandHandler(GenerateInvoiceCommand)
export class GenerateInvoiceHandler implements ICommandHandler<GenerateInvoiceCommand> {
  async execute(command: GenerateInvoiceCommand): Promise<Invoice> {
    // 1. Fetch unbilled time entries
    const timeEntries = await this.timeEntryRepository.findForInvoicing({
      organizationId: command.organizationId,
      clientId: command.clientId,
      projectIds: command.projectIds,
      startDate: command.startDate,
      endDate: command.endDate,
      status: TimeEntryStatus.APPROVED, // Only approved entries
    });

    if (timeEntries.length === 0) {
      throw new Error('No billable time entries found for the specified criteria');
    }

    // 2. Group time entries by project
    const groupedByProject = this.groupByProject(timeEntries);

    // 3. Create invoice line items
    const lineItems: InvoiceLineItem[] = [];
    for (const [projectId, entries] of Object.entries(groupedByProject)) {
      const project = await this.projectRepository.findById(projectId);
      const totalHours = entries.reduce((sum, e) => sum + e.duration.hours, 0);
      const rate = project.billing.hourlyRate || command.defaultHourlyRate || 150;

      lineItems.push(
        InvoiceLineItem.create(
          this.generateLineItemId(),
          `${project.name} - Professional Services (${totalHours.toFixed(2)} hours)`,
          totalHours,
          rate,
        ),
      );
    }

    // 4. Calculate totals
    const subtotal = lineItems.reduce((sum, item) => sum + item.totalAmount, 0);
    const taxRate = command.taxRate || 0;
    const tax = subtotal * taxRate;
    const total = subtotal + tax;

    // 5. Create invoice
    const invoice = Invoice.create({
      organizationId: command.organizationId,
      clientId: command.clientId,
      lineItems,
      subtotal,
      tax,
      total,
      taxRate,
      issueDate: command.issueDate,
      dueDate: command.dueDate,
      notes: command.notes,
      paymentTerms: command.paymentTerms,
      createdBy: command.userId,
      updatedBy: command.userId,
    });

    // 6. Persist invoice
    await this.invoiceRepository.save(invoice);

    // 7. Mark time entries as billed
    await this.timeEntryRepository.markAsBilled(
      timeEntries.map((e) => e.id),
      invoice.id,
    );

    // 8. Publish event
    this.eventBus.publish(
      new InvoiceGeneratedEvent(invoice.id, command.clientId, timeEntries.length),
    );

    return invoice;
  }
}
```

**Key Considerations:**

- Only bill approved time entries
- Group by project for clear line items
- Mark time entries as billed to prevent double-billing
- Support configurable hourly rates per project
- Validate business rules (due date > issue date, etc.)

### 2. Invoice Lifecycle Management

```typescript
export enum InvoiceStatus {
  DRAFT = 'DRAFT', // Being prepared
  SENT = 'SENT', // Sent to client
  PAID = 'PAID', // Payment received
  OVERDUE = 'OVERDUE', // Past due date
  CANCELLED = 'CANCELLED', // Cancelled/voided
}

export class Invoice {
  // State machine for invoice lifecycle
  private static readonly ALLOWED_TRANSITIONS: Record<InvoiceStatus, InvoiceStatus[]> = {
    [InvoiceStatus.DRAFT]: [InvoiceStatus.SENT, InvoiceStatus.CANCELLED],
    [InvoiceStatus.SENT]: [InvoiceStatus.PAID, InvoiceStatus.OVERDUE, InvoiceStatus.CANCELLED],
    [InvoiceStatus.OVERDUE]: [InvoiceStatus.PAID, InvoiceStatus.CANCELLED],
    [InvoiceStatus.PAID]: [], // Terminal state
    [InvoiceStatus.CANCELLED]: [], // Terminal state
  };

  private validateTransition(newStatus: InvoiceStatus): void {
    const allowedStatuses = Invoice.ALLOWED_TRANSITIONS[this.status];
    if (!allowedStatuses.includes(newStatus)) {
      throw new Error(
        `Cannot transition from ${this.status} to ${newStatus}. ` +
          `Allowed transitions: ${allowedStatuses.join(', ')}`,
      );
    }
  }

  send(userId: string): void {
    this.validateTransition(InvoiceStatus.SENT);

    if (this.lineItems.length === 0) {
      throw new Error('Cannot send invoice without line items');
    }

    if (!this.dueDate) {
      throw new Error('Cannot send invoice without due date');
    }

    this.status = InvoiceStatus.SENT;
    this.sentDate = new Date();
    this.updatedBy = userId;

    this.addDomainEvent(new InvoiceSentEvent(this.id, this.clientId));
  }
}
```

**Lifecycle Best Practices:**

- Explicit state machine with allowed transitions
- Validation before state changes
- Immutable once paid (terminal state)
- Domain events for integration points
- Audit trail for all status changes

### 3. PDF Generation Pattern

```typescript
export class InvoicePdfService {
  async generateInvoicePdf(
    invoice: Invoice,
    client: Client,
    organization: Organization,
  ): Promise<Buffer> {
    return new Promise((resolve, reject) => {
      const doc = new PDFDocument({ margin: 50, size: 'A4' });
      const chunks: Buffer[] = [];

      // Collect PDF chunks
      doc.on('data', (chunk) => chunks.push(chunk));
      doc.on('end', () => resolve(Buffer.concat(chunks)));
      doc.on('error', reject);

      // 1. Header with organization branding
      doc
        .fontSize(20)
        .text(organization.name.getValue(), 50, 50)
        .fontSize(10)
        .text(organization.primaryDomain.getValue(), 50, 75);

      // 2. Invoice metadata
      doc
        .fontSize(16)
        .text('INVOICE', 400, 50)
        .fontSize(10)
        .text(`Invoice #: ${invoice.invoiceNumber}`, 400, 75)
        .text(`Date: ${this.formatDate(invoice.issueDate)}`, 400, 90)
        .text(`Due Date: ${this.formatDate(invoice.dueDate)}`, 400, 105);

      // 3. Client information (Bill To)
      doc
        .fontSize(12)
        .text('Bill To:', 50, 150)
        .fontSize(10)
        .text(client.name, 50, 170)
        .text(client.companyName || '', 50, 185);

      if (!client.billingAddress.isEmpty()) {
        const addr = client.billingAddress;
        doc
          .text(addr.street || '', 50, 200)
          .text(`${addr.city}, ${addr.state} ${addr.postalCode}`, 50, 215)
          .text(addr.country || '', 50, 230);
      }

      // 4. Line items table
      const tableTop = 300;
      this.generateTable(doc, invoice, tableTop);

      // 5. Totals
      const totalsY = tableTop + invoice.lineItems.length * 30 + 50;
      doc
        .fontSize(10)
        .text('Subtotal:', 400, totalsY)
        .text(this.formatCurrency(invoice.subtotal), 500, totalsY, { align: 'right' });

      if (invoice.tax > 0) {
        doc
          .text(`Tax (${(invoice.taxRate * 100).toFixed(1)}%):`, 400, totalsY + 15)
          .text(this.formatCurrency(invoice.tax), 500, totalsY + 15, { align: 'right' });
      }

      doc
        .fontSize(12)
        .font('Helvetica-Bold')
        .text('Total:', 400, totalsY + 35)
        .text(this.formatCurrency(invoice.total), 500, totalsY + 35, { align: 'right' });

      // 6. Payment status
      if (invoice.status === InvoiceStatus.PAID) {
        doc
          .fontSize(14)
          .fillColor('green')
          .text('PAID', 50, totalsY + 50)
          .fillColor('black')
          .fontSize(10)
          .text(`Payment Date: ${this.formatDate(invoice.paidDate)}`, 50, totalsY + 70);
      }

      // 7. Notes and payment terms
      if (invoice.notes || invoice.paymentTerms) {
        doc.fontSize(10).text('Notes:', 50, totalsY + 100);
        if (invoice.notes) {
          doc.text(invoice.notes, 50, totalsY + 115, { width: 500 });
        }
        if (invoice.paymentTerms) {
          doc.text(`Payment Terms: ${invoice.paymentTerms}`, 50, totalsY + 140);
        }
      }

      // 8. Footer
      doc.fontSize(8).text('Thank you for your business!', 50, doc.page.height - 50, {
        align: 'center',
        width: doc.page.width - 100,
      });

      doc.end();
    });
  }

  private generateTable(doc: PDFKit.PDFDocument, invoice: Invoice, startY: number): void {
    // Table headers
    doc
      .fontSize(10)
      .font('Helvetica-Bold')
      .text('Description', 50, startY)
      .text('Quantity', 300, startY)
      .text('Rate', 380, startY)
      .text('Amount', 480, startY, { align: 'right' });

    // Horizontal line
    doc
      .moveTo(50, startY + 15)
      .lineTo(550, startY + 15)
      .stroke();

    // Line items
    let y = startY + 25;
    doc.font('Helvetica');

    invoice.lineItems.forEach((item, index) => {
      doc
        .text(item.description, 50, y, { width: 240 })
        .text(item.quantity.toFixed(2), 300, y)
        .text(this.formatCurrency(item.unitPrice), 380, y)
        .text(this.formatCurrency(item.totalAmount), 480, y, { align: 'right' });

      y += 30;
    });

    // Bottom line
    doc.moveTo(50, y).lineTo(550, y).stroke();
  }
}
```

**PDF Generation Best Practices:**

- Stream PDF generation (don't load entire doc in memory)
- Professional formatting with proper spacing
- Include all legally required information
- Support multiple currency formats
- Brand with organization logo/colors (future enhancement)
- Generate on-demand (don't store PDFs)

### 4. Automated Overdue Detection

```typescript
@Injectable()
export class InvoiceOverdueCheckTask {
  private readonly logger = new Logger(InvoiceOverdueCheckTask.name);

  // Run daily at 2 AM
  @Cron('0 2 * * *')
  async checkOverdueInvoices(): Promise<void> {
    this.logger.log('Starting overdue invoice check...');

    try {
      const command = new MarkInvoicesOverdueCommand();
      const result = await this.commandBus.execute(command);

      this.logger.log(
        `Overdue invoice check completed. ` +
          `Marked ${result.markedOverdueCount} invoices as overdue.`,
      );
    } catch (error) {
      this.logger.error('Failed to check overdue invoices:', error);
    }
  }
}

@CommandHandler(MarkInvoicesOverdueCommand)
export class MarkInvoicesOverdueHandler {
  async execute(command: MarkInvoicesOverdueCommand): Promise<{ markedOverdueCount: number }> {
    // Find all sent invoices past their due date
    const overdueInvoices = await this.invoiceRepository.findOverdue();

    let markedCount = 0;

    for (const invoice of overdueInvoices) {
      try {
        // Use domain method for state transition
        invoice.markOverdue('system');
        await this.invoiceRepository.save(invoice);

        // Publish event for notifications
        this.eventBus.publish(new InvoiceOverdueEvent(invoice.id, invoice.clientId));

        markedCount++;
      } catch (error) {
        this.logger.error(`Failed to mark invoice ${invoice.id} as overdue:`, error);
      }
    }

    return { markedOverdueCount: markedCount };
  }
}
```

**Scheduled Task Best Practices:**

- Run during off-peak hours
- Use cron expressions for scheduling
- Handle errors gracefully (don't fail entire batch)
- Log all operations for debugging
- Publish events for downstream actions (email notifications, etc.)
- Use system user ID for automated actions

---

## Time Tracking Patterns

### 1. Timer State Management

```typescript
export class TimeEntry {
  startTimer(userId: string): void {
    if (this.status !== TimeEntryStatus.DRAFT) {
      throw new Error('Can only start timer on draft time entries');
    }

    if (this.timerStartedAt) {
      throw new Error('Timer is already running');
    }

    this.timerStartedAt = new Date();
    this.updatedBy = userId;

    this.addDomainEvent(new TimerStartedEvent(this.id, userId));
  }

  stopTimer(userId: string): void {
    if (!this.timerStartedAt) {
      throw new Error('Timer is not running');
    }

    const now = new Date();
    const elapsedMs = now.getTime() - this.timerStartedAt.getTime();
    const hours = elapsedMs / (1000 * 60 * 60);

    // Update duration
    const currentHours = this.duration?.hours || 0;
    this.duration = Duration.fromHours(currentHours + hours);

    this.timerStartedAt = null;
    this.updatedBy = userId;

    this.addDomainEvent(new TimerStoppedEvent(this.id, userId, hours));
  }
}

// Query to find running timer
export class GetRunningTimerHandler {
  async execute(query: GetRunningTimerQuery): Promise<TimeEntry | null> {
    return this.timeEntryRepository.findRunningTimer(query.userId, query.organizationId);
  }
}
```

**Timer Pattern Benefits:**

- Prevents multiple running timers per user
- Accurately captures elapsed time
- State stored in database (survives browser refresh)
- Domain events for real-time UI updates

### 2. Approval Workflow Pattern

```typescript
export class TimeEntry {
  submit(userId: string): void {
    if (this.status !== TimeEntryStatus.DRAFT) {
      throw new Error('Only draft time entries can be submitted');
    }

    if (!this.duration || this.duration.hours === 0) {
      throw new Error('Cannot submit time entry with zero duration');
    }

    if (!this.projectId) {
      throw new Error('Cannot submit time entry without a project');
    }

    this.status = TimeEntryStatus.PENDING_APPROVAL;
    this.submittedAt = new Date();
    this.updatedBy = userId;

    this.addDomainEvent(new TimeEntrySubmittedEvent(this.id, this.projectId));
  }

  approve(approvedBy: string): void {
    if (this.status !== TimeEntryStatus.PENDING_APPROVAL) {
      throw new Error('Can only approve pending time entries');
    }

    this.status = TimeEntryStatus.APPROVED;
    this.comrovedBy = approvedBy;
    this.comrovedAt = new Date();
    this.updatedBy = approvedBy;

    this.addDomainEvent(new TimeEntryApprovedEvent(this.id, this.userId));
  }

  reject(rejectedBy: string, reason: string): void {
    if (this.status !== TimeEntryStatus.PENDING_APPROVAL) {
      throw new Error('Can only reject pending time entries');
    }

    this.status = TimeEntryStatus.REJECTED;
    this.rejectedBy = rejectedBy;
    this.rejectedAt = new Date();
    this.rejectionReason = reason;
    this.updatedBy = rejectedBy;

    this.addDomainEvent(new TimeEntryRejectedEvent(this.id, this.userId, reason));
  }
}

// Bulk approval handler
@CommandHandler(ApproveMultipleTimeEntriesCommand)
export class ApproveMultipleTimeEntriesHandler {
  async execute(command: ApproveMultipleTimeEntriesCommand): Promise<void> {
    const results = await Promise.allSettled(
      command.timeEntryIds.map((id) => this.comroveTimeEntry(id, command.comrovedBy)),
    );

    const succeeded = results.filter((r) => r.status === 'fulfilled').length;
    const failed = results.filter((r) => r.status === 'rejected').length;

    this.logger.log(`Bulk approval completed: ${succeeded} succeeded, ${failed} failed`);
  }
}
```

**Workflow Best Practices:**

- Explicit state transitions with validation
- Capture approval/rejection metadata
- Support bulk operations for efficiency
- Emit events for notifications
- Prevent double-approval with status checks

---

## Project & Resource Management

### 1. Team Assignment Pattern

```typescript
export class ProjectAssignment {
  static create(data: {
    projectId: string;
    userId: string;
    role: string;
    hourlyRate?: number;
    startDate: Date;
    endDate?: Date;
    assignedBy: string;
  }): ProjectAssignment {
    // Validation
    if (data.endDate && data.endDate <= data.startDate) {
      throw new Error('End date must be after start date');
    }

    const assignment = new ProjectAssignment();
    assignment.id = uuidv4();
    assignment.projectId = data.projectId;
    assignment.userId = data.userId;
    assignment.role = data.role;
    assignment.hourlyRate = data.hourlyRate;
    assignment.startDate = data.startDate;
    assignment.endDate = data.endDate;
    assignment.assignedBy = data.assignedBy;
    assignment.createdAt = new Date();

    assignment.addDomainEvent(new TeamMemberAssignedEvent(data.projectId, data.userId, data.role));

    return assignment;
  }

  isActive(date: Date = new Date()): boolean {
    if (date < this.startDate) return false;
    if (this.endDate && date > this.endDate) return false;
    return true;
  }
}

// Query active assignments
export class GetProjectTeamHandler {
  async execute(query: GetProjectTeamQuery): Promise<ProjectAssignment[]> {
    const assignments = await this.assignmentRepository.findByProject(query.projectId);

    // Filter to active assignments
    return assignments.filter((a) => a.isActive());
  }
}
```

**Assignment Best Practices:**

- Date-based validity (start/end dates)
- Per-assignment hourly rates (overrides project default)
- Role tracking for reporting
- Domain events for notifications
- Active assignment queries for scheduling

### 2. Resource Availability Pattern

```typescript
export class ResourceAvailability {
  async getAvailableResources(
    organizationId: string,
    startDate: Date,
    endDate: Date,
  ): Promise<ResourceAvailabilityDTO[]> {
    // 1. Get all users in organization
    const users = await this.userRepository.findByOrganization(organizationId);

    // 2. Get all assignments in date range
    const assignments = await this.assignmentRepository.findInDateRange(
      organizationId,
      startDate,
      endDate,
    );

    // 3. Get time entries in date range
    const timeEntries = await this.timeEntryRepository.findInDateRange(
      organizationId,
      startDate,
      endDate,
    );

    // 4. Calculate availability for each user
    return users.map((user) => {
      const userAssignments = assignments.filter((a) => a.userId === user.id);
      const userTimeEntries = timeEntries.filter((t) => t.userId === user.id);

      const totalHoursLogged = userTimeEntries.reduce(
        (sum, entry) => sum + entry.duration.hours,
        0,
      );

      const activeProjects = userAssignments.filter((a) => a.isActive()).map((a) => a.projectId);

      return {
        userId: user.id,
        name: `${user.firstName} ${user.lastName}`,
        activeProjects,
        hoursLogged: totalHoursLogged,
        utilization: this.calculateUtilization(totalHoursLogged, startDate, endDate),
      };
    });
  }

  private calculateUtilization(hoursLogged: number, startDate: Date, endDate: Date): number {
    // Assume 40-hour work week
    const workDays = this.getWorkDays(startDate, endDate);
    const expectedHours = workDays * 8;

    return expectedHours > 0 ? (hoursLogged / expectedHours) * 100 : 0;
  }
}
```

**Resource Management Insights:**

- Track utilization % for capacity planning
- Identify over/under-allocated resources
- Support project staffing decisions
- Calculate billable vs non-billable time
- Forecast availability based on current assignments

---

## Multi-tenancy in PSA

### 1. Organization Isolation Pattern

```rust
// All queries filtered by organization_id
pub struct ClientRepositoryImpl;

impl ClientRepository for ClientRepositoryImpl {
    async fn find_by_id(&self, pool: &sqlx::PgPool, id: &str) -> Result<Option<Client>, sqlx::Error> {
        let result = sqlx::query_as!(
            ClientRow,
            r#"
            SELECT *
            FROM clients
            WHERE id = $1
            AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(pool)
        .await?;

        Ok(result.map(|row| self.to_domain(row)))
    }

    async fn find_by_organization(
        &self,
        pool: &sqlx::PgPool,
        organization_id: &str,
    ) -> Result<Vec<Client>, sqlx::Error> {
        let results = sqlx::query_as!(
            ClientRow,
            r#"
            SELECT *
            FROM clients
            WHERE organization_id = $1
            AND deleted_at IS NULL
            "#,
            organization_id
        )
        .fetch_all(pool)
        .await?;

        Ok(results.into_iter().map(|row| self.to_domain(row)).collect())
    }
}

// Middleware to enforce organization context (using Axum framework)
pub async fn organization_context_middleware(
    Extension(claims): Extension<JwtClaims>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Inject organization_id from JWT into request extensions
    if let Some(organization_id) = claims.organization_id {
        request.extensions_mut().insert(OrganizationContext {
            organization_id,
        });
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
```

**Isolation Best Practices:**

- Every entity has organizationId
- All queries filtered by organization
- Guards inject organization context
- Database indexes on organizationId
- Foreign keys maintain referential integrity

### 2. Organization Billing Pattern

```typescript
export class OrganizationBilling {
  static readonly FREE_USER_LIMIT = 5;
  static readonly STARTER_USER_LIMIT = 25;
  static readonly PRO_USER_LIMIT = 100;

  static readonly STARTER_PER_USER = 15; // $15/user/month
  static readonly PRO_PER_USER = 25; // $25/user/month
  static readonly ENTERPRISE_BASE = 2500; // $2500/month base

  calculateMonthlyAmount(): number {
    const userCount = this.activeUserCount;

    if (userCount <= OrganizationBilling.FREE_USER_LIMIT) {
      return 0; // FREE tier
    }

    if (userCount <= OrganizationBilling.STARTER_USER_LIMIT) {
      return userCount * OrganizationBilling.STARTER_PER_USER; // STARTER
    }

    if (userCount <= OrganizationBilling.PRO_USER_LIMIT) {
      return userCount * OrganizationBilling.PRO_PER_USER; // PRO
    }

    return OrganizationBilling.ENTERPRISE_BASE; // ENTERPRISE
  }

  getTier(): BillingTier {
    const userCount = this.activeUserCount;

    if (userCount <= OrganizationBilling.FREE_USER_LIMIT) return 'FREE';
    if (userCount <= OrganizationBilling.STARTER_USER_LIMIT) return 'STARTER';
    if (userCount <= OrganizationBilling.PRO_USER_LIMIT) return 'PRO';
    return 'ENTERPRISE';
  }
}

// Event handler to update user count
@EventsHandler(UserActivatedEvent)
export class OnUserActivatedUpdateBilling {
  async handle(event: UserActivatedEvent): Promise<void> {
    const org = await this.orgRepository.findById(event.organizationId);
    if (!org) return;

    org.billing.activeUserCount += 1;
    await this.orgRepository.save(org);
  }
}
```

**Billing Best Practices:**

- Automatic tier calculation based on user count
- Value object encapsulates billing logic
- Event-driven user count updates
- Support for custom enterprise pricing
- Integration points for Stripe/payment gateways

---

## Financial Operations

### 1. Money Value Object Pattern

```typescript
export class Money {
  private readonly _amount: number;
  private readonly _currency: string;

  private constructor(amount: number, currency: string = 'USD') {
    if (amount < 0) {
      throw new Error('Amount cannot be negative');
    }

    // Store as cents to avoid floating point issues
    this._amount = Math.round(amount * 100);
    this._currency = currency.toUpperCase();
  }

  static fromDollars(dollars: number, currency: string = 'USD'): Money {
    return new Money(dollars, currency);
  }

  static fromCents(cents: number, currency: string = 'USD'): Money {
    return new Money(cents / 100, currency);
  }

  get amount(): number {
    return this._amount / 100; // Return as dollars
  }

  get cents(): number {
    return this._amount;
  }

  get currency(): string {
    return this._currency;
  }

  add(other: Money): Money {
    if (this._currency !== other._currency) {
      throw new Error('Cannot add different currencies');
    }
    return Money.fromCents(this._amount + other._amount, this._currency);
  }

  multiply(factor: number): Money {
    return Money.fromCents(Math.round(this._amount * factor), this._currency);
  }

  format(locale: string = 'en-US'): string {
    return new Intl.NumberFormat(locale, {
      style: 'currency',
      currency: this._currency,
    }).format(this.amount);
  }
}
```

**Money Pattern Benefits:**

- Avoids floating point precision errors
- Encapsulates currency logic
- Type-safe arithmetic operations
- Automatic formatting for display
- Prevents currency mixing errors

### 2. Revenue Recognition Pattern

```typescript
export class RevenueRecognition {
  async recognizeRevenue(invoice: Invoice): Promise<void> {
    if (invoice.status !== InvoiceStatus.PAID) {
      throw new Error('Can only recognize revenue for paid invoices');
    }

    // Record revenue by project/client
    const recognition: RevenueRecord = {
      id: uuidv4(),
      organizationId: invoice.organizationId,
      clientId: invoice.clientId,
      invoiceId: invoice.id,
      amount: invoice.total,
      recognizedDate: invoice.paidDate,
      fiscalYear: invoice.paidDate.getFullYear(),
      fiscalQuarter: this.getFiscalQuarter(invoice.paidDate),
    };

    await this.revenueRepository.save(recognition);

    // Update project financials
    for (const lineItem of invoice.lineItems) {
      await this.projectRepository.addRecognizedRevenue(lineItem.projectId, lineItem.totalAmount);
    }
  }

  async getRevenueByPeriod(
    organizationId: string,
    startDate: Date,
    endDate: Date,
  ): Promise<RevenueSummary> {
    const records = await this.revenueRepository.findInDateRange(
      organizationId,
      startDate,
      endDate,
    );

    return {
      totalRevenue: records.reduce((sum, r) => sum + r.amount, 0),
      recordCount: records.length,
      byClient: this.groupByClient(records),
      byProject: this.groupByProject(records),
      byMonth: this.groupByMonth(records),
    };
  }
}
```

**Revenue Best Practices:**

- Recognize revenue only on payment
- Track by fiscal period for reporting
- Link to specific projects and clients
- Support revenue forecasting
- Integrate with accounting systems

---

## Reporting & Analytics

### 1. Time Entry Analytics

```typescript
export class TimeEntryAnalytics {
  async getUtilizationReport(
    organizationId: string,
    startDate: Date,
    endDate: Date,
  ): Promise<UtilizationReport> {
    const timeEntries = await this.timeEntryRepository.findInDateRange(
      organizationId,
      startDate,
      endDate,
    );

    // Calculate billable vs non-billable
    const billable = timeEntries.filter((t) => t.isBillable);
    const nonBillable = timeEntries.filter((t) => !t.isBillable);

    const billableHours = this.sumHours(billable);
    const nonBillableHours = this.sumHours(nonBillable);
    const totalHours = billableHours + nonBillableHours;

    return {
      totalHours,
      billableHours,
      nonBillableHours,
      billablePercentage: (billableHours / totalHours) * 100,
      byUser: this.calculateByUser(timeEntries),
      byProject: this.calculateByProject(timeEntries),
      byClient: this.calculateByClient(timeEntries),
    };
  }

  async getProjectProfitability(projectId: string): Promise<ProfitabilityAnalysis> {
    // Get all time entries for project
    const timeEntries = await this.timeEntryRepository.findByProject(projectId);
    const totalHours = this.sumHours(timeEntries);

    // Get all invoices for project
    const invoices = await this.invoiceRepository.findByProject(projectId);
    const totalRevenue = invoices
      .filter((i) => i.status === InvoiceStatus.PAID)
      .reduce((sum, i) => sum + i.total, 0);

    // Calculate costs (internal hourly rates)
    const costs = await this.calculateInternalCosts(timeEntries);

    return {
      totalHours,
      totalRevenue,
      totalCosts: costs,
      grossProfit: totalRevenue - costs,
      profitMargin: totalRevenue > 0 ? ((totalRevenue - costs) / totalRevenue) * 100 : 0,
    };
  }
}
```

**Analytics Best Practices:**

- Aggregate data for performance
- Cache expensive calculations
- Support date range filtering
- Group by multiple dimensions
- Export to CSV/Excel for reporting

### 2. Dashboard Metrics Pattern

```typescript
export class DashboardService {
  async getKPIs(organizationId: string): Promise<DashboardKPIs> {
    // Run queries in parallel for performance
    const [activeProjects, pendingInvoices, overdueInvoices, utilization, monthlyRevenue] =
      await Promise.all([
        this.projectRepository.countActive(organizationId),
        this.invoiceRepository.countByStatus(organizationId, InvoiceStatus.SENT),
        this.invoiceRepository.countByStatus(organizationId, InvoiceStatus.OVERDUE),
        this.getUtilizationRate(organizationId),
        this.getMonthlyRevenue(organizationId),
      ]);

    return {
      activeProjects,
      pendingInvoices,
      overdueInvoices,
      utilizationRate: utilization,
      monthlyRevenue,
      trends: await this.calculateTrends(organizationId),
    };
  }

  private async getUtilizationRate(organizationId: string): Promise<number> {
    const startOfMonth = startOfMonth(new Date());
    const endOfMonth = endOfMonth(new Date());

    const timeEntries = await this.timeEntryRepository.findInDateRange(
      organizationId,
      startOfMonth,
      endOfMonth,
    );

    const billableHours = timeEntries
      .filter((t) => t.isBillable)
      .reduce((sum, t) => sum + t.duration.hours, 0);

    const totalHours = timeEntries.reduce((sum, t) => sum + t.duration.hours, 0);

    return totalHours > 0 ? (billableHours / totalHours) * 100 : 0;
  }
}
```

**Dashboard Best Practices:**

- Parallel query execution
- Caching for expensive metrics
- Real-time vs cached balance
- Trend calculations (WoW, MoM)
- Drill-down to detail views

---

## Integration Patterns

### 1. QuickBooks Integration Pattern

```typescript
// Anti-Corruption Layer for external accounting system
export class QuickBooksAdapter {
  async syncInvoice(invoice: Invoice): Promise<void> {
    // Map internal invoice to QuickBooks format
    const qbInvoice = {
      CustomerRef: await this.getCustomerRef(invoice.clientId),
      TxnDate: invoice.issueDate.toISOString().split('T')[0],
      DueDate: invoice.dueDate?.toISOString().split('T')[0],
      Line: invoice.lineItems.map((item) => ({
        DetailType: 'SalesItemLineDetail',
        Amount: item.totalAmount,
        SalesItemLineDetail: {
          ItemRef: await this.getItemRef(item.description),
          Qty: item.quantity,
          UnitPrice: item.unitPrice,
        },
      })),
    };

    // Send to QuickBooks API
    await this.quickbooksClient.createInvoice(qbInvoice);
  }

  async syncPayment(invoice: Invoice): Promise<void> {
    const qbPayment = {
      CustomerRef: await this.getCustomerRef(invoice.clientId),
      TotalAmt: invoice.total,
      TxnDate: invoice.paidDate.toISOString().split('T')[0],
      Line: [
        {
          Amount: invoice.total,
          LinkedTxn: [
            {
              TxnId: await this.getQBInvoiceId(invoice.id),
              TxnType: 'Invoice',
            },
          ],
        },
      ],
    };

    await this.quickbooksClient.createPayment(qbPayment);
  }
}

// Event handler for automatic sync
@EventsHandler(InvoicePaidEvent)
export class OnInvoicePaidSyncToQuickBooks {
  async handle(event: InvoicePaidEvent): Promise<void> {
    const invoice = await this.invoiceRepository.findById(event.invoiceId);

    try {
      await this.quickbooksAdapter.syncPayment(invoice);
    } catch (error) {
      // Don't fail the payment if QB sync fails
      this.logger.error('Failed to sync payment to QuickBooks:', error);

      // Queue for retry
      await this.retryQueue.add({
        type: 'quickbooks-payment-sync',
        invoiceId: invoice.id,
        attempt: 1,
      });
    }
  }
}
```

**Integration Best Practices:**

- Anti-corruption layer for external systems
- Async/event-driven sync (don't block operations)
- Retry logic for failed syncs
- Error logging and monitoring
- Idempotency (handle duplicate syncs)

### 2. Email Notification Pattern

```typescript
@EventsHandler(InvoiceSentEvent)
export class OnInvoiceSentSendEmail {
  async handle(event: InvoiceSentEvent): Promise<void> {
    const invoice = await this.invoiceRepository.findById(event.invoiceId);
    const client = await this.clientRepository.findById(event.clientId);
    const org = await this.orgRepository.findById(invoice.organizationId);

    // Generate PDF
    const pdfBuffer = await this.pdfService.generateInvoicePdf(invoice, client, org);

    // Send email
    await this.emailService.send({
      to: client.email.value,
      from: `billing@${org.primaryDomain.getValue()}`,
      subject: `Invoice ${invoice.invoiceNumber} from ${org.name.getValue()}`,
      html: this.renderEmailTemplate(invoice, client, org),
      attachments: [
        {
          filename: `Invoice-${invoice.invoiceNumber}.pdf`,
          content: pdfBuffer,
          contentType: 'application/pdf',
        },
      ],
    });
  }

  private renderEmailTemplate(invoice: Invoice, client: Client, org: Organization): string {
    return `
      <h2>Invoice ${invoice.invoiceNumber}</h2>
      <p>Dear ${client.name},</p>
      <p>
        Please find attached invoice ${invoice.invoiceNumber}
        for ${invoice.total.toFixed(2)} USD.
      </p>
      <p>
        Due Date: ${invoice.dueDate.toLocaleDateString()}<br/>
        Payment Terms: ${invoice.paymentTerms || 'Net 30'}
      </p>
      <p>
        Thank you for your business!<br/>
        ${org.name.getValue()}
      </p>
    `;
  }
}
```

**Email Best Practices:**

- Event-driven (decoupled from business logic)
- Attach generated PDFs
- Professional HTML templates
- Track email delivery status
- Support email preferences/opt-out

---

## Performance Optimization

### 1. Query Optimization Patterns

```rust
// BAD: N+1 query problem
pub async fn get_invoices_bad(pool: &sqlx::PgPool) -> Result<Vec<InvoiceWithClient>, sqlx::Error> {
    let invoices = sqlx::query_as!(InvoiceRow, "SELECT * FROM invoices")
        .fetch_all(pool)
        .await?;

    let mut invoices_with_clients = Vec::new();

    // ❌ N+1: One query per invoice to get client
    for invoice in invoices {
        let client = sqlx::query_as!(
            ClientRow,
            "SELECT * FROM clients WHERE id = $1",
            invoice.client_id
        )
        .fetch_one(pool)
        .await?;

        invoices_with_clients.push(InvoiceWithClient {
            invoice: to_domain(invoice),
            client: to_domain_client(client),
        });
    }

    Ok(invoices_with_clients)
}

// ✅ GOOD: Join to fetch related data
pub async fn find_with_clients(
    pool: &sqlx::PgPool,
    organization_id: &str,
) -> Result<Vec<InvoiceWithClient>, sqlx::Error> {
    let results = sqlx::query!(
        r#"
        SELECT
            i.*,
            c.id as client_id,
            c.name as client_name,
            c.email as client_email
        FROM invoices i
        LEFT JOIN clients c ON i.client_id = c.id
        WHERE i.organization_id = $1
        "#,
        organization_id
    )
    .fetch_all(pool)
    .await?;

    Ok(results
        .into_iter()
        .map(|row| InvoiceWithClient {
            invoice: to_domain_invoice(row),
            client: row.client_id.map(|_| to_domain_client_from_row(row)),
        })
        .collect())
}
```

**Query Optimization Best Practices:**

- Use joins to fetch related data
- Add indexes on foreign keys and filter columns
- Paginate large result sets
- Use projection to fetch only needed columns
- Cache frequently accessed data

### 2. Caching Strategy

```typescript
@Injectable()
export class PermissionCacheService {
  private readonly CACHE_TTL = 300; // 5 minutes

  async getUserPermissions(userId: string): Promise<Permission[]> {
    const cacheKey = `user:${userId}:permissions`;

    // Try cache first
    const cached = await this.redis.get(cacheKey);
    if (cached) {
      return JSON.parse(cached);
    }

    // Cache miss - fetch from DB
    const permissions = await this.roleRepository.getUserPermissions(userId);

    // Store in cache
    await this.redis.setex(cacheKey, this.CACHE_TTL, JSON.stringify(permissions));

    return permissions;
  }

  async invalidateUserCache(userId: string): Promise<void> {
    const cacheKey = `user:${userId}:permissions`;
    await this.redis.del(cacheKey);
  }
}

// Invalidate on role changes
@EventsHandler(UserRoleChangedEvent)
export class OnUserRoleChangedInvalidateCache {
  async handle(event: UserRoleChangedEvent): Promise<void> {
    await this.cacheService.invalidateUserCache(event.userId);
  }
}
```

**Caching Best Practices:**

- Cache frequently read, infrequently changed data
- Use appropriate TTL based on data volatility
- Invalidate cache on writes
- Use cache-aside pattern
- Monitor cache hit rates

### 3. Database Indexing Strategy

```sql
-- Organization filtering (most common)
CREATE INDEX idx_clients_organization_id
ON clients(organization_id)
WHERE deleted_at IS NULL;

-- Email lookups (uniqueness checks)
CREATE UNIQUE INDEX idx_clients_email_org
ON clients(LOWER(email), organization_id)
WHERE deleted_at IS NULL;

-- Invoice filtering by status
CREATE INDEX idx_invoices_status_org
ON invoices(organization_id, status)
WHERE deleted_at IS NULL;

-- Overdue invoice detection
CREATE INDEX idx_invoices_due_date
ON invoices(due_date)
WHERE status = 'SENT' AND deleted_at IS NULL;

-- Time entry queries by user/project
CREATE INDEX idx_time_entries_user_date
ON time_entries(user_id, created_at)
WHERE deleted_at IS NULL;

CREATE INDEX idx_time_entries_project_date
ON time_entries(project_id, created_at)
WHERE deleted_at IS NULL;

-- Composite index for invoice generation
CREATE INDEX idx_time_entries_invoicing
ON time_entries(organization_id, status, billed)
WHERE deleted_at IS NULL;
```

**Indexing Best Practices:**

- Index foreign keys
- Index filter columns (status, deleted_at)
- Partial indexes with WHERE clause for soft deletes
- Composite indexes for common query patterns
- Monitor query performance with EXPLAIN ANALYZE

---

## Summary

### Core PSA Patterns Checklist

✅ **Domain Modeling**

- Rich domain entities with business logic
- Value objects for domain concepts (Money, Email, Duration)
- Domain events for integration
- State machines for lifecycle management

✅ **Multi-tenancy**

- Organization-based isolation
- Filtered queries by organizationId
- Guards for context injection
- Separate billing per organization

✅ **Financial Operations**

- Invoice lifecycle (Draft → Sent → Paid → Overdue)
- Automated invoice generation from time
- PDF generation with professional formatting
- Revenue recognition on payment

✅ **Time Tracking**

- Timer functionality with accurate elapsed time
- Approval workflows (Draft → Pending → Approved/Rejected)
- Billable vs non-billable tracking
- Project and client association

✅ **Performance**

- Query optimization (joins, indexes)
- Caching strategy (Redis)
- Pagination for large datasets
- Parallel query execution

✅ **Integration**

- Anti-corruption layers for external systems
- Event-driven sync (don't block operations)
- Retry logic for failures
- Email notifications

---

## Related Patterns

- [CQRS Pattern](./05-CQRS-Pattern.md)
- [Repository Pattern](./06-Repository-Pattern.md)
- [Domain-Driven Design](./04-Domain-Driven-Design.md)
- [Database Constraint Race Condition Pattern](./41-Database-Constraint-Race-Condition-Pattern.md)
- [User-Friendly Error Handling Pattern](./52-User-Friendly-Error-Handling-Pattern.md)
- [Database Performance Optimization Pattern](./53-Database-Performance-Optimization-Pattern.md)

---

**Version History:**

- 1.0 (October 2025): Initial comprehensive PSA patterns document based on WellOS implementation
