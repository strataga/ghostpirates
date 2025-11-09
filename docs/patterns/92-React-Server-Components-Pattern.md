# Pattern 92: React Server Components Pattern

**Status**: ✅ Recommended for Q1 2026 MVP
**Category**: Frontend Performance / Server-Side Rendering / Next.js 15+
**Related Patterns**: [46 - Caching Strategy](./46-Caching-Strategy-Patterns.md), [18 - Frontend Patterns Guide](./18-Frontend-Patterns-Guide.md), [87 - Time-Series Visualization](./87-Time-Series-Visualization-Pattern.md)

## Problem

Traditional React applications suffer from performance bottlenecks:

**Client-Side Rendering (CSR) Challenges**:
- ❌ **Large JavaScript bundles** - 500KB-2MB initial downloads
- ❌ **Slow Time-to-Interactive (TTI)** - Users wait 3-5 seconds before page is usable
- ❌ **Poor SEO** - Search engines see empty HTML shells
- ❌ **Network waterfalls** - Multiple sequential API calls from browser
- ❌ **Duplicate logic** - Data fetching code duplicated client/server

**Example: WellOS Dashboard**:
```tsx
// ❌ Traditional Client-Side Approach
export default function DashboardPage() {
  const { data: wells, isLoading } = useQuery({
    queryKey: ['wells'],
    queryFn: () => fetch('/api/wells').then(r => r.json())  // Client fetch
  });

  const { data: production } = useQuery({
    queryKey: ['production'],
    queryFn: () => fetch('/api/production').then(r => r.json())  // Another client fetch
  });

  if (isLoading) return <Spinner />;

  return (
    <div>
      <WellsTable wells={wells} />
      <ProductionChart data={production} />
    </div>
  );
}
```

**Problems**:
1. **Bundle size**: React Query + data fetching = +80KB JavaScript
2. **Network waterfalls**: 2 sequential API calls after page load
3. **Loading states**: Users see spinners for 2-3 seconds
4. **Time to First Byte (TTFB)**: 800ms before any content
5. **Cumulative Layout Shift (CLS)**: Content pops in causing visual jumps

**Business Impact**:
- **Slow dashboards** - Operators wait 3-5 seconds for data
- **High infrastructure costs** - Large bundles = more bandwidth
- **Poor mobile experience** - Slow networks amplify problems
- **Reduced engagement** - Users abandon slow-loading pages

## Context

Next.js 15+ introduces **React Server Components (RSC)** which fundamentally change how React applications work:

```
Traditional React (Client Components)        React Server Components
┌──────────────────────────────────┐       ┌──────────────────────────────────┐
│  Browser (Client)                │       │  Server                          │
│                                  │       │                                  │
│  1. Download HTML shell          │       │  1. Fetch data from database     │
│  2. Download 500KB JavaScript    │       │  2. Render React components      │
│  3. Hydrate React                │       │  3. Send pre-rendered RSC        │
│  4. Fetch /api/wells             │       │  4. Stream HTML progressively    │
│  5. Fetch /api/production        │       │                                  │
│  6. Re-render with data          │       │  ↓ Browser receives HTML         │
│                                  │       │                                  │
│  Total: 3-5 seconds              │       │  1. Show content (instant!)      │
│  JavaScript: 500KB               │       │  2. Download 50KB JavaScript     │
│  TTFB: 800ms                     │       │  3. Hydrate interactive parts    │
│                                  │       │                                  │
│                                  │       │  Total: 0.5-1 second             │
│                                  │       │  JavaScript: 50KB (90% smaller!) │
│                                  │       │  TTFB: 80ms (10x faster!)        │
└──────────────────────────────────┘       └──────────────────────────────────┘
```

## Solution

Use **React Server Components** to render on the server and stream HTML to the client.

### Key Concepts

#### 1. Server Components (Default in Next.js 15 App Router)

```tsx
// app/(dashboard)/wells/page.tsx
// ✅ Server Component (default - no "use client" directive)

import { getWells } from '@/lib/repositories/well.repository';
import { WellsTable } from './wells-table';

export default async function WellsPage() {
  // This runs on the server ONLY
  // Direct database access, no API route needed
  const wells = await getWells();

  return (
    <div>
      <h1>Wells</h1>
      <WellsTable wells={wells} />  {/* Pass data as props */}
    </div>
  );
}
```

