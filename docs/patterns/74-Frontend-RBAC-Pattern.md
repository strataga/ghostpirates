# Frontend Role-Based Access Control (RBAC) Pattern

**Category**: Security
**Complexity**: Medium
**Status**: ✅ Implemented
**Sprint**: 6-7

## Intent

Control UI element visibility and feature access based on user roles in the frontend application, ensuring users only see and can interact with features they're authorized to use.

## Problem

Without frontend RBAC:
- Navigation shows all menu items to all users
- Action buttons (Add, Edit, Delete, Export) visible to all roles
- Users can attempt unauthorized actions (blocked by backend, but poor UX)
- No clear visual indication of role-based permissions
- Inconsistent access control across components

## Solution

Implement a comprehensive frontend RBAC system with:
1. **Reusable RBAC hook** (`useRBAC`) for role checking
2. **Declarative component** (`<RequireRole>`) for conditional rendering
3. **Navigation filtering** based on user roles
4. **Component-level access control** for buttons, forms, and features

## Structure

```
apps/web/
├── hooks/
│   └── use-rbac.ts                    # RBAC hook with role utilities
├── components/
│   └── rbac/
│       └── require-role.tsx           # Conditional rendering component
├── app/
│   └── dashboard/
│       ├── layout.tsx                 # Navigation with RBAC filtering
│       ├── page.tsx                   # Dashboard with role-based features
│       └── wells/
│           └── page.tsx               # Wells page with role-based actions
```

## Implementation

### 1. RBAC Hook (`use-rbac.ts`)

```typescript
'use client';

import { useAuth } from './use-auth';

export type UserRole = 'ADMIN' | 'MANAGER' | 'OPERATOR';

export function useRBAC() {
  const { user } = useAuth();

  /**
   * Check if user has any of the specified roles
   */
  const hasAnyRole = (roles: UserRole[]): boolean => {
    if (!user?.role) return false;
    return roles.includes(user.role as UserRole);
  };

  /**
   * Check if user has specific role
   */
  const hasRole = (role: UserRole): boolean => {
    return user?.role === role;
  };

  /**
   * Check if user is admin
   */
  const isAdmin = (): boolean => {
    return user?.role === 'ADMIN';
  };

  /**
   * Check if user is manager or admin
   */
  const isManagerOrAdmin = (): boolean => {
    return user?.role === 'ADMIN' || user?.role === 'MANAGER';
  };

  /**
   * Check if user is operator
   */
  const isOperator = (): boolean => {
    return user?.role === 'OPERATOR';
  };

  return {
    hasAnyRole,
    hasRole,
    isAdmin,
    isManagerOrAdmin,
    isOperator,
    role: user?.role as UserRole | undefined,
  };
}
```

### 2. RequireRole Component (`require-role.tsx`)

```typescript
'use client';

import { useRBAC, UserRole } from '@/hooks/use-rbac';

interface RequireRoleProps {
  roles: UserRole[];
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

export function RequireRole({ roles, children, fallback = null }: RequireRoleProps) {
  const { hasAnyRole } = useRBAC();

  if (!hasAnyRole(roles)) {
    return <>{fallback}</>;
  }

  return <>{children}</>;
}
```

### 3. Navigation with RBAC (dashboard/layout.tsx)

```typescript
import { useRBAC, UserRole } from '@/hooks/use-rbac';

const navigation = [
  { name: 'Dashboard', href: '/dashboard', icon: LayoutDashboard, roles: ['ADMIN', 'MANAGER', 'OPERATOR'] },
  { name: 'Wells', href: '/dashboard/wells', icon: Droplet, roles: ['ADMIN', 'MANAGER', 'OPERATOR'] },
  { name: 'Production', href: '/dashboard/production', icon: TrendingUp, roles: ['ADMIN', 'MANAGER', 'OPERATOR'] },
  { name: 'Reports', href: '/dashboard/reports', icon: FileText, roles: ['ADMIN', 'MANAGER', 'OPERATOR'] },
  { name: 'Team', href: '/dashboard/team', icon: Users, roles: ['ADMIN'] }, // Admin only
  { name: 'Settings', href: '/dashboard/settings', icon: Settings, roles: ['ADMIN', 'MANAGER', 'OPERATOR'] },
];

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const { hasAnyRole } = useRBAC();

  return (
    <nav>
      {navigation
        .filter((item) => hasAnyRole(item.roles as UserRole[]))
        .map((item) => (
          <Link key={item.name} href={item.href}>
            {item.name}
          </Link>
        ))}
    </nav>
  );
}
```

### 4. Component-Level RBAC (wells/page.tsx)

```typescript
import { RequireRole } from '@/components/rbac/require-role';

export default function WellsPage() {
  return (
    <div>
      <h1>Wells</h1>

      {/* Only ADMIN and MANAGER can add wells */}
      <RequireRole roles={['ADMIN', 'MANAGER']}>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Well
        </Button>
      </RequireRole>

      {/* Wells list visible to all authenticated users */}
      <WellsList />
    </div>
  );
}
```

### 5. Export Button RBAC (dashboard/page.tsx)

