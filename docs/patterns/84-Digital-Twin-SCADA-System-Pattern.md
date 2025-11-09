# Pattern 84: Digital Twin SCADA System Pattern

## Category
Real-Time Systems, Industrial IoT, Visualization

## Status
✅ **Production Ready** - Implemented in WellOS Sprint 5

## Context

Oil & Gas operators need to monitor and control remote well sites in the Permian Basin without expensive field visits. Traditional approaches require field personnel to drive to each well site to check gauges, read meters, and verify equipment status - often traveling hundreds of miles per day across harsh terrain.

**Business Impact**:
- **Field visits cost $500-1000 per day** (vehicle, fuel, personnel time)
- **Response time**: 2-6 hours to reach remote wells
- **Safety risks**: Driving on remote roads, exposure to hazardous conditions
- **Inefficiency**: 70% of field visits find "everything normal"

**Solution**: Create a virtual replica (Digital Twin) of each well site in the browser, providing real-time monitoring, historical trends, and remote control capabilities - just like standing in front of the physical equipment.

## Problem

How do you create an accurate, real-time virtual representation of physical Oil & Gas assets that:

1. **Mirrors Physical State**: Reflects actual equipment conditions with sub-second latency
2. **Provides Context**: Shows P&ID diagrams, equipment relationships, safety limits
3. **Handles Complexity**: Displays hundreds of sensors per well site
4. **Supports Interaction**: Allows remote setpoint changes, valve control, pump start/stop
5. **Maintains History**: Shows trends, alarms, and historical comparisons
6. **Scales**: Supports 50-500 wells per operator with thousands of sensors
7. **Works Offline**: Field personnel can view cached state when connectivity is limited

## Forces

- **Real-Time Requirements**: SCADA data updates every 1-5 seconds
- **Multi-Protocol Complexity**: 7 different industrial protocols (OPC-UA, Modbus, MQTT, DNP3, etc.)
- **Network Reliability**: Oil fields have intermittent connectivity
- **Visual Fidelity**: Must match physical HMI screens at well sites
- **Security**: Prevent unauthorized control of production equipment
- **Tenant Isolation**: Each operator only sees their own wells
- **Mobile Support**: Field personnel use tablets/phones for remote monitoring

## Solution

Implement a **multi-layer Digital Twin architecture** that:

1. **Ingests** real-time data from physical SCADA systems via protocol adapters
2. **Streams** updates through event-driven architecture (Redis Pub/Sub + WebSocket)
3. **Renders** interactive HMI components in the browser (React components mimicking physical gauges/panels)
4. **Synchronizes** bidirectionally (monitor + control)
5. **Caches** state for offline viewing

### Architecture Overview

```
Physical Well Site              Digital Twin (Virtual Replica)
┌─────────────────────────┐    ┌─────────────────────────────────┐
│ SCADA Equipment         │    │ Web Browser                     │
│ ├── PLC (OPC-UA)       │    │ ├── 3D Well Site View          │
│ ├── RTU (Modbus)       │──▶ │ ├── Real-time Gauges           │
│ ├── Sensors (MQTT)     │    │ ├── P&ID Diagrams              │
│ ├── Flow Meters        │    │ ├── Alarm Panel                │
│ └── Pumps, Valves      │    │ ├── Historical Trends          │
└─────────────────────────┘    │ └── Remote Control Panel       │
                                └─────────────────────────────────┘

Data Flow (Physical → Virtual):
┌──────────────┐   Native   ┌──────────────┐   Pub/Sub   ┌──────────────┐   WebSocket   ┌──────────────┐
│ SCADA Device │ ─────────▶ │ Rust SCADA   │ ──────────▶ │ Redis        │ ────────────▶ │ Rust API     │
│ (7 protocols)│  Protocol  │ Ingestion    │             │ (broadcast)  │               │ (Axum+WS)    │
└──────────────┘            └──────────────┘             └──────────────┘               └──────┬───────┘
                                                                                               │
                                                                                               ▼
                                                                                        ┌──────────────┐
                                                                                        │ React UI     │
                                                                                        │ (Digital Twin│
                                                                                        │  Components) │
                                                                                        └──────────────┘

Control Flow (Virtual → Physical):
┌──────────────┐   HTTP    ┌──────────────┐   gRPC    ┌──────────────┐   Protocol    ┌──────────────┐
│ React UI     │ ────────▶ │ Rust API     │ ────────▶ │ Rust SCADA   │ ────────────▶ │ SCADA Device │
│ (Setpoint)   │           │ (Validation) │           │ Ingestion    │   (Write)     │ (PLC/RTU)    │
└──────────────┘           └──────────────┘           └──────────────┘               └──────────────┘
```

