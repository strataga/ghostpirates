# Strategy Pattern

## Overview

The Strategy pattern defines a family of algorithms, encapsulates each one, and
makes them interchangeable. It lets the algorithm vary independently from
clients that use it. This pattern is useful when you have multiple ways to
perform a task and want to choose the implementation at runtime.

## Core Concepts

### Strategy Interface

Defines the contract that all concrete strategies must implement.

### Concrete Strategies

Different implementations of the strategy interface, each representing a
different algorithm.

### Context

The class that uses a strategy to perform some task.

### Strategy Selection

The mechanism by which the appropriate strategy is chosen.

## Benefits

- **Flexibility**: Easy to add new algorithms without modifying existing code
- **Runtime Selection**: Can change algorithms at runtime based on conditions
- **Testability**: Each strategy can be tested independently
- **Single Responsibility**: Each strategy has one reason to change
- **Open/Closed Principle**: Open for extension, closed for modification
- **Eliminates Conditionals**: Replaces complex if-else chains with polymorphism

## Implementation in Our Project

### Before: Hardcoded Conditional Logic

```typescript
@Injectable()
export class PaymentCalculationService {
  async calculatePayment(
    vendor: Vendor,
    amount: Money,
    paymentType: string,
    urgency: string,
  ): Promise<PaymentCalculation> {
    let fee: Money;
    let processingTime: number;
    let requiredApprovals: string[];

    // Complex conditional logic for different payment types
    if (paymentType === 'ACH') {
      if (urgency === 'STANDARD') {
        fee = Money.fromAmount(5.0, 'USD');
        processingTime = 3; // days
        requiredApprovals = ['MANAGER'];
      } else if (urgency === 'EXPEDITED') {
        fee = Money.fromAmount(15.0, 'USD');
        processingTime = 1;
        requiredApprovals = ['MANAGER', 'FINANCE'];
      } else if (urgency === 'EMERGENCY') {
        fee = Money.fromAmount(50.0, 'USD');
        processingTime = 0; // same day
        requiredApprovals = ['MANAGER', 'FINANCE', 'EXECUTIVE'];
      }
    } else if (paymentType === 'WIRE') {
      if (urgency === 'STANDARD') {
        fee = Money.fromAmount(25.0, 'USD');
        processingTime = 1;
        requiredApprovals = ['MANAGER', 'FINANCE'];
      } else if (urgency === 'EXPEDITED') {
        fee = Money.fromAmount(45.0, 'USD');
        processingTime = 0;
        requiredApprovals = ['MANAGER', 'FINANCE', 'EXECUTIVE'];
      }
      // Wire transfers don't support emergency processing
    } else if (paymentType === 'CHECK') {
      if (urgency === 'STANDARD') {
        fee = Money.fromAmount(2.0, 'USD');
        processingTime = 7;
        requiredApprovals = ['MANAGER'];
      }
      // Checks only support standard processing
    } else {
      throw new Error(`Unsupported payment type: ${paymentType}`);
    }

    // Additional business rules mixed in
    if (amount.getAmount() > 50000) {
      requiredApprovals.push('EXECUTIVE');
      fee = fee.add(Money.fromAmount(100.0, 'USD'));
    }

    if (vendor.getRiskScore() > 7) {
      requiredApprovals.push('RISK_MANAGER');
      processingTime += 1;
    }

    return new PaymentCalculation(amount, fee, processingTime, requiredApprovals);
  }

  async processPayment(payment: Payment): Promise<PaymentResult> {
    const type = payment.getType();

    // More conditional logic for processing
    if (type === 'ACH') {
      // ACH-specific processing logic
      const achGateway = new AchPaymentGateway();
      const result = await achGateway.processPayment({
        accountNumber: payment.getAccountNumber(),
        routingNumber: payment.getRoutingNumber(),
        amount: payment.getAmount().getAmount(),
        description: payment.getDescription(),
      });

      return new PaymentResult(
        payment.getId(),
        result.success,
        result.transactionId,
        result.errorMessage,
      );
    } else if (type === 'WIRE') {
      // Wire-specific processing logic
      const wireGateway = new WireTransferGateway();
      const result = await wireGateway.sendWire({
        beneficiaryBank: payment.getBeneficiaryBank(),
        beneficiaryAccount: payment.getBeneficiaryAccount(),
        amount: payment.getAmount().getAmount(),
        reference: payment.getReference(),
      });

      return new PaymentResult(payment.getId(), result.success, result.wireId, result.errorMessage);
    } else if (type === 'CHECK') {
      // Check printing logic
      const checkPrinter = new CheckPrintingService();
      await checkPrinter.printCheck({
        payee: payment.getPayee(),
        amount: payment.getAmount().getAmount(),
        memo: payment.getMemo(),
      });

      return new PaymentResult(payment.getId(), true, generateCheckNumber(), null);
    } else {
      throw new Error(`Unsupported payment type: ${type}`);
    }
  }
}
```

