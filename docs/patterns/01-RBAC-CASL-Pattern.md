# RBAC/CASL Pattern - Role-Based Access Control with Attribute-Based Access Control

## Overview

RBAC (Role-Based Access Control) combined with CASL (Attribute-Based Access
Control) provides fine-grained permission management for oil & gas operations
where different users need varying levels of access to wells, production data,
and compliance reports.

## Purpose

- Control who can access what resources
- Implement field-level permissions (e.g., view production but not costs)
- Support multi-tenant isolation
- Enable dynamic permission rules based on data attributes
- Audit and compliance requirements

## Oil & Gas Use Cases

- Pumpers can update production but not financial data
- Partners can view only their ownership percentage data
- Regulators can view compliance reports but not edit
- Operators can manage their wells but not others'
- Accountants can view all financial data but not modify operations

## Before Implementation

```typescript
// ❌ Poor: Hard-coded role checks scattered throughout
class WellController {
  async updateProduction(wellId: string, data: ProductionData, user: User) {
    // Hard-coded role check
    if (user.role !== 'operator' && user.role !== 'admin') {
      throw new Error('Not authorized');
    }

    // No field-level permissions
    const well = await this.wellService.getWell(wellId);

    // No tenant isolation
    if (well.operatorId !== user.organizationId) {
      throw new Error('Not your well');
    }

    return await this.wellService.updateProduction(wellId, data);
  }

  async viewFinancials(wellId: string, user: User) {
    // Duplicate authorization logic
    if (user.role !== 'operator' && user.role !== 'admin' && user.role !== 'accountant') {
      throw new Error('Not authorized');
    }

    return await this.wellService.getFinancials(wellId);
  }
}

// Problems:
// - Authorization logic duplicated
// - No central permission management
// - Hard to test
// - No audit trail
// - Role changes require code changes
```

## After Implementation

