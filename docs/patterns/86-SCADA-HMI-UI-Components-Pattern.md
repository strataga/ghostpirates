# Pattern 86: SCADA HMI/UI Components Pattern

## Category
User Interface, Industrial IoT, Visualization, Component Library

## Status
✅ **Production Ready** - Implemented in WellOS Sprint 5

## Context

Field personnel in Oil & Gas are trained on physical control panels (HMI - Human-Machine Interface) at well sites. These panels feature:

- **Circular gauges** with colored zones (green = normal, amber = warning, red = critical)
- **Tank level indicators** showing fluid levels
- **P&ID diagrams** (Piping and Instrumentation Diagrams) with real-time values
- **Alarm lights** that flash when limits are exceeded
- **Physical buttons and switches** for control

When transitioning to digital monitoring (web/mobile), operators need familiar visual language to reduce training time and increase confidence in the system.

**Traditional Approach**:
- Generic charts and tables that don't match physical panels
- **Problems**:
  - Field personnel don't recognize interface (training required)
  - Critical information harder to spot (no color-coded zones)
  - No spatial context (unlike P&ID diagrams showing equipment relationships)

**Modern Approach** (SCADA HMI Components):
- React component library that replicates physical HMI panels
- Familiar gauges, tanks, P&IDs that operators already know
- Real-time updates via WebSocket
- Responsive design for tablets/phones

## Problem

How do you build a reusable UI component library for SCADA systems that:

1. **Matches Physical Interfaces**: Looks like the actual control panels at well sites
2. **Real-Time Updates**: Reflects live SCADA data with smooth animations
3. **Accessibility**: Works on desktop, tablet, and mobile
4. **Performance**: Handles 100+ gauges updating 1-5 times per second
5. **Customizable**: Operators configure alarm limits, colors, units per well
6. **Composable**: Components combine to build complex dashboards
7. **Type-Safe**: Full TypeScript support with proper interfaces

## Forces

- **Visual Fidelity**: Must match physical HMI panels for user acceptance
- **Performance**: Animations and updates must be smooth (60 FPS)
- **Data Binding**: Components must react to real-time SCADA readings
- **Customization**: Different wells have different operating ranges
- **Responsive Design**: Same component works on 27" monitor and 10" tablet
- **Framework Choice**: React, Vue, Angular, or framework-agnostic Web Components
- **Styling**: CSS-in-JS, Tailwind, or traditional CSS modules

## Solution

Build a **comprehensive SCADA HMI component library** with:

1. **Gauge Components** - Circular, linear, tank, radial
2. **Indicator Components** - Status lights, alarm banners, quality badges
3. **Chart Components** - Real-time trends, historical comparison
4. **Diagram Components** - P&ID, 3D well visualization, equipment layout
5. **Control Components** - Setpoint sliders, on/off switches, emergency stop
6. **Panel Components** - Alarm list, tag browser, event log

All components:
- Accept real-time SCADA readings via props
- Support customizable alarm limits and visual thresholds
- Render using SVG (scalable, performant)
- Follow industrial color standards (ISA-101 / ANSI/ISA-5.1)

## Implementation

### 1. Circular Gauge Component

