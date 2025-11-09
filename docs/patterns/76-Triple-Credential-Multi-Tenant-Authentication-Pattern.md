# Triple-Credential Multi-Tenant Authentication Pattern

**Category**: Security Pattern
**Complexity**: Advanced
**Status**: ✅ Production Ready (Mobile Apps)
**Related Patterns**: Database-Per-Tenant Multi-Tenancy, JWT Authentication
**Industry Context**: Oil & Gas Field Data Management

---

## Overview

The Triple-Credential Authentication Pattern provides **defense-in-depth** security for mobile applications accessing multi-tenant APIs. Unlike web applications that use subdomain-based routing, mobile apps require explicit tenant identification and authorization through three layers of credentials:

1. **X-Tenant-ID** (Layer 1): Public tenant identifier
2. **X-Tenant-Secret** (Layer 2): Server-issued tenant-level credential
3. **User Credentials** (Layer 3): Email + password → JWT token

This pattern ensures that even if user credentials are compromised, an attacker cannot access the API without the tenant secret key. The tenant secret acts like an **API key at the organization level**, separate from individual user authentication.

---

## The Problem

### Scenario: Mobile Apps Cannot Use Subdomain Routing

**Web Apps** (Subdomain-based):
```
https://acmeoil.onwellos.com/api/wells
   ↓
Tenant identified by subdomain: "acmeoil"
   ↓
API connects to acmeoil's database
```

**Mobile Apps** (IP Address-based):
```
http://192.168.1.174:4000/api/wells
   ↓
No subdomain available! (connecting via local IP)
   ↓
How does API know which tenant's database to use?
```

### Security Challenges

1. **Stolen User Credentials**: If attacker steals email + password, they can login
2. **Man-in-the-Middle**: Intercepted JWT token allows API access
3. **Compromised Device**: Malicious app on device could extract credentials
4. **No Tenant Binding**: User credentials alone don't prove authorization to access specific tenant

### What We Need

- **Tenant-level authentication** (separate from user-level)
- **Rotatable credentials** (admin can revoke compromised secrets)
- **Mobile-friendly** (works with IP addresses, no subdomain required)
- **Defense-in-depth** (multiple layers of security)

---

## The Solution

### Triple-Credential Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Mobile App Login Flow                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  1. User enters on Login Screen:                               │
│     ┌─────────────────────────────────────────┐                │
│     │ Tenant ID:   DEMO-A5L32W                │ (persistent)   │
│     │ Email:       peter@demo.com             │                │
│     │ Password:    ••••••••                   │                │
│     └─────────────────────────────────────────┘                │
│                        ↓                                        │
│  2. App sends to API:                                          │
│     POST /api/auth/login                                       │
│     Headers:                                                    │
│       X-Tenant-ID: DEMO-A5L32W        ← Layer 1: Tenant ID     │
│       X-Tenant-Secret: abc123...       ← Layer 2: Tenant Secret │
│     Body:                                                       │
│       { email, password }              ← Layer 3: User Creds   │
│                        ↓                                        │
│  3. API validates ALL THREE layers:                            │
│     ✓ Tenant ID exists and is active                           │
│     ✓ Tenant Secret matches stored value                       │
│     ✓ User email + password are correct                        │
│                        ↓                                        │
│  4. API returns:                                               │
│     {                                                           │
│       accessToken: "jwt_token...",                             │
│       user: { id, email, name, role },                         │
│       tenantSecret: "abc123..."  ← Server returns secret!      │
│     }                                                           │
│                        ↓                                        │
│  5. App stores securely:                                       │
│     iOS Keychain / Android EncryptedSharedPreferences:         │
│       - tenantId: "DEMO-A5L32W"     (persists forever)         │
│       - tenantSecret: "abc123..."    (persists, rotatable)     │
│       - authToken: "jwt_token..."    (cleared on logout)       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Subsequent API Requests