```typescript
// ✅ Good: Centralized RBAC/CASL implementation

// 1. Define Abilities Factory
import { AbilityBuilder, PureAbility, createMongoAbility } from '@casl/ability';

export type Actions =
  | 'create'
  | 'read'
  | 'update'
  | 'delete'
  | 'updateProduction'
  | 'viewFinancials'
  | 'submitCompliance'
  | 'approveAFE'
  | 'exportData';

export type Subjects =
  | 'Well'
  | 'Production'
  | 'Financial'
  | 'ComplianceReport'
  | 'AFE'
  | 'Partner'
  | 'all';

export type AppAbility = PureAbility<[Actions, Subjects]>;

// 2. Define Permission Rules
@Injectable()
export class CaslAbilityFactory {
  createForUser(user: User): AppAbility {
    const { can, cannot, build } = new AbilityBuilder<AppAbility>(createMongoAbility);

    // Admin - full access
    if (user.roles.includes('ADMIN')) {
      can('manage', 'all');
      return build();
    }

    // Operator - manage own wells
    if (user.roles.includes('OPERATOR')) {
      // Can manage wells they operate
      can('read', 'Well', { operatorId: user.organizationId });
      can('update', 'Well', {
        operatorId: user.organizationId,
        status: { $ne: 'PLUGGED' }, // Can't modify plugged wells
      });

      // Can update production for active wells
      can('updateProduction', 'Well', {
        operatorId: user.organizationId,
        status: 'PRODUCING',
      });

      // Can view but not modify partner data
      can('read', 'Partner', { wellOperatorId: user.organizationId });
      cannot('update', 'Partner');

      // Can submit compliance reports
      can('submitCompliance', 'ComplianceReport', {
        operatorId: user.organizationId,
      });
    }

    // Pumper - limited operational access
    if (user.roles.includes('PUMPER')) {
      // Can only update production for assigned wells
      can('updateProduction', 'Well', {
        assignedPumperId: user.id,
        status: 'PRODUCING',
      });

      // Can read well info but not financials
      can('read', 'Well', { assignedPumperId: user.id });
      cannot('read', 'Financial');
    }

    // Partner - view only their interest
    if (user.roles.includes('PARTNER')) {
      // Can view wells they have interest in
      can('read', 'Well', {
        'partners.partnerId': user.partnerId,
      });

      // Can view production data
      can('read', 'Production', {
        'well.partners.partnerId': user.partnerId,
      });

      // Can view their revenue share only
      can('read', 'Financial', {
        partnerId: user.partnerId,
        type: 'REVENUE_DISTRIBUTION',
      });

      // Cannot modify anything
      cannot('update', 'all');
      cannot('delete', 'all');
    }

    // Accountant - financial access
    if (user.roles.includes('ACCOUNTANT')) {
      can('read', 'Financial');
      can('update', 'Financial', {
        status: { $ne: 'POSTED' }, // Can't modify posted entries
      });

      can('read', 'Well');
      can('read', 'Production');

      // Can approve AFEs within budget limits
      can('approveAFE', 'AFE', {
        amount: { $lte: user.comrovalLimit || 50000 },
      });
    }

    // Field-level permissions
    if (user.roles.includes('REGULATOR')) {
      // Can read compliance data but not costs
      can('read', 'Well', ['name', 'apiNumber', 'status', 'location']);
      can('read', 'Production', ['date', 'oilVolume', 'gasVolume', 'waterVolume']);
      cannot('read', 'Financial');

      // Can export compliance reports
      can('exportData', 'ComplianceReport');
    }

    return build();
  }
}

// 3. Create Guards for Route Protection
@Injectable()
export class PoliciesGuard implements CanActivate {
  constructor(
    private reflector: Reflector,
    private caslAbilityFactory: CaslAbilityFactory,
  ) {}

  async canActivate(context: ExecutionContext): Promise<boolean> {
    const policyHandlers =
      this.reflector.get<PolicyHandler[]>(CHECK_POLICIES_KEY, context.getHandler()) || [];

    const request = context.switchToHttp().getRequest();
    const user = request.user;
    const ability = this.caslAbilityFactory.createForUser(user);

    return policyHandlers.every((handler) => this.execPolicyHandler(handler, ability, request));
  }

  private execPolicyHandler(handler: PolicyHandler, ability: AppAbility, request: any): boolean {
    if (typeof handler === 'function') {
      return handler(ability, request);
    }
    return handler.handle(ability, request);
  }
}

// 4. Create Decorators for Easy Use
export const CheckPolicies = (...handlers: PolicyHandler[]) =>
  SetMetadata(CHECK_POLICIES_KEY, handlers);

// Policy handler for well access
export class UpdateWellPolicyHandler implements IPolicyHandler {
  handle(ability: AppAbility, request: any): boolean {
    const well = request.well; // Assume well is loaded by middleware
    return ability.can('update', subject('Well', well));
  }
}

// 5. Use in Controllers
@Controller('wells')
@UseGuards(JwtAuthGuard, PoliciesGuard)
export class WellController {
  constructor(
    private wellService: WellService,
    private caslAbilityFactory: CaslAbilityFactory,
  ) {}

  @Put(':id/production')
  @CheckPolicies((ability, req) => ability.can('updateProduction', subject('Well', req.well)))
  async updateProduction(
    @Param('id') wellId: string,
    @Body() data: UpdateProductionDto,
    @CurrentUser() user: User,
  ) {
    // Authorization already handled by guard
    return await this.wellService.updateProduction(wellId, data);
  }

  @Get(':id/financials')
  @CheckPolicies(
    (ability) => ability.can('read', 'Financial'),
    (ability, req) => ability.can('read', subject('Well', req.well)),
  )
  async viewFinancials(@Param('id') wellId: string, @CurrentUser() user: User) {
    const ability = this.caslAbilityFactory.createForUser(user);

    // Field-level filtering
    const financials = await this.wellService.getFinancials(wellId);

    // Filter fields based on permissions
    return this.filterFieldsByAbility(financials, ability);
  }

  private filterFieldsByAbility(data: any, ability: AppAbility): any {
    const allowedFields = ability.permittedFieldsOf('read', 'Financial');

    if (allowedFields) {
      return pick(data, allowedFields);
    }

    return data;
  }
}

// 6. Implement Audit Trail
@Injectable()
export class PermissionAuditService {
  async logAccess(
    user: User,
    action: string,
    resource: string,
    result: 'ALLOWED' | 'DENIED',
    context?: any,
  ) {
    await this.auditRepository.create({
      userId: user.id,
      userRole: user.roles,
      action,
      resource,
      result,
      context,
      timestamp: new Date(),
      ip: context?.ip,
      userAgent: context?.userAgent,
    });
  }
}

// 7. Dynamic Permission Updates
@Injectable()
export class DynamicPermissionService {
  private permissionCache = new Map<string, AppAbility>();

  async refreshUserPermissions(userId: string) {
    // Clear cache when user permissions change
    this.permissionCache.delete(userId);

    // Emit event for real-time updates
    this.eventEmitter.emit('permissions.updated', { userId });
  }

  async addTemporaryPermission(userId: string, permission: Permission, expiresAt: Date) {
    // Add time-bound permission (e.g., emergency access)
    await this.tempPermissionRepository.create({
      userId,
      permission,
      expiresAt,
    });

    await this.refreshUserPermissions(userId);
  }
}

// 8. Testing Permissions
describe('Well Permissions', () => {
  it('should allow operator to update their wells', () => {
    const user = createUser({
      role: 'OPERATOR',
      organizationId: 'org-1',
    });
    const well = createWell({
      operatorId: 'org-1',
      status: 'PRODUCING',
    });

    const ability = caslAbilityFactory.createForUser(user);

    expect(ability.can('update', subject('Well', well))).toBe(true);
    expect(ability.can('updateProduction', subject('Well', well))).toBe(true);
  });

  it('should prevent partner from updating wells', () => {
    const user = createUser({ role: 'PARTNER' });
    const well = createWell({ operatorId: 'org-1' });

    const ability = caslAbilityFactory.createForUser(user);

    expect(ability.cannot('update', subject('Well', well))).toBe(true);
  });
});
```

