# 77 - Form Field Auto-Generation Pattern

**Category:** Frontend Patterns
**Complexity:** Medium
**Last Updated:** October 25, 2025

---

## Problem Statement

Enterprise admin forms often require multiple related fields that can be automatically derived from a single user input:

- **Manual field entry** is tedious and error-prone (e.g., entering company name, slug, subdomain separately)
- **Field synchronization** becomes difficult when values must stay consistent
- **User errors** increase when manually typing derived values (typos in slug vs subdomain)
- **Complex transformation rules** need to be consistently applied (e.g., "ACME Oil & Gas" → "acme-oil-gas")

Traditional form implementations require users to manually fill each field, even when logical transformations exist between them.

## Solution Overview

Implement cascading auto-generation using React Hook Form's `watch` and `useEffect` to automatically derive field values from user input while preserving manual override capability.

**Core Principles:**

1. **Watch Primary Fields** - Use `form.watch()` to detect changes in source fields
2. **Transform & Validate** - Apply transformation rules (lowercase, hyphenate, sanitize)
3. **Conditional Updates** - Only auto-generate for new entries (not edits)
4. **Cascade Dependencies** - Chain transformations (name → slug → subdomain)
5. **User Override** - Allow manual editing if auto-generation doesn't fit

## When to Use

✅ **Use this pattern when:**

- Multiple form fields have logical derivation relationships
- Field values follow consistent transformation rules (e.g., URL slugs, email prefixes)
- Creating admin/configuration interfaces with technical field requirements
- Reducing user input burden while maintaining data consistency
- Building tenant/organization creation forms

❌ **Don't use this pattern when:**

- Fields are truly independent with no logical relationship
- Transformation rules are complex or ambiguous (require user decision)
- Users need full control over all field values (no assumptions)
- Form values come from external systems (API pre-population)

## Implementation

### Example: Tenant Creation Form with Auto-Generated Slug and Subdomain

#### Step 1: Define Form Schema with Related Fields

```typescript
// apps/admin/components/tenants/tenant-dialog.tsx
import { zodResolver } from '@hookform/resolvers/zod';
import { useForm } from 'react-hook-form';
import * as z from 'zod';

const tenantFormSchema = z.object({
  name: z.string().min(1, 'Company name is required'),

  // Auto-generated from 'name'
  slug: z
    .string()
    .min(3, 'Slug must be at least 3 characters')
    .regex(/^[a-z0-9-]+$/, 'Slug can only contain lowercase letters, numbers, and hyphens'),

  // Auto-generated from 'slug'
  subdomain: z
    .string()
    .min(3, 'Subdomain must be at least 3 characters')
    .regex(/^[a-z0-9-]+$/, 'Subdomain can only contain lowercase letters, numbers, and hyphens'),

  contactEmail: z.string().email('Invalid email address'),
  subscriptionTier: z.enum(['STARTER', 'PROFESSIONAL', 'ENTERPRISE', 'ENTERPRISE_PLUS']),
  trialDays: z.number().min(0).max(90).optional(),
});

type TenantFormValues = z.infer<typeof tenantFormSchema>;
```

#### Step 2: Initialize Form with Watch Capability

```typescript
export function TenantDialog({ open, onOpenChange, tenant, onSubmit }: TenantDialogProps) {
  const isEditMode = !!tenant;
  const [isSubmitting, setIsSubmitting] = React.useState(false);

  const form = useForm<TenantFormValues>({
    resolver: zodResolver(tenantFormSchema),
    defaultValues: {
      name: tenant?.name || '',
      slug: tenant?.slug || '',
      subdomain: tenant?.subdomain || '',
      contactEmail: tenant?.contactEmail || '',
      subscriptionTier: tenant?.subscriptionTier as TenantFormValues['subscriptionTier'] || 'STARTER',
      trialDays: 30,
    },
  });

  // ... rest of component
}
```

#### Step 3: Auto-Generate Slug from Name (Primary Transformation)

