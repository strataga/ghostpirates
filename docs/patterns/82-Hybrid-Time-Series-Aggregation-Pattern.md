# Pattern 82: Hybrid Time-Series + Relational Data Aggregation Pattern

**Status**: ✅ Implemented
**Category**: Data Access / Query Patterns / Multi-Source Aggregation
**Related Patterns**: [15 - Repository](./15-Repository-Pattern.md), [81 - Multi-Tenant SCADA Ingestion](./81-Multi-Tenant-SCADA-Ingestion-Pattern.md)

## Problem

How do you efficiently query and display production data when you have **two different data sources** optimized for different access patterns:

1. **Time-Series SCADA Data** (TimescaleDB):
   - High-frequency automated sensor readings (every 1-5 minutes)
   - Optimized for write-heavy workloads (500K+ tags/second)
   - Examples: `oil_rate`, `gas_rate`, `tubing_pressure`, `casing_pressure`

2. **Manual Field Data** (PostgreSQL):
   - Low-frequency manual pumper entries (twice daily)
   - Optimized for offline sync and audit trails
   - Examples: Daily production totals, equipment inspections, maintenance records

**UI Requirements**:
- Dashboard needs to show **both** SCADA readings AND manual entries on a single timeline
- Production charts need to overlay automated vs manual data for validation
- Users need to see all data sources in a unified view

**Anti-Patterns to Avoid**:
- ❌ Creating a database view (cross-table joins are expensive and inflexible)
- ❌ Denormalizing data (defeats the purpose of specialized storage)
- ❌ Frontend aggregation (two network requests, more complexity, no caching)

## Context

WellOS's architecture separates high-frequency time-series data from low-frequency relational data for performance optimization:

```
┌─────────────────────────────────────────────────────────────┐
│  Tenant Database (PostgreSQL + TimescaleDB)                 │
│                                                              │
│  ┌──────────────────────┐      ┌──────────────────────┐    │
│  │ scada_readings       │      │ field_entries        │    │
│  │ (TimescaleDB)        │      │ (PostgreSQL)         │    │
│  │─────────────────────-│      │──────────────────────│    │
│  │ High-frequency       │      │ Low-frequency        │    │
│  │ Automated sensors    │      │ Manual pumper entry  │    │
│  │ Every 1-5 minutes    │      │ Twice daily (6AM/PM) │    │
│  │                      │      │                      │    │
│  │ - timestamp          │      │ - recordedAt         │    │
│  │ - wellId             │      │ - wellId             │    │
│  │ - tagNodeId          │      │ - productionData {}  │    │
│  │ - value              │      │ - inspectionData {}  │    │
│  │ - quality            │      │ - maintenanceData {} │    │
│  └──────────────────────┘      └──────────────────────┘    │
│           ↓                              ↓                  │
│           └──────────────┬───────────────┘                  │
│                          ↓                                  │
│                  Backend Aggregation                        │
│           (CQRS Query - Merge in Application Layer)        │
└─────────────────────────────────────────────────────────────┘
                           ↓
                     Unified DTO
                           ↓
                    UI (Single API Call)
```

## Solution

### Backend Aggregation Service Pattern

Create a **unified CQRS query handler** that:
1. Queries both data sources independently (parallel execution)
2. Merges results in the application layer
3. Returns a unified DTO to the frontend
4. Enables caching at the API layer (Redis)

### Architecture