## Benefits

1. **Centralized Authorization**: Single source of truth for permissions
2. **Declarative Rules**: Easy to understand and modify
3. **Field-Level Security**: Control access to specific data fields
4. **Dynamic Permissions**: Can be updated without code changes
5. **Testable**: Easy to unit test permission logic
6. **Audit Trail**: Track all access attempts
7. **Performance**: Permissions can be cached
8. **Multi-tenant**: Built-in support for tenant isolation

## Role-Based UI Filtering

**Frontend Navigation**: Filter navigation items based on user roles to provide intuitive UX where users only see features they can access.

```typescript
// ✅ Good: Role-based navigation filtering
export function SidebarNav() {
  const { user } = useAuth();
  const permissions = usePermissions();

  // Filter navigation items based on role
  const filteredNavItems = navItems.filter((item) => {
    // Super admin gets restricted access to system-level tools only
    if (user?.role === 'SUPER_ADMIN') {
      const allowedForSuperAdmin = ['Dashboard', 'Support'];
      return allowedForSuperAdmin.includes(item.title);
    }

    // Regular users: check permissions
    if (!item.permission) return true;
    return item.permission(permissions);
  });

  // Role-specific settings items
  const filteredSettingsItems = user?.role === 'SUPER_ADMIN'
    ? superAdminItems // System health, AI ratings, cost tracking
    : settingsItems.filter((item) => {
        if (!item.permission) return true;
        return item.permission(permissions);
      });

  return (
    <nav>
      {filteredNavItems.map((item) => (
        <NavLink key={item.href} {...item} />
      ))}
      {/* Settings section */}
      {filteredSettingsItems.map((item) => (
        <NavLink key={item.href} {...item} />
      ))}
    </nav>
  );
}
```

**Benefits**:

- Prevents confusion - users don't see features they can't access
- Reduces authorization errors - no navigation to restricted pages
- Cleaner UX - role-specific interface
- Separation of concerns - system admin vs. organization admin roles

**Use Cases**:

- **Super Admin**: System health monitoring, cost tracking, AI ratings (no org data access)
- **Organization Admin**: Full operational features (projects, clients, invoices, etc.)
- **Manager**: Team management, approvals, reporting
- **Consultant**: Time tracking, own expenses, project assignments

## Implementation Checklist

- [ ] Define all actions and subjects
- [ ] Create ability factory with rules
- [ ] Implement guards for route protection
- [ ] Add decorators for easy use
- [ ] Set up permission caching
- [ ] Create audit logging
- [ ] Add field-level filtering
- [ ] Implement dynamic permission updates
- [ ] Create permission testing utilities
- [ ] Document permission matrix
- [ ] Add permission management UI
- [ ] Set up emergency access procedures
- [ ] Implement role-based UI navigation filtering
