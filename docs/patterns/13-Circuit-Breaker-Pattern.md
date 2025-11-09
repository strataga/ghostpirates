# Circuit Breaker Pattern (Bulkhead Pattern)

## Overview

The Circuit Breaker pattern prevents an application from repeatedly trying to
execute an operation that's likely to fail. It acts like an electrical circuit
breaker, monitoring calls to remote services and switching to "open" state when
failure rates exceed a threshold, preventing cascading failures and allowing the
system to recover.

## Core Concepts

### Closed State

Normal operation where requests pass through and are executed.

### Open State

Failure state where requests are immediately rejected without execution.

### Half-Open State

Testing state that allows a limited number of test requests to check if the
service has recovered.

### Failure Threshold

The number or percentage of failures that triggers the circuit breaker to open.

### Recovery Timeout

The time to wait before attempting to close the circuit breaker.

## Benefits

- **Fault Tolerance**: Prevents cascading failures across services
- **Fast Failure**: Immediate failure response instead of hanging operations
- **Automatic Recovery**: Self-healing mechanism for transient failures
- **Resource Protection**: Prevents resource exhaustion from repeated failures
- **System Stability**: Maintains overall system stability during partial
  outages
- **Performance**: Avoids slow operations when services are degraded

## Implementation in Our Project

### Before: No Circuit Breaking

```typescript
@Injectable()
export class ExternalIntegrationService {
  constructor(private readonly httpService: HttpService) {}

  async syncVendorToERP(vendor: Vendor): Promise<void> {
    // No circuit breaking - keeps trying even when ERP is down
    try {
      const response = await this.httpService
        .post('/api/vendors', {
          vendorId: vendor.getId().getValue(),
          name: vendor.getName().getValue(),
          code: vendor.getCode().getValue(),
          // ... other vendor data
        })
        .toPromise();

      if (response.status !== 200) {
        throw new Error('ERP sync failed');
      }
    } catch (error) {
      // Just logs error but keeps trying next time
      console.error('Failed to sync vendor to ERP', error);
      throw error;
    }
  }

  async updatePaymentStatus(paymentId: string, status: string): Promise<void> {
    // No protection against repeated failures
    try {
      await this.httpService
        .put(`/api/payments/${paymentId}`, {
          status: status,
          updatedAt: new Date().toISOString(),
        })
        .toPromise();
    } catch (error) {
      // Continues to hammer failing service
      console.error('Failed to update payment status', error);
      throw error;
    }
  }

  async sendNotification(notification: NotificationData): Promise<void> {
    // All external calls without protection
    try {
      await this.httpService.post('/api/notifications', notification).toPromise();
    } catch (error) {
      console.error('Failed to send notification', error);
      throw error;
    }
  }
}
```

### After: Circuit Breaker Implementation