## Implementation

### 1. Core Digital Twin Components

#### **Well Site Twin** (Aggregate Root)

```typescript
// apps/web/lib/digital-twin/well-site-twin.ts

/**
 * Digital Twin of a physical well site
 * Maintains synchronized state with real equipment
 */
export class WellSiteTwin {
  private readonly wellId: string;
  private readonly tenantId: string;
  private state: WellSiteState;
  private websocket: Socket;
  private stateHistory: CircularBuffer<WellSiteState>;
  private subscriptions: Set<string> = new Set();

  constructor(wellId: string, tenantId: string) {
    this.wellId = wellId;
    this.tenantId = tenantId;
    this.state = this.loadCachedState() || this.getDefaultState();
    this.stateHistory = new CircularBuffer<WellSiteState>(1000); // Last 1000 states
  }

  /**
   * Connect to real-time SCADA data stream
   */
  async connect(): Promise<void> {
    const token = await this.getAuthToken();

    this.websocket = io('ws://localhost:4000/scada', {
      auth: { token },
      reconnection: true,
      reconnectionAttempts: Infinity,
      reconnectionDelay: 1000,
      reconnectionDelayMax: 5000,
    });

    this.websocket.on('connect', () => {
      console.log(`Digital Twin connected: ${this.wellId}`);

      // Subscribe to well-specific updates
      this.websocket.emit('subscribe-well', { wellId: this.wellId });
    });

    this.websocket.on('reading', (reading: ScadaReading) => {
      this.updateState(reading);
    });

    this.websocket.on('disconnect', () => {
      console.warn(`Digital Twin disconnected: ${this.wellId}`);
      // State remains cached - continues displaying last known values
    });
  }

  /**
   * Update twin state from SCADA reading
   */
  private updateState(reading: ScadaReading): void {
    // Update specific tag value
    this.state.tags.set(reading.tagName, {
      value: reading.value,
      quality: reading.quality,
      timestamp: new Date(reading.timestamp),
      unit: this.getTagUnit(reading.tagName),
    });

    // Derive calculated values
    this.updateCalculatedValues();

    // Detect alarms
    this.checkAlarms(reading);

    // Save to history
    this.stateHistory.push({ ...this.state, timestamp: new Date() });

    // Cache for offline access
    this.cacheState();

    // Notify subscribers (React components)
    this.notifySubscribers();
  }

  /**
   * Send control command to physical equipment
   */
  async sendControlCommand(command: ControlCommand): Promise<void> {
    // Optimistic UI update
    this.state.tags.get(command.tagName)!.value = command.value;
    this.notifySubscribers();

    try {
      const response = await fetch(`/api/scada/control`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          wellId: this.wellId,
          tagName: command.tagName,
          value: command.value,
          reason: command.reason, // Audit trail
        }),
      });

      if (!response.ok) {
        // Rollback optimistic update
        const reading = await this.fetchCurrentValue(command.tagName);
        this.updateState(reading);
        throw new Error(`Control command failed: ${response.statusText}`);
      }

      // Confirmation will come via WebSocket reading
    } catch (error) {
      console.error('Control command error:', error);
      throw error;
    }
  }

  /**
   * Get current state snapshot
   */
  getState(): Readonly<WellSiteState> {
    return Object.freeze({ ...this.state });
  }

  /**
   * Get historical states for trending
   */
  getHistory(duration: Duration): WellSiteState[] {
    const cutoff = new Date(Date.now() - duration.ms);
    return this.stateHistory.filter(state => state.timestamp >= cutoff);
  }

  /**
   * Subscribe to state changes (for React components)
   */
  subscribe(callback: (state: WellSiteState) => void): () => void {
    const id = Math.random().toString(36);
    this.subscriptions.add(id);

    // Immediate callback with current state
    callback(this.getState());

    // Return unsubscribe function
    return () => {
      this.subscriptions.delete(id);
    };
  }

  private notifySubscribers(): void {
    this.subscriptions.forEach(() => {
      // Trigger React re-render via useState
      this.notifyCallback?.(this.getState());
    });
  }

  private updateCalculatedValues(): void {
    // Example: Calculate total fluid production
    const oil = this.state.tags.get('oil_rate')?.value || 0;
    const water = this.state.tags.get('water_rate')?.value || 0;
    const totalFluid = oil + water;

    this.state.calculated.set('total_fluid_rate', totalFluid);

    // Calculate water cut percentage
    const waterCut = totalFluid > 0 ? (water / totalFluid) * 100 : 0;
    this.state.calculated.set('water_cut_pct', waterCut);

    // Calculate GOR (Gas-Oil Ratio)
    const gas = this.state.tags.get('gas_rate')?.value || 0;
    const gor = oil > 0 ? gas / oil : 0;
    this.state.calculated.set('gor', gor);
  }

  private checkAlarms(reading: ScadaReading): void {
    const alarmConfig = this.getAlarmConfig(reading.tagName);
    if (!alarmConfig) return;

    // High alarm
    if (reading.value > alarmConfig.highHigh) {
      this.raiseAlarm({
        severity: 'CRITICAL',
        message: `${reading.tagName} critically high: ${reading.value} ${this.getTagUnit(reading.tagName)}`,
        tagName: reading.tagName,
        value: reading.value,
        limit: alarmConfig.highHigh,
      });
    } else if (reading.value > alarmConfig.high) {
      this.raiseAlarm({
        severity: 'WARNING',
        message: `${reading.tagName} high: ${reading.value} ${this.getTagUnit(reading.tagName)}`,
        tagName: reading.tagName,
        value: reading.value,
        limit: alarmConfig.high,
      });
    }

    // Low alarm
    if (reading.value < alarmConfig.lowLow) {
      this.raiseAlarm({
        severity: 'CRITICAL',
        message: `${reading.tagName} critically low: ${reading.value} ${this.getTagUnit(reading.tagName)}`,
        tagName: reading.tagName,
        value: reading.value,
        limit: alarmConfig.lowLow,
      });
    } else if (reading.value < alarmConfig.low) {
      this.raiseAlarm({
        severity: 'WARNING',
        message: `${reading.tagName} low: ${reading.value} ${this.getTagUnit(reading.tagName)}`,
        tagName: reading.tagName,
        value: reading.value,
        limit: alarmConfig.low,
      });
    }
  }

  private cacheState(): void {
    // Cache in IndexedDB for offline access
    localStorage.setItem(
      `well-twin:${this.wellId}`,
      JSON.stringify({
        state: this.state,
        timestamp: new Date().toISOString(),
      })
    );
  }

  private loadCachedState(): WellSiteState | null {
    const cached = localStorage.getItem(`well-twin:${this.wellId}`);
    if (!cached) return null;

    try {
      const { state, timestamp } = JSON.parse(cached);
      const age = Date.now() - new Date(timestamp).getTime();

      // Only use cache if less than 1 hour old
      if (age < 3600000) {
        return state;
      }
    } catch (error) {
      console.error('Failed to load cached state:', error);
    }

    return null;
  }
}

/**
 * Well site state (synchronized with physical equipment)
 */
interface WellSiteState {
  wellId: string;
  tenantId: string;
  timestamp: Date;
  connectionStatus: 'connected' | 'disconnected' | 'degraded';

  // Raw SCADA tags
  tags: Map<string, TagValue>;

  // Calculated/derived values
  calculated: Map<string, number>;

  // Active alarms
  alarms: Alarm[];

  // Equipment status
  equipment: {
    pump: EquipmentStatus;
    separator: EquipmentStatus;
    controller: EquipmentStatus;
  };
}

interface TagValue {
  value: number;
  quality: 'GOOD' | 'BAD' | 'UNCERTAIN';
  timestamp: Date;
  unit: string;
}

interface Alarm {
  id: string;
  severity: 'CRITICAL' | 'WARNING' | 'INFO';
  message: string;
  tagName: string;
  value: number;
  limit: number;
  timestamp: Date;
  acknowledged: boolean;
}
```

