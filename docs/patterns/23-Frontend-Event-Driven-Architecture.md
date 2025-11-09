# Frontend Event-Driven Architecture

## Overview

Event-Driven Architecture (EDA) in frontend applications enables loose coupling
between components through asynchronous event communication. This pattern allows
components to react to state changes without direct dependencies, improving
scalability and maintainability.

## Problem Statement

Traditional frontend applications often suffer from:

- **Tight coupling** between components through prop drilling
- **Complex state management** with deeply nested callbacks
- **Difficult feature additions** due to interdependent components
- **Poor scalability** as the application grows
- **Hard to test** components with many dependencies

## Solution

Implement an Event-Driven Architecture using an event bus system that allows
components to communicate through events, reducing coupling and improving
maintainability.

## Implementation

### Event Bus Core

```typescript
// lib/events/event-bus.ts
export type EventCallback<T = any> = (data: T) => void | Promise<void>;
export type EventUnsubscribe = () => void;

export interface IEventBus {
  emit<T>(event: string, data: T): Promise<void>;
  on<T>(event: string, callback: EventCallback<T>): EventUnsubscribe;
  off(event: string, callback: EventCallback): void;
  once<T>(event: string, callback: EventCallback<T>): EventUnsubscribe;
  clear(): void;
}

export class EventBus implements IEventBus {
  private events: Map<string, EventCallback[]> = new Map();
  private onceEvents: Map<string, EventCallback[]> = new Map();

  async emit<T>(event: string, data: T): Promise<void> {
    // Handle regular listeners
    const callbacks = this.events.get(event) || [];
    const promises = callbacks.map((callback) => {
      try {
        return Promise.resolve(callback(data));
      } catch (error) {
        console.error(`Error in event listener for ${event}:`, error);
        return Promise.resolve();
      }
    });

    // Handle once listeners
    const onceCallbacks = this.onceEvents.get(event) || [];
    const oncePromises = onceCallbacks.map((callback) => {
      try {
        return Promise.resolve(callback(data));
      } catch (error) {
        console.error(`Error in once event listener for ${event}:`, error);
        return Promise.resolve();
      }
    });

    // Clear once listeners after execution
    if (onceCallbacks.length > 0) {
      this.onceEvents.delete(event);
    }

    // Wait for all listeners to complete
    await Promise.all([...promises, ...oncePromises]);
  }

  on<T>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    const callbacks = this.events.get(event) || [];
    callbacks.push(callback);
    this.events.set(event, callbacks);

    // Return unsubscribe function
    return () => this.off(event, callback);
  }

  off(event: string, callback: EventCallback): void {
    const callbacks = this.events.get(event) || [];
    const index = callbacks.indexOf(callback);
    if (index > -1) {
      callbacks.splice(index, 1);
      if (callbacks.length === 0) {
        this.events.delete(event);
      } else {
        this.events.set(event, callbacks);
      }
    }
  }

  once<T>(event: string, callback: EventCallback<T>): EventUnsubscribe {
    const callbacks = this.onceEvents.get(event) || [];
    callbacks.push(callback);
    this.onceEvents.set(event, callbacks);

    // Return unsubscribe function
    return () => {
      const onceCallbacks = this.onceEvents.get(event) || [];
      const index = onceCallbacks.indexOf(callback);
      if (index > -1) {
        onceCallbacks.splice(index, 1);
        if (onceCallbacks.length === 0) {
          this.onceEvents.delete(event);
        } else {
          this.onceEvents.set(event, onceCallbacks);
        }
      }
    };
  }

  clear(): void {
    this.events.clear();
    this.onceEvents.clear();
  }

  // Debug methods
  getEventCount(): number {
    return this.events.size + this.onceEvents.size;
  }

  getListenerCount(event: string): number {
    const regular = this.events.get(event)?.length || 0;
    const once = this.onceEvents.get(event)?.length || 0;
    return regular + once;
  }
}

// Global event bus instance
export const eventBus = new EventBus();
```

