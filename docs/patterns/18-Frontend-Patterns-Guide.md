# Frontend Patterns Guide for Next.js Web App

## Executive Summary

Based on the excellent patterns in the API app, here are the essential patterns
needed for the Next.js web app to achieve the same level of architectural
excellence.

## 🎯 **High Priority Frontend Patterns**

### 1. **Repository Pattern for Frontend Data Management**

**Current State**: Basic API service layer  
**Need**: Centralized data access with caching, optimistic updates, and soft
delete support

```typescript
// Create: lib/repositories/base.repository.ts
export abstract class BaseRepository<T extends { id: string; deletedAt?: Date }> {
  protected abstract apiService: any;
  protected abstract cacheKey: string;

  // Soft delete support
  async softDelete(id: string): Promise<void> {
    await this.apiService.softDelete(id);
    this.invalidateCache();
  }

  // Get all with soft delete filtering
  async getAll(includeDeleted = false): Promise<T[]> {
    const data = await this.apiService.getAll();
    return includeDeleted ? data : data.filter((item) => !item.deletedAt);
  }

  // Optimistic updates
  async update(id: string, data: Partial<T>): Promise<T> {
    // Optimistically update cache
    this.updateCache(id, data);

    try {
      const result = await this.apiService.update(id, data);
      this.updateCache(id, result);
      return result;
    } catch (error) {
      // Revert optimistic update
      this.revertCache(id);
      throw error;
    }
  }

  protected abstract invalidateCache(): void;
  protected abstract updateCache(id: string, data: Partial<T>): void;
  protected abstract revertCache(id: string): void;
}

// Create: lib/repositories/user.repository.ts
export class UserRepository extends BaseRepository<User> {
  protected apiService = userApi;
  protected cacheKey = 'users';

  protected invalidateCache(): void {
    queryClient.invalidateQueries({ queryKey: [this.cacheKey] });
  }

  protected updateCache(id: string, data: Partial<User>): void {
    queryClient.setQueryData([this.cacheKey], (old: User[] | undefined) =>
      old?.map((user) => (user.id === id ? { ...user, ...data } : user)),
    );
  }

  protected revertCache(id: string): void {
    queryClient.invalidateQueries({ queryKey: [this.cacheKey] });
  }
}
```

### 2. **Command/Query Separation (Frontend CQRS)**

**Current State**: Mixed read/write operations in hooks  
**Need**: Separate command and query responsibilities

```typescript
// Create: lib/commands/user.commands.ts
export class UserCommands {
  constructor(private repository: UserRepository) {}

  async inviteUser(data: InviteUserRequest): Promise<void> {
    await this.repository.invite(data);
    // Emit domain event
    eventBus.emit('user.invited', { email: data.email });
  }

  async assignRole(userId: string, role: UserRole): Promise<void> {
    await this.repository.assignRole(userId, role);
    eventBus.emit('user.role.assigned', { userId, role });
  }

  async softDeleteUser(userId: string): Promise<void> {
    await this.repository.softDelete(userId);
    eventBus.emit('user.deleted', { userId });
  }
}

// Create: lib/queries/user.queries.ts
export class UserQueries {
  constructor(private repository: UserRepository) {}

  async getActiveUsers(): Promise<User[]> {
    return this.repository.getAll(false); // exclude deleted
  }

  async getUsersByRole(role: UserRole): Promise<User[]> {
    const users = await this.repository.getAll();
    return users.filter((user) => user.role === role);
  }

  async getDeletedUsers(): Promise<User[]> {
    const users = await this.repository.getAll(true);
    return users.filter((user) => user.deletedAt);
  }
}

// Enhanced hook: hooks/use-users.ts
export function useUsers() {
  const repository = new UserRepository();
  const commands = new UserCommands(repository);
  const queries = new UserQueries(repository);

  // Separate command and query methods
  return {
    // Queries
    users: useQuery({
      queryKey: ['users'],
      queryFn: () => queries.getActiveUsers(),
    }),
    deletedUsers: useQuery({
      queryKey: ['users', 'deleted'],
      queryFn: () => queries.getDeletedUsers(),
    }),

    // Commands
    inviteUser: useMutation({
      mutationFn: (data: InviteUserRequest) => commands.inviteUser(data),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['users'] });
        toast.success('User invited successfully!');
      },
    }),

    softDeleteUser: useMutation({
      mutationFn: (id: string) => commands.softDeleteUser(id),
      onSuccess: () => {
        queryClient.invalidateQueries({ queryKey: ['users'] });
        toast.success('User deleted successfully!');
      },
    }),
  };
}
```

