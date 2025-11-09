# Custom Metrics Dashboard Pattern

## Context

Many SaaS platforms use dedicated monitoring tools like Grafana, Datadog, or New Relic for visualizing application metrics. While powerful, these tools can be expensive, especially when running on cloud platforms like Azure where egress costs and service fees add up quickly.

**Problem**: Running Grafana on Azure Container Apps with persistent storage and public access can cost $50-150/month for small deployments, which is significant for a bootstrapped startup.

**Solution**: Build a custom metrics dashboard in the admin portal that reads directly from Prometheus metrics endpoint, providing real-time visibility without external tooling costs.

---

## Applicability

Use this pattern when:

- ✅ You need basic metrics visualization (gauges, counters, histograms)
- ✅ Real-time data (10-60 second refresh) is sufficient
- ✅ You want to minimize cloud infrastructure costs
- ✅ Your admin team is small (< 10 people)
- ✅ You already expose Prometheus metrics at `/metrics` endpoint
- ✅ Custom branding and integration with your admin portal is valuable

**Don't use this pattern when**:

- ❌ You need advanced visualizations (heatmaps, complex queries, PromQL)
- ❌ You require historical data retention beyond a few hours
- ❌ You need alerting and notification rules
- ❌ Multiple teams need different dashboard views with RBAC
- ❌ You're already paying for APM tools (might as well use their dashboards)

---

## Solution Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────┐
│                     Admin Portal (Next.js)                  │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  Metrics Dashboard Page (/metrics)                    │  │
│  │  - Auto-refresh every 10 seconds                      │  │
│  │  - Parses Prometheus text format                      │  │
│  │  - Displays cards with color-coded status             │  │
│  └───────────────────────┬───────────────────────────────┘  │
└────────────────────────────┼───────────────────────────────┘
                             │ HTTP GET /metrics
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                  WellOS API (Rust + Axum)                │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  prometheus-client library                             │  │
│  │  - Exposes /metrics endpoint                          │  │
│  │  - Default system metrics (CPU, memory, threads)      │  │
│  │  - Custom gauges via ConnectionPoolMetrics struct     │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

1. **Prometheus Metrics Endpoint** (`/metrics`)
   - Exposes metrics in Prometheus text format
   - Automatically collects Node.js process metrics
   - Custom metrics updated every 10 seconds by background service

2. **Metrics Dashboard Page** (`apps/admin/app/(dashboard)/metrics/page.tsx`)
   - Client-side React component with `useEffect` polling
   - Parses Prometheus text format using regex
   - Displays data in shadcn/ui cards, badges, and tabs
   - Color-coded status indicators (healthy/warning/critical)

3. **Connection Pool Metrics Service** (`ConnectionPoolMetricsService`)
   - Background service that runs every 10 seconds
   - Collects pool metrics from all tenant connections
   - Updates Prometheus gauges

---

## Implementation

### Backend: Prometheus Metrics Endpoint

**File**: `apps/api/src/infrastructure/monitoring/metrics.rs`

```rust
use axum::{routing::get, Router};
use prometheus_client::{
    encoding::text::encode,
    metrics::family::Family,
    metrics::gauge::Gauge,
    registry::Registry,
};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MetricsRegistry {
    registry: Arc<RwLock<Registry>>,
    pool_size: Family<PoolLabels, Gauge>,
    pool_idle: Family<PoolLabels, Gauge>,
    pool_waiting: Family<PoolLabels, Gauge>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        let mut registry = Registry::default();

        let pool_size = Family::<PoolLabels, Gauge>::default();
        let pool_idle = Family::<PoolLabels, Gauge>::default();
        let pool_waiting = Family::<PoolLabels, Gauge>::default();

        registry.register(
            "wellos_tenant_connection_pool_size",
            "Total connections in pool",
            pool_size.clone(),
        );

        registry.register(
            "wellos_tenant_connection_pool_idle",
            "Idle connections in pool",
            pool_idle.clone(),
        );

        registry.register(
            "wellos_tenant_connection_pool_waiting",
            "Clients waiting for connection",
            pool_waiting.clone(),
        );

        Self {
            registry: Arc::new(RwLock::new(registry)),
            pool_size,
            pool_idle,
            pool_waiting,
        }
    }
}

pub fn metrics_routes(metrics: Arc<MetricsRegistry>) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(metrics)
}

async fn metrics_handler(
    State(metrics): State<Arc<MetricsRegistry>>,
) -> String {
    let registry = metrics.registry.read().await;
    let mut buffer = String::new();
    encode(&mut buffer, &registry).unwrap();
    buffer
}
```