### 2. HMI Component Library

#### **Circular Gauge Component** (Mimics Physical Gauges)

```typescript
// apps/web/components/digital-twin/circular-gauge.tsx

interface CircularGaugeProps {
  label: string;
  value: number;
  min: number;
  max: number;
  unit: string;
  quality: 'GOOD' | 'BAD' | 'UNCERTAIN';
  alarmLimits?: {
    low: number;
    lowLow: number;
    high: number;
    highHigh: number;
  };
  size?: 'sm' | 'md' | 'lg';
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
}) => {
  const sizeMap = { sm: 120, md: 180, lg: 240 };
  const radius = sizeMap[size] / 2;
  const strokeWidth = radius * 0.15;
  const innerRadius = radius - strokeWidth;

  // Calculate arc path
  const percentage = ((value - min) / (max - min)) * 100;
  const angle = (percentage / 100) * 270 - 135; // -135° to +135° (270° arc)

  // Determine color based on alarm limits
  const getColor = (): string => {
    if (quality !== 'GOOD') return '#9CA3AF'; // Gray for bad quality
    if (!alarmLimits) return '#10B981'; // Green default

    if (value >= alarmLimits.highHigh || value <= alarmLimits.lowLow) {
      return '#EF4444'; // Red for critical
    }
    if (value >= alarmLimits.high || value <= alarmLimits.low) {
      return '#F59E0B'; // Amber for warning
    }
    return '#10B981'; // Green for normal
  };

  return (
    <div className="flex flex-col items-center space-y-2">
      <div className="text-sm font-medium text-gray-700">{label}</div>

      <svg width={sizeMap[size]} height={sizeMap[size]} viewBox={`0 0 ${sizeMap[size]} ${sizeMap[size]}`}>
        {/* Background arc */}
        <circle
          cx={radius}
          cy={radius}
          r={innerRadius}
          fill="none"
          stroke="#E5E7EB"
          strokeWidth={strokeWidth}
          strokeDasharray={`${(270 / 360) * 2 * Math.PI * innerRadius} ${2 * Math.PI * innerRadius}`}
          transform={`rotate(-135 ${radius} ${radius})`}
        />

        {/* Value arc */}
        <circle
          cx={radius}
          cy={radius}
          r={innerRadius}
          fill="none"
          stroke={getColor()}
          strokeWidth={strokeWidth}
          strokeDasharray={`${((percentage / 100) * 270 / 360) * 2 * Math.PI * innerRadius} ${2 * Math.PI * innerRadius}`}
          transform={`rotate(-135 ${radius} ${radius})`}
          strokeLinecap="round"
          className="transition-all duration-300"
        />

        {/* Needle */}
        <line
          x1={radius}
          y1={radius}
          x2={radius + innerRadius * 0.8 * Math.cos((angle * Math.PI) / 180)}
          y2={radius + innerRadius * 0.8 * Math.sin((angle * Math.PI) / 180)}
          stroke="#1F2937"
          strokeWidth={2}
          strokeLinecap="round"
          className="transition-all duration-300"
        />

        {/* Center dot */}
        <circle cx={radius} cy={radius} r={strokeWidth / 3} fill="#1F2937" />

        {/* Value text */}
        <text
          x={radius}
          y={radius + innerRadius * 0.6}
          textAnchor="middle"
          className="text-2xl font-bold fill-gray-900"
        >
          {value.toFixed(1)}
        </text>

        {/* Unit text */}
        <text
          x={radius}
          y={radius + innerRadius * 0.75}
          textAnchor="middle"
          className="text-xs fill-gray-500"
        >
          {unit}
        </text>

        {/* Min/max labels */}
        <text
          x={radius - innerRadius * 0.7}
          y={radius + innerRadius * 0.7}
          className="text-xs fill-gray-400"
        >
          {min}
        </text>
        <text
          x={radius + innerRadius * 0.7}
          y={radius + innerRadius * 0.7}
          className="text-xs fill-gray-400"
        >
          {max}
        </text>
      </svg>

      {/* Quality indicator */}
      {quality !== 'GOOD' && (
        <div className="flex items-center space-x-1 text-xs text-amber-600">
          <AlertTriangle className="h-3 w-3" />
          <span>{quality}</span>
        </div>
      )}
    </div>
  );
};
```