### After: Strategy Pattern Implementation

```typescript
// Strategy interface for payment calculation
export interface IPaymentCalculationStrategy {
  calculateFee(amount: Money, urgency: PaymentUrgency): Money;
  getProcessingTime(urgency: PaymentUrgency): number;
  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[];
  supportsUrgency(urgency: PaymentUrgency): boolean;
}

// Concrete strategies for different payment types
export class AchPaymentStrategy implements IPaymentCalculationStrategy {
  calculateFee(amount: Money, urgency: PaymentUrgency): Money {
    switch (urgency) {
      case PaymentUrgency.STANDARD:
        return Money.fromAmount(5.0, amount.getCurrency());
      case PaymentUrgency.EXPEDITED:
        return Money.fromAmount(15.0, amount.getCurrency());
      case PaymentUrgency.EMERGENCY:
        return Money.fromAmount(50.0, amount.getCurrency());
      default:
        throw new UnsupportedUrgencyError(urgency);
    }
  }

  getProcessingTime(urgency: PaymentUrgency): number {
    switch (urgency) {
      case PaymentUrgency.STANDARD:
        return 3;
      case PaymentUrgency.EXPEDITED:
        return 1;
      case PaymentUrgency.EMERGENCY:
        return 0;
      default:
        throw new UnsupportedUrgencyError(urgency);
    }
  }

  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[] {
    const approvals: ApprovalLevel[] = [ApprovalLevel.MANAGER];

    if (urgency !== PaymentUrgency.STANDARD) {
      approvals.push(ApprovalLevel.FINANCE);
    }

    if (urgency === PaymentUrgency.EMERGENCY) {
      approvals.push(ApprovalLevel.EXECUTIVE);
    }

    return approvals;
  }

  supportsUrgency(urgency: PaymentUrgency): boolean {
    return [PaymentUrgency.STANDARD, PaymentUrgency.EXPEDITED, PaymentUrgency.EMERGENCY].includes(
      urgency,
    );
  }
}

export class WireTransferStrategy implements IPaymentCalculationStrategy {
  calculateFee(amount: Money, urgency: PaymentUrgency): Money {
    switch (urgency) {
      case PaymentUrgency.STANDARD:
        return Money.fromAmount(25.0, amount.getCurrency());
      case PaymentUrgency.EXPEDITED:
        return Money.fromAmount(45.0, amount.getCurrency());
      default:
        throw new UnsupportedUrgencyError(urgency);
    }
  }

  getProcessingTime(urgency: PaymentUrgency): number {
    switch (urgency) {
      case PaymentUrgency.STANDARD:
        return 1;
      case PaymentUrgency.EXPEDITED:
        return 0;
      default:
        throw new UnsupportedUrgencyError(urgency);
    }
  }

  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[] {
    const approvals: ApprovalLevel[] = [ApprovalLevel.MANAGER, ApprovalLevel.FINANCE];

    if (urgency === PaymentUrgency.EXPEDITED) {
      approvals.push(ApprovalLevel.EXECUTIVE);
    }

    return approvals;
  }

  supportsUrgency(urgency: PaymentUrgency): boolean {
    return [PaymentUrgency.STANDARD, PaymentUrgency.EXPEDITED].includes(urgency);
  }
}

export class CheckPaymentStrategy implements IPaymentCalculationStrategy {
  calculateFee(amount: Money, urgency: PaymentUrgency): Money {
    if (urgency !== PaymentUrgency.STANDARD) {
      throw new UnsupportedUrgencyError(urgency);
    }
    return Money.fromAmount(2.0, amount.getCurrency());
  }

  getProcessingTime(urgency: PaymentUrgency): number {
    if (urgency !== PaymentUrgency.STANDARD) {
      throw new UnsupportedUrgencyError(urgency);
    }
    return 7;
  }

  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[] {
    return [ApprovalLevel.MANAGER];
  }

  supportsUrgency(urgency: PaymentUrgency): boolean {
    return urgency === PaymentUrgency.STANDARD;
  }
}

// Payment processing strategies
export interface IPaymentProcessingStrategy {
  processPayment(payment: Payment): Promise<PaymentResult>;
  getPaymentType(): PaymentType;
}

export class AchProcessingStrategy implements IPaymentProcessingStrategy {
  constructor(private readonly achGateway: IAchPaymentGateway) {}

  async processPayment(payment: Payment): Promise<PaymentResult> {
    const result = await this.achGateway.processPayment({
      accountNumber: payment.getAccountNumber(),
      routingNumber: payment.getRoutingNumber(),
      amount: payment.getAmount().getAmount(),
      description: payment.getDescription(),
    });

    return new PaymentResult(
      payment.getId(),
      result.success,
      result.transactionId,
      result.errorMessage,
    );
  }

  getPaymentType(): PaymentType {
    return PaymentType.ACH;
  }
}

export class WireProcessingStrategy implements IPaymentProcessingStrategy {
  constructor(private readonly wireGateway: IWireTransferGateway) {}

  async processPayment(payment: Payment): Promise<PaymentResult> {
    const result = await this.wireGateway.sendWire({
      beneficiaryBank: payment.getBeneficiaryBank(),
      beneficiaryAccount: payment.getBeneficiaryAccount(),
      amount: payment.getAmount().getAmount(),
      reference: payment.getReference(),
    });

    return new PaymentResult(payment.getId(), result.success, result.wireId, result.errorMessage);
  }

  getPaymentType(): PaymentType {
    return PaymentType.WIRE;
  }
}

// Context classes using strategies
@Injectable()
export class PaymentCalculationService {
  private strategies: Map<PaymentType, IPaymentCalculationStrategy> = new Map();

  constructor() {
    this.strategies.set(PaymentType.ACH, new AchPaymentStrategy());
    this.strategies.set(PaymentType.WIRE, new WireTransferStrategy());
    this.strategies.set(PaymentType.CHECK, new CheckPaymentStrategy());
  }

  async calculatePayment(
    vendor: Vendor,
    amount: Money,
    paymentType: PaymentType,
    urgency: PaymentUrgency,
  ): Promise<PaymentCalculation> {
    const strategy = this.strategies.get(paymentType);

    if (!strategy) {
      throw new UnsupportedPaymentTypeError(paymentType);
    }

    if (!strategy.supportsUrgency(urgency)) {
      throw new UnsupportedUrgencyError(urgency);
    }

    let fee = strategy.calculateFee(amount, urgency);
    let processingTime = strategy.getProcessingTime(urgency);
    let requiredApprovals = strategy.getRequiredApprovals(amount, urgency);

    // Apply business rules that are common across all payment types
    if (amount.getAmount() > 50000) {
      requiredApprovals = this.addHighValueApproval(requiredApprovals);
      fee = fee.add(Money.fromAmount(100.0, amount.getCurrency()));
    }

    if (vendor.getRiskScore() > 7) {
      requiredApprovals = this.addRiskApproval(requiredApprovals);
      processingTime += 1;
    }

    return new PaymentCalculation(amount, fee, processingTime, requiredApprovals);
  }

  private addHighValueApproval(approvals: ApprovalLevel[]): ApprovalLevel[] {
    const result = [...comrovals];
    if (!result.includes(ApprovalLevel.EXECUTIVE)) {
      result.push(ApprovalLevel.EXECUTIVE);
    }
    return result;
  }

  private addRiskApproval(approvals: ApprovalLevel[]): ApprovalLevel[] {
    const result = [...comrovals];
    if (!result.includes(ApprovalLevel.RISK_MANAGER)) {
      result.push(ApprovalLevel.RISK_MANAGER);
    }
    return result;
  }
}

@Injectable()
export class PaymentProcessingService {
  private strategies: Map<PaymentType, IPaymentProcessingStrategy> = new Map();

  constructor(
    achGateway: IAchPaymentGateway,
    wireGateway: IWireTransferGateway,
    checkService: ICheckPrintingService,
  ) {
    this.strategies.set(PaymentType.ACH, new AchProcessingStrategy(achGateway));
    this.strategies.set(PaymentType.WIRE, new WireProcessingStrategy(wireGateway));
    this.strategies.set(PaymentType.CHECK, new CheckProcessingStrategy(checkService));
  }

  async processPayment(payment: Payment): Promise<PaymentResult> {
    const strategy = this.strategies.get(payment.getType());

    if (!strategy) {
      throw new UnsupportedPaymentTypeError(payment.getType());
    }

    return await strategy.processPayment(payment);
  }
}
```