**Exclude `/metrics` from tenant middleware**:

```rust
// apps/api/src/main.rs
async fn tenant_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    // Skip tenant resolution for public endpoints
    if req.uri().path().starts_with("/health")
        || req.uri().path().starts_with("/metrics")  // NEW - No tenant context needed
        || req.uri().path().starts_with("/tenants") {
        return Ok(next.run(req).await);
    }

    // Extract tenant from subdomain...
}
```

### Custom Metrics Collection

**File**: `apps/api/src/infrastructure/monitoring/connection_pool_metrics.rs`

```rust
use prometheus_client::metrics::gauge::Gauge;
use prometheus_client::metrics::family::Family;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::{info, error};
use crate::infrastructure::database::TenantDatabaseService;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PoolLabels {
    pub tenant_id: String,
    pub database_name: String,
}

impl prometheus_client::encoding::EncodeLabelSet for PoolLabels {
    fn encode(&self, encoder: &mut prometheus_client::encoding::LabelSetEncoder) -> Result<(), std::fmt::Error> {
        ("tenant_id", self.tenant_id.as_str()).encode(encoder)?;
        ("database_name", self.database_name.as_str()).encode(encoder)?;
        Ok(())
    }
}

pub struct ConnectionPoolMetrics {
    pool_size: Family<PoolLabels, Gauge>,
    pool_idle: Family<PoolLabels, Gauge>,
    pool_waiting: Family<PoolLabels, Gauge>,
    tenant_db_service: Arc<TenantDatabaseService>,
}

impl ConnectionPoolMetrics {
    pub fn new(
        pool_size: Family<PoolLabels, Gauge>,
        pool_idle: Family<PoolLabels, Gauge>,
        pool_waiting: Family<PoolLabels, Gauge>,
        tenant_db_service: Arc<TenantDatabaseService>,
    ) -> Self {
        Self {
            pool_size,
            pool_idle,
            pool_waiting,
            tenant_db_service,
        }
    }

    pub async fn start_collection(self: Arc<Self>) {
        let mut ticker = interval(Duration::from_secs(10)); // Every 10 seconds

        loop {
            ticker.tick().await;
            self.collect_metrics().await;
        }
    }

    async fn collect_metrics(&self) {
        let connections = self.tenant_db_service.get_all_connections().await;

        for (tenant_id, pool) in connections {
            let database_name = format!("{}_wellos", tenant_id);
            let labels = PoolLabels {
                tenant_id: tenant_id.clone(),
                database_name,
            };

            // Get pool stats from SQLx
            let total_count = pool.size();
            let idle_count = pool.num_idle();
            let waiting_count = 0; // SQLx doesn't expose this directly

            self.pool_size.get_or_create(&labels).set(total_count as i64);
            self.pool_idle.get_or_create(&labels).set(idle_count as usize as i64);
            self.pool_waiting.get_or_create(&labels).set(waiting_count);

            info!(
                tenant_id = %tenant_id,
                total = total_count,
                idle = idle_count,
                "Collected pool metrics"
            );
        }
    }
}
```

**Key Insight**: Use `prometheus_client`'s `Family<Labels, Gauge>` pattern for labeled metrics. The metrics registry automatically aggregates metrics with the same label set.

### Frontend: Custom Dashboard

**File**: `apps/admin/app/(dashboard)/metrics/page.tsx`

