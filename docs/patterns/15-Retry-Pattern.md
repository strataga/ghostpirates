# Retry Pattern

## Overview

The Retry pattern handles transient failures by automatically retrying failed
operations with configurable strategies. It's essential for building resilient
systems that can recover from temporary network issues, service unavailability,
or resource contention without manual intervention.

## Core Concepts

### Transient Failures

Temporary failures that might succeed if retried (network timeouts, temporary
service unavailability, resource contention).

### Retry Strategy

The algorithm that determines when and how many times to retry (linear,
exponential backoff, fixed interval).

### Backoff Policy

The waiting strategy between retry attempts to avoid overwhelming the failing
service.

### Retry Conditions

Logic to determine which failures should trigger retries and which should fail
immediately.

## Benefits

- **Resilience**: Handles transient failures automatically
- **User Experience**: Reduces apparent failures from temporary issues
- **System Stability**: Prevents cascade failures from temporary problems
- **Availability**: Improves overall system availability
- **Resource Efficiency**: Intelligently spaces retry attempts
- **Configurability**: Allows fine-tuning for different scenarios

## Implementation in Our Project

### Before: No Retry Logic

```typescript
@Injectable()
export class ExternalApiService {
  constructor(private readonly httpService: HttpService) {}

  async syncVendorData(vendor: Vendor): Promise<void> {
    // No retry logic - fails on first attempt
    try {
      await this.httpService
        .post('/api/vendor-sync', {
          vendorId: vendor.getId(),
          data: vendor.toSyncData(),
        })
        .toPromise();
    } catch (error) {
      // Single failure causes complete operation failure
      console.error('Vendor sync failed', error);
      throw error;
    }
  }

  async updatePaymentStatus(paymentId: string, status: string): Promise<void> {
    // No protection against temporary failures
    await this.httpService
      .put(`/api/payments/${paymentId}/status`, {
        status: status,
        timestamp: new Date().toISOString(),
      })
      .toPromise();
  }

  async sendNotification(notification: NotificationData): Promise<void> {
    // Fails immediately on network issues
    await this.httpService.post('/api/notifications', notification).toPromise();
  }
}

@Injectable()
export class DatabaseOperationService {
  constructor(private readonly database: Database) {}

  async saveVendorBatch(vendors: Vendor[]): Promise<void> {
    // No retry for database deadlocks or connection issues
    const transaction = await this.database.beginTransaction();

    try {
      for (const vendor of vendors) {
        await this.database.insert('vendors', vendor.toDbData());
      }
      await transaction.commit();
    } catch (error) {
      await transaction.rollback();
      throw error; // Fails immediately
    }
  }
}
```

### After: Retry Pattern Implementation