## Advanced Strategy Patterns

### Dynamic Strategy Selection

```typescript
// Strategy factory based on business rules
@Injectable()
export class PaymentStrategyFactory {
  constructor(
    private readonly vendorRepository: IVendorRepository,
    private readonly configurationService: IConfigurationService,
  ) {}

  async createCalculationStrategy(
    vendorId: VendorId,
    amount: Money,
    urgency: PaymentUrgency,
  ): Promise<IPaymentCalculationStrategy> {
    const vendor = await this.vendorRepository.findById(vendorId);

    if (!vendor) {
      throw new VendorNotFoundError(vendorId.getValue());
    }

    // Business rules for strategy selection
    const preferredPaymentType = this.determinePreferredPaymentType(vendor, amount);

    switch (preferredPaymentType) {
      case PaymentType.ACH:
        return new AchPaymentStrategy();

      case PaymentType.WIRE:
        return new WireTransferStrategy();

      case PaymentType.CHECK:
        return new CheckPaymentStrategy();

      default:
        throw new Error(`Cannot determine payment strategy for vendor ${vendorId.getValue()}`);
    }
  }

  private determinePreferredPaymentType(vendor: Vendor, amount: Money): PaymentType {
    // High-value payments prefer wire transfers
    if (amount.getAmount() > 100000) {
      return PaymentType.WIRE;
    }

    // International vendors might require wire transfers
    if (vendor.isInternational()) {
      return PaymentType.WIRE;
    }

    // Vendors without electronic banking use checks
    if (!vendor.hasElectronicBankingInfo()) {
      return PaymentType.CHECK;
    }

    // Default to ACH for domestic vendors with banking info
    return PaymentType.ACH;
  }
}

// Usage with dynamic strategy selection
@Injectable()
export class PaymentCommandHandler implements ICommandHandler<CreatePaymentCommand> {
  constructor(
    private readonly strategyFactory: PaymentStrategyFactory,
    private readonly paymentRepository: IPaymentRepository,
  ) {}

  async execute(command: CreatePaymentCommand): Promise<string> {
    const strategy = await this.strategyFactory.createCalculationStrategy(
      new VendorId(command.vendorId),
      Money.fromAmount(command.amount, command.currency),
      command.urgency,
    );

    const calculation = await strategy.calculatePayment(
      command.vendorId,
      command.amount,
      command.urgency,
    );

    const payment = Payment.create({
      vendorId: new VendorId(command.vendorId),
      amount: Money.fromAmount(command.amount, command.currency),
      calculation: calculation,
      urgency: command.urgency,
    });

    await this.paymentRepository.save(payment);

    return payment.getId().getValue();
  }
}
```

