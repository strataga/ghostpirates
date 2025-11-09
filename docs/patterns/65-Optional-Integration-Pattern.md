# Optional Integration Pattern - Lazy Validation for Third-Party Services

## Overview

The Optional Integration Pattern allows services to initialize without external credentials, deferring validation until features are actually used. This enables development workflows where not all integrations need to be configured, preventing application startup failures when optional third-party services are unavailable.

## Purpose

- Enable API startup without all integration credentials configured
- Support development workflows without requiring all external service setups
- Provide clear error messages when features are used without configuration
- Maintain production-grade validation when features are actually invoked
- Allow gradual integration rollout without blocking core application functionality

## Use Cases

- QuickBooks integration that's optional for organizations
- Email service providers (SendGrid, Mailgun) that aren't needed in all environments
- Payment processors (Stripe, PayPal) that may not be configured in development
- Analytics services (Segment, Mixpanel) not needed for testing
- Cloud storage providers (AWS S3, Google Cloud Storage) with local alternatives

## Before Implementation

```typescript
// ❌ Poor: Service throws error on startup without credentials
@Injectable()
export class QuickBooksAdapter {
  private readonly clientId: string;
  private readonly clientSecret: string;
  private readonly redirectUri: string;

  constructor(private configService: ConfigService) {
    // Throws error immediately if credentials missing
    this.clientId = configService.get('QUICKBOOKS_CLIENT_ID')!;
    this.clientSecret = configService.get('QUICKBOOKS_CLIENT_SECRET')!;
    this.redirectUri = configService.get('QUICKBOOKS_REDIRECT_URI')!;

    // Validation in constructor
    this.validateConfiguration(); // ❌ Throws if missing
  }

  private validateConfiguration(): void {
    if (!this.clientId) {
      throw new Error('QUICKBOOKS_CLIENT_ID is not configured');
    }
    if (!this.clientSecret) {
      throw new Error('QUICKBOOKS_CLIENT_SECRET is not configured');
    }
    if (!this.redirectUri) {
      throw new Error('QUICKBOOKS_REDIRECT_URI is not configured');
    }
  }

  getAuthorizationUrl(state: string): string {
    const params = new URLSearchParams({
      client_id: this.clientId,
      scope: 'com.intuit.quickbooks.accounting',
      redirect_uri: this.redirectUri,
      response_type: 'code',
      state,
    });

    return `${this.authUrl}?${params.toString()}`;
  }

  async exchangeCodeForTokens(code: string): Promise<QuickBooksTokens> {
    const auth = Buffer.from(`${this.clientId}:${this.clientSecret}`).toString('base64');

    const response = await this.httpClient.post(
      this.tokenUrl,
      new URLSearchParams({
        grant_type: 'authorization_code',
        code,
        redirect_uri: this.redirectUri,
      }).toString(),
      {
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          Authorization: `Basic ${auth}`,
          Accept: 'application/json',
        },
      },
    );

    return {
      accessToken: response.data.access_token,
      refreshToken: response.data.refresh_token,
      expiresIn: response.data.expires_in,
      tokenType: response.data.token_type,
    };
  }
}

// Problems:
// - API fails to start without QuickBooks credentials
// - Developers can't work on other features without QB setup
// - Production deployments fail if QB integration not ready
// - No graceful degradation
// - Error messages not helpful (just "undefined")
```

## After Implementation

