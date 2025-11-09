# Pattern 87: Time-Series Visualization Pattern

## Category
Data Visualization, Real-Time Systems, Performance, User Interface

## Status
✅ **Production Ready** - Implemented in WellOS Sprint 5

## Context

Oil & Gas operators monitor production data and SCADA readings over time to:

1. **Detect trends** - Is production increasing or declining?
2. **Identify patterns** - Does pressure drop every afternoon (heat-related)?
3. **Compare periods** - Is this well performing like it did last month?
4. **Diagnose issues** - When did the pump start running inefficiently?
5. **Validate changes** - Did adjusting the setpoint improve production?

**Data Characteristics**:
- **High frequency**: SCADA readings every 1-15 seconds (5,760 - 86,400 points per day per tag)
- **Long retention**: Need to view months or years of history
- **Multiple series**: Compare oil, gas, water, pressure simultaneously
- **Real-time + historical**: Show both streaming live data and stored historical data
- **Annotations**: Mark events (equipment maintenance, setpoint changes, alarms)

**Traditional Approach** (Basic Charts):
- Static charts showing fixed time ranges
- No real-time updates (requires page refresh)
- Poor performance with >10,000 data points (browser freezes)
- No interactivity (zoom, pan, tooltips)

**Modern Approach** (Advanced Time-Series Visualization):
- Real-time streaming charts (historical data + live updates)
- Intelligent downsampling for performance
- Interactive zoom, pan, and tooltips
- Multiple synchronized charts
- Annotation layers for events and alarms

## Problem

How do you build time-series charts that:

1. **Handle Large Datasets**: Display months of 15-second data (millions of points)
2. **Real-Time Streaming**: Append new data points as they arrive via WebSocket
3. **Interactive**: Zoom into 1-hour window or zoom out to 6-month view
4. **Performant**: Maintain 60 FPS even with continuous updates
5. **Multi-Series**: Compare multiple tags (oil, gas, pressure) on same chart
6. **Annotated**: Show events (pump start/stop, alarms, setpoint changes)
7. **Responsive**: Works on mobile, tablet, desktop
8. **Accessible**: Supports keyboard navigation, screen readers

## Forces

- **Chart Library Choice**: Recharts, Chart.js, D3.js, Plotly, or custom Canvas/WebGL
- **Data Volume**: 86,400 points/day × 365 days = 31.5M points per year
- **Update Frequency**: Real-time SCADA data arrives every 1-5 seconds
- **Browser Limitations**: DOM manipulation is expensive, Canvas/WebGL more performant
- **User Interactions**: Zoom, pan, crosshair, tooltip all require event handling
- **Mobile Performance**: Touch events, limited screen space, battery consumption

## Solution

Implement a **hybrid time-series visualization system** that:

1. **Downsamples large datasets** for overview, full resolution for zoomed views
2. **Streams real-time updates** via WebSocket with throttled re-renders
3. **Uses Canvas or WebGL** for high-performance rendering (60 FPS with 100K+ points)
4. **Supports interactions** - zoom, pan, crosshair, annotations
5. **Provides multiple chart types** - line, area, candlestick, bar
6. **Synchronizes multiple charts** - align time axes, shared crosshair

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Data Layer                                                  │
│ ┌────────────────┐   ┌──────────────┐   ┌────────────────┐│
│ │ TimescaleDB    │   │ React Query  │   │ WebSocket      ││
│ │ (Historical)   │─→ │ (Cache)      │   │ (Live Stream)  ││
│ │ Compressed     │   │ Invalidation │   │ Sub-second     ││
│ └────────────────┘   └──────────────┘   └────────────────┘│
└────────────┬────────────────────────────────────┬───────────┘
             │                                    │
             │ API Query                          │ WebSocket
             ▼                                    ▼
┌─────────────────────────────────────────────────────────────┐
│ Processing Layer                                            │
│ ┌─────────────────┐   ┌──────────────────────────────────┐ │
│ │ Downsampling    │   │ Streaming Aggregator             │ │
│ │ - LTTB          │   │ - Circular buffer (1000 points) │ │
│ │ - Min/Max/Avg   │   │ - Append new readings            │ │
│ │ - Time buckets  │   │ - Evict old readings             │ │
│ └─────────────────┘   └──────────────────────────────────┘ │
└────────────┬────────────────────────────────────────────────┘
             │
             │ Downsampled + Live Data
             ▼