### Composite Strategies

```typescript
// Composite strategy that combines multiple strategies
export class CompositePaymentStrategy implements IPaymentCalculationStrategy {
  constructor(
    private readonly baseStrategy: IPaymentCalculationStrategy,
    private readonly modifiers: IPaymentModifier[],
  ) {}

  calculateFee(amount: Money, urgency: PaymentUrgency): Money {
    let fee = this.baseStrategy.calculateFee(amount, urgency);

    for (const modifier of this.modifiers) {
      fee = modifier.modifyFee(fee, amount, urgency);
    }

    return fee;
  }

  getProcessingTime(urgency: PaymentUrgency): number {
    let time = this.baseStrategy.getProcessingTime(urgency);

    for (const modifier of this.modifiers) {
      time = modifier.modifyProcessingTime(time, urgency);
    }

    return time;
  }

  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[] {
    let approvals = this.baseStrategy.getRequiredApprovals(amount, urgency);

    for (const modifier of this.modifiers) {
      approvals = modifier.modifyApprovals(approvals, amount, urgency);
    }

    return approvals;
  }

  supportsUrgency(urgency: PaymentUrgency): boolean {
    return this.baseStrategy.supportsUrgency(urgency);
  }
}

// Payment modifiers
export interface IPaymentModifier {
  modifyFee(currentFee: Money, amount: Money, urgency: PaymentUrgency): Money;
  modifyProcessingTime(currentTime: number, urgency: PaymentUrgency): number;
  modifyApprovals(
    currentApprovals: ApprovalLevel[],
    amount: Money,
    urgency: PaymentUrgency,
  ): ApprovalLevel[];
}

export class HighValuePaymentModifier implements IPaymentModifier {
  private readonly threshold: number;

  constructor(threshold: number = 50000) {
    this.threshold = threshold;
  }

  modifyFee(currentFee: Money, amount: Money, urgency: PaymentUrgency): Money {
    if (amount.getAmount() > this.threshold) {
      return currentFee.add(Money.fromAmount(100, currentFee.getCurrency()));
    }
    return currentFee;
  }

  modifyProcessingTime(currentTime: number, urgency: PaymentUrgency): number {
    return currentTime; // No change to processing time
  }

  modifyApprovals(
    currentApprovals: ApprovalLevel[],
    amount: Money,
    urgency: PaymentUrgency,
  ): ApprovalLevel[] {
    if (amount.getAmount() > this.threshold) {
      const result = [...currentApprovals];
      if (!result.includes(ApprovalLevel.EXECUTIVE)) {
        result.push(ApprovalLevel.EXECUTIVE);
      }
      return result;
    }
    return currentApprovals;
  }
}

export class VendorRiskModifier implements IPaymentModifier {
  constructor(private readonly vendorRepository: IVendorRepository) {}

  modifyFee(currentFee: Money, amount: Money, urgency: PaymentUrgency): Money {
    return currentFee; // No fee modification for risk
  }

  modifyProcessingTime(currentTime: number, urgency: PaymentUrgency): number {
    // Risk assessment might add processing time
    return currentTime + 1;
  }

  async modifyApprovals(
    currentApprovals: ApprovalLevel[],
    amount: Money,
    urgency: PaymentUrgency,
  ): Promise<ApprovalLevel[]> {
    // This would require vendor information - simplified for example
    const result = [...currentApprovals];
    result.push(ApprovalLevel.RISK_MANAGER);
    return result;
  }
}
```

