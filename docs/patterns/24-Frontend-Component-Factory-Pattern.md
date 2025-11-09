# Frontend Component Factory Pattern

## Overview

The Component Factory Pattern in React applications provides a systematic way to
create components dynamically based on configuration, type, or runtime
conditions. This pattern promotes code reuse, consistency, and maintainability
by centralizing component creation logic.

## Problem Statement

Frontend applications often face challenges with:

- **Repetitive component code** for similar UI patterns
- **Inconsistent implementations** of common components
- **Difficult maintenance** when updating shared patterns
- **Complex conditional rendering** logic scattered throughout components
- **Poor scalability** when adding new component variants

## Solution

Implement a Component Factory Pattern that creates components based on
configuration objects, types, or runtime conditions, ensuring consistency and
reducing code duplication.

## Implementation

### Base Factory Interface

```typescript
// lib/factories/interfaces.ts
export interface ComponentFactory<TConfig, TComponent> {
  create(config: TConfig): TComponent;
  canCreate(config: TConfig): boolean;
}

export interface ComponentConfig {
  type: string;
  props?: Record<string, any>;
  children?: React.ReactNode;
}
```

### Form Factory Implementation

```typescript
// lib/factories/form.factory.ts
export interface FormFieldConfig {
  name: string;
  type: 'text' | 'email' | 'password' | 'select' | 'checkbox' | 'textarea' | 'date';
  label: string;
  placeholder?: string;
  required?: boolean;
  validation?: any; // Zod schema or validation rules
  options?: { value: string; label: string }[]; // For select fields
  disabled?: boolean;
  description?: string;
}

export interface FormConfig {
  fields: FormFieldConfig[];
  submitText: string;
  cancelText?: string;
  validationSchema: any; // Zod schema
  onSubmit: (data: any) => void | Promise<void>;
  onCancel?: () => void;
  loading?: boolean;
}

export class FormFactory {
  static createField(config: FormFieldConfig): React.ReactElement {
    const { name, type, label, placeholder, required, options, disabled, description } = config;

    const baseProps = {
      name,
      label,
      placeholder,
      required,
      disabled,
      description,
    };

    switch (type) {
      case 'text':
      case 'email':
      case 'password':
        return (
          <FormField key={name} {...baseProps}>
            <Input type={type} {...baseProps} />
          </FormField>
        );

      case 'textarea':
        return (
          <FormField key={name} {...baseProps}>
            <Textarea {...baseProps} />
          </FormField>
        );

      case 'select':
        return (
          <FormField key={name} {...baseProps}>
            <Select {...baseProps}>
              {options?.map(option => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </Select>
          </FormField>
        );

      case 'checkbox':
        return (
          <FormField key={name} {...baseProps}>
            <Checkbox {...baseProps} />
          </FormField>
        );

      case 'date':
        return (
          <FormField key={name} {...baseProps}>
            <DatePicker {...baseProps} />
          </FormField>
        );

      default:
        throw new Error(`Unsupported field type: ${type}`);
    }
  }

  static createForm(config: FormConfig): React.ReactElement {
    const { fields, submitText, cancelText, validationSchema, onSubmit, onCancel, loading } = config;

    return (
      <Form
        schema={validationSchema}
        onSubmit={onSubmit}
        className="space-y-4"
      >
        {fields.map(fieldConfig => this.createField(fieldConfig))}

        <div className="flex gap-2 pt-4">
          <Button type="submit" loading={loading}>
            {submitText}
          </Button>
          {cancelText && onCancel && (
            <Button type="button" variant="outline" onClick={onCancel}>
              {cancelText}
            </Button>
          )}
        </div>
      </Form>
    );
  }

  // Predefined form configurations
  static createUserInviteForm(onSubmit: (data: any) => void): React.ReactElement {
    const config: FormConfig = {
      fields: [
        {
          name: 'email',
          type: 'email',
          label: 'Email Address',
          placeholder: 'user@example.com',
          required: true,
        },
        {
          name: 'firstName',
          type: 'text',
          label: 'First Name',
          placeholder: 'John',
          required: true,
        },
        {
          name: 'lastName',
          type: 'text',
          label: 'Last Name',
          placeholder: 'Doe',
          required: true,
        },
        {
          name: 'role',
          type: 'select',
          label: 'Role',
          required: true,
          options: [
            { value: 'pumper', label: 'Pumper' },
            { value: 'manager', label: 'Manager' },
            { value: 'owner', label: 'Owner' },
          ],
        },
      ],
      submitText: 'Send Invitation',
      cancelText: 'Cancel',
      validationSchema: inviteUserSchema,
      onSubmit,
    };

    return this.createForm(config);
  }

  static createUserEditForm(
    user: User,
    onSubmit: (data: any) => void
  ): React.ReactElement {
    const config: FormConfig = {
      fields: [
        {
          name: 'firstName',
          type: 'text',
          label: 'First Name',
          required: true,
        },
        {
          name: 'lastName',
          type: 'text',
          label: 'Last Name',
          required: true,
        },
        {
          name: 'email',
          type: 'email',
          label: 'Email Address',
          required: true,
        },
        {
          name: 'phone',
          type: 'text',
          label: 'Phone Number',
          placeholder: '+1 (555) 123-4567',
        },
        {
          name: 'isActive',
          type: 'checkbox',
          label: 'Active User',
          description: 'User can log in and access the system',
        },
      ],
      submitText: 'Update User',
      cancelText: 'Cancel',
      validationSchema: updateUserSchema,
      onSubmit,
    };

    return this.createForm(config);
  }
}
```

