# JWT Authentication & Session Management Pattern

**Pattern Type:** Security & Authentication
**Complexity:** Intermediate
**Last Updated:** October 2025

## Overview

The JWT Authentication & Session Management pattern provides a secure, scalable approach to user authentication using dual-token strategy with in-memory access tokens and httpOnly refresh tokens. This pattern balances security (XSS protection), user experience (persistent sessions), and reliability (avoiding race conditions).

## Problem Statement

Modern web applications need to:

- **Protect against XSS attacks** by avoiding localStorage for sensitive tokens
- **Maintain persistent sessions** across page refreshes and browser restarts
- **Handle token expiration gracefully** without disrupting user experience
- **Prevent race conditions** during token refresh and authentication state checks
- **Support proper cookie configuration** across different environments (development/production)

Common pitfalls:

- Storing JWT in localStorage (vulnerable to XSS)
- Aggressive refetching causing logout race conditions
- Cookie configuration mismatches preventing proper clearing
- Token expiry mismatches between client and server
- Loss of session on page refresh due to in-memory storage

## Solution

### Architecture

```
┌─────────────────────────────────────────────────────┐
│                    Frontend                          │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │         Access Token (In-Memory)         │      │
│  │  - Short-lived (7 days)                  │      │
│  │  - Lost on page refresh                  │      │
│  │  - Included in Authorization header      │      │
│  └──────────────────────────────────────────┘      │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │    Refresh Token (httpOnly Cookie)       │      │
│  │  - Long-lived (30 days)                  │      │
│  │  - Persists across page refreshes        │      │
│  │  - httpOnly (XSS protected)              │      │
│  │  - Secure in production                  │      │
│  └──────────────────────────────────────────┘      │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │          React Query Config              │      │
│  │  - staleTime: 30 minutes                 │      │
│  │  - refetchOnWindowFocus: false           │      │
│  │  - refetchOnReconnect: false             │      │
│  └──────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────┘
                        │
                        │ API Requests
                        ▼
┌─────────────────────────────────────────────────────┐
│                    Backend (API)                     │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │         JWT Strategy (Passport)          │      │
│  │  - Validates access token                │      │
│  │  - Checks signature & expiry             │      │
│  │  - Extracts user payload                 │      │
│  └──────────────────────────────────────────┘      │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │      Refresh Token Repository            │      │
│  │  - Database persistence                  │      │
│  │  - Validates refresh tokens              │      │
│  │  - Manages expiry (30 days)              │      │
│  └──────────────────────────────────────────┘      │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │          Cookie Configuration            │      │
│  │  - maxAge: 30 days (match DB)            │      │
│  │  - path: '/' (all routes)                │      │
│  │  - domain: 'localhost' (dev only)        │      │
│  │  - sameSite: 'lax' (dev) / 'strict' (prod)│     │
│  └──────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────┘
```

### Key Components

#### 1. Access Token (In-Memory)

```typescript
// Frontend: lib/api/client.ts
let accessToken: string | null = null;

export const setAccessToken = (token: string | null): void => {
  accessToken = token;
};

export const getAccessToken = (): string | null => {
  return accessToken;
};

export const clearAccessToken = (): void => {
  accessToken = null;
};
```

**Why in-memory?**