## Complex Strategy Examples

### LOS Processing Strategies

```typescript
// Different strategies for processing Lease Operating Statements
export interface ILosProcessingStrategy {
  processExpenses(los: LeaseOperatingStatement): Promise<void>;
  calculateAllocations(los: LeaseOperatingStatement): Promise<AllocationResult[]>;
  generateReports(los: LeaseOperatingStatement): Promise<ReportResult[]>;
  getProcessingType(): LosProcessingType;
}

export class StandardLosProcessingStrategy implements ILosProcessingStrategy {
  constructor(
    private readonly allocationService: IAllocationService,
    private readonly reportService: IReportService,
  ) {}

  async processExpenses(los: LeaseOperatingStatement): Promise<void> {
    // Standard expense processing logic
    const expenses = los.getExpenseLineItems();

    for (const expense of expenses) {
      await this.validateExpense(expense);
      await this.categorizeExpense(expense);
    }
  }

  async calculateAllocations(los: LeaseOperatingStatement): Promise<AllocationResult[]> {
    // Standard allocation based on working interest
    return await this.allocationService.calculateWorkingInterestAllocations(los);
  }

  async generateReports(los: LeaseOperatingStatement): Promise<ReportResult[]> {
    return [
      await this.reportService.generateStandardLosReport(los),
      await this.reportService.generateExpenseSummary(los),
    ];
  }

  getProcessingType(): LosProcessingType {
    return LosProcessingType.STANDARD;
  }
}

export class JointVentureLosProcessingStrategy implements ILosProcessingStrategy {
  constructor(
    private readonly jvAllocationService: IJointVentureAllocationService,
    private readonly reportService: IReportService,
  ) {}

  async processExpenses(los: LeaseOperatingStatement): Promise<void> {
    const expenses = los.getExpenseLineItems();

    for (const expense of expenses) {
      await this.validateExpense(expense);
      await this.categorizeExpense(expense);
      await this.allocateToJointVenturePartners(expense);
    }
  }

  async calculateAllocations(los: LeaseOperatingStatement): Promise<AllocationResult[]> {
    // Joint venture allocation based on participation agreements
    return await this.jvAllocationService.calculateJointVentureAllocations(los);
  }

  async generateReports(los: LeaseOperatingStatement): Promise<ReportResult[]> {
    return [
      await this.reportService.generateJointVentureReport(los),
      await this.reportService.generatePartnerAllocationReport(los),
      await this.reportService.generateExpenseSummary(los),
    ];
  }

  getProcessingType(): LosProcessingType {
    return LosProcessingType.JOINT_VENTURE;
  }
}

// Context for LOS processing
@Injectable()
export class LosProcessingService {
  private strategies: Map<LosProcessingType, ILosProcessingStrategy> = new Map();

  constructor(
    standardStrategy: StandardLosProcessingStrategy,
    jointVentureStrategy: JointVentureLosProcessingStrategy,
    farmoutStrategy: FarmoutLosProcessingStrategy,
  ) {
    this.strategies.set(LosProcessingType.STANDARD, standardStrategy);
    this.strategies.set(LosProcessingType.JOINT_VENTURE, jointVentureStrategy);
    this.strategies.set(LosProcessingType.FARMOUT, farmoutStrategy);
  }

  async processLos(los: LeaseOperatingStatement): Promise<LosProcessingResult> {
    const processingType = this.determineProcessingType(los);
    const strategy = this.strategies.get(processingType);

    if (!strategy) {
      throw new UnsupportedLosProcessingTypeError(processingType);
    }

    await strategy.processExpenses(los);
    const allocations = await strategy.calculateAllocations(los);
    const reports = await strategy.generateReports(los);

    return new LosProcessingResult(los.getId(), processingType, allocations, reports);
  }

  private determineProcessingType(los: LeaseOperatingStatement): LosProcessingType {
    // Business logic to determine processing strategy
    const lease = los.getLease();

    if (lease.hasJointVenturePartners()) {
      return LosProcessingType.JOINT_VENTURE;
    }

    if (lease.hasFarmoutAgreements()) {
      return LosProcessingType.FARMOUT;
    }

    return LosProcessingType.STANDARD;
  }
}
```