```typescript
// ✅ Good: Optional Integration with Lazy Validation

@Injectable()
export class QuickBooksAdapter {
  private readonly logger = new Logger(QuickBooksAdapter.name);
  private readonly clientId: string;
  private readonly clientSecret: string;
  private readonly redirectUri: string;
  private readonly authUrl: string;
  private readonly tokenUrl: string;
  private readonly apiBaseUrl: string;
  private readonly environment: string;

  constructor(private readonly configService: ConfigService) {
    // Load configuration (allow empty for optional integration)
    this.clientId = this.configService.get<string>('QUICKBOOKS_CLIENT_ID', '');
    this.clientSecret = this.configService.get<string>('QUICKBOOKS_CLIENT_SECRET', '');
    this.redirectUri = this.configService.get<string>('QUICKBOOKS_REDIRECT_URI', '');
    this.environment = this.configService.get<string>('QUICKBOOKS_ENVIRONMENT', 'sandbox');

    // OAuth endpoints with defaults
    this.authUrl = this.configService.get<string>(
      'QUICKBOOKS_AUTH_URL',
      'https://appcenter.intuit.com/connect/oauth2',
    );
    this.tokenUrl = this.configService.get<string>(
      'QUICKBOOKS_TOKEN_URL',
      'https://oauth.platform.intuit.com/oauth2/v1/tokens/bearer',
    );
    this.apiBaseUrl = this.configService.get<string>(
      'QUICKBOOKS_API_BASE_URL',
      this.environment === 'production'
        ? 'https://quickbooks.api.intuit.com'
        : 'https://sandbox-quickbooks.api.intuit.com',
    );

    // Log initialization status (warn if not configured)
    if (this.isConfigured()) {
      this.logger.log(`QuickBooksAdapter initialized (${this.environment} environment)`);
    } else {
      this.logger.warn(
        'QuickBooksAdapter initialized without credentials (QuickBooks integration disabled)',
      );
    }
  }

  /**
   * Check if QuickBooks is configured
   */
  private isConfigured(): boolean {
    return !!(this.clientId && this.clientSecret && this.redirectUri);
  }

  /**
   * Ensure QuickBooks is configured before making OAuth calls
   * @throws Error if configuration is missing
   */
  private ensureConfigured(): void {
    if (!this.isConfigured()) {
      throw new Error(
        'QuickBooks integration is not configured. Please set QUICKBOOKS_CLIENT_ID, ' +
          'QUICKBOOKS_CLIENT_SECRET, and QUICKBOOKS_REDIRECT_URI environment variables. ' +
          'See docs/guides/quickbooks-setup.md for setup instructions.',
      );
    }
  }

  /**
   * Generate OAuth 2.0 authorization URL
   */
  getAuthorizationUrl(state: string): string {
    this.ensureConfigured(); // ✅ Validate on use, not construction

    if (!state || state.trim() === '') {
      throw new Error('State parameter is required for CSRF protection');
    }

    const params = new URLSearchParams({
      client_id: this.clientId,
      scope: 'com.intuit.quickbooks.accounting',
      redirect_uri: this.redirectUri,
      response_type: 'code',
      state,
    });

    const url = `${this.authUrl}?${params.toString()}`;

    this.logger.log(`Generated authorization URL with state: ${state}`);

    return url;
  }

  /**
   * Exchange authorization code for access and refresh tokens
   */
  async exchangeCodeForTokens(authorizationCode: string): Promise<QuickBooksTokens> {
    this.ensureConfigured(); // ✅ Validate on use

    if (!authorizationCode || authorizationCode.trim() === '') {
      throw new Error('Authorization code is required');
    }

    this.logger.log('Exchanging authorization code for tokens');

    try {
      const auth = Buffer.from(`${this.clientId}:${this.clientSecret}`).toString('base64');

      const response = await this.httpClient.post(
        this.tokenUrl,
        new URLSearchParams({
          grant_type: 'authorization_code',
          code: authorizationCode,
          redirect_uri: this.redirectUri,
        }).toString(),
        {
          headers: {
            'Content-Type': 'application/x-www-form-urlencoded',
            Authorization: `Basic ${auth}`,
            Accept: 'application/json',
          },
        },
      );

      const data = response.data;

      this.logger.log('Successfully exchanged code for tokens');

      return {
        accessToken: data.access_token,
        refreshToken: data.refresh_token,
        expiresIn: data.expires_in || 3600,
        refreshTokenExpiresIn: data.x_refresh_token_expires_in || 8640000,
        tokenType: data.token_type,
        realmId: data.realmId,
      };
    } catch (error) {
      this.logger.error('Failed to exchange authorization code:', error);
      throw this.handleApiError(error, 'Token exchange failed');
    }
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshAccessToken(refreshToken: string): Promise<QuickBooksTokens> {
    this.ensureConfigured(); // ✅ Validate on use

    if (!refreshToken || refreshToken.trim() === '') {
      throw new Error('Refresh token is required');
    }

    // ... implementation with error handling
  }

  /**
   * Revoke access and refresh tokens
   */
  async revokeTokens(accessToken: string): Promise<void> {
    this.ensureConfigured(); // ✅ Validate on use

    // ... implementation
  }

  /**
   * Make an API call to QuickBooks
   */
  async makeApiCall<T>(
    method: 'GET' | 'POST' | 'PUT' | 'DELETE',
    endpoint: string,
    realmId: string,
    accessToken: string,
    data?: any,
  ): Promise<T> {
    this.ensureConfigured(); // ✅ Validate on use

    // ... implementation
  }
}

// 2. Token Encryption Service Example
@Injectable()
export class TokenEncryptionService {
  private readonly logger = new Logger(TokenEncryptionService.name);
  private readonly algorithm = 'aes-256-cbc';
  private readonly key: Buffer | null; // ✅ Nullable type
  private readonly ivLength = 16; // AES block size

  constructor(private readonly configService: ConfigService) {
    const encryptionKey = this.configService.get<string>('QUICKBOOKS_TOKEN_ENCRYPTION_KEY', '');

    // Allow service to initialize without key for optional integration
    if (!encryptionKey) {
      this.key = null;
      this.logger.warn(
        'TokenEncryptionService initialized without encryption key (QuickBooks integration disabled)',
      );
      return; // ✅ Early return instead of throwing
    }

    // Validate key length (must be 64 hex characters = 32 bytes for AES-256)
    if (encryptionKey.length !== 64) {
      throw new Error(
        `QUICKBOOKS_TOKEN_ENCRYPTION_KEY must be 64 hex characters (32 bytes). ` +
          `Got: ${encryptionKey.length} characters. ` +
          `Generate with: openssl rand -hex 32`,
      );
    }

    // Validate hex format
    if (!/^[0-9a-fA-F]{64}$/.test(encryptionKey)) {
      throw new Error(
        'QUICKBOOKS_TOKEN_ENCRYPTION_KEY must be a valid hex string. ' +
          'Generate with: openssl rand -hex 32',
      );
    }

    this.key = Buffer.from(encryptionKey, 'hex');

    this.logger.log('🔐 TokenEncryptionService initialized with AES-256-CBC encryption');
  }

  /**
   * Ensure encryption key is configured
   * @throws Error if encryption key is not configured
   */
  private ensureConfigured(): void {
    if (!this.key) {
      throw new Error(
        'QuickBooks token encryption is not configured. Please set QUICKBOOKS_TOKEN_ENCRYPTION_KEY ' +
          'environment variable. ' +
          "Generate with: node -e \"console.log(require('crypto').randomBytes(32).toString('hex'))\" " +
          'See docs/guides/quickbooks-setup.md for setup instructions.',
      );
    }
  }

  /**
   * Encrypt a plain text token
   */
  encrypt(plainText: string): string {
    this.ensureConfigured(); // ✅ Validate on use

    if (!plainText) {
      throw new Error('Cannot encrypt empty token');
    }

    try {
      // Generate random IV for this encryption
      const iv = crypto.randomBytes(this.ivLength);

      // Create cipher with key and IV
      // ✅ Non-null assertion safe after ensureConfigured()
      const cipher = crypto.createCipheriv(this.algorithm, this.key!, iv);

      // Encrypt the token
      let encrypted = cipher.update(plainText, 'utf8', 'hex');
      encrypted += cipher.final('hex');

      // Return IV and encrypted data separated by colon
      return `${iv.toString('hex')}:${encrypted}`;
    } catch (error) {
      this.logger.error('Failed to encrypt token:', error);
      throw new Error('Token encryption failed');
    }
  }

  /**
   * Decrypt an encrypted token
   */
  decrypt(encryptedText: string): string {
    this.ensureConfigured(); // ✅ Validate on use

    if (!encryptedText) {
      throw new Error('Cannot decrypt empty token');
    }

    try {
      // Split IV and encrypted data
      const parts = encryptedText.split(':');

      if (parts.length !== 2) {
        throw new Error('Invalid encrypted token format. Expected: "iv:encryptedData"');
      }

      const iv = Buffer.from(parts[0], 'hex');
      const encrypted = parts[1];

      // Validate IV length
      if (iv.length !== this.ivLength) {
        throw new Error(`Invalid IV length. Expected: ${this.ivLength}, Got: ${iv.length}`);
      }

      // Create decipher with key and IV
      const decipher = crypto.createDecipheriv(this.algorithm, this.key!, iv);

      // Decrypt the token
      let decrypted = decipher.update(encrypted, 'hex', 'utf8');
      decrypted += decipher.final('utf8');

      return decrypted;
    } catch (error) {
      this.logger.error('Failed to decrypt token:', error);
      throw new Error('Token decryption failed');
    }
  }

  /**
   * Verify encryption/decryption is working correctly
   */
  async verifyEncryption(): Promise<boolean> {
    // Skip verification if not configured (optional integration)
    if (!this.key) {
      this.logger.warn('Skipping encryption verification (not configured)');
      return false;
    }

    try {
      const testToken = 'test-token-12345';
      const encrypted = this.encrypt(testToken);
      const decrypted = this.decrypt(encrypted);

      if (decrypted !== testToken) {
        this.logger.error('Encryption verification failed: token mismatch');
        return false;
      }

      this.logger.log('✓ Encryption verification successful');
      return true;
    } catch (error) {
      this.logger.error('Encryption verification failed:', error);
      return false;
    }
  }
}

// 3. Testing Optional Integration
describe('QuickBooksAdapter (Optional Integration)', () => {
  describe('without credentials', () => {
    it('should initialize without throwing', () => {
      const configService = createMockConfigService({});

      expect(() => new QuickBooksAdapter(configService)).not.toThrow();
    });

    it('should log warning when not configured', () => {
      const configService = createMockConfigService({});
      const adapter = new QuickBooksAdapter(configService);

      expect(mockLogger.warn).toHaveBeenCalledWith(
        expect.stringContaining('initialized without credentials'),
      );
    });

    it('should throw helpful error when getAuthorizationUrl is called', () => {
      const configService = createMockConfigService({});
      const adapter = new QuickBooksAdapter(configService);

      expect(() => adapter.getAuthorizationUrl('state')).toThrowError(
        /QuickBooks integration is not configured/,
      );
      expect(() => adapter.getAuthorizationUrl('state')).toThrowError(/QUICKBOOKS_CLIENT_ID/);
      expect(() => adapter.getAuthorizationUrl('state')).toThrowError(/quickbooks-setup.md/);
    });
  });

  describe('with credentials', () => {
    it('should initialize successfully', () => {
      const configService = createMockConfigService({
        QUICKBOOKS_CLIENT_ID: 'client123',
        QUICKBOOKS_CLIENT_SECRET: 'secret456',
        QUICKBOOKS_REDIRECT_URI: 'http://localhost:3001/callback',
      });

      expect(() => new QuickBooksAdapter(configService)).not.toThrow();
    });

    it('should log success when configured', () => {
      const configService = createMockConfigService({
        QUICKBOOKS_CLIENT_ID: 'client123',
        QUICKBOOKS_CLIENT_SECRET: 'secret456',
        QUICKBOOKS_REDIRECT_URI: 'http://localhost:3001/callback',
      });
      const adapter = new QuickBooksAdapter(configService);

      expect(mockLogger.log).toHaveBeenCalledWith(
        expect.stringContaining('initialized (sandbox environment)'),
      );
    });

    it('should allow getAuthorizationUrl to be called', () => {
      const configService = createMockConfigService({
        QUICKBOOKS_CLIENT_ID: 'client123',
        QUICKBOOKS_CLIENT_SECRET: 'secret456',
        QUICKBOOKS_REDIRECT_URI: 'http://localhost:3001/callback',
      });
      const adapter = new QuickBooksAdapter(configService);

      const url = adapter.getAuthorizationUrl('state123');

      expect(url).toContain('client_id=client123');
      expect(url).toContain('redirect_uri=');
      expect(url).toContain('state=state123');
    });
  });
});
```