```typescript
// Watch the 'name' field for changes
const watchName = form.watch('name');

React.useEffect(() => {
  // Only auto-generate for new tenants (not edits)
  if (!isEditMode && watchName) {
    // Transformation rules:
    // 1. Convert to lowercase
    // 2. Remove special characters (keep alphanumeric, spaces, hyphens)
    // 3. Replace spaces with hyphens
    // 4. Collapse multiple hyphens
    // 5. Truncate to 50 chars
    const generatedSlug = watchName
      .toLowerCase()
      .replace(/[^a-z0-9\s-]/g, '')  // Remove special chars
      .replace(/\s+/g, '-')           // Spaces to hyphens
      .replace(/-+/g, '-')            // Collapse multiple hyphens
      .substring(0, 50);              // Limit length

    form.setValue('slug', generatedSlug);
  }
}, [watchName, isEditMode, form]);
```

**Transformation Examples:**

| Input Name          | Generated Slug       |
|---------------------|----------------------|
| "ACME Oil & Gas"    | "acme-oil-gas"       |
| "Demo Corporation"  | "demo-corporation"   |
| "WellOS Internal"| "wellos-internal" |
| "Test   Company!!!" | "test-company"       |

#### Step 4: Auto-Generate Subdomain from Slug (Cascading Transformation)

```typescript
// Watch the 'slug' field for changes (cascades from 'name' changes)
const watchSlug = form.watch('slug');

React.useEffect(() => {
  // Only auto-generate for new tenants
  if (!isEditMode && watchSlug) {
    // Subdomain mirrors slug exactly (already sanitized)
    form.setValue('subdomain', watchSlug);
  }
}, [watchSlug, isEditMode, form]);
```

**Cascading Flow:**

```
User types: "ACME Oil & Gas"
     ↓
name = "ACME Oil & Gas"
     ↓ (useEffect #1 triggers)
slug = "acme-oil-gas"
     ↓ (useEffect #2 triggers)
subdomain = "acme-oil-gas"
     ↓
Final URL: acme-oil-gas.onwellos.com
```

#### Step 5: Render Form Fields with Visual Feedback

```typescript
<Form {...form}>
  <form onSubmit={form.handleSubmit(handleSubmit)} className="space-y-4">
    {/* Primary Field - User Input */}
    <FormField
      control={form.control}
      name="name"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Company Name</FormLabel>
          <FormControl>
            <Input
              placeholder="ACME Oil & Gas"
              {...field}
              disabled={isEditMode}  // Lock on edit
            />
          </FormControl>
          <FormDescription>
            Used to generate the tenant ID (e.g., ACME-A5L32W)
          </FormDescription>
          <FormMessage />
        </FormItem>
      )}
    />

    {/* Auto-Generated Field #1 - Slug */}
    <FormField
      control={form.control}
      name="slug"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Slug</FormLabel>
          <FormControl>
            <Input
              placeholder="acme-oil-gas"
              {...field}
              disabled={isEditMode}  // Lock on edit (DB constraint)
            />
          </FormControl>
          <FormDescription>
            Auto-generated from company name. Used for database naming.
          </FormDescription>
          <FormMessage />
        </FormItem>
      )}
    />

    {/* Auto-Generated Field #2 - Subdomain */}
    <FormField
      control={form.control}
      name="subdomain"
      render={({ field }) => (
        <FormItem>
          <FormLabel>Subdomain</FormLabel>
          <FormControl>
            <div className="flex items-center">
              <Input
                placeholder="acme"
                {...field}
                disabled={isEditMode}  // Lock on edit (DB constraint)
              />
              <span className="ml-2 text-sm text-muted-foreground">
                .onwellos.com
              </span>
            </div>
          </FormControl>
          <FormDescription>
            {isEditMode
              ? 'Subdomain cannot be changed after creation'
              : 'Auto-generated from slug. Used for tenant-specific URLs.'}
          </FormDescription>
          <FormMessage />
        </FormItem>
      )}
    />

    {/* ... other fields ... */}
  </form>
</Form>
```

#### Step 6: Prevent Auto-Generation on Edit Mode

```typescript
React.useEffect(() => {
  if (tenant) {
    // Editing existing tenant - populate with saved values
    form.reset({
      name: tenant.name,
      slug: tenant.slug,
      subdomain: tenant.subdomain,
      contactEmail: tenant.contactEmail,
      subscriptionTier: tenant.subscriptionTier as TenantFormValues['subscriptionTier'],
      trialDays: 30,
    });
  } else {
    // Creating new tenant - reset to defaults
    form.reset({
      name: '',
      slug: '',
      subdomain: '',
      contactEmail: '',
      subscriptionTier: 'STARTER',
      trialDays: 30,
    });
  }
}, [tenant, form]);
```

