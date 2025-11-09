# Frontend Repository Pattern

## Overview

The Repository Pattern in frontend applications provides a centralized
abstraction layer for data access, caching, and state management. Unlike backend
repositories that abstract database access, frontend repositories abstract API
calls, local storage, and cache management.

## Problem Statement

Frontend applications often suffer from:

- **Scattered API calls** throughout components and hooks
- **Inconsistent caching strategies** leading to stale data
- **No centralized error handling** for data operations
- **Difficult testing** due to tight coupling with API services
- **Lack of optimistic updates** for better UX

## Solution

Implement a Repository Pattern that centralizes data access with built-in
caching, error handling, and optimistic updates.

## Implementation

### Base Repository

```typescript
// lib/repositories/base.repository.ts
export abstract class BaseRepository<T extends { id: string }> {
  protected abstract apiService: any;
  protected abstract cacheKey: string;
  protected queryClient: QueryClient;

  constructor(queryClient: QueryClient) {
    this.queryClient = queryClient;
  }

  // Get all records with caching
  async getAll(): Promise<T[]> {
    return this.queryClient.fetchQuery({
      queryKey: [this.cacheKey],
      queryFn: () => this.apiService.getAll(),
      staleTime: 5 * 60 * 1000, // 5 minutes
    });
  }

  // Get single record with caching
  async getById(id: string): Promise<T | null> {
    return this.queryClient.fetchQuery({
      queryKey: [this.cacheKey, id],
      queryFn: () => this.apiService.getById(id),
      staleTime: 5 * 60 * 1000,
    });
  }

  // Create with optimistic update
  async create(data: Omit<T, 'id'>): Promise<T> {
    // Optimistic update
    const tempId = `temp-${Date.now()}`;
    const optimisticItem = { ...data, id: tempId } as T;

    this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
      old ? [...old, optimisticItem] : [optimisticItem],
    );

    try {
      const result = await this.apiService.create(data);

      // Replace optimistic update with real data
      this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
        old ? old.map((item) => (item.id === tempId ? result : item)) : [result],
      );

      return result;
    } catch (error) {
      // Revert optimistic update
      this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
        old ? old.filter((item) => item.id !== tempId) : [],
      );
      throw error;
    }
  }

  // Update with optimistic update
  async update(id: string, data: Partial<T>): Promise<T> {
    // Optimistic update
    this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
      old ? old.map((item) => (item.id === id ? { ...item, ...data } : item)) : [],
    );

    try {
      const result = await this.apiService.update(id, data);

      // Update with real data
      this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
        old ? old.map((item) => (item.id === id ? result : item)) : [result],
      );

      return result;
    } catch (error) {
      // Revert optimistic update - refetch to get current state
      this.queryClient.invalidateQueries({ queryKey: [this.cacheKey] });
      throw error;
    }
  }

  // Delete with optimistic update
  async delete(id: string): Promise<void> {
    // Store item for potential rollback
    const currentData = this.queryClient.getQueryData<T[]>([this.cacheKey]);
    const itemToDelete = currentData?.find((item) => item.id === id);

    // Optimistic update
    this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
      old ? old.filter((item) => item.id !== id) : [],
    );

    try {
      await this.apiService.delete(id);
    } catch (error) {
      // Revert optimistic update
      if (itemToDelete) {
        this.queryClient.setQueryData<T[]>([this.cacheKey], (old) =>
          old ? [...old, itemToDelete] : [itemToDelete],
        );
      }
      throw error;
    }
  }

  // Cache management
  invalidateCache(): void {
    this.queryClient.invalidateQueries({ queryKey: [this.cacheKey] });
  }

  prefetchById(id: string): void {
    this.queryClient.prefetchQuery({
      queryKey: [this.cacheKey, id],
      queryFn: () => this.apiService.getById(id),
    });
  }
}
```

### Concrete Repository Implementation

```typescript
// lib/repositories/user.repository.ts
export class UserRepository extends BaseRepository<User> {
  protected apiService = userApi;
  protected cacheKey = 'users';

  // Domain-specific methods
  async getByRole(role: UserRole): Promise<User[]> {
    const users = await this.getAll();
    return users.filter((user) => user.role === role);
  }

  async getActiveUsers(): Promise<User[]> {
    const users = await this.getAll();
    return users.filter((user) => user.isActive && !user.deletedAt);
  }

  async inviteUser(data: InviteUserRequest): Promise<void> {
    await this.apiService.inviteUser(data);
    // Refresh cache to show invited user
    this.invalidateCache();
  }

  async assignRole(id: string, role: UserRole): Promise<User> {
    return this.update(id, { role });
  }

  async toggleStatus(id: string, isActive: boolean): Promise<User> {
    return this.update(id, { isActive });
  }

  // Soft delete support
  async softDelete(id: string): Promise<void> {
    await this.update(id, {
      deletedAt: new Date().toISOString(),
      deletedBy: 'current-user-id', // Get from auth context
    });
  }

  async restore(id: string): Promise<User> {
    return this.update(id, {
      deletedAt: undefined,
      deletedBy: undefined,
    });
  }
}
```

### Repository Hook Integration