```typescript
import { RequireRole } from '@/components/rbac/require-role';

export default function DashboardPage() {
  return (
    <div>
      <RequireRole roles={['ADMIN', 'MANAGER']}>
        <ExportButton
          getExportData={() => formatDashboardMetricsForCSV(metricsData)}
          filename="wellos-dashboard"
        />
      </RequireRole>
    </div>
  );
}
```

## Role Hierarchy

```
ADMIN (Full Access)
  ├── Team management (view/edit users)
  ├── System settings
  ├── Data export
  ├── Create/Edit/Delete wells
  └── All MANAGER permissions

MANAGER (Management Access)
  ├── Data export
  ├── Create/Edit wells
  ├── View reports
  └── All OPERATOR permissions

OPERATOR (Read-Only Access)
  ├── View dashboard
  ├── View wells
  ├── View production data
  └── View reports
```

## Usage Examples

### Imperative (Hook)

```typescript
function MyComponent() {
  const { isAdmin, isManagerOrAdmin, hasAnyRole } = useRBAC();

  if (isAdmin()) {
    // Admin-specific logic
  }

  if (isManagerOrAdmin()) {
    // Manager or Admin logic
  }

  if (hasAnyRole(['ADMIN', 'MANAGER'])) {
    // Multiple role check
  }
}
```

### Declarative (Component)

```typescript
function MyComponent() {
  return (
    <div>
      {/* Admin only */}
      <RequireRole roles={['ADMIN']}>
        <AdminPanel />
      </RequireRole>

      {/* Manager or Admin */}
      <RequireRole roles={['ADMIN', 'MANAGER']}>
        <ManagementTools />
      </RequireRole>

      {/* All roles */}
      <RequireRole roles={['ADMIN', 'MANAGER', 'OPERATOR']}>
        <PublicContent />
      </RequireRole>

      {/* With fallback */}
      <RequireRole
        roles={['ADMIN']}
        fallback={<p>Admin access required</p>}
      >
        <AdminContent />
      </RequireRole>
    </div>
  );
}
```

## Benefits

1. **Security**: Prevents unauthorized UI access
2. **UX**: Users only see features they can use
3. **Maintainability**: Centralized role logic
4. **Reusability**: Hook and component work everywhere
5. **Type Safety**: TypeScript enforces valid roles
6. **Performance**: No unnecessary rendering

## Backend Integration

Frontend RBAC is **UI-only security** - backend MUST still enforce permissions:

```typescript
// ❌ Bad: Frontend RBAC alone
<RequireRole roles={['ADMIN']}>
  <Button onClick={() => deleteWell(id)}>Delete</Button>
</RequireRole>

// ✅ Good: Frontend RBAC + Backend Guards
// Frontend:
<RequireRole roles={['ADMIN']}>
  <Button onClick={() => deleteWell(id)}>Delete</Button>
</RequireRole>

// Backend (Rust + Axum):
async fn delete_well(
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check role authorization
    require_role(&user, &[Role::Admin])?;
    // Delete logic
    Ok(StatusCode::NO_CONTENT)
}
```

## Testing Strategy

### Manual Testing Checklist

1. **Navigation Test**:
   - Login as ADMIN → See all menu items
   - Login as MANAGER → See all except "Team"
   - Login as OPERATOR → See all except "Team"

2. **Component Test**:
   - ADMIN → See all buttons (Add Well, Export, etc.)
   - MANAGER → See management buttons (Add Well, Export)
   - OPERATOR → See no action buttons (read-only)

3. **Backend Verification**:
   - OPERATOR tries to access `/api/users` → 403 Forbidden
   - OPERATOR tries to POST `/api/wells` → 403 Forbidden

### Automated Testing (Future)

```typescript
describe('RequireRole', () => {
  it('shows content for authorized roles', () => {
    render(
      <AuthProvider user={{ role: 'ADMIN' }}>
        <RequireRole roles={['ADMIN']}>
          <div>Admin Content</div>
        </RequireRole>
      </AuthProvider>
    );
    expect(screen.getByText('Admin Content')).toBeInTheDocument();
  });

  it('hides content for unauthorized roles', () => {
    render(
      <AuthProvider user={{ role: 'OPERATOR' }}>
        <RequireRole roles={['ADMIN']}>
          <div>Admin Content</div>
        </RequireRole>
      </AuthProvider>
    );
    expect(screen.queryByText('Admin Content')).not.toBeInTheDocument();
  });
});
```

## Related Patterns

- **[Backend RBAC Pattern](./XX-Backend-RBAC-Pattern.md)**: Server-side role enforcement
- **[JWT Authentication Pattern](./XX-JWT-Authentication-Pattern.md)**: User role in JWT tokens
- **[Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)**: Tenant-scoped permissions

## References

- Axum Middleware: https://docs.rs/axum/latest/axum/middleware/
- React Context API: https://react.dev/reference/react/useContext
- Zustand: https://zustand-demo.pmnd.rs/

---

**Pattern Status**: ✅ Production-ready
**Last Updated**: October 31, 2025
**Sprint**: 6-7 (Analytics Dashboard)
