# Frontend Proxy Pattern

## Overview

The Proxy Pattern in React applications provides a surrogate or placeholder that
controls access to another object. This pattern is essential for implementing
intelligent caching, offline support, request optimization, lazy loading, and
access control in complex oil & gas applications.

## Problem Statement

Modern frontend applications need to:

- **Cache API responses** intelligently to improve performance
- **Support offline functionality** when network is unavailable
- **Optimize network requests** by batching, deduplication, and throttling
- **Implement lazy loading** for expensive operations
- **Control access** to sensitive data or operations
- **Add logging and monitoring** to API calls
- **Handle rate limiting** and retry logic

Direct API calls lead to:

- **Poor performance** due to unnecessary network requests
- **No offline support** when connectivity is lost
- **Inconsistent caching** strategies across the application
- **Difficult monitoring** of API usage
- **Complex retry logic** scattered throughout components

## Solution

Implement the Proxy Pattern to create intelligent intermediaries that handle
caching, offline support, request optimization, and access control
transparently.

## Implementation

### Base Proxy Interface

```typescript
// lib/proxy/interfaces.ts
export interface ApiProxy<T> {
  get(id: string | number): Promise<T>;
  getAll(params?: QueryParams): Promise<T[]>;
  create(data: Partial<T>): Promise<T>;
  update(id: string | number, data: Partial<T>): Promise<T>;
  delete(id: string | number): Promise<void>;
  search(query: string, params?: QueryParams): Promise<T[]>;
}

export interface CacheConfig {
  ttl: number; // Time to live in milliseconds
  maxSize: number; // Maximum number of cached items
  strategy: 'lru' | 'fifo' | 'ttl';
  persistToStorage: boolean;
  storageKey: string;
}

export interface OfflineConfig {
  enabled: boolean;
  storageKey: string;
  syncOnReconnect: boolean;
  maxOfflineActions: number;
}

export interface RequestConfig {
  timeout: number;
  retries: number;
  retryDelay: number;
  batchSize: number;
  debounceMs: number;
}

export interface CachedItem<T> {
  data: T;
  timestamp: number;
  ttl: number;
  key: string;
}

export interface OfflineAction {
  id: string;
  type: 'create' | 'update' | 'delete';
  resource: string;
  data: any;
  timestamp: number;
}
```

### Cache Manager