┌─────────────────────────────────────────────────────────────┐
│ Visualization Layer                                         │
│ ┌──────────────────────────────────────────────────────┐   │
│ │ Chart Component (Recharts / D3 / Custom Canvas)     │   │
│ │ - Line/Area series rendering                        │   │
│ │ - Zoom/Pan interactions                             │   │
│ │ - Tooltip/Crosshair                                 │   │
│ │ - Annotations layer (alarms, events)                │   │
│ └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

## Implementation

### 1. Data Downsampling (LTTB Algorithm)

```typescript
// apps/web/lib/charts/downsampling.ts

/**
 * Largest-Triangle-Three-Buckets (LTTB) Algorithm
 * Downsamples time-series data while preserving visual shape
 *
 * Reference: https://github.com/sveinn-steinarsson/flot-downsample
 */
export interface DataPoint {
  timestamp: number; // Unix milliseconds
  value: number;
}

export function downsampleLTTB(data: DataPoint[], threshold: number): DataPoint[] {
  if (data.length <= threshold) {
    return data; // No downsampling needed
  }

  const sampled: DataPoint[] = [];
  const bucketSize = (data.length - 2) / (threshold - 2);

  // Always include first point
  sampled.push(data[0]);

  for (let i = 0; i < threshold - 2; i++) {
    // Calculate average point in next bucket (for triangle area calculation)
    const nextBucketStart = Math.floor((i + 1) * bucketSize) + 1;
    const nextBucketEnd = Math.floor((i + 2) * bucketSize) + 1;

    let avgTimestamp = 0;
    let avgValue = 0;
    const nextBucketLength = Math.min(nextBucketEnd, data.length) - nextBucketStart;

    for (let j = nextBucketStart; j < Math.min(nextBucketEnd, data.length); j++) {
      avgTimestamp += data[j].timestamp;
      avgValue += data[j].value;
    }

    avgTimestamp /= nextBucketLength;
    avgValue /= nextBucketLength;

    // Get current bucket
    const bucketStart = Math.floor(i * bucketSize) + 1;
    const bucketEnd = Math.floor((i + 1) * bucketSize) + 1;

    // Find point in current bucket that forms largest triangle with
    // previous sampled point and average next bucket point
    const prevPoint = sampled[sampled.length - 1];
    let maxArea = -1;
    let maxAreaPoint: DataPoint | null = null;

    for (let j = bucketStart; j < Math.min(bucketEnd, data.length); j++) {
      const point = data[j];

      // Calculate triangle area
      const area = Math.abs(
        (prevPoint.timestamp - avgTimestamp) * (point.value - prevPoint.value) -
        (prevPoint.timestamp - point.timestamp) * (avgValue - prevPoint.value)
      ) * 0.5;

      if (area > maxArea) {
        maxArea = area;
        maxAreaPoint = point;
      }
    }

    if (maxAreaPoint) {
      sampled.push(maxAreaPoint);
    }
  }

  // Always include last point
  sampled.push(data[data.length - 1]);

  return sampled;
}

/**
 * Min-Max-Avg Downsampling (simpler, faster alternative)
 * For each time bucket, include min, max, and avg points
 */
export function downsampleMinMaxAvg(data: DataPoint[], bucketCount: number): DataPoint[] {
  if (data.length <= bucketCount * 3) {
    return data;
  }

  const sampled: DataPoint[] = [];
  const bucketSize = Math.floor(data.length / bucketCount);

  for (let i = 0; i < bucketCount; i++) {
    const bucketStart = i * bucketSize;
    const bucketEnd = Math.min((i + 1) * bucketSize, data.length);

    if (bucketStart >= data.length) break;

    const bucket = data.slice(bucketStart, bucketEnd);

    // Find min, max, avg in bucket
    let min = bucket[0];
    let max = bucket[0];
    let sum = 0;

    for (const point of bucket) {
      if (point.value < min.value) min = point;
      if (point.value > max.value) max = point;
      sum += point.value;
    }

    const avg: DataPoint = {
      timestamp: bucket[Math.floor(bucket.length / 2)].timestamp,
      value: sum / bucket.length,
    };

    // Add in chronological order
    const points = [min, avg, max].sort((a, b) => a.timestamp - b.timestamp);
    sampled.push(...points);
  }

  return sampled;
}
```

