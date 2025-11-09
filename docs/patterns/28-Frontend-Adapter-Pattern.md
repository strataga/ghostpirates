# Frontend Adapter Pattern

## Overview

The Adapter Pattern in frontend applications provides a way to integrate with
external APIs and services by creating a consistent interface that translates
between different data formats and protocols. This pattern is essential for oil
& gas applications that need to integrate with regulatory databases, mapping
services, and third-party data providers.

## Problem Statement

Frontend applications often need to integrate with multiple external systems
that have:

- **Different API formats** and response structures
- **Varying authentication methods** and protocols
- **Inconsistent data models** and naming conventions
- **Different error handling** approaches
- **Changing external interfaces** that break existing code

## Solution

Implement the Adapter Pattern to create a consistent internal interface while
handling the complexity of external system integration behind the scenes.

## Implementation

### Base Adapter Interface

```typescript
// lib/adapters/interfaces.ts
export interface ApiAdapter<TRequest, TResponse> {
  request(data: TRequest): Promise<TResponse>;
  isAvailable(): Promise<boolean>;
  getProviderInfo(): ProviderInfo;
}

export interface ProviderInfo {
  name: string;
  version: string;
  baseUrl: string;
  rateLimit?: RateLimit;
  authentication: AuthType;
}

export interface RateLimit {
  requestsPerMinute: number;
  requestsPerHour: number;
  requestsPerDay: number;
}

export type AuthType = 'api-key' | 'oauth' | 'basic' | 'bearer' | 'none';
```

### Regulatory API Adapter

```typescript
// lib/adapters/regulatory-api.adapter.ts
export interface RegulatoryApiAdapter {
  validateApiNumber(apiNumber: string): Promise<ApiValidationResult>;
  getWellData(apiNumber: string): Promise<RegulatoryWellData>;
  submitReport(report: RegulatoryReport): Promise<SubmissionResult>;
  getPermitStatus(permitNumber: string): Promise<PermitStatus>;
}

export interface ApiValidationResult {
  isValid: boolean;
  details: {
    format: boolean;
    exists: boolean;
    active: boolean;
  };
  errors: string[];
  warnings: string[];
}

export interface RegulatoryWellData {
  apiNumber: string;
  operatorName: string;
  wellName: string;
  location: {
    latitude: number;
    longitude: number;
    county: string;
    state: string;
  };
  status: WellStatus;
  permits: Permit[];
  lastInspection?: Date;
}

// Texas Railroad Commission Adapter
export class TexasRRCAdapter implements RegulatoryApiAdapter {
  private baseUrl = 'https://api.rrc.texas.gov/v1';
  private apiKey: string;
  private rateLimiter: RateLimiter;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
    this.rateLimiter = new RateLimiter({
      requestsPerMinute: 60,
      requestsPerHour: 1000,
    });
  }

  async validateApiNumber(apiNumber: string): Promise<ApiValidationResult> {
    await this.rateLimiter.waitForSlot();

    try {
      const response = await fetch(`${this.baseUrl}/wells/validate/${apiNumber}`, {
        headers: {
          Authorization: `Bearer ${this.apiKey}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new AdapterError(`Texas RRC API error: ${response.status}`);
      }

      const data = await response.json();

      // Transform Texas RRC response to internal format
      return this.transformValidationResponse(data);
    } catch (error) {
      throw new AdapterError(`Failed to validate API number: ${error.message}`);
    }
  }

  async getWellData(apiNumber: string): Promise<RegulatoryWellData> {
    await this.rateLimiter.waitForSlot();

    const response = await fetch(`${this.baseUrl}/wells/${apiNumber}`, {
      headers: {
        Authorization: `Bearer ${this.apiKey}`,
      },
    });

    const data = await response.json();
    return this.transformWellDataResponse(data);
  }

  async submitReport(report: RegulatoryReport): Promise<SubmissionResult> {
    await this.rateLimiter.waitForSlot();

    const transformedReport = this.transformReportToTexasFormat(report);

    const response = await fetch(`${this.baseUrl}/reports`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(transformedReport),
    });

    const result = await response.json();
    return this.transformSubmissionResponse(result);
  }

  private transformValidationResponse(data: any): ApiValidationResult {
    return {
      isValid: data.valid === true,
      details: {
        format: data.format_valid === true,
        exists: data.well_exists === true,
        active: data.status === 'ACTIVE',
      },
      errors: data.errors || [],
      warnings: data.warnings || [],
    };
  }

  private transformWellDataResponse(data: any): RegulatoryWellData {
    return {
      apiNumber: data.api_number,
      operatorName: data.operator_name,
      wellName: data.well_name,
      location: {
        latitude: parseFloat(data.surface_latitude),
        longitude: parseFloat(data.surface_longitude),
        county: data.county_name,
        state: 'TX',
      },
      status: this.mapWellStatus(data.well_status),
      permits: data.permits?.map(this.transformPermit) || [],
      lastInspection: data.last_inspection_date ? new Date(data.last_inspection_date) : undefined,
    };
  }

  private transformReportToTexasFormat(report: RegulatoryReport): any {
    return {
      api_number: report.apiNumber,
      report_type: report.type,
      reporting_period: {
        start_date: report.period.startDate.toISOString().split('T')[0],
        end_date: report.period.endDate.toISOString().split('T')[0],
      },
      production_data: {
        oil_volume: report.production.oil,
        gas_volume: report.production.gas,
        water_volume: report.production.water,
      },
      // ... other transformations
    };
  }
}

