# Pattern 51: Feature Flags Pattern

**Version**: 1.0
**Last Updated**: October 8, 2025
**Category**: Deployment & Release Management

---

## Table of Contents

1. [Overview](#overview)
2. [When to Use](#when-to-use)
3. [Feature Flag Types](#feature-flag-types)
4. [Flag Naming Conventions](#flag-naming-conventions)
5. [Backend Implementation](#backend-implementation)
6. [Frontend Implementation](#frontend-implementation)
7. [Flag Evaluation](#flag-evaluation)
8. [Targeting Rules](#targeting-rules)
9. [A/B Testing](#ab-testing)
10. [Flag Management](#flag-management)
11. [Performance Optimization](#performance-optimization)
12. [Best Practices](#best-practices)
13. [Anti-Patterns](#anti-patterns)
14. [Related Patterns](#related-patterns)
15. [References](#references)

---

## Overview

**Feature Flags** (also called feature toggles or feature switches) allow you to enable/disable features without deploying code. They decouple deployment from release, enabling progressive rollouts, A/B testing, and emergency kill switches.

**Without Feature Flags**:

```typescript
// Deploy new invoice UI to production - All or nothing!
return <NewInvoiceForm />
```

**With Feature Flags**:

```typescript
// Deploy to production, but only show to beta users
if (featureFlags.isEnabled('new-invoice-ui', user)) {
  return <NewInvoiceForm />
} else {
  return <OldInvoiceForm />
}
```

**Use Cases in WellOS**:

- **Progressive rollouts** - "Enable new time tracking UI for 10% of users"
- **Beta features** - "Show AI-powered insights only to beta testers"
- **Kill switches** - "Disable QuickBooks integration if API is down"
- **A/B testing** - "Show variant A to 50%, variant B to 50%"
- **Dark launches** - Deploy code to production but keep hidden
- **Canary releases** - Test on internal users first

---

## When to Use

### ✅ Use Feature Flags When

1. **Testing in production** - Validate with real users before full release
   - New dashboard layout
   - AI-powered features
   - Performance optimizations

2. **Progressive rollouts** - Gradual rollout to mitigate risk
   - Start with 1%, then 10%, 50%, 100%
   - Monitor metrics at each stage

3. **A/B testing** - Compare feature variants
   - Pricing page layouts
   - Onboarding flows
   - Call-to-action button text

4. **Emergency kill switches** - Disable problematic features instantly
   - External API integrations
   - Resource-intensive features

5. **Beta programs** - Exclusive access for beta testers
   - Early access to new features
   - Invite-only functionality

### ❌ Don't Use Feature Flags When

1. **Simple bug fixes** - Just deploy the fix
2. **Critical security patches** - Deploy immediately without flags
3. **Database schema changes** - Use migrations instead
4. **Long-term conditional logic** - Becomes technical debt

---

## Feature Flag Types

### 1. Release Flags

**Purpose**: Control feature rollout
**Lifetime**: Temporary (remove after 100% rollout)

```typescript
if (featureFlags.isEnabled('new-dashboard')) {
  return <NewDashboard />
}
```

### 2. Experiment Flags

**Purpose**: A/B testing
**Lifetime**: Until experiment concludes

```typescript
const variant = featureFlags.getVariant('pricing-page-test', user);
if (variant === 'A') {
  return <PricingPageA />
} else {
  return <PricingPageB />
}
```

### 3. Operational Flags

**Purpose**: Runtime control (kill switches)
**Lifetime**: Permanent

```typescript
if (featureFlags.isEnabled('quickbooks-integration')) {
  await quickbooksService.sync();
}
```

### 4. Permission Flags

**Purpose**: User-specific access control
**Lifetime**: Permanent

```typescript
if (featureFlags.isEnabled('admin-panel', user)) {
  return <AdminPanel />
}
```

---

## Flag Naming Conventions

Use consistent naming for clarity:

```typescript
// ✅ Good naming
'new-invoice-ui'; // Release flag
'ai-insights-experiment'; // Experiment flag
'quickbooks-integration-kill'; // Operational flag
'beta-features'; // Permission flag

// ❌ Bad naming
'feature1'; // Not descriptive
'newStuff'; // Inconsistent casing
'temp-fix-dont-use'; // Unclear purpose
```

**Convention**:

- Use kebab-case
- Be descriptive
- Include context (component, experiment, etc.)
- Suffix with `-kill` for kill switches
- Suffix with `-experiment` for A/B tests

---

## Backend Implementation

### Feature Flag Schema

```sql
-- Feature flags schema
CREATE TABLE feature_flags (
  id VARCHAR(255) PRIMARY KEY,
  key VARCHAR(100) NOT NULL UNIQUE,
  description TEXT NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT false,
  type VARCHAR(50) NOT NULL, -- 'release' | 'experiment' | 'operational' | 'permission'
  targeting_rules JSONB,
  variants JSONB,
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE TABLE user_feature_flags (
  id VARCHAR(255) PRIMARY KEY,
  user_id VARCHAR(255) NOT NULL,
  flag_key VARCHAR(100) NOT NULL,
  enabled BOOLEAN NOT NULL,
  variant VARCHAR(100),
  created_at TIMESTAMP NOT NULL DEFAULT NOW(),
  UNIQUE(user_id, flag_key)
);

CREATE INDEX idx_feature_flags_key ON feature_flags(key);
CREATE INDEX idx_user_feature_flags_lookup ON user_feature_flags(user_id, flag_key);
```

### Feature Flag Service

```typescript
// Feature flag service with caching
export interface TargetingRule {
  attribute: string; // 'userId' | 'organizationId' | 'email' | 'role'
  operator: 'equals' | 'in' | 'contains' | 'percentage';
  value: any;
}

export interface FlagVariant {
  name: string;
  weight: number; // 0-100 (percentage)
}

export interface FlagContext {
  userId?: string;
  organizationId?: string;
  email?: string;
  role?: string;
  [key: string]: any;
}

export class FeatureFlagService {
  constructor(
    private readonly db: DatabaseConnection,
    private readonly cache: CacheService,
  ) {}

  async isEnabled(flagKey: string, context: FlagContext = {}): Promise<boolean> {
    // Check user-specific override first
    if (context.userId) {
      const userFlag = await this.getUserFlag(flagKey, context.userId);
      if (userFlag !== null) {
        return userFlag;
      }
    }

    // Get flag definition (with caching)
    const flag = await this.getFlag(flagKey);
    if (!flag) {
      return false; // Flag doesn't exist - default to disabled
    }

    if (!flag.enabled) {
      return false; // Flag is globally disabled
    }

    // Evaluate targeting rules
    return this.evaluateTargetingRules(flag.targetingRules || [], context);
  }

  async getVariant(flagKey: string, context: FlagContext): Promise<string | null> {
    if (!(await this.isEnabled(flagKey, context))) {
      return null;
    }

    const flag = await this.getFlag(flagKey);
    if (!flag?.variants || flag.variants.length === 0) {
      return null;
    }

    // Deterministic variant assignment based on user ID
    if (context.userId) {
      return this.assignVariant(context.userId, flag.variants);
    }

    // Random assignment if no user ID
    return this.assignVariantRandom(flag.variants);
  }

  async setUserFlag(flagKey: string, userId: string, enabled: boolean): Promise<void> {
    await this.db.query(
      `INSERT INTO user_feature_flags (id, user_id, flag_key, enabled)
       VALUES ($1, $2, $3, $4)
       ON CONFLICT (user_id, flag_key)
       DO UPDATE SET enabled = $4`,
      [uuidv4(), userId, flagKey, enabled]
    );

    // Invalidate cache
    await this.cache.delete(`user-flag:${userId}:${flagKey}`);
  }

  private async getFlag(flagKey: string): Promise<any> {
    const cacheKey = `flag:${flagKey}`;
    const cached = await this.cache.get(cacheKey);
    if (cached) return cached;

    const result = await this.db.query(
      `SELECT * FROM feature_flags WHERE key = $1 LIMIT 1`,
      [flagKey]
    );

    const flag = result.rows[0] || null;
    await this.cache.set(cacheKey, flag, 300); // Cache for 5 minutes

    return flag;
  }

  private async getUserFlag(flagKey: string, userId: string): Promise<boolean | null> {
    const cacheKey = `user-flag:${userId}:${flagKey}`;
    const cached = await this.cache.get<boolean>(cacheKey);
    if (cached !== null) return cached;

    const result = await this.db.query(
      `SELECT enabled FROM user_feature_flags
       WHERE user_id = $1 AND flag_key = $2 LIMIT 1`,
      [userId, flagKey]
    );

    const enabled = result.rows[0]?.enabled ?? null;
    await this.cache.set(cacheKey, enabled, 300);

    return enabled;
  }

  private evaluateTargetingRules(rules: TargetingRule[], context: FlagContext): boolean {
    if (rules.length === 0) {
      return true; // No rules = enabled for all
    }

    // All rules must match (AND logic)
    return rules.every((rule) => this.evaluateRule(rule, context));
  }

  private evaluateRule(rule: TargetingRule, context: FlagContext): boolean {
    const contextValue = context[rule.attribute];

    switch (rule.operator) {
      case 'equals':
        return contextValue === rule.value;

      case 'in':
        return Array.isArray(rule.value) && rule.value.includes(contextValue);

      case 'contains':
        return typeof contextValue === 'string' && contextValue.includes(rule.value);

      case 'percentage':
        // Deterministic percentage based on user ID hash
        if (!context.userId) return false;
        const hash = this.hashString(context.userId);
        const percentage = hash % 100;
        return percentage < rule.value;

      default:
        return false;
    }
  }

  private assignVariant(userId: string, variants: FlagVariant[]): string {
    // Deterministic assignment using user ID hash
    const hash = this.hashString(userId);
    const percentage = hash % 100;

    let cumulative = 0;
    for (const variant of variants) {
      cumulative += variant.weight;
      if (percentage < cumulative) {
        return variant.name;
      }
    }

    return variants[variants.length - 1].name;
  }

  private assignVariantRandom(variants: FlagVariant[]): string {
    const random = Math.random() * 100;

    let cumulative = 0;
    for (const variant of variants) {
      cumulative += variant.weight;
      if (random < cumulative) {
        return variant.name;
      }
    }

    return variants[variants.length - 1].name;
  }

  private hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = (hash << 5) - hash + char;
      hash |= 0; // Convert to 32-bit integer
    }
    return Math.abs(hash);
  }
}
```

---

## Frontend Implementation

### Feature Flag Hook

```typescript
// apps/web/hooks/use-feature-flag.ts
import { useQuery } from '@tanstack/react-query';
import { featureFlagRepository } from '@/lib/repositories/feature-flag.repository';

export const useFeatureFlag = (flagKey: string) => {
  return useQuery({
    queryKey: ['feature-flag', flagKey],
    queryFn: () => featureFlagRepository.isEnabled(flagKey),
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
    cacheTime: 10 * 60 * 1000,
  });
};

export const useFeatureVariant = (flagKey: string) => {
  return useQuery({
    queryKey: ['feature-variant', flagKey],
    queryFn: () => featureFlagRepository.getVariant(flagKey),
    staleTime: 5 * 60 * 1000,
    cacheTime: 10 * 60 * 1000,
  });
};

// Usage in components
function InvoiceForm() {
  const { data: isNewUIEnabled, isLoading } = useFeatureFlag('new-invoice-ui');

  if (isLoading) {
    return <Skeleton />;
  }

  if (isNewUIEnabled) {
    return <NewInvoiceForm />;
  }

  return <OldInvoiceForm />;
}
```

### Feature Flag Component

```typescript
// apps/web/components/feature-flag.tsx
'use client';

import React from 'react';
import { useFeatureFlag } from '@/hooks/use-feature-flag';

interface FeatureFlagProps {
  flag: string;
  children: React.ReactNode;
  fallback?: React.ReactNode;
}

export const FeatureFlag: React.FC<FeatureFlagProps> = ({
  flag,
  children,
  fallback = null,
}) => {
  const { data: isEnabled, isLoading } = useFeatureFlag(flag);

  if (isLoading) {
    return <>{fallback}</>;
  }

  return isEnabled ? <>{children}</> : <>{fallback}</>;
};

// Usage
<FeatureFlag flag="ai-insights" fallback={<Skeleton />}>
  <AIInsightsPanel />
</FeatureFlag>
```

---

## Flag Evaluation

### Targeting Rules Examples

#### 1. User-Based Targeting

```typescript
// Enable for specific users
const flag = {
  key: 'beta-features',
  enabled: true,
  targetingRules: [
    {
      attribute: 'userId',
      operator: 'in',
      value: ['user-1', 'user-2', 'user-3'],
    },
  ],
};
```

#### 2. Organization-Based Targeting

```typescript
// Enable for specific organizations
const flag = {
  key: 'enterprise-features',
  enabled: true,
  targetingRules: [
    {
      attribute: 'organizationId',
      operator: 'in',
      value: ['org-acme', 'org-beta-corp'],
    },
  ],
};
```

#### 3. Percentage Rollout

```typescript
// Enable for 25% of users
const flag = {
  key: 'new-dashboard',
  enabled: true,
  targetingRules: [
    {
      attribute: 'userId',
      operator: 'percentage',
      value: 25,
    },
  ],
};
```

#### 4. Email Domain Targeting

```typescript
// Enable for internal users only
const flag = {
  key: 'internal-tools',
  enabled: true,
  targetingRules: [
    {
      attribute: 'email',
      operator: 'contains',
      value: '@company.com',
    },
  ],
};
```

#### 5. Role-Based Targeting

```typescript
// Enable for admins only
const flag = {
  key: 'admin-analytics',
  enabled: true,
  targetingRules: [
    {
      attribute: 'role',
      operator: 'equals',
      value: 'ADMIN',
    },
  ],
};
```

---

## A/B Testing

### Variant Configuration

```typescript
// A/B test: Pricing page layout
const flag = {
  key: 'pricing-page-experiment',
  type: 'experiment',
  enabled: true,
  variants: [
    { name: 'control', weight: 50 },   // 50% see control
    { name: 'variant-a', weight: 50 }, // 50% see variant A
  ],
};

// Usage
const variant = await featureFlags.getVariant('pricing-page-experiment', { userId });

if (variant === 'variant-a') {
  return <PricingPageVariantA />;
} else {
  return <PricingPageControl />;
}
```

### Multi-Variant Testing

```typescript
// A/B/C test: Onboarding flow
const flag = {
  key: 'onboarding-flow-test',
  type: 'experiment',
  enabled: true,
  variants: [
    { name: 'control', weight: 33 },
    { name: 'short-flow', weight: 33 },
    { name: 'gamified', weight: 34 },
  ],
};
```

### Tracking Experiment Metrics

```typescript
// Experiment tracking service
export class ExperimentTrackerService {
  constructor(
    private readonly featureFlags: FeatureFlagService,
    private readonly analytics: AnalyticsService,
  ) {}

  async trackConversion(
    experimentKey: string,
    userId: string,
    metric: string,
    value: number,
  ): Promise<void> {
    const variant = await this.featureFlags.getVariant(experimentKey, { userId });

    if (variant) {
      await this.analytics.track({
        event: 'experiment_conversion',
        userId,
        properties: {
          experiment: experimentKey,
          variant,
          metric,
          value,
        },
      });
    }
  }
}

// Usage in application code
await experimentTracker.trackConversion(
  'pricing-page-experiment',
  userId,
  'subscription_created',
  1,
);
```

---

## Flag Management

### Admin API for Flag Management

```typescript
// Admin API for feature flag management
export class FeatureFlagsController {
  constructor(private readonly featureFlags: FeatureFlagService) {}

  async listFlags() {
    return this.featureFlags.listAll();
  }

  async createFlag(dto: CreateFeatureFlagDto) {
    return this.featureFlags.create(dto);
  }

  async updateFlag(key: string, dto: UpdateFeatureFlagDto) {
    return this.featureFlags.update(key, dto);
  }

  async updateRollout(key: string, dto: RolloutDto) {
    // Gradually increase percentage rollout
    return this.featureFlags.updateTargetingRules(key, [
      {
        attribute: 'userId',
        operator: 'percentage',
        value: dto.percentage,
      },
    ]);
  }

  async deleteFlag(key: string) {
    return this.featureFlags.delete(key);
  }
}
```

### Admin UI

```typescript
// apps/web/app/(dashboard)/admin/feature-flags/page.tsx
'use client';

export default function FeatureFlagsPage() {
  const { data: flags } = useQuery({
    queryKey: ['admin', 'feature-flags'],
    queryFn: () => fetch('/api/admin/feature-flags').then(r => r.json()),
  });

  return (
    <div className="space-y-4">
      <h1>Feature Flags</h1>

      {flags?.map((flag) => (
        <Card key={flag.key}>
          <div className="flex items-center justify-between">
            <div>
              <h3>{flag.key}</h3>
              <p className="text-sm text-gray-600">{flag.description}</p>
            </div>

            <Switch
              checked={flag.enabled}
              onCheckedChange={(enabled) => updateFlag(flag.key, { enabled })}
            />
          </div>

          {flag.targetingRules?.map((rule, i) => (
            <div key={i} className="text-sm mt-2">
              {rule.attribute} {rule.operator} {JSON.stringify(rule.value)}
            </div>
          ))}
        </Card>
      ))}
    </div>
  );
}
```

---

## Performance Optimization

### Bulk Flag Evaluation

```typescript
// Fetch all flags for a user in one request
async getAllFlags(context: FlagContext): Promise<Record<string, boolean>> {
  const flags = await this.db.select().from(featureFlags);

  const results: Record<string, boolean> = {};
  for (const flag of flags) {
    results[flag.key] = await this.isEnabled(flag.key, context);
  }

  return results;
}

// Frontend usage
const { data: allFlags } = useQuery({
  queryKey: ['feature-flags', 'all'],
  queryFn: () => featureFlagRepository.getAll(),
});

// Check flags without additional API calls
if (allFlags?.['new-invoice-ui']) {
  return <NewInvoiceForm />;
}
```

### Server-Side Flag Injection

```typescript
// apps/web/app/layout.tsx
import { FeatureFlagProvider } from '@/providers/feature-flag-provider';

export default async function RootLayout({ children }) {
  // Fetch flags server-side
  const flags = await getFeatureFlags();

  return (
    <html>
      <body>
        <FeatureFlagProvider initialFlags={flags}>
          {children}
        </FeatureFlagProvider>
      </body>
    </html>
  );
}

// Provider makes flags available without API calls
function useFeatureFlag(key: string) {
  const flags = useContext(FeatureFlagContext);
  return flags[key] ?? false;
}
```

---

## Best Practices

### ✅ DO

1. **Remove old flags** - Delete after full rollout (technical debt!)

   ```typescript
   // Set reminder to remove flag after 100% rollout
   // TODO: Remove 'new-dashboard' flag by 2025-11-01
   ```

2. **Use descriptive names** - Clear purpose

   ```typescript
   'ai-insights-beta'; // Good
   'feature-x'; // Bad
   ```

3. **Default to disabled** - Fail-safe

   ```typescript
   const enabled = (await featureFlags.isEnabled(key)) ?? false;
   ```

4. **Cache flag evaluations** - Reduce database load

   ```typescript
   staleTime: 5 * 60 * 1000; // 5 minutes
   ```

5. **Track flag usage** - Understand impact

   ```typescript
   logger.info('Feature flag evaluated', { flag: key, enabled });
   ```

6. **Document flags** - Why it exists, when to remove

   ```typescript
   description: 'New invoice UI - Remove after Q4 2025 rollout';
   ```

---

### ❌ DON'T

1. **Don't use flags for permission checks** - Use RBAC instead

   ```typescript
   // ❌ Bad: Flag for permissions
   if (featureFlags.isEnabled('admin-access', user)) {
   }

   // ✅ Good: RBAC
   if (user.hasRole('ADMIN')) {
   }
   ```

2. **Don't nest flags deeply** - Complexity nightmare

   ```typescript
   // ❌ Bad: Nested flags
   if (flagA) {
     if (flagB) {
       if (flagC) {
         /* ... */
       }
     }
   }

   // ✅ Good: Combine into single flag
   if (flagCombined) {
     /* ... */
   }
   ```

3. **Don't forget to clean up** - Set expiration dates
4. **Don't hardcode flag keys** - Use constants

   ```typescript
   // ❌ Bad
   featureFlags.isEnabled('new-dashboard');

   // ✅ Good
   const FLAGS = {
     NEW_DASHBOARD: 'new-dashboard',
   } as const;
   featureFlags.isEnabled(FLAGS.NEW_DASHBOARD);
   ```

---

## Anti-Patterns

### 1. Flag Debt

```typescript
// ❌ Anti-pattern: 5-year-old flag still in code
if (featureFlags.isEnabled('legacy-feature-2020')) {
  // This flag has been 100% enabled for 4 years!
}

// ✅ Solution: Remove flag, simplify code
// Just use the new code path directly
```

### 2. Permission Flags

```typescript
// ❌ Anti-pattern: Flags for authorization
if (featureFlags.isEnabled('can-delete-projects', user)) {
  await projectRepository.delete(id);
}

// ✅ Solution: Use RBAC
if (user.hasPermission('projects:delete')) {
  await projectRepository.delete(id);
}
```

---

## Related Patterns

- **Pattern 03: Hexagonal Architecture** - Keep flag logic out of domain
- **Pattern 05: CQRS Pattern** - Flags for feature experiments
- **Pattern 46: Caching Strategy Patterns** - Cache flag evaluations
- **Pattern 47: Monitoring & Observability Patterns** - Track flag metrics

---

## References

### Services

- **LaunchDarkly** - Feature flag platform (SaaS)
- **Unleash** - Open-source feature flag platform
- **Split.io** - Feature flags + experimentation
- **Flagsmith** - Open-source feature flags

### Books & Articles

- **"Feature Toggles"** by Pete Hodgson (Martin Fowler blog)
- **"Effective Feature Management"** - Best practices guide
- **"Trunk-Based Development"** - Continuous integration with flags

---

## Summary

**Feature Flags Pattern** enables progressive feature rollouts:

✅ **Progressive rollouts** - Start with 1%, gradually increase to 100%
✅ **A/B testing** - Compare variants scientifically
✅ **Kill switches** - Disable problematic features instantly
✅ **Beta programs** - Exclusive access for testers
✅ **Dark launches** - Deploy code without exposing features
✅ **Decouple deployment from release** - Deploy anytime, release when ready

**Remember**: Feature flags are temporary (except operational flags). Remove them after full rollout to avoid technical debt!

---

**Next Steps**:

1. Set up feature flag database schema
2. Implement feature flag service with caching
3. Create admin UI for flag management
4. Add feature flag hooks for frontend
5. Define flag naming conventions
6. Set up automated flag cleanup reminders