### 3. **Specification Pattern for Complex UI Logic**

**Current State**: Inline filtering and business rules  
**Need**: Reusable business rules for UI components

```typescript
// Create: lib/specifications/user.specifications.ts
export abstract class Specification<T> {
  abstract isSatisfiedBy(candidate: T): boolean;

  and(other: Specification<T>): Specification<T> {
    return new AndSpecification(this, other);
  }

  or(other: Specification<T>): Specification<T> {
    return new OrSpecification(this, other);
  }
}

export class ActiveUserSpecification extends Specification<User> {
  isSatisfiedBy(user: User): boolean {
    return user.isActive && !user.deletedAt;
  }
}

export class UserRoleSpecification extends Specification<User> {
  constructor(private role: UserRole) {
    super();
  }

  isSatisfiedBy(user: User): boolean {
    return user.role === this.role;
  }
}

export class CanInviteUsersSpecification extends Specification<User> {
  isSatisfiedBy(user: User): boolean {
    return ['owner', 'manager'].includes(user.role);
  }
}

// Usage in components:
export function UserList({ users }: { users: User[] }) {
  const activeUsersSpec = new ActiveUserSpecification();
  const managerSpec = new UserRoleSpecification('manager');

  const activeUsers = users.filter(user => activeUsersSpec.isSatisfiedBy(user));
  const activeManagers = users.filter(user =>
    activeUsersSpec.and(managerSpec).isSatisfiedBy(user)
  );

  return (
    <div>
      <h3>Active Users ({activeUsers.length})</h3>
      <h3>Active Managers ({activeManagers.length})</h3>
    </div>
  );
}
```

### 4. **Observer Pattern for UI Events**

**Current State**: Direct prop drilling and callbacks  
**Need**: Event-driven UI updates

```typescript
// Create: lib/events/event-bus.ts
type EventCallback<T = any> = (data: T) => void;

export class EventBus {
  private events: Map<string, EventCallback[]> = new Map();

  emit<T>(event: string, data: T): void {
    const callbacks = this.events.get(event) || [];
    callbacks.forEach(callback => callback(data));
  }

  on<T>(event: string, callback: EventCallback<T>): () => void {
    const callbacks = this.events.get(event) || [];
    callbacks.push(callback);
    this.events.set(event, callbacks);

    // Return unsubscribe function
    return () => {
      const updatedCallbacks = this.events.get(event) || [];
      const index = updatedCallbacks.indexOf(callback);
      if (index > -1) {
        updatedCallbacks.splice(index, 1);
        this.events.set(event, updatedCallbacks);
      }
    };
  }
}

export const eventBus = new EventBus();

// Create: hooks/use-event-listener.ts
export function useEventListener<T>(
  event: string,
  callback: EventCallback<T>
) {
  useEffect(() => {
    const unsubscribe = eventBus.on(event, callback);
    return unsubscribe;
  }, [event, callback]);
}

// Usage in components:
export function UserStats() {
  const [stats, setStats] = useState({ total: 0, active: 0 });

  useEventListener('user.invited', () => {
    setStats(prev => ({ ...prev, total: prev.total + 1 }));
  });

  useEventListener('user.activated', () => {
    setStats(prev => ({ ...prev, active: prev.active + 1 }));
  });

  useEventListener('user.deleted', () => {
    setStats(prev => ({ ...prev, total: prev.total - 1 }));
  });

  return <div>Total: {stats.total}, Active: {stats.active}</div>;
}
```

### 5. **Strategy Pattern for Dynamic UI Behavior**

**Current State**: Hardcoded UI logic  
**Need**: Configurable UI behavior based on user roles/context