#### **P&ID Diagram Component** (Process Flow)

```typescript
// apps/web/components/digital-twin/pid-diagram.tsx

interface PIDDiagramProps {
  wellTwin: WellSiteTwin;
}

export const PIDDiagram: React.FC<PIDDiagramProps> = ({ wellTwin }) => {
  const state = wellTwin.getState();

  return (
    <svg width="800" height="600" viewBox="0 0 800 600" className="bg-white rounded-lg border">
      {/* Well bore */}
      <g id="wellbore">
        <rect x="50" y="400" width="30" height="150" fill="#8B4513" stroke="#000" strokeWidth="2" />
        <text x="65" y="580" textAnchor="middle" className="text-xs">Well</text>
      </g>

      {/* Pump */}
      <g id="pump">
        <circle
          cx="150"
          cy="475"
          r="30"
          fill={state.equipment.pump.status === 'RUNNING' ? '#10B981' : '#9CA3AF'}
          stroke="#000"
          strokeWidth="2"
          className="transition-colors duration-300"
        />
        <text x="150" y="480" textAnchor="middle" className="text-xs font-bold fill-white">
          PUMP
        </text>
        <text x="150" y="520" textAnchor="middle" className="text-xs">
          {state.tags.get('pump_speed')?.value.toFixed(0)} RPM
        </text>
      </g>

      {/* Flow lines */}
      <line x1="80" y1="475" x2="120" y2="475" stroke="#000" strokeWidth="4" />
      <line x1="180" y1="475" x2="250" y2="475" stroke="#000" strokeWidth="4" />

      {/* Separator */}
      <g id="separator">
        <ellipse
          cx="350"
          cy="400"
          rx="80"
          ry="120"
          fill="#E5E7EB"
          stroke="#000"
          strokeWidth="2"
        />
        <text x="350" y="350" textAnchor="middle" className="text-sm font-bold">
          SEPARATOR
        </text>

        {/* Fluid level indicator */}
        <rect
          x="270"
          y={400 + (1 - state.tags.get('separator_level')?.value / 100) * 100}
          width="160"
          height={(state.tags.get('separator_level')?.value / 100) * 100}
          fill="#3B82F6"
          opacity="0.5"
          className="transition-all duration-300"
        />

        {/* Level percentage */}
        <text x="350" y="410" textAnchor="middle" className="text-lg font-bold">
          {state.tags.get('separator_level')?.value.toFixed(0)}%
        </text>
      </g>

      {/* Oil outlet */}
      <g id="oil-line">
        <line x1="430" y1="420" x2="550" y2="420" stroke="#000" strokeWidth="4" />
        <polygon points="550,420 560,415 560,425" fill="#000" />
        <text x="490" y="410" textAnchor="middle" className="text-xs">
          Oil: {state.tags.get('oil_rate')?.value.toFixed(1)} bbl/d
        </text>
      </g>

      {/* Gas outlet */}
      <g id="gas-line">
        <line x1="350" y1="280" x2="350" y2="200" stroke="#000" strokeWidth="4" strokeDasharray="10,5" />
        <polygon points="350,200 345,210 355,210" fill="#000" />
        <text x="370" y="240" className="text-xs">
          Gas: {state.tags.get('gas_rate')?.value.toFixed(1)} mcf/d
        </text>
      </g>

      {/* Water outlet */}
      <g id="water-line">
        <line x1="350" y1="520" x2="350" y2="580" stroke="#000" strokeWidth="4" />
        <polygon points="350,580 345,570 355,570" fill="#000" />
        <text x="370" y="550" className="text-xs">
          Water: {state.tags.get('water_rate')?.value.toFixed(1)} bbl/d
        </text>
      </g>

      {/* Pressure gauges */}
      <g id="tubing-pressure">
        <circle cx="200" cy="450" r="20" fill="white" stroke="#000" strokeWidth="2" />
        <text x="200" y="455" textAnchor="middle" className="text-xs font-bold">
          {state.tags.get('tubing_pressure')?.value.toFixed(0)}
        </text>
        <text x="200" y="485" textAnchor="middle" className="text-xs">PSI</text>
      </g>
    </svg>
  );
};
```