```typescript
// 1. Unified DTO (what the UI receives)
interface ProductionDataDto {
  timestamp: Date;
  wellId: string;
  source: 'SCADA' | 'MANUAL';  // Track data source

  // Production metrics
  oilRate?: number;     // Barrels per day
  gasRate?: number;     // MCF per day
  waterRate?: number;   // Barrels per day

  // Equipment metrics (SCADA only)
  tubingPressure?: number;  // PSI
  casingPressure?: number;  // PSI
  temperature?: number;     // Fahrenheit

  // Manual entry metadata (MANUAL only)
  runTime?: number;         // Hours
  comments?: string;
  enteredBy?: string;
  deviceId?: string;

  // Data quality
  quality: 'Good' | 'Bad' | 'Uncertain';
}

// 2. CQRS Query
export class GetProductionDataQuery implements IQuery {
  constructor(
    public readonly tenantId: string,
    public readonly wellId: string,
    public readonly startDate: Date,
    public readonly endDate: Date,
    public readonly includeSCADA = true,
    public readonly includeManual = true,
  ) {}
}

// 3. Query Handler (aggregation logic)
@QueryHandler(GetProductionDataQuery)
export class GetProductionDataHandler
  implements IQueryHandler<GetProductionDataQuery, ProductionDataDto[]>
{
  constructor(
    @Inject('IScadaReadingRepository')
    private readonly scadaRepo: IScadaReadingRepository,

    @Inject('IFieldEntryRepository')
    private readonly fieldEntryRepo: IFieldEntryRepository,
  ) {}

  async execute(query: GetProductionDataQuery): Promise<ProductionDataDto[]> {
    const { tenantId, wellId, startDate, endDate, includeSCADA, includeManual } = query;

    // Step 1: Query both sources in parallel
    const [scadaReadings, fieldEntries] = await Promise.all([
      includeSCADA
        ? this.scadaRepo.findByWellAndDateRange(tenantId, wellId, startDate, endDate)
        : [],
      includeManual
        ? this.fieldEntryRepo.findAll(tenantId, {
            wellId,
            entryType: 'PRODUCTION',
            startDate,
            endDate,
          })
        : [],
    ]);

    // Step 2: Transform SCADA readings to unified DTO
    const scadaData: ProductionDataDto[] = scadaReadings
      .filter((r) => r.quality === 'Good') // Filter bad quality readings
      .map((reading) => ({
        timestamp: reading.timestamp,
        wellId: reading.wellId,
        source: 'SCADA' as const,

        // Map tag names to metrics
        oilRate: reading.tagNodeId === 'oil_rate' ? reading.value : undefined,
        gasRate: reading.tagNodeId === 'gas_rate' ? reading.value : undefined,
        waterRate: reading.tagNodeId === 'water_rate' ? reading.value : undefined,
        tubingPressure: reading.tagNodeId === 'tubing_pressure' ? reading.value : undefined,
        casingPressure: reading.tagNodeId === 'casing_pressure' ? reading.value : undefined,
        temperature: reading.tagNodeId === 'temperature' ? reading.value : undefined,

        quality: reading.quality,
      }));

    // Step 3: Transform field entries to unified DTO
    const manualData: ProductionDataDto[] = fieldEntries
      .filter((entry) => entry.productionData)
      .map((entry) => ({
        timestamp: entry.recordedAt,
        wellId: entry.wellId,
        source: 'MANUAL' as const,

        oilRate: entry.productionData?.oilVolume,
        gasRate: entry.productionData?.gasVolume,
        waterRate: entry.productionData?.waterVolume,

        runTime: entry.productionData?.runTime,
        comments: entry.notes,
        enteredBy: entry.createdBy,
        deviceId: entry.deviceId,

        quality: 'Good', // Manual entries are assumed good quality
      }));

    // Step 4: Merge and sort by timestamp
    const combined = [...scadaData, ...manualData];
    combined.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());

    return combined;
  }
}
```

### Controller Endpoint

```typescript
@Controller('production-data')
export class ProductionDataController {
  constructor(private readonly queryBus: QueryBus) {}

  @Get('wells/:wellId')
  async getProductionData(
    @TenantContext() tenantId: string,
    @Param('wellId') wellId: string,
    @Query('startDate') startDate: string,
    @Query('endDate') endDate: string,
    @Query('includeSCADA') includeSCADA = true,
    @Query('includeManual') includeManual = true,
  ): Promise<ProductionDataDto[]> {
    const query = new GetProductionDataQuery(
      tenantId,
      wellId,
      new Date(startDate),
      new Date(endDate),
      includeSCADA,
      includeManual,
    );

    return this.queryBus.execute(query);
  }
}
```

### Frontend Usage (React Query)