// New Mexico Oil Conservation Division Adapter
export class NewMexicoOCDAdapter implements RegulatoryApiAdapter {
  private baseUrl = 'https://api.emnrd.nm.gov/ocd/v2';
  private credentials: BasicAuth;

  constructor(username: string, password: string) {
    this.credentials = { username, password };
  }

  async validateApiNumber(apiNumber: string): Promise<ApiValidationResult> {
    const response = await fetch(`${this.baseUrl}/wells/lookup`, {
      method: 'POST',
      headers: {
        Authorization: `Basic ${btoa(`${this.credentials.username}:${this.credentials.password}`)}`,
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ api_number: apiNumber }),
    });

    const data = await response.json();
    return this.transformNewMexicoValidationResponse(data);
  }

  // ... other methods with New Mexico specific implementations

  private transformNewMexicoValidationResponse(data: any): ApiValidationResult {
    return {
      isValid: data.found === true && data.status !== 'INVALID',
      details: {
        format: data.format_check === 'PASS',
        exists: data.found === true,
        active: data.status === 'ACTIVE',
      },
      errors: data.validation_errors || [],
      warnings: data.validation_warnings || [],
    };
  }
}
```

### Mapping Service Adapter

```typescript
// lib/adapters/mapping-service.adapter.ts
export interface MappingServiceAdapter {
  geocodeAddress(address: string): Promise<GeocodeResult>;
  reverseGeocode(lat: number, lng: number): Promise<ReverseGeocodeResult>;
  getStaticMap(options: StaticMapOptions): Promise<string>;
  calculateDistance(from: Coordinates, to: Coordinates): Promise<number>;
}

export interface GeocodeResult {
  coordinates: Coordinates;
  formattedAddress: string;
  accuracy: 'exact' | 'approximate' | 'low';
  components: AddressComponents;
}

// Google Maps Adapter
export class GoogleMapsAdapter implements MappingServiceAdapter {
  private apiKey: string;
  private baseUrl = 'https://maps.googleapis.com/maps/api';

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async geocodeAddress(address: string): Promise<GeocodeResult> {
    const response = await fetch(
      `${this.baseUrl}/geocode/json?address=${encodeURIComponent(address)}&key=${this.apiKey}`,
    );

    const data = await response.json();

    if (data.status !== 'OK' || !data.results.length) {
      throw new AdapterError(`Geocoding failed: ${data.status}`);
    }

    return this.transformGoogleGeocodeResponse(data.results[0]);
  }

  async getStaticMap(options: StaticMapOptions): Promise<string> {
    const params = new URLSearchParams({
      center: `${options.center.lat},${options.center.lng}`,
      zoom: options.zoom.toString(),
      size: `${options.width}x${options.height}`,
      maptype: options.mapType || 'roadmap',
      key: this.apiKey,
    });

    // Add markers
    options.markers?.forEach((marker, index) => {
      params.comend('markers', `color:${marker.color}|${marker.lat},${marker.lng}`);
    });

    return `${this.baseUrl}/staticmap?${params.toString()}`;
  }