```typescript
// Circuit Breaker States
export enum CircuitBreakerState {
  CLOSED = 'CLOSED',
  OPEN = 'OPEN',
  HALF_OPEN = 'HALF_OPEN',
}

// Circuit Breaker Configuration
export interface CircuitBreakerConfig {
  failureThreshold: number;
  recoveryTimeoutMs: number;
  monitoringPeriodMs: number;
  halfOpenMaxCalls: number;
  minimumThroughput: number;
}

// Circuit Breaker Statistics
export interface CircuitBreakerStats {
  totalCalls: number;
  successfulCalls: number;
  failedCalls: number;
  failureRate: number;
  averageResponseTime: number;
  lastFailureTime: Date | null;
}

// Circuit Breaker Implementation
export class CircuitBreaker {
  private state: CircuitBreakerState = CircuitBreakerState.CLOSED;
  private failureCount = 0;
  private successCount = 0;
  private halfOpenCalls = 0;
  private lastFailureTime: Date | null = null;
  private lastExecutionTime: Date | null = null;
  private stats: CircuitBreakerStats = {
    totalCalls: 0,
    successfulCalls: 0,
    failedCalls: 0,
    failureRate: 0,
    averageResponseTime: 0,
    lastFailureTime: null,
  };

  constructor(
    private readonly name: string,
    private readonly config: CircuitBreakerConfig,
  ) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    if (this.state === CircuitBreakerState.OPEN) {
      if (this.shouldAttemptReset()) {
        this.moveToHalfOpen();
      } else {
        throw new CircuitBreakerOpenError(`Circuit breaker ${this.name} is OPEN`);
      }
    }

    if (this.state === CircuitBreakerState.HALF_OPEN) {
      if (this.halfOpenCalls >= this.config.halfOpenMaxCalls) {
        throw new CircuitBreakerOpenError(
          `Circuit breaker ${this.name} is HALF_OPEN with max calls exceeded`,
        );
      }
    }

    return await this.callService(operation);
  }

  private async callService<T>(operation: () => Promise<T>): Promise<T> {
    const startTime = Date.now();
    this.stats.totalCalls++;

    if (this.state === CircuitBreakerState.HALF_OPEN) {
      this.halfOpenCalls++;
    }

    try {
      const result = await operation();
      this.onSuccess(Date.now() - startTime);
      return result;
    } catch (error) {
      this.onFailure(Date.now() - startTime);
      throw error;
    }
  }

  private onSuccess(responseTime: number): void {
    this.successCount++;
    this.stats.successfulCalls++;
    this.updateAverageResponseTime(responseTime);

    if (this.state === CircuitBreakerState.HALF_OPEN) {
      if (this.halfOpenCalls >= this.config.halfOpenMaxCalls) {
        this.reset();
      }
    } else {
      // Reset failure count on successful call
      this.failureCount = 0;
    }

    this.lastExecutionTime = new Date();
    this.updateFailureRate();
  }

  private onFailure(responseTime: number): void {
    this.failureCount++;
    this.stats.failedCalls++;
    this.lastFailureTime = new Date();
    this.lastExecutionTime = new Date();
    this.updateAverageResponseTime(responseTime);
    this.updateFailureRate();

    if (this.state === CircuitBreakerState.HALF_OPEN) {
      this.moveToOpen();
    } else if (this.shouldTrip()) {
      this.moveToOpen();
    }
  }

  private shouldTrip(): boolean {
    return (
      this.failureCount >= this.config.failureThreshold &&
      this.stats.totalCalls >= this.config.minimumThroughput
    );
  }

  private shouldAttemptReset(): boolean {
    if (!this.lastFailureTime) return false;

    const timeSinceLastFailure = Date.now() - this.lastFailureTime.getTime();
    return timeSinceLastFailure >= this.config.recoveryTimeoutMs;
  }

  private reset(): void {
    this.state = CircuitBreakerState.CLOSED;
    this.failureCount = 0;
    this.successCount = 0;
    this.halfOpenCalls = 0;
  }

  private moveToOpen(): void {
    this.state = CircuitBreakerState.OPEN;
    this.halfOpenCalls = 0;
  }

  private moveToHalfOpen(): void {
    this.state = CircuitBreakerState.HALF_OPEN;
    this.halfOpenCalls = 0;
  }

  private updateFailureRate(): void {
    if (this.stats.totalCalls === 0) {
      this.stats.failureRate = 0;
    } else {
      this.stats.failureRate = this.stats.failedCalls / this.stats.totalCalls;
    }
  }

  private updateAverageResponseTime(responseTime: number): void {
    if (this.stats.totalCalls === 1) {
      this.stats.averageResponseTime = responseTime;
    } else {
      this.stats.averageResponseTime =
        (this.stats.averageResponseTime * (this.stats.totalCalls - 1) + responseTime) /
        this.stats.totalCalls;
    }
  }

  // Getters for monitoring
  getState(): CircuitBreakerState {
    return this.state;
  }

  getStats(): CircuitBreakerStats {
    return { ...this.stats, lastFailureTime: this.lastFailureTime };
  }

  getName(): string {
    return this.name;
  }

  // Manual control methods
  forceOpen(): void {
    this.state = CircuitBreakerState.OPEN;
  }

  forceClose(): void {
    this.reset();
  }

  forceClear(): void {
    this.reset();
    this.stats = {
      totalCalls: 0,
      successfulCalls: 0,
      failedCalls: 0,
      failureRate: 0,
      averageResponseTime: 0,
      lastFailureTime: null,
    };
  }
}

// Circuit Breaker Registry
@Injectable()
export class CircuitBreakerRegistry {
  private circuitBreakers: Map<string, CircuitBreaker> = new Map();

  getOrCreate(name: string, config: CircuitBreakerConfig): CircuitBreaker {
    let circuitBreaker = this.circuitBreakers.get(name);

    if (!circuitBreaker) {
      circuitBreaker = new CircuitBreaker(name, config);
      this.circuitBreakers.set(name, circuitBreaker);
    }

    return circuitBreaker;
  }

  get(name: string): CircuitBreaker | undefined {
    return this.circuitBreakers.get(name);
  }

  getAll(): Map<string, CircuitBreaker> {
    return new Map(this.circuitBreakers);
  }

  remove(name: string): boolean {
    return this.circuitBreakers.delete(name);
  }
}

// Protected External Integration Service
@Injectable()
export class ExternalIntegrationService {
  private readonly erpCircuitBreaker: CircuitBreaker;
  private readonly notificationCircuitBreaker: CircuitBreaker;
  private readonly paymentCircuitBreaker: CircuitBreaker;

  constructor(
    private readonly httpService: HttpService,
    private readonly circuitBreakerRegistry: CircuitBreakerRegistry,
    @Inject('ERP_CONFIG') private readonly erpConfig: any,
  ) {
    // Initialize circuit breakers for different external services
    this.erpCircuitBreaker = this.circuitBreakerRegistry.getOrCreate('ERP_SERVICE', {
      failureThreshold: 5,
      recoveryTimeoutMs: 30000, // 30 seconds
      monitoringPeriodMs: 60000, // 1 minute
      halfOpenMaxCalls: 3,
      minimumThroughput: 10,
    });

    this.notificationCircuitBreaker = this.circuitBreakerRegistry.getOrCreate(
      'NOTIFICATION_SERVICE',
      {
        failureThreshold: 3,
        recoveryTimeoutMs: 10000, // 10 seconds
        monitoringPeriodMs: 30000,
        halfOpenMaxCalls: 2,
        minimumThroughput: 5,
      },
    );

    this.paymentCircuitBreaker = this.circuitBreakerRegistry.getOrCreate('PAYMENT_SERVICE', {
      failureThreshold: 3,
      recoveryTimeoutMs: 20000, // 20 seconds
      monitoringPeriodMs: 45000,
      halfOpenMaxCalls: 2,
      minimumThroughput: 5,
    });
  }

  async syncVendorToERP(vendor: Vendor): Promise<void> {
    try {
      await this.erpCircuitBreaker.execute(async () => {
        const response = await this.httpService
          .post('/api/vendors', {
            vendorId: vendor.getId().getValue(),
            name: vendor.getName().getValue(),
            code: vendor.getCode().getValue(),
          })
          .toPromise();

        if (response.status !== 200) {
          throw new Error('ERP sync failed');
        }

        return response.data;
      });
    } catch (error) {
      if (error instanceof CircuitBreakerOpenError) {
        // Circuit breaker is open - use fallback or queue for later
        await this.queueVendorSyncForLater(vendor);
        console.warn('ERP circuit breaker is open, queued vendor sync for later', {
          vendorId: vendor.getId().getValue(),
        });
        return; // Don't throw - handle gracefully
      }

      // Re-throw other errors
      throw error;
    }
  }

  async updatePaymentStatus(paymentId: string, status: string): Promise<void> {
    try {
      await this.paymentCircuitBreaker.execute(async () => {
        const response = await this.httpService
          .put(`/api/payments/${paymentId}`, {
            status: status,
            updatedAt: new Date().toISOString(),
          })
          .toPromise();

        return response.data;
      });
    } catch (error) {
      if (error instanceof CircuitBreakerOpenError) {
        // Store for retry later
        await this.storePaymentUpdateForRetry(paymentId, status);
        console.warn('Payment service circuit breaker is open, stored update for retry');
        return;
      }

      throw error;
    }
  }

  async sendNotification(notification: NotificationData): Promise<void> {
    try {
      await this.notificationCircuitBreaker.execute(async () => {
        const response = await this.httpService
          .post('/api/notifications', notification)
          .toPromise();
        return response.data;
      });
    } catch (error) {
      if (error instanceof CircuitBreakerOpenError) {
        // Use fallback notification method or queue
        await this.useFallbackNotification(notification);
        console.warn('Notification service circuit breaker is open, used fallback');
        return;
      }

      throw error;
    }
  }

  // Fallback methods
  private async queueVendorSyncForLater(vendor: Vendor): Promise<void> {
    // Implementation to queue vendor sync for later processing
    // Could use database queue, Redis, or message queue
  }

  private async storePaymentUpdateForRetry(paymentId: string, status: string): Promise<void> {
    // Implementation to store payment updates for retry
  }

  private async useFallbackNotification(notification: NotificationData): Promise<void> {
    // Implementation of fallback notification mechanism
    // Could be email, SMS, or internal queue
  }

  // Monitoring methods
  getCircuitBreakerStats() {
    return {
      erp: this.erpCircuitBreaker.getStats(),
      notifications: this.notificationCircuitBreaker.getStats(),
      payments: this.paymentCircuitBreaker.getStats(),
    };
  }
}

// Circuit Breaker Error
export class CircuitBreakerOpenError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'CircuitBreakerOpenError';
  }
}
```