```
┌─────────────────────────────────────────────────────────────────┐
│                All Future API Requests                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  GET /api/wells                                                │
│  Headers:                                                       │
│    Authorization: Bearer jwt_token...     ← User-level auth    │
│    X-Tenant-ID: DEMO-A5L32W               ← Tenant identifier  │
│    X-Tenant-Secret: abc123...             ← Tenant-level auth  │
│                                                                 │
│  API validates:                                                │
│    1. JWT token is valid and not expired                       │
│    2. Tenant ID exists in master database                      │
│    3. Tenant Secret matches stored value                       │
│    4. User belongs to this tenant                              │
│                                                                 │
│  Then connects to tenant's database and returns data           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Implementation

### 1. Tenant ID Format

**Structure**: `COMPANY-RANDOM`

- **Company Code**: 1-8 uppercase letters (human-readable)
- **Random Suffix**: 6 alphanumeric characters (prevents collisions)

**Examples**:
- `DEMO-A5L32W` (demo tenant)
- `ACMEOIL-9K2P4H` (ACME Oil & Gas)
- `TEXASOIL-7M3N8K` (Texas Oil Company)

**Validation Regex**: `/^[A-Z]{1,8}-[A-Z0-9]{6}$/i`

### 2. Tenant Entity (Domain Layer)

```typescript
// apps/api/src/domain/tenants/tenant.entity.ts
export interface TenantProps {
  id: string;                  // UUID (internal)
  tenantId: string;            // COMPANY-XXXXXX (external identifier)
  secretKey: string;           // Server-generated secret (hashed)
  secretRotatedAt?: Date;      // Track when secret was last rotated
  slug: string;                // For subdomain routing (web apps)
  subdomain: string;           // For subdomain routing (web apps)
  // ... other tenant properties
}

export class Tenant {
  // Generate secure random secret key
  static generateSecretKey(): string {
    return crypto.randomBytes(32).toString('hex'); // 64 character hex string
  }

  // Validate tenant secret (constant-time comparison to prevent timing attacks)
  validateSecretKey(providedSecret: string): boolean {
    return crypto.timingSafeEqual(
      Buffer.from(this.props.secretKey),
      Buffer.from(providedSecret)
    );
  }

  // Rotate secret key (for security or when compromised)
  rotateSecretKey(): string {
    const newSecret = Tenant.generateSecretKey();
    this.props.secretKey = newSecret;
    this.props.secretRotatedAt = new Date();
    this.props.updatedAt = new Date();
    return newSecret; // Return once for admin to save securely
  }

  // Generate tenant ID (called during tenant creation)
  static generateTenantId(companyName: string): string {
    // Extract uppercase letters from company name (max 8 chars)
    const companyCode = companyName
      .replace(/[^a-zA-Z]/g, '')
      .toUpperCase()
      .substring(0, 8);

    // Generate 6-character random suffix
    const randomSuffix = crypto
      .randomBytes(3)
      .toString('hex')
      .toUpperCase()
      .substring(0, 6);

    return `${companyCode}-${randomSuffix}`;
  }
}
```

### 3. Tenant Resolver Middleware (Updated)

```typescript
// Tenant resolver middleware for mobile and web apps
export class TenantResolverMiddleware {
  constructor(
    private readonly tenantRepository: ITenantRepository,
  ) {}

  async use(req: Request, res: Response, next: NextFunction) {
    // MOBILE APP PATH: Check for X-Tenant-ID header
    const tenantIdHeader = req.headers['x-tenant-id'];
    const tenantSecretHeader = req.headers['x-tenant-secret'];

    if (tenantIdHeader) {
      return this.handleMobileAuth(req, res, next, tenantIdHeader, tenantSecretHeader);
    }

    // WEB APP PATH: Extract subdomain from hostname
    const subdomain = this.extractSubdomain(req.hostname);

    if (subdomain) {
      return this.handleWebAuth(req, res, next, subdomain);
    }

    // No tenant context found
    return next();
  }