### Domain Events

```typescript
// lib/events/user.events.ts
export interface UserInvitedEvent {
  email: string;
  organizationId: string;
  invitedBy: string;
  invitedAt: Date;
}

export interface UserRoleAssignedEvent {
  userId: string;
  oldRole: UserRole;
  newRole: UserRole;
  assignedBy: string;
  assignedAt: Date;
}

export interface UserSoftDeletedEvent {
  userId: string;
  deletedBy: string;
  deletedAt: Date;
}

export interface UserRestoredEvent {
  userId: string;
  restoredBy: string;
  restoredAt: Date;
}

export interface UserActivatedEvent {
  userId: string;
  activatedBy: string;
  activatedAt: Date;
}

export interface UserDeactivatedEvent {
  userId: string;
  deactivatedBy: string;
  deactivatedAt: Date;
}

// Event names as constants
export const USER_EVENTS = {
  INVITED: 'user.invited',
  ROLE_ASSIGNED: 'user.role.assigned',
  SOFT_DELETED: 'user.soft.deleted',
  RESTORED: 'user.restored',
  ACTIVATED: 'user.activated',
  DEACTIVATED: 'user.deactivated',
} as const;
```

### Event Publisher Service

```typescript
// lib/events/user-event.publisher.ts
export class UserEventPublisher {
  constructor(private eventBus: IEventBus) {}

  async publishUserInvited(event: UserInvitedEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.INVITED, event);
  }

  async publishRoleAssigned(event: UserRoleAssignedEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.ROLE_ASSIGNED, event);
  }

  async publishUserSoftDeleted(event: UserSoftDeletedEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.SOFT_DELETED, event);
  }

  async publishUserRestored(event: UserRestoredEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.RESTORED, event);
  }

  async publishUserActivated(event: UserActivatedEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.ACTIVATED, event);
  }

  async publishUserDeactivated(event: UserDeactivatedEvent): Promise<void> {
    await this.eventBus.emit(USER_EVENTS.DEACTIVATED, event);
  }
}
```

### React Hook for Event Listening

```typescript
// hooks/use-event-listener.ts
export function useEventListener<T>(
  event: string,
  callback: EventCallback<T>,
  deps: React.DependencyList = [],
): void {
  const callbackRef = useRef(callback);

  // Update callback ref when dependencies change
  useEffect(() => {
    callbackRef.current = callback;
  }, deps);

  useEffect(() => {
    const wrappedCallback = (data: T) => callbackRef.current(data);
    const unsubscribe = eventBus.on(event, wrappedCallback);

    return unsubscribe;
  }, [event]);
}

// Hook for one-time event listening
export function useEventListenerOnce<T>(
  event: string,
  callback: EventCallback<T>,
  deps: React.DependencyList = [],
): void {
  const callbackRef = useRef(callback);

  useEffect(() => {
    callbackRef.current = callback;
  }, deps);

  useEffect(() => {
    const wrappedCallback = (data: T) => callbackRef.current(data);
    const unsubscribe = eventBus.once(event, wrappedCallback);

    return unsubscribe;
  }, [event]);
}

// Hook for emitting events
export function useEventEmitter() {
  return {
    emit: <T>(event: string, data: T) => eventBus.emit(event, data),
  };
}
```

### Event-Driven Components