```typescript
// apps/web/components/scada-hmi/circular-gauge.tsx

import React, { useMemo } from 'react';

export interface CircularGaugeProps {
  /** Display label */
  label: string;

  /** Current value */
  value: number;

  /** Minimum value on scale */
  min: number;

  /** Maximum value on scale */
  max: number;

  /** Unit of measurement */
  unit: string;

  /** Data quality indicator */
  quality: 'GOOD' | 'BAD' | 'UNCERTAIN';

  /** Alarm thresholds (optional) */
  alarmLimits?: {
    low: number;
    lowLow: number;
    high: number;
    highHigh: number;
  };

  /** Component size */
  size?: 'sm' | 'md' | 'lg' | 'xl';

  /** Color theme */
  theme?: 'light' | 'dark';

  /** Decimal places for display */
  precision?: number;

  /** Show min/max labels */
  showMinMax?: boolean;
}

export const CircularGauge: React.FC<CircularGaugeProps> = ({
  label,
  value,
  min,
  max,
  unit,
  quality,
  alarmLimits,
  size = 'md',
  theme = 'light',
  precision = 1,
  showMinMax = true,
}) => {
  // Size configuration
  const sizeConfig = {
    sm: { diameter: 120, strokeWidth: 12, fontSize: { value: 18, label: 10, unit: 9 } },
    md: { diameter: 180, strokeWidth: 18, fontSize: { value: 24, label: 12, unit: 10 } },
    lg: { diameter: 240, strokeWidth: 24, fontSize: { value: 32, label: 14, unit: 12 } },
    xl: { diameter: 320, strokeWidth: 32, fontSize: { value: 42, label: 16, unit: 14 } },
  };

  const config = sizeConfig[size];
  const radius = config.diameter / 2;
  const innerRadius = radius - config.strokeWidth;
  const centerX = radius;
  const centerY = radius;

  // Arc configuration (270° arc, -135° to +135°)
  const arcStartAngle = -135;
  const arcEndAngle = 135;
  const arcSpan = arcEndAngle - arcStartAngle;

  // Calculate value percentage and angle
  const percentage = useMemo(() => {
    return ((value - min) / (max - min)) * 100;
  }, [value, min, max]);

  const valueAngle = useMemo(() => {
    return arcStartAngle + (percentage / 100) * arcSpan;
  }, [percentage, arcStartAngle, arcSpan]);

  // Determine color based on alarm limits (ISA-101 standard colors)
  const getColor = useMemo(() => {
    if (quality !== 'GOOD') {
      return theme === 'dark' ? '#9CA3AF' : '#6B7280'; // Gray for bad quality
    }

    if (!alarmLimits) {
      return theme === 'dark' ? '#10B981' : '#059669'; // Green for normal (no limits)
    }

    // Critical alarms (red)
    if (value >= alarmLimits.highHigh || value <= alarmLimits.lowLow) {
      return '#EF4444'; // Red
    }

    // Warning alarms (amber)
    if (value >= alarmLimits.high || value <= alarmLimits.low) {
      return '#F59E0B'; // Amber
    }

    // Normal (green)
    return theme === 'dark' ? '#10B981' : '#059669';
  }, [value, alarmLimits, quality, theme]);

  // Arc path calculation
  const describeArc = (startAngle: number, endAngle: number) => {
    const start = polarToCartesian(centerX, centerY, innerRadius, endAngle);
    const end = polarToCartesian(centerX, centerY, innerRadius, startAngle);
    const largeArcFlag = endAngle - startAngle <= 180 ? '0' : '1';

    return [
      'M', start.x, start.y,
      'A', innerRadius, innerRadius, 0, largeArcFlag, 0, end.x, end.y
    ].join(' ');
  };

  const polarToCartesian = (centerX: number, centerY: number, radius: number, angleInDegrees: number) => {
    const angleInRadians = (angleInDegrees - 90) * Math.PI / 180.0;
    return {
      x: centerX + (radius * Math.cos(angleInRadians)),
      y: centerY + (radius * Math.sin(angleInRadians))
    };
  };

  // Needle coordinates
  const needleEnd = polarToCartesian(centerX, centerY, innerRadius * 0.85, valueAngle);

  // Color zones (if alarm limits provided)
  const renderColorZones = () => {
    if (!alarmLimits) return null;

    const zones = [
      { start: min, end: alarmLimits.lowLow, color: '#EF4444' }, // Critical low (red)
      { start: alarmLimits.lowLow, end: alarmLimits.low, color: '#F59E0B' }, // Warning low (amber)
      { start: alarmLimits.low, end: alarmLimits.high, color: '#10B981' }, // Normal (green)
      { start: alarmLimits.high, end: alarmLimits.highHigh, color: '#F59E0B' }, // Warning high (amber)
      { start: alarmLimits.highHigh, end: max, color: '#EF4444' }, // Critical high (red)
    ];

    return zones.map((zone, index) => {
      const startAngle = arcStartAngle + ((zone.start - min) / (max - min)) * arcSpan;
      const endAngle = arcStartAngle + ((zone.end - min) / (max - min)) * arcSpan;

      return (
        <path
          key={index}
          d={describeArc(startAngle, endAngle)}
          fill="none"
          stroke={zone.color}
          strokeWidth={config.strokeWidth * 0.3}
          opacity={0.3}
        />
      );
    });
  };

  const bgColor = theme === 'dark' ? '#1F2937' : '#FFFFFF';
  const textColor = theme === 'dark' ? '#F9FAFB' : '#111827';
  const mutedTextColor = theme === 'dark' ? '#9CA3AF' : '#6B7280';

  return (
    <div className={`flex flex-col items-center space-y-2 p-4 rounded-lg bg-${theme === 'dark' ? 'gray-800' : 'white'}`}>
      {/* Label */}
      <div className="text-sm font-medium" style={{ color: textColor, fontSize: config.fontSize.label }}>
        {label}
      </div>

      {/* Gauge */}
      <svg width={config.diameter} height={config.diameter} viewBox={`0 0 ${config.diameter} ${config.diameter}`}>
        {/* Background circle */}
        <circle
          cx={centerX}
          cy={centerY}
          r={innerRadius}
          fill="none"
          stroke={theme === 'dark' ? '#374151' : '#E5E7EB'}
          strokeWidth={config.strokeWidth}
        />

        {/* Color zones (if alarm limits) */}
        {renderColorZones()}

        {/* Background arc */}
        <path
          d={describeArc(arcStartAngle, arcEndAngle)}
          fill="none"
          stroke={theme === 'dark' ? '#374151' : '#E5E7EB'}
          strokeWidth={config.strokeWidth}
          strokeLinecap="round"
        />

        {/* Value arc */}
        <path
          d={describeArc(arcStartAngle, valueAngle)}
          fill="none"
          stroke={getColor}
          strokeWidth={config.strokeWidth}
          strokeLinecap="round"
          className="transition-all duration-300"
        />

        {/* Tick marks */}
        {[0, 25, 50, 75, 100].map((tick) => {
          const tickAngle = arcStartAngle + (tick / 100) * arcSpan;
          const outerPoint = polarToCartesian(centerX, centerY, innerRadius + config.strokeWidth * 0.7, tickAngle);
          const innerPoint = polarToCartesian(centerX, centerY, innerRadius + config.strokeWidth * 0.3, tickAngle);

          return (
            <line
              key={tick}
              x1={innerPoint.x}
              y1={innerPoint.y}
              x2={outerPoint.x}
              y2={outerPoint.y}
              stroke={mutedTextColor}
              strokeWidth={2}
            />
          );
        })}

        {/* Needle */}
        <line
          x1={centerX}
          y1={centerY}
          x2={needleEnd.x}
          y2={needleEnd.y}
          stroke={theme === 'dark' ? '#F9FAFB' : '#111827'}
          strokeWidth={3}
          strokeLinecap="round"
          className="transition-all duration-300"
        />

        {/* Center dot */}
        <circle
          cx={centerX}
          cy={centerY}
          r={config.strokeWidth / 3}
          fill={theme === 'dark' ? '#F9FAFB' : '#111827'}
        />

        {/* Value text */}
        <text
          x={centerX}
          y={centerY + innerRadius * 0.5}
          textAnchor="middle"
          style={{
            fontSize: config.fontSize.value,
            fontWeight: 'bold',
            fill: textColor,
          }}
        >
          {value.toFixed(precision)}
        </text>

        {/* Unit text */}
        <text
          x={centerX}
          y={centerY + innerRadius * 0.65}
          textAnchor="middle"
          style={{
            fontSize: config.fontSize.unit,
            fill: mutedTextColor,
          }}
        >
          {unit}
        </text>

        {/* Min/max labels */}
        {showMinMax && (
          <>
            <text
              x={centerX - innerRadius * 0.7}
              y={centerY + innerRadius * 0.8}
              style={{ fontSize: config.fontSize.unit, fill: mutedTextColor }}
            >
              {min}
            </text>
            <text
              x={centerX + innerRadius * 0.7}
              y={centerY + innerRadius * 0.8}
              textAnchor="end"
              style={{ fontSize: config.fontSize.unit, fill: mutedTextColor }}
            >
              {max}
            </text>
          </>
        )}
      </svg>

      {/* Quality indicator */}
      {quality !== 'GOOD' && (
        <div className="flex items-center space-x-1 text-xs text-amber-600">
          <svg className="h-3 w-3" fill="currentColor" viewBox="0 0 20 20">
            <path fillRule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clipRule="evenodd" />
          </svg>
          <span>{quality}</span>
        </div>
      )}
    </div>
  );
};
```