```typescript
// Create: lib/strategies/user-action.strategies.ts
export interface UserActionStrategy {
  canEdit(user: User, currentUser: User): boolean;
  canDelete(user: User, currentUser: User): boolean;
  canAssignRole(user: User, currentUser: User): boolean;
  getAvailableActions(user: User, currentUser: User): string[];
}

export class OwnerActionStrategy implements UserActionStrategy {
  canEdit(user: User, currentUser: User): boolean {
    return true; // Owners can edit anyone
  }

  canDelete(user: User, currentUser: User): boolean {
    return user.id !== currentUser.id; // Can't delete self
  }

  canAssignRole(user: User, currentUser: User): boolean {
    return true; // Owners can assign any role
  }

  getAvailableActions(user: User, currentUser: User): string[] {
    const actions = ['edit', 'activate', 'deactivate'];
    if (this.canDelete(user, currentUser)) actions.push('delete');
    if (this.canAssignRole(user, currentUser)) actions.push('assign-role');
    return actions;
  }
}

export class ManagerActionStrategy implements UserActionStrategy {
  canEdit(user: User, currentUser: User): boolean {
    return user.role !== 'owner'; // Can't edit owners
  }

  canDelete(user: User, currentUser: User): boolean {
    return user.role === 'pumper' && user.id !== currentUser.id;
  }

  canAssignRole(user: User, currentUser: User): boolean {
    return user.role === 'pumper'; // Can only assign pumper roles
  }

  getAvailableActions(user: User, currentUser: User): string[] {
    const actions: string[] = [];
    if (this.canEdit(user, currentUser)) actions.push('edit');
    if (this.canDelete(user, currentUser)) actions.push('delete');
    if (this.canAssignRole(user, currentUser)) actions.push('assign-role');
    return actions;
  }
}

// Create: lib/factories/user-action-strategy.factory.ts
export class UserActionStrategyFactory {
  static create(userRole: UserRole): UserActionStrategy {
    switch (userRole) {
      case 'owner':
        return new OwnerActionStrategy();
      case 'manager':
        return new ManagerActionStrategy();
      default:
        return new PumperActionStrategy();
    }
  }
}

// Usage in components:
export function UserActionMenu({ user, currentUser }: Props) {
  const strategy = UserActionStrategyFactory.create(currentUser.role);
  const availableActions = strategy.getAvailableActions(user, currentUser);

  return (
    <DropdownMenu>
      {availableActions.includes('edit') && (
        <DropdownMenuItem onClick={() => onEdit(user)}>
          Edit User
        </DropdownMenuItem>
      )}
      {availableActions.includes('delete') && (
        <DropdownMenuItem onClick={() => onDelete(user)}>
          Delete User
        </DropdownMenuItem>
      )}
      {availableActions.includes('assign-role') && (
        <DropdownMenuItem>
          Assign Role
        </DropdownMenuItem>
      )}
    </DropdownMenu>
  );
}
```

## 🔧 **Medium Priority Frontend Patterns**

### 6. **Factory Pattern for Component Creation**

```typescript
// Create: lib/factories/form.factory.ts
export class FormFactory {
  static createUserForm(type: 'create' | 'edit' | 'invite'): FormConfig {
    const baseFields = [
      { name: 'firstName', type: 'text', required: true },
      { name: 'lastName', type: 'text', required: true },
      { name: 'email', type: 'email', required: true },
    ];

    switch (type) {
      case 'create':
        return {
          fields: [...baseFields, { name: 'role', type: 'select', required: true }],
          submitText: 'Create User',
          validationSchema: createUserSchema,
        };
      case 'edit':
        return {
          fields: [...baseFields, { name: 'isActive', type: 'checkbox' }],
          submitText: 'Update User',
          validationSchema: updateUserSchema,
        };
      case 'invite':
        return {
          fields: [...baseFields, { name: 'organizationId', type: 'text', required: true }],
          submitText: 'Send Invitation',
          validationSchema: inviteUserSchema,
        };
    }
  }
}
```

### 7. **State Management Pattern with Zustand**