**Benefits**:
- ✅ **Direct database access** - No API route overhead
- ✅ **Zero client JavaScript** - Component code doesn't ship to browser
- ✅ **Instant data** - No loading spinner needed
- ✅ **SEO-friendly** - Fully rendered HTML
- ✅ **Secure** - API keys/secrets stay on server

#### 2. Client Components (For Interactivity)

```tsx
// app/(dashboard)/wells/wells-table.tsx
// ✅ Client Component (needs "use client" for hooks/events)

'use client';  // ← This directive marks it as client component

import { useState } from 'react';
import { useRouter } from 'next/navigation';

export function WellsTable({ wells }: { wells: Well[] }) {
  const [sortBy, setSortBy] = useState('name');
  const router = useRouter();

  return (
    <table>
      <thead>
        <tr>
          <th onClick={() => setSortBy('name')}>Name</th>  {/* Client interaction */}
          <th onClick={() => setSortBy('status')}>Status</th>
        </tr>
      </thead>
      <tbody>
        {wells
          .sort((a, b) => a[sortBy].localeCompare(b[sortBy]))
          .map(well => (
            <tr key={well.id} onClick={() => router.push(`/wells/${well.id}`)}>
              <td>{well.name}</td>
              <td>{well.status}</td>
            </tr>
          ))}
      </tbody>
    </table>
  );
}
```

**When to use Client Components**:
- ✅ Event handlers (onClick, onChange)
- ✅ React hooks (useState, useEffect, useRef)
- ✅ Browser APIs (localStorage, window, document)
- ✅ Third-party libraries with hooks
- ✅ Animations and transitions

**When to use Server Components**:
- ✅ Data fetching (database queries)
- ✅ Server-only operations (file system, env vars)
- ✅ Static content (no interactivity)
- ✅ Large dependencies (Markdown parser, PDF generator)
- ✅ Security-sensitive code (API keys)

#### 3. Streaming and Suspense

```tsx
// app/(dashboard)/page.tsx
// ✅ Stream components as they complete

import { Suspense } from 'react';
import { WellsTable } from './wells-table';
import { ProductionChart } from './production-chart';
import { RecentAlerts } from './recent-alerts';

export default async function DashboardPage() {
  return (
    <div className="grid grid-cols-2 gap-4">
      {/* Fast component - renders immediately */}
      <Suspense fallback={<WellsSkeleton />}>
        <WellsSection />  {/* Fetches wells (50ms) */}
      </Suspense>

      {/* Slow component - streams in when ready */}
      <Suspense fallback={<ProductionSkeleton />}>
        <ProductionSection />  {/* Fetches production (200ms) */}
      </Suspense>

      {/* Another slow component */}
      <Suspense fallback={<AlertsSkeleton />}>
        <AlertsSection />  {/* Fetches alerts (150ms) */}
      </Suspense>
    </div>
  );
}

async function WellsSection() {
  const wells = await getWells();  // 50ms query
  return <WellsTable wells={wells} />;
}

async function ProductionSection() {
  const production = await getProduction();  // 200ms query
  return <ProductionChart data={production} />;
}

async function AlertsSection() {
  const alerts = await getAlerts();  // 150ms query
  return <RecentAlerts alerts={alerts} />;
}
```

**How Streaming Works**:
```
Traditional (Wait for All):
Server: Wait 200ms → Send complete HTML
Browser: Shows nothing → Shows everything at once
TTFB: 200ms

Streaming (Progressive Rendering):
Server: Send HTML shell immediately (5ms)
        → Stream WellsSection (50ms)
        → Stream AlertsSection (150ms)
        → Stream ProductionSection (200ms)
Browser: Shows skeleton (5ms)
         → Shows wells (50ms)
         → Shows alerts (150ms)
         → Shows production (200ms)
TTFB: 5ms (40x faster!)
```

#### 4. Data Fetching Patterns