```typescript
// components/user-management/user-stats.tsx
export function UserStats() {
  const [stats, setStats] = useState({
    total: 0,
    active: 0,
    byRole: { owner: 0, manager: 0, pumper: 0 },
  });

  // Listen to user events and update stats
  useEventListener<UserInvitedEvent>(
    USER_EVENTS.INVITED,
    (event) => {
      setStats(prev => ({
        ...prev,
        total: prev.total + 1,
      }));
      toast.info(`New user invited: ${event.email}`);
    }
  );

  useEventListener<UserRoleAssignedEvent>(
    USER_EVENTS.ROLE_ASSIGNED,
    (event) => {
      setStats(prev => ({
        ...prev,
        byRole: {
          ...prev.byRole,
          [event.oldRole]: prev.byRole[event.oldRole] - 1,
          [event.newRole]: prev.byRole[event.newRole] + 1,
        },
      }));
      toast.info(`User role changed from ${event.oldRole} to ${event.newRole}`);
    }
  );

  useEventListener<UserSoftDeletedEvent>(
    USER_EVENTS.SOFT_DELETED,
    (event) => {
      setStats(prev => ({
        ...prev,
        total: prev.total - 1,
        active: prev.active - 1,
      }));
      toast.warning('User moved to trash');
    }
  );

  useEventListener<UserActivatedEvent>(
    USER_EVENTS.ACTIVATED,
    (event) => {
      setStats(prev => ({
        ...prev,
        active: prev.active + 1,
      }));
    }
  );

  useEventListener<UserDeactivatedEvent>(
    USER_EVENTS.DEACTIVATED,
    (event) => {
      setStats(prev => ({
        ...prev,
        active: prev.active - 1,
      }));
    }
  );

  return (
    <div className="grid grid-cols-4 gap-4">
      <StatCard title="Total Users" value={stats.total} />
      <StatCard title="Active Users" value={stats.active} />
      <StatCard title="Owners" value={stats.byRole.owner} />
      <StatCard title="Managers" value={stats.byRole.manager} />
    </div>
  );
}
```

### Event-Driven Notifications

```typescript
// components/notifications/notification-listener.tsx
export function NotificationListener() {
  useEventListener<UserInvitedEvent>(USER_EVENTS.INVITED, (event) => {
    toast.success(`Invitation sent to ${event.email}`, {
      description: `User will be added to the organization`,
      action: {
        label: 'View Users',
        onClick: () => router.push('/dashboard/users'),
      },
    });
  });

  useEventListener<UserRoleAssignedEvent>(USER_EVENTS.ROLE_ASSIGNED, (event) => {
    toast.success('Role updated successfully', {
      description: `User role changed to ${event.newRole}`,
    });
  });

  useEventListener<UserSoftDeletedEvent>(USER_EVENTS.SOFT_DELETED, (event) => {
    toast.warning('User moved to trash', {
      description: 'User can be restored from the trash',
      action: {
        label: 'Undo',
        onClick: () => {
          // Emit restore event
          eventBus.emit('user.restore.requested', { userId: event.userId });
        },
      },
    });
  });

  return null; // This component only listens to events
}
```

### Event-Driven Data Synchronization

```typescript
// hooks/use-user-sync.ts
export function useUserSync() {
  const queryClient = useQueryClient();

  // Sync user list when users are modified
  useEventListener<UserInvitedEvent>(USER_EVENTS.INVITED, async () => {
    // Invalidate and refetch user queries
    await queryClient.invalidateQueries({ queryKey: ['users'] });
  });

  useEventListener<UserRoleAssignedEvent>(USER_EVENTS.ROLE_ASSIGNED, async (event) => {
    // Update specific user in cache
    queryClient.setQueryData<User[]>(['users'], (old) =>
      old?.map((user) => (user.id === event.userId ? { ...user, role: event.newRole } : user)),
    );
  });

  useEventListener<UserSoftDeletedEvent>(USER_EVENTS.SOFT_DELETED, async (event) => {
    // Remove user from active list
    queryClient.setQueryData<User[]>(['users'], (old) =>
      old?.filter((user) => user.id !== event.userId),
    );

    // Invalidate deleted users query
    await queryClient.invalidateQueries({ queryKey: ['users', 'deleted'] });
  });
}
```

### Integration with Commands