  private transformGoogleGeocodeResponse(result: any): GeocodeResult {
    return {
      coordinates: {
        lat: result.geometry.location.lat,
        lng: result.geometry.location.lng,
      },
      formattedAddress: result.formatted_address,
      accuracy: this.mapGoogleAccuracy(result.geometry.location_type),
      components: this.extractAddressComponents(result.address_components),
    };
  }
}

// Mapbox Adapter
export class MapboxAdapter implements MappingServiceAdapter {
  private accessToken: string;
  private baseUrl = 'https://api.mapbox.com';

  constructor(accessToken: string) {
    this.accessToken = accessToken;
  }

  async geocodeAddress(address: string): Promise<GeocodeResult> {
    const response = await fetch(
      `${this.baseUrl}/geocoding/v5/mapbox.places/${encodeURIComponent(address)}.json?access_token=${this.accessToken}`,
    );

    const data = await response.json();

    if (!data.features.length) {
      throw new AdapterError('No geocoding results found');
    }

    return this.transformMapboxGeocodeResponse(data.features[0]);
  }

  // ... other methods with Mapbox specific implementations
}
```

### Adapter Factory

```typescript
// lib/adapters/adapter.factory.ts
export class AdapterFactory {
  private static regulatoryAdapters = new Map<string, () => RegulatoryApiAdapter>();
  private static mappingAdapters = new Map<string, () => MappingServiceAdapter>();

  static registerRegulatoryAdapter(state: string, factory: () => RegulatoryApiAdapter): void {
    this.regulatoryAdapters.set(state.toLowerCase(), factory);
  }

  static registerMappingAdapter(provider: string, factory: () => MappingServiceAdapter): void {
    this.mappingAdapters.set(provider.toLowerCase(), factory);
  }

  static createRegulatoryAdapter(state: string): RegulatoryApiAdapter {
    const factory = this.regulatoryAdapters.get(state.toLowerCase());
    if (!factory) {
      throw new Error(`No regulatory adapter registered for state: ${state}`);
    }
    return factory();
  }

  static createMappingAdapter(provider: string): MappingServiceAdapter {
    const factory = this.mappingAdapters.get(provider.toLowerCase());
    if (!factory) {
      throw new Error(`No mapping adapter registered for provider: ${provider}`);
    }
    return factory();
  }
}

// Register adapters
AdapterFactory.registerRegulatoryAdapter(
  'texas',
  () => new TexasRRCAdapter(process.env.TEXAS_RRC_API_KEY!),
);

AdapterFactory.registerRegulatoryAdapter(
  'new-mexico',
  () => new NewMexicoOCDAdapter(process.env.NM_OCD_USERNAME!, process.env.NM_OCD_PASSWORD!),
);

AdapterFactory.registerMappingAdapter(
  'google',
  () => new GoogleMapsAdapter(process.env.GOOGLE_MAPS_API_KEY!),
);

AdapterFactory.registerMappingAdapter(
  'mapbox',
  () => new MapboxAdapter(process.env.MAPBOX_ACCESS_TOKEN!),
);
```

### React Hook Integration

```typescript
// hooks/use-regulatory-adapter.ts
export function useRegulatoryAdapter(state: string) {
  const [adapter, setAdapter] = useState<RegulatoryApiAdapter | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    try {
      const regulatoryAdapter = AdapterFactory.createRegulatoryAdapter(state);
      setAdapter(regulatoryAdapter);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
      setAdapter(null);
    } finally {
      setLoading(false);
    }
  }, [state]);

  const validateApiNumber = useCallback(
    async (apiNumber: string) => {
      if (!adapter) throw new Error('Adapter not available');
      return adapter.validateApiNumber(apiNumber);
    },
    [adapter],
  );

  const getWellData = useCallback(
    async (apiNumber: string) => {
      if (!adapter) throw new Error('Adapter not available');
      return adapter.getWellData(apiNumber);
    },
    [adapter],
  );

  const submitReport = useCallback(
    async (report: RegulatoryReport) => {
      if (!adapter) throw new Error('Adapter not available');
      return adapter.submitReport(report);
    },
    [adapter],
  );

  return {
    adapter,
    loading,
    error,
    validateApiNumber,
    getWellData,
    submitReport,
  };
}