## Advanced Circuit Breaker Patterns

### Adaptive Circuit Breaker

```typescript
// Adaptive circuit breaker that adjusts thresholds based on historical data
export class AdaptiveCircuitBreaker extends CircuitBreaker {
  private historicalFailureRates: number[] = [];
  private adaptiveConfig: CircuitBreakerConfig;

  constructor(name: string, baseConfig: CircuitBreakerConfig) {
    super(name, baseConfig);
    this.adaptiveConfig = { ...baseConfig };
  }

  protected onFailure(responseTime: number): void {
    super.onFailure(responseTime);
    this.updateAdaptiveThresholds();
  }

  private updateAdaptiveThresholds(): void {
    // Keep last 100 failure rates
    this.historicalFailureRates.push(this.getStats().failureRate);
    if (this.historicalFailureRates.length > 100) {
      this.historicalFailureRates.shift();
    }

    // Calculate average historical failure rate
    const avgHistoricalFailureRate =
      this.historicalFailureRates.reduce((sum, rate) => sum + rate, 0) /
      this.historicalFailureRates.length;

    // Adjust failure threshold based on historical data
    if (avgHistoricalFailureRate > 0.2) {
      // High historical failure rate - be more sensitive
      this.adaptiveConfig.failureThreshold = Math.max(2, this.adaptiveConfig.failureThreshold - 1);
    } else if (avgHistoricalFailureRate < 0.05) {
      // Low historical failure rate - be less sensitive
      this.adaptiveConfig.failureThreshold = Math.min(10, this.adaptiveConfig.failureThreshold + 1);
    }
  }

  protected shouldTrip(): boolean {
    return (
      this.getFailureCount() >= this.adaptiveConfig.failureThreshold &&
      this.getStats().totalCalls >= this.adaptiveConfig.minimumThroughput
    );
  }
}
```

