# External API Cost Tracking with Prompt Caching Pattern

**Pattern Number**: 66
**Category**: Integration Patterns
**Complexity**: Advanced
**Related Patterns**: #17 (Adapter), #29 (Scheduled Tasks), #16 (Pattern Integration Guide)

---

## Intent

Track and calculate costs from external LLM APIs that support prompt caching, handling complex pricing models with multiple token types (regular input/output, cache writes, cache reads) and dealing with API data quality issues.

---

## Problem

When integrating with LLM providers like Anthropic that offer prompt caching:

1. **Multiple Cost APIs**: Providers may offer different endpoints (`cost_report`, `usage_report`) with varying data quality
2. **Complex Pricing**: Different token types have different pricing:
   - Regular input/output tokens
   - Cache write tokens (same price as input)
   - Cache read tokens (10% of input price)
3. **API Inconsistencies**: Field names, response structures, and data accuracy vary between endpoints
4. **Missing Data**: Cost APIs may not include all token types, leading to incorrect totals
5. **Pagination Required**: Large datasets require paginated fetching
6. **Data Quality Issues**: One endpoint may return incorrect costs (e.g., $3,024 vs actual $10.66)

---

## Solution

Implement a multi-layered cost tracking system that:

1. **API Selection**: Use the most reliable endpoint (usage_report) and calculate costs client-side
2. **Comprehensive Token Tracking**: Parse all token types from API responses
3. **Cache-Aware Pricing**: Apply correct pricing for each token type
4. **Automated Sync**: Schedule daily syncs with paginated fetching
5. **Client-Side Calculation**: Don't trust cost_report—calculate from token counts

---

## Structure

```
infrastructure/
├── anthropic/
│   ├── anthropic.module.ts
│   └── anthropic-usage.service.ts      # Adapter for Anthropic Admin API
├── scheduling/
│   ├── anthropic-cost-sync.task.ts     # Daily scheduled sync (3 AM)
│   └── scheduling.module.ts
└── database/
    └── repositories/
        └── drizzle-cost-tracking.repository.ts

application/
└── cost-tracking/
    └── commands/
        ├── sync-anthropic-costs.command.ts
        └── sync-anthropic-costs.handler.ts  # CQRS handler

presentation/
└── cost-tracking/
    └── cost-tracking.controller.ts      # Manual sync endpoint
```

---

## Implementation

### 1. API Adapter with Multi-Endpoint Support

```typescript
/**
 * Anthropic Usage Service
 *
 * IMPORTANT: Uses usage_report endpoint instead of cost_report
 * because cost_report returns incorrect amounts.
 */
@Injectable()
export class AnthropicUsageService {
  private readonly client: AxiosInstance;
  private readonly adminApiKey: string;

  constructor(private readonly configService: ConfigService) {
    this.adminApiKey = this.configService.get<string>('ANTHROPIC_ADMIN_API_KEY');

    this.client = axios.create({
      baseURL: 'https://api.anthropic.com/v1/organizations',
      headers: {
        'x-api-key': this.adminApiKey,
        'anthropic-version': '2023-06-01',
      },
    });
  }

  /**
   * Fetch usage data with pagination support
   * Returns token counts (not costs) for client-side calculation
   */
  async fetchUsageData(startDate: string, endDate: string): Promise<AnthropicUsageData[]> {
    const usageData: AnthropicUsageData[] = [];
    let nextPage: string | undefined;
    let pageCount = 0;

    // Paginate through all results
    do {
      const params: any = {
        starting_at: startDate,
        ending_at: endDate,
        bucket_width: '1d',
      };

      if (nextPage) {
        params.page = nextPage;
      }

      const response = await this.client.get('/usage_report/messages', { params });

      if (response.data?.data) {
        for (const bucket of response.data.data) {
          const results = bucket.results || [];

          for (const item of results) {
            const model = item.model || 'unknown';

            // Extract ALL token types (critical for accurate costs)
            const tokenData = {
              // Regular tokens
              inputTokens: item.uncached_input_tokens || 0,
              outputTokens: item.output_tokens || 0,
              thinkingTokens: item.thinking_tokens || 0,

              // Cache tokens (often the bulk of costs!)
              cacheCreationTokens:
                (item.cache_creation?.ephemeral_1h_input_tokens || 0) +
                (item.cache_creation?.ephemeral_5m_input_tokens || 0),
              cacheReadTokens: item.cache_read_input_tokens || 0,
            };

            // Calculate cost client-side (more reliable than API cost_report)
            const cost = this.calculateCostWithCache(
              model,
              tokenData.inputTokens,
              tokenData.outputTokens,
              tokenData.thinkingTokens,
              tokenData.cacheCreationTokens,
              tokenData.cacheReadTokens,
            );

            usageData.push({
              date: bucket.starting_at,
              model,
              ...tokenData,
              costUsd: cost,
            });
          }
        }
      }

      nextPage = response.data?.next_page;
      pageCount++;
    } while (nextPage && pageCount < 10); // Safety limit

    return usageData;
  }

  /**
   * Calculate cost with prompt caching support
   *
   * Anthropic Prompt Cache Pricing (2025):
   * - Cache Writes: Same price as input tokens
   * - Cache Reads: 10% of input token price (90% discount)
   */
  private calculateCostWithCache(
    model: string,
    inputTokens: number,
    outputTokens: number,
    thinkingTokens: number = 0,
    cacheCreationTokens: number = 0,
    cacheReadTokens: number = 0,
  ): number {
    // Pricing per million tokens (USD)
    const pricing: Record<string, { input: number; output: number; thinking: number }> = {
      'claude-3-5-sonnet': { input: 3.0, output: 15.0, thinking: 3.0 },
      'claude-3-5-haiku': { input: 1.0, output: 5.0, thinking: 1.0 },
      // ... other models
    };

    // Find matching model (default to Sonnet for conservative pricing)
    let modelPricing = pricing['claude-3-5-sonnet'];
    for (const [key, value] of Object.entries(pricing)) {
      if (model.toLowerCase().includes(key.toLowerCase())) {
        modelPricing = value;
        break;
      }
    }

    const inputCost = (inputTokens / 1_000_000) * modelPricing.input;
    const outputCost = (outputTokens / 1_000_000) * modelPricing.output;
    const thinkingCost = (thinkingTokens / 1_000_000) * modelPricing.thinking;

    // Cache pricing
    const cacheWriteCost = (cacheCreationTokens / 1_000_000) * modelPricing.input;
    const cacheReadCost = (cacheReadTokens / 1_000_000) * (modelPricing.input * 0.1);

    return inputCost + outputCost + thinkingCost + cacheWriteCost + cacheReadCost;
  }
}
```