```typescript
'use client';

import { useEffect, useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

interface ParsedMetrics {
  connectionPools: Array<{
    tenantId: string;
    databaseName: string;
    totalConnections: number;
    idleConnections: number;
    waitingClients: number;
  }>;
  httpMetrics: { totalRequests: number; avgLatency: number };
  systemMetrics: { cpuUsage: number; memoryUsed: number; memoryTotal: number };
}

export default function MetricsPage() {
  const [metrics, setMetrics] = useState<ParsedMetrics | null>(null);
  const [autoRefresh, setAutoRefresh] = useState(true);

  const fetchMetrics = async () => {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4000'}/metrics`,
    );
    const metricsText = await response.text();
    const parsed = parsePrometheusMetrics(metricsText);
    setMetrics(parsed);
  };

  useEffect(() => {
    fetchMetrics();
    if (autoRefresh) {
      const interval = setInterval(fetchMetrics, 10000); // 10 seconds
      return () => clearInterval(interval);
    }
  }, [autoRefresh]);

  const parsePrometheusMetrics = (text: string): ParsedMetrics => {
    // Parse Prometheus text format using regex
    // Extract: tenant_connection_pool_size{tenant_id="...",database_name="..."} 10
    // ... (see full implementation in metrics page)
  };

  return (
    <div>
      <h1>System Metrics</h1>
      <Tabs defaultValue="pools">
        <TabsList>
          <TabsTrigger value="pools">Connection Pools</TabsTrigger>
          <TabsTrigger value="http">HTTP Metrics</TabsTrigger>
          <TabsTrigger value="system">System Resources</TabsTrigger>
        </TabsList>

        <TabsContent value="pools">
          {metrics?.connectionPools.map((pool) => {
            const utilizationPercent = Math.round(
              ((pool.totalConnections - pool.idleConnections) / pool.totalConnections) * 100,
            );
            const status =
              pool.waitingClients > 0
                ? 'critical'
                : utilizationPercent > 80
                ? 'warning'
                : 'healthy';

            return (
              <Card key={`${pool.tenantId}:${pool.databaseName}`}>
                <CardHeader>
                  <CardTitle>{pool.tenantId}</CardTitle>
                  <Badge variant={status === 'critical' ? 'destructive' : 'default'}>
                    {status.toUpperCase()}
                  </Badge>
                </CardHeader>
                <CardContent>
                  <div>Total: {pool.totalConnections}</div>
                  <div>Active: {pool.totalConnections - pool.idleConnections}</div>
                  <div>Idle: {pool.idleConnections}</div>
                  {pool.waitingClients > 0 && <div>Waiting: {pool.waitingClients}</div>}
                </CardContent>
              </Card>
            );
          })}
        </TabsContent>
      </Tabs>
    </div>
  );
}
```

---

## Status Indicators and Alerts

### Color-Coded Status Logic

```typescript
const status =
  pool.waitingClients > 0
    ? 'critical' // Red badge - clients are being blocked
    : utilizationPercent > 80
      ? 'warning' // Yellow badge - high utilization
      : 'healthy'; // Green badge - normal operation