### Circuit Breaker with Bulkhead

```typescript
// Bulkhead pattern implementation with circuit breakers
@Injectable()
export class BulkheadService {
  private readonly vendorOperationsCircuitBreaker: CircuitBreaker;
  private readonly paymentOperationsCircuitBreaker: CircuitBreaker;
  private readonly reportingOperationsCircuitBreaker: CircuitBreaker;

  constructor(circuitBreakerRegistry: CircuitBreakerRegistry) {
    // Separate circuit breakers for different operation types
    this.vendorOperationsCircuitBreaker = circuitBreakerRegistry.getOrCreate('VENDOR_OPERATIONS', {
      failureThreshold: 5,
      recoveryTimeoutMs: 30000,
      monitoringPeriodMs: 60000,
      halfOpenMaxCalls: 3,
      minimumThroughput: 10,
    });

    this.paymentOperationsCircuitBreaker = circuitBreakerRegistry.getOrCreate(
      'PAYMENT_OPERATIONS',
      {
        failureThreshold: 3,
        recoveryTimeoutMs: 15000,
        monitoringPeriodMs: 30000,
        halfOpenMaxCalls: 2,
        minimumThroughput: 5,
      },
    );

    this.reportingOperationsCircuitBreaker = circuitBreakerRegistry.getOrCreate(
      'REPORTING_OPERATIONS',
      {
        failureThreshold: 10, // More tolerant for reporting
        recoveryTimeoutMs: 60000,
        monitoringPeriodMs: 120000,
        halfOpenMaxCalls: 5,
        minimumThroughput: 20,
      },
    );
  }

  async executeVendorOperation<T>(operation: () => Promise<T>): Promise<T> {
    return await this.vendorOperationsCircuitBreaker.execute(operation);
  }

  async executePaymentOperation<T>(operation: () => Promise<T>): Promise<T> {
    return await this.paymentOperationsCircuitBreaker.execute(operation);
  }

  async executeReportingOperation<T>(operation: () => Promise<T>): Promise<T> {
    return await this.reportingOperationsCircuitBreaker.execute(operation);
  }
}

// Usage in different services
@Injectable()
export class VendorService {
  constructor(private readonly bulkheadService: BulkheadService) {}

  async activateVendor(vendorId: string): Promise<void> {
    await this.bulkheadService.executeVendorOperation(async () => {
      // Vendor activation logic
      const vendor = await this.vendorRepository.findById(vendorId);
      vendor.activate();
      await this.vendorRepository.save(vendor);
    });
  }
}

@Injectable()
export class PaymentService {
  constructor(private readonly bulkheadService: BulkheadService) {}

  async processPayment(paymentId: string): Promise<void> {
    await this.bulkheadService.executePaymentOperation(async () => {
      // Payment processing logic
      const payment = await this.paymentRepository.findById(paymentId);
      await this.processPaymentWithGateway(payment);
    });
  }
}
```