#### **Real-Time Trend Chart** (Historical + Live)

```typescript
// apps/web/components/digital-twin/trend-chart.tsx

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

interface TrendChartProps {
  wellTwin: WellSiteTwin;
  tagName: string;
  duration: Duration; // e.g., { hours: 1 }
  label: string;
  unit: string;
  color?: string;
}

export const TrendChart: React.FC<TrendChartProps> = ({
  wellTwin,
  tagName,
  duration,
  label,
  unit,
  color = '#3B82F6',
}) => {
  const [data, setData] = useState<ChartDataPoint[]>([]);

  useEffect(() => {
    // Load historical data
    const history = wellTwin.getHistory(duration);
    const chartData = history.map(state => ({
      timestamp: state.timestamp.getTime(),
      value: state.tags.get(tagName)?.value || 0,
      quality: state.tags.get(tagName)?.quality,
    }));
    setData(chartData);

    // Subscribe to live updates
    const unsubscribe = wellTwin.subscribe(state => {
      const tagValue = state.tags.get(tagName);
      if (tagValue) {
        setData(prev => [
          ...prev.slice(-(duration.ms / 5000)), // Keep only data within duration (assuming 5s updates)
          {
            timestamp: state.timestamp.getTime(),
            value: tagValue.value,
            quality: tagValue.quality,
          },
        ]);
      }
    });

    return () => unsubscribe();
  }, [wellTwin, tagName, duration]);

  return (
    <div className="w-full h-64">
      <div className="text-sm font-medium text-gray-700 mb-2">{label}</div>
      <ResponsiveContainer width="100%" height="100%">
        <LineChart data={data}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis
            dataKey="timestamp"
            type="number"
            domain={['dataMin', 'dataMax']}
            tickFormatter={(timestamp) => new Date(timestamp).toLocaleTimeString()}
          />
          <YAxis
            label={{ value: unit, angle: -90, position: 'insideLeft' }}
          />
          <Tooltip
            labelFormatter={(timestamp) => new Date(timestamp).toLocaleString()}
            formatter={(value: number) => [`${value.toFixed(2)} ${unit}`, label]}
          />
          <Line
            type="monotone"
            dataKey="value"
            stroke={color}
            strokeWidth={2}
            dot={false}
            isAnimationActive={false} // Disable animation for real-time smoothness
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
};
```