```typescript
// lib/proxy/cache-manager.ts
export class CacheManager<T> {
  private cache = new Map<string, CachedItem<T>>();
  private config: CacheConfig;

  constructor(config: CacheConfig) {
    this.config = config;

    if (config.persistToStorage) {
      this.loadFromStorage();
    }

    // Cleanup expired items periodically
    setInterval(() => this.cleanup(), 60000); // Every minute
  }

  set(key: string, data: T, customTtl?: number): void {
    const ttl = customTtl || this.config.ttl;
    const item: CachedItem<T> = {
      data,
      timestamp: Date.now(),
      ttl,
      key,
    };

    // Remove oldest item if cache is full
    if (this.cache.size >= this.config.maxSize) {
      this.evictOldest();
    }

    this.cache.set(key, item);

    if (this.config.persistToStorage) {
      this.saveToStorage();
    }
  }

  get(key: string): T | null {
    const item = this.cache.get(key);

    if (!item) {
      return null;
    }

    // Check if item has expired
    if (Date.now() - item.timestamp > item.ttl) {
      this.cache.delete(key);
      return null;
    }

    return item.data;
  }

  has(key: string): boolean {
    return this.get(key) !== null;
  }

  delete(key: string): void {
    this.cache.delete(key);

    if (this.config.persistToStorage) {
      this.saveToStorage();
    }
  }

  clear(): void {
    this.cache.clear();

    if (this.config.persistToStorage) {
      localStorage.removeItem(this.config.storageKey);
    }
  }

  getStats(): { size: number; hitRate: number; memoryUsage: number } {
    return {
      size: this.cache.size,
      hitRate: 0, // Would need to track hits/misses
      memoryUsage: this.estimateMemoryUsage(),
    };
  }

  private evictOldest(): void {
    if (this.config.strategy === 'lru') {
      // Remove least recently used (first in Map)
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    } else if (this.config.strategy === 'fifo') {
      // Remove first in, first out
      const firstKey = this.cache.keys().next().value;
      if (firstKey) {
        this.cache.delete(firstKey);
      }
    } else if (this.config.strategy === 'ttl') {
      // Remove item with shortest remaining TTL
      let shortestTtlKey: string | null = null;
      let shortestRemainingTtl = Infinity;

      for (const [key, item] of this.cache.entries()) {
        const remainingTtl = item.ttl - (Date.now() - item.timestamp);
        if (remainingTtl < shortestRemainingTtl) {
          shortestRemainingTtl = remainingTtl;
          shortestTtlKey = key;
        }
      }

      if (shortestTtlKey) {
        this.cache.delete(shortestTtlKey);
      }
    }
  }

  private cleanup(): void {
    const now = Date.now();
    const expiredKeys: string[] = [];

    for (const [key, item] of this.cache.entries()) {
      if (now - item.timestamp > item.ttl) {
        expiredKeys.push(key);
      }
    }

    expiredKeys.forEach((key) => this.cache.delete(key));

    if (expiredKeys.length > 0 && this.config.persistToStorage) {
      this.saveToStorage();
    }
  }

  private saveToStorage(): void {
    try {
      const serializable = Array.from(this.cache.entries());
      localStorage.setItem(this.config.storageKey, JSON.stringify(serializable));
    } catch (error) {
      console.warn('Failed to save cache to storage:', error);
    }
  }

  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(this.config.storageKey);
      if (stored) {
        const entries = JSON.parse(stored);
        this.cache = new Map(entries);

        // Clean up expired items
        this.cleanup();
      }
    } catch (error) {
      console.warn('Failed to load cache from storage:', error);
    }
  }

  private estimateMemoryUsage(): number {
    // Rough estimation of memory usage
    let size = 0;
    for (const item of this.cache.values()) {
      size += JSON.stringify(item).length * 2; // Rough estimate
    }
    return size;
  }
}
```

### Offline Manager

```typescript
// lib/proxy/offline-manager.ts
export class OfflineManager {
  private offlineActions: OfflineAction[] = [];
  private config: OfflineConfig;
  private isOnline: boolean = navigator.onLine;

  constructor(config: OfflineConfig) {
    this.config = config;
    this.loadOfflineActions();
    this.setupOnlineListener();
  }

  isOffline(): boolean {
    return !this.isOnline;
  }

  addOfflineAction(action: Omit<OfflineAction, 'id' | 'timestamp'>): void {
    if (this.offlineActions.length >= this.config.maxOfflineActions) {
      // Remove oldest action
      this.offlineActions.shift();
    }

    const offlineAction: OfflineAction = {
      ...action,
      id: crypto.randomUUID(),
      timestamp: Date.now(),
    };

    this.offlineActions.push(offlineAction);
    this.saveOfflineActions();
  }

  getOfflineActions(): OfflineAction[] {
    return [...this.offlineActions];
  }

  clearOfflineActions(): void {
    this.offlineActions = [];
    this.saveOfflineActions();
  }

  async syncOfflineActions(apiService: any): Promise<void> {
    if (this.isOffline() || this.offlineActions.length === 0) {
      return;
    }

    const actionsToSync = [...this.offlineActions];
    const syncResults: Array<{
      action: OfflineAction;
      success: boolean;
      error?: string;
    }> = [];

    for (const action of actionsToSync) {
      try {
        switch (action.type) {
          case 'create':
            await apiService.create(action.resource, action.data);
            break;
          case 'update':
            await apiService.update(action.resource, action.data.id, action.data);
            break;
          case 'delete':
            await apiService.delete(action.resource, action.data.id);
            break;
        }

        syncResults.push({ action, success: true });

        // Remove successfully synced action
        this.offlineActions = this.offlineActions.filter((a) => a.id !== action.id);
      } catch (error) {
        syncResults.push({
          action,
          success: false,
          error: error instanceof Error ? error.message : 'Unknown error',
        });
      }
    }

    this.saveOfflineActions();
    return syncResults;
  }

  private setupOnlineListener(): void {
    window.addEventListener('online', () => {
      this.isOnline = true;
      if (this.config.syncOnReconnect) {
        // Trigger sync (would need to be implemented by the proxy)
        window.dispatchEvent(new CustomEvent('offline-sync-requested'));
      }
    });

    window.addEventListener('offline', () => {
      this.isOnline = false;
    });
  }

  private saveOfflineActions(): void {
    try {
      localStorage.setItem(this.config.storageKey, JSON.stringify(this.offlineActions));
    } catch (error) {
      console.warn('Failed to save offline actions:', error);
    }
  }

  private loadOfflineActions(): void {
    try {
      const stored = localStorage.getItem(this.config.storageKey);
      if (stored) {
        this.offlineActions = JSON.parse(stored);
      }
    } catch (error) {
      console.warn('Failed to load offline actions:', error);
      this.offlineActions = [];
    }
  }
}
```