### Table Factory Implementation

```typescript
// lib/factories/table.factory.ts
export interface TableColumnConfig<T> {
  key: keyof T | string;
  header: string;
  sortable?: boolean;
  filterable?: boolean;
  render?: (value: any, row: T) => React.ReactNode;
  width?: string;
  align?: 'left' | 'center' | 'right';
}

export interface TableConfig<T> {
  columns: TableColumnConfig<T>[];
  data: T[];
  loading?: boolean;
  pagination?: {
    page: number;
    pageSize: number;
    total: number;
    onPageChange: (page: number) => void;
  };
  sorting?: {
    column: string;
    direction: 'asc' | 'desc';
    onSort: (column: string, direction: 'asc' | 'desc') => void;
  };
  filtering?: {
    filters: Record<string, any>;
    onFilter: (filters: Record<string, any>) => void;
  };
  actions?: {
    label: string;
    onClick: (row: T) => void;
    variant?: 'default' | 'destructive' | 'outline';
    icon?: React.ReactNode;
  }[];
}

export class TableFactory {
  static createColumn<T>(config: TableColumnConfig<T>): React.ReactElement {
    const { key, header, sortable, render, width, align } = config;

    return (
      <TableHead
        key={String(key)}
        className={`${width ? `w-${width}` : ''} text-${align || 'left'}`}
      >
        {sortable ? (
          <Button variant="ghost" className="h-auto p-0 font-semibold">
            {header}
            <ArrowUpDown className="ml-2 h-4 w-4" />
          </Button>
        ) : (
          header
        )}
      </TableHead>
    );
  }

  static createCell<T>(
    config: TableColumnConfig<T>,
    row: T
  ): React.ReactElement {
    const { key, render, align } = config;
    const value = typeof key === 'string' ? (row as any)[key] : row[key];

    return (
      <TableCell key={String(key)} className={`text-${align || 'left'}`}>
        {render ? render(value, row) : value}
      </TableCell>
    );
  }

  static createTable<T>(config: TableConfig<T>): React.ReactElement {
    const { columns, data, loading, pagination, actions } = config;

    return (
      <div className="space-y-4">
        <Table>
          <TableHeader>
            <TableRow>
              {columns.map(column => this.createColumn(column))}
              {actions && actions.length > 0 && (
                <TableHead className="text-right">Actions</TableHead>
              )}
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading ? (
              <TableRow>
                <TableCell colSpan={columns.length + (actions ? 1 : 0)}>
                  <div className="flex justify-center py-8">
                    <Spinner />
                  </div>
                </TableCell>
              </TableRow>
            ) : data.length === 0 ? (
              <TableRow>
                <TableCell colSpan={columns.length + (actions ? 1 : 0)}>
                  <div className="text-center py-8 text-muted-foreground">
                    No data available
                  </div>
                </TableCell>
              </TableRow>
            ) : (
              data.map((row, index) => (
                <TableRow key={index}>
                  {columns.map(column => this.createCell(column, row))}
                  {actions && actions.length > 0 && (
                    <TableCell className="text-right">
                      <div className="flex gap-2 justify-end">
                        {actions.map((action, actionIndex) => (
                          <Button
                            key={actionIndex}
                            variant={action.variant || 'outline'}
                            size="sm"
                            onClick={() => action.onClick(row)}
                          >
                            {action.icon}
                            {action.label}
                          </Button>
                        ))}
                      </div>
                    </TableCell>
                  )}
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>

        {pagination && (
          <div className="flex justify-between items-center">
            <div className="text-sm text-muted-foreground">
              Showing {(pagination.page - 1) * pagination.pageSize + 1} to{' '}
              {Math.min(pagination.page * pagination.pageSize, pagination.total)} of{' '}
              {pagination.total} results
            </div>
            <Pagination
              currentPage={pagination.page}
              totalPages={Math.ceil(pagination.total / pagination.pageSize)}
              onPageChange={pagination.onPageChange}
            />
          </div>
        )}
      </div>
    );
  }

  // Predefined table configurations
  static createUserTable(
    users: User[],
    onEdit: (user: User) => void,
    onDelete: (user: User) => void,
    onAssignRole: (user: User) => void
  ): React.ReactElement {
    const config: TableConfig<User> = {
      columns: [
        {
          key: 'firstName',
          header: 'Name',
          render: (_, user) => (
            <div className="flex items-center gap-3">
              <Avatar className="h-8 w-8">
                <AvatarFallback>
                  {user.firstName?.[0]}{user.lastName?.[0]}
                </AvatarFallback>
              </Avatar>
              <div>
                <div className="font-medium">
                  {user.firstName} {user.lastName}
                </div>
                <div className="text-sm text-muted-foreground">
                  {user.email}
                </div>
              </div>
            </div>
          ),
        },
        {
          key: 'role',
          header: 'Role',
          render: (role) => (
            <Badge variant={role === 'owner' ? 'default' : 'secondary'}>
              {role}
            </Badge>
          ),
        },
        {
          key: 'isActive',
          header: 'Status',
          render: (isActive) => (
            <Badge variant={isActive ? 'success' : 'destructive'}>
              {isActive ? 'Active' : 'Inactive'}
            </Badge>
          ),
        },
        {
          key: 'createdAt',
          header: 'Created',
          render: (createdAt) => formatDistanceToNow(new Date(createdAt)),
        },
      ],
      data: users,
      actions: [
        {
          label: 'Edit',
          onClick: onEdit,
          icon: <Edit className="h-4 w-4 mr-1" />,
        },
        {
          label: 'Assign Role',
          onClick: onAssignRole,
          icon: <UserCheck className="h-4 w-4 mr-1" />,
        },
        {
          label: 'Delete',
          onClick: onDelete,
          variant: 'destructive',
          icon: <Trash2 className="h-4 w-4 mr-1" />,
        },
      ],
    };

    return this.createTable(config);
  }
}
```