```typescript
// lib/commands/user.commands.ts (updated)
export class InviteUserCommand implements ICommand<InviteUserRequest, void> {
  constructor(
    private userRepository: UserRepository,
    private eventPublisher: UserEventPublisher,
  ) {}

  async execute(request: InviteUserRequest): Promise<void> {
    // Execute business logic
    await this.userRepository.inviteUser(request);

    // Publish domain event
    await this.eventPublisher.publishUserInvited({
      email: request.email,
      organizationId: request.organizationId,
      invitedBy: request.invitedBy,
      invitedAt: new Date(),
    });
  }
}
```

### Event Provider

```typescript
// components/providers/event-provider.tsx
interface EventProviderProps {
  children: React.ReactNode;
}

export function EventProvider({ children }: EventProviderProps) {
  // Initialize event listeners that should be active globally
  useUserSync();

  return (
    <>
      {children}
      <NotificationListener />
    </>
  );
}

// In app/layout.tsx
export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        <QueryClientProvider client={queryClient}>
          <EventProvider>
            {children}
          </EventProvider>
          <Toaster />
        </QueryClientProvider>
      </body>
    </html>
  );
}
```

## Benefits

### 1. **Loose Coupling**

- Components don't need direct references
- Easy to add/remove features
- Better separation of concerns

### 2. **Scalability**

- Easy to add new event listeners
- No performance impact from unused features
- Modular architecture

### 3. **Real-time Updates**

- Automatic UI synchronization
- Consistent state across components
- Better user experience

### 4. **Testability**

- Easy to mock events
- Isolated component testing
- Clear event contracts

## Best Practices

### 1. **Event Naming**

```typescript
// ✅ Good: Hierarchical, past tense
const USER_EVENTS = {
  INVITED: 'user.invited',
  ROLE_ASSIGNED: 'user.role.assigned',
  PROFILE_UPDATED: 'user.profile.updated',
};

// ❌ Bad: Generic or present tense
const EVENTS = {
  USER_INVITE: 'invite',
  ASSIGN_ROLE: 'assign',
  UPDATE: 'update',
};
```

### 2. **Event Data Structure**

```typescript
// ✅ Good: Rich, structured data
interface UserInvitedEvent {
  email: string;
  organizationId: string;
  invitedBy: string;
  invitedAt: Date;
  metadata?: Record<string, any>;
}

// ❌ Bad: Minimal or unstructured data
interface UserInvitedEvent {
  userId: string;
}
```

### 3. **Error Handling**

```typescript
// ✅ Good: Graceful error handling
useEventListener<UserInvitedEvent>(USER_EVENTS.INVITED, async (event) => {
  try {
    await updateUserStats(event);
  } catch (error) {
    console.error('Failed to update user stats:', error);
    // Don't throw - let other listeners continue
  }
});
```

## Testing

```typescript
// __tests__/events/event-bus.test.ts
describe('EventBus', () => {
  let eventBus: EventBus;

  beforeEach(() => {
    eventBus = new EventBus();
  });

  afterEach(() => {
    eventBus.clear();
  });

  it('should emit and receive events', async () => {
    const callback = jest.fn();
    const eventData = { test: 'data' };

    eventBus.on('test.event', callback);
    await eventBus.emit('test.event', eventData);

    expect(callback).toHaveBeenCalledWith(eventData);
  });

  it('should handle once listeners correctly', async () => {
    const callback = jest.fn();
    const eventData = { test: 'data' };

    eventBus.once('test.event', callback);

    await eventBus.emit('test.event', eventData);
    await eventBus.emit('test.event', eventData);

    expect(callback).toHaveBeenCalledTimes(1);
  });

  it('should unsubscribe correctly', async () => {
    const callback = jest.fn();
    const unsubscribe = eventBus.on('test.event', callback);

    unsubscribe();
    await eventBus.emit('test.event', { test: 'data' });

    expect(callback).not.toHaveBeenCalled();
  });
});
```

This Event-Driven Architecture provides a scalable foundation for component
communication and real-time UI updates.
