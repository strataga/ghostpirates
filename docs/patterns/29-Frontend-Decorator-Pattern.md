# Frontend Decorator Pattern

## Overview

The Decorator Pattern in React applications allows you to add behavior to
components dynamically without altering their structure. This pattern is
particularly useful for adding cross-cutting concerns like analytics,
permissions, error handling, and logging to components in a reusable and
composable way.

## Problem Statement

React applications often need to add common functionality across multiple
components:

- **Analytics tracking** for user interactions
- **Permission checks** before rendering sensitive components
- **Error boundaries** for graceful error handling
- **Loading states** and skeleton screens
- **Audit logging** for compliance requirements
- **Performance monitoring** and metrics collection

Adding these concerns directly to components leads to:

- **Code duplication** across similar components
- **Mixed responsibilities** within components
- **Difficult maintenance** when requirements change
- **Testing complexity** due to tightly coupled concerns

## Solution

Implement the Decorator Pattern using Higher-Order Components (HOCs) and React
hooks to add cross-cutting concerns in a composable, reusable way.

## Implementation

### Base Decorator Types

```typescript
// lib/decorators/types.ts
export type ComponentDecorator<P = {}> = <T extends ComponentType<P>>(
  Component: T,
) => ComponentType<P>;

export interface DecoratorConfig {
  displayName?: string;
  skipProps?: string[];
  forwardRef?: boolean;
}

export interface AnalyticsConfig {
  eventName: string;
  properties?: Record<string, any>;
  trackOnMount?: boolean;
  trackOnUnmount?: boolean;
  trackOnClick?: boolean;
}

export interface PermissionConfig {
  action: string;
  subject?: string;
  fallback?: ComponentType;
  redirectTo?: string;
}

export interface ErrorBoundaryConfig {
  fallback?: ComponentType<{ error: Error; resetError: () => void }>;
  onError?: (error: Error, errorInfo: ErrorInfo) => void;
  resetOnPropsChange?: boolean;
}
```

### Analytics Decorator

```typescript
// lib/decorators/with-analytics.tsx
export function withAnalytics<P extends object>(
  config: AnalyticsConfig
): ComponentDecorator<P> {
  return function AnalyticsDecorator<T extends ComponentType<P>>(Component: T) {
    const WrappedComponent = (props: P) => {
      const analytics = useAnalytics();
      const componentName = Component.displayName || Component.name || 'Component';

      // Track component mount
      useEffect(() => {
        if (config.trackOnMount) {
          analytics.track(config.eventName, {
            component: componentName,
            action: 'mount',
            ...config.properties,
          });
        }

        // Track component unmount
        return () => {
          if (config.trackOnUnmount) {
            analytics.track(config.eventName, {
              component: componentName,
              action: 'unmount',
              ...config.properties,
            });
          }
        };
      }, [analytics, componentName]);

      // Add click tracking if enabled
      const enhancedProps = config.trackOnClick
        ? {
            ...props,
            onClick: (event: MouseEvent) => {
              analytics.track(config.eventName, {
                component: componentName,
                action: 'click',
                ...config.properties,
              });

              // Call original onClick if it exists
              if ('onClick' in props && typeof props.onClick === 'function') {
                (props.onClick as Function)(event);
              }
            },
          }
        : props;

      return <Component {...enhancedProps} />;
    };

    WrappedComponent.displayName = `withAnalytics(${Component.displayName || Component.name})`;
    return WrappedComponent as ComponentType<P>;
  };
}

// Usage examples
const UserListWithAnalytics = withAnalytics({
  eventName: 'user_list_viewed',
  trackOnMount: true,
  properties: { section: 'user_management' },
})(UserList);

const InviteButtonWithAnalytics = withAnalytics({
  eventName: 'invite_user_clicked',
  trackOnClick: true,
  properties: { source: 'user_list' },
})(Button);
```

### Permission Decorator

