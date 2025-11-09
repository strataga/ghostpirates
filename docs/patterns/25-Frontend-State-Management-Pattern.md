# Frontend State Management Pattern

## Overview

The State Management Pattern in React applications provides a structured
approach to managing application state using Zustand, React Query, and local
component state. This pattern ensures predictable state updates, optimal
performance, and clear separation between different types of state.

## Problem Statement

Frontend applications often struggle with:

- **Complex state management** across multiple components
- **Prop drilling** for shared state
- **Inconsistent state updates** leading to bugs
- **Poor performance** due to unnecessary re-renders
- **Difficult debugging** of state changes
- **Mixed concerns** between server state and client state

## Solution

Implement a layered state management approach using different tools for
different types of state:

- **Zustand** for global client state
- **React Query** for server state
- **Local state** for component-specific state

## Implementation

### State Classification

```typescript
// lib/state/types.ts
export interface StateClassification {
  // Server State - Data from APIs
  serverState: {
    users: User[];
    organizations: Organization[];
    wells: Well[];
  };

  // Global Client State - Shared across components
  globalClientState: {
    currentUser: User | null;
    theme: 'light' | 'dark';
    sidebarCollapsed: boolean;
    notifications: Notification[];
  };

  // Local Component State - Component-specific
  localState: {
    formData: any;
    dialogOpen: boolean;
    selectedItems: string[];
  };
}
```

### Zustand Store Implementation

```typescript
// lib/stores/app.store.ts
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

interface AppState {
  // Current user
  currentUser: User | null;
  setCurrentUser: (user: User | null) => void;

  // Theme
  theme: 'light' | 'dark';
  toggleTheme: () => void;

  // UI State
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (collapsed: boolean) => void;

  // Notifications
  notifications: Notification[];
  addNotification: (notification: Omit<Notification, 'id'>) => void;
  removeNotification: (id: string) => void;
  clearNotifications: () => void;

  // Loading states
  globalLoading: boolean;
  setGlobalLoading: (loading: boolean) => void;

  // Error handling
  globalError: string | null;
  setGlobalError: (error: string | null) => void;
}

export const useAppStore = create<AppState>()(
  devtools(
    persist(
      (set, get) => ({
        // Current user
        currentUser: null,
        setCurrentUser: (user) => set({ currentUser: user }),

        // Theme
        theme: 'light',
        toggleTheme: () =>
          set((state) => ({
            theme: state.theme === 'light' ? 'dark' : 'light',
          })),

        // UI State
        sidebarCollapsed: false,
        setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),

        // Notifications
        notifications: [],
        addNotification: (notification) =>
          set((state) => ({
            notifications: [...state.notifications, { ...notification, id: crypto.randomUUID() }],
          })),
        removeNotification: (id) =>
          set((state) => ({
            notifications: state.notifications.filter((n) => n.id !== id),
          })),
        clearNotifications: () => set({ notifications: [] }),

        // Loading states
        globalLoading: false,
        setGlobalLoading: (loading) => set({ globalLoading: loading }),

        // Error handling
        globalError: null,
        setGlobalError: (error) => set({ globalError: error }),
      }),
      {
        name: 'app-store',
        partialize: (state) => ({
          theme: state.theme,
          sidebarCollapsed: state.sidebarCollapsed,
          currentUser: state.currentUser,
        }),
      },
    ),
    { name: 'AppStore' },
  ),
);
```

### Domain-Specific Stores

```typescript
// lib/stores/user.store.ts
interface UserState {
  // UI State
  selectedUsers: string[];
  userFilters: UserFilters;
  showDeletedUsers: boolean;

  // Actions
  setSelectedUsers: (userIds: string[]) => void;
  toggleUserSelection: (userId: string) => void;
  clearSelection: () => void;
  setUserFilters: (filters: UserFilters) => void;
  setShowDeletedUsers: (show: boolean) => void;

  // Computed values
  selectedUserCount: number;
  hasSelection: boolean;
}

export const useUserStore = create<UserState>()(
  devtools(
    (set, get) => ({
      // UI State
      selectedUsers: [],
      userFilters: {
        role: null,
        status: 'active',
        search: '',
      },
      showDeletedUsers: false,

      // Actions
      setSelectedUsers: (userIds) => set({ selectedUsers: userIds }),

      toggleUserSelection: (userId) =>
        set((state) => {
          const isSelected = state.selectedUsers.includes(userId);
          return {
            selectedUsers: isSelected
              ? state.selectedUsers.filter((id) => id !== userId)
              : [...state.selectedUsers, userId],
          };
        }),

      clearSelection: () => set({ selectedUsers: [] }),

      setUserFilters: (filters) => set({ userFilters: filters }),

      setShowDeletedUsers: (show) => set({ showDeletedUsers: show }),

      // Computed values (using getters)
      get selectedUserCount() {
        return get().selectedUsers.length;
      },

      get hasSelection() {
        return get().selectedUsers.length > 0;
      },
    }),
    { name: 'UserStore' },
  ),
);
```

### Server State Management with React Query