### 2. Tank Level Indicator

```typescript
// apps/web/components/scada-hmi/tank-level-indicator.tsx

export interface TankLevelProps {
  label: string;
  level: number; // 0-100 percentage
  capacity: number; // Total capacity in barrels
  unit: string;
  quality: 'GOOD' | 'BAD' | 'UNCERTAIN';
  alarmLimits?: {
    low: number; // Low level alarm (%)
    lowLow: number; // Critical low level (%)
    high: number; // High level alarm (%)
    highHigh: number; // Critical high level (%)
  };
  size?: 'sm' | 'md' | 'lg';
  theme?: 'light' | 'dark';
}

export const TankLevelIndicator: React.FC<TankLevelProps> = ({
  label,
  level,
  capacity,
  unit,
  quality,
  alarmLimits,
  size = 'md',
  theme = 'light',
}) => {
  const sizeConfig = {
    sm: { width: 120, height: 160 },
    md: { width: 180, height: 240 },
    lg: { width: 240, height: 320 },
  };

  const { width, height } = sizeConfig[size];
  const tankWidth = width * 0.7;
  const tankHeight = height * 0.7;
  const tankX = (width - tankWidth) / 2;
  const tankY = (height - tankHeight) / 2;

  // Calculate fluid height
  const fluidHeight = (level / 100) * tankHeight;
  const fluidY = tankY + tankHeight - fluidHeight;

  // Determine color
  const getFluidColor = () => {
    if (quality !== 'GOOD') return '#9CA3AF';

    if (alarmLimits) {
      if (level >= alarmLimits.highHigh || level <= alarmLimits.lowLow) {
        return '#EF4444'; // Red - critical
      }
      if (level >= alarmLimits.high || level <= alarmLimits.low) {
        return '#F59E0B'; // Amber - warning
      }
    }

    return '#3B82F6'; // Blue - normal
  };

  const currentVolume = (level / 100) * capacity;
  const textColor = theme === 'dark' ? '#F9FAFB' : '#111827';

  return (
    <div className="flex flex-col items-center space-y-2">
      <div className="text-sm font-medium" style={{ color: textColor }}>
        {label}
      </div>

      <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`}>
        {/* Tank outline */}
        <rect
          x={tankX}
          y={tankY}
          width={tankWidth}
          height={tankHeight}
          fill="none"
          stroke={theme === 'dark' ? '#4B5563' : '#D1D5DB'}
          strokeWidth={3}
          rx={8}
        />

        {/* Alarm limit lines */}
        {alarmLimits && (
          <>
            {/* High-High */}
            <line
              x1={tankX}
              x2={tankX + tankWidth}
              y1={tankY + (1 - alarmLimits.highHigh / 100) * tankHeight}
              y2={tankY + (1 - alarmLimits.highHigh / 100) * tankHeight}
              stroke="#EF4444"
              strokeWidth={2}
              strokeDasharray="5,5"
              opacity={0.7}
            />

            {/* High */}
            <line
              x1={tankX}
              x2={tankX + tankWidth}
              y1={tankY + (1 - alarmLimits.high / 100) * tankHeight}
              y2={tankY + (1 - alarmLimits.high / 100) * tankHeight}
              stroke="#F59E0B"
              strokeWidth={2}
              strokeDasharray="5,5"
              opacity={0.7}
            />

            {/* Low */}
            <line
              x1={tankX}
              x2={tankX + tankWidth}
              y1={tankY + (1 - alarmLimits.low / 100) * tankHeight}
              y2={tankY + (1 - alarmLimits.low / 100) * tankHeight}
              stroke="#F59E0B"
              strokeWidth={2}
              strokeDasharray="5,5"
              opacity={0.7}
            />

            {/* Low-Low */}
            <line
              x1={tankX}
              x2={tankX + tankWidth}
              y1={tankY + (1 - alarmLimits.lowLow / 100) * tankHeight}
              y2={tankY + (1 - alarmLimits.lowLow / 100) * tankHeight}
              stroke="#EF4444"
              strokeWidth={2}
              strokeDasharray="5,5"
              opacity={0.7}
            />
          </>
        )}

        {/* Fluid fill */}
        <rect
          x={tankX}
          y={fluidY}
          width={tankWidth}
          height={fluidHeight}
          fill={getFluidColor()}
          opacity={0.7}
          rx={8}
          className="transition-all duration-500"
        />

        {/* Wave effect on fluid surface */}
        <path
          d={`M ${tankX} ${fluidY}
              Q ${tankX + tankWidth * 0.25} ${fluidY - 5}, ${tankX + tankWidth * 0.5} ${fluidY}
              T ${tankX + tankWidth} ${fluidY}
              L ${tankX + tankWidth} ${fluidY + 10}
              L ${tankX} ${fluidY + 10}
              Z`}
          fill={getFluidColor()}
          opacity={0.5}
          className="transition-all duration-500"
        />

        {/* Percentage text */}
        <text
          x={width / 2}
          y={height * 0.85}
          textAnchor="middle"
          style={{
            fontSize: 24,
            fontWeight: 'bold',
            fill: textColor,
          }}
        >
          {level.toFixed(1)}%
        </text>

        {/* Volume text */}
        <text
          x={width / 2}
          y={height * 0.93}
          textAnchor="middle"
          style={{
            fontSize: 12,
            fill: theme === 'dark' ? '#9CA3AF' : '#6B7280',
          }}
        >
          {currentVolume.toFixed(0)} / {capacity} {unit}
        </text>
      </svg>

      {quality !== 'GOOD' && (
        <div className="text-xs text-amber-600">⚠️ {quality}</div>
      )}
    </div>
  );
};
```

### 3. Status Indicator (Alarm Light)

```typescript
// apps/web/components/scada-hmi/status-indicator.tsx