## Testing Strategies

### Strategy Testing

```typescript
describe('PaymentCalculationStrategies', () => {
  describe('AchPaymentStrategy', () => {
    let strategy: AchPaymentStrategy;

    beforeEach(() => {
      strategy = new AchPaymentStrategy();
    });

    describe('calculateFee', () => {
      it('should calculate standard ACH fee', () => {
        const amount = Money.fromAmount(1000, 'USD');
        const fee = strategy.calculateFee(amount, PaymentUrgency.STANDARD);

        expect(fee.getAmount()).toBe(5.0);
        expect(fee.getCurrency()).toBe('USD');
      });

      it('should calculate expedited ACH fee', () => {
        const amount = Money.fromAmount(1000, 'USD');
        const fee = strategy.calculateFee(amount, PaymentUrgency.EXPEDITED);

        expect(fee.getAmount()).toBe(15.0);
      });

      it('should calculate emergency ACH fee', () => {
        const amount = Money.fromAmount(1000, 'USD');
        const fee = strategy.calculateFee(amount, PaymentUrgency.EMERGENCY);

        expect(fee.getAmount()).toBe(50.0);
      });
    });

    describe('getProcessingTime', () => {
      it('should return correct processing times', () => {
        expect(strategy.getProcessingTime(PaymentUrgency.STANDARD)).toBe(3);
        expect(strategy.getProcessingTime(PaymentUrgency.EXPEDITED)).toBe(1);
        expect(strategy.getProcessingTime(PaymentUrgency.EMERGENCY)).toBe(0);
      });
    });

    describe('supportsUrgency', () => {
      it('should support all urgency levels', () => {
        expect(strategy.supportsUrgency(PaymentUrgency.STANDARD)).toBe(true);
        expect(strategy.supportsUrgency(PaymentUrgency.EXPEDITED)).toBe(true);
        expect(strategy.supportsUrgency(PaymentUrgency.EMERGENCY)).toBe(true);
      });
    });
  });

  describe('WireTransferStrategy', () => {
    let strategy: WireTransferStrategy;

    beforeEach(() => {
      strategy = new WireTransferStrategy();
    });

    describe('supportsUrgency', () => {
      it('should not support emergency urgency', () => {
        expect(strategy.supportsUrgency(PaymentUrgency.STANDARD)).toBe(true);
        expect(strategy.supportsUrgency(PaymentUrgency.EXPEDITED)).toBe(true);
        expect(strategy.supportsUrgency(PaymentUrgency.EMERGENCY)).toBe(false);
      });
    });
  });
});

describe('PaymentCalculationService', () => {
  let service: PaymentCalculationService;
  let vendor: Vendor;

  beforeEach(() => {
    service = new PaymentCalculationService();
    vendor = createTestVendor({ riskScore: 5 });
  });

  it('should use ACH strategy for ACH payments', async () => {
    const amount = Money.fromAmount(1000, 'USD');

    const calculation = await service.calculatePayment(
      vendor,
      amount,
      PaymentType.ACH,
      PaymentUrgency.STANDARD,
    );

    expect(calculation.getFee().getAmount()).toBe(5.0);
    expect(calculation.getProcessingTime()).toBe(3);
  });

  it('should apply high value surcharge', async () => {
    const amount = Money.fromAmount(60000, 'USD');

    const calculation = await service.calculatePayment(
      vendor,
      amount,
      PaymentType.ACH,
      PaymentUrgency.STANDARD,
    );

    expect(calculation.getFee().getAmount()).toBe(105.0); // 5 + 100
    expect(calculation.getRequiredApprovals()).toContain(ApprovalLevel.EXECUTIVE);
  });

  it('should apply risk adjustments', async () => {
    const highRiskVendor = createTestVendor({ riskScore: 8 });
    const amount = Money.fromAmount(1000, 'USD');

    const calculation = await service.calculatePayment(
      highRiskVendor,
      amount,
      PaymentType.ACH,
      PaymentUrgency.STANDARD,
    );

    expect(calculation.getProcessingTime()).toBe(4); // 3 + 1
    expect(calculation.getRequiredApprovals()).toContain(ApprovalLevel.RISK_MANAGER);
  });

  it('should throw error for unsupported payment type', async () => {
    const amount = Money.fromAmount(1000, 'USD');

    await expect(
      service.calculatePayment(
        vendor,
        amount,
        'CRYPTOCURRENCY' as PaymentType,
        PaymentUrgency.STANDARD,
      ),
    ).rejects.toThrow(UnsupportedPaymentTypeError);
  });
});
```