## Benefits

1. **Developer Experience**: API starts without all integrations configured
2. **Graceful Degradation**: Missing credentials log warnings instead of crashing
3. **Clear Error Messages**: Helpful errors with exact setup instructions when features are used
4. **Production Safe**: Validation still enforced before external API calls
5. **Flexible Deployment**: Enable integrations when ready, not all at once
6. **Type Safety**: TypeScript null checks ensure proper validation
7. **Testable**: Easy to test both configured and unconfigured states
8. **Documentation Driven**: Error messages link to setup guides

## Implementation Pattern

### 1. Constructor Pattern

```typescript
constructor(private configService: ConfigService) {
  // ✅ Use default empty values
  this.clientId = this.configService.get<string>('SERVICE_CLIENT_ID', '');
  this.clientSecret = this.configService.get<string>('SERVICE_CLIENT_SECRET', '');

  // ✅ Log warning if not configured
  if (!this.isConfigured()) {
    this.logger.warn('Service initialized without credentials (integration disabled)');
  } else {
    this.logger.log('Service initialized successfully');
  }
}
```

### 2. Helper Methods Pattern

```typescript
/**
 * Check if service is configured
 */
private isConfigured(): boolean {
  return !!(this.clientId && this.clientSecret && this.apiKey);
}

/**
 * Ensure service is configured before making API calls
 * @throws Error with helpful message if not configured
 */
private ensureConfigured(): void {
  if (!this.isConfigured()) {
    throw new Error(
      'Service integration is not configured. ' +
      'Please set SERVICE_CLIENT_ID, SERVICE_CLIENT_SECRET, and SERVICE_API_KEY. ' +
      'Generate API key with: npm run generate:api-key ' +
      'See docs/guides/service-setup.md for setup instructions.'
    );
  }
}
```