```tsx
// ✅ Pattern 1: Parallel Data Fetching (Fast!)
async function DashboardPage() {
  // Fetch in parallel (takes 200ms total, not 400ms sequential)
  const [wells, production] = await Promise.all([
    getWells(),       // 200ms
    getProduction(),  // 200ms
  ]);

  return (
    <>
      <WellsTable wells={wells} />
      <ProductionChart data={production} />
    </>
  );
}

// ✅ Pattern 2: Sequential When Needed
async function WellDetailPage({ params }: { params: { id: string } }) {
  const well = await getWell(params.id);  // Must fetch first

  // Then fetch dependent data
  const [production, alarms] = await Promise.all([
    getProduction(well.id),
    getAlarms(well.id),
  ]);

  return <WellDetail well={well} production={production} alarms={alarms} />;
}

// ✅ Pattern 3: Streaming (Best UX)
async function DashboardPage() {
  return (
    <div>
      {/* Critical data - no suspense */}
      <WellsSummary wells={await getWells()} />

      {/* Non-critical - stream in when ready */}
      <Suspense fallback={<ChartSkeleton />}>
        <ProductionChartAsync />
      </Suspense>
    </div>
  );
}

async function ProductionChartAsync() {
  const data = await getProduction();  // Slow query
  return <ProductionChart data={data} />;
}
```

### Implementation

#### Project Structure

```
apps/web/
├── app/                          # Next.js 15 App Router
│   ├── (dashboard)/             # Route group (doesn't affect URL)
│   │   ├── layout.tsx           # Server Component (shared layout)
│   │   ├── page.tsx             # Server Component (dashboard home)
│   │   ├── wells/
│   │   │   ├── page.tsx         # Server Component (fetch wells)
│   │   │   ├── wells-table.tsx  # Client Component (interactive table)
│   │   │   └── [id]/
│   │   │       ├── page.tsx     # Server Component (fetch well detail)
│   │   │       └── well-chart.tsx  # Client Component (interactive chart)
│   │   └── production/
│   │       ├── page.tsx         # Server Component
│   │       └── production-form.tsx  # Client Component
│   └── api/                     # API routes (still needed for mutations)
│       └── wells/
│           └── route.ts         # POST/PUT/DELETE endpoints
└── lib/
    ├── repositories/            # Server-only data access
    │   ├── well.repository.ts   # Direct database queries
    │   └── production.repository.ts
    └── hooks/                   # Client-only React hooks
        ├── use-wells.ts         # For client-side mutations
        └── use-production.ts
```

#### Example: Dashboard Page

```tsx
// app/(dashboard)/page.tsx
// ✅ Server Component - Fetches data on server

import { Suspense } from 'react';
import { getWellsSummary } from '@/lib/repositories/well.repository';
import { getProductionSummary } from '@/lib/repositories/production.repository';
import { WellsMap } from './wells-map';
import { ProductionChart } from './production-chart';
import { RecentAlerts } from './recent-alerts';

export default async function DashboardPage() {
  // Fast query - no suspense needed
  const wellsSummary = await getWellsSummary();

  return (
    <div className="space-y-6">
      {/* Header with critical info (instant) */}
      <div className="grid grid-cols-4 gap-4">
        <StatCard title="Active Wells" value={wellsSummary.activeCount} />
        <StatCard title="Producing" value={wellsSummary.producingCount} />
        <StatCard title="Shut In" value={wellsSummary.shutInCount} />
        <StatCard title="Alarms" value={wellsSummary.alarmCount} />
      </div>

      {/* Map (streams in when ready) */}
      <Suspense fallback={<MapSkeleton />}>
        <WellsMapAsync />
      </Suspense>

      {/* Production chart (streams in when ready) */}
      <Suspense fallback={<ChartSkeleton />}>
        <ProductionChartAsync />
      </Suspense>

      {/* Recent alerts (streams in when ready) */}
      <Suspense fallback={<AlertsSkeleton />}>
        <RecentAlertsAsync />
      </Suspense>
    </div>
  );
}

// Separate async components for streaming
async function WellsMapAsync() {
  const wells = await getWells();  // 100ms query
  return <WellsMap wells={wells} />;
}

async function ProductionChartAsync() {
  const production = await getProductionSummary();  // 300ms query
  return <ProductionChart data={production} />;
}

async function RecentAlertsAsync() {
  const alerts = await getRecentAlerts();  // 150ms query
  return <RecentAlerts alerts={alerts} />;
}
```

#### Example: Interactive Table (Client Component)