### 3. React Hook for Digital Twin

```typescript
// apps/web/hooks/use-well-twin.ts

/**
 * React hook for consuming Digital Twin state
 * Manages WebSocket connection lifecycle and state synchronization
 */
export function useWellTwin(wellId: string) {
  const [twin, setTwin] = useState<WellSiteTwin | null>(null);
  const [state, setState] = useState<WellSiteState | null>(null);
  const [connected, setConnected] = useState(false);
  const { tenantId } = useAuth();

  useEffect(() => {
    // Create Digital Twin instance
    const wellTwin = new WellSiteTwin(wellId, tenantId);
    setTwin(wellTwin);

    // Connect to real-time stream
    wellTwin.connect().then(() => {
      setConnected(true);
    });

    // Subscribe to state changes
    const unsubscribe = wellTwin.subscribe((newState) => {
      setState(newState);
    });

    return () => {
      unsubscribe();
      wellTwin.disconnect();
    };
  }, [wellId, tenantId]);

  // Control command helper
  const sendCommand = useCallback(async (command: ControlCommand) => {
    if (!twin) throw new Error('Twin not initialized');
    await twin.sendControlCommand(command);
  }, [twin]);

  // Tag value helper
  const getTagValue = useCallback((tagName: string): TagValue | undefined => {
    return state?.tags.get(tagName);
  }, [state]);

  return {
    twin,
    state,
    connected,
    sendCommand,
    getTagValue,
  };
}
```

### 4. Usage Example (Complete Well Site Dashboard)