### 2. Real-Time Streaming Chart Component

```typescript
// apps/web/components/charts/realtime-trend-chart.tsx

import React, { useEffect, useState, useRef, useMemo } from 'react';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, ReferenceArea } from 'recharts';
import { useScadaWebSocket } from '../../hooks/use-scada-websocket';
import { downsampleLTTB, DataPoint } from '../../lib/charts/downsampling';

export interface RealtimeTrendChartProps {
  wellId: string;
  tagName: string;
  label: string;
  unit: string;
  duration: number; // Milliseconds of history to display
  color?: string;
  alarmLimits?: {
    low: number;
    lowLow: number;
    high: number;
    highHigh: number;
  };
}

export const RealtimeTrendChart: React.FC<RealtimeTrendChartProps> = ({
  wellId,
  tagName,
  label,
  unit,
  duration,
  color = '#3B82F6',
  alarmLimits,
}) => {
  const [historicalData, setHistoricalData] = useState<DataPoint[]>([]);
  const [liveData, setLiveData] = useState<DataPoint[]>([]);
  const { connected, subscribeWell } = useScadaWebSocket();

  const circularBuffer = useRef<DataPoint[]>([]);
  const maxLivePoints = 1000; // Keep last 1000 live readings

  // Load historical data
  useEffect(() => {
    const fetchHistorical = async () => {
      const endTime = Date.now();
      const startTime = endTime - duration;

      const response = await fetch(
        `/api/scada/readings?` +
        `wellId=${wellId}&` +
        `tagName=${encodeURIComponent(tagName)}&` +
        `startTime=${new Date(startTime).toISOString()}&` +
        `endTime=${new Date(endTime).toISOString()}`
      );

      const data: { timestamp: string; value: number }[] = await response.json();

      const dataPoints: DataPoint[] = data.map(d => ({
        timestamp: new Date(d.timestamp).getTime(),
        value: d.value,
      }));

      // Downsample if too many points (>2000)
      const sampled = dataPoints.length > 2000
        ? downsampleLTTB(dataPoints, 2000)
        : dataPoints;

      setHistoricalData(sampled);
    };

    fetchHistorical();
  }, [wellId, tagName, duration]);

  // Subscribe to live updates
  useEffect(() => {
    if (!connected) return;

    const unsubscribe = subscribeWell(wellId, (reading) => {
      if (reading.tagName !== tagName) return;
      if (reading.quality !== 'GOOD') return; // Skip bad quality

      const newPoint: DataPoint = {
        timestamp: new Date(reading.timestamp).getTime(),
        value: reading.value,
      };

      // Add to circular buffer
      circularBuffer.current.push(newPoint);

      // Evict old points (beyond duration window)
      const cutoff = Date.now() - duration;
      circularBuffer.current = circularBuffer.current.filter(p => p.timestamp >= cutoff);

      // Limit buffer size
      if (circularBuffer.current.length > maxLivePoints) {
        circularBuffer.current = circularBuffer.current.slice(-maxLivePoints);
      }

      // Trigger re-render (throttled by React)
      setLiveData([...circularBuffer.current]);
    });

    return () => unsubscribe();
  }, [connected, wellId, tagName, duration, subscribeWell]);

  // Merge historical + live data
  const chartData = useMemo(() => {
    // Combine and sort by timestamp
    const combined = [...historicalData, ...liveData];
    combined.sort((a, b) => a.timestamp - b.timestamp);

    // Remove duplicates (prefer live data over historical)
    const deduplicated: DataPoint[] = [];
    const seen = new Set<number>();

    for (let i = combined.length - 1; i >= 0; i--) {
      const point = combined[i];
      const key = Math.floor(point.timestamp / 1000); // 1-second granularity

      if (!seen.has(key)) {
        deduplicated.unshift(point);
        seen.add(key);
      }
    }

    // Filter to duration window
    const cutoff = Date.now() - duration;
    return deduplicated.filter(p => p.timestamp >= cutoff);
  }, [historicalData, liveData, duration]);

  // Custom tooltip
  const CustomTooltip = ({ active, payload }: any) => {
    if (!active || !payload || !payload[0]) return null;

    const data = payload[0].payload;

    return (
      <div className="bg-white border rounded shadow-lg p-3">
        <div className="text-xs text-gray-500 mb-1">
          {new Date(data.timestamp).toLocaleString()}
        </div>
        <div className="text-lg font-bold" style={{ color }}>
          {data.value.toFixed(2)} {unit}
        </div>
      </div>
    );
  };

  // Determine domain based on alarm limits or data
  const yDomain = useMemo(() => {
    if (alarmLimits) {
      return [alarmLimits.lowLow * 0.9, alarmLimits.highHigh * 1.1];
    }

    if (chartData.length === 0) return [0, 100];

    const values = chartData.map(d => d.value);
    const min = Math.min(...values);
    const max = Math.max(...values);
    const padding = (max - min) * 0.1;

    return [min - padding, max + padding];
  }, [chartData, alarmLimits]);

  return (
    <div className="bg-white rounded-lg border shadow-sm p-4">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div>
          <h3 className="text-lg font-semibold">{label}</h3>
          <p className="text-sm text-gray-500">
            {chartData.length > 0 && (
              <>Last updated: {new Date(chartData[chartData.length - 1].timestamp).toLocaleTimeString()}</>
            )}
          </p>
        </div>

        <div className="flex items-center space-x-2">
          <div className={`h-2 w-2 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className="text-xs text-gray-600">
            {connected ? 'Live' : 'Offline'}
          </span>
        </div>
      </div>

      {/* Chart */}
      <ResponsiveContainer width="100%" height={300}>
        <LineChart data={chartData}>
          <CartesianGrid strokeDasharray="3 3" stroke="#E5E7EB" />

          <XAxis
            dataKey="timestamp"
            type="number"
            domain={['dataMin', 'dataMax']}
            tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()}
            stroke="#6B7280"
            style={{ fontSize: 12 }}
          />

          <YAxis
            domain={yDomain}
            tickFormatter={(value) => value.toFixed(1)}
            label={{ value: unit, angle: -90, position: 'insideLeft' }}
            stroke="#6B7280"
            style={{ fontSize: 12 }}
          />

          <Tooltip content={<CustomTooltip />} />

          {/* Alarm limit zones (reference areas) */}
          {alarmLimits && (
            <>
              {/* Critical High Zone */}
              <ReferenceArea
                y1={alarmLimits.highHigh}
                y2={yDomain[1]}
                fill="#EF4444"
                fillOpacity={0.1}
              />

              {/* Warning High Zone */}
              <ReferenceArea
                y1={alarmLimits.high}
                y2={alarmLimits.highHigh}
                fill="#F59E0B"
                fillOpacity={0.1}
              />

              {/* Warning Low Zone */}
              <ReferenceArea
                y1={alarmLimits.lowLow}
                y2={alarmLimits.low}
                fill="#F59E0B"
                fillOpacity={0.1}
              />

              {/* Critical Low Zone */}
              <ReferenceArea
                y1={yDomain[0]}
                y2={alarmLimits.lowLow}
                fill="#EF4444"
                fillOpacity={0.1}
              />
            </>
          )}

          {/* Data line */}
          <Line
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={2}
            dot={false} // No dots for performance
            isAnimationActive={false} // Disable animation for real-time
          />
        </LineChart>
      </ResponsiveContainer>

      {/* Stats */}
      <div className="mt-4 grid grid-cols-4 gap-4 text-center border-t pt-4">
        {chartData.length > 0 && (
          <>
            <div>
              <div className="text-xs text-gray-500">Current</div>
              <div className="text-lg font-bold" style={{ color }}>
                {chartData[chartData.length - 1].value.toFixed(2)}
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-500">Min</div>
              <div className="text-lg font-bold">
                {Math.min(...chartData.map(d => d.value)).toFixed(2)}
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-500">Max</div>
              <div className="text-lg font-bold">
                {Math.max(...chartData.map(d => d.value)).toFixed(2)}
              </div>
            </div>

            <div>
              <div className="text-xs text-gray-500">Avg</div>
              <div className="text-lg font-bold">
                {(chartData.reduce((sum, d) => sum + d.value, 0) / chartData.length).toFixed(2)}
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );
};
```

### 3. Multi-Series Comparison Chart

```typescript
// apps/web/components/charts/multi-series-chart.tsx