```typescript
// Create: lib/stores/user.store.ts
interface UserState {
  users: User[];
  deletedUsers: User[];
  loading: boolean;
  error: string | null;

  // Actions
  setUsers: (users: User[]) => void;
  addUser: (user: User) => void;
  updateUser: (id: string, updates: Partial<User>) => void;
  softDeleteUser: (id: string) => void;
  restoreUser: (id: string) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

export const useUserStore = create<UserState>((set, get) => ({
  users: [],
  deletedUsers: [],
  loading: false,
  error: null,

  setUsers: (users) => set({ users }),

  addUser: (user) =>
    set((state) => ({
      users: [...state.users, user],
    })),

  updateUser: (id, updates) =>
    set((state) => ({
      users: state.users.map((user) => (user.id === id ? { ...user, ...updates } : user)),
    })),

  softDeleteUser: (id) =>
    set((state) => {
      const user = state.users.find((u) => u.id === id);
      if (!user) return state;

      const deletedUser = { ...user, deletedAt: new Date() };
      return {
        users: state.users.filter((u) => u.id !== id),
        deletedUsers: [...state.deletedUsers, deletedUser],
      };
    }),

  restoreUser: (id) =>
    set((state) => {
      const user = state.deletedUsers.find((u) => u.id === id);
      if (!user) return state;

      const restoredUser = { ...user, deletedAt: undefined };
      return {
        users: [...state.users, restoredUser],
        deletedUsers: state.deletedUsers.filter((u) => u.id !== id),
      };
    }),

  setLoading: (loading) => set({ loading }),
  setError: (error) => set({ error }),
}));
```

## 🛡️ **Soft Delete Implementation**

### Universal Soft Delete Support

```typescript
// Create: lib/types/soft-deletable.ts
export interface SoftDeletable {
  id: string;
  deletedAt?: Date;
  deletedBy?: string;
}

// Create: lib/utils/soft-delete.utils.ts
export class SoftDeleteUtils {
  static isDeleted<T extends SoftDeletable>(item: T): boolean {
    return !!item.deletedAt;
  }

  static filterActive<T extends SoftDeletable>(items: T[]): T[] {
    return items.filter(item => !this.isDeleted(item));
  }

  static filterDeleted<T extends SoftDeletable>(items: T[]): T[] {
    return items.filter(item => this.isDeleted(item));
  }

  static markAsDeleted<T extends SoftDeletable>(
    item: T,
    deletedBy?: string
  ): T {
    return {
      ...item,
      deletedAt: new Date(),
      deletedBy,
    };
  }

  static restore<T extends SoftDeletable>(item: T): T {
    const { deletedAt, deletedBy, ...restored } = item;
    return restored as T;
  }
}

// Usage in components:
export function UserList({ users }: { users: User[] }) {
  const [showDeleted, setShowDeleted] = useState(false);

  const displayUsers = showDeleted
    ? SoftDeleteUtils.filterDeleted(users)
    : SoftDeleteUtils.filterActive(users);

  return (
    <div>
      <button onClick={() => setShowDeleted(!showDeleted)}>
        {showDeleted ? 'Show Active' : 'Show Deleted'} Users
      </button>

      {displayUsers.map(user => (
        <UserCard
          key={user.id}
          user={user}
          isDeleted={SoftDeleteUtils.isDeleted(user)}
        />
      ))}
    </div>
  );
}
```

## 📁 **Recommended Directory Structure**

```
apps/web/
├── app/                    # Next.js App Router
├── components/             # UI Components
│   ├── ui/                # ShadCN components
│   ├── forms/             # Form components
│   ├── providers/         # Context providers
│   └── domain/            # Domain-specific components
├── lib/                   # Core libraries
│   ├── api/               # API services
│   ├── repositories/      # Data repositories
│   ├── commands/          # Command handlers
│   ├── queries/           # Query handlers
│   ├── specifications/    # Business rules
│   ├── strategies/        # Strategy implementations
│   ├── factories/         # Factory classes
│   ├── events/            # Event system
│   ├── stores/            # State management
│   └── utils/             # Utilities
├── hooks/                 # Custom React hooks
├── types/                 # TypeScript definitions
└── __tests__/             # Test files
```

## 🚀 **Implementation Priority**

### Phase 1: Foundation (Week 1)

1. ✅ **Repository Pattern** - Centralized data access
2. ✅ **Soft Delete Utils** - Universal soft delete support
3. ✅ **Event Bus** - Event-driven architecture

### Phase 2: Business Logic (Week 2)

1. ✅ **Command/Query Separation** - Clean data operations
2. ✅ **Specification Pattern** - Reusable business rules
3. ✅ **Strategy Pattern** - Dynamic UI behavior

### Phase 3: Enhancement (Week 3)

1. ✅ **Factory Pattern** - Component/form generation
2. ✅ **Enhanced State Management** - Zustand integration
3. ✅ **Advanced Event Handling** - Complex UI interactions

This architecture will give your frontend the same level of excellence as your
API, with proper separation of concerns, testability, and maintainability.