### Circuit Breaker Decorator

```typescript
// Decorator for automatic circuit breaker application
export function CircuitBreaker(config: CircuitBreakerConfig) {
  return function (target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const method = descriptor.value;
    const circuitBreakerName = `${target.constructor.name}_${propertyName}`;

    descriptor.value = async function (...args: any[]) {
      const circuitBreakerRegistry: CircuitBreakerRegistry =
        this.circuitBreakerRegistry || new CircuitBreakerRegistry();

      const circuitBreaker = circuitBreakerRegistry.getOrCreate(circuitBreakerName, config);

      return await circuitBreaker.execute(async () => {
        return await method.comly(this, args);
      });
    };

    return descriptor;
  };
}

// Usage with decorator
@Injectable()
export class ExternalApiService {
  constructor(
    private readonly httpService: HttpService,
    private readonly circuitBreakerRegistry: CircuitBreakerRegistry,
  ) {}

  @CircuitBreaker({
    failureThreshold: 3,
    recoveryTimeoutMs: 10000,
    monitoringPeriodMs: 30000,
    halfOpenMaxCalls: 2,
    minimumThroughput: 5,
  })
  async callExternalApi(data: any): Promise<any> {
    const response = await this.httpService.post('/external-api', data).toPromise();
    return response.data;
  }

  @CircuitBreaker({
    failureThreshold: 5,
    recoveryTimeoutMs: 30000,
    monitoringPeriodMs: 60000,
    halfOpenMaxCalls: 3,
    minimumThroughput: 10,
  })
  async syncData(syncData: any): Promise<void> {
    await this.httpService.put('/sync', syncData).toPromise();
  }
}
```

## Circuit Breaker Monitoring and Alerting

### Health Check Integration