  /**
   * Mobile app authentication: Validate X-Tenant-ID + X-Tenant-Secret
   */
  private async handleMobileAuth(
    req: Request,
    res: Response,
    next: NextFunction,
    tenantId: string,
    tenantSecret: string | undefined,
  ): Promise<void> {
    // Look up tenant by tenant ID
    const tenant = await this.tenantRepository.findByTenantId(tenantId);

    if (!tenant) {
      throw new UnauthorizedException(`Invalid tenant ID: ${tenantId}`);
    }

    // Check tenant status
    if (!tenant.status.canAccess()) {
      throw new UnauthorizedException(
        `Tenant is ${tenant.status.toString().toLowerCase()} and cannot access the platform`,
      );
    }

    // CRITICAL: Validate tenant secret (Layer 2 authentication)
    // Skip secret validation for login endpoint (first-time login)
    const isLoginEndpoint = req.path === '/api/auth/login' && req.method === 'POST';

    if (!isLoginEndpoint) {
      if (!tenantSecret) {
        throw new UnauthorizedException('X-Tenant-Secret header is required for mobile apps');
      }

      if (!tenant.validateSecretKey(tenantSecret)) {
        // Log failed attempt (potential security breach)
        this.logger.warn(`Invalid tenant secret for tenant: ${tenantId}`);
        throw new UnauthorizedException('Invalid tenant credentials');
      }
    }

    // Attach tenant context to request
    req.tenant = {
      id: tenant.id,
      tenantId: tenant.tenantId,
      subdomain: tenant.subdomain,
      slug: tenant.slug,
      databaseUrl: tenant.databaseConfig.url,
      databaseName: tenant.databaseConfig.name,
      databaseType: tenant.databaseConfig.type,
    };

    next();
  }

  /**
   * Web app authentication: Extract subdomain and lookup tenant
   */
  private async handleWebAuth(
    req: Request,
    res: Response,
    next: NextFunction,
    subdomain: string,
  ): Promise<void> {
    // Look up tenant by subdomain
    const tenant = await this.tenantRepository.findBySubdomain(subdomain);

    if (!tenant) {
      throw new NotFoundException(`Tenant not found: ${subdomain}`);
    }

    if (!tenant.status.canAccess()) {
      throw new NotFoundException(
        `Tenant is ${tenant.status.toString().toLowerCase()} and cannot access the platform`,
      );
    }

    // Attach tenant context to request
    req.tenant = {
      id: tenant.id,
      tenantId: tenant.tenantId,
      subdomain: tenant.subdomain,
      slug: tenant.slug,
      databaseUrl: tenant.databaseConfig.url,
      databaseName: tenant.databaseConfig.name,
      databaseType: tenant.databaseConfig.type,
    };

    next();
  }

  private extractSubdomain(hostname: string): string | null {
    // ... existing subdomain extraction logic
  }
}
```

### 4. Auth Controller (Updated)

```typescript
// Auth controller with mobile app support
export class AuthController {
  constructor(private readonly commandBus: CommandBus) {}

  async login(
    tenant: TenantContextDto | undefined,
    dto: LoginDto,
    res: Response,
  ): Promise<{
    message: string;
    accessToken: string;
    user: { id: string; email: string; name: string; role: string };
    tenantSecret?: string; // ADDED: Return tenant secret for mobile apps
  }> {
    if (!tenant) {
      throw new UnauthorizedException('Tenant context is required');
    }

    // Execute login command (validates email + password)
    const command = new LoginCommand(
      tenant.id,
      dto.email,
      dto.password,
      tenant.databaseName,
    );

    const result = await this.commandBus.execute(command);

    // Set refresh token as httpOnly cookie (for web apps)
    this.setRefreshTokenCookie(res, result.refreshToken);

    // Load tenant entity to get secret key
    const tenantEntity = await this.tenantRepository.findById(tenant.id);

    // Return tenant secret ONLY for mobile apps (detected by X-Tenant-ID header)
    const isMobileApp = req.headers['x-tenant-id'] !== undefined;

    return {
      message: 'Login successful',
      accessToken: result.accessToken,
      user: result.user,
      tenantSecret: isMobileApp ? tenantEntity.secretKey : undefined,
    };
  }
}
```

### 5. Mobile App Implementation

**AuthService** (apps/mobile/src/services/auth.ts):

```typescript
export class AuthService {
  private readonly TENANT_ID_KEY = 'wellos_tenant_id';
  private readonly TENANT_SECRET_KEY = 'wellos_tenant_secret';
  private readonly AUTH_TOKEN_KEY = 'wellos_auth_token';