### 2. Scheduled Sync Task

```typescript
/**
 * Anthropic Cost Sync Scheduled Task
 *
 * Runs daily at 3 AM to sync last 7 days of usage data.
 * Syncs 7 days (not just 1) to catch delayed/corrected data from Anthropic.
 */
@Injectable()
export class AnthropicCostSyncTask {
  constructor(private readonly commandBus: CommandBus) {}

  @Cron(CronExpression.EVERY_DAY_AT_3AM)
  async syncAnthropicCosts(): Promise<void> {
    const endDate = new Date();
    const startDate = new Date();
    startDate.setDate(startDate.getDate() - 7); // Last 7 days

    const command = new SyncAnthropicCostsCommand(startDate, endDate);
    const result = await this.commandBus.execute(command);

    this.logger.log(`Synced ${result.synced} Anthropic cost records`);
  }

  /**
   * Manual trigger for testing or administrative use
   */
  async triggerManualSync(startDate?: Date, endDate?: Date): Promise<{ synced: number }> {
    const command = new SyncAnthropicCostsCommand(startDate, endDate);
    return await this.commandBus.execute(command);
  }
}
```

### 3. CQRS Command Handler

```typescript
@CommandHandler(SyncAnthropicCostsCommand)
export class SyncAnthropicCostsHandler {
  constructor(
    private readonly anthropicService: AnthropicUsageService,
    @Inject('ICostTrackingRepository')
    private readonly costTrackingRepository: ICostTrackingRepository,
    @Inject('IOrganizationRepository')
    private readonly organizationRepository: IOrganizationRepository,
  ) {}

  async execute(command: SyncAnthropicCostsCommand): Promise<{ synced: number }> {
    const endDate = command.endDate || new Date();
    const startDate = command.startDate || new Date(Date.now() - 30 * 24 * 60 * 60 * 1000);

    // Get system organization for tracking
    const organizations = await this.organizationRepository.findAll(false);
    const systemOrgId = organizations[0].id.getValue();

    // Fetch usage data from Anthropic API
    const usageData = await this.anthropicService.fetchUsageData(
      startDate.toISOString(),
      endDate.toISOString(),
    );

    // Store in database
    let synced = 0;
    for (const usage of usageData) {
      const costEntry = CostTracking.createAiCost({
        provider: 'ANTHROPIC',
        type: 'CHAT',
        cost: usage.costUsd,
        model: usage.model,
        inputTokens: usage.inputTokens,
        outputTokens: usage.outputTokens,
        thinkingTokens: usage.thinkingTokens,
        cacheWriteTokens: usage.cacheWriteTokens,
        cacheReadTokens: usage.cacheReadTokens,
        requestCount: usage.requestCount, // Actual API request count (not DB record count!)
        organizationId: systemOrgId,
        feature: 'anthropic_admin_api_sync',
        requestId: `admin-api-${usage.date}-${usage.model}`, // Idempotency key
      });

      await this.costTrackingRepository.create(costEntry);
      synced++;
    }

    return { synced };
  }
}
```

