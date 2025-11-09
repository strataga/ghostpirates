# Frontend Command Query Separation (CQRS)

## Overview

Command Query Separation (CQS) in frontend applications separates operations
that change state (Commands) from operations that return data (Queries). This
pattern improves code organization, testability, and maintainability in React
applications.

## Problem Statement

Frontend applications often mix read and write operations, leading to:

- **Complex hooks** that handle both data fetching and mutations
- **Unclear responsibilities** between different parts of the application
- **Difficult testing** due to mixed concerns
- **Inconsistent error handling** across operations
- **Poor separation of concerns** making code hard to maintain

## Solution

Implement Command Query Separation by creating distinct classes for commands
(write operations) and queries (read operations), with clear interfaces and
responsibilities.

## Implementation

### Base Command and Query Interfaces

```typescript
// lib/cqrs/interfaces.ts
export interface ICommand<TRequest, TResponse> {
  execute(request: TRequest): Promise<TResponse>;
}

export interface IQuery<TRequest, TResponse> {
  execute(request: TRequest): Promise<TResponse>;
}

export interface ICommandHandler<TCommand, TResult> {
  handle(command: TCommand): Promise<TResult>;
}

export interface IQueryHandler<TQuery, TResult> {
  handle(query: TQuery): Promise<TResult>;
}
```

### Command Implementation

```typescript
// lib/commands/user.commands.ts
export class InviteUserCommand implements ICommand<InviteUserRequest, void> {
  constructor(
    private userRepository: UserRepository,
    private eventBus: EventBus,
    private notificationService: NotificationService,
  ) {}

  async execute(request: InviteUserRequest): Promise<void> {
    // Validation
    if (!request.email || !request.organizationId) {
      throw new ValidationError('Email and organization ID are required');
    }

    // Business logic
    const existingUser = await this.userRepository.findByEmail(request.email);
    if (existingUser) {
      throw new BusinessError('User already exists');
    }

    // Execute command
    await this.userRepository.inviteUser(request);

    // Side effects
    await this.notificationService.sendInvitationEmail(request.email);

    // Emit domain event
    this.eventBus.emit('user.invited', {
      email: request.email,
      organizationId: request.organizationId,
      invitedAt: new Date(),
    });
  }
}

export class AssignRoleCommand implements ICommand<AssignRoleRequest, User> {
  constructor(
    private userRepository: UserRepository,
    private eventBus: EventBus,
  ) {}

  async execute(request: AssignRoleRequest): Promise<User> {
    // Validation
    if (!request.userId || !request.role) {
      throw new ValidationError('User ID and role are required');
    }

    // Business rules
    const user = await this.userRepository.getById(request.userId);
    if (!user) {
      throw new NotFoundError('User not found');
    }

    if (user.role === 'owner' && request.role !== 'owner') {
      const ownerCount = await this.userRepository.countByRole('owner');
      if (ownerCount <= 1) {
        throw new BusinessError('Cannot change role of the last owner');
      }
    }

    // Execute command
    const updatedUser = await this.userRepository.assignRole(request.userId, request.role);

    // Emit domain event
    this.eventBus.emit('user.role.assigned', {
      userId: request.userId,
      oldRole: user.role,
      newRole: request.role,
      assignedAt: new Date(),
    });

    return updatedUser;
  }
}

export class SoftDeleteUserCommand implements ICommand<SoftDeleteUserRequest, void> {
  constructor(
    private userRepository: UserRepository,
    private eventBus: EventBus,
  ) {}

  async execute(request: SoftDeleteUserRequest): Promise<void> {
    const { userId, deletedBy } = request;

    // Business rules
    const user = await this.userRepository.getById(userId);
    if (!user) {
      throw new NotFoundError('User not found');
    }

    if (user.role === 'owner') {
      const activeOwners = await this.userRepository.countActiveByRole('owner');
      if (activeOwners <= 1) {
        throw new BusinessError('Cannot delete the last owner');
      }
    }

    // Execute command
    await this.userRepository.softDelete(userId, deletedBy);

    // Emit domain event
    this.eventBus.emit('user.soft-deleted', {
      userId,
      deletedBy,
      deletedAt: new Date(),
    });
  }
}
```

### Query Implementation