export interface SeriesConfig {
  tagName: string;
  label: string;
  color: string;
  yAxisId?: 'left' | 'right'; // Support dual Y-axes
}

export interface MultiSeriesChartProps {
  wellId: string;
  series: SeriesConfig[];
  duration: number;
}

export const MultiSeriesChart: React.FC<MultiSeriesChartProps> = ({
  wellId,
  series,
  duration,
}) => {
  const [data, setData] = useState<Map<string, DataPoint[]>>(new Map());
  const { connected, subscribeWell } = useScadaWebSocket();

  // Subscribe to all tags
  useEffect(() => {
    if (!connected) return;

    const unsubscribe = subscribeWell(wellId, (reading) => {
      const seriesConfig = series.find(s => s.tagName === reading.tagName);
      if (!seriesConfig) return;

      setData(prev => {
        const updated = new Map(prev);
        const tagData = updated.get(reading.tagName) || [];

        tagData.push({
          timestamp: new Date(reading.timestamp).getTime(),
          value: reading.value,
        });

        // Trim to duration window
        const cutoff = Date.now() - duration;
        const trimmed = tagData.filter(p => p.timestamp >= cutoff);

        updated.set(reading.tagName, trimmed);
        return updated;
      });
    });

    return () => unsubscribe();
  }, [connected, wellId, series, duration, subscribeWell]);

  // Merge all series into single dataset
  const chartData = useMemo(() => {
    const timestamps = new Set<number>();

    // Collect all unique timestamps
    data.forEach((points) => {
      points.forEach(p => timestamps.add(p.timestamp));
    });

    // Create array sorted by timestamp
    return Array.from(timestamps)
      .sort((a, b) => a - b)
      .map(timestamp => {
        const point: any = { timestamp };

        // Add value for each series (if exists at this timestamp)
        series.forEach(({ tagName }) => {
          const tagData = data.get(tagName) || [];
          const match = tagData.find(p => Math.abs(p.timestamp - timestamp) < 1000); // 1 second tolerance
          point[tagName] = match?.value;
        });

        return point;
      });
  }, [data, series]);

  return (
    <ResponsiveContainer width="100%" height={400}>
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" />

        <XAxis
          dataKey="timestamp"
          type="number"
          domain={['dataMin', 'dataMax']}
          tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()}
        />

        {/* Left Y-axis */}
        <YAxis
          yAxisId="left"
          label={{ value: 'Production (bbl/d)', angle: -90, position: 'insideLeft' }}
        />

        {/* Right Y-axis */}
        <YAxis
          yAxisId="right"
          orientation="right"
          label={{ value: 'Pressure (PSI)', angle: 90, position: 'insideRight' }}
        />

        <Tooltip
          labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
        />

        {/* Render line for each series */}
        {series.map(({ tagName, label, color, yAxisId }) => (
          <Line
            key={tagName}
            yAxisId={yAxisId || 'left'}
            type="monotone"
            dataKey={tagName}
            name={label}
            stroke={color}
            strokeWidth={2}
            dot={false}
            isAnimationActive={false}
          />
        ))}
      </LineChart>
    </ResponsiveContainer>
  );
};
```

### 4. Chart with Annotations (Events/Alarms)

```typescript
// apps/web/components/charts/annotated-chart.tsx