### API Proxy Implementation

```typescript
// lib/proxy/api-proxy.ts
export class ApiServiceProxy<T> implements ApiProxy<T> {
  private cacheManager: CacheManager<T>;
  private offlineManager: OfflineManager;
  private realApiService: ApiProxy<T>;
  private requestConfig: RequestConfig;
  private pendingRequests = new Map<string, Promise<any>>();

  constructor(
    realApiService: ApiProxy<T>,
    cacheConfig: CacheConfig,
    offlineConfig: OfflineConfig,
    requestConfig: RequestConfig,
  ) {
    this.realApiService = realApiService;
    this.cacheManager = new CacheManager<T>(cacheConfig);
    this.offlineManager = new OfflineManager(offlineConfig);
    this.requestConfig = requestConfig;

    // Setup offline sync listener
    window.addEventListener('offline-sync-requested', () => {
      this.syncOfflineActions();
    });
  }

  async get(id: string | number): Promise<T> {
    const cacheKey = `get-${id}`;

    // Check cache first
    const cached = this.cacheManager.get(cacheKey);
    if (cached) {
      return cached;
    }

    // Check for pending request
    if (this.pendingRequests.has(cacheKey)) {
      return this.pendingRequests.get(cacheKey);
    }

    // If offline, try to return cached data or throw error
    if (this.offlineManager.isOffline()) {
      throw new Error('Data not available offline');
    }

    // Make request with deduplication
    const requestPromise = this.makeRequest(() => this.realApiService.get(id));
    this.pendingRequests.set(cacheKey, requestPromise);

    try {
      const result = await requestPromise;
      this.cacheManager.set(cacheKey, result);
      return result;
    } finally {
      this.pendingRequests.delete(cacheKey);
    }
  }

  async getAll(params?: QueryParams): Promise<T[]> {
    const cacheKey = `getAll-${JSON.stringify(params || {})}`;

    // Check cache first
    const cached = this.cacheManager.get(cacheKey);
    if (cached) {
      return cached as T[];
    }

    // Check for pending request
    if (this.pendingRequests.has(cacheKey)) {
      return this.pendingRequests.get(cacheKey);
    }

    // If offline, try to return cached data or throw error
    if (this.offlineManager.isOffline()) {
      throw new Error('Data not available offline');
    }

    const requestPromise = this.makeRequest(() => this.realApiService.getAll(params));
    this.pendingRequests.set(cacheKey, requestPromise);

    try {
      const result = await requestPromise;
      this.cacheManager.set(cacheKey, result as any);
      return result;
    } finally {
      this.pendingRequests.delete(cacheKey);
    }
  }

  async create(data: Partial<T>): Promise<T> {
    if (this.offlineManager.isOffline()) {
      // Store action for later sync
      this.offlineManager.addOfflineAction({
        type: 'create',
        resource: this.getResourceName(),
        data,
      });

      // Return optimistic result
      return { ...data, id: `temp-${Date.now()}` } as T;
    }

    const result = await this.makeRequest(() => this.realApiService.create(data));

    // Invalidate relevant cache entries
    this.invalidateCache('getAll');

    return result;
  }

  async update(id: string | number, data: Partial<T>): Promise<T> {
    if (this.offlineManager.isOffline()) {
      // Store action for later sync
      this.offlineManager.addOfflineAction({
        type: 'update',
        resource: this.getResourceName(),
        data: { ...data, id },
      });

      // Return optimistic result
      const cached = this.cacheManager.get(`get-${id}`);
      const optimisticResult = { ...cached, ...data } as T;
      this.cacheManager.set(`get-${id}`, optimisticResult);
      return optimisticResult;
    }

    const result = await this.makeRequest(() => this.realApiService.update(id, data));

    // Update cache
    this.cacheManager.set(`get-${id}`, result);
    this.invalidateCache('getAll');

    return result;
  }

  async delete(id: string | number): Promise<void> {
    if (this.offlineManager.isOffline()) {
      // Store action for later sync
      this.offlineManager.addOfflineAction({
        type: 'delete',
        resource: this.getResourceName(),
        data: { id },
      });

      // Optimistically remove from cache
      this.cacheManager.delete(`get-${id}`);
      return;
    }

    await this.makeRequest(() => this.realApiService.delete(id));

    // Remove from cache
    this.cacheManager.delete(`get-${id}`);
    this.invalidateCache('getAll');
  }

  async search(query: string, params?: QueryParams): Promise<T[]> {
    const cacheKey = `search-${query}-${JSON.stringify(params || {})}`;

    // Check cache first
    const cached = this.cacheManager.get(cacheKey);
    if (cached) {
      return cached as T[];
    }

    // If offline, return empty results or cached data
    if (this.offlineManager.isOffline()) {
      return [];
    }

    const result = await this.makeRequest(() => this.realApiService.search(query, params));

    // Cache with shorter TTL for search results
    this.cacheManager.set(cacheKey, result as any, 300000); // 5 minutes

    return result;
  }

  private async makeRequest<R>(requestFn: () => Promise<R>): Promise<R> {
    let lastError: Error;

    for (let attempt = 0; attempt <= this.requestConfig.retries; attempt++) {
      try {
        const timeoutPromise = new Promise<never>((_, reject) => {
          setTimeout(() => reject(new Error('Request timeout')), this.requestConfig.timeout);
        });

        const result = await Promise.race([requestFn(), timeoutPromise]);
        return result;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error('Unknown error');

        if (attempt < this.requestConfig.retries) {
          await this.delay(this.requestConfig.retryDelay * Math.pow(2, attempt));
        }
      }
    }

    throw lastError!;
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  private invalidateCache(pattern: string): void {
    // Simple pattern matching for cache invalidation
    // In a real implementation, this would be more sophisticated
    if (pattern === 'getAll') {
      // Clear all getAll cache entries
      // Implementation would depend on cache structure
    }
  }

  private getResourceName(): string {
    // Extract resource name from API service
    // This is a simplified implementation
    return 'resource';
  }

  private async syncOfflineActions(): Promise<void> {
    try {
      await this.offlineManager.syncOfflineActions(this.realApiService);
    } catch (error) {
      console.error('Failed to sync offline actions:', error);
    }
  }

  // Utility methods
  getCacheStats() {
    return this.cacheManager.getStats();
  }

  getOfflineActions() {
    return this.offlineManager.getOfflineActions();
  }

  clearCache() {
    this.cacheManager.clear();
  }

  clearOfflineActions() {
    this.offlineManager.clearOfflineActions();
  }
}
```

