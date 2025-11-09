# Soft Delete Implementation Guide

## Executive Summary

This guide provides a comprehensive implementation of soft delete functionality
across all models in both the API and frontend applications, ensuring data
integrity while maintaining audit trails.

## 🎯 **Soft Delete Strategy**

### Core Principles

1. **Never physically delete data** - Always use soft delete
2. **Maintain audit trails** - Track who deleted what and when
3. **Cascade soft deletes** - Handle related entities properly
4. **Restore capability** - Allow undeleting when appropriate
5. **Query filtering** - Default to excluding soft-deleted records

## 🔧 **Backend Implementation (API)**

### 1. Database Schema Updates

```sql
-- Add soft delete columns to all tables
-- Users table (already has some fields)
ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP;
ALTER TABLE users ADD COLUMN IF NOT EXISTS deleted_by UUID REFERENCES users(id);

-- Organizations table
ALTER TABLE organizations ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE organizations ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Wells table
ALTER TABLE wells ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE wells ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Leases table
ALTER TABLE leases ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE leases ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Production records table
ALTER TABLE production_records ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE production_records ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Partners table
ALTER TABLE partners ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE partners ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Equipment table
ALTER TABLE equipment ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE equipment ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Documents table
ALTER TABLE documents ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE documents ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Compliance reports table
ALTER TABLE compliance_reports ADD COLUMN deleted_at TIMESTAMP;
ALTER TABLE compliance_reports ADD COLUMN deleted_by UUID REFERENCES users(id);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_deleted_at ON users(deleted_at);
CREATE INDEX IF NOT EXISTS idx_organizations_deleted_at ON organizations(deleted_at);
CREATE INDEX IF NOT EXISTS idx_wells_deleted_at ON wells(deleted_at);
CREATE INDEX IF NOT EXISTS idx_leases_deleted_at ON leases(deleted_at);
CREATE INDEX IF NOT EXISTS idx_production_records_deleted_at ON production_records(deleted_at);
CREATE INDEX IF NOT EXISTS idx_partners_deleted_at ON partners(deleted_at);
CREATE INDEX IF NOT EXISTS idx_equipment_deleted_at ON equipment(deleted_at);
CREATE INDEX IF NOT EXISTS idx_documents_deleted_at ON documents(deleted_at);
CREATE INDEX IF NOT EXISTS idx_compliance_reports_deleted_at ON compliance_reports(deleted_at);
```

### 2. Database Schema Updates (SQL)

```sql
-- Update: apps/api/src/infrastructure/database/migrations/users.sql
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  organization_id UUID REFERENCES organizations(id),
  email VARCHAR(255) NOT NULL UNIQUE,
  first_name VARCHAR(100),
  last_name VARCHAR(100),
  role VARCHAR(50) NOT NULL DEFAULT 'pumper',
  phone VARCHAR(20),
  password_hash VARCHAR(255),
  email_verified BOOLEAN DEFAULT false,
  email_verification_token VARCHAR(255),
  email_verification_expires_at TIMESTAMP,
  failed_login_attempts INTEGER DEFAULT 0,
  locked_until TIMESTAMP,
  lockout_count INTEGER DEFAULT 0,
  password_reset_token VARCHAR(255),
  password_reset_expires_at TIMESTAMP,
  is_active BOOLEAN DEFAULT true,
  last_login_at TIMESTAMP,
  created_at TIMESTAMP DEFAULT NOW(),
  updated_at TIMESTAMP DEFAULT NOW(),
  -- Soft delete fields
  deleted_at TIMESTAMP,
  deleted_by UUID REFERENCES users(id)
);

-- Create base soft delete trait in Rust
trait SoftDeletable {
    fn deleted_at(&self) -> Option<DateTime<Utc>>;
    fn deleted_by(&self) -> Option<String>;
}
```

### 3. Base Repository with Soft Delete