### Dialog Factory Implementation

```typescript
// lib/factories/dialog.factory.ts
export interface DialogConfig {
  title: string;
  description?: string;
  content: React.ReactNode;
  footer?: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export class DialogFactory {
  static createDialog(config: DialogConfig): React.ReactElement {
    const { title, description, content, footer, size = 'md', open, onOpenChange } = config;

    const sizeClasses = {
      sm: 'max-w-sm',
      md: 'max-w-md',
      lg: 'max-w-lg',
      xl: 'max-w-xl',
    };

    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className={sizeClasses[size]}>
          <DialogHeader>
            <DialogTitle>{title}</DialogTitle>
            {description && (
              <DialogDescription>{description}</DialogDescription>
            )}
          </DialogHeader>

          <div className="py-4">
            {content}
          </div>

          {footer && (
            <DialogFooter>
              {footer}
            </DialogFooter>
          )}
        </DialogContent>
      </Dialog>
    );
  }

  static createConfirmDialog(
    title: string,
    message: string,
    onConfirm: () => void,
    onCancel: () => void,
    open: boolean,
    onOpenChange: (open: boolean) => void
  ): React.ReactElement {
    const config: DialogConfig = {
      title,
      description: message,
      content: null,
      footer: (
        <div className="flex gap-2">
          <Button variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={onConfirm}>
            Confirm
          </Button>
        </div>
      ),
      size: 'sm',
      open,
      onOpenChange,
    };

    return this.createDialog(config);
  }

  static createFormDialog(
    title: string,
    form: React.ReactElement,
    open: boolean,
    onOpenChange: (open: boolean) => void
  ): React.ReactElement {
    const config: DialogConfig = {
      title,
      content: form,
      size: 'md',
      open,
      onOpenChange,
    };

    return this.createDialog(config);
  }
}
```

