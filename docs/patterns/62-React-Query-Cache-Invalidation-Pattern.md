# Pattern 62: React Query Cache Invalidation Pattern

**Category**: Frontend / State Management
**Complexity**: Intermediate
**Last Updated**: 2025-10-19

## Overview

This pattern provides best practices for cache invalidation in React Query mutations to ensure UI updates are immediate, reliable, and avoid race conditions when multiple mutations execute in rapid succession.

## Problem Statement

When using React Query mutations:

1. **Redundant refetches**: Calling both `invalidateQueries` AND `refetchQueries` causes duplicate network requests
2. **Race conditions**: Using `await` on cache operations in rapid succession can cause stale UI states
3. **Slow UI updates**: Manual `refetch()` calls in components add unnecessary complexity
4. **Batch operations**: Multiple rapid mutations (like bulk approvals) don't update the UI properly

## Solution

### Correct Cache Invalidation (✅ Good)

```typescript
// apps/web/hooks/expense/use-approve-expense-mutation.ts
export function useApproveExpenseMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => expenseRepository.comroveExpense(id),
    onSuccess: (_, id) => {
      // ✅ Just invalidate - React Query handles the refetch automatically
      queryClient.invalidateQueries({ queryKey: ['expenses', id] });
      queryClient.invalidateQueries({ queryKey: ['expenses'] });
    },
    retry: false,
  });
}
```

### Why This Works

1. **No `await`**: `invalidateQueries` returns immediately and schedules refetches in the background
2. **No `refetchQueries`**: Redundant when `invalidateQueries` is used - React Query auto-refetches active queries
3. **Synchronous invalidation**: All cache invalidations happen immediately, preventing race conditions
4. **Batch-safe**: Multiple mutations can invalidate the same cache simultaneously without conflicts

## Anti-Patterns (❌ Bad)

### Anti-Pattern 1: Redundant refetchQueries

```typescript
// ❌ BAD: Both invalidate AND refetch
onSuccess: async (_, id) => {
  await queryClient.invalidateQueries({ queryKey: ['expenses', id] });
  await queryClient.refetchQueries({ queryKey: ['expenses', id] }); // Redundant!
  await queryClient.invalidateQueries({ queryKey: ['expenses'] });
  await queryClient.refetchQueries({ queryKey: ['expenses'] }); // Redundant!
};
```

**Problem**:

- Double network requests for each query
- Potential race conditions with `await`
- Slower UI updates

### Anti-Pattern 2: Manual refetch in Components

```typescript
// ❌ BAD: Manual refetch in component
const { data, refetch } = useExpensesQuery({ status: 'DRAFT' });
const approveMutation = useApproveExpenseMutation();

const handleApprove = async (id: string) => {
  await approveMutation.mutateAsync(id);
  refetch(); // Redundant if mutation hook invalidates cache!
};
```

**Problem**:

- Violates single responsibility principle
- Duplicates cache invalidation logic
- Easy to forget, leading to stale UI

### Anti-Pattern 3: Using await on Invalidation

```typescript
// ❌ BAD: Awaiting invalidation
onSuccess: async (_, id) => {
  await queryClient.invalidateQueries({ queryKey: ['expenses', id] });
  await queryClient.invalidateQueries({ queryKey: ['expenses'] });
};
```

**Problem**:

- Causes sequential invalidation instead of parallel
- Race conditions when multiple mutations execute rapidly
- Unnecessary async overhead

## Implementation Guide

### Step 1: Mutation Hook with Proper Invalidation

```typescript
// apps/web/hooks/resource/use-update-resource-mutation.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';

export function useUpdateResourceMutation() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, data }) => resourceRepository.update(id, data),
    onSuccess: (updatedResource, { id }) => {
      // Invalidate specific resource
      queryClient.invalidateQueries({ queryKey: ['resources', id] });

      // Invalidate list queries
      queryClient.invalidateQueries({ queryKey: ['resources'] });

      // Optional: Invalidate related resources
      if (updatedResource.projectId) {
        queryClient.invalidateQueries({
          queryKey: ['projects', updatedResource.projectId],
        });
      }
    },
    retry: false,
  });
}
```

### Step 2: Component Usage (No Manual Refetch)

```typescript
// Component using the mutation
export function ResourceApprovals() {
  // ✅ No need to destructure 'refetch'
  const { data, isLoading } = useResourcesQuery({ status: 'DRAFT' });
  const approveMutation = useApproveResourceMutation();

  const handleApprove = async (id: string) => {
    try {
      await approveMutation.mutateAsync(id);
      // ✅ No manual refetch - cache invalidation handles it automatically
      toast.success('Approved successfully');
    } catch (error) {
      toast.error('Failed to approve');
    }
  };

  return (
    // Render UI - it will auto-update when cache invalidates
  );
}
```