**Edit Mode Behavior:**

- `isEditMode = !!tenant` - Determines if we're editing existing entity
- Auto-generation `useEffect` hooks check `!isEditMode` before updating
- Fields are `disabled={isEditMode}` to prevent accidental changes
- Database constraints (unique slug, subdomain) prevent changes anyway

## Advanced Variations

### Variation 1: Conditional Auto-Generation with Toggle

Allow users to disable auto-generation if they want full control:

```typescript
const [autoGenerate, setAutoGenerate] = React.useState(true);

React.useEffect(() => {
  if (!isEditMode && watchName && autoGenerate) {
    const generatedSlug = transformNameToSlug(watchName);
    form.setValue('slug', generatedSlug);
  }
}, [watchName, isEditMode, autoGenerate, form]);

// UI Toggle
<div className="flex items-center space-x-2">
  <Switch
    id="auto-generate"
    checked={autoGenerate}
    onCheckedChange={setAutoGenerate}
  />
  <Label htmlFor="auto-generate">Auto-generate slug and subdomain</Label>
</div>
```

### Variation 2: Live Preview Before Auto-Generation

Show users the generated value before applying it:

```typescript
const previewSlug = React.useMemo(() => {
  if (!watchName) return '';
  return watchName
    .toLowerCase()
    .replace(/[^a-z0-9\s-]/g, '')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .substring(0, 50);
}, [watchName]);

// UI Preview
<div className="text-sm text-muted-foreground">
  Preview: <code>{previewSlug}</code>
  <Button size="sm" onClick={() => form.setValue('slug', previewSlug)}>
    Use this slug
  </Button>
</div>
```

### Variation 3: Multi-Source Auto-Generation

Combine multiple fields for complex transformations:

```typescript
const watchFirstName = form.watch('firstName');
const watchLastName = form.watch('lastName');

React.useEffect(() => {
  if (!isEditMode && watchFirstName && watchLastName) {
    const email = `${watchFirstName.toLowerCase()}.${watchLastName.toLowerCase()}@onwellos.com`;
    form.setValue('email', email);
  }
}, [watchFirstName, watchLastName, isEditMode, form]);
```

## Trade-offs

### Advantages ✅

1. **Reduced User Input** - Users only type primary field (company name)
2. **Consistency** - Transformations applied uniformly across all tenants
3. **Error Prevention** - Eliminates typos in technical fields (slugs, subdomains)
4. **Speed** - Forms complete faster with fewer required inputs
5. **Validation** - Zod schema ensures auto-generated values meet constraints

### Disadvantages ❌