### React Hook Integration

```typescript
// hooks/use-api-proxy.ts
export function useApiProxy<T>(
  apiService: ApiProxy<T>,
  config: {
    cache?: Partial<CacheConfig>;
    offline?: Partial<OfflineConfig>;
    request?: Partial<RequestConfig>;
  } = {},
) {
  const [proxy] = useState(() => {
    const cacheConfig: CacheConfig = {
      ttl: 300000, // 5 minutes
      maxSize: 100,
      strategy: 'lru',
      persistToStorage: true,
      storageKey: 'api-cache',
      ...config.cache,
    };

    const offlineConfig: OfflineConfig = {
      enabled: true,
      storageKey: 'offline-actions',
      syncOnReconnect: true,
      maxOfflineActions: 50,
      ...config.offline,
    };

    const requestConfig: RequestConfig = {
      timeout: 10000,
      retries: 3,
      retryDelay: 1000,
      batchSize: 10,
      debounceMs: 300,
      ...config.request,
    };

    return new ApiServiceProxy(apiService, cacheConfig, offlineConfig, requestConfig);
  });

  const [isOnline, setIsOnline] = useState(navigator.onLine);

  useEffect(() => {
    const handleOnline = () => setIsOnline(true);
    const handleOffline = () => setIsOnline(false);

    window.addEventListener('online', handleOnline);
    window.addEventListener('offline', handleOffline);

    return () => {
      window.removeEventListener('online', handleOnline);
      window.removeEventListener('offline', handleOffline);
    };
  }, []);

  return {
    proxy,
    isOnline,
    getCacheStats: () => proxy.getCacheStats(),
    getOfflineActions: () => proxy.getOfflineActions(),
    clearCache: () => proxy.clearCache(),
    clearOfflineActions: () => proxy.clearOfflineActions(),
  };
}

// hooks/use-cached-data.ts
export function useCachedData<T>(
  fetchFn: () => Promise<T>,
  dependencies: any[] = [],
  cacheKey?: string,
) {
  const [data, setData] = useState<T | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const cacheManager = useMemo(
    () =>
      new CacheManager<T>({
        ttl: 300000,
        maxSize: 50,
        strategy: 'lru',
        persistToStorage: true,
        storageKey: cacheKey || 'cached-data',
      }),
    [cacheKey],
  );

  const fetchData = useCallback(async () => {
    const key = cacheKey || JSON.stringify(dependencies);

    // Check cache first
    const cached = cacheManager.get(key);
    if (cached) {
      setData(cached);
      setLoading(false);
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const result = await fetchFn();
      cacheManager.set(key, result);
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [fetchFn, cacheManager, cacheKey, ...dependencies]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const refetch = useCallback(() => {
    const key = cacheKey || JSON.stringify(dependencies);
    cacheManager.delete(key);
    fetchData();
  }, [fetchData, cacheManager, cacheKey, ...dependencies]);

  return { data, loading, error, refetch };
}
```