export interface StatusIndicatorProps {
  label: string;
  status: 'OK' | 'WARNING' | 'CRITICAL' | 'UNKNOWN';
  animated?: boolean;
  size?: 'sm' | 'md' | 'lg';
}

export const StatusIndicator: React.FC<StatusIndicatorProps> = ({
  label,
  status,
  animated = false,
  size = 'md',
}) => {
  const sizeMap = { sm: 12, md: 16, lg: 24 };
  const indicatorSize = sizeMap[size];

  const colorMap = {
    OK: '#10B981', // Green
    WARNING: '#F59E0B', // Amber
    CRITICAL: '#EF4444', // Red
    UNKNOWN: '#6B7280', // Gray
  };

  return (
    <div className="flex items-center space-x-2">
      <div
        className={`rounded-full ${animated && status !== 'OK' ? 'animate-pulse' : ''}`}
        style={{
          width: indicatorSize,
          height: indicatorSize,
          backgroundColor: colorMap[status],
          boxShadow: `0 0 ${indicatorSize / 2}px ${colorMap[status]}`,
        }}
      />
      <span className="text-sm font-medium text-gray-700">{label}</span>
    </div>
  );
};
```

### 4. P&ID Component (Simplified)

```typescript
// apps/web/components/scada-hmi/pid-component.tsx