```typescript
// 1. API Repository
export class ProductionDataRepository {
  async getProductionData(
    wellId: string,
    startDate: Date,
    endDate: Date,
    options?: {
      includeSCADA?: boolean;
      includeManual?: boolean;
    }
  ): Promise<ProductionDataDto[]> {
    const params = new URLSearchParams({
      startDate: startDate.toISOString(),
      endDate: endDate.toISOString(),
      includeSCADA: String(options?.includeSCADA ?? true),
      includeManual: String(options?.includeManual ?? true),
    });

    const response = await this.client.get(
      `/production-data/wells/${wellId}?${params}`
    );
    return response.data;
  }
}

// 2. React Query Hook
export function useProductionData(
  wellId: string,
  startDate: Date,
  endDate: Date
) {
  const repo = useProductionDataRepository();

  return useQuery({
    queryKey: ['production-data', wellId, startDate, endDate],
    queryFn: () => repo.getProductionData(wellId, startDate, endDate),
    staleTime: 5 * 60 * 1000, // 5 minutes (SCADA data updates every 5 min)
  });
}

// 3. Component Usage
export function ProductionChart({ wellId }: { wellId: string }) {
  const startDate = useMemo(() => subDays(new Date(), 7), []);
  const endDate = useMemo(() => new Date(), []);

  const { data, isLoading } = useProductionData(wellId, startDate, endDate);

  if (isLoading) return <Spinner />;

  // Separate SCADA and Manual for visualization
  const scadaData = data?.filter((d) => d.source === 'SCADA') ?? [];
  const manualData = data?.filter((d) => d.source === 'MANUAL') ?? [];

  return (
    <LineChart>
      <Line
        data={scadaData}
        dataKey="oilRate"
        stroke="#3b82f6"
        name="SCADA (Automated)"
      />
      <Scatter
        data={manualData}
        dataKey="oilRate"
        fill="#22c55e"
        name="Pumper (Manual)"
      />
    </LineChart>
  );
}
```

## Benefits

### 1. Performance Optimization
- **Parallel queries**: Fetch from both sources simultaneously
- **Caching**: Add Redis caching at the API layer (SCADA data TTL: 5 min, Manual data TTL: 1 hour)
- **No expensive joins**: Each database optimized for its access pattern

### 2. Flexibility
- **Filter by source**: UI can request SCADA-only, Manual-only, or both
- **Granularity control**: SCADA for real-time, Manual for daily summaries
- **Easy to extend**: Add new data sources without changing UI

### 3. Separation of Concerns
- **Domain logic in application layer**: Proper hexagonal architecture
- **Frontend simplicity**: Single API call, single React Query hook
- **Testability**: Mock repositories independently

### 4. Data Quality
- **Source tracking**: UI knows which data came from which source
- **Quality filtering**: Can filter out bad SCADA readings
- **Validation**: Compare automated vs manual for discrepancy detection

## Trade-offs

### ✅ Advantages
- **Performance**: Optimized queries + caching + parallel execution
- **Flexibility**: Easy to add filters, aggregations, new sources
- **Clean architecture**: Follows CQRS and hexagonal patterns
- **Single API call**: Frontend makes one request, not two

### ❌ Disadvantages
- **More code**: Query handler, DTOs, repository methods
- **Memory usage**: Merging large datasets in memory (mitigated by pagination)
- **Latency**: Application-layer aggregation adds ~10-50ms vs database view

## Alternative Approaches

### Option 1: Database View (NOT RECOMMENDED)
```sql
-- ❌ Expensive cross-table joins, inflexible
CREATE VIEW unified_production_data AS
SELECT
  timestamp,
  well_id,
  'SCADA' as source,
  ...
FROM scada_readings
UNION ALL
SELECT
  recorded_at as timestamp,
  well_id,
  'MANUAL' as source,
  ...
FROM field_entries;
```

**Why Not?**
- Expensive UNION ALL on large datasets
- Can't leverage TimescaleDB optimizations
- Hard to add caching or filtering logic

### Option 2: Frontend Aggregation (NOT RECOMMENDED)
```typescript
// ❌ Two network requests, more complexity
const { data: scadaData } = useScadaReadings(wellId, startDate, endDate);
const { data: manualData } = useFieldEntries(wellId, startDate, endDate);

const combined = useMemo(() => {
  return [...(scadaData ?? []), ...(manualData ?? [])]
    .sort((a, b) => a.timestamp - b.timestamp);
}, [scadaData, manualData]);
```

**Why Not?**
- Two API calls (more latency, more bandwidth)
- No backend caching benefits
- Merging logic duplicated in every component

### Option 3: Materialized View (OVERKILL for most cases)
```sql
-- Precomputed aggregate that combines both sources
CREATE MATERIALIZED VIEW hourly_production_summary AS
SELECT ... (complex aggregation logic);

-- Refresh periodically
REFRESH MATERIALIZED VIEW hourly_production_summary;
```

**Use Only If**:
- You have **very large datasets** (millions of rows per query)
- Queries take > 5 seconds even with optimization
- Data staleness is acceptable (e.g., 1-hour refresh)

## Real-World Usage

### Use Case 1: Production Dashboard
```typescript
// Show last 7 days of production data (SCADA + Manual)
const { data } = useProductionData(wellId, subDays(new Date(), 7), new Date());
```