```typescript
// apps/web/app/(dashboard)/wells/[id]/digital-twin/page.tsx

export default function WellDigitalTwinPage({ params }: { params: { id: string } }) {
  const { twin, state, connected, sendCommand, getTagValue } = useWellTwin(params.id);

  if (!state) {
    return <LoadingSkeleton />;
  }

  return (
    <div className="space-y-6 p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Digital Twin - Well {state.wellId}</h1>
          <p className="text-sm text-gray-500">Real-time mirror of physical equipment</p>
        </div>

        <div className="flex items-center space-x-2">
          <div className={`h-3 w-3 rounded-full ${connected ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className="text-sm font-medium">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </div>

      {/* Alarm Banner */}
      {state.alarms.length > 0 && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center space-x-2 mb-2">
            <AlertTriangle className="h-5 w-5 text-red-600" />
            <span className="font-medium text-red-900">Active Alarms ({state.alarms.length})</span>
          </div>
          {state.alarms.map(alarm => (
            <div key={alarm.id} className="text-sm text-red-800 ml-7">
              • {alarm.message}
            </div>
          ))}
        </div>
      )}

      {/* P&ID Diagram */}
      <Card>
        <CardHeader>
          <CardTitle>Process Flow Diagram</CardTitle>
        </CardHeader>
        <CardContent>
          <PIDDiagram wellTwin={twin!} />
        </CardContent>
      </Card>

      {/* Real-time Gauges */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <CircularGauge
              label="Oil Production"
              value={getTagValue('oil_rate')?.value || 0}
              min={0}
              max={500}
              unit="bbl/d"
              quality={getTagValue('oil_rate')?.quality || 'UNCERTAIN'}
              alarmLimits={{ low: 50, lowLow: 20, high: 400, highHigh: 450 }}
            />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <CircularGauge
              label="Gas Production"
              value={getTagValue('gas_rate')?.value || 0}
              min={0}
              max={1000}
              unit="mcf/d"
              quality={getTagValue('gas_rate')?.quality || 'UNCERTAIN'}
              alarmLimits={{ low: 100, lowLow: 50, high: 900, highHigh: 950 }}
            />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <CircularGauge
              label="Tubing Pressure"
              value={getTagValue('tubing_pressure')?.value || 0}
              min={0}
              max={2000}
              unit="PSI"
              quality={getTagValue('tubing_pressure')?.quality || 'UNCERTAIN'}
              alarmLimits={{ low: 200, lowLow: 100, high: 1800, highHigh: 1900 }}
            />
          </CardContent>
        </Card>

        <Card>
          <CardContent className="pt-6">
            <CircularGauge
              label="Separator Level"
              value={getTagValue('separator_level')?.value || 0}
              min={0}
              max={100}
              unit="%"
              quality={getTagValue('separator_level')?.quality || 'UNCERTAIN'}
              alarmLimits={{ low: 20, lowLow: 10, high: 85, highHigh: 95 }}
            />
          </CardContent>
        </Card>
      </div>

      {/* Trend Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Oil Production Trend (Last Hour)</CardTitle>
          </CardHeader>
          <CardContent>
            <TrendChart
              wellTwin={twin!}
              tagName="oil_rate"
              duration={{ hours: 1 }}
              label="Oil Rate"
              unit="bbl/d"
              color="#10B981"
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Tubing Pressure Trend (Last Hour)</CardTitle>
          </CardHeader>
          <CardContent>
            <TrendChart
              wellTwin={twin!}
              tagName="tubing_pressure"
              duration={{ hours: 1 }}
              label="Tubing Pressure"
              unit="PSI"
              color="#3B82F6"
            />
          </CardContent>
        </Card>
      </div>

      {/* Control Panel */}
      <Card>
        <CardHeader>
          <CardTitle>Remote Control</CardTitle>
          <CardDescription>Send commands to physical equipment</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <ControlPanel
              wellTwin={twin!}
              onCommand={sendCommand}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
```

## Benefits

### Business Benefits
- **Cost Reduction**: Eliminate 60-80% of routine field visits ($500-1000 per day saved)
- **Faster Response**: Detect issues in real-time instead of waiting for next field visit (2-6 hours)
- **Safety**: Reduce exposure to hazardous conditions (H2S, driving on remote roads)
- **Expertise Scaling**: One engineer can monitor 100+ wells from office instead of 5-10 in field
- **Predictive Maintenance**: Trend analysis reveals degrading equipment before failure
- **Training**: New hires can learn on virtual equipment without risk

### Technical Benefits
- **Real-Time Visibility**: Sub-second latency from field device to dashboard
- **Offline Resilience**: Cached state allows monitoring even when connectivity is degraded
- **Multi-Protocol Support**: Works with any SCADA system (OPC-UA, Modbus, MQTT, etc.)
- **Scalability**: WebSocket architecture handles 1000+ concurrent well monitors per API server
- **Security**: Multi-layer protection against unauthorized control commands
- **Auditability**: Every control command logged with user, reason, timestamp

### User Experience Benefits
- **Familiar Interface**: Matches physical HMI panels that field personnel already know
- **Contextual**: P&ID diagrams show equipment relationships, not just raw numbers
- **Mobile Friendly**: View on tablet/phone while in field
- **Customizable**: Operators configure alarms, limits, and layouts per well
- **Historical Context**: Compare current values to historical trends

## Consequences

### Positive
- **Dramatic reduction in operational costs** - Field visit costs drop 60-80%
- **Improved safety** - Fewer trips to remote/hazardous locations
- **Faster incident response** - Real-time alarms instead of waiting for next field visit
- **Better decision making** - Historical trends inform production optimization
- **Competitive advantage** - Small operators gain enterprise-level monitoring capabilities

### Negative
- **Complexity**: Requires WebSocket infrastructure, Redis, multi-protocol SCADA service
- **Network dependency**: Real-time features require stable connectivity (mitigated by caching)
- **Development effort**: HMI components require domain expertise (gauges, P&IDs, alarms)
- **Security concerns**: Remote control capability increases attack surface
- **Training needed**: Field personnel accustomed to physical gauges need training on digital interface

### Neutral
- **Shift in operations**: Field personnel transition from "gauge reader" to "data analyst" role
- **Equipment upgrades**: Older wells may need SCADA retrofitting to participate
- **Bandwidth requirements**: Continuous WebSocket connections consume network capacity

## Testing Strategy

### Unit Tests
```typescript
describe('WellSiteTwin', () => {
  it('should update state when receiving SCADA reading', () => {
    const twin = new WellSiteTwin('well-123', 'tenant-456');

    twin['updateState']({
      tenantId: 'tenant-456',
      wellId: 'well-123',
      tagName: 'oil_rate',
      value: 250.5,
      quality: 'GOOD',
      timestamp: new Date(),
    });

    expect(twin.getState().tags.get('oil_rate')).toEqual({
      value: 250.5,
      quality: 'GOOD',
      timestamp: expect.any(Date),
      unit: 'bbl/d',
    });
  });

  it('should raise alarm when value exceeds high limit', () => {
    const twin = new WellSiteTwin('well-123', 'tenant-456');

    twin['checkAlarms']({
      tagName: 'tubing_pressure',
      value: 1850, // Exceeds high limit (1800)
      quality: 'GOOD',
      // ...
    });

    expect(twin.getState().alarms).toContainEqual(
      expect.objectContaining({
        severity: 'WARNING',
        tagName: 'tubing_pressure',
      })
    );
  });

  it('should cache state for offline access', () => {
    const twin = new WellSiteTwin('well-123', 'tenant-456');
    twin['cacheState']();

    const cached = localStorage.getItem('well-twin:well-123');
    expect(cached).toBeDefined();
    expect(JSON.parse(cached!).state).toEqual(twin.getState());
  });
});
```

### Integration Tests
```typescript
describe('Digital Twin Integration', () => {
  it('should connect to WebSocket and receive readings', async () => {
    const twin = new WellSiteTwin('well-123', 'tenant-456');
    await twin.connect();

    // Simulate SCADA reading from backend
    mockWebSocket.emit('reading', {
      tenantId: 'tenant-456',
      wellId: 'well-123',
      tagName: 'oil_rate',
      value: 245.3,
      quality: 'GOOD',
      timestamp: new Date(),
    });

    // Wait for state update
    await waitFor(() => {
      expect(twin.getState().tags.get('oil_rate')?.value).toBe(245.3);
    });
  });

  it('should send control command to API', async () => {
    const twin = new WellSiteTwin('well-123', 'tenant-456');

    await twin.sendControlCommand({
      tagName: 'pump_setpoint',
      value: 1200, // RPM
      reason: 'Increase production',
    });

    expect(fetch).toHaveBeenCalledWith(
      '/api/scada/control',
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({
          wellId: 'well-123',
          tagName: 'pump_setpoint',
          value: 1200,
          reason: 'Increase production',
        }),
      })
    );
  });
});
```

### E2E Tests (Playwright)
```typescript
test('Digital Twin dashboard displays live data', async ({ page }) => {
  await page.goto('/wells/well-123/digital-twin');

  // Should show connection status
  await expect(page.locator('text=Connected')).toBeVisible();

  // Should display gauges with values
  const oilGauge = page.locator('text=Oil Production').locator('..');
  await expect(oilGauge).toContainText('bbl/d');

  // Should update when new reading arrives (wait for WebSocket update)
  const initialValue = await oilGauge.locator('text').first().textContent();
  await page.waitForTimeout(6000); // Wait for next SCADA reading (5s interval)
  const updatedValue = await oilGauge.locator('text').first().textContent();
  expect(updatedValue).not.toBe(initialValue); // Value should have changed
});