export interface PIDComponentProps {
  /** Component type */
  type: 'pump' | 'valve' | 'separator' | 'pipe' | 'tank';

  /** Position on diagram */
  x: number;
  y: number;

  /** Component state */
  state: {
    running?: boolean;
    open?: boolean;
    level?: number;
    flowRate?: number;
  };

  /** Label */
  label?: string;

  /** Size multiplier */
  scale?: number;
}

export const PIDComponent: React.FC<PIDComponentProps> = ({
  type,
  x,
  y,
  state,
  label,
  scale = 1,
}) => {
  const renderPump = () => (
    <g transform={`translate(${x}, ${y}) scale(${scale})`}>
      <circle
        cx="0"
        cy="0"
        r="30"
        fill={state.running ? '#10B981' : '#9CA3AF'}
        stroke="#000"
        strokeWidth="2"
        className="transition-colors duration-300"
      />
      <text
        x="0"
        y="5"
        textAnchor="middle"
        className="text-xs font-bold fill-white"
      >
        PUMP
      </text>
      {state.running && (
        <animateTransform
          attributeName="transform"
          type="rotate"
          from="0 0 0"
          to="360 0 0"
          dur="2s"
          repeatCount="indefinite"
        />
      )}
    </g>
  );

  const renderValve = () => (
    <g transform={`translate(${x}, ${y}) scale(${scale})`}>
      <polygon
        points="-15,20 15,20 0,-20"
        fill={state.open ? '#10B981' : '#EF4444'}
        stroke="#000"
        strokeWidth="2"
        className="transition-colors duration-300"
      />
      <text
        x="0"
        y="35"
        textAnchor="middle"
        className="text-xs"
      >
        {state.open ? 'OPEN' : 'CLOSED'}
      </text>
    </g>
  );

  const renderSeparator = () => (
    <g transform={`translate(${x}, ${y}) scale(${scale})`}>
      <ellipse
        cx="0"
        cy="0"
        rx="50"
        ry="80"
        fill="#E5E7EB"
        stroke="#000"
        strokeWidth="2"
      />

      {/* Fluid level */}
      {state.level !== undefined && (
        <rect
          x="-50"
          y={(1 - state.level / 100) * 80 - 80}
          width="100"
          height={(state.level / 100) * 160}
          fill="#3B82F6"
          opacity="0.5"
          className="transition-all duration-500"
        />
      )}

      <text x="0" y="5" textAnchor="middle" className="text-sm font-bold">
        {state.level?.toFixed(0)}%
      </text>
    </g>
  );

  const componentMap = {
    pump: renderPump,
    valve: renderValve,
    separator: renderSeparator,
    pipe: () => null, // Implement pipe rendering
    tank: () => null, // Implement tank rendering
  };

  return (
    <>
      {componentMap[type]()}
      {label && (
        <text
          x={x}
          y={y + 60 * scale}
          textAnchor="middle"
          className="text-xs font-medium"
        >
          {label}
        </text>
      )}
    </>
  );
};
```

### 5. Alarm Panel

```typescript
// apps/web/components/scada-hmi/alarm-panel.tsx