### Step 3: Bulk Operations

```typescript
// Handle bulk approvals with proper cache invalidation
const handleBulkApprove = async (ids: string[]) => {
  try {
    // Execute all mutations
    await Promise.all(ids.map((id) => approveMutation.mutateAsync(id)));

    // Each mutation's onSuccess will invalidate the cache
    // React Query will batch the refetches automatically
    toast.success(`Approved ${ids.length} items`);
  } catch (error) {
    toast.error('Some approvals failed');
  }
};
```

## When to Use Each Approach

### Use `invalidateQueries` (Most Common)

```typescript
// ✅ Standard approach - let React Query handle refetching
queryClient.invalidateQueries({ queryKey: ['resources'] });
```

**When**:

- Normal CRUD operations
- The query is actively being used in a component
- You want React Query to decide when to refetch based on query state

### Use `setQueryData` (Optimistic Updates)

```typescript
// ✅ For immediate UI feedback without waiting for server
onMutate: async (newData) => {
  await queryClient.cancelQueries({ queryKey: ['resources', id] });

  const previous = queryClient.getQueryData(['resources', id]);

  queryClient.setQueryData(['resources', id], newData);

  return { previous };
},
onError: (err, variables, context) => {
  // Rollback on error
  queryClient.setQueryData(['resources', id], context.previous);
}
```

**When**:

- Optimistic updates for better UX
- You have the new data immediately
- Server response is slow

### Use `refetchQueries` (Rare)

```typescript
// ✅ Only when you need to force refetch inactive queries
queryClient.refetchQueries({
  queryKey: ['resources'],
  type: 'inactive', // Force refetch even if not mounted
});
```

**When**:

- You need to refetch queries that aren't currently active
- Pre-fetching data for upcoming views
- Very rare in normal applications

## Advanced: Query Key Patterns

```typescript
// Define query key factory for consistency
export const resourceKeys = {
  all: ['resources'] as const,
  lists: () => [...resourceKeys.all, 'list'] as const,
  list: (filters: ResourceFilters) => [...resourceKeys.lists(), { filters }] as const,
  details: () => [...resourceKeys.all, 'detail'] as const,
  detail: (id: string) => [...resourceKeys.details(), id] as const,
};

// Usage in mutation
onSuccess: (_, id) => {
  // Invalidate specific resource
  queryClient.invalidateQueries({
    queryKey: resourceKeys.detail(id),
  });

  // Invalidate ALL list queries (all filter combinations)
  queryClient.invalidateQueries({
    queryKey: resourceKeys.lists(),
  });
};
```

## Testing

```typescript
// Test that mutation properly invalidates cache
it('should invalidate cache on successful approval', async () => {
  const queryClient = new QueryClient();
  const { result } = renderHook(() => useApproveExpenseMutation(), {
    wrapper: createWrapper(queryClient),
  });

  const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

  await result.current.mutateAsync('expense-123');

  expect(invalidateSpy).toHaveBeenCalledWith({
    queryKey: ['expenses', 'expense-123'],
  });
  expect(invalidateSpy).toHaveBeenCalledWith({
    queryKey: ['expenses'],
  });
});
```

## Performance Considerations

1. **Invalidation is cheap**: `invalidateQueries` is a synchronous operation that just marks cache as stale
2. **React Query batches refetches**: Multiple rapid invalidations trigger a single batched refetch
3. **Only active queries refetch**: Inactive queries won't refetch until they're used again
4. **Network deduplication**: Multiple components using the same query share one network request

## Related Patterns

- **Pattern 4**: Repository Pattern (data access layer)
- **Pattern 8**: Command Query Responsibility Segregation (CQRS)
- **Pattern 16**: Pattern Integration Guide (when to use which pattern)

## Real-World Example

From `apps/web/hooks/expense/use-approve-expense-mutation.ts`:

```typescript
// Before (had issues with bulk approvals)
onSuccess: async (_, id) => {
  await queryClient.invalidateQueries({ queryKey: ['expenses', id] });
  await queryClient.refetchQueries({ queryKey: ['expenses', id] }); // ❌
  await queryClient.invalidateQueries({ queryKey: ['expenses'] });
  await queryClient.refetchQueries({ queryKey: ['expenses'] }); // ❌
};

// After (works perfectly with bulk approvals)
onSuccess: (_, id) => {
  queryClient.invalidateQueries({ queryKey: ['expenses', id] }); // ✅
  queryClient.invalidateQueries({ queryKey: ['expenses'] }); // ✅
};
```

## References

- [React Query Invalidation Documentation](https://tanstack.com/query/latest/docs/react/guides/query-invalidation)
- [React Query Mutations Documentation](https://tanstack.com/query/latest/docs/react/guides/mutations)
- [React Query Best Practices](https://tkdodo.eu/blog/react-query-render-optimizations)
