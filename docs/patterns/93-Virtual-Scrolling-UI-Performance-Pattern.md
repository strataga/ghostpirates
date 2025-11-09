# Pattern 93: Virtual Scrolling UI Performance Pattern

**Status**: ✅ Recommended for Q2 2026
**Category**: Frontend Performance / UI Optimization / Large Datasets
**Related Patterns**: [87 - Time-Series Visualization](./87-Time-Series-Visualization-Pattern.md), [92 - React Server Components](./92-React-Server-Components-Pattern.md), [18 - Frontend Patterns Guide](./18-Frontend-Patterns-Guide.md)

## Problem

Rendering large lists or tables in the browser causes severe performance issues:

**Traditional Rendering Problems**:
- ❌ **Slow initial render** - Rendering 10,000 rows takes 5-15 seconds
- ❌ **High memory usage** - 10,000 DOM nodes = 500MB+ memory
- ❌ **Laggy scrolling** - Browser struggles with layout calculations
- ❌ **Poor mobile experience** - Crashes on devices with limited memory
- ❌ **Unusable UI** - Page becomes unresponsive during render

**Example: WellOS Production Data Table**:
```tsx
// ❌ Traditional Approach (Renders ALL rows)
export function ProductionTable({ data }: { data: ProductionRecord[] }) {
  return (
    <table>
      <tbody>
        {data.map(record => (  // 10,000 rows = 10,000 DOM nodes
          <tr key={record.id}>
            <td>{record.wellName}</td>
            <td>{record.date}</td>
            <td>{record.oilVolume}</td>
            <td>{record.gasVolume}</td>
            <td>{record.waterVolume}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}
```

**Performance Impact** (10,000 rows):
| Metric | Value | User Experience |
|--------|-------|-----------------|
| Initial render | 12 seconds | User sees blank screen |
| Memory usage | 580 MB | Mobile devices crash |
| Scroll FPS | 12 FPS | Janky, unusable scrolling |
| Time to Interactive | 18 seconds | Page frozen |

**Business Impact**:
- **Operators can't view data** - Page crashes or freezes
- **Slow decision-making** - 18-second wait to see production reports
- **Support tickets** - Users complain about "broken" dashboard
- **Mobile incompatibility** - iPad/iPhone users can't access data

## Context

WellOS displays large datasets in tables:
- **Production data**: 365 days × 100 wells = 36,500 rows
- **SCADA readings**: 86,400 readings/day × 10 days = 864,000 rows
- **Alarm history**: 1,000+ alarms across wells
- **Well lists**: 500+ wells with nested data

**Traditional Solutions**:
1. **Pagination** - Only show 20 rows at a time
   - ❌ Users lose context switching pages
   - ❌ Can't scroll through data naturally
   - ❌ Poor for data exploration

2. **Infinite scroll** - Load more on scroll
   - ❌ Still renders all previous rows (memory leak)
   - ❌ Scroll position jumps
   - ❌ Performance degrades over time

3. **"Load More" button** - Manual batch loading
   - ❌ Interrupts user flow
   - ❌ Still accumulates DOM nodes

## Solution

Use **virtual scrolling** (windowing) to render only visible rows.

### Key Concept

```
Physical Table (10,000 rows):              Virtual Viewport (Visible rows only):
┌─────────────────────────┐                ┌─────────────────────────┐
│ Row 1                   │ ← Not rendered │ [Empty space: 0-99]     │ ← Calculated height
│ Row 2                   │                │ Row 100                 │ ← DOM node
│ ...                     │                │ Row 101                 │ ← DOM node
│ Row 99                  │                │ Row 102                 │ ← DOM node
│ Row 100                 │ ← Rendered     │ ...                     │
│ Row 101                 │ ← Rendered     │ Row 115                 │ ← DOM node (last visible)
│ Row 102                 │ ← Rendered     │ [Empty space: 116-10K]  │ ← Calculated height
│ ...                     │                └─────────────────────────┘
│ Row 115                 │ ← Rendered              16 DOM nodes
│ Row 116                 │                      (not 10,000!)
│ ...                     │ ← Not rendered
│ Row 10,000              │
└─────────────────────────┘
    10,000 DOM nodes
```