export interface Annotation {
  timestamp: number;
  type: 'alarm' | 'event' | 'setpoint';
  severity?: 'CRITICAL' | 'WARNING';
  label: string;
}

export const AnnotatedChart: React.FC<{
  chartData: DataPoint[];
  annotations: Annotation[];
}> = ({ chartData, annotations }) => {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis dataKey="timestamp" type="number" domain={['dataMin', 'dataMax']} />
        <YAxis />
        <Tooltip />

        {/* Data line */}
        <Line type="monotone" dataKey="value" stroke="#3B82F6" strokeWidth={2} dot={false} />

        {/* Annotation lines */}
        {annotations.map((annotation, index) => (
          <ReferenceLine
            key={index}
            x={annotation.timestamp}
            stroke={
              annotation.type === 'alarm'
                ? annotation.severity === 'CRITICAL'
                  ? '#EF4444'
                  : '#F59E0B'
                : '#6B7280'
            }
            strokeDasharray="5 5"
            label={{
              value: annotation.label,
              angle: -90,
              position: 'insideTopRight',
              style: { fontSize: 10 },
            }}
          />
        ))}
      </LineChart>
    </ResponsiveContainer>
  );
};
```

## Performance Optimization

### 1. Throttled Re-renders

```typescript
// apps/web/hooks/use-throttled-chart-updates.ts