### Component Usage

```typescript
// components/well-list.tsx
export function WellList() {
  const wellApiService = useWellApiService();
  const { proxy, isOnline, getCacheStats } = useApiProxy(wellApiService, {
    cache: {
      ttl: 600000, // 10 minutes for well data
      maxSize: 200,
      storageKey: 'well-cache',
    },
    offline: {
      enabled: true,
      storageKey: 'well-offline-actions',
      maxOfflineActions: 100,
    },
  });

  const [wells, setWells] = useState<Well[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadWells();
  }, []);

  const loadWells = async () => {
    try {
      setLoading(true);
      const data = await proxy.getAll();
      setWells(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load wells');
    } finally {
      setLoading(false);
    }
  };

  const handleCreateWell = async (wellData: Partial<Well>) => {
    try {
      const newWell = await proxy.create(wellData);
      setWells(prev => [...prev, newWell]);

      if (!isOnline) {
        toast.info('Well created offline. Will sync when connection is restored.');
      } else {
        toast.success('Well created successfully');
      }
    } catch (err) {
      toast.error('Failed to create well');
    }
  };

  const handleUpdateWell = async (id: string, updates: Partial<Well>) => {
    try {
      const updatedWell = await proxy.update(id, updates);
      setWells(prev => prev.map(well => well.id === id ? updatedWell : well));

      if (!isOnline) {
        toast.info('Well updated offline. Will sync when connection is restored.');
      } else {
        toast.success('Well updated successfully');
      }
    } catch (err) {
      toast.error('Failed to update well');
    }
  };

  return (
    <div className="space-y-4">
      {/* Connection Status */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Wells</h2>
        <div className="flex items-center gap-2">
          <div className={`w-3 h-3 rounded-full ${isOnline ? 'bg-green-500' : 'bg-red-500'}`} />
          <span className="text-sm text-muted-foreground">
            {isOnline ? 'Online' : 'Offline'}
          </span>
        </div>
      </div>

      {/* Cache Stats */}
      <div className="text-sm text-muted-foreground">
        Cache: {getCacheStats().size} items
      </div>

      {/* Well List */}
      {loading ? (
        <div>Loading wells...</div>
      ) : error ? (
        <div className="text-red-600">Error: {error}</div>
      ) : (
        <div className="grid gap-4">
          {wells.map(well => (
            <WellCard
              key={well.id}
              well={well}
              onUpdate={(updates) => handleUpdateWell(well.id, updates)}
            />
          ))}
        </div>
      )}

      <Button onClick={() => handleCreateWell({ name: 'New Well' })}>
        Add Well
      </Button>
    </div>
  );
}
```

