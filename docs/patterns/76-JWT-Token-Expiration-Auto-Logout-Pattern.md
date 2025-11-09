# Pattern 76: JWT Token Expiration and Auto-Logout Pattern

**Category**: Security, Authentication, Mobile
**Complexity**: Medium
**Last Updated**: 2025-10-26

## Intent

Automatically validate JWT token expiration and log users out when tokens expire, especially after the app has been suspended or offline for extended periods.

## Problem

Mobile apps can be suspended for hours or days, and JWT tokens have limited lifespans (typically 24 hours). Without proper validation:
- Users may continue using the app with expired tokens
- API calls fail unexpectedly with 401 errors
- Poor user experience when offline for more than token lifetime
- Security risk of accepting expired credentials

## Solution

Implement automatic token expiration checking at multiple checkpoints:

1. **On App Resume** - Check when app comes from background to foreground
2. **On Auth Check** - Validate token expiration in `isAuthenticated()`
3. **JWT Decoding** - Parse JWT payload to extract expiration claim
4. **Auto-Logout** - Clear credentials when token is expired

## Implementation

### Auth Service (apps/mobile/src/services/auth.ts)

```typescript
/**
 * Check if user is authenticated and token is valid (not expired)
 */
async isAuthenticated(): Promise<boolean> {
  const token = await this.getAuthToken();
  if (!token) {
    return false;
  }

  // Check if token is expired
  const isExpired = this.isTokenExpired(token);
  if (isExpired) {
    // Auto-logout if token expired
    console.log('[Auth] Token expired, logging out...');
    await this.logout();
    return false;
  }

  return true;
}

/**
 * Decode JWT token and check if it's expired
 */
private isTokenExpired(token: string): boolean {
  try {
    // JWT structure: header.payload.signature
    const parts = token.split('.');
    if (parts.length !== 3) {
      console.error('[Auth] Invalid JWT token format');
      return true; // Treat invalid token as expired
    }

    // Decode the payload (base64)
    const payload = parts[1];
    const decodedPayload = JSON.parse(this.base64UrlDecode(payload));

    // Check expiration (exp is in seconds, Date.now() is in milliseconds)
    if (!decodedPayload.exp) {
      console.error('[Auth] Token missing expiration claim');
      return true; // No expiration = treat as expired for security
    }

    const expirationTime = decodedPayload.exp * 1000; // Convert to milliseconds
    const currentTime = Date.now();
    const isExpired = currentTime >= expirationTime;

    if (isExpired) {
      console.log('[Auth] Token expired:', {
        expirationTime: new Date(expirationTime).toISOString(),
        currentTime: new Date(currentTime).toISOString(),
      });
    }

    return isExpired;
  } catch (error) {
    console.error('[Auth] Failed to decode token:', error);
    return true; // Treat decode errors as expired
  }
}

/**
 * Base64 URL decode (handles JWT base64url encoding)
 */
private base64UrlDecode(str: string): string {
  // Replace URL-safe characters
  let base64 = str.replace(/-/g, '+').replace(/_/g, '/');

  // Pad with '=' to make length multiple of 4
  const padding = base64.length % 4;
  if (padding > 0) {
    base64 += '='.repeat(4 - padding);
  }

  // Decode base64
  if (Platform.OS === 'web') {
    return atob(base64);
  } else {
    // React Native: Use manual base64 decode
    return this.decodeBase64(base64);
  }
}

/**
 * Manual base64 decode for React Native
 */
private decodeBase64(base64: string): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
  let result = '';
  let buffer = 0;
  let bits = 0;

  for (let i = 0; i < base64.length; i++) {
    const char = base64[i];
    if (char === '=') break;

    const value = chars.indexOf(char);
    if (value === -1) continue;

    buffer = (buffer << 6) | value;
    bits += 6;

    if (bits >= 8) {
      bits -= 8;
      result += String.fromCharCode((buffer >> bits) & 0xff);
    }
  }

  return result;
}
```

### App State Listener (apps/mobile/app/_layout.tsx)