### Use Case 2: Manual vs Automated Validation
```typescript
// Compare manual pumper entries against SCADA readings
const { data } = useProductionData(wellId, startDate, endDate);

const discrepancies = data.filter((scada) => {
  if (scada.source !== 'SCADA') return false;

  const manual = data.find(
    (m) => m.source === 'MANUAL' &&
    isSameDay(m.timestamp, scada.timestamp)
  );

  if (!manual) return false;

  const diff = Math.abs((scada.oilRate ?? 0) - (manual.oilRate ?? 0));
  return diff > 10; // More than 10 bbl/day difference
});
```

### Use Case 3: SCADA-Only Real-Time Monitoring
```typescript
// Show only SCADA data for real-time monitoring (no manual entries)
const { data } = useQuery({
  queryKey: ['production-data', wellId, startDate, endDate, 'scada-only'],
  queryFn: () => repo.getProductionData(wellId, startDate, endDate, {
    includeSCADA: true,
    includeManual: false,
  }),
  refetchInterval: 60000, // Refresh every minute
});
```

## Performance Optimization

### Add Caching Layer
```typescript
@QueryHandler(GetProductionDataQuery)
export class GetProductionDataHandler {
  constructor(
    @Inject('IScadaReadingRepository')
    private readonly scadaRepo: IScadaReadingRepository,

    @Inject('IFieldEntryRepository')
    private readonly fieldEntryRepo: IFieldEntryRepository,

    @Inject('CACHE_MANAGER')
    private readonly cacheManager: Cache,
  ) {}

  async execute(query: GetProductionDataQuery): Promise<ProductionDataDto[]> {
    // Check cache first
    const cacheKey = `production-data:${query.tenantId}:${query.wellId}:${query.startDate}:${query.endDate}`;
    const cached = await this.cacheManager.get<ProductionDataDto[]>(cacheKey);

    if (cached) {
      return cached;
    }

    // ... (query logic from above)

    // Cache for 5 minutes (SCADA update interval)
    await this.cacheManager.set(cacheKey, combined, { ttl: 300 });

    return combined;
  }
}
```

### Add Pagination
```typescript
export class GetProductionDataQuery implements IQuery {
  constructor(
    public readonly tenantId: string,
    public readonly wellId: string,
    public readonly startDate: Date,
    public readonly endDate: Date,
    public readonly page = 1,
    public readonly limit = 1000, // Return max 1000 records per page
  ) {}
}
```

## Testing

### Unit Test (Query Handler)
```typescript
describe('GetProductionDataHandler', () => {
  it('should merge SCADA and manual data sorted by timestamp', async () => {
    const mockScadaRepo = {
      findByWellAndDateRange: jest.fn().mockResolvedValue([
        {
          timestamp: new Date('2025-10-30T14:00:00Z'),
          wellId: 'well-1',
          tagNodeId: 'oil_rate',
          value: 248.3,
          quality: 'Good',
        },
      ]),
    };

    const mockFieldEntryRepo = {
      findAll: jest.fn().mockResolvedValue([
        {
          recordedAt: new Date('2025-10-30T06:00:00Z'),
          wellId: 'well-1',
          productionData: {
            oilVolume: 250,
            gasVolume: 448,
            waterVolume: 180,
          },
        },
      ]),
    };

    const handler = new GetProductionDataHandler(mockScadaRepo, mockFieldEntryRepo);
    const result = await handler.execute(
      new GetProductionDataQuery('tenant-1', 'well-1', startDate, endDate)
    );

    expect(result).toHaveLength(2);
    expect(result[0].source).toBe('MANUAL'); // 6 AM entry
    expect(result[1].source).toBe('SCADA');  // 2 PM reading
  });
});
```

## Related Patterns

- **[Pattern 15: Repository](./15-Repository-Pattern.md)** - Data access abstraction for both SCADA and field data
- **[Pattern 81: Multi-Tenant SCADA Ingestion](./81-Multi-Tenant-SCADA-Ingestion-Pattern.md)** - How SCADA data is written to TimescaleDB
- **[Pattern 4: CQRS](./04-CQRS-Pattern.md)** - Command-Query separation for write vs read optimization

## References

- WellOS Implementation: `apps/api/src/application/production/queries/get-production-data.query.ts`
- TimescaleDB Continuous Aggregates: https://docs.timescale.com/use-timescale/latest/continuous-aggregates/
- React Query Caching: https://tanstack.com/query/latest/docs/framework/react/guides/caching

## Version History

- **v1.0** (2025-10-30): Initial pattern documentation with backend aggregation service approach

---

*Pattern ID: 82*
*Created: 2025-10-30*
*Last Updated: 2025-10-30*