```typescript
// Retry Configuration
export interface RetryConfig {
  maxAttempts: number;
  baseDelayMs: number;
  maxDelayMs: number;
  strategy: RetryStrategy;
  retryCondition: (error: Error) => boolean;
  onRetry?: (attempt: number, error: Error) => void;
  jitter?: boolean;
}

export enum RetryStrategy {
  FIXED_DELAY = 'FIXED_DELAY',
  LINEAR_BACKOFF = 'LINEAR_BACKOFF',
  EXPONENTIAL_BACKOFF = 'EXPONENTIAL_BACKOFF',
  EXPONENTIAL_BACKOFF_FULL_JITTER = 'EXPONENTIAL_BACKOFF_FULL_JITTER',
}

// Retry Implementation
export class RetryExecutor {
  constructor(private readonly config: RetryConfig) {}

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    let lastError: Error;
    let attempt = 0;

    while (attempt < this.config.maxAttempts) {
      attempt++;

      try {
        const result = await operation();

        // Log successful retry if it wasn't the first attempt
        if (attempt > 1) {
          console.info('Operation succeeded after retry', {
            attempts: attempt,
            totalTime: Date.now(),
          });
        }

        return result;
      } catch (error) {
        lastError = error;

        // Check if this error should trigger a retry
        if (!this.config.retryCondition(error)) {
          console.warn('Operation failed with non-retryable error', {
            attempt,
            error: error.message,
          });
          throw error;
        }

        // If this was the last attempt, throw the error
        if (attempt >= this.config.maxAttempts) {
          console.error('Operation failed after all retry attempts', {
            maxAttempts: this.config.maxAttempts,
            error: error.message,
          });
          throw new MaxRetriesExceededError(
            `Operation failed after ${this.config.maxAttempts} attempts: ${error.message}`,
            error,
            attempt,
          );
        }

        // Call retry callback if provided
        if (this.config.onRetry) {
          this.config.onRetry(attempt, error);
        }

        // Wait before next attempt
        const delayMs = this.calculateDelay(attempt);

        console.warn('Operation failed, retrying', {
          attempt,
          maxAttempts: this.config.maxAttempts,
          delayMs,
          error: error.message,
        });

        await this.delay(delayMs);
      }
    }

    throw lastError;
  }

  private calculateDelay(attempt: number): number {
    let delay: number;

    switch (this.config.strategy) {
      case RetryStrategy.FIXED_DELAY:
        delay = this.config.baseDelayMs;
        break;

      case RetryStrategy.LINEAR_BACKOFF:
        delay = this.config.baseDelayMs * attempt;
        break;

      case RetryStrategy.EXPONENTIAL_BACKOFF:
        delay = this.config.baseDelayMs * Math.pow(2, attempt - 1);
        break;

      case RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER:
        const exponentialDelay = this.config.baseDelayMs * Math.pow(2, attempt - 1);
        delay = Math.random() * exponentialDelay;
        break;

      default:
        delay = this.config.baseDelayMs;
    }

    // Apply jitter if enabled (for non-full-jitter strategies)
    if (
      this.config.jitter &&
      this.config.strategy !== RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER
    ) {
      const jitterRange = delay * 0.1; // 10% jitter
      delay = delay + Math.random() * jitterRange * 2 - jitterRange;
    }

    // Ensure delay doesn't exceed maximum
    return Math.min(delay, this.config.maxDelayMs);
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

// Retry Factory
@Injectable()
export class RetryFactory {
  createHttpRetry(): RetryExecutor {
    return new RetryExecutor({
      maxAttempts: 3,
      baseDelayMs: 1000, // 1 second
      maxDelayMs: 30000, // 30 seconds max
      strategy: RetryStrategy.EXPONENTIAL_BACKOFF,
      jitter: true,
      retryCondition: (error: Error) => this.isRetryableHttpError(error),
      onRetry: (attempt, error) => {
        console.warn(`HTTP operation failed, attempt ${attempt}`, {
          error: error.message,
          type: error.constructor.name,
        });
      },
    });
  }

  createDatabaseRetry(): RetryExecutor {
    return new RetryExecutor({
      maxAttempts: 5,
      baseDelayMs: 500, // 500ms
      maxDelayMs: 10000, // 10 seconds max
      strategy: RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER,
      retryCondition: (error: Error) => this.isRetryableDatabaseError(error),
      onRetry: (attempt, error) => {
        console.warn(`Database operation failed, attempt ${attempt}`, {
          error: error.message,
          type: error.constructor.name,
        });
      },
    });
  }

  createExternalServiceRetry(serviceName: string): RetryExecutor {
    return new RetryExecutor({
      maxAttempts: 4,
      baseDelayMs: 2000, // 2 seconds
      maxDelayMs: 60000, // 1 minute max
      strategy: RetryStrategy.EXPONENTIAL_BACKOFF,
      jitter: true,
      retryCondition: (error: Error) => this.isRetryableServiceError(error),
      onRetry: (attempt, error) => {
        console.warn(`External service ${serviceName} failed, attempt ${attempt}`, {
          service: serviceName,
          error: error.message,
          type: error.constructor.name,
        });
      },
    });
  }

  private isRetryableHttpError(error: Error): boolean {
    // Retry on network errors, timeouts, and server errors
    const retryableStatuses = [408, 429, 500, 502, 503, 504];
    const retryableErrorTypes = ['ECONNRESET', 'ENOTFOUND', 'ECONNREFUSED', 'ETIMEDOUT'];

    // Check HTTP status codes
    if ('status' in error) {
      return retryableStatuses.includes((error as any).status);
    }

    // Check network error codes
    if ('code' in error) {
      return retryableErrorTypes.includes((error as any).code);
    }

    // Check error messages for common transient issues
    const errorMessage = error.message.toLowerCase();
    const retryableMessages = [
      'timeout',
      'connection reset',
      'connection refused',
      'network error',
      'service unavailable',
    ];

    return retryableMessages.some((msg) => errorMessage.includes(msg));
  }

  private isRetryableDatabaseError(error: Error): boolean {
    const errorMessage = error.message.toLowerCase();
    const retryableMessages = [
      'deadlock',
      'connection lost',
      'connection timeout',
      'lock wait timeout',
      'temporary failure',
      'resource temporarily unavailable',
    ];

    return retryableMessages.some((msg) => errorMessage.includes(msg));
  }

  private isRetryableServiceError(error: Error): boolean {
    // Combine HTTP and service-specific retry conditions
    return this.isRetryableHttpError(error) || this.isRetryableDatabaseError(error);
  }
}

// Enhanced External API Service with Retry
@Injectable()
export class ExternalApiService {
  private readonly httpRetry: RetryExecutor;
  private readonly erpRetry: RetryExecutor;
  private readonly bankingRetry: RetryExecutor;

  constructor(
    private readonly httpService: HttpService,
    private readonly retryFactory: RetryFactory,
  ) {
    this.httpRetry = this.retryFactory.createHttpRetry();
    this.erpRetry = this.retryFactory.createExternalServiceRetry('ERP');
    this.bankingRetry = this.retryFactory.createExternalServiceRetry('Banking');
  }

  async syncVendorData(vendor: Vendor): Promise<void> {
    await this.httpRetry.execute(async () => {
      const response = await this.httpService
        .post('/api/vendor-sync', {
          vendorId: vendor.getId(),
          data: vendor.toSyncData(),
        })
        .toPromise();

      if (response.status !== 200) {
        throw new Error(`Vendor sync failed with status: ${response.status}`);
      }

      return response.data;
    });
  }

  async updatePaymentStatus(paymentId: string, status: string): Promise<void> {
    await this.httpRetry.execute(async () => {
      await this.httpService
        .put(`/api/payments/${paymentId}/status`, {
          status: status,
          timestamp: new Date().toISOString(),
        })
        .toPromise();
    });
  }

  async sendNotification(notification: NotificationData): Promise<void> {
    await this.httpRetry.execute(async () => {
      await this.httpService.post('/api/notifications', notification).toPromise();
    });
  }

  async syncWithErp(data: any): Promise<void> {
    await this.erpRetry.execute(async () => {
      const response = await this.httpService.post('/api/erp/sync', data).toPromise();

      if (response.status >= 400) {
        throw new Error(`ERP sync failed: ${response.statusText}`);
      }

      return response.data;
    });
  }

  async processPayment(paymentData: any): Promise<any> {
    return await this.bankingRetry.execute(async () => {
      const response = await this.httpService
        .post('/api/banking/payments', paymentData)
        .toPromise();

      if (!response.data.success) {
        throw new Error(`Payment processing failed: ${response.data.error}`);
      }

      return response.data;
    });
  }
}

// Database Service with Retry
@Injectable()
export class DatabaseOperationService {
  private readonly databaseRetry: RetryExecutor;

  constructor(
    private readonly database: Database,
    private readonly retryFactory: RetryFactory,
  ) {
    this.databaseRetry = this.retryFactory.createDatabaseRetry();
  }

  async saveVendorBatch(vendors: Vendor[]): Promise<void> {
    await this.databaseRetry.execute(async () => {
      const transaction = await this.database.beginTransaction();

      try {
        for (const vendor of vendors) {
          await this.database.insert('vendors', vendor.toDbData());
        }
        await transaction.commit();
      } catch (error) {
        await transaction.rollback();
        throw error;
      }
    });
  }

  async updateVendorStatus(vendorId: string, status: string): Promise<void> {
    await this.databaseRetry.execute(async () => {
      const result = await this.database.update(
        'vendors',
        { status, updatedAt: new Date() },
        { id: vendorId },
      );

      if (result.affectedRows === 0) {
        throw new Error('Vendor not found or no changes made');
      }
    });
  }
}

// Custom Error for Max Retries
export class MaxRetriesExceededError extends Error {
  constructor(
    message: string,
    public readonly originalError: Error,
    public readonly attemptCount: number,
  ) {
    super(message);
    this.name = 'MaxRetriesExceededError';
  }
}
```