### 3. Protected Public Methods Pattern

```typescript
/**
 * Public method - protected with ensureConfigured()
 */
public async makeApiCall(params: ApiParams): Promise<ApiResponse> {
  this.ensureConfigured(); // ✅ Validate on use

  // ... implementation
}
```

### 4. Nullable Type Pattern (for encryption keys, buffers, etc.)

```typescript
private readonly key: Buffer | null; // ✅ Allow null

constructor(private configService: ConfigService) {
  const encryptionKey = this.configService.get<string>('ENCRYPTION_KEY', '');

  if (!encryptionKey) {
    this.key = null; // ✅ Set to null instead of throwing
    this.logger.warn('Encryption not configured');
    return; // ✅ Early return
  }

  this.key = Buffer.from(encryptionKey, 'hex');
}

encrypt(data: string): string {
  this.ensureConfigured(); // Throws if key is null

  // ✅ Non-null assertion safe after validation
  const cipher = crypto.createCipheriv(algorithm, this.key!, iv);
  // ...
}
```

## Implementation Checklist

- [ ] Identify optional third-party integrations
- [ ] Update service constructors to accept empty credentials
- [ ] Add `isConfigured()` helper method
- [ ] Add `ensureConfigured()` validation method with helpful error messages
- [ ] Protect all public methods with `ensureConfigured()`
- [ ] Update types to allow null where needed (Buffer | null, etc.)
- [ ] Add warning logs when service initializes without configuration
- [ ] Add success logs when service initializes with configuration
- [ ] Write tests for both configured and unconfigured states
- [ ] Update `.env.example` with setup instructions
- [ ] Create setup guide documentation (e.g., `docs/guides/service-setup.md`)
- [ ] Update error messages to include:
  - [ ] Exact environment variables needed
  - [ ] Commands to generate keys/tokens
  - [ ] Link to setup documentation