```

**Thresholds**:

- **Healthy** (Green): < 80% pool utilization, no waiting clients
- **Warning** (Yellow): ≥ 80% pool utilization, but no waiting clients yet
- **Critical** (Red): Any clients waiting for connections (pool exhausted)

---

## Cost Savings Analysis

### Traditional Grafana Setup on Azure

```
Azure Container Apps (Grafana):
- 1 vCPU, 2GB RAM: $30/month
- Azure PostgreSQL (metrics storage): $50/month
- Egress (data transfer): $10/month
- Backup/disaster recovery: $10/month
---------------------------------------------
Total: $100/month = $1,200/year
```

### Custom Dashboard in Admin Portal

```
Next.js Admin App (already running):
- No additional compute cost
- No additional storage cost
- No egress fees (metrics endpoint is internal)
- Development time: 4 hours (~$200 one-time)
---------------------------------------------
Total: $0/month ongoing, $200 one-time
```

**Break-even point**: 2 months

**5-year savings**: $6,000 - $200 = **$5,800**

---

## Trade-offs

### Advantages ✅

1. **Cost Savings**: Zero ongoing infrastructure costs
2. **Simplicity**: No external tool setup, login management, or configuration
3. **Integration**: Built into existing admin portal with same authentication
4. **Customization**: Full control over UI/UX, can add business-specific metrics
5. **Performance**: No external network calls, metrics endpoint is co-located

### Disadvantages ❌

1. **No Historical Data**: Only shows current state (Grafana can store weeks/months)
2. **Limited Visualizations**: Basic cards/tables vs. Grafana's rich chart library
3. **No Alerting**: Must manually check dashboard (Grafana has alerting rules)
4. **Manual Refresh**: Polling-based (Grafana has push subscriptions)
5. **Development Time**: Must build and maintain custom parsing/display logic

---

## When to Upgrade to Grafana

As your platform grows, consider switching to Grafana when:

1. **Team Size**: > 10 people need access to metrics with different permissions
2. **Data Retention**: Business needs historical trend analysis (> 1 week)
3. **Alerting**: Need automated alerts/notifications (PagerDuty, Slack, email)
4. **Complex Queries**: Need PromQL for aggregations, rate calculations, percentiles
5. **Multiple Data Sources**: Combining metrics from multiple systems (Postgres, Redis, external APIs)
6. **SLA Reporting**: Generating monthly uptime/performance reports for customers

---

## Evolution Path

### Phase 1: Custom Dashboard (Current) - Months 1-12

- Basic connection pool metrics
- HTTP request counts
- System resource usage
- Manual refresh every 10 seconds

### Phase 2: Enhanced Dashboard - Months 13-24

- Add time-series charts using recharts or chart.js
- Store last 24 hours in Redis with TTL
- Add CSV export for ad-hoc analysis
- Email alerts when pool utilization > 90%

### Phase 3: Grafana Migration - Year 2+

- Deploy Grafana when team > 10 people
- Migrate custom dashboards to Grafana JSON
- Keep custom dashboard for lightweight access
- Use Grafana for advanced analysis, custom dashboard for quick checks

---

## Related Patterns

- **[Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)** - Why connection pool metrics are critical for multi-tenant systems
- **[Health Check Pattern](./13-Health-Check-Pattern.md)** - Complementary monitoring for uptime
- **Observer Pattern** - Background metrics collection service

---

## Testing Strategy

### Unit Tests

```typescript
describe('parsePrometheusMetrics', () => {
  it('should parse connection pool metrics correctly', () => {
    const input = `
# HELP tenant_connection_pool_size Total connections
# TYPE tenant_connection_pool_size gauge
tenant_connection_pool_size{tenant_id="acme",database_name="acme_wellos"} 10
tenant_connection_pool_idle{tenant_id="acme",database_name="acme_wellos"} 3
tenant_connection_pool_waiting{tenant_id="acme",database_name="acme_wellos"} 0
    `;

    const result = parsePrometheusMetrics(input);

    expect(result.connectionPools).toEqual([
      {
        tenantId: 'acme',
        databaseName: 'acme_wellos',
        totalConnections: 10,
        idleConnections: 3,
        waitingClients: 0,
      },
    ]);
  });
});
```

### E2E Tests

```typescript
test('metrics dashboard displays connection pool data', async ({ page }) => {
  // Navigate to metrics page
  await page.goto('/metrics');

  // Wait for metrics to load
  await page.waitForSelector('[data-testid="connection-pool-card"]');

  // Verify pool metrics are displayed
  const poolCard = page.locator('[data-testid="connection-pool-card"]').first();
  await expect(poolCard).toContainText('Total Connections');
  await expect(poolCard).toContainText('Active');
  await expect(poolCard).toContainText('Idle');

  // Verify status badge is present
  await expect(poolCard.locator('[data-testid="status-badge"]')).toBeVisible();
});
```

---

## Key Insights

★ **Insight ─────────────────────────────────────**

1. **Cost-Driven Architecture**: Sometimes the best solution is the simplest one. Before adding external tools, evaluate if you can build a basic version in-house. The custom dashboard saved $1,200/year with 4 hours of development.

2. **Prometheus Text Format Parsing**: Prometheus metrics use a simple text format that's easy to parse with regex. No need for specialized client libraries or SDKs. Example:

   ```
   metric_name{label1="value1",label2="value2"} 42
   ```

3. **Metrics Registry Pattern**: When using `prometheus_client` in Rust, use `Family<Labels, Gauge>` for labeled metrics:
   ```rust
   use prometheus_client::metrics::family::Family;
   use prometheus_client::metrics::gauge::Gauge;

   let pool_size = Family::<PoolLabels, Gauge>::default();
   registry.register("metric_name", "Help text", pool_size.clone());
   ```
   This pattern allows efficient metric collection with multiple label dimensions.

─────────────────────────────────────────────────

---

## References

- [Prometheus Exposition Formats](https://prometheus.io/docs/instrumenting/exposition_formats/)
- [prometheus-client (Rust Prometheus Client)](https://github.com/prometheus/client_rust)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
- [SQLx Connection Pooling](https://github.com/launchbadge/sqlx)