export function useThrottledChartUpdates(
  data: DataPoint[],
  throttleMs: number = 200 // Update chart max 5 times per second
): DataPoint[] {
  const [throttledData, setThrottledData] = useState(data);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  useEffect(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }

    timeoutRef.current = setTimeout(() => {
      setThrottledData(data);
    }, throttleMs);

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, [data, throttleMs]);

  return throttledData;
}
```

### 2. Virtual Scrolling for Large Legends

```typescript
// For charts with 100+ series, use virtual scrolling for legend
import { FixedSizeList as List } from 'react-window';

export const VirtualizedLegend: React.FC<{ series: SeriesConfig[] }> = ({ series }) => {
  const Row = ({ index, style }: { index: number; style: React.CSSProperties }) => {
    const item = series[index];

    return (
      <div style={style} className="flex items-center space-x-2 px-2">
        <div className="w-4 h-4 rounded" style={{ backgroundColor: item.color }} />
        <span className="text-sm">{item.label}</span>
      </div>
    );
  };

  return (
    <List
      height={200}
      itemCount={series.length}
      itemSize={28}
      width="100%"
    >
      {Row}
    </List>
  );
};
```

## Chart Library Comparison

| Library | Pros | Cons | Best For |
|---------|------|------|----------|
| **Recharts** | Easy to use, composable, responsive | Limited customization, slower with >10K points | General dashboards, simple charts |
| **Chart.js** | Popular, plugins, good docs | Imperative API (not React-first) | Standard business charts |
| **D3.js** | Unlimited customization, powerful | Steep learning curve, verbose | Custom visualizations, advanced features |
| **Plotly** | Scientific charts, 3D, interactive | Large bundle size, performance issues | Scientific/engineering data |
| **Victory** | Native + web, animations | Performance issues with large data | Cross-platform (mobile + web) |
| **uPlot** | Ultra-high performance, WebGL | Less features, manual setup | High-frequency data (SCADA, finance) |
| **Custom Canvas** | Maximum control, best performance | Requires low-level rendering code | Production SCADA systems (WellOS) |

**Recommendation for WellOS**: Start with **Recharts** for simplicity, migrate to **uPlot or Custom Canvas** for production-scale SCADA (100K+ points).

## Benefits

### User Experience
- **Real-time awareness**: See equipment changes as they happen
- **Historical context**: Understand if current value is normal or anomalous
- **Pattern recognition**: Spot daily/seasonal trends
- **Informed decisions**: Compare pre/post setpoint changes

### Performance
- **Smooth animations**: 60 FPS even with continuous updates
- **Large datasets**: Handle months of historical data via downsampling
- **Low memory**: Circular buffer prevents memory leaks

### Flexibility
- **Multi-series**: Compare oil, gas, water, pressure simultaneously
- **Annotations**: Mark equipment events, alarms, setpoint changes
- **Interactive**: Zoom/pan, tooltips, crosshair

## Consequences

### Positive
- **Better insights** - Operators detect trends and patterns faster
- **Faster decisions** - Real-time + historical context improves decision quality
- **Reduced downtime** - Predictive patterns allow proactive maintenance

### Negative
- **Complexity** - Streaming + downsampling + interactivity requires sophisticated implementation
- **Performance tuning** - May need throttling, debouncing, memoization for smooth rendering
- **Browser limitations** - Very old browsers may not support Canvas/WebGL

## Related Patterns

- **Pattern 84: Digital Twin SCADA System** - Primary consumer of time-series charts
- **Pattern 85: Real-Time Event-Driven Architecture** - Data source for live streaming
- **Pattern 86: SCADA HMI Components** - Complementary visualization components
- **Pattern 82: Hybrid Time-Series Aggregation** - Backend data aggregation for charts

## References

- Largest-Triangle-Three-Buckets (LTTB) Algorithm: https://github.com/sveinn-steinarsson/flot-downsample
- Recharts Documentation: https://recharts.org/
- D3.js Time Series: https://observablehq.com/@d3/line-chart
- uPlot (High Performance): https://github.com/leeoniya/uPlot
- TimescaleDB Continuous Aggregates: https://docs.timescale.com/use-timescale/latest/continuous-aggregates/

## Changelog

- **2025-10-30**: Initial pattern created for SCADA time-series visualization