```typescript
// hooks/use-repository.ts
export function useRepository<T extends { id: string }>(repository: BaseRepository<T>) {
  const queryClient = useQueryClient();

  return {
    // Queries
    useGetAll: () =>
      useQuery({
        queryKey: [repository.cacheKey],
        queryFn: () => repository.getAll(),
      }),

    useGetById: (id: string) =>
      useQuery({
        queryKey: [repository.cacheKey, id],
        queryFn: () => repository.getById(id),
        enabled: !!id,
      }),

    // Mutations
    useCreate: () =>
      useMutation({
        mutationFn: (data: Omit<T, 'id'>) => repository.create(data),
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: [repository.cacheKey] });
        },
      }),

    useUpdate: () =>
      useMutation({
        mutationFn: ({ id, data }: { id: string; data: Partial<T> }) => repository.update(id, data),
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: [repository.cacheKey] });
        },
      }),

    useDelete: () =>
      useMutation({
        mutationFn: (id: string) => repository.delete(id),
        onSuccess: () => {
          queryClient.invalidateQueries({ queryKey: [repository.cacheKey] });
        },
      }),
  };
}
```

### Enhanced Hook with Repository

```typescript
// hooks/use-users.ts
export function useUsers() {
  const queryClient = useQueryClient();
  const userRepository = new UserRepository(queryClient);
  const repositoryHooks = useRepository(userRepository);

  return {
    // Data queries
    users: repositoryHooks.useGetAll(),
    getUserById: repositoryHooks.useGetById,

    // Domain-specific queries
    useActiveUsers: () =>
      useQuery({
        queryKey: ['users', 'active'],
        queryFn: () => userRepository.getActiveUsers(),
      }),

    useUsersByRole: (role: UserRole) =>
      useQuery({
        queryKey: ['users', 'role', role],
        queryFn: () => userRepository.getByRole(role),
      }),

    // Mutations
    createUser: repositoryHooks.useCreate(),
    updateUser: repositoryHooks.useUpdate(),
    deleteUser: repositoryHooks.useDelete(),

    // Domain-specific mutations
    inviteUser: useMutation({
      mutationFn: (data: InviteUserRequest) => userRepository.inviteUser(data),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['users'] });
        toast.success('User invited successfully!');
      },
    }),

    assignRole: useMutation({
      mutationFn: ({ id, role }: { id: string; role: UserRole }) =>
        userRepository.assignRole(id, role),
      onSuccess: () => {
        toast.success('Role assigned successfully!');
      },
    }),

    softDeleteUser: useMutation({
      mutationFn: (id: string) => userRepository.softDelete(id),
      onSuccess: () => {
        toast.success('User moved to trash');
      },
    }),

    restoreUser: useMutation({
      mutationFn: (id: string) => userRepository.restore(id),
      onSuccess: () => {
        toast.success('User restored successfully');
      },
    }),
  };
}
```

## Benefits

### 1. **Centralized Data Access**

- All API calls go through repositories
- Consistent error handling
- Unified caching strategy

### 2. **Optimistic Updates**

- Better user experience
- Automatic rollback on errors
- Reduced perceived latency

### 3. **Better Testing**

- Easy to mock repositories
- Isolated business logic
- Predictable behavior

### 4. **Cache Management**

- Automatic cache invalidation
- Prefetching capabilities
- Stale data handling

## Best Practices

### 1. **Repository Scope**

```typescript
// ✅ Good: Domain-focused repository
class UserRepository extends BaseRepository<User> {
  async getActiveUsers(): Promise<User[]> {
    /* ... */
  }
  async assignRole(id: string, role: UserRole): Promise<User> {
    /* ... */
  }
}

// ❌ Bad: Generic repository for everything
class DataRepository {
  async getUsers(): Promise<User[]> {
    /* ... */
  }
  async getWells(): Promise<Well[]> {
    /* ... */
  }
  async getLeases(): Promise<Lease[]> {
    /* ... */
  }
}
```

### 2. **Error Handling**

```typescript
// ✅ Good: Specific error handling
async create(data: Omit<T, 'id'>): Promise<T> {
  try {
    return await this.apiService.create(data);
  } catch (error) {
    if (error instanceof ValidationError) {
      throw new RepositoryValidationError(error.message);
    }
    throw new RepositoryError('Failed to create record');
  }
}
```

### 3. **Cache Keys**

```typescript
// ✅ Good: Hierarchical cache keys
const cacheKeys = {
  users: ['users'],
  userById: (id: string) => ['users', id],
  usersByRole: (role: string) => ['users', 'role', role],
  activeUsers: ['users', 'active'],
};
```

## Testing

```typescript
// __tests__/repositories/user.repository.test.ts
describe('UserRepository', () => {
  let repository: UserRepository;
  let mockQueryClient: jest.Mocked<QueryClient>;

  beforeEach(() => {
    mockQueryClient = createMockQueryClient();
    repository = new UserRepository(mockQueryClient);
  });

  it('should create user with optimistic update', async () => {
    const userData = { email: 'test@example.com', firstName: 'Test' };
    const expectedUser = { id: '1', ...userData };

    jest.spyOn(userApi, 'create').mockResolvedValue(expectedUser);

    const result = await repository.create(userData);

    expect(result).toEqual(expectedUser);
    expect(mockQueryClient.setQueryData).toHaveBeenCalledWith(['users'], expect.any(Function));
  });

  it('should rollback optimistic update on error', async () => {
    const userData = { email: 'test@example.com', firstName: 'Test' };
    const error = new Error('API Error');

    jest.spyOn(userApi, 'create').mockRejectedValue(error);

    await expect(repository.create(userData)).rejects.toThrow(error);

    // Verify rollback
    expect(mockQueryClient.setQueryData).toHaveBeenCalledTimes(2);
  });
});
```

## Integration with Existing Code

The Repository Pattern can be gradually introduced:

1. **Start with one entity** (e.g., Users)
2. **Keep existing hooks** as fallback
3. **Migrate component by component**
4. **Remove old code** once proven

This pattern provides the foundation for scalable, maintainable frontend data
management.