```typescript
// lib/queries/user.queries.ts
export const userQueries = {
  // Query keys
  keys: {
    all: ['users'] as const,
    lists: () => [...userQueries.keys.all, 'list'] as const,
    list: (filters: UserFilters) => [...userQueries.keys.lists(), filters] as const,
    details: () => [...userQueries.keys.all, 'detail'] as const,
    detail: (id: string) => [...userQueries.keys.details(), id] as const,
    stats: () => [...userQueries.keys.all, 'stats'] as const,
  },

  // Query functions
  list: (filters: UserFilters) => ({
    queryKey: userQueries.keys.list(filters),
    queryFn: () => userApi.getUsers(filters),
    staleTime: 5 * 60 * 1000, // 5 minutes
  }),

  detail: (id: string) => ({
    queryKey: userQueries.keys.detail(id),
    queryFn: () => userApi.getUserById(id),
    staleTime: 10 * 60 * 1000, // 10 minutes
  }),

  stats: () => ({
    queryKey: userQueries.keys.stats(),
    queryFn: () => userApi.getUserStats(),
    staleTime: 2 * 60 * 1000, // 2 minutes
  }),
};

// Mutations
export const userMutations = {
  invite: () => ({
    mutationFn: (data: InviteUserRequest) => userApi.inviteUser(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: userQueries.keys.lists() });
      queryClient.invalidateQueries({ queryKey: userQueries.keys.stats() });
    },
  }),

  update: () => ({
    mutationFn: ({ id, data }: { id: string; data: UpdateUserRequest }) =>
      userApi.updateUser(id, data),
    onSuccess: (updatedUser: User) => {
      // Update specific user in cache
      queryClient.setQueryData(userQueries.keys.detail(updatedUser.id), updatedUser);
      // Invalidate lists to reflect changes
      queryClient.invalidateQueries({ queryKey: userQueries.keys.lists() });
    },
  }),

  softDelete: () => ({
    mutationFn: (id: string) => userApi.softDeleteUser(id),
    onMutate: async (id: string) => {
      // Optimistic update
      await queryClient.cancelQueries({ queryKey: userQueries.keys.lists() });

      const previousUsers = queryClient.getQueriesData({
        queryKey: userQueries.keys.lists(),
      });

      // Remove user from all list queries
      queryClient.setQueriesData(
        { queryKey: userQueries.keys.lists() },
        (old: User[] | undefined) => old?.filter((user) => user.id !== id),
      );

      return { previousUsers };
    },
    onError: (err, id, context) => {
      // Revert optimistic update
      if (context?.previousUsers) {
        context.previousUsers.forEach(([queryKey, data]) => {
          queryClient.setQueryData(queryKey, data);
        });
      }
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: userQueries.keys.lists() });
    },
  }),
};
```

### Integrated Hooks

```typescript
// hooks/use-user-state.ts
export function useUserState() {
  // Zustand store for UI state
  const {
    selectedUsers,
    userFilters,
    showDeletedUsers,
    setSelectedUsers,
    toggleUserSelection,
    clearSelection,
    setUserFilters,
    setShowDeletedUsers,
    selectedUserCount,
    hasSelection,
  } = useUserStore();

  // React Query for server state
  const usersQuery = useQuery(userQueries.list(userFilters));
  const statsQuery = useQuery(userQueries.stats());

  // Mutations
  const inviteMutation = useMutation(userMutations.invite());
  const updateMutation = useMutation(userMutations.update());
  const softDeleteMutation = useMutation(userMutations.softDelete());

  // Computed values
  const filteredUsers = useMemo(() => {
    if (!usersQuery.data) return [];

    let users = usersQuery.data;

    if (!showDeletedUsers) {
      users = users.filter((user) => !user.deletedAt);
    }

    if (userFilters.search) {
      const search = userFilters.search.toLowerCase();
      users = users.filter(
        (user) =>
          user.firstName.toLowerCase().includes(search) ||
          user.lastName.toLowerCase().includes(search) ||
          user.email.toLowerCase().includes(search),
      );
    }

    return users;
  }, [usersQuery.data, showDeletedUsers, userFilters.search]);

  // Actions
  const actions = {
    // Server actions
    inviteUser: inviteMutation.mutate,
    updateUser: updateMutation.mutate,
    softDeleteUser: softDeleteMutation.mutate,

    // UI actions
    selectUser: toggleUserSelection,
    selectAllUsers: () => setSelectedUsers(filteredUsers.map((u) => u.id)),
    clearSelection,
    updateFilters: setUserFilters,
    toggleShowDeleted: () => setShowDeletedUsers(!showDeletedUsers),

    // Bulk actions
    bulkDelete: () => {
      selectedUsers.forEach((id) => softDeleteMutation.mutate(id));
      clearSelection();
    },
  };

  return {
    // Data
    users: filteredUsers,
    stats: statsQuery.data,
    selectedUsers,
    userFilters,
    showDeletedUsers,

    // Computed
    selectedUserCount,
    hasSelection,

    // Loading states
    loading: usersQuery.isLoading,
    statsLoading: statsQuery.isLoading,
    inviting: inviteMutation.isPending,
    updating: updateMutation.isPending,
    deleting: softDeleteMutation.isPending,

    // Error states
    error: usersQuery.error,
    statsError: statsQuery.error,

    // Actions
    ...actions,
  };
}
```