```typescript
// Update: apps/api/src/infrastructure/repositories/base.repository.ts
export abstract class BaseRepository<T extends PgTable<TableConfig>> {
  constructor(
    protected readonly db: DatabaseService,
    protected readonly table: T,
  ) {}

  // Override findAll to exclude soft deleted by default
  async findAll(includeDeleted = false): Promise<T['$inferSelect'][]> {
    let query = this.db.select().from(this.table);

    if (!includeDeleted) {
      query = query.where(isNull(this.table.deletedAt));
    }

    return query;
  }

  // Override findById to exclude soft deleted by default
  async findById(id: string, includeDeleted = false): Promise<T['$inferSelect'] | null> {
    let query = this.db.select().from(this.table).where(eq(this.table.id, id));

    if (!includeDeleted) {
      query = query.where(and(eq(this.table.id, id), isNull(this.table.deletedAt)));
    }

    const result = await query;
    return result[0] || null;
  }

  // Soft delete method
  async softDelete(id: string, deletedBy: string): Promise<void> {
    await this.db
      .update(this.table)
      .set({
        deletedAt: new Date(),
        deletedBy,
        updatedAt: new Date(),
      } as any)
      .where(eq(this.table.id, id));
  }

  // Restore method
  async restore(id: string): Promise<void> {
    await this.db
      .update(this.table)
      .set({
        deletedAt: null,
        deletedBy: null,
        updatedAt: new Date(),
      } as any)
      .where(eq(this.table.id, id));
  }

  // Get only soft deleted records
  async findDeleted(): Promise<T['$inferSelect'][]> {
    return this.db.select().from(this.table).where(isNotNull(this.table.deletedAt));
  }

  // Check if record is soft deleted
  async isDeleted(id: string): Promise<boolean> {
    const result = await this.db
      .select({ deletedAt: this.table.deletedAt })
      .from(this.table)
      .where(eq(this.table.id, id));

    return result[0]?.deletedAt !== null;
  }

  // Hard delete (use with extreme caution)
  async hardDelete(id: string): Promise<void> {
    await this.db.delete(this.table).where(eq(this.table.id, id));
  }
}
```

### 4. Domain Entity Updates

```typescript
// Update: apps/api/src/domain/entities/user.entity.ts
export class User {
  constructor(
    private readonly id: string,
    private readonly organizationId: string,
    private email: string,
    private firstName: string,
    private lastName: string,
    private role: UserRole,
    private phone?: string,
    private passwordHash?: string,
    private emailVerified: boolean = false,
    private isActive: boolean = true,
    private readonly createdAt: Date = new Date(),
    private updatedAt: Date = new Date(),
    // Soft delete properties
    private deletedAt?: Date,
    private deletedBy?: string,
  ) {}

  // Soft delete methods
  softDelete(deletedBy: string): void {
    if (this.isDeleted()) {
      throw new Error('User is already deleted');
    }

    this.deletedAt = new Date();
    this.deletedBy = deletedBy;
    this.updatedAt = new Date();

    // Emit domain event
    this.addDomainEvent(new UserSoftDeletedEvent(this.id, deletedBy));
  }

  restore(): void {
    if (!this.isDeleted()) {
      throw new Error('User is not deleted');
    }

    this.deletedAt = undefined;
    this.deletedBy = undefined;
    this.updatedAt = new Date();

    // Emit domain event
    this.addDomainEvent(new UserRestoredEvent(this.id));
  }

  isDeleted(): boolean {
    return this.deletedAt !== undefined;
  }

  getDeletedAt(): Date | undefined {
    return this.deletedAt;
  }

  getDeletedBy(): string | undefined {
    return this.deletedBy;
  }

  // Prevent operations on deleted entities
  updateEmail(email: string): void {
    if (this.isDeleted()) {
      throw new Error('Cannot update deleted user');
    }
    this.email = email;
    this.updatedAt = new Date();
  }

  assignRole(role: UserRole): void {
    if (this.isDeleted()) {
      throw new Error('Cannot assign role to deleted user');
    }
    this.role = role;
    this.updatedAt = new Date();
  }
}
```

### 5. Service Layer Updates