```typescript
// lib/decorators/with-permissions.tsx
export function withPermissions<P extends object>(
  config: PermissionConfig
): ComponentDecorator<P> {
  return function PermissionDecorator<T extends ComponentType<P>>(Component: T) {
    const WrappedComponent = (props: P) => {
      const { can } = useAbilities();
      const router = useRouter();

      const hasPermission = can(config.action, config.subject);

      useEffect(() => {
        if (!hasPermission && config.redirectTo) {
          router.push(config.redirectTo);
        }
      }, [hasPermission, router]);

      if (!hasPermission) {
        if (config.fallback) {
          const FallbackComponent = config.fallback;
          return <FallbackComponent />;
        }

        return (
          <div className="flex items-center justify-center p-8">
            <div className="text-center">
              <Lock className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="text-lg font-semibold mb-2">Access Denied</h3>
              <p className="text-muted-foreground">
                You don't have permission to view this content.
              </p>
            </div>
          </div>
        );
      }

      return <Component {...props} />;
    };

    WrappedComponent.displayName = `withPermissions(${Component.displayName || Component.name})`;
    return WrappedComponent as ComponentType<P>;
  };
}

// Usage examples
const AdminUserList = withPermissions({
  action: 'read',
  subject: 'User',
  fallback: UnauthorizedMessage,
})(UserList);

const DeleteUserButton = withPermissions({
  action: 'delete',
  subject: 'User',
})(Button);
```

### Error Boundary Decorator

```typescript
// lib/decorators/with-error-boundary.tsx
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
}

export function withErrorBoundary<P extends object>(
  config: ErrorBoundaryConfig = {}
): ComponentDecorator<P> {
  return function ErrorBoundaryDecorator<T extends ComponentType<P>>(Component: T) {
    class ErrorBoundaryWrapper extends React.Component<P, ErrorBoundaryState> {
      constructor(props: P) {
        super(props);
        this.state = { hasError: false, error: null };
      }

      static getDerivedStateFromError(error: Error): ErrorBoundaryState {
        return { hasError: true, error };
      }

      componentDidCatch(error: Error, errorInfo: ErrorInfo) {
        console.error('Error caught by boundary:', error, errorInfo);

        if (config.onError) {
          config.onError(error, errorInfo);
        }

        // Send to error reporting service
        if (typeof window !== 'undefined' && window.Sentry) {
          window.Sentry.captureException(error, {
            contexts: { errorInfo },
            tags: { component: Component.displayName || Component.name },
          });
        }
      }

      componentDidUpdate(prevProps: P) {
        if (config.resetOnPropsChange && this.state.hasError) {
          // Reset error state if props changed
          const propsChanged = Object.keys(this.props).some(
            key => (this.props as any)[key] !== (prevProps as any)[key]
          );

          if (propsChanged) {
            this.setState({ hasError: false, error: null });
          }
        }
      }

      resetError = () => {
        this.setState({ hasError: false, error: null });
      };

      render() {
        if (this.state.hasError) {
          if (config.fallback) {
            const FallbackComponent = config.fallback;
            return (
              <FallbackComponent
                error={this.state.error!}
                resetError={this.resetError}
              />
            );
          }

          return (
            <div className="flex items-center justify-center p-8">
              <div className="text-center">
                <AlertTriangle className="h-12 w-12 text-destructive mx-auto mb-4" />
                <h3 className="text-lg font-semibold mb-2">Something went wrong</h3>
                <p className="text-muted-foreground mb-4">
                  {this.state.error?.message || 'An unexpected error occurred'}
                </p>
                <Button onClick={this.resetError} variant="outline">
                  Try Again
                </Button>
              </div>
            </div>
          );
        }

        return <Component {...this.props} />;
      }
    }

    ErrorBoundaryWrapper.displayName = `withErrorBoundary(${Component.displayName || Component.name})`;
    return ErrorBoundaryWrapper as ComponentType<P>;
  };
}

// Custom error fallback component
const CustomErrorFallback = ({ error, resetError }: { error: Error; resetError: () => void }) => (
  <div className="border border-destructive rounded-lg p-6 m-4">
    <h2 className="text-lg font-semibold text-destructive mb-2">Component Error</h2>
    <p className="text-sm text-muted-foreground mb-4">{error.message}</p>
    <Button onClick={resetError} size="sm">Reset Component</Button>
  </div>
);

// Usage
const SafeUserList = withErrorBoundary({
  fallback: CustomErrorFallback,
  resetOnPropsChange: true,
})(UserList);
```

### Loading Decorator