## Advanced Retry Patterns

### Retry with Circuit Breaker Integration

```typescript
@Injectable()
export class RetryWithCircuitBreakerService {
  constructor(
    private readonly retryFactory: RetryFactory,
    private readonly circuitBreakerRegistry: CircuitBreakerRegistry,
  ) {}

  async executeWithRetryAndCircuitBreaker<T>(
    operationName: string,
    operation: () => Promise<T>,
  ): Promise<T> {
    const circuitBreaker = this.circuitBreakerRegistry.getOrCreate(operationName, {
      failureThreshold: 5,
      recoveryTimeoutMs: 30000,
      monitoringPeriodMs: 60000,
      halfOpenMaxCalls: 3,
      minimumThroughput: 10,
    });

    const retry = this.retryFactory.createHttpRetry();

    // Wrap operation with circuit breaker
    const protectedOperation = async (): Promise<T> => {
      return await circuitBreaker.execute(operation);
    };

    // Apply retry to the protected operation
    return await retry.execute(protectedOperation);
  }
}

// Usage
@Injectable()
export class PaymentService {
  constructor(private readonly retryCircuitBreakerService: RetryWithCircuitBreakerService) {}

  async processPayment(payment: Payment): Promise<PaymentResult> {
    return await this.retryCircuitBreakerService.executeWithRetryAndCircuitBreaker(
      'payment-processing',
      async () => {
        // Payment processing logic
        const result = await this.paymentGateway.processPayment(payment);
        if (!result.success) {
          throw new PaymentProcessingError(result.error);
        }
        return result;
      },
    );
  }
}
```