export interface Alarm {
  id: string;
  severity: 'CRITICAL' | 'WARNING' | 'INFO';
  message: string;
  tagName: string;
  value: number;
  limit: number;
  timestamp: Date;
  acknowledged: boolean;
}

export interface AlarmPanelProps {
  alarms: Alarm[];
  onAcknowledge: (alarmId: string) => void;
  maxHeight?: number;
}

export const AlarmPanel: React.FC<AlarmPanelProps> = ({
  alarms,
  onAcknowledge,
  maxHeight = 400,
}) => {
  const severityConfig = {
    CRITICAL: {
      bg: 'bg-red-50',
      border: 'border-red-200',
      text: 'text-red-900',
      icon: '🔴',
    },
    WARNING: {
      bg: 'bg-amber-50',
      border: 'border-amber-200',
      text: 'text-amber-900',
      icon: '⚠️',
    },
    INFO: {
      bg: 'bg-blue-50',
      border: 'border-blue-200',
      text: 'text-blue-900',
      icon: 'ℹ️',
    },
  };

  const sortedAlarms = useMemo(() => {
    return [...alarms].sort((a, b) => {
      // Sort by: 1) unacknowledged first, 2) severity, 3) newest first
      if (a.acknowledged !== b.acknowledged) {
        return a.acknowledged ? 1 : -1;
      }

      const severityOrder = { CRITICAL: 0, WARNING: 1, INFO: 2 };
      if (severityOrder[a.severity] !== severityOrder[b.severity]) {
        return severityOrder[a.severity] - severityOrder[b.severity];
      }

      return b.timestamp.getTime() - a.timestamp.getTime();
    });
  }, [alarms]);

  return (
    <div className="bg-white rounded-lg border shadow-sm">
      <div className="px-4 py-3 border-b">
        <div className="flex items-center justify-between">
          <h3 className="text-lg font-semibold">Alarms</h3>
          <div className="flex space-x-2">
            <span className="px-2 py-1 text-xs font-medium bg-red-100 text-red-800 rounded">
              {alarms.filter(a => a.severity === 'CRITICAL' && !a.acknowledged).length} Critical
            </span>
            <span className="px-2 py-1 text-xs font-medium bg-amber-100 text-amber-800 rounded">
              {alarms.filter(a => a.severity === 'WARNING' && !a.acknowledged).length} Warning
            </span>
          </div>
        </div>
      </div>

      <div
        className="divide-y overflow-y-auto"
        style={{ maxHeight }}
      >
        {sortedAlarms.length === 0 ? (
          <div className="p-8 text-center text-gray-500">
            <span className="text-4xl">✅</span>
            <p className="mt-2 text-sm">All systems normal</p>
          </div>
        ) : (
          sortedAlarms.map((alarm) => {
            const config = severityConfig[alarm.severity];

            return (
              <div
                key={alarm.id}
                className={`p-4 ${config.bg} ${config.border} border-l-4 ${
                  alarm.acknowledged ? 'opacity-50' : ''
                }`}
              >
                <div className="flex items-start justify-between">
                  <div className="flex items-start space-x-3">
                    <span className="text-xl">{config.icon}</span>
                    <div>
                      <div className={`font-medium ${config.text}`}>
                        {alarm.message}
                      </div>
                      <div className="mt-1 text-xs text-gray-600">
                        {alarm.tagName}: {alarm.value.toFixed(1)} (limit: {alarm.limit.toFixed(1)})
                      </div>
                      <div className="mt-1 text-xs text-gray-500">
                        {alarm.timestamp.toLocaleString()}
                      </div>
                    </div>
                  </div>

                  {!alarm.acknowledged && (
                    <button
                      onClick={() => onAcknowledge(alarm.id)}
                      className="px-3 py-1 text-xs font-medium text-white bg-gray-700 rounded hover:bg-gray-800"
                    >
                      Acknowledge
                    </button>
                  )}
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
```

### 6. Setpoint Control Component

```typescript
// apps/web/components/scada-hmi/setpoint-control.tsx

export interface SetpointControlProps {
  label: string;
  currentValue: number;
  setpoint: number;
  min: number;
  max: number;
  unit: string;
  step?: number;
  precision?: number;
  onSetpointChange: (newSetpoint: number) => void;
  disabled?: boolean;
}

export const SetpointControl: React.FC<SetpointControlProps> = ({
  label,
  currentValue,
  setpoint,
  min,
  max,
  unit,
  step = 1,
  precision = 1,
  onSetpointChange,
  disabled = false,
}) => {
  const [localSetpoint, setLocalSetpoint] = useState(setpoint);
  const [isEditing, setIsEditing] = useState(false);

  const handleIncrement = () => {
    const newValue = Math.min(localSetpoint + step, max);
    setLocalSetpoint(newValue);
  };

  const handleDecrement = () => {
    const newValue = Math.max(localSetpoint - step, min);
    setLocalSetpoint(newValue);
  };

  const handleApply = () => {
    onSetpointChange(localSetpoint);
    setIsEditing(false);
  };

  const handleCancel = () => {
    setLocalSetpoint(setpoint);
    setIsEditing(false);
  };

  return (
    <div className="bg-white rounded-lg border p-4 space-y-3">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">{label}</span>
        {!disabled && !isEditing && (
          <button
            onClick={() => setIsEditing(true)}
            className="px-3 py-1 text-xs font-medium text-blue-600 border border-blue-600 rounded hover:bg-blue-50"
          >
            Edit
          </button>
        )}
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* Current Value */}
        <div>
          <div className="text-xs text-gray-500 mb-1">Current Value</div>
          <div className="text-2xl font-bold text-gray-900">
            {currentValue.toFixed(precision)} <span className="text-sm text-gray-500">{unit}</span>
          </div>
        </div>

        {/* Setpoint */}
        <div>
          <div className="text-xs text-gray-500 mb-1">Setpoint</div>
          {isEditing ? (
            <div className="flex items-center space-x-1">
              <button
                onClick={handleDecrement}
                className="p-1 text-gray-600 bg-gray-100 rounded hover:bg-gray-200"
              >
                −
              </button>
              <input
                type="number"
                value={localSetpoint}
                onChange={(e) => setLocalSetpoint(parseFloat(e.target.value))}
                min={min}
                max={max}
                step={step}
                className="w-20 px-2 py-1 text-center border rounded"
              />
              <button
                onClick={handleIncrement}
                className="p-1 text-gray-600 bg-gray-100 rounded hover:bg-gray-200"
              >
                +
              </button>
            </div>
          ) : (
            <div className="text-2xl font-bold text-blue-600">
              {setpoint.toFixed(precision)} <span className="text-sm text-gray-500">{unit}</span>
            </div>
          )}
        </div>
      </div>

      {/* Slider */}
      {isEditing && (
        <div className="space-y-2">
          <input
            type="range"
            min={min}
            max={max}
            step={step}
            value={localSetpoint}
            onChange={(e) => setLocalSetpoint(parseFloat(e.target.value))}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-gray-500">
            <span>{min} {unit}</span>
            <span>{max} {unit}</span>
          </div>
        </div>
      )}

      {/* Action buttons */}
      {isEditing && (
        <div className="flex space-x-2">
          <button
            onClick={handleApply}
            className="flex-1 px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded hover:bg-blue-700"
          >
            Apply
          </button>
          <button
            onClick={handleCancel}
            className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded hover:bg-gray-200"
          >
            Cancel
          </button>
        </div>
      )}
    </div>
  );
};
```

## Industrial Color Standards (ISA-101)

Follow ISA-101 HMI design standards for consistency:

```typescript
export const ISA_COLORS = {
  // Normal operating states
  NORMAL: '#10B981', // Green
  ACTIVE: '#3B82F6', // Blue
  INACTIVE: '#6B7280', // Gray

  // Alarm states
  WARNING: '#F59E0B', // Amber/Yellow
  CRITICAL: '#EF4444', // Red
  ADVISORY: '#F59E0B', // Amber

  // Equipment states
  RUNNING: '#10B981', // Green
  STOPPED: '#6B7280', // Gray
  MAINTENANCE: '#F59E0B', // Amber
  FAULT: '#EF4444', // Red

  // Data quality
  GOOD: '#10B981', // Green
  UNCERTAIN: '#F59E0B', // Amber
  BAD: '#EF4444', // Red
};
```

## Performance Optimization

```typescript
// apps/web/hooks/use-throttled-updates.ts

/**
 * Throttle rapid SCADA updates to prevent excessive re-renders
 * Updates no more than once per throttle period
 */
export function useThrottledUpdates<T>(value: T, throttleMs: number = 100): T {
  const [throttledValue, setThrottledValue] = useState(value);
  const lastUpdate = useRef(Date.now());

  useEffect(() => {
    const now = Date.now();
    const timeSinceLastUpdate = now - lastUpdate.current;

    if (timeSinceLastUpdate >= throttleMs) {
      setThrottledValue(value);
      lastUpdate.current = now;
    } else {
      const timeout = setTimeout(() => {
        setThrottledValue(value);
        lastUpdate.current = Date.now();
      }, throttleMs - timeSinceLastUpdate);

      return () => clearTimeout(timeout);
    }
  }, [value, throttleMs]);

  return throttledValue;
}

// Usage in component
export function RealTimeGauge({ value }: { value: number }) {
  const throttledValue = useThrottledUpdates(value, 200); // Update max 5 times per second

  return <CircularGauge value={throttledValue} {...otherProps} />;
}
```

## Benefits

### User Experience
- **Familiar Interface**: Matches physical HMI panels operators already know (reduced training)
- **Visual Clarity**: Color-coded zones make abnormal conditions immediately obvious
- **Spatial Context**: P&ID diagrams show equipment relationships
- **Responsive**: Works on desktop, tablet, phone

### Performance
- **SVG Rendering**: Scalable without pixelation, GPU-accelerated
- **Throttled Updates**: Prevent excessive re-renders from rapid SCADA updates
- **Smooth Animations**: CSS transitions provide fluid gauge movement

### Maintainability
- **Reusable Components**: Build once, use throughout application
- **Type-Safe**: Full TypeScript support prevents errors
- **Customizable**: Props allow per-well configuration (limits, colors, units)
- **Composable**: Combine components to build complex dashboards

## Consequences

### Positive
- **High user acceptance** - Familiar visual language reduces resistance to digital tools
- **Faster training** - New users already know how to read the interface
- **Better situational awareness** - Color coding and spatial layout improve decision-making
- **Cross-platform** - Same components work on web, tablet, mobile

### Negative
- **Development effort** - Custom SVG components take longer than generic charts
- **Browser compatibility** - Older browsers may have SVG rendering issues
- **File size** - Comprehensive component library increases bundle size
- **Performance tuning** - May need throttling/debouncing for 100+ simultaneous gauges

## Related Patterns

- **Pattern 84: Digital Twin SCADA System** - Primary consumer of HMI components
- **Pattern 85: Real-Time Event-Driven Architecture** - Data source for real-time updates
- **Pattern 87: Time-Series Visualization** - Historical trend charts
- **Atomic Design Pattern** - Component organization (atoms → molecules → organisms)
- **Compound Component Pattern** - Complex components with subcomponents

## References

- ISA-101 HMI Design Standard: https://www.isa.org/standards-and-publications/isa-standards/isa-standards-committees/isa101
- ANSI/ISA-5.1 Instrumentation Symbols: https://www.isa.org/products/ansi-isa-5-1-2022-instrumentation-symbols-and-ide
- High Performance Browser Rendering: https://web.dev/rendering-performance/
- WellOS Digital Twin Implementation (Pattern 84)

## Changelog

- **2025-10-30**: Initial pattern created for SCADA HMI component library