```typescript
// Update: apps/api/src/application/services/users.service.ts
@Injectable()
export class UsersService {
  constructor(
    @Inject('UsersRepository')
    private readonly usersRepository: UsersRepository,
    private readonly eventBus: EventBus,
  ) {}

  async softDeleteUser(id: string, deletedBy: string): Promise<void> {
    const user = await this.usersRepository.findById(id);
    if (!user) {
      throw new NotFoundException('User not found');
    }

    // Check if user can be deleted (business rules)
    if (user.role === 'owner') {
      const activeOwners = await this.usersRepository.countByRole('owner');
      if (activeOwners <= 1) {
        throw new BadRequestException('Cannot delete the last owner');
      }
    }

    // Perform soft delete
    await this.usersRepository.softDelete(id, deletedBy);

    // Emit event for side effects (notifications, audit logs, etc.)
    this.eventBus.emit('user.soft-deleted', { userId: id, deletedBy });
  }

  async restoreUser(id: string): Promise<void> {
    const user = await this.usersRepository.findById(id, true); // include deleted
    if (!user) {
      throw new NotFoundException('User not found');
    }

    if (!user.deletedAt) {
      throw new BadRequestException('User is not deleted');
    }

    await this.usersRepository.restore(id);
    this.eventBus.emit('user.restored', { userId: id });
  }

  async getDeletedUsers(organizationId: string): Promise<User[]> {
    return this.usersRepository.findDeleted(organizationId);
  }

  // Cascade soft delete for organization
  async softDeleteUsersByOrganization(organizationId: string, deletedBy: string): Promise<void> {
    const users = await this.usersRepository.findByOrganization(organizationId);

    for (const user of users) {
      await this.softDeleteUser(user.id, deletedBy);
    }
  }
}
```

### 6. Controller Updates

```typescript
// Update: apps/api/src/presentation/controllers/users.controller.ts
@Controller('users')
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @Delete(':id/soft')
  @CheckAbilities({ action: 'delete', subject: 'User' })
  async softDeleteUser(
    @Param('id') id: string,
    @CurrentUser() currentUser: any,
  ): Promise<{ message: string }> {
    await this.usersService.softDeleteUser(id, currentUser.id);
    return { message: 'User soft deleted successfully' };
  }

  @Post(':id/restore')
  @CheckAbilities({ action: 'restore', subject: 'User' })
  async restoreUser(@Param('id') id: string): Promise<{ message: string }> {
    await this.usersService.restoreUser(id);
    return { message: 'User restored successfully' };
  }

  @Get('deleted')
  @CheckAbilities({ action: 'read', subject: 'User' })
  async getDeletedUsers(@CurrentUser() currentUser: any): Promise<User[]> {
    return this.usersService.getDeletedUsers(currentUser.organizationId);
  }

  @Delete(':id/hard')
  @CheckAbilities({ action: 'hardDelete', subject: 'User' })
  async hardDeleteUser(@Param('id') id: string): Promise<{ message: string }> {
    // This should be extremely restricted and logged
    await this.usersService.hardDeleteUser(id);
    return { message: 'User permanently deleted' };
  }
}
```

## 🎨 **Frontend Implementation**

### 1. Type Definitions

```typescript
// Update: apps/web/types/user.ts
export interface SoftDeletable {
  deletedAt?: string; // ISO string from API
  deletedBy?: string;
}

export interface User extends SoftDeletable {
  id: string;
  organizationId: string;
  email: string;
  firstName: string;
  lastName: string;
  role: UserRole;
  phone?: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

// Add utility type for filtering
export type ActiveUser = Omit<User, 'deletedAt' | 'deletedBy'>;
export type DeletedUser = User & Required<Pick<User, 'deletedAt' | 'deletedBy'>>;
```

### 2. API Service Updates

```typescript
// Update: apps/web/lib/api/users.ts
export class UserApi {
  // Soft delete user
  async softDeleteUser(id: string): Promise<void> {
    await apiRequest(`/users/${id}/soft`, {
      method: 'DELETE',
    });
  }

  // Restore user
  async restoreUser(id: string): Promise<User> {
    return apiRequest<User>(`/users/${id}/restore`, {
      method: 'POST',
    });
  }

  // Get deleted users
  async getDeletedUsers(): Promise<User[]> {
    return apiRequest<User[]>('/users/deleted');
  }

  // Get all users (active by default)
  async getUsers(includeDeleted = false): Promise<User[]> {
    const params = includeDeleted ? '?includeDeleted=true' : '';
    return apiRequest<User[]>(`/users${params}`);
  }

  // Hard delete (admin only)
  async hardDeleteUser(id: string): Promise<void> {
    await apiRequest(`/users/${id}/hard`, {
      method: 'DELETE',
    });
  }
}
```