```tsx
// app/(dashboard)/wells/wells-table.tsx
// ✅ Client Component - Handles sorting, filtering, pagination

'use client';

import { useState, useMemo } from 'react';
import { useRouter } from 'next/navigation';
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  flexRender,
} from '@tanstack/react-table';

interface WellsTableProps {
  wells: Well[];  // Passed from server component
}

export function WellsTable({ wells }: WellsTableProps) {
  const router = useRouter();
  const [sorting, setSorting] = useState<SortingState>([]);
  const [filtering, setFiltering] = useState('');

  const columns = useMemo(() => [
    {
      accessorKey: 'name',
      header: 'Well Name',
      cell: (info) => info.getValue(),
    },
    {
      accessorKey: 'status',
      header: 'Status',
      cell: (info) => (
        <StatusBadge status={info.getValue() as WellStatus} />
      ),
    },
    {
      accessorKey: 'production',
      header: 'Production (bbl/d)',
      cell: (info) => info.getValue()?.toFixed(1) ?? 'N/A',
    },
  ], []);

  const table = useReactTable({
    data: wells,
    columns,
    state: {
      sorting,
      globalFilter: filtering,
    },
    onSortingChange: setSorting,
    onGlobalFilterChange: setFiltering,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
  });

  return (
    <div className="space-y-4">
      {/* Search input (client-side filtering) */}
      <input
        type="text"
        value={filtering}
        onChange={(e) => setFiltering(e.target.value)}
        placeholder="Search wells..."
        className="input"
      />

      {/* Table */}
      <table className="table">
        <thead>
          {table.getHeaderGroups().map(headerGroup => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map(header => (
                <th
                  key={header.id}
                  onClick={header.column.getToggleSortingHandler()}
                  className="cursor-pointer"
                >
                  {flexRender(
                    header.column.columnDef.header,
                    header.getContext()
                  )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map(row => (
            <tr
              key={row.id}
              onClick={() => router.push(`/wells/${row.original.id}`)}
              className="cursor-pointer hover:bg-gray-50"
            >
              {row.getVisibleCells().map(cell => (
                <td key={cell.id}>
                  {flexRender(cell.column.columnDef.cell, cell.getContext())}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>

      {/* Pagination (client-side) */}
      <div className="flex justify-between items-center">
        <button
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          Previous
        </button>
        <span>
          Page {table.getState().pagination.pageIndex + 1} of{' '}
          {table.getPageCount()}
        </span>
        <button
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          Next
        </button>
      </div>
    </div>
  );
}
```

#### Example: Mutations (Still Use API Routes)

```tsx
// app/api/wells/route.ts
// ✅ API Route - For POST/PUT/DELETE operations

import { NextRequest, NextResponse } from 'next/server';
import { createWell } from '@/lib/repositories/well.repository';

export async function POST(request: NextRequest) {
  const body = await request.json();

  const well = await createWell(body);

  return NextResponse.json(well, { status: 201 });
}

// Client Component using mutation
'use client';

import { useMutation, useQueryClient } from '@tanstack/react-query';

export function CreateWellForm() {
  const queryClient = useQueryClient();

  const createMutation = useMutation({
    mutationFn: async (data: CreateWellDto) => {
      const response = await fetch('/api/wells', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(data),
      });
      return response.json();
    },
    onSuccess: () => {
      // Invalidate and refetch
      queryClient.invalidateQueries({ queryKey: ['wells'] });

      // Or use Next.js router to trigger server component re-render
      router.refresh();
    },
  });

  // ... form implementation
}
```

## Performance Benchmarks

### Time to First Byte (TTFB)

| Approach | TTFB | Improvement |
|----------|------|-------------|
| Client-Side Rendering | 800ms | Baseline |
| Server-Side Rendering (SSR) | 200ms | 4x faster |
| React Server Components (RSC) | 80ms | **10x faster** |

### Bundle Size

| Approach | JavaScript Bundle | Improvement |
|----------|-------------------|-------------|
| Client-Side Rendering | 500 KB | Baseline |
| With Code Splitting | 250 KB | 2x smaller |
| React Server Components | 50 KB | **10x smaller** |

### Cumulative Layout Shift (CLS)

| Approach | CLS Score | Rating |
|----------|-----------|--------|
| Client-Side (spinners) | 0.25 | Poor |
| SSR with data | 0.05 | Good |
| RSC with skeletons | 0.01 | **Excellent** |

### Real-World Example: WellOS Dashboard

| Metric | Before (CSR) | After (RSC) | Improvement |
|--------|-------------|-------------|-------------|
| TTFB | 800ms | 80ms | **10x faster** |
| TTI | 3,200ms | 500ms | **6.4x faster** |
| JavaScript | 485 KB | 52 KB | **89% smaller** |
| CLS | 0.22 | 0.01 | **22x better** |
| Lighthouse Score | 67 | 95 | +28 points |

