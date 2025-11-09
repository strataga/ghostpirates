# User-Friendly Error Handling Pattern

**Pattern Category**: Frontend Architecture
**Complexity**: Medium
**Last Updated**: October 10, 2025

## Overview

The User-Friendly Error Handling pattern transforms technical API errors into user-friendly messages that provide actionable guidance. This pattern prevents exposing technical details (HTTP status codes, PostgreSQL errors, stack traces) to end users while maintaining error information for debugging.

## Problem

Raw API errors expose technical implementation details to users:

- "Request failed with status code 500" - Meaningless to users
- "Request failed with status code 401" - HTTP jargon
- "PostgresError: relation 'pending_users' does not exist" - Database internals leaked
- "Network Error" - Too vague to be helpful

**Consequences:**

- Poor user experience and confusion
- Unprofessional appearance
- Increased support requests
- Security information disclosure
- Users unable to self-recover from errors

## Solution

Implement a multi-layered error handling architecture that transforms errors at the repository boundary:

```
┌─────────────────────────────────────────────────────┐
│                   UI Component                       │
│  Displays: "Invalid email or password"              │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│              Error Transformer                       │
│  getUserErrorMessage(err) → User-friendly message   │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│                Repository Layer                      │
│  Catches axios errors, throws ApiError              │
└───────────────────┬─────────────────────────────────┘
                    │
┌───────────────────▼─────────────────────────────────┐
│                 API Client                           │
│  Axios error: { status: 401, message: "..." }       │
└─────────────────────────────────────────────────────┘
```

## Implementation

### 1. Custom Error Classes

Create typed error classes that extend native Error:

```typescript
// apps/web/lib/errors/api-error.ts

export class ApiError extends Error {
  constructor(
    message: string,
    public readonly statusCode?: number,
    public readonly originalError?: unknown,
    public readonly details?: Record<string, unknown>,
  ) {
    super(message);
    this.name = 'ApiError';
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, ApiError);
    }
  }
}

export class AuthenticationError extends ApiError {
  constructor(message: string, originalError?: unknown) {
    super(message, 401, originalError);
    this.name = 'AuthenticationError';
  }
}

export class ValidationError extends ApiError {
  constructor(
    message: string,
    originalError?: unknown,
    public readonly validationErrors?: Record<string, string[]>,
  ) {
    super(message, 400, originalError, { validationErrors });
    this.name = 'ValidationError';
  }
}
```

### 2. Error Message Mapping

Define context-specific error messages:

```typescript
// apps/web/lib/errors/error-messages.ts

const AUTH_ERROR_MESSAGES: Record<string, Record<number, string>> = {
  login: {
    401: 'Invalid email or password. Please try again.',
    429: 'Too many login attempts. Please wait a moment and try again.',
    500: 'Unable to sign in at this time. Please try again later.',
  },
  register: {
    400: 'Invalid registration information. Please check your input.',
    409: 'An account with this email already exists. Please sign in or use a different email.',
    500: 'Unable to create account at this time. Please try again later.',
  },
};

export function transformAxiosError(error: unknown, context?: string): ApiError {
  if (!isAxiosError(error)) {
    return new ApiError('An unexpected error occurred. Please try again.');
  }

  const statusCode = error.response?.status;
  const operationContext = context || getOperationContext(error.config?.url);

  // Priority: contextual message > API message > default message
  const userMessage =
    getContextualMessage(operationContext, statusCode) ||
    extractApiErrorMessage(error) ||
    DEFAULT_ERROR_MESSAGES[statusCode] ||
    'An error occurred. Please try again.';

  // Create appropriate error type
  switch (statusCode) {
    case 401:
      return new AuthenticationError(userMessage, error);
    case 400:
    case 422:
      return new ValidationError(userMessage, error);
    // ... other cases
    default:
      return new ApiError(userMessage, statusCode, error);
  }
}
```

### 3. Repository Layer Integration

Transform errors at the repository boundary:

```typescript
// apps/web/lib/repositories/auth.repository.ts

import { transformAxiosError } from '@/lib/errors';

export class AuthRepository {
  async login(data: LoginRequest): Promise<LoginResponse> {
    try {
      const response = await apiClient.post<LoginResponse>('/auth/login', data);
      return response.data;
    } catch (error) {
      // Transform axios error to user-friendly API error
      throw transformAxiosError(error, 'login');
    }
  }

  async register(data: RegisterRequest): Promise<User> {
    try {
      const response = await apiClient.post<User>('/auth/register', data);
      return response.data;
    } catch (error) {
      throw transformAxiosError(error, 'register');
    }
  }
}
```

### 4. Component Usage

Components display error messages safely:

```typescript
// apps/web/components/auth/login-form.tsx

import { getUserErrorMessage } from '@/lib/errors';

export function LoginForm() {
  const [error, setError] = useState<string | null>(null);

  const onSubmit = async (data: LoginFormData) => {
    try {
      await login(data);
    } catch (err: unknown) {
      // Get user-friendly message (already transformed by repository)
      const errorMessage = getUserErrorMessage(
        err,
        'Unable to sign in. Please try again.'
      );
      setError(errorMessage);
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      {error && (
        <div className="rounded-md bg-destructive/15 px-4 py-3 text-sm text-destructive">
          {error}
        </div>
      )}
      {/* form fields */}
    </form>
  );
}
```

## Benefits

✅ **User Experience**

- Clear, actionable error messages
- No technical jargon or status codes
- Contextual guidance for recovery

✅ **Security**

- No exposure of internal implementation details
- No database schema information leaked
- Stack traces kept server-side only

✅ **Maintainability**