### 3. Enhanced Hook with Soft Delete

```typescript
// Update: apps/web/hooks/use-users.ts
export function useUsers() {
  const [showDeleted, setShowDeleted] = useState(false);

  // Query for active users
  const activeUsersQuery = useQuery({
    queryKey: ['users', 'active'],
    queryFn: () => userApi.getUsers(false),
  });

  // Query for deleted users
  const deletedUsersQuery = useQuery({
    queryKey: ['users', 'deleted'],
    queryFn: () => userApi.getDeletedUsers(),
    enabled: showDeleted,
  });

  // Soft delete mutation
  const softDeleteMutation = useMutation({
    mutationFn: (id: string) => userApi.softDeleteUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User moved to trash');
    },
    onError: (error) => {
      toast.error('Failed to delete user');
    },
  });

  // Restore mutation
  const restoreMutation = useMutation({
    mutationFn: (id: string) => userApi.restoreUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User restored successfully');
    },
    onError: (error) => {
      toast.error('Failed to restore user');
    },
  });

  // Hard delete mutation (admin only)
  const hardDeleteMutation = useMutation({
    mutationFn: (id: string) => userApi.hardDeleteUser(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['users'] });
      toast.success('User permanently deleted');
    },
    onError: (error) => {
      toast.error('Failed to permanently delete user');
    },
  });

  return {
    // Data
    users: showDeleted ? deletedUsersQuery.data || [] : activeUsersQuery.data || [],
    loading: activeUsersQuery.isLoading || deletedUsersQuery.isLoading,
    error: activeUsersQuery.error || deletedUsersQuery.error,

    // State
    showDeleted,
    setShowDeleted,

    // Actions
    softDeleteUser: softDeleteMutation.mutate,
    restoreUser: restoreMutation.mutate,
    hardDeleteUser: hardDeleteMutation.mutate,

    // Status
    isSoftDeleting: softDeleteMutation.isPending,
    isRestoring: restoreMutation.isPending,
    isHardDeleting: hardDeleteMutation.isPending,
  };
}
```

### 4. UI Components for Soft Delete

```typescript
// Create: apps/web/components/user-management/deleted-users-view.tsx
export function DeletedUsersView() {
  const { users, restoreUser, hardDeleteUser, isRestoring, isHardDeleting } = useUsers();

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">Deleted Users</h2>
        <Badge variant="secondary">{users.length} deleted users</Badge>
      </div>

      {users.map((user) => (
        <Card key={user.id} className="p-4 opacity-75">
          <div className="flex items-center justify-between">
            <div>
              <h3 className="font-semibold text-red-600">
                {user.firstName} {user.lastName}
              </h3>
              <p className="text-sm text-muted-foreground">{user.email}</p>
              <p className="text-xs text-muted-foreground">
                Deleted {formatDistanceToNow(new Date(user.deletedAt!))} ago
              </p>
            </div>

            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => restoreUser(user.id)}
                disabled={isRestoring}
              >
                <RotateCcw className="h-4 w-4 mr-2" />
                Restore
              </Button>

              <Button
                variant="destructive"
                size="sm"
                onClick={() => {
                  if (confirm('Permanently delete this user? This cannot be undone.')) {
                    hardDeleteUser(user.id);
                  }
                }}
                disabled={isHardDeleting}
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete Forever
              </Button>
            </div>
          </div>
        </Card>
      ))}
    </div>
  );
}
```

## 🔄 **Migration Strategy**

### Phase 1: Database Schema (Week 1)

1. Add soft delete columns to all tables
2. Create migration scripts
3. Update Drizzle schemas

### Phase 2: Backend Implementation (Week 2)

1. Update base repository with soft delete methods
2. Modify all domain entities
3. Update service layer logic
4. Add new controller endpoints

### Phase 3: Frontend Implementation (Week 3)

1. Update API services
2. Enhance hooks and state management
3. Create soft delete UI components
4. Add restore functionality

### Phase 4: Testing & Rollout (Week 4)

1. Comprehensive testing
2. Data migration for existing records
3. User training and documentation
4. Gradual rollout with monitoring

This implementation ensures that all data is preserved while providing a clean
user experience and maintaining full audit trails.