**How It Works**:
1. Calculate total list height (e.g., 10,000 rows × 40px = 400,000px)
2. Determine visible rows based on scroll position
3. Render only visible rows + buffer (e.g., 20 visible + 10 buffer = 30 total)
4. Update on scroll (recycling DOM nodes)

**Result**: Constant 30 DOM nodes regardless of dataset size.

## Implementation

### 1. TanStack Virtual (Recommended)

**Best for**: React applications, flexible, modern API

```tsx
// Install: pnpm add @tanstack/react-virtual

import { useVirtualizer } from '@tanstack/react-virtual';
import { useRef } from 'react';

interface ProductionRecord {
  id: string;
  wellName: string;
  date: string;
  oilVolume: number;
  gasVolume: number;
  waterVolume: number;
}

export function VirtualizedProductionTable({
  data
}: {
  data: ProductionRecord[]
}) {
  const parentRef = useRef<HTMLDivElement>(null);

  // Create virtualizer instance
  const rowVirtualizer = useVirtualizer({
    count: data.length,           // Total number of rows
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,       // Estimated row height (px)
    overscan: 10,                 // Render 10 extra rows above/below viewport
  });

  return (
    <div
      ref={parentRef}
      className="h-screen overflow-auto"  // Fixed height + overflow
    >
      {/* Total height container */}
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,  // e.g., 400,000px
          width: '100%',
          position: 'relative',
        }}
      >
        {/* Render only visible rows */}
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const record = data[virtualRow.index];

          return (
            <div
              key={virtualRow.key}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,  // Position row
              }}
            >
              <div className="flex border-b">
                <div className="flex-1 p-2">{record.wellName}</div>
                <div className="flex-1 p-2">{record.date}</div>
                <div className="flex-1 p-2">{record.oilVolume}</div>
                <div className="flex-1 p-2">{record.gasVolume}</div>
                <div className="flex-1 p-2">{record.waterVolume}</div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

### 2. React Window (Lightweight Alternative)

**Best for**: Simple lists, smaller bundle size

```tsx
// Install: pnpm add react-window

import { FixedSizeList as List } from 'react-window';

export function ProductionTableReactWindow({ data }: { data: ProductionRecord[] }) {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const record = data[index];

    return (
      <div style={style} className="flex border-b">
        <div className="flex-1 p-2">{record.wellName}</div>
        <div className="flex-1 p-2">{record.date}</div>
        <div className="flex-1 p-2">{record.oilVolume}</div>
      </div>
    );
  };

  return (
    <List
      height={800}         // Viewport height
      itemCount={data.length}
      itemSize={40}        // Row height
      width="100%"
    >
      {Row}
    </List>
  );
}
```

### 3. Variable Row Heights

**For rows with dynamic content** (e.g., expanded rows, multi-line text):

```tsx
import { useVirtualizer } from '@tanstack/react-virtual';