- ✅ Protected from XSS attacks (can't be stolen via malicious scripts)
- ✅ Automatic cleanup on page close
- ❌ Lost on page refresh (acceptable with refresh token mechanism)

#### 2. Refresh Token (httpOnly Cookie)

```typescript
// Backend: presentation/auth/auth.controller.ts
private setRefreshTokenCookie(res: Response, refreshToken: string): void {
  const isProduction = process.env.NODE_ENV === 'production';
  const maxAge = 30 * 24 * 60 * 60 * 1000; // 30 days (MUST match DB expiry)

  res.cookie('refreshToken', refreshToken, {
    httpOnly: true,                        // Prevents JavaScript access
    secure: isProduction,                  // HTTPS only in production
    sameSite: isProduction ? 'strict' : 'lax', // CSRF protection
    maxAge,                                // Cookie expiry
    path: '/',                             // Available on all routes
    domain: isProduction ? undefined : 'localhost', // Cross-port in dev
  });
}
```

**Critical: Cookie Clearing Must Match Setting**

```typescript
private clearRefreshTokenCookie(res: Response): void {
  const isProduction = process.env.NODE_ENV === 'production';
  res.clearCookie('refreshToken', {
    httpOnly: true,
    secure: isProduction,
    sameSite: isProduction ? 'strict' : 'lax',
    path: '/',                             // MUST match setting
    domain: isProduction ? undefined : 'localhost', // MUST match setting
  });
}
```

#### 3. Token Refresh Interceptor

```typescript
// Frontend: lib/api/client.ts
apiClient.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean };

    // If error is 401 and we haven't retried yet
    if (error.response?.status === 401 && !originalRequest._retry) {
      // Don't retry if it's the refresh endpoint itself
      if (originalRequest.url?.includes('/auth/refresh')) {
        clearAccessToken();
        return Promise.reject(error);
      }

      if (isRefreshing) {
        // Queue this request to retry after refresh completes
        return new Promise((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        }).then(() => apiClient(originalRequest));
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        // Refresh using httpOnly cookie
        const response = await axios.post<RefreshTokenResponse>(
          `${API_URL}/auth/refresh`,
          {},
          { withCredentials: true },
        );

        const newAccessToken = response.data.accessToken;
        setAccessToken(newAccessToken);

        // Update original request with new token
        originalRequest.headers.Authorization = `Bearer ${newAccessToken}`;

        // Process queued requests
        processQueue(null, newAccessToken);

        return apiClient(originalRequest);
      } catch (refreshError) {
        processQueue(refreshError as Error, null);
        clearAccessToken();

        // Redirect to login
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }
        return Promise.reject(refreshError);
      } finally {
        isRefreshing = false;
      }
    }

    return Promise.reject(error);
  },
);
```

#### 4. React Query Configuration (Anti-Race Condition)

```typescript
// Frontend: hooks/auth/use-auth-query.ts
export function useAuthQuery() {
  return useQuery<User, Error>({
    queryKey: authQueryKeys.currentUser(),
    queryFn: async () => {
      const token = getAccessToken();
      if (!token) {
        throw new Error('No access token available');
      }
      return authRepository.getCurrentUser();
    },
    enabled: true,
    retry: (failureCount, error) => {
      if (error.message === 'No access token available') {
        return false; // Don't retry if no token
      }
      return failureCount < 1;
    },
    // CRITICAL: Prevent race conditions
    staleTime: 30 * 60 * 1000, // 30 minutes (data stays fresh)
    refetchOnWindowFocus: false, // Don't refetch on tab switch
    refetchOnReconnect: false, // Don't refetch on network reconnect
  });
}
```

#### 5. Auth Initialization Flow

```typescript
// Frontend: lib/auth/auth-context.tsx
useEffect(() => {
  if (isInitialized) return;

  const initializeAuth = async () => {
    try {
      setLoading(true);

      // Check if we already have an access token (recent login)
      const existingToken = getAccessToken();

      if (!existingToken) {
        // No access token, try to refresh using httpOnly cookie
        const { accessToken: newToken } = await refreshTokenMutation.mutateAsync();
        setAccessToken(newToken);

        // Invalidate auth query to trigger refetch with new token
        await queryClient.invalidateQueries({ queryKey: authQueryKeys.currentUser() });
      }
    } catch {
      // No valid session, clear everything
      clearAccessToken();
      queryClient.removeQueries({ queryKey: authQueryKeys.all });
    } finally {
      setLoading(false);
      setInitialized(true);
    }
  };

  initializeAuth();
}, [isInitialized]);
```

## Implementation Checklist

### Backend Setup

- [ ] **JWT Module Configuration**
  - [ ] Access token expiry configured (e.g., `JWT_EXPIRES_IN=7d`)
  - [ ] Refresh token expiry configured (e.g., `JWT_REFRESH_EXPIRES_IN=30d`)
  - [ ] JWT secret is strong and unique (`JWT_SECRET`)
  - [ ] Refresh secret is different from JWT secret (`JWT_REFRESH_SECRET`)

- [ ] **Cookie Configuration**
  - [ ] `maxAge` matches database refresh token expiry
  - [ ] `path: '/'` set explicitly
  - [ ] `domain` configured for development (e.g., `'localhost'`)
  - [ ] `sameSite` appropriate for environment (`'lax'` dev, `'strict'` prod)
  - [ ] `httpOnly: true` for security
  - [ ] `secure: true` in production

- [ ] **Clear Cookie Configuration**
  - [ ] ALL properties match cookie setting (domain, path, sameSite, secure, httpOnly)

- [ ] **Refresh Token Repository**
  - [ ] Database persistence with expiry tracking
  - [ ] Cleanup of expired tokens (scheduled job)
  - [ ] Validation of token on refresh

### Frontend Setup

- [ ] **Token Storage**
  - [ ] Access token in memory only (NOT localStorage)
  - [ ] Refresh token in httpOnly cookie (automatic)
  - [ ] Clear token functions implemented

- [ ] **API Client**
  - [ ] Authorization header interceptor
  - [ ] 401 response interceptor with refresh logic
  - [ ] Queue mechanism for simultaneous requests
  - [ ] Prevent infinite refresh loops

- [ ] **React Query Configuration**
  - [ ] `staleTime` set to prevent aggressive refetching (30+ minutes)
  - [ ] `refetchOnWindowFocus: false` to prevent race conditions
  - [ ] `refetchOnReconnect: false` to prevent race conditions
  - [ ] Proper retry logic (don't retry if no token)

- [ ] **Auth Context**
  - [ ] Initialize auth on mount
  - [ ] Attempt refresh if no access token
  - [ ] Handle initialization failure gracefully
  - [ ] Clear state on logout

- [ ] **Auth Guard**
  - [ ] Check `isLoading` before `isAuthenticated`
  - [ ] Show loading state during initialization
  - [ ] Redirect to login only after initialization completes

## Common Pitfalls & Solutions

### 1. Random Logouts on Tab Switch

**Problem:**

```typescript
// ❌ BAD: Refetches on every tab switch
refetchOnWindowFocus: true,
```

**Why it happens:**

- User switches tabs
- React Query refetches `/auth/me`
- Access token is temporarily unavailable during refresh
- Request fails → User logged out

**Solution:**

```typescript
// ✅ GOOD: Only refetch when explicitly needed
refetchOnWindowFocus: false,
refetchOnReconnect: false,
staleTime: 30 * 60 * 1000, // Keep data fresh for 30 minutes
```

### 2. Cookie Not Clearing Properly

**Problem:**

```typescript
// ❌ BAD: Missing properties
res.clearCookie('refreshToken', {
  httpOnly: true,
  secure: isProduction,
});
```

**Why it happens:**

- Browser matches cookies by ALL properties (domain, path, sameSite, etc.)
- If clearing options don't match setting options, cookie won't be cleared

**Solution:**

```typescript
// ✅ GOOD: Match ALL properties
res.clearCookie('refreshToken', {
  httpOnly: true,
  secure: isProduction,
  sameSite: isProduction ? 'strict' : 'lax',
  path: '/', // MUST match
  domain: isProduction ? undefined : 'localhost', // MUST match
});
```

### 3. Session Lost on Page Refresh

**Problem:**

```typescript
// ❌ BAD: No initialization logic
const { user } = useAuthQuery();
```

**Why it happens:**

- Access token is in memory only
- Page refresh clears memory
- No attempt to restore session from refresh token cookie

**Solution:**

```typescript
// ✅ GOOD: Initialize on mount
useEffect(() => {
  const existingToken = getAccessToken();

  if (!existingToken) {
    // Refresh using httpOnly cookie
    refreshTokenMutation.mutateAsync().then(({ accessToken }) => setAccessToken(accessToken));
  }
}, []);
```

### 4. Cookie/Token Expiry Mismatch

**Problem:**

```typescript
// ❌ BAD: Mismatched expiry times
// Backend cookie
const maxAge = 7 * 24 * 60 * 60 * 1000; // 7 days

// Backend database
const expiresAt = calculateExpiry(30); // 30 days
```

**Why it happens:**

- Cookie expires after 7 days (browser deletes it)
- Database token still valid for 30 days
- User can't refresh even though token is "valid"

**Solution:**

```typescript
// ✅ GOOD: Match both expiry times
const REFRESH_TOKEN_DAYS = 30;

// Cookie
const maxAge = REFRESH_TOKEN_DAYS * 24 * 60 * 60 * 1000;

// Database
const expiresAt = calculateExpiry(REFRESH_TOKEN_DAYS);
```

### 5. Cross-Port Cookie Issues (Development)

**Problem:**

```typescript
// ❌ BAD: No domain set
res.cookie('refreshToken', token, {
  httpOnly: true,
  // missing domain
});
```

**Why it happens:**

- Frontend runs on `localhost:3000`
- Backend runs on `localhost:3001`
- Without domain, cookie is port-specific

**Solution:**

```typescript
// ✅ GOOD: Set domain to localhost (no port)
res.cookie('refreshToken', token, {
  httpOnly: true,
  domain: isProduction ? undefined : 'localhost', // Works across ports
});
```

### 6. Cross-Origin Cookie Blocking (Subdomain Development)

**Problem:**

```typescript
// ❌ Cookies don't work across different origins
// API: localhost:4000
// Web: demo.localhost:4001
// Browser blocks cookies even with Domain=.localhost
```

**Why it happens:**

- API and web app are on **different origins** (`localhost:4000` vs `demo.localhost:4001`)
- Browser security prevents cross-origin cookies even with proper domain settings
- `Domain=.localhost` doesn't work when the setting origin is `localhost:4000`
- httpOnly cookies won't be sent with requests from `demo.localhost:4001`

**Impact:**

1. Login succeeds and sets `refreshToken` cookie
2. Cookie appears in response headers but browser doesn't store it
3. Subsequent requests to protected routes don't include cookie
4. Middleware redirects to login (sees no cookie)
5. **Result:** Login successful toast shows, but redirect to dashboard fails

**Solution (Development Only):**

```typescript
// apps/web/middleware.ts
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;

  // ... public route checks ...

  // DEVELOPMENT MODE: Skip cookie check entirely
  // In dev, API is at localhost:4000 and web is at demo.localhost:4001 (different origins)
  // Cookies set by API won't be accessible to web app due to browser security
  // We rely on client-side auth checks instead
  const isDevelopment = process.env.NODE_ENV !== 'production';
  const refreshToken = request.cookies.get('refreshToken');
  const isAuthenticated = isDevelopment ? true : !!refreshToken;

  // ... rest of middleware logic ...
}
```

**Production Solution:**

In production, both API and web are on the same parent domain:

```typescript
// Production deployment:
// API:  api.onwellos.com
// Web:  demo.onwellos.com
// Domain: .onwellos.com (works perfectly)

res.cookie('refreshToken', token, {
  httpOnly: true,
  secure: true,
  sameSite: 'strict',
  domain: '.onwellos.com', // Shared across subdomains
  path: '/',
});
```

**Key Insights:**

- **Development:** Cross-origin restrictions can't be bypassed - disable server-side auth checks, rely on client-side
- **Production:** Same-domain architecture allows proper cookie sharing
- **Security:** Client-side auth in dev is acceptable (local machine only)
- **Alternative:** Use same-origin proxy in development if strict testing needed

### 7. Rate Limiting Blocking Development

**Problem:**

```typescript
// ❌ Production rate limits too strict for testing
@Throttle({ default: { limit: 5, ttl: 900000 } }) // 5 requests per 15 min
async login(@Body() dto: LoginDto) { ... }
```

**Why it happens:**

- Brute-force protection limits login attempts
- Development requires frequent testing/retries
- Hit rate limit quickly during development/debugging
- Returns HTTP 429 Too Many Requests

**Impact:**

1. Test login flow multiple times
2. Hit 5-attempt limit within minutes
3. Locked out for 15 minutes
4. **Result:** "Request failed with status code 429" error

**Solution:**

```typescript
// apps/api/src/presentation/auth/auth.controller.ts
@Throttle({
  default: {
    limit: process.env.NODE_ENV === 'production' ? 5 : 100,
    ttl: 900000, // 15 minutes
  },
})
async login(
  @TenantContext() tenant: TenantContextDto | undefined,
  @Body() dto: LoginDto,
  @Res({ passthrough: true }) res: Response,
) { ... }
```

**Emergency Reset:**

```bash
# Clear Redis throttle cache to immediately reset limits
redis-cli FLUSHDB
```

**Key Insights:**

- **Development:** Generous limits (100+) for rapid iteration
- **Production:** Strict limits (5) for security
- **Environment-aware:** Same codebase, different behavior
- **Cache clearing:** Use sparingly, only when blocked

## Testing Strategy

### Unit Tests

```typescript
describe('JWT Service', () => {
  it('should sign token with correct expiry', () => {
    const token = jwtService.signV2({ userId: '123', roleIds: ['admin'] });
    const decoded = jwt.verify(token, JWT_SECRET);

    expect(decoded.exp - decoded.iat).toBe(7 * 24 * 60 * 60); // 7 days
  });
});

describe('Cookie Configuration', () => {
  it('should set cookie with matching clear properties', () => {
    const res = mockResponse();

    setRefreshTokenCookie(res, 'token');
    const setCookieArgs = res.cookie.mock.calls[0][2];

    clearRefreshTokenCookie(res);
    const clearCookieArgs = res.clearCookie.mock.calls[0][1];

    // Verify all properties match
    expect(setCookieArgs.path).toBe(clearCookieArgs.path);
    expect(setCookieArgs.domain).toBe(clearCookieArgs.domain);
    expect(setCookieArgs.sameSite).toBe(clearCookieArgs.sameSite);
  });
});
```

### Integration Tests

```typescript
describe('Token Refresh Flow', () => {
  it('should refresh token and retry failed request', async () => {
    // 1. Login to get tokens
    const loginRes = await request(app)
      .post('/auth/login')
      .send({ email: 'user@test.com', password: 'password' });

    const refreshCookie = loginRes.headers['set-cookie'];

    // 2. Clear access token (simulate expiry)
    // 3. Make authenticated request (should fail with 401)
    // 4. Interceptor should refresh and retry
    const res = await request(app).get('/protected-route').set('Cookie', refreshCookie).expect(200);

    expect(res.body).toBeDefined();
  });
});
```

### E2E Tests

```typescript
test('should maintain session across page refresh', async ({ page }) => {
  // 1. Login
  await page.goto('/login');
  await page.fill('[name="email"]', 'user@test.com');
  await page.fill('[name="password"]', 'password');
  await page.click('[type="submit"]');

  // 2. Verify logged in
  await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();

  // 3. Refresh page
  await page.reload();

  // 4. Should still be logged in
  await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
});

test('should not logout on tab switch', async ({ page }) => {
  // 1. Login
  await page.goto('/login');
  // ... login steps

  // 2. Open new tab
  const newPage = await page.context().newPage();
  await newPage.goto('https://example.com');

  // 3. Wait a few seconds
  await page.waitForTimeout(3000);

  // 4. Return to original tab
  await page.bringToFront();

  // 5. Should still be logged in
  await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
});
```

## Security Considerations

### XSS Protection

- ✅ Access token in memory (not accessible via `document.cookie` or `localStorage`)
- ✅ Refresh token in httpOnly cookie (not accessible via JavaScript)
- ✅ Proper Content Security Policy headers

### CSRF Protection

- ✅ SameSite cookie attribute (`'strict'` in production)
- ✅ httpOnly cookies (can't be read by JavaScript)
- ✅ Proper origin validation

### Token Rotation

- ✅ New refresh token issued on each refresh
- ✅ Old refresh tokens invalidated
- ✅ Database tracking of active tokens

### Logout Security

- ✅ Clear both access and refresh tokens
- ✅ Invalidate refresh token in database
- ✅ Clear React Query cache
- ✅ Redirect to login page

## Performance Considerations

- **Token Size**: Keep JWT payload small (only essential claims)
- **Cache Duration**: Balance between security (short) and performance (long)
- **Refresh Frequency**: Only refresh when access token actually expires
- **Database Queries**: Index refresh tokens by token value for fast lookup

## Related Patterns

- **01-RBAC-CASL-Pattern.md** - Role-based access control with JWT claims
- **03-Hexagonal-Architecture.md** - Separation of auth logic across layers
- **06-Repository-Pattern.md** - Refresh token persistence

## References

- [RFC 7519 - JWT Specification](https://tools.ietf.org/html/rfc7519)
- [OWASP Authentication Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [HttpOnly Cookies](https://owasp.org/www-community/HttpOnly)

## Changelog

- **2025-10-25**: Added cross-origin and rate limiting documentation
  - Documented cross-origin cookie blocking in subdomain development
  - Added middleware bypass solution for development mode
  - Documented rate limiting issues and environment-based limits
  - Added troubleshooting for login redirect loops
  - Included Redis cache clearing for rate limit reset

- **2025-10-15**: Initial pattern documentation
  - Documented dual-token authentication strategy
  - Added cookie configuration best practices
  - Included React Query race condition solutions
  - Added common pitfalls and solutions