```typescript
// lib/queries/user.queries.ts
export class GetUsersQuery implements IQuery<GetUsersRequest, User[]> {
  constructor(private userRepository: UserRepository) {}

  async execute(request: GetUsersRequest): Promise<User[]> {
    const { organizationId, includeDeleted = false, role } = request;

    let users = await this.userRepository.getByOrganization(organizationId, includeDeleted);

    if (role) {
      users = users.filter((user) => user.role === role);
    }

    return users;
  }
}

export class GetUserStatsQuery implements IQuery<GetUserStatsRequest, UserStats> {
  constructor(private userRepository: UserRepository) {}

  async execute(request: GetUserStatsRequest): Promise<UserStats> {
    const { organizationId } = request;

    const users = await this.userRepository.getByOrganization(organizationId);

    return {
      total: users.length,
      active: users.filter((user) => user.isActive).length,
      byRole: {
        owner: users.filter((user) => user.role === 'owner').length,
        manager: users.filter((user) => user.role === 'manager').length,
        pumper: users.filter((user) => user.role === 'pumper').length,
      },
      recentlyCreated: users.filter(
        (user) => new Date(user.createdAt) > new Date(Date.now() - 7 * 24 * 60 * 60 * 1000),
      ).length,
    };
  }
}

export class GetDeletedUsersQuery implements IQuery<GetDeletedUsersRequest, User[]> {
  constructor(private userRepository: UserRepository) {}

  async execute(request: GetDeletedUsersRequest): Promise<User[]> {
    const { organizationId } = request;

    const users = await this.userRepository.getByOrganization(organizationId, true);
    return users.filter((user) => user.deletedAt);
  }
}
```

### Command and Query Bus

```typescript
// lib/cqrs/command-bus.ts
export class CommandBus {
  private handlers = new Map<string, ICommandHandler<any, any>>();

  register<TCommand, TResult>(
    commandType: string,
    handler: ICommandHandler<TCommand, TResult>,
  ): void {
    this.handlers.set(commandType, handler);
  }

  async execute<TCommand, TResult>(commandType: string, command: TCommand): Promise<TResult> {
    const handler = this.handlers.get(commandType);
    if (!handler) {
      throw new Error(`No handler registered for command: ${commandType}`);
    }

    try {
      return await handler.handle(command);
    } catch (error) {
      // Log error, emit events, etc.
      console.error(`Command execution failed: ${commandType}`, error);
      throw error;
    }
  }
}

// lib/cqrs/query-bus.ts
export class QueryBus {
  private handlers = new Map<string, IQueryHandler<any, any>>();

  register<TQuery, TResult>(queryType: string, handler: IQueryHandler<TQuery, TResult>): void {
    this.handlers.set(queryType, handler);
  }

  async execute<TQuery, TResult>(queryType: string, query: TQuery): Promise<TResult> {
    const handler = this.handlers.get(queryType);
    if (!handler) {
      throw new Error(`No handler registered for query: ${queryType}`);
    }

    return await handler.handle(query);
  }
}
```

### CQRS Service Integration

```typescript
// lib/services/user-cqrs.service.ts
export class UserCQRSService {
  private commandBus: CommandBus;
  private queryBus: QueryBus;

  constructor(
    userRepository: UserRepository,
    eventBus: EventBus,
    notificationService: NotificationService,
  ) {
    this.commandBus = new CommandBus();
    this.queryBus = new QueryBus();

    // Register command handlers
    this.commandBus.register(
      'InviteUser',
      new InviteUserCommand(userRepository, eventBus, notificationService),
    );
    this.commandBus.register('AssignRole', new AssignRoleCommand(userRepository, eventBus));
    this.commandBus.register('SoftDeleteUser', new SoftDeleteUserCommand(userRepository, eventBus));

    // Register query handlers
    this.queryBus.register('GetUsers', new GetUsersQuery(userRepository));
    this.queryBus.register('GetUserStats', new GetUserStatsQuery(userRepository));
    this.queryBus.register('GetDeletedUsers', new GetDeletedUsersQuery(userRepository));
  }

  // Command methods
  async inviteUser(request: InviteUserRequest): Promise<void> {
    return this.commandBus.execute('InviteUser', request);
  }

  async assignRole(request: AssignRoleRequest): Promise<User> {
    return this.commandBus.execute('AssignRole', request);
  }

  async softDeleteUser(request: SoftDeleteUserRequest): Promise<void> {
    return this.commandBus.execute('SoftDeleteUser', request);
  }

  // Query methods
  async getUsers(request: GetUsersRequest): Promise<User[]> {
    return this.queryBus.execute('GetUsers', request);
  }

  async getUserStats(request: GetUserStatsRequest): Promise<UserStats> {
    return this.queryBus.execute('GetUserStats', request);
  }

  async getDeletedUsers(request: GetDeletedUsersRequest): Promise<User[]> {
    return this.queryBus.execute('GetDeletedUsers', request);
  }
}
```