1. **Loss of Control** - Users can't easily customize derived values (must disable auto-gen)
2. **Complexity** - Multiple `useEffect` hooks increase component complexity
3. **Performance** - Watchers trigger on every keystroke (mitigated by React's batching)
4. **Testing** - More edge cases to test (empty strings, special characters, cascading updates)
5. **Debugging** - Form state changes can be hard to trace with multiple auto-updates

## Testing Strategies

### Unit Tests for Transformation Functions

```typescript
// __tests__/utils/slug-transformer.test.ts
import { transformNameToSlug } from '@/lib/utils/slug-transformer';

describe('transformNameToSlug', () => {
  it('should convert uppercase to lowercase', () => {
    expect(transformNameToSlug('ACME')).toBe('acme');
  });

  it('should replace spaces with hyphens', () => {
    expect(transformNameToSlug('ACME Oil Gas')).toBe('acme-oil-gas');
  });

  it('should remove special characters', () => {
    expect(transformNameToSlug('ACME Oil & Gas!')).toBe('acme-oil-gas');
  });

  it('should collapse multiple hyphens', () => {
    expect(transformNameToSlug('ACME   Oil---Gas')).toBe('acme-oil-gas');
  });

  it('should truncate to 50 characters', () => {
    const longName = 'A'.repeat(100);
    expect(transformNameToSlug(longName).length).toBe(50);
  });
});
```

### Integration Tests for Form Behavior

```typescript
// __tests__/components/tenant-dialog.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { TenantDialog } from '@/components/tenants/tenant-dialog';

describe('TenantDialog Auto-Generation', () => {
  it('should auto-generate slug when name is typed', async () => {
    const user = userEvent.setup();
    render(
      <TenantDialog
        open={true}
        onOpenChange={jest.fn()}
        onSubmit={jest.fn()}
      />
    );

    const nameInput = screen.getByLabelText(/company name/i);
    await user.type(nameInput, 'ACME Oil & Gas');

    const slugInput = screen.getByLabelText(/slug/i);
    await waitFor(() => {
      expect(slugInput).toHaveValue('acme-oil-gas');
    });
  });

  it('should auto-generate subdomain when slug changes', async () => {
    const user = userEvent.setup();
    render(
      <TenantDialog
        open={true}
        onOpenChange={jest.fn()}
        onSubmit={jest.fn()}
      />
    );

    const nameInput = screen.getByLabelText(/company name/i);
    await user.type(nameInput, 'Demo Corp');

    const subdomainInput = screen.getByLabelText(/subdomain/i);
    await waitFor(() => {
      expect(subdomainInput).toHaveValue('demo-corp');
    });
  });

  it('should NOT auto-generate in edit mode', async () => {
    const user = userEvent.setup();
    const existingTenant = {
      id: '1',
      name: 'ACME',
      slug: 'acme-original',
      subdomain: 'acme-original',
      // ...
    };

    render(
      <TenantDialog
        open={true}
        onOpenChange={jest.fn()}
        tenant={existingTenant}
        onSubmit={jest.fn()}
      />
    );

    const slugInput = screen.getByLabelText(/slug/i);
    expect(slugInput).toHaveValue('acme-original');
    expect(slugInput).toBeDisabled(); // Cannot edit
  });
});
```

## Related Patterns

- **[07 - DTO Pattern](./07-DTO-Pattern.md)** - Form values transform to DTOs for API submission
- **[61 - Value Object Layer Boundary Pattern](./61-Value-Object-Layer-Boundary-Pattern.md)** - Generated values often become value objects in domain layer
- **[52 - User-Friendly Error Handling Pattern](./52-User-Friendly-Error-Handling-Pattern.md)** - Validation errors for auto-generated fields

## Real-World Examples in WellOS

### 1. Tenant Creation (Admin Portal)

**File:** `apps/admin/components/tenants/tenant-dialog.tsx`

```
User Input:    "ACME Oil & Gas"
     ↓
Auto-Gen Slug: "acme-oil-gas"
     ↓
Auto-Gen Sub:  "acme-oil-gas"
     ↓
Final URL:     acme-oil-gas.onwellos.com
Database:      acme_oil_gas_wellos
```

### 2. Well Creation (Operator Dashboard)

**Future Implementation:**

```typescript
// Auto-generate API number from well name
const watchWellName = form.watch('wellName');
React.useEffect(() => {
  if (watchWellName) {
    const apiNumber = generateAPINumber(watchWellName, county, operator);
    form.setValue('apiNumber', apiNumber);
  }
}, [watchWellName]);
```

### 3. User Email Generation (Organization Setup)

```typescript
// Auto-generate email from first/last name
const watchFirstName = form.watch('firstName');
const watchLastName = form.watch('lastName');
React.useEffect(() => {
  if (watchFirstName && watchLastName) {
    const email = `${watchFirstName}.${watchLastName}@${organizationDomain}`;
    form.setValue('email', email.toLowerCase());
  }
}, [watchFirstName, watchLastName]);
```

## Key Insights

`★ Insight ─────────────────────────────────────`
**React Hook Form Watchers** - `form.watch()` creates reactive subscriptions that trigger `useEffect` on every field change. This enables cascading transformations without manual event handlers.

**Edit Mode Guards** - Always check `!isEditMode` before auto-generating to prevent overwriting existing database records. Database constraints (unique indexes) provide backup protection.

**Transformation Chains** - Multiple `useEffect` hooks can cascade (name → slug → subdomain). React batches state updates, so intermediate values don't cause extra re-renders.
`─────────────────────────────────────────────────`

## References

- **React Hook Form Documentation:** https://react-hook-form.com/api/useform/watch
- **Zod Validation:** https://zod.dev/
- **Shadcn/ui Form Components:** https://ui.shadcn.com/docs/components/form
- **URL Slug Best Practices:** https://moz.com/learn/seo/url

---

**Pattern Status:** ✅ Active
**Production Usage:** WellOS Admin Portal (Tenant Creation)
**Last Validated:** October 25, 2025