```typescript
@Injectable()
export class CircuitBreakerHealthIndicator extends HealthIndicator {
  constructor(private readonly circuitBreakerRegistry: CircuitBreakerRegistry) {
    super();
  }

  async isHealthy(key: string): Promise<HealthIndicatorResult> {
    const circuitBreakers = this.circuitBreakerRegistry.getAll();
    const unhealthyBreakers: string[] = [];

    for (const [name, breaker] of circuitBreakers) {
      if (breaker.getState() === CircuitBreakerState.OPEN) {
        unhealthyBreakers.push(name);
      }
    }

    const isHealthy = unhealthyBreakers.length === 0;

    const result = this.getStatus(key, isHealthy, {
      openCircuitBreakers: unhealthyBreakers,
      totalCircuitBreakers: circuitBreakers.size,
      circuitBreakerStats: Object.fromEntries(
        Array.from(circuitBreakers.entries()).map(([name, breaker]) => [
          name,
          {
            state: breaker.getState(),
            failureRate: breaker.getStats().failureRate,
            totalCalls: breaker.getStats().totalCalls,
          },
        ]),
      ),
    });

    if (!isHealthy) {
      throw new HealthCheckError('Circuit breakers are open', result);
    }

    return result;
  }
}

// Circuit breaker metrics
@Injectable()
export class CircuitBreakerMetricsService {
  constructor(
    private readonly circuitBreakerRegistry: CircuitBreakerRegistry,
    private readonly metricsService: IMetricsService,
  ) {}

  @Cron('0 * * * * *') // Every minute
  async collectMetrics(): Promise<void> {
    const circuitBreakers = this.circuitBreakerRegistry.getAll();

    for (const [name, breaker] of circuitBreakers) {
      const stats = breaker.getStats();

      // Emit metrics
      this.metricsService.gauge(`circuit_breaker.failure_rate.${name}`, stats.failureRate);
      this.metricsService.gauge(`circuit_breaker.total_calls.${name}`, stats.totalCalls);
      this.metricsService.gauge(`circuit_breaker.successful_calls.${name}`, stats.successfulCalls);
      this.metricsService.gauge(`circuit_breaker.failed_calls.${name}`, stats.failedCalls);
      this.metricsService.gauge(
        `circuit_breaker.average_response_time.${name}`,
        stats.averageResponseTime,
      );

      // State as numeric value for monitoring
      const stateValue =
        breaker.getState() === CircuitBreakerState.CLOSED
          ? 0
          : breaker.getState() === CircuitBreakerState.HALF_OPEN
            ? 1
            : 2;
      this.metricsService.gauge(`circuit_breaker.state.${name}`, stateValue);
    }
  }
}
```

## Testing Circuit Breaker

### Circuit Breaker Testing