### Retry Decorator

```typescript
// Decorator for automatic retry application
export function Retry(config?: Partial<RetryConfig>) {
  return function (target: any, propertyName: string, descriptor: PropertyDescriptor) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      const retryConfig: RetryConfig = {
        maxAttempts: 3,
        baseDelayMs: 1000,
        maxDelayMs: 30000,
        strategy: RetryStrategy.EXPONENTIAL_BACKOFF,
        jitter: true,
        retryCondition: (error: Error) => {
          // Default retry condition
          return (
            error.message.includes('timeout') ||
            error.message.includes('connection') ||
            error.message.includes('network')
          );
        },
        ...config,
      };

      const retryExecutor = new RetryExecutor(retryConfig);

      return await retryExecutor.execute(async () => {
        return await originalMethod.comly(this, args);
      });
    };

    return descriptor;
  };
}

// Usage with decorator
@Injectable()
export class ApiService {
  @Retry({
    maxAttempts: 5,
    strategy: RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER,
    baseDelayMs: 2000,
  })
  async callExternalApi(data: any): Promise<any> {
    const response = await this.httpService.post('/external-api', data).toPromise();
    return response.data;
  }

  @Retry({
    maxAttempts: 3,
    strategy: RetryStrategy.LINEAR_BACKOFF,
    retryCondition: (error: Error) => error.message.includes('rate limit'),
  })
  async callRateLimitedApi(data: any): Promise<any> {
    const response = await this.httpService.get('/rate-limited-api', { params: data }).toPromise();
    return response.data;
  }
}
```

### Bulk Operation Retry