### Local State Management

```typescript
// hooks/use-local-state.ts
export function useLocalState<T>(initialState: T) {
  const [state, setState] = useState<T>(initialState);

  const updateState = useCallback((updates: Partial<T>) => {
    setState(prev => ({ ...prev, ...updates }));
  }, []);

  const resetState = useCallback(() => {
    setState(initialState);
  }, [initialState]);

  return {
    state,
    setState,
    updateState,
    resetState,
  };
}

// Usage in components
export function UserInviteDialog() {
  const { state, updateState, resetState } = useLocalState({
    open: false,
    loading: false,
    formData: {
      email: '',
      firstName: '',
      lastName: '',
      role: 'pumper' as UserRole,
    },
  });

  const { inviteUser } = useUserState();

  const handleSubmit = async (data: InviteUserRequest) => {
    updateState({ loading: true });
    try {
      await inviteUser(data);
      resetState();
      toast.success('User invited successfully!');
    } catch (error) {
      toast.error('Failed to invite user');
    } finally {
      updateState({ loading: false });
    }
  };

  return (
    <Dialog open={state.open} onOpenChange={(open) => updateState({ open })}>
      {/* Dialog content */}
    </Dialog>
  );
}
```

### State Persistence

```typescript
// lib/stores/persisted.store.ts
interface PersistedState {
  userPreferences: {
    tablePageSize: number;
    defaultFilters: UserFilters;
    favoriteViews: string[];
  };
  recentSearches: string[];
  lastVisitedPages: string[];
}

export const usePersistedStore = create<PersistedState>()(
  persist(
    (set, get) => ({
      userPreferences: {
        tablePageSize: 10,
        defaultFilters: {
          role: null,
          status: 'active',
          search: '',
        },
        favoriteViews: [],
      },
      recentSearches: [],
      lastVisitedPages: [],

      // Actions
      updateUserPreferences: (preferences: Partial<UserPreferences>) =>
        set((state) => ({
          userPreferences: { ...state.userPreferences, ...preferences },
        })),

      addRecentSearch: (search: string) =>
        set((state) => ({
          recentSearches: [search, ...state.recentSearches.filter((s) => s !== search)].slice(
            0,
            10,
          ),
        })),

      addVisitedPage: (page: string) =>
        set((state) => ({
          lastVisitedPages: [page, ...state.lastVisitedPages.filter((p) => p !== page)].slice(0, 5),
        })),
    }),
    {
      name: 'user-preferences',
      version: 1,
    },
  ),
);
```

### State Debugging

```typescript
// lib/stores/debug.ts
export const useDebugStore = create<{
  stateHistory: any[];
  addStateSnapshot: (state: any) => void;
  clearHistory: () => void;
}>()(
  devtools(
    (set, get) => ({
      stateHistory: [],
      addStateSnapshot: (state) =>
        set((prev) => ({
          stateHistory: [
            ...prev.stateHistory,
            {
              timestamp: Date.now(),
              state,
            },
          ].slice(-50), // Keep last 50 snapshots
        })),
      clearHistory: () => set({ stateHistory: [] }),
    }),
    { name: 'DebugStore' },
  ),
);

// Development helper
if (process.env.NODE_ENV === 'development') {
  // Log state changes
  useAppStore.subscribe((state) => {
    console.log('App State Changed:', state);
    useDebugStore.getState().addStateSnapshot(state);
  });
}
```

## Benefits

### 1. **Clear Separation of Concerns**

- Server state managed by React Query
- Global client state managed by Zustand
- Local state managed by React hooks

### 2. **Optimal Performance**

- Minimal re-renders with Zustand
- Automatic caching with React Query
- Selective subscriptions

### 3. **Developer Experience**

- DevTools integration
- Time-travel debugging
- State persistence

### 4. **Predictable State Updates**

- Immutable updates
- Clear action patterns
- Easy testing

## Best Practices

### 1. **State Classification**

```typescript
// ✅ Good: Clear state classification
const serverState = useQuery(userQueries.list());
const globalState = useAppStore();
const localState = useState();

// ❌ Bad: Mixed state management
const [users, setUsers] = useState(); // Should be server state
const globalUsers = useAppStore((state) => state.users); // Should be server state
```

### 2. **Store Organization**

```typescript
// ✅ Good: Domain-focused stores
const useUserStore = create(() => ({
  /* user-specific state */
}));
const useWellStore = create(() => ({
  /* well-specific state */
}));

// ❌ Bad: Monolithic store
const useAppStore = create(() => ({
  users: [],
  wells: [],
  leases: [],
}));
```

### 3. **Action Naming**

```typescript
// ✅ Good: Clear action names
const { setSelectedUsers, toggleUserSelection, clearSelection } = useUserStore();

// ❌ Bad: Generic action names
const { set, toggle, clear } = useUserStore();
```

This State Management Pattern provides a scalable, performant, and maintainable
approach to handling all types of state in your React application.