### Component Factory Registry

```typescript
// lib/factories/component.registry.ts
export interface ComponentRegistryConfig {
  type: string;
  factory: (props: any) => React.ReactElement;
  defaultProps?: Record<string, any>;
}

export class ComponentRegistry {
  private static factories = new Map<string, ComponentRegistryConfig>();

  static register(config: ComponentRegistryConfig): void {
    this.factories.set(config.type, config);
  }

  static create(type: string, props: any = {}): React.ReactElement {
    const config = this.factories.get(type);
    if (!config) {
      throw new Error(`No factory registered for component type: ${type}`);
    }

    const mergedProps = { ...config.defaultProps, ...props };
    return config.factory(mergedProps);
  }

  static canCreate(type: string): boolean {
    return this.factories.has(type);
  }

  static getRegisteredTypes(): string[] {
    return Array.from(this.factories.keys());
  }
}

// Register common components
ComponentRegistry.register({
  type: 'user-invite-form',
  factory: (props) => FormFactory.createUserInviteForm(props.onSubmit),
});

ComponentRegistry.register({
  type: 'user-edit-form',
  factory: (props) => FormFactory.createUserEditForm(props.user, props.onSubmit),
});

ComponentRegistry.register({
  type: 'user-table',
  factory: (props) =>
    TableFactory.createUserTable(props.users, props.onEdit, props.onDelete, props.onAssignRole),
});
```

### Usage in Components

```typescript
// components/user-management/user-management-page.tsx
export function UserManagementPage() {
  const [isInviteDialogOpen, setIsInviteDialogOpen] = useState(false);
  const [editingUser, setEditingUser] = useState<User | null>(null);
  const { users, loading } = useUsers();

  const handleInviteUser = async (data: any) => {
    await inviteUser(data);
    setIsInviteDialogOpen(false);
  };

  const handleEditUser = (user: User) => {
    setEditingUser(user);
  };

  const handleUpdateUser = async (data: any) => {
    if (editingUser) {
      await updateUser(editingUser.id, data);
      setEditingUser(null);
    }
  };

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold">User Management</h1>
        <Button onClick={() => setIsInviteDialogOpen(true)}>
          Invite User
        </Button>
      </div>

      {/* User table created by factory */}
      {TableFactory.createUserTable(
        users,
        handleEditUser,
        handleDeleteUser,
        handleAssignRole
      )}

      {/* Invite dialog created by factory */}
      {DialogFactory.createFormDialog(
        'Invite New User',
        FormFactory.createUserInviteForm(handleInviteUser),
        isInviteDialogOpen,
        setIsInviteDialogOpen
      )}

      {/* Edit dialog created by factory */}
      {editingUser && DialogFactory.createFormDialog(
        'Edit User',
        FormFactory.createUserEditForm(editingUser, handleUpdateUser),
        !!editingUser,
        () => setEditingUser(null)
      )}
    </div>
  );
}
```

## Benefits

### 1. **Consistency**

- Standardized component creation
- Uniform styling and behavior
- Reduced implementation variations

### 2. **Reusability**

- Components created from configuration
- Easy to create similar components
- Shared patterns across the application

### 3. **Maintainability**

- Centralized component logic
- Easy to update patterns globally
- Clear separation of concerns

### 4. **Flexibility**

- Runtime component creation
- Configuration-driven UI
- Easy customization

## Best Practices

### 1. **Factory Scope**

```typescript
// ✅ Good: Focused factory
class FormFactory {
  static createUserForm() {}
  static createWellForm() {}
}

// ❌ Bad: Generic factory
class ComponentFactory {
  static createForm() {}
  static createTable() {}
  static createDialog() {}
}
```

### 2. **Configuration Validation**

```typescript
// ✅ Good: Validate configuration
static createForm(config: FormConfig): React.ReactElement {
  if (!config.fields || config.fields.length === 0) {
    throw new Error('Form must have at least one field');
  }
  // Create form...
}
```

### 3. **Type Safety**

```typescript
// ✅ Good: Strongly typed
interface TableConfig<T> {
  columns: TableColumnConfig<T>[];
  data: T[];
}

// ❌ Bad: Loosely typed
interface TableConfig {
  columns: any[];
  data: any[];
}
```

This Component Factory Pattern provides a scalable way to create consistent,
reusable UI components throughout your application.