- [ ] Add health check endpoint that reports configuration status
- [ ] Update deployment documentation for production configuration

## Common Pitfalls

### ❌ Pitfall 1: Using Non-Null Assertion Before Validation

```typescript
// ❌ Bad: Using key! before checking if null
encrypt(data: string): string {
  const cipher = crypto.createCipheriv(algorithm, this.key!, iv); // Crash if null!
  // ...
}
```

```typescript
// ✅ Good: Validate first, then use non-null assertion
encrypt(data: string): string {
  this.ensureConfigured(); // Throws if this.key is null
  const cipher = crypto.createCipheriv(algorithm, this.key!, iv); // Safe!
  // ...
}
```

### ❌ Pitfall 2: Vague Error Messages

```typescript
// ❌ Bad: Unhelpful error
private ensureConfigured(): void {
  if (!this.isConfigured()) {
    throw new Error('Not configured');
  }
}
```

```typescript
// ✅ Good: Helpful error with actionable steps
private ensureConfigured(): void {
  if (!this.isConfigured()) {
    throw new Error(
      'QuickBooks integration is not configured. ' +
      'Please set QUICKBOOKS_CLIENT_ID, QUICKBOOKS_CLIENT_SECRET, and QUICKBOOKS_REDIRECT_URI. ' +
      'See docs/guides/quickbooks-setup.md for setup instructions.'
    );
  }
}
```