### React Hook Integration

```typescript
// hooks/use-user-commands.ts
export function useUserCommands() {
  const queryClient = useQueryClient();
  const userCQRSService = useUserCQRSService(); // DI container

  const inviteUser = useMutation({
    mutationFn: (request: InviteUserRequest) => userCQRSService.inviteUser(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User invitation sent successfully!');
    },
    onError: (error) => {
      toast.error(`Failed to invite user: ${error.message}`);
    },
  });

  const assignRole = useMutation({
    mutationFn: (request: AssignRoleRequest) => userCQRSService.assignRole(request),
    onSuccess: (user) => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success(`Role updated to ${user.role}`);
    },
    onError: (error) => {
      toast.error(`Failed to assign role: ${error.message}`);
    },
  });

  const softDeleteUser = useMutation({
    mutationFn: (request: SoftDeleteUserRequest) => userCQRSService.softDeleteUser(request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User moved to trash');
    },
    onError: (error) => {
      toast.error(`Failed to delete user: ${error.message}`);
    },
  });

  return {
    inviteUser,
    assignRole,
    softDeleteUser,
  };
}

// hooks/use-user-queries.ts
export function useUserQueries() {
  const userCQRSService = useUserCQRSService();

  const useUsers = (request: GetUsersRequest) =>
    useQuery({
      queryKey: ['users', request],
      queryFn: () => userCQRSService.getUsers(request),
      staleTime: 5 * 60 * 1000, // 5 minutes
    });

  const useUserStats = (request: GetUserStatsRequest) =>
    useQuery({
      queryKey: ['users', 'stats', request],
      queryFn: () => userCQRSService.getUserStats(request),
      staleTime: 2 * 60 * 1000, // 2 minutes
    });

  const useDeletedUsers = (request: GetDeletedUsersRequest) =>
    useQuery({
      queryKey: ['users', 'deleted', request],
      queryFn: () => userCQRSService.getDeletedUsers(request),
      staleTime: 10 * 60 * 1000, // 10 minutes
    });

  return {
    useUsers,
    useUserStats,
    useDeletedUsers,
  };
}
```

### Component Usage

```typescript
// components/user-management/user-management-page.tsx
export function UserManagementPage() {
  const { inviteUser, assignRole, softDeleteUser } = useUserCommands();
  const { useUsers, useUserStats } = useUserQueries();

  const organizationId = useCurrentOrganization().id;

  const users = useUsers({ organizationId });
  const stats = useUserStats({ organizationId });

  const handleInviteUser = async (data: InviteFormData) => {
    await inviteUser.mutateAsync({
      email: data.email,
      firstName: data.firstName,
      lastName: data.lastName,
      organizationId,
    });
  };

  const handleAssignRole = async (userId: string, role: UserRole) => {
    await assignRole.mutateAsync({ userId, role });
  };

  const handleDeleteUser = async (userId: string) => {
    const currentUserId = useCurrentUser().id;
    await softDeleteUser.mutateAsync({
      userId,
      deletedBy: currentUserId
    });
  };

  return (
    <div>
      <UserStats stats={stats.data} />
      <UserList
        users={users.data || []}
        onAssignRole={handleAssignRole}
        onDeleteUser={handleDeleteUser}
      />
      <InviteUserDialog onInvite={handleInviteUser} />
    </div>
  );
}
```

## Benefits

### 1. **Clear Separation of Concerns**

- Commands handle state changes
- Queries handle data retrieval
- No mixed responsibilities

### 2. **Better Error Handling**

- Command-specific error handling
- Query-specific caching strategies
- Centralized error logging

### 3. **Improved Testability**

- Easy to mock commands and queries
- Isolated business logic
- Clear test boundaries

### 4. **Enhanced Maintainability**