```typescript
// lib/decorators/with-loading.tsx
export interface LoadingConfig {
  skeleton?: ComponentType;
  minLoadingTime?: number;
  loadingText?: string;
}

export function withLoading<P extends { loading?: boolean }>(
  config: LoadingConfig = {}
): ComponentDecorator<P> {
  return function LoadingDecorator<T extends ComponentType<P>>(Component: T) {
    const WrappedComponent = (props: P) => {
      const [showLoading, setShowLoading] = useState(props.loading || false);
      const [minTimeElapsed, setMinTimeElapsed] = useState(false);

      useEffect(() => {
        if (props.loading && config.minLoadingTime) {
          setMinTimeElapsed(false);
          const timer = setTimeout(() => {
            setMinTimeElapsed(true);
          }, config.minLoadingTime);

          return () => clearTimeout(timer);
        } else {
          setMinTimeElapsed(true);
        }
      }, [props.loading]);

      useEffect(() => {
        if (config.minLoadingTime) {
          setShowLoading(props.loading || false);
        } else {
          setShowLoading(props.loading || false);
        }
      }, [props.loading, minTimeElapsed]);

      if (showLoading && (!config.minLoadingTime || !minTimeElapsed)) {
        if (config.skeleton) {
          const SkeletonComponent = config.skeleton;
          return <SkeletonComponent />;
        }

        return (
          <div className="flex items-center justify-center p-8">
            <div className="flex items-center space-x-2">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span>{config.loadingText || 'Loading...'}</span>
            </div>
          </div>
        );
      }

      return <Component {...props} />;
    };

    WrappedComponent.displayName = `withLoading(${Component.displayName || Component.name})`;
    return WrappedComponent as ComponentType<P>;
  };
}

// Custom skeleton component
const UserListSkeleton = () => (
  <div className="space-y-4">
    {Array.from({ length: 5 }).map((_, i) => (
      <div key={i} className="flex items-center space-x-4">
        <Skeleton className="h-12 w-12 rounded-full" />
        <div className="space-y-2">
          <Skeleton className="h-4 w-[200px]" />
          <Skeleton className="h-4 w-[150px]" />
        </div>
      </div>
    ))}
  </div>
);

// Usage
const LoadingUserList = withLoading({
  skeleton: UserListSkeleton,
  minLoadingTime: 500,
  loadingText: 'Loading users...',
})(UserList);
```

### Audit Logging Decorator

```typescript
// lib/decorators/with-audit-log.tsx
export interface AuditConfig {
  action: string;
  resource?: string;
  logOnMount?: boolean;
  logOnUnmount?: boolean;
  logOnInteraction?: boolean;
  sensitiveProps?: string[];
}

export function withAuditLog<P extends object>(
  config: AuditConfig
): ComponentDecorator<P> {
  return function AuditLogDecorator<T extends ComponentType<P>>(Component: T) {
    const WrappedComponent = (props: P) => {
      const { user } = useAuth();
      const auditLogger = useAuditLogger();

      const logAuditEvent = useCallback((action: string, details?: any) => {
        auditLogger.log({
          userId: user?.id,
          action,
          resource: config.resource,
          component: Component.displayName || Component.name,
          timestamp: new Date(),
          details: details || {},
          ipAddress: window.location.hostname,
          userAgent: navigator.userAgent,
        });
      }, [user, auditLogger]);

      useEffect(() => {
        if (config.logOnMount) {
          const sanitizedProps = config.sensitiveProps
            ? Object.fromEntries(
                Object.entries(props).filter(([key]) => !config.sensitiveProps!.includes(key))
              )
            : props;

          logAuditEvent(`${config.action}_viewed`, { props: sanitizedProps });
        }

        return () => {
          if (config.logOnUnmount) {
            logAuditEvent(`${config.action}_closed`);
          }
        };
      }, [logAuditEvent]);

      const enhancedProps = config.logOnInteraction
        ? {
            ...props,
            onClick: (event: MouseEvent) => {
              logAuditEvent(`${config.action}_clicked`);

              if ('onClick' in props && typeof props.onClick === 'function') {
                (props.onClick as Function)(event);
              }
            },
          }
        : props;

      return <Component {...enhancedProps} />;
    };

    WrappedComponent.displayName = `withAuditLog(${Component.displayName || Component.name})`;
    return WrappedComponent as ComponentType<P>;
  };
}

// Usage
const AuditedUserList = withAuditLog({
  action: 'user_management',
  resource: 'users',
  logOnMount: true,
  sensitiveProps: ['password', 'token'],
})(UserList);
```

### Decorator Composition