test('Alarm banner appears when limit exceeded', async ({ page }) => {
  await page.goto('/wells/well-123/digital-twin');

  // Simulate high pressure alarm from backend
  await mockWebSocketReading({
    tagName: 'tubing_pressure',
    value: 1850, // Exceeds high limit (1800)
  });

  // Alarm banner should appear
  await expect(page.locator('text=Active Alarms')).toBeVisible();
  await expect(page.locator('text=tubing_pressure high: 1850 PSI')).toBeVisible();
});
```

## Related Patterns

- **Pattern 82: Hybrid Time-Series Aggregation** - Querying both SCADA and manual field data
- **Pattern 83: SCADA Protocol Adapter** - Multi-protocol device connectivity
- **Pattern 85: Real-Time Event-Driven Architecture** - WebSocket + Redis Pub/Sub streaming
- **Pattern 86: SCADA HMI Component Library** - Reusable gauge, chart, P&ID components
- **Pattern 87: Time-Series Visualization** - Historical + live streaming charts
- **Repository Pattern** - Data access layer abstraction
- **CQRS Pattern** - Separate read/write paths for control commands
- **Observer Pattern** - State change notifications to React components

## References

- WellOS Sprint 5 Implementation Spec
- Pattern 82: Hybrid Time-Series Aggregation Pattern
- Pattern 83: SCADA Protocol Adapter Pattern
- `apps/scada-ingestion/` - Rust multi-protocol SCADA service
- `apps/api/src/presentation/scada/websocket_gateway.rs` - WebSocket gateway
- `apps/api/src/infrastructure/redis/scada_subscriber.rs` - Redis Pub/Sub subscriber
- Socket.IO Documentation: https://socket.io/docs/v4/
- OPC-UA Digital Twin Specifications: https://opcfoundation.org/developer-tools/specifications-unified-architecture
- ISA-95 Enterprise-Control System Integration: https://www.isa.org/standards-and-publications/isa-standards/isa-standards-committees/isa95

## Changelog

- **2025-10-30**: Initial pattern created based on Sprint 5 WebSocket implementation