- Single responsibility principle
- Easy to add new operations
- Clear code organization

## Best Practices

### 1. **React Query Cache Invalidation with Parameterized Queries**

When using React Query with queries that have parameters in their query keys, invalidation must account for partial key matching:

```typescript
// ❌ Bad: Won't invalidate queries with params
const useContactsQuery = (params?: GetContactsParams) => {
  return useQuery({
    queryKey: ['contacts', params], // Key: ['contacts', {clientId: 'abc', page: 1}]
    queryFn: () => contactRepository.getContacts(params),
  });
};

const useUpdateContactMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input) => contactRepository.updateContact(input),
    onSuccess: () => {
      // This invalidates ONLY ['contacts'], not ['contacts', {...params}]
      queryClient.invalidateQueries({ queryKey: ['contacts'] });
    },
  });
};

// ✅ Good: Invalidates all queries starting with 'contacts'
const useUpdateContactMutation = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input) => contactRepository.updateContact(input),
    onSuccess: async () => {
      // exact: false matches all queries that START with this key
      // refetchType: 'active' immediately refetches mounted queries
      await queryClient.invalidateQueries({
        queryKey: ['contacts'],
        exact: false, // Match ['contacts'], ['contacts', {...}], etc.
        refetchType: 'active', // Refetch before component unmounts
      });
    },
  });
};
```

**Why this matters:**

- Query keys with params: `['contacts', {clientId: 'abc', page: 1}]`
- Without `exact: false`, only the exact key `['contacts']` is invalidated
- Active queries with params won't be refetched, causing stale UI
- `refetchType: 'active'` ensures immediate refetch before dialogs/modals close

**Common Pattern for All Mutations:**

```typescript
// Create, Update, Delete mutations should all use this pattern
onSuccess: async () => {
  await queryClient.invalidateQueries({
    queryKey: [QUERY_KEY_BASE],
    exact: false, // Match all variations
    refetchType: 'active', // Immediate refetch
  });
};
```

### 2. **Command Naming**

```typescript
// ✅ Good: Imperative, action-oriented
class InviteUserCommand {}
class AssignRoleCommand {}
class DeleteUserCommand {}

// ❌ Bad: Noun-based or unclear
class UserInvitation {}
class RoleAssignment {}
class UserDeletion {}
```

### 3. **Query Naming**

```typescript
// ✅ Good: Question-oriented
class GetUsersQuery {}
class GetUserStatsQuery {}
class FindUserByEmailQuery {}

// ❌ Bad: Action-oriented
class FetchUsersQuery {}
class LoadUserStatsQuery {}
class RetrieveUserQuery {}
```

### 4. **Error Handling**

```typescript
// ✅ Good: Specific error types
class BusinessError extends Error {}
class ValidationError extends Error {}
class NotFoundError extends Error {}

// ❌ Bad: Generic errors
throw new Error('Something went wrong');
```

## Testing

```typescript
// __tests__/commands/invite-user.command.test.ts
describe('InviteUserCommand', () => {
  let command: InviteUserCommand;
  let mockRepository: jest.Mocked<UserRepository>;
  let mockEventBus: jest.Mocked<EventBus>;

  beforeEach(() => {
    mockRepository = createMockUserRepository();
    mockEventBus = createMockEventBus();
    command = new InviteUserCommand(mockRepository, mockEventBus);
  });

  it('should invite user successfully', async () => {
    const request = {
      email: 'test@example.com',
      firstName: 'Test',
      lastName: 'User',
      organizationId: 'org-1',
    };

    mockRepository.findByEmail.mockResolvedValue(null);
    mockRepository.inviteUser.mockResolvedValue();

    await command.execute(request);

    expect(mockRepository.inviteUser).toHaveBeenCalledWith(request);
    expect(mockEventBus.emit).toHaveBeenCalledWith('user.invited', {
      email: request.email,
      organizationId: request.organizationId,
      invitedAt: expect.any(Date),
    });
  });

  it('should throw error if user already exists', async () => {
    const request = { email: 'existing@example.com' };
    const existingUser = { id: '1', email: 'existing@example.com' };

    mockRepository.findByEmail.mockResolvedValue(existingUser);

    await expect(command.execute(request)).rejects.toThrow('User already exists');
  });
});
```

This pattern provides clear separation between read and write operations, making
the codebase more maintainable and testable.