// hooks/use-mapping-adapter.ts
export function useMappingAdapter(provider: string = 'google') {
  const [adapter, setAdapter] = useState<MappingServiceAdapter | null>(null);

  useEffect(() => {
    const mappingAdapter = AdapterFactory.createMappingAdapter(provider);
    setAdapter(mappingAdapter);
  }, [provider]);

  const geocodeAddress = useCallback(
    async (address: string) => {
      if (!adapter) throw new Error('Mapping adapter not available');
      return adapter.geocodeAddress(address);
    },
    [adapter],
  );

  const getStaticMap = useCallback(
    async (options: StaticMapOptions) => {
      if (!adapter) throw new Error('Mapping adapter not available');
      return adapter.getStaticMap(options);
    },
    [adapter],
  );

  return {
    adapter,
    geocodeAddress,
    getStaticMap,
  };
}
```

### Component Usage

```typescript
// components/well-management/api-number-validator.tsx
export function ApiNumberValidator({ state, onValidation }: Props) {
  const { validateApiNumber, loading, error } = useRegulatoryAdapter(state);
  const [apiNumber, setApiNumber] = useState('');
  const [validationResult, setValidationResult] = useState<ApiValidationResult | null>(null);

  const handleValidate = async () => {
    try {
      const result = await validateApiNumber(apiNumber);
      setValidationResult(result);
      onValidation(result);
    } catch (err) {
      toast.error('Validation failed: ' + err.message);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex gap-2">
        <Input
          value={apiNumber}
          onChange={(e) => setApiNumber(e.target.value)}
          placeholder="Enter 14-digit API number"
          maxLength={14}
        />
        <Button onClick={handleValidate} disabled={loading || !apiNumber}>
          {loading ? 'Validating...' : 'Validate'}
        </Button>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {validationResult && (
        <ValidationResultDisplay result={validationResult} />
      )}
    </div>
  );
}
```

## Benefits

### 1. **Consistent Interface**

- Uniform API across different external services
- Simplified component integration
- Reduced coupling to external systems

### 2. **Easy Provider Switching**

- Change providers without changing application code
- A/B test different services
- Fallback to alternative providers

### 3. **Error Handling**

- Centralized error handling and transformation
- Consistent error formats across providers
- Graceful degradation when services are unavailable

### 4. **Rate Limiting & Caching**

- Built-in rate limiting for external APIs
- Response caching to reduce API calls
- Cost optimization for paid services

## Best Practices

### 1. **Interface Design**

```typescript
// ✅ Good: Clear, domain-specific interface
interface RegulatoryApiAdapter {
  validateApiNumber(apiNumber: string): Promise<ApiValidationResult>;
  getWellData(apiNumber: string): Promise<RegulatoryWellData>;
}

// ❌ Bad: Generic, unclear interface
interface ApiAdapter {
  request(data: any): Promise<any>;
}
```

### 2. **Error Handling**

```typescript
// ✅ Good: Specific error types
class AdapterError extends Error {
  constructor(
    message: string,
    public provider: string,
    public originalError?: Error,
  ) {
    super(message);
  }
}

// ❌ Bad: Generic errors
throw new Error('Something went wrong');
```

### 3. **Configuration**

```typescript
// ✅ Good: Environment-based configuration
const adapter = new TexasRRCAdapter(process.env.TEXAS_RRC_API_KEY!);

// ❌ Bad: Hardcoded configuration
const adapter = new TexasRRCAdapter('hardcoded-key');
```

## Testing

```typescript
// __tests__/adapters/texas-rrc.adapter.test.ts
describe('TexasRRCAdapter', () => {
  let adapter: TexasRRCAdapter;
  let mockFetch: jest.MockedFunction<typeof fetch>;

  beforeEach(() => {
    mockFetch = jest.fn();
    global.fetch = mockFetch;
    adapter = new TexasRRCAdapter('test-api-key');
  });

  it('should validate API number successfully', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        valid: true,
        format_valid: true,
        well_exists: true,
        status: 'ACTIVE',
      }),
    } as Response);

    const result = await adapter.validateApiNumber('12345678901234');

    expect(result.isValid).toBe(true);
    expect(result.details.format).toBe(true);
    expect(result.details.exists).toBe(true);
    expect(result.details.active).toBe(true);
  });

  it('should handle API errors gracefully', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401,
    } as Response);

    await expect(adapter.validateApiNumber('12345678901234')).rejects.toThrow(
      'Texas RRC API error: 401',
    );
  });
});
```

The Adapter Pattern provides a clean, maintainable way to integrate with
external services while keeping your application code decoupled from specific
provider implementations.