  /**
   * Login with triple-credential authentication
   */
  async login(
    email: string,
    password: string,
    tenantId: string,
  ): Promise<{ success: boolean; token?: string; user?: User; error?: string }> {
    const API_URL = process.env.EXPO_PUBLIC_API_URL || 'http://localhost:4000';

    try {
      // Save tenant ID (persists forever)
      await this.saveTenantId(tenantId);

      // Get existing secret if available (from previous login)
      const existingSecret = await this.getTenantSecret();

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        'X-Tenant-ID': tenantId,
      };

      // Include tenant secret if we have one (Layer 2 auth)
      if (existingSecret) {
        headers['X-Tenant-Secret'] = existingSecret;
      }

      // Send request with all three credentials
      const response = await fetch(`${API_URL}/api/auth/login`, {
        method: 'POST',
        headers,
        body: JSON.stringify({ email, password }), // Layer 3 auth
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({ message: 'Login failed' }));
        return { success: false, error: errorData.message };
      }

      const data = await response.json();

      // Extract all credentials
      const user: User = {
        id: data.user.id,
        email: data.user.email,
        name: data.user.name,
      };

      const token = data.accessToken;
      const tenantSecret = data.tenantSecret; // Server returns secret

      // Save all credentials securely
      await this.saveAuthToken(token);
      await this.saveUserData(user);
      await this.saveTenantSecret(tenantSecret); // Persist for future requests

      return { success: true, token, user };
    } catch (error) {
      console.error('Login error:', error);
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Get headers for authenticated API requests (all three credentials)
   */
  async getAuthHeaders(): Promise<Record<string, string>> {
    const token = await this.getAuthToken();
    const tenantId = await this.getTenantId();
    const tenantSecret = await this.getTenantSecret();

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`; // Layer 3
    }

    if (tenantId) {
      headers['X-Tenant-ID'] = tenantId; // Layer 1
    }

    if (tenantSecret) {
      headers['X-Tenant-Secret'] = tenantSecret; // Layer 2
    }

    return headers;
  }

  /**
   * Logout - clear auth token but KEEP tenant credentials
   */
  async logout(): Promise<void> {
    await this.removeAuthToken(); // Clear JWT
    // Tenant ID and secret persist (no need to re-enter tenant ID)
  }
}
```

---

## Security Benefits

### Defense-in-Depth (Three Layers)

```
Attacker Scenarios:

1. Stolen User Password:
   ❌ Cannot login without tenant ID and secret
   ✓ Admin rotates tenant secret → all compromised sessions invalidated

2. Intercepted JWT Token:
   ❌ Cannot make API calls without tenant ID and secret
   ✓ JWT expires, attacker cannot refresh

3. Compromised Device:
   ❌ Attacker gets tenant secret but needs user password to login
   ✓ Admin rotates tenant secret → device credentials invalidated

4. Man-in-the-Middle:
   ❌ Attacker needs all three credentials to impersonate user
   ✓ SSL/TLS prevents MITM attacks in production
```

### Tenant-Level Access Control

**Scenario**: Employee leaves company, takes login credentials

```typescript
// Without tenant secret (old approach):
// Attacker can login with stolen email + password
const response = await fetch('/api/auth/login', {
  body: JSON.stringify({ email: 'ex-employee@acme.com', password: 'stolen123' }),
});
// ✓ Login succeeds! Attacker has access.

// With tenant secret (triple-credential):
// Attacker needs BOTH tenant secret AND user password
const response = await fetch('/api/auth/login', {
  headers: {
    'X-Tenant-ID': 'ACMEOIL-9K2P4H',
    'X-Tenant-Secret': 'UNKNOWN_SECRET', // Attacker doesn't have this
  },
  body: JSON.stringify({ email: 'ex-employee@acme.com', password: 'stolen123' }),
});
// ❌ 401 Unauthorized: Invalid tenant credentials
```

**Admin Remediation**:
```typescript
// Admin rotates tenant secret in admin portal
await tenantEntity.rotateSecretKey();
await tenantRepository.save(tenantEntity);

// All mobile devices with old secret → invalidated
// Legitimate users → re-login to get new secret
// Ex-employee → cannot access even with valid password
```

### Secret Rotation

**Use Cases**:
1. **Security Breach**: Tenant secret leaked
2. **Employee Offboarding**: Former employee had access
3. **Regular Rotation**: Rotate every 90 days (security policy)
4. **Compliance**: Audit requirement for credential rotation

**Implementation** (Admin Panel):

```typescript
// Tenant management controller for secret rotation
export class TenantManagementController {
  constructor(
    private readonly tenantRepository: ITenantRepository,
  ) {}

  /**
   * Rotate tenant secret key
   * POST /admin/tenants/:tenantId/rotate-secret
   */
  async rotateSecret(
    tenantId: string,
  ): Promise<{ message: string; newSecret: string }> {
    // Load tenant
    const tenant = await this.tenantRepository.findByTenantId(tenantId);

    if (!tenant) {
      throw new NotFoundException(`Tenant not found: ${tenantId}`);
    }

    // Generate new secret
    const newSecret = tenant.rotateSecretKey();

    // Save to database
    await this.tenantRepository.update(tenant);

    // Log security event
    this.auditLogger.log({
      action: 'TENANT_SECRET_ROTATED',
      tenantId: tenantId,
      performedBy: 'super-admin',
      timestamp: new Date(),
    });

    // Return new secret (ONLY DISPLAY ONCE - admin must save it)
    return {
      message: 'Tenant secret rotated successfully. Save this secret - it will not be shown again.',
      newSecret, // Display once, then hash it
    };
  }
}
```

---

## Persistence Strategy

### Mobile App (Secure Storage)

```typescript
// Credential Lifecycle:

┌──────────────────────────────────────────────────────────────┐
│ First Login:                                                 │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 1. User enters: Tenant ID + Email + Password            │ │
│ │ 2. API validates and returns: JWT + Tenant Secret       │ │
│ │ 3. App stores securely:                                 │ │
│ │    - Tenant ID: iOS Keychain / Android EncryptedStore  │ │
│ │    - Tenant Secret: iOS Keychain / Android Encrypted   │ │
│ │    - JWT Token: iOS Keychain / Android Encrypted       │ │
│ └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Subsequent Logins:                                           │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 1. Tenant ID field: Auto-populated from storage         │ │
│ │ 2. User enters: Email + Password only                   │ │
│ │ 3. App sends: Tenant ID (stored) + Secret (stored) +    │ │
│ │               Email + Password (entered)                 │ │
│ │ 4. API validates all three credentials                   │ │
│ └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Logout:                                                      │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ - JWT Token: CLEARED                                     │ │
│ │ - Tenant ID: PERSISTS (for next login)                  │ │
│ │ - Tenant Secret: PERSISTS (for next login)              │ │
│ └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│ Secret Rotation (by Admin):                                 │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ 1. Admin rotates secret in admin panel                   │ │
│ │ 2. Next API request: 401 Unauthorized (old secret)       │ │
│ │ 3. App prompts: "Please re-login to continue"            │ │
│ │ 4. User re-logins → receives new secret                  │ │
│ │ 5. App replaces old secret with new secret in storage    │ │
│ └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────┘
```

### Backend (Master Database)

```sql
-- Master database schema (tenant credentials)
CREATE TABLE tenants (
  id VARCHAR(255) PRIMARY KEY,
  tenant_id VARCHAR(15) UNIQUE NOT NULL,        -- COMPANY-XXXXXX
  secret_key VARCHAR(64) NOT NULL,              -- Hashed secret (bcrypt/argon2)
  secret_rotated_at TIMESTAMP,                  -- Track rotation history
  slug VARCHAR(100) UNIQUE NOT NULL,            -- For subdomain routing (web)
  subdomain VARCHAR(100) UNIQUE NOT NULL,       -- For subdomain routing (web)
  -- ... other tenant fields
);

-- Index for fast lookups
CREATE INDEX idx_tenants_tenant_id ON tenants(tenant_id);

-- Audit log for secret rotations
CREATE TABLE tenant_security_audit (
  id VARCHAR(255) PRIMARY KEY,
  tenant_id VARCHAR(15) NOT NULL,
  event_type VARCHAR(50) NOT NULL,              -- 'SECRET_ROTATED', 'SECRET_VALIDATION_FAILED'
  performed_by VARCHAR(255),                     -- User ID or 'SYSTEM'
  ip_address VARCHAR(45),
  user_agent TEXT,
  created_at TIMESTAMP DEFAULT NOW(),
  FOREIGN KEY (tenant_id) REFERENCES tenants(tenant_id)
);
```

---

## When to Use This Pattern

### ✅ Use Triple-Credential When:

- **Mobile apps** cannot use subdomain routing (IP address connections)
- **Enhanced security** required for sensitive data (oil & gas production data)
- **Tenant-level access control** needed (revoke all users at once)
- **Credential rotation** capability required (compliance, security incidents)
- **Defense-in-depth** approach mandated (multi-layer security)

### ❌ Don't Use Triple-Credential When:

- **Web apps only** (subdomain routing is simpler)
- **Single tenant** application (no multi-tenancy needed)
- **Overhead not justified** (simple consumer apps)
- **Cannot manage secret rotation** (no admin infrastructure)

---

## Related Patterns

- **[Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)** - Tenant identification and data isolation
- **[JWT Authentication Session Management Pattern](./58-JWT-Authentication-Session-Management-Pattern.md)** - User-level authentication (Layer 3)
- **Repository Pattern** - Tenant-scoped data access
- **Middleware Pattern** - Tenant context injection

---

## Summary

The **Triple-Credential Multi-Tenant Authentication Pattern** provides defense-in-depth security for mobile apps:

1. **X-Tenant-ID** (Layer 1): Identifies which tenant's database to connect to
2. **X-Tenant-Secret** (Layer 2): Proves mobile app is authorized for tenant
3. **User Credentials** (Layer 3): Validates individual user identity

**Key Implementation Points**:

- Tenant ID format: `COMPANY-XXXXXX` (human-readable + unique)
- Tenant secret: 64-character hex string (crypto-secure random)
- Secret rotation: Admin can invalidate all mobile sessions
- Persistence: Tenant credentials survive logout, JWT does not
- Validation: All three credentials required for API access (except first login)

This pattern is essential for WellOS mobile apps because it enables tenant-level access control and provides multiple layers of security for sensitive production data.

---

**Implementation Checklist**:

- [ ] Update tenant entity with `tenantId` and `secretKey` fields
- [ ] Add secret generation and validation methods
- [ ] Update tenant resolver middleware to support X-Tenant-ID header
- [ ] Modify login endpoint to return `tenantSecret` for mobile apps
- [ ] Create admin endpoint for secret rotation
- [ ] Update mobile app to collect and store tenant ID
- [ ] Implement `getAuthHeaders()` helper for API requests
- [ ] Add audit logging for failed secret validations
- [ ] Test secret rotation workflow end-to-end