```typescript
describe('CircuitBreaker', () => {
  let circuitBreaker: CircuitBreaker;
  const config: CircuitBreakerConfig = {
    failureThreshold: 3,
    recoveryTimeoutMs: 1000,
    monitoringPeriodMs: 5000,
    halfOpenMaxCalls: 2,
    minimumThroughput: 3,
  };

  beforeEach(() => {
    circuitBreaker = new CircuitBreaker('test-breaker', config);
  });

  describe('CLOSED state', () => {
    it('should execute operation successfully', async () => {
      const operation = jest.fn().mockResolvedValue('success');

      const result = await circuitBreaker.execute(operation);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(1);
      expect(circuitBreaker.getState()).toBe(CircuitBreakerState.CLOSED);
    });

    it('should move to OPEN after failure threshold', async () => {
      const operation = jest.fn().mockRejectedValue(new Error('Service down'));

      // Execute enough times to exceed failure threshold
      for (let i = 0; i < config.failureThreshold; i++) {
        try {
          await circuitBreaker.execute(operation);
        } catch (error) {
          // Expected failures
        }
      }

      expect(circuitBreaker.getState()).toBe(CircuitBreakerState.OPEN);
    });
  });

  describe('OPEN state', () => {
    beforeEach(async () => {
      // Force circuit breaker to OPEN state
      const failingOperation = jest.fn().mockRejectedValue(new Error('Service down'));

      for (let i = 0; i < config.failureThreshold; i++) {
        try {
          await circuitBreaker.execute(failingOperation);
        } catch (error) {
          // Expected failures
        }
      }
    });

    it('should reject calls immediately', async () => {
      const operation = jest.fn().mockResolvedValue('success');

      await expect(circuitBreaker.execute(operation)).rejects.toThrow(CircuitBreakerOpenError);

      expect(operation).not.toHaveBeenCalled();
    });

    it('should move to HALF_OPEN after recovery timeout', async () => {
      // Wait for recovery timeout
      await new Promise((resolve) => setTimeout(resolve, config.recoveryTimeoutMs + 100));

      const operation = jest.fn().mockResolvedValue('success');

      // Should move to HALF_OPEN and allow the call
      const result = await circuitBreaker.execute(operation);

      expect(result).toBe('success');
      expect(circuitBreaker.getState()).toBe(CircuitBreakerState.HALF_OPEN);
    });
  });

  describe('HALF_OPEN state', () => {
    beforeEach(async () => {
      // Move to OPEN state first
      const failingOperation = jest.fn().mockRejectedValue(new Error('Service down'));
      for (let i = 0; i < config.failureThreshold; i++) {
        try {
          await circuitBreaker.execute(failingOperation);
        } catch (error) {
          // Expected failures
        }
      }

      // Wait for recovery timeout to move to HALF_OPEN
      await new Promise((resolve) => setTimeout(resolve, config.recoveryTimeoutMs + 100));
    });

    it('should close on successful calls', async () => {
      const operation = jest.fn().mockResolvedValue('success');

      // Execute successful operations up to half open max calls
      for (let i = 0; i < config.halfOpenMaxCalls; i++) {
        await circuitBreaker.execute(operation);
      }

      expect(circuitBreaker.getState()).toBe(CircuitBreakerState.CLOSED);
    });

    it('should open again on failure', async () => {
      const operation = jest.fn().mockRejectedValue(new Error('Still failing'));

      try {
        await circuitBreaker.execute(operation);
      } catch (error) {
        // Expected failure
      }

      expect(circuitBreaker.getState()).toBe(CircuitBreakerState.OPEN);
    });
  });

  describe('statistics', () => {
    it('should track call statistics correctly', async () => {
      const successOperation = jest.fn().mockResolvedValue('success');
      const failOperation = jest.fn().mockRejectedValue(new Error('fail'));

      // Execute some successful operations
      await circuitBreaker.execute(successOperation);
      await circuitBreaker.execute(successOperation);

      // Execute some failed operations
      try {
        await circuitBreaker.execute(failOperation);
      } catch (error) {
        // Expected failure
      }

      const stats = circuitBreaker.getStats();

      expect(stats.totalCalls).toBe(3);
      expect(stats.successfulCalls).toBe(2);
      expect(stats.failedCalls).toBe(1);
      expect(stats.failureRate).toBeCloseTo(0.333, 2);
    });
  });
});
```

## Best Practices

### 1. Service-Specific Configuration

```typescript
// Good: Different configurations for different services
const configs = {
  ERP_SERVICE: {
    failureThreshold: 5, // ERP is critical, allow more failures
    recoveryTimeoutMs: 60000, // Longer recovery time
    minimumThroughput: 20,
  },
  NOTIFICATION_SERVICE: {
    failureThreshold: 2, // Notifications should fail fast
    recoveryTimeoutMs: 10000, // Quick recovery
    minimumThroughput: 5,
  },
};
```

### 2. Graceful Degradation

```typescript
// Good: Provide fallbacks when circuit breaker is open
async function sendNotification(notification: NotificationData): Promise<void> {
  try {
    await circuitBreaker.execute(() => externalNotificationService.send(notification));
  } catch (error) {
    if (error instanceof CircuitBreakerOpenError) {
      // Use fallback mechanism
      await fallbackNotificationService.send(notification);
      return;
    }
    throw error;
  }
}
```

### 3. Monitoring and Alerting

```typescript
// Good: Monitor circuit breaker state changes
class CircuitBreakerMonitor {
  @OnEvent('circuit-breaker.state-changed')
  async handleStateChange(event: {
    name: string;
    oldState: string;
    newState: string;
  }): Promise<void> {
    if (event.newState === CircuitBreakerState.OPEN) {
      await this.alertingService.sendAlert({
        severity: 'WARNING',
        message: `Circuit breaker ${event.name} is now OPEN`,
        tags: ['circuit-breaker', 'service-degradation'],
      });
    }
  }
}
```

The Circuit Breaker pattern in our oil & gas management system provides fault
tolerance and system stability by preventing cascading failures when external
services or internal components become unavailable, while supporting automatic
recovery and graceful degradation.