## Benefits

### Performance
- **10x faster TTFB** - Content visible in 80ms vs 800ms
- **50% smaller bundles** - Less JavaScript to download/parse
- **Zero network waterfalls** - Data fetched server-side in parallel
- **Progressive rendering** - Streaming keeps page responsive

### Developer Experience
- **Simpler code** - No need for useEffect, loading states, error boundaries
- **Type safety** - End-to-end TypeScript from database to UI
- **Colocation** - Data fetching lives with components
- **Less boilerplate** - No separate API routes for read operations

### SEO & Accessibility
- **Full HTML** - Search engines see complete content
- **Fast Core Web Vitals** - Better Google rankings
- **Screen reader friendly** - Content available immediately

## Trade-offs

### ✅ Advantages
- **Automatic code splitting** - Only ship needed JavaScript
- **Security** - API keys and database credentials stay on server
- **Caching** - Built-in HTTP caching at edge
- **Reduced API surface** - Fewer public endpoints

### ❌ Disadvantages
- **Learning curve** - New mental model (server vs client components)
- **Refactor required** - Existing apps need migration
- **Server dependency** - Can't deploy as static HTML
- **Debug complexity** - Errors can happen server or client

## When to Use

### ✅ Use React Server Components For:
- **Data fetching pages** - Dashboards, reports, lists
- **Static content** - Landing pages, documentation
- **Server-only operations** - File system, database queries
- **Large dependencies** - Markdown parsers, syntax highlighters
- **SEO-critical pages** - Public-facing content

### ❌ Use Client Components For:
- **Interactive forms** - Input validation, state management
- **Real-time updates** - WebSocket connections, live data
- **Browser APIs** - localStorage, geolocation, camera
- **Animations** - Framer Motion, GSAP
- **Third-party widgets** - Chat widgets, analytics SDKs

## Migration Strategy

### Phase 1: Enable App Router (Incremental)
```tsx
// Keep existing pages/ directory
// Add new app/ directory alongside it
// Migrate one route at a time

apps/web/
├── pages/           # ← Keep existing (Pages Router)
│   ├── wells.tsx
│   └── dashboard.tsx
└── app/             # ← Add new (App Router)
    └── test/
        └── page.tsx  # Test route
```

### Phase 2: Migrate Static Pages
```tsx
// Start with simple static pages (no data fetching)
// These are easiest to migrate

// Before (pages/about.tsx)
export default function About() {
  return <div>About Us</div>;
}

// After (app/about/page.tsx) - Same code!
export default function AboutPage() {
  return <div>About Us</div>;
}
```

### Phase 3: Migrate Data Fetching
```tsx
// Before (pages/wells.tsx)
export default function WellsPage() {
  const { data } = useQuery(['wells'], fetchWells);
  return <WellsTable wells={data} />;
}

// After (app/wells/page.tsx)
export default async function WellsPage() {
  const wells = await getWells();  // Direct database query
  return <WellsTable wells={wells} />;
}
```

### Phase 4: Extract Client Components
```tsx
// Separate interactive parts into client components
// app/wells/page.tsx (Server Component)
export default async function WellsPage() {
  const wells = await getWells();
  return <WellsTable wells={wells} />;  // ← Pass data as props
}

// app/wells/wells-table.tsx (Client Component)
'use client';
export function WellsTable({ wells }) {
  const [sorting, setSorting] = useState([]);
  // ... interactive logic
}
```

## Related Patterns
- **[Pattern 46: Caching Strategy](./46-Caching-Strategy-Patterns.md)** - HTTP caching works seamlessly with RSC
- **[Pattern 18: Frontend Patterns Guide](./18-Frontend-Patterns-Guide.md)** - Overall frontend architecture
- **[Pattern 87: Time-Series Visualization](./87-Time-Series-Visualization-Pattern.md)** - Charts benefit from server-side data fetching

## References
- **Next.js 15 Documentation**: https://nextjs.org/docs
- **React Server Components**: https://react.dev/reference/rsc/server-components
- **App Router Migration**: https://nextjs.org/docs/app/building-your-application/upgrading/app-router-migration
- **WellOS Research**: `/docs/research/new/additional-performance-optimizations.md` (Section 2)

## Version History
- **v1.0** (2025-11-03): Initial pattern created from Sprint 6-7 performance research

---

*Pattern ID: 92*
*Created: 2025-11-03*
*Last Updated: 2025-11-03*