export function VariableHeightTable({ data }: { data: Record[] }) {
  const parentRef = useRef<HTMLDivElement>(null);

  const rowVirtualizer = useVirtualizer({
    count: data.length,
    getScrollElement: () => parentRef.current,

    // Provide function to measure actual row height
    estimateSize: (index) => {
      const record = data[index];
      // Estimate based on content
      return record.expanded ? 200 : 40;  // Expanded rows are taller
    },

    // Enable dynamic measurement (slower but accurate)
    measureElement: (element) => {
      return element?.getBoundingClientRect().height ?? 40;
    },

    overscan: 5,
  });

  return (
    <div ref={parentRef} className="h-screen overflow-auto">
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          position: 'relative',
        }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const record = data[virtualRow.index];

          return (
            <div
              key={virtualRow.key}
              ref={rowVirtualizer.measureElement}  // Measure actual height
              data-index={virtualRow.index}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              {/* Row content with variable height */}
              <div className={record.expanded ? 'h-48' : 'h-10'}>
                {record.content}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

### 4. Virtual Table with TanStack Table

**Combine virtual scrolling with sorting, filtering, pagination**:

```tsx
import { useVirtualizer } from '@tanstack/react-virtual';
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  flexRender,
} from '@tanstack/react-table';

export function VirtualizedDataTable({ data, columns }) {
  const parentRef = useRef<HTMLDivElement>(null);

  // TanStack Table (sorting, filtering)
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });

  const { rows } = table.getRowModel();

  // TanStack Virtual (rendering)
  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,
    overscan: 10,
  });

  return (
    <div ref={parentRef} className="h-screen overflow-auto">
      <table className="w-full">
        {/* Table header */}
        <thead className="sticky top-0 bg-white z-10">
          {table.getHeaderGroups().map(headerGroup => (
            <tr key={headerGroup.id}>
              {headerGroup.headers.map(header => (
                <th
                  key={header.id}
                  onClick={header.column.getToggleSortingHandler()}
                  className="p-2 cursor-pointer"
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

        {/* Virtual table body */}
        <tbody
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map(virtualRow => {
            const row = rows[virtualRow.index];

            return (
              <tr
                key={row.id}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
              >
                {row.getVisibleCells().map(cell => (
                  <td key={cell.id} className="p-2">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
```

### 5. Infinite Loading with Virtual Scrolling

**For lazy-loading data as user scrolls**:

```tsx
import { useVirtualizer } from '@tanstack/react-virtual';
import { useInfiniteQuery } from '@tanstack/react-query';

export function InfiniteVirtualList() {
  const parentRef = useRef<HTMLDivElement>(null);

  // React Query infinite loading
  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
  } = useInfiniteQuery({
    queryKey: ['production'],
    queryFn: ({ pageParam = 0 }) => fetchProduction(pageParam),
    getNextPageParam: (lastPage, pages) => lastPage.nextCursor,
  });

  const allRows = data?.pages.flatMap(page => page.data) ?? [];

  const rowVirtualizer = useVirtualizer({
    count: hasNextPage ? allRows.length + 1 : allRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 40,
    overscan: 10,
  });

  // Fetch more when scrolled near end
  const items = rowVirtualizer.getVirtualItems();
  const lastItem = items[items.length - 1];

  useEffect(() => {
    if (
      lastItem &&
      lastItem.index >= allRows.length - 1 &&
      hasNextPage &&
      !isFetchingNextPage
    ) {
      fetchNextPage();
    }
  }, [lastItem, hasNextPage, fetchNextPage, isFetchingNextPage, allRows.length]);

  return (
    <div ref={parentRef} className="h-screen overflow-auto">
      <div
        style={{
          height: `${rowVirtualizer.getTotalSize()}px`,
          position: 'relative',
        }}
      >
        {rowVirtualizer.getVirtualItems().map((virtualRow) => {
          const isLoaderRow = virtualRow.index > allRows.length - 1;
          const record = allRows[virtualRow.index];

          return (
            <div
              key={virtualRow.key}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualRow.size}px`,
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              {isLoaderRow ? (
                hasNextPage ? 'Loading more...' : 'No more data'
              ) : (
                <div>{record.wellName} - {record.oilVolume}</div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
```

## Performance Benchmarks

### Render Performance

| Dataset Size | Traditional | Virtual Scrolling | Improvement |
|--------------|------------|------------------|-------------|
| 100 rows | 50ms | 20ms | 2.5x faster |
| 1,000 rows | 800ms | 25ms | **32x faster** |
| 10,000 rows | 12,000ms | 30ms | **400x faster** |
| 100,000 rows | Crashes | 35ms | **Usable!** |

### Memory Usage

| Dataset Size | Traditional | Virtual Scrolling | Savings |
|--------------|------------|------------------|---------|
| 1,000 rows | 58 MB | 2.5 MB | **96% less** |
| 10,000 rows | 580 MB | 2.8 MB | **99.5% less** |
| 100,000 rows | Crashes | 3.2 MB | **Prevents crash** |

### Scroll Performance

| Metric | Traditional (10K rows) | Virtual Scrolling | Improvement |
|--------|----------------------|------------------|-------------|
| FPS | 12 FPS | 60 FPS | **5x smoother** |
| Frame Time | 83ms | 16ms | **5x faster** |
| Jank | Constant | None | Perfect |

## Benefits

### Performance
- **400x faster rendering** - 30ms vs 12 seconds for 10,000 rows
- **99.5% less memory** - 2.8MB vs 580MB
- **60 FPS scrolling** - Buttery smooth on all devices
- **No crashes** - Works with 100,000+ rows

### User Experience
- **Instant page load** - No blank screens or spinners
- **Smooth scrolling** - Feels like native app
- **Mobile-friendly** - Works on iPad/iPhone
- **Large datasets** - View entire year of data

### Developer Experience
- **Drop-in replacement** - Minimal code changes
- **Flexible** - Works with any data structure
- **TypeScript support** - Full type safety
- **Battle-tested** - Used by Google, Atlassian, Meta

## Trade-offs

### ✅ Advantages
- **Constant performance** - 30ms render regardless of dataset size
- **Low memory** - ~3MB for any dataset
- **Smooth scrolling** - 60 FPS on all devices
- **Easy integration** - Works with existing React code

### ❌ Disadvantages
- **Fixed container height** - Requires known viewport height
- **Absolute positioning** - Can complicate CSS layouts
- **Accessibility** - Screen readers may struggle with virtualization
- **Find-in-page** - Browser Ctrl+F only finds visible rows

## When to Use

### ✅ Use Virtual Scrolling When:
- **Large datasets** - >500 rows
- **Performance critical** - User expects instant render
- **Mobile support** - iPad/iPhone users
- **Real-time data** - Continuous updates (SCADA, logs)
- **Memory constrained** - Embedded devices, low-end phones

### ❌ Don't Use Virtual Scrolling When:
- **Small datasets** - <100 rows (traditional rendering is fine)
- **Variable heights** - Complex layouts with dynamic content
- **Accessibility critical** - Screen reader users (provide alternative)
- **Print-friendly** - User needs to print full table
- **SEO required** - Content must be in initial HTML (use server rendering)

## Library Comparison

| Library | Bundle Size | Features | Best For |
|---------|------------|----------|----------|
| **@tanstack/react-virtual** | 5.8 KB | Variable heights, grid, sticky headers | Modern apps, complex layouts |
| **react-window** | 3.2 KB | Fixed heights, simple API | Simple lists, small bundle |
| **react-virtualized** | 27 KB | Feature-rich, legacy | Legacy apps (deprecated) |
| **react-virtuoso** | 9.5 KB | Smart scrolling, dynamic heights | Complex virtualization |

**Recommendation**: Use **@tanstack/react-virtual** (best modern API, maintained by TanStack team).

## Accessibility Considerations

### 1. Provide Alternative View

```tsx
export function AccessibleProductionTable({ data }) {
  const [useVirtual, setUseVirtual] = useState(true);

  return (
    <div>
      <button onClick={() => setUseVirtual(!useVirtual)}>
        {useVirtual ? 'Switch to Standard View' : 'Switch to Virtual View'}
      </button>

      {useVirtual ? (
        <VirtualizedTable data={data} />
      ) : (
        <StandardTable data={data.slice(0, 100)} />  // Limit rows
      )}
    </div>
  );
}
```

### 2. ARIA Attributes

```tsx
<div
  role="table"
  aria-label="Production data table"
  aria-rowcount={data.length}
>
  {virtualRows.map(virtualRow => (
    <div
      role="row"
      aria-rowindex={virtualRow.index + 1}
      key={virtualRow.key}
    >
      {/* Row content */}
    </div>
  ))}
</div>
```

### 3. Keyboard Navigation

```tsx
const handleKeyDown = (e: KeyboardEvent) => {
  if (e.key === 'ArrowDown') {
    // Scroll to next row
    rowVirtualizer.scrollToIndex(currentIndex + 1);
  } else if (e.key === 'ArrowUp') {
    // Scroll to previous row
    rowVirtualizer.scrollToIndex(currentIndex - 1);
  }
};
```

## Related Patterns
- **[Pattern 87: Time-Series Visualization](./87-Time-Series-Visualization-Pattern.md)** - Uses virtual scrolling for chart legends
- **[Pattern 92: React Server Components](./92-React-Server-Components-Pattern.md)** - Combine with RSC for optimal data loading
- **[Pattern 82: Hybrid Time-Series Aggregation](./82-Hybrid-Time-Series-Aggregation-Pattern.md)** - Backend pagination for virtual scrolling

## References
- **TanStack Virtual**: https://tanstack.com/virtual/latest
- **React Window**: https://github.com/bvaughn/react-window
- **Performance Guide**: https://web.dev/virtualize-long-lists-react-window/
- **WellOS Research**: `/docs/research/new/additional-performance-optimizations.md` (Section 7)

## Version History
- **v1.0** (2025-11-03): Initial pattern created from Sprint 6-7 performance research

---

*Pattern ID: 93*
*Created: 2025-11-03*
*Last Updated: 2025-11-03*