## Best Practices

### 1. Clear Strategy Interface

```typescript
// Good: Clear, focused interface
export interface IPaymentCalculationStrategy {
  calculateFee(amount: Money, urgency: PaymentUrgency): Money;
  getProcessingTime(urgency: PaymentUrgency): number;
  getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[];
  supportsUrgency(urgency: PaymentUrgency): boolean;
}

// Avoid: Interface that's too broad
export interface IPaymentStrategy {
  calculateEverything(params: any): any; // Too generic
  handlePayment(data: any): any; // Unclear responsibility
}
```

### 2. Strategy Registration

```typescript
// Good: Registry pattern for managing strategies
@Injectable()
export class PaymentStrategyRegistry {
  private strategies: Map<PaymentType, IPaymentCalculationStrategy> = new Map();

  register(paymentType: PaymentType, strategy: IPaymentCalculationStrategy): void {
    this.strategies.set(paymentType, strategy);
  }

  getStrategy(paymentType: PaymentType): IPaymentCalculationStrategy {
    const strategy = this.strategies.get(paymentType);
    if (!strategy) {
      throw new UnsupportedPaymentTypeError(paymentType);
    }
    return strategy;
  }

  getSupportedTypes(): PaymentType[] {
    return Array.from(this.strategies.keys());
  }
}
```

### 3. Strategy Validation

```typescript
// Good: Validate strategy capabilities
export abstract class BasePaymentStrategy implements IPaymentCalculationStrategy {
  abstract calculateFee(amount: Money, urgency: PaymentUrgency): Money;
  abstract getProcessingTime(urgency: PaymentUrgency): number;
  abstract getRequiredApprovals(amount: Money, urgency: PaymentUrgency): ApprovalLevel[];
  abstract supportsUrgency(urgency: PaymentUrgency): boolean;

  protected validateUrgency(urgency: PaymentUrgency): void {
    if (!this.supportsUrgency(urgency)) {
      throw new UnsupportedUrgencyError(
        `${this.constructor.name} does not support urgency: ${urgency}`,
      );
    }
  }

  protected validateAmount(amount: Money): void {
    if (amount.getAmount() <= 0) {
      throw new InvalidAmountError('Payment amount must be greater than zero');
    }
  }
}
```

The Strategy pattern in our oil & gas management system allows us to handle
different payment processing methods, LOS processing types, and other business
algorithms in a flexible, maintainable way that can be easily extended and
tested.