## Benefits

### 1. **Intelligent Caching**

- Automatic caching with configurable TTL and eviction strategies
- Persistent cache across browser sessions
- Cache invalidation and refresh strategies

### 2. **Offline Support**

- Seamless offline functionality with optimistic updates
- Automatic sync when connection is restored
- Queue management for offline actions

### 3. **Performance Optimization**

- Request deduplication and batching
- Retry logic with exponential backoff
- Timeout handling and error recovery

### 4. **Transparent Operation**

- Same interface as original API service
- No changes required in existing components
- Easy to enable/disable proxy features

## Best Practices

### 1. **Cache Configuration**

```typescript
// ✅ Good: Appropriate cache settings
const cacheConfig = {
  ttl: 300000, // 5 minutes for dynamic data
  maxSize: 100,
  strategy: 'lru',
};

// ❌ Bad: Inappropriate cache settings
const cacheConfig = {
  ttl: 86400000, // 24 hours for frequently changing data
  maxSize: 10000, // Too large, memory issues
};
```

### 2. **Error Handling**

```typescript
// ✅ Good: Graceful degradation
if (this.offlineManager.isOffline()) {
  const cached = this.cacheManager.get(cacheKey);
  if (cached) {
    return cached;
  }
  throw new Error('Data not available offline');
}

// ❌ Bad: No offline handling
return this.realApiService.get(id); // Fails when offline
```

### 3. **Memory Management**

```typescript
// ✅ Good: Limited cache size
const cacheConfig = {
  maxSize: 100,
  strategy: 'lru',
};

// ❌ Bad: Unlimited cache
const cacheConfig = {
  maxSize: Infinity, // Memory leak risk
};
```

## Testing

```typescript
// __tests__/proxy/api-proxy.test.ts
describe('ApiServiceProxy', () => {
  let mockApiService: jest.Mocked<ApiProxy<any>>;
  let proxy: ApiServiceProxy<any>;

  beforeEach(() => {
    mockApiService = {
      get: jest.fn(),
      getAll: jest.fn(),
      create: jest.fn(),
      update: jest.fn(),
      delete: jest.fn(),
      search: jest.fn(),
    };

    proxy = new ApiServiceProxy(
      mockApiService,
      {
        ttl: 1000,
        maxSize: 10,
        strategy: 'lru',
        persistToStorage: false,
        storageKey: 'test',
      },
      {
        enabled: true,
        storageKey: 'test-offline',
        syncOnReconnect: false,
        maxOfflineActions: 10,
      },
      {
        timeout: 5000,
        retries: 2,
        retryDelay: 100,
        batchSize: 5,
        debounceMs: 100,
      },
    );
  });

  it('should cache GET requests', async () => {
    const testData = { id: 1, name: 'Test' };
    mockApiService.get.mockResolvedValue(testData);

    // First call
    const result1 = await proxy.get(1);
    expect(result1).toEqual(testData);
    expect(mockApiService.get).toHaveBeenCalledTimes(1);

    // Second call should use cache
    const result2 = await proxy.get(1);
    expect(result2).toEqual(testData);
    expect(mockApiService.get).toHaveBeenCalledTimes(1); // Still 1
  });

  it('should handle offline operations', async () => {
    // Simulate offline
    Object.defineProperty(navigator, 'onLine', {
      value: false,
      writable: true,
    });

    const newData = { name: 'New Item' };
    const result = await proxy.create(newData);

    expect(result).toHaveProperty('id');
    expect(mockApiService.create).not.toHaveBeenCalled();
    expect(proxy.getOfflineActions()).toHaveLength(1);
  });
});
```

The Proxy Pattern provides a powerful way to add intelligent caching, offline
support, and request optimization to your React applications while maintaining a
clean, transparent interface.