```typescript
// Retry pattern for bulk operations with partial failure handling
export class BulkRetryExecutor<TItem, TResult> {
  constructor(
    private readonly retryConfig: RetryConfig,
    private readonly batchSize: number = 10,
  ) {}

  async executeBulk(
    items: TItem[],
    operation: (item: TItem) => Promise<TResult>,
  ): Promise<BulkOperationResult<TItem, TResult>> {
    const results: TResult[] = [];
    const failures: BulkOperationFailure<TItem>[] = [];

    // Process in batches
    for (let i = 0; i < items.length; i += this.batchSize) {
      const batch = items.slice(i, i + this.batchSize);
      const batchResults = await this.processBatch(batch, operation);

      results.push(...batchResults.successes);
      failures.push(...batchResults.failures);
    }

    return new BulkOperationResult(results, failures);
  }

  private async processBatch(
    items: TItem[],
    operation: (item: TItem) => Promise<TResult>,
  ): Promise<{
    successes: TResult[];
    failures: BulkOperationFailure<TItem>[];
  }> {
    const promises = items.map(async (item, index) => {
      const retryExecutor = new RetryExecutor(this.retryConfig);

      try {
        const result = await retryExecutor.execute(() => operation(item));
        return { success: true, result, item, index };
      } catch (error) {
        return { success: false, error, item, index };
      }
    });

    const outcomes = await Promise.allSettled(promises);
    const successes: TResult[] = [];
    const failures: BulkOperationFailure<TItem>[] = [];

    outcomes.forEach((outcome, index) => {
      if (outcome.status === 'fulfilled') {
        const result = outcome.value;
        if (result.success) {
          successes.push(result.result);
        } else {
          failures.push(new BulkOperationFailure(result.item, result.error, index));
        }
      } else {
        failures.push(new BulkOperationFailure(items[index], outcome.reason, index));
      }
    });

    return { successes, failures };
  }
}

// Usage for bulk vendor sync
@Injectable()
export class VendorSyncService {
  private readonly bulkRetry: BulkRetryExecutor<Vendor, SyncResult>;

  constructor(private readonly externalApiService: ExternalApiService) {
    this.bulkRetry = new BulkRetryExecutor(
      {
        maxAttempts: 3,
        baseDelayMs: 1000,
        maxDelayMs: 10000,
        strategy: RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER,
        retryCondition: (error: Error) => {
          return !error.message.includes('validation') && !error.message.includes('duplicate');
        },
      },
      20,
    ); // Process 20 vendors at a time
  }

  async syncVendors(vendors: Vendor[]): Promise<VendorSyncReport> {
    const result = await this.bulkRetry.executeBulk(vendors, async (vendor) => {
      return await this.externalApiService.syncVendorData(vendor);
    });

    return new VendorSyncReport(
      result.successes.length,
      result.failures.length,
      result.failures.map((f) => ({
        vendorId: f.item.getId().getValue(),
        error: f.error.message,
      })),
    );
  }
}
```

## Retry Monitoring and Metrics

### Retry Metrics Collection