### ❌ Pitfall 3: Forgetting to Protect All Public Methods

```typescript
// ❌ Bad: Some methods protected, some not
class ServiceAdapter {
  getAuthUrl(): string {
    this.ensureConfigured(); // ✅ Protected
    // ...
  }

  makeApiCall(): Promise<Response> {
    // ❌ Missing ensureConfigured()!
    return this.httpClient.post(/* uses this.clientId which might be empty */);
  }
}
```

```typescript
// ✅ Good: All public methods protected
class ServiceAdapter {
  getAuthUrl(): string {
    this.ensureConfigured(); // ✅
    // ...
  }

  makeApiCall(): Promise<Response> {
    this.ensureConfigured(); // ✅
    return this.httpClient.post(/* ... */);
  }

  refreshToken(): Promise<Tokens> {
    this.ensureConfigured(); // ✅
    // ...
  }
}
```

## Related Patterns

- **Circuit Breaker Pattern** (#63) - Combine with optional integration for resilient external service calls
- **Anti-Corruption Layer** (#62) - Use together to isolate external API specifics
- **Configuration Management Pattern** - Centralized configuration validation
- **Health Check Pattern** - Report integration status in health endpoints

## Real-World Example

**QuickBooks Integration Implementation** (Sprint 11.1):

The QuickBooks OAuth integration in Catalyst PSA uses this pattern to allow developers to work on other features without setting up QuickBooks credentials. The API starts successfully, logs a warning about missing QB credentials, but throws helpful errors if users attempt to connect QuickBooks without configuration.

**Result**:

- ✅ 100% of developers can run the API without QB setup
- ✅ Clear errors guide developers to setup documentation
- ✅ Production deployments can enable QB when ready
- ✅ No security compromise - validation enforced before external calls

## References

- Sprint 11.1 Implementation: `docs/sprints/sprint-11-quickbooks-integration.md`
- QuickBooks Adapter: `apps/api/src/infrastructure/quickbooks/quickbooks.adapter.ts`
- Token Encryption Service: `apps/api/src/infrastructure/quickbooks/token-encryption.service.ts`
- Setup Guide: `docs/guides/quickbooks-setup.md`