```typescript
// lib/decorators/compose-decorators.ts
export function composeDecorators<P extends object>(
  ...decorators: ComponentDecorator<P>[]
): ComponentDecorator<P> {
  return function ComposedDecorator<T extends ComponentType<P>>(Component: T) {
    return decorators.reduceRight(
      (acc, decorator) => decorator(acc),
      Component,
    ) as ComponentType<P>;
  };
}

// Usage - compose multiple decorators
const EnhancedUserList = composeDecorators(
  withErrorBoundary({ resetOnPropsChange: true }),
  withPermissions({ action: 'read', subject: 'User' }),
  withAnalytics({ eventName: 'user_list_viewed', trackOnMount: true }),
  withAuditLog({ action: 'user_management', logOnMount: true }),
  withLoading({ minLoadingTime: 300 }),
)(UserList);

// Alternative fluent API
const FluentUserList = withErrorBoundary({ resetOnPropsChange: true })(
  withPermissions({ action: 'read', subject: 'User' })(
    withAnalytics({ eventName: 'user_list_viewed', trackOnMount: true })(
      withAuditLog({ action: 'user_management', logOnMount: true })(
        withLoading({ minLoadingTime: 300 })(UserList),
      ),
    ),
  ),
);
```

### Hook-Based Decorators

```typescript
// lib/decorators/use-decorators.ts
export function useDecorators<P extends object>(
  Component: ComponentType<P>,
  decorators: ComponentDecorator<P>[]
): ComponentType<P> {
  return useMemo(() => {
    return decorators.reduce(
      (acc, decorator) => decorator(acc),
      Component
    );
  }, [Component, decorators]);
}

// Usage in components
export function UserManagementPage() {
  const DecoratedUserList = useDecorators(UserList, [
    withErrorBoundary(),
    withPermissions({ action: 'read', subject: 'User' }),
    withAnalytics({ eventName: 'user_list_viewed', trackOnMount: true }),
  ]);

  return (
    <div>
      <h1>User Management</h1>
      <DecoratedUserList users={users} />
    </div>
  );
}
```

## Benefits

### 1. **Separation of Concerns**

- Cross-cutting concerns separated from business logic
- Components focus on their primary responsibility
- Reusable decorators across different components

### 2. **Composability**

- Mix and match decorators as needed
- Easy to add or remove functionality
- Clear dependency chain

### 3. **Testability**

- Test decorators independently
- Test components without decorators
- Mock decorator behavior easily

### 4. **Maintainability**

- Centralized implementation of common patterns
- Easy to update behavior across all components
- Clear separation of concerns

## Best Practices

### 1. **Decorator Naming**

```typescript
// ✅ Good: Clear, descriptive names
withAnalytics();
withPermissions();
withErrorBoundary();

// ❌ Bad: Generic or unclear names
enhance();
wrap();
decorate();
```

### 2. **Configuration**

```typescript
// ✅ Good: Flexible configuration
withAnalytics({
  eventName: 'user_action',
  trackOnMount: true,
  properties: { section: 'users' },
});

// ❌ Bad: Hardcoded behavior
withAnalytics('user_action');
```

### 3. **Error Handling**

```typescript
// ✅ Good: Graceful degradation
const WrappedComponent = (props: P) => {
  try {
    return <Component {...props} />;
  } catch (error) {
    console.error('Decorator error:', error);
    return <Component {...props} />; // Fallback to original
  }
};
```

## Testing

```typescript
// __tests__/decorators/with-analytics.test.tsx
describe('withAnalytics', () => {
  const mockAnalytics = {
    track: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    (useAnalytics as jest.Mock).mockReturnValue(mockAnalytics);
  });

  it('should track component mount', () => {
    const TestComponent = () => <div>Test</div>;
    const AnalyticsComponent = withAnalytics({
      eventName: 'test_event',
      trackOnMount: true,
    })(TestComponent);

    render(<AnalyticsComponent />);

    expect(mockAnalytics.track).toHaveBeenCalledWith('test_event', {
      component: 'TestComponent',
      action: 'mount',
    });
  });

  it('should track clicks when enabled', () => {
    const TestComponent = ({ onClick }: { onClick?: () => void }) => (
      <button onClick={onClick}>Click me</button>
    );

    const AnalyticsComponent = withAnalytics({
      eventName: 'test_click',
      trackOnClick: true,
    })(TestComponent);

    render(<AnalyticsComponent />);
    fireEvent.click(screen.getByText('Click me'));

    expect(mockAnalytics.track).toHaveBeenCalledWith('test_click', {
      component: 'TestComponent',
      action: 'click',
    });
  });
});
```

The Decorator Pattern provides a clean, composable way to add cross-cutting
concerns to React components while maintaining separation of concerns and
reusability.