```typescript
import {AppState, AppStateStatus} from 'react-native';
import {authService} from '../src/services/auth';

export default function RootLayout() {
  const appState = useRef(AppState.currentState);
  const router = useRouter();
  const segments = useSegments();

  useEffect(() => {
    // Listen for app state changes (background/foreground)
    const subscription = AppState.addEventListener('change', handleAppStateChange);

    return () => {
      subscription.remove();
    };
  }, []);

  const handleAppStateChange = async (nextAppState: AppStateStatus) => {
    // Check if app is coming to foreground from background
    if (appState.current.match(/inactive|background/) && nextAppState === 'active') {
      console.log('[App] App has come to the foreground - checking auth...');

      // Check if user is authenticated and token is valid
      const isAuthenticated = await authService.isAuthenticated();

      if (!isAuthenticated) {
        // Token expired or invalid - redirect to login
        console.log('[App] Token expired or invalid, redirecting to login...');

        // Only redirect if not already on auth screens
        const inAuthGroup = segments[0] === '(auth)';
        if (!inAuthGroup) {
          router.replace('/(auth)/login');
        }
      }
    }

    appState.current = nextAppState;
  };
}
```

## When to Use

- Mobile apps with JWT authentication
- Apps that can be suspended for extended periods
- Offline-first apps where token may expire while offline
- Any application requiring secure session management

## Benefits

1. **Security** - Expired tokens are automatically detected and cleared
2. **User Experience** - Clear feedback when session expires
3. **Offline Handling** - Works correctly when app is offline for days
4. **App Resume** - Validates auth every time user returns to app
5. **No Silent Failures** - API calls don't fail unexpectedly with 401 errors

## Drawbacks

1. **Complexity** - Requires JWT decoding without external libraries
2. **Manual Base64** - Need custom base64 decoder for React Native
3. **Network Check** - Can't validate with server when offline (client-side only)

## Related Patterns

- **58 - JWT Authentication & Session Management**: Core JWT authentication
- **70 - Offline Batch Sync Pattern**: Handling offline scenarios
- **Retry with Exponential Backoff**: For API calls when token validation fails

## Example Usage

### Scenario: User leaves app suspended overnight

1. User logs in at 8 AM → Token expires at 8 AM next day (24h lifetime)
2. User suspends app at 6 PM
3. User returns at 9 AM next day (app comes to foreground)
4. **App State Listener triggers**:
   - Detects app resumed from background
   - Calls `authService.isAuthenticated()`
   - JWT decoder checks `exp` claim: 8 AM < 9 AM → **expired**
   - Auto-logout: Clears token from SecureStore
   - Router redirects to login screen
5. User sees login screen with clear session expiration message

### Scenario: User is offline for 3 days

1. User logs in → Token expires in 24 hours
2. User goes offline (no network) for 3 days
3. Token expires after 24 hours (detected client-side only)
4. User opens app on day 3:
   - `isAuthenticated()` checks JWT locally
   - Token is expired → Auto-logout
   - User must log in again (requires network)

## Testing

```typescript
// Test token expiration detection
describe('JWT Token Expiration', () => {
  it('should detect expired token', async () => {
    // Create expired token (exp in the past)
    const expiredToken = createJWT({exp: Date.now() / 1000 - 3600}); // 1 hour ago
    await authService.saveAuthToken(expiredToken);

    const isAuth = await authService.isAuthenticated();
    expect(isAuth).toBe(false);

    // Token should be cleared
    const token = await authService.getAuthToken();
    expect(token).toBeNull();
  });

  it('should allow valid token', async () => {
    // Create valid token (exp in future)
    const validToken = createJWT({exp: Date.now() / 1000 + 3600}); // 1 hour from now
    await authService.saveAuthToken(validToken);

    const isAuth = await authService.isAuthenticated();
    expect(isAuth).toBe(true);
  });
});
```

## Security Considerations

1. **Client-Side Validation** - This only validates expiration locally; server must still validate
2. **Clock Skew** - Device clock may be incorrect; server validation is authoritative
3. **Logout Side Effects** - Ensure all user data is cleared on auto-logout
4. **Tenant Secret** - Tenant credentials persist across logout (intentional)

## See Also

- `apps/mobile/src/services/auth.ts` - Full implementation
- `apps/mobile/app/_layout.tsx` - App state listener setup
- JWT RFC 7519: https://datatracker.ietf.org/doc/html/rfc7519