- Centralized error message definitions
- Consistent error handling across application
- Easy to update messages without changing components

✅ **Developer Experience**

- Type-safe error classes with instanceof checks
- Original error preserved for debugging
- Clear error transformation flow

## Anti-Patterns

❌ **Don't expose raw errors to users:**

```typescript
// BAD
catch (err) {
  setError(err.message); // Shows "Request failed with status code 500"
}
```

❌ **Don't ignore error context:**

```typescript
// BAD - Generic message for all errors
return 'Something went wrong';
```

❌ **Don't lose error information:**

```typescript
// BAD - Original error discarded, can't debug
throw new Error('Login failed');
```

❌ **Don't transform errors in multiple places:**

```typescript
// BAD - Inconsistent transformations
// Some in components, some in mutations, some in API client
```

## Best Practices

### 1. Provide Actionable Guidance

```typescript
// ✅ GOOD - Tells user what to do
'Invalid email or password. Please try again.';
'Connection lost. Please check your internet connection and try again.';

// ❌ BAD - No guidance
'Invalid credentials';
'Network error';
```

### 2. Context-Specific Messages

```typescript
// ✅ GOOD - Different messages for different operations
login: { 401: 'Invalid email or password. Please try again.' }
register: { 409: 'An account with this email already exists.' }

// ❌ BAD - Generic message for all operations
{ 401: 'Unauthorized' }
```

### 3. Preserve Original Error

```typescript
// ✅ GOOD - Original error available for debugging
new ApiError(userMessage, statusCode, originalError);

// ❌ BAD - Original error lost
throw new Error(userMessage);
```

### 4. Transform at Repository Boundary

```typescript
// ✅ GOOD - Single transformation point
async register(data) {
  try {
    return await apiClient.post('/auth/register', data);
  } catch (error) {
    throw transformAxiosError(error, 'register');
  }
}

// ❌ BAD - Transform in every component
```

## Testing

### Unit Tests

```typescript
describe('transformAxiosError', () => {
  it('should transform 401 login error to user-friendly message', () => {
    const axiosError = createAxiosError(401, '/auth/login');
    const result = transformAxiosError(axiosError, 'login');

    expect(result).toBeInstanceOf(AuthenticationError);
    expect(result.message).toBe('Invalid email or password. Please try again.');
    expect(result.statusCode).toBe(401);
  });

  it('should preserve original error for debugging', () => {
    const axiosError = createAxiosError(500);
    const result = transformAxiosError(axiosError);

    expect(result.originalError).toBe(axiosError);
  });
});
```

### Integration Tests

```typescript
it('should show user-friendly error on failed login', async () => {
  server.use(
    http.post('/auth/login', () => {
      return HttpResponse.json(
        { message: 'Invalid credentials' },
        { status: 401 }
      );
    })
  );

  render(<LoginForm />);

  // Fill form and submit
  await userEvent.type(screen.getByLabelText(/email/i), 'test@example.com');
  await userEvent.type(screen.getByLabelText(/password/i), 'wrong');
  await userEvent.click(screen.getByRole('button', { name: /sign in/i }));

  // Should show user-friendly message, not "Request failed with status code 401"
  expect(await screen.findByText(/invalid email or password/i)).toBeInTheDocument();
  expect(screen.queryByText(/status code 401/i)).not.toBeInTheDocument();
});
```

## Related Patterns

- **Repository Pattern** - Error transformation happens at repository boundary
- **Adapter Pattern** - Adapts technical errors to user-facing messages
- **Strategy Pattern** - Different error strategies for different contexts
- **Error Boundaries** (React) - Catch and handle unexpected errors in UI

## When to Use

✅ Use this pattern when:

- Building user-facing applications
- Integrating with external APIs
- Need to prevent information disclosure
- Want consistent error messaging
- Supporting non-technical users

❌ Don't use when:

- Building internal developer tools (technical errors may be preferred)
- API responses already return user-friendly messages
- Error context is highly dynamic and unpredictable

## Migration Strategy

If you have existing error handling:

1. **Create error utilities** (api-error.ts, error-messages.ts)
2. **Update one repository at a time** - Start with auth repository
3. **Update components using that repository** - Import getUserErrorMessage
4. **Test thoroughly** - Ensure all error paths show friendly messages
5. **Repeat for other repositories** - Time entries, clients, invoices, etc.
6. **Monitor production** - Watch for any errors that slip through

## File Structure

```
apps/web/lib/errors/
├── index.ts                  # Public exports
├── api-error.ts              # Custom error classes
└── error-messages.ts         # Message mapping and transformation

apps/web/lib/repositories/
└── *.repository.ts           # Repositories use transformAxiosError()

apps/web/components/
└── */                        # Components use getUserErrorMessage()
```

## Real-World Example

**Before (Test Report Issue #4):**

```
User sees: "Request failed with status code 500"
Console shows: "PostgresError: relation 'pending_users' does not exist"
```

**After (With Pattern Applied):**

```
User sees: "Unable to create account at this time. Please try again later."
Console shows: ApiError with original PostgresError preserved for debugging
Sentry/logging: Full error details with stack trace
```

## References

- Test Report 2025-10-10: High Priority Issue #4 - Poor Error Messages
- apps/web/lib/errors/api-error.ts:1 (implementation)
- apps/web/lib/errors/error-messages.ts:1 (message mapping)
- apps/web/lib/repositories/auth.repository.ts:10 (usage example)

---

**Status**: ✅ Implemented
**Used In**: Auth flows (login, register, password reset)
**Next Steps**: Extend to all repositories (clients, invoices, time entries, projects)