```typescript
@Injectable()
export class RetryMetricsService {
  constructor(private readonly metricsService: IMetricsService) {}

  recordRetryAttempt(operationName: string, attempt: number, success: boolean): void {
    this.metricsService.increment('retry.attempts.total', {
      operation: operationName,
      attempt: attempt.toString(),
      success: success.toString(),
    });

    if (success && attempt > 1) {
      this.metricsService.increment('retry.success_after_retry.total', {
        operation: operationName,
        attempts: attempt.toString(),
      });
    }

    if (!success && attempt === 1) {
      this.metricsService.increment('retry.first_attempt_failures.total', {
        operation: operationName,
      });
    }
  }

  recordRetryFailure(operationName: string, totalAttempts: number, finalError: string): void {
    this.metricsService.increment('retry.max_attempts_exceeded.total', {
      operation: operationName,
      max_attempts: totalAttempts.toString(),
      error_type: finalError,
    });
  }

  recordRetryDelay(operationName: string, delayMs: number): void {
    this.metricsService.histogram('retry.delay_ms', delayMs, {
      operation: operationName,
    });
  }
}

// Enhanced Retry Executor with Metrics
export class MetricsEnabledRetryExecutor extends RetryExecutor {
  constructor(
    config: RetryConfig,
    private readonly metricsService: RetryMetricsService,
    private readonly operationName: string,
  ) {
    super(config);
  }

  async execute<T>(operation: () => Promise<T>): Promise<T> {
    let attempt = 0;
    let lastError: Error;

    while (attempt < this.config.maxAttempts) {
      attempt++;

      try {
        const result = await operation();

        // Record successful attempt
        this.metricsService.recordRetryAttempt(this.operationName, attempt, true);

        return result;
      } catch (error) {
        lastError = error;

        // Record failed attempt
        this.metricsService.recordRetryAttempt(this.operationName, attempt, false);

        if (!this.config.retryCondition(error) || attempt >= this.config.maxAttempts) {
          if (attempt >= this.config.maxAttempts) {
            this.metricsService.recordRetryFailure(
              this.operationName,
              attempt,
              error.constructor.name,
            );
          }
          throw error;
        }

        const delayMs = this.calculateDelay(attempt);
        this.metricsService.recordRetryDelay(this.operationName, delayMs);

        if (this.config.onRetry) {
          this.config.onRetry(attempt, error);
        }

        await this.delay(delayMs);
      }
    }

    throw lastError;
  }
}
```

## Testing Retry Pattern

### Retry Testing

```typescript
describe('RetryExecutor', () => {
  let retryExecutor: RetryExecutor;

  beforeEach(() => {
    retryExecutor = new RetryExecutor({
      maxAttempts: 3,
      baseDelayMs: 100,
      maxDelayMs: 5000,
      strategy: RetryStrategy.EXPONENTIAL_BACKOFF,
      jitter: false,
      retryCondition: (error: Error) => error.message.includes('retry'),
    });
  });

  describe('successful operations', () => {
    it('should succeed on first attempt', async () => {
      const operation = jest.fn().mockResolvedValue('success');

      const result = await retryExecutor.execute(operation);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(1);
    });

    it('should succeed after retries', async () => {
      const operation = jest
        .fn()
        .mockRejectedValueOnce(new Error('retry-able error'))
        .mockRejectedValueOnce(new Error('retry-able error'))
        .mockResolvedValue('success');

      const result = await retryExecutor.execute(operation);

      expect(result).toBe('success');
      expect(operation).toHaveBeenCalledTimes(3);
    });
  });

  describe('failing operations', () => {
    it('should not retry non-retryable errors', async () => {
      const operation = jest.fn().mockRejectedValue(new Error('non-retryable'));

      await expect(retryExecutor.execute(operation)).rejects.toThrow('non-retryable');
      expect(operation).toHaveBeenCalledTimes(1);
    });

    it('should fail after max attempts', async () => {
      const operation = jest.fn().mockRejectedValue(new Error('retry-able error'));

      await expect(retryExecutor.execute(operation)).rejects.toThrow(MaxRetriesExceededError);
      expect(operation).toHaveBeenCalledTimes(3);
    });
  });

  describe('retry strategies', () => {
    it('should calculate exponential backoff delays correctly', () => {
      const delays = [1, 2, 3].map((attempt) => retryExecutor['calculateDelay'](attempt));

      expect(delays[0]).toBe(100); // 100 * 2^0
      expect(delays[1]).toBe(200); // 100 * 2^1
      expect(delays[2]).toBe(400); // 100 * 2^2
    });

    it('should respect maximum delay', () => {
      const retryWithMaxDelay = new RetryExecutor({
        maxAttempts: 10,
        baseDelayMs: 1000,
        maxDelayMs: 5000,
        strategy: RetryStrategy.EXPONENTIAL_BACKOFF,
        jitter: false,
        retryCondition: () => true,
      });

      const longDelay = retryWithMaxDelay['calculateDelay'](10);
      expect(longDelay).toBe(5000); // Should be capped at maxDelayMs
    });
  });

  describe('retry conditions', () => {
    const httpRetryFactory = new RetryFactory();
    const httpRetry = httpRetryFactory.createHttpRetry();

    it('should retry on network errors', () => {
      const networkError = new Error('ECONNRESET');
      (networkError as any).code = 'ECONNRESET';

      expect(httpRetry['config'].retryCondition(networkError)).toBe(true);
    });

    it('should retry on server errors', () => {
      const serverError = new Error('Server Error');
      (serverError as any).status = 500;

      expect(httpRetry['config'].retryCondition(serverError)).toBe(true);
    });

    it('should not retry on client errors', () => {
      const clientError = new Error('Bad Request');
      (clientError as any).status = 400;

      expect(httpRetry['config'].retryCondition(clientError)).toBe(false);
    });
  });
});

describe('ExternalApiService with Retry', () => {
  let service: ExternalApiService;
  let mockHttpService: jest.Mocked<HttpService>;
  let mockRetryFactory: jest.Mocked<RetryFactory>;

  beforeEach(() => {
    mockHttpService = {
      post: jest.fn(),
      put: jest.fn(),
      get: jest.fn(),
    } as any;

    mockRetryFactory = {
      createHttpRetry: jest.fn(),
      createDatabaseRetry: jest.fn(),
      createExternalServiceRetry: jest.fn(),
    };

    service = new ExternalApiService(mockHttpService, mockRetryFactory);
  });

  describe('syncVendorData', () => {
    it('should retry on failure and succeed', async () => {
      const vendor = createTestVendor();

      mockHttpService.post
        .mockReturnValueOnce(throwError(new Error('Network timeout')))
        .mockReturnValueOnce(of({ status: 200, data: 'success' }));

      // Mock retry executor to actually retry
      const mockRetryExecutor = {
        execute: jest.fn().mockImplementation(async (op) => {
          // Simulate retry behavior
          try {
            return await op();
          } catch (error) {
            return await op(); // Retry once
          }
        }),
      };

      mockRetryFactory.createHttpRetry.mockReturnValue(mockRetryExecutor as any);

      await service.syncVendorData(vendor);

      expect(mockHttpService.post).toHaveBeenCalledTimes(2);
      expect(mockRetryExecutor.execute).toHaveBeenCalledTimes(1);
    });
  });
});
```