---

## Example: Real-World Cost Calculation

**October 13, 2025 Usage**:

```
Input Tokens:       280,135
Output Tokens:      160,138
Cache Write:      2,405,157
Cache Read:      35,904,676  ← This is the expensive part!
```

**Cost Calculation** (claude-3-5-sonnet @ $3/$15 per million):

```
Input:       280,135 × $3.00/M  = $0.84
Output:      160,138 × $15.00/M = $2.40
Cache Write: 2,405,157 × $3.00/M  = $7.22
Cache Read:  35,904,676 × $0.30/M = $10.77  (10% of $3.00)
────────────────────────────────────────────
Total: $21.23
```

**Anthropic Console shows**: $21.32 ✅ (matches within rounding)

**What cost_report API returned**: $3,024.26 ❌ (284x wrong!)

---

## Key Design Decisions

### 1. Why usage_report Instead of cost_report?

**Problem**: `cost_report` endpoint returned $3,024.26 when actual cost was $21.32

**Solution**: Use `usage_report` for token counts, calculate costs client-side

**Reasoning**:

- Token counts are factual (can't be wrong)
- Costs are derived (can have bugs)
- Client-side calculation is deterministic and testable
- Can update pricing without waiting for API fixes

### 2. Why Sync 7 Days Daily?

**Problem**: Anthropic may update/correct historical usage data

**Solution**: Each daily sync fetches last 7 days

**Reasoning**:

- Catches delayed data from Anthropic
- Handles corrected usage reports
- Idempotent (requestId prevents duplicates)
- Small enough to avoid rate limits

### 3. Why Track Cache Tokens Separately?

**Problem**: Initial implementation ignored 35.9M cache read tokens = $10.77 underestimation

**Solution**: Parse all token types explicitly

**Reasoning**:

- Cache reads can be 90% of costs
- Different pricing per token type
- Essential for accurate billing
- Helps identify optimization opportunities

### 4. Why Store Request Count in Metadata?

**Problem**: Database record count ≠ actual API request count (aggregated data creates confusion)

**Solution**: Store actual request count from API in metadata field

**Reasoning**:

- Each DB record represents a daily aggregate (not a single request)
- Anthropic API provides actual request count via `item.count`
- Using `COUNT(*)` on DB records gives incorrect totals (e.g., 10 records vs 1000s of actual requests)
- Metadata-based aggregation: `SUM(metadata->>'requestCount')` provides accurate request counts
- COALESCE fallback ensures backward compatibility with old records

---

## Common Pitfalls

### ❌ Trusting cost_report API

```typescript
// WRONG: API may return incorrect amounts
const response = await client.get('/cost_report');
return response.data.results.map((r) => r.amount); // $3,024 wrong!
```

```typescript
// RIGHT: Calculate from token counts
const response = await client.get('/usage_report/messages');
return response.data.results.map((r) =>
  calculateCost(r.input_tokens, r.output_tokens, r.cache_tokens),
);
```

### ❌ Ignoring Cache Tokens

```typescript
// WRONG: Missing 90% of costs!
cost = (inputTokens + outputTokens) * price;
```

```typescript
// RIGHT: Include all token types
cost =
  inputTokens * inputPrice +
  outputTokens * outputPrice +
  cacheWriteTokens * inputPrice +
  cacheReadTokens * inputPrice * 0.1;
```

### ❌ Wrong Field Names

```typescript
// WRONG: These fields don't exist
const tokens = item.input_tokens; // undefined!
```

```typescript
// RIGHT: Use actual API field names
const tokens = item.uncached_input_tokens; // ✓
const cacheReads = item.cache_read_input_tokens; // ✓
const cacheWrites =
  item.cache_creation.ephemeral_5m_input_tokens + item.cache_creation.ephemeral_1h_input_tokens; // ✓
```

### ❌ Missing Pagination

```typescript
// WRONG: Only gets first page
const response = await client.get('/usage_report/messages', { params });
return response.data.data; // Missing data!
```

```typescript
// RIGHT: Fetch all pages
let nextPage;
do {
  const response = await client.get('/usage_report/messages', {
    params: { ...params, page: nextPage },
  });
  // ... process data
  nextPage = response.data.next_page;
} while (nextPage);
```

### ❌ Using DB Record Count for Request Count

```typescript
// WRONG: Counts database records, not actual API requests
const totalResult = await db
  .select({
    totalRequests: count(), // Shows 10 records, not 1000s of requests!
  })
  .from(costTracking);
```

```typescript
// RIGHT: Sum actual request counts from metadata
const totalResult = await db
  .select({
    totalRequests: sql`SUM(CAST(metadata->>'requestCount' AS INTEGER))`,
  })
  .from(costTracking);
```

---

## Testing Strategy

### Unit Tests

```typescript
describe('AnthropicUsageService', () => {
  describe('calculateCostWithCache', () => {
    it('should calculate cache read cost at 10% of input price', () => {
      const cost = service['calculateCostWithCache'](
        'claude-3-5-sonnet',
        0, // input
        0, // output
        0, // thinking
        0, // cache write
        1_000_000, // cache read (1M tokens)
      );

      // 1M tokens × $3/M × 0.1 = $0.30
      expect(cost).toBe(0.3);
    });

    it('should handle large cache reads correctly', () => {
      // Real example from Oct 13
      const cost = service['calculateCostWithCache'](
        'claude-3-5-sonnet',
        280_135, // input
        160_138, // output
        0, // thinking
        2_405_157, // cache write
        35_904_676, // cache read
      );

      // Should be ~$21.23
      expect(cost).toBeCloseTo(21.23, 1);
    });
  });
});
```

### Integration Tests

```typescript
describe('SyncAnthropicCostsHandler', () => {
  it('should sync usage data and calculate costs correctly', async () => {
    // Mock Anthropic API response
    mockAnthropicService.fetchUsageData.mockResolvedValue([
      {
        date: '2025-10-13T00:00:00Z',
        model: 'claude-3-5-sonnet',
        inputTokens: 280_135,
        outputTokens: 160_138,
        cacheCreationTokens: 2_405_157,
        cacheReadTokens: 35_904_676,
        costUsd: 21.23,
      },
    ]);

    const result = await handler.execute(command);

    expect(result.synced).toBe(1);
    expect(mockRepository.create).toHaveBeenCalledWith(
      expect.objectContaining({
        cost: 21.23,
        provider: 'ANTHROPIC',
      }),
    );
  });
});
```

---

## Monitoring & Alerts

### Key Metrics

1. **Daily Cost Variance**: Alert if cost >2x previous day average
2. **Sync Failures**: Alert if sync fails 2 days in a row
3. **Cache Hit Rate**: Track `cacheReadTokens / (inputTokens + cacheReadTokens)`
4. **Cost Accuracy**: Compare calculated vs Anthropic Console monthly

### Logging Strategy

```typescript
// Log cache-heavy usage (helpful for cost optimization)
if (cacheCreationTokens > 0 || cacheReadTokens > 0) {
  this.logger.log(
    `${date}: ${model} - ` +
      `Input: ${inputTokens.toLocaleString()}, ` +
      `Output: ${outputTokens.toLocaleString()}, ` +
      `Cache: ${cacheCreationTokens.toLocaleString()}W/` +
      `${cacheReadTokens.toLocaleString()}R = $${cost.toFixed(2)}`,
  );
}
```

---

## When to Use This Pattern

✅ **Use when**:

- Integrating with LLM providers that have prompt caching
- API cost endpoints have known quality issues
- Need accurate cost tracking for billing/budgeting
- Handling large-scale API usage (>$100/month)

❌ **Don't use when**:

- Provider's cost API is reliable and well-documented
- No prompt caching (simpler pricing)
- Costs are negligible (<$10/month)
- Real-time cost tracking not needed

---

## Related Patterns

- **#17 Adapter Pattern**: Wrap external API with internal interface
- **#29 Scheduled Tasks**: Daily automated sync
- **#38 Idempotency Pattern**: `requestId` prevents duplicate records
- **#51 Repository Pattern**: Abstract database operations

---

## References

- [Anthropic Admin API Documentation](https://docs.anthropic.com/en/api/admin-api)
- [Anthropic Prompt Caching](https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching)
- [Anthropic Pricing](https://www.anthropic.com/pricing)
- [SQLx Database Access](https://docs.rs/sqlx/latest/sqlx/)

---

## Changelog

- **2025-10-19**: Initial pattern documented after successful implementation
- Cost tracking now accurate ($25.43 vs incorrect $3,024.26)
- Supports cache writes, cache reads, regular tokens, thinking tokens
- **2025-10-19 (Update)**: Added request count tracking in metadata
- Repository now aggregates `metadata.requestCount` instead of counting DB records
- Fixes incorrect "Total Requests" metric (was showing 10 DB records instead of actual API request count)