## Best Practices

### 1. Appropriate Retry Conditions

```typescript
// Good: Specific retry conditions
const retryCondition = (error: Error) => {
  const retryableStatuses = [408, 429, 500, 502, 503, 504];
  const retryableMessages = ['timeout', 'network', 'connection'];

  return (
    retryableStatuses.includes(error.status) ||
    retryableMessages.some((msg) => error.message.toLowerCase().includes(msg))
  );
};

// Avoid: Retrying everything
const badRetryCondition = (error: Error) => true; // Will retry validation errors, etc.
```

### 2. Jitter for Distributed Systems

```typescript
// Good: Use jitter to avoid thundering herd
const config: RetryConfig = {
  strategy: RetryStrategy.EXPONENTIAL_BACKOFF_FULL_JITTER,
  // or
  jitter: true,
};

// Avoid: Fixed intervals that can cause thundering herd
const badConfig: RetryConfig = {
  strategy: RetryStrategy.FIXED_DELAY,
  jitter: false,
};
```

### 3. Reasonable Limits

```typescript
// Good: Reasonable retry limits
const config: RetryConfig = {
  maxAttempts: 3, // Don't retry forever
  maxDelayMs: 30000, // Cap maximum delay at 30 seconds
  baseDelayMs: 1000, // Start with reasonable base delay
};

// Avoid: Excessive retries
const badConfig: RetryConfig = {
  maxAttempts: 100, // Too many attempts
  maxDelayMs: 300000, // 5 minute delays
  baseDelayMs: 10, // Too frequent initial retries
};
```

The Retry pattern in our oil & gas management system provides resilience against
transient failures, improves system availability, and ensures that temporary
issues don't cause permanent operation failures, while being configurable for
different types of operations and external services.
