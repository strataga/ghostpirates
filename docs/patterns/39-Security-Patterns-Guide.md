# Security Patterns for API and Web Applications

## Overview

Security is not an afterthought - it's a foundational aspect of the WellOS application that must be integrated into every layer of the architecture. This guide provides comprehensive security patterns for both the NestJS backend (API) and Next.js frontend (Web) applications.

### Purpose

- **Protect User Data** - Ensure confidentiality, integrity, and availability of sensitive information
- **Prevent Common Attacks** - XSS, CSRF, SQL injection, authentication bypass
- **Ensure Compliance** - SOC 2, GDPR, data retention requirements
- **Build Trust** - Demonstrate security best practices to clients
- **Enable Auditing** - Track all security-relevant actions

### When to Apply Security Patterns

**Always**. Security patterns must be applied from day one and continuously maintained throughout the application lifecycle. Every feature, every endpoint, every component must consider security implications.

### Relationship to Other Patterns

Security patterns integrate with:

- **Hexagonal Architecture** - Security concerns in infrastructure layer
- **CQRS** - Authorization checks in command handlers
- **Repository Pattern** - Parameterized queries prevent SQL injection
- **DTO Pattern** - Input validation at boundaries
- **Domain Events** - Audit logging through event handlers
- **Observer Pattern** - Security event notifications
- **Soft Delete Pattern** - Data retention for compliance

---

## Backend Security Patterns (Rust + Axum)

### 1. Authentication & Authorization

#### 1.1 Password Hashing Pattern

**Problem**: Storing passwords in plain text or using weak hashing algorithms puts user credentials at risk.

**Solution**: Use bcrypt with cost factor 12 to hash passwords before storage.

**Implementation**:

```typescript
// apps/api/src/domain/user/hashed-password.vo.ts
import * as bcrypt from 'bcrypt';

/**
 * HashedPassword Value Object
 * Encapsulates password hashing and verification logic
 */
export class HashedPassword {
  private readonly hash: string;
  private static readonly SALT_ROUNDS = 12;

  private constructor(hash: string) {
    this.hash = hash;
  }

  /**
   * Create HashedPassword from plain text
   * Validates password strength before hashing
   */
  static async fromPlainText(password: string): Promise<HashedPassword> {
    // Validate password strength
    this.validateStrength(password);

    // Hash with bcrypt (cost 12)
    const hash = await bcrypt.hash(password, HashedPassword.SALT_ROUNDS);
    return new HashedPassword(hash);
  }

  /**
   * Create HashedPassword from existing hash
   * Used when loading from database
   */
  static fromHash(hash: string): HashedPassword {
    if (!hash) {
      throw new Error('Hash cannot be empty');
    }
    return new HashedPassword(hash);
  }

  /**
   * Verify plain text password against hash
   */
  async verify(plainPassword: string): Promise<boolean> {
    return bcrypt.compare(plainPassword, this.hash);
  }

  /**
   * Validate password strength requirements
   */
  private static validateStrength(password: string): void {
    if (password.length < 8) {
      throw new WeakPasswordException('Password must be at least 8 characters');
    }
    if (!/[A-Z]/.test(password)) {
      throw new WeakPasswordException('Password must contain an uppercase letter');
    }
    if (!/[a-z]/.test(password)) {
      throw new WeakPasswordException('Password must contain a lowercase letter');
    }
    if (!/[0-9]/.test(password)) {
      throw new WeakPasswordException('Password must contain a number');
    }
    if (!/[^A-Za-z0-9]/.test(password)) {
      throw new WeakPasswordException('Password must contain a special character');
    }
  }

  getHash(): string {
    return this.hash;
  }
}

// Custom exception
export class WeakPasswordException extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'WeakPasswordException';
  }
}
```

**Password Strength Requirements**:

- Minimum 8 characters
- At least 1 uppercase letter (A-Z)
- At least 1 lowercase letter (a-z)
- At least 1 number (0-9)
- At least 1 special character (!@#$%^&\*)

**Optional Enhancements**:

- Check against leaked password databases (HaveIBeenPwned API)
- Prevent common passwords ("password123", "qwerty")
- Prevent passwords containing username/email

---

#### 1.2 JWT Token Pattern (Access + Refresh)

**Problem**: Long-lived tokens increase security risk. If compromised, attacker has prolonged access.

**Solution**: Use short-lived access tokens (15 min) + long-lived refresh tokens (7 days).

**Access Token Implementation**:

```rust
// src/infrastructure/auth/jwt_service.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtPayload {
    pub sub: String,      // User ID
    pub email: String,
    pub role: String,
    pub r#type: String,   // 'access' or 'refresh'
    pub iat: i64,
    pub exp: i64,
}

pub struct JwtService {
    secret: String,
}

impl JwtService {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    /// Sign access token (short-lived)
    pub fn sign_access_token(&self, user_id: &str, email: &str, role: &str) -> Result<String> {
        let now = Utc::now();
        let payload = JwtPayload {
            sub: user_id.to_string(),
            email: email.to_string(),
            role: role.to_string(),
            r#type: "access".to_string(),
            iat: now.timestamp(),
            exp: (now + Duration::minutes(15)).timestamp(), // Short-lived
        };

        encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(self.secret.as_ref()),
        )
        .map_err(|e| anyhow!("Failed to sign token: {}", e))
    }

    /// Verify access token
    pub fn verify_access_token(&self, token: &str) -> Result<JwtPayload> {
        let validation = Validation::default();

        let token_data = decode::<JwtPayload>(
            token,
            &DecodingKey::from_secret(self.secret.as_ref()),
            &validation,
        )
        .map_err(|_| anyhow!("Invalid or expired access token"))?;

        // Verify token type
        if token_data.claims.r#type != "access" {
            return Err(anyhow!("Invalid token type"));
        }

        Ok(token_data.claims)
    }
}
```

**Refresh Token Implementation**:

```typescript
// apps/api/src/domain/auth/refresh-token.entity.ts
import * as crypto from 'crypto';
import { AggregateRoot } from '@/domain/shared/aggregate-root';
import { RefreshTokenId } from './refresh-token-id.vo';
import { UserId } from '@/domain/user/user-id.vo';
import { RefreshTokenRevokedEvent } from './events/refresh-token-revoked.event';

/**
 * RefreshToken Aggregate Root
 * Long-lived token stored in database for token rotation
 */
export class RefreshToken extends AggregateRoot {
  private constructor(
    public readonly id: RefreshTokenId,
    public readonly userId: UserId,
    public readonly token: string,
    public readonly expiresAt: Date,
    public revokedAt: Date | null,
    public readonly createdAt: Date,
  ) {
    super();
  }

  /**
   * Create new refresh token
   * Generates cryptographically secure random token
   */
  static create(userId: UserId): RefreshToken {
    const token = crypto.randomBytes(64).toString('hex');
    const expiresAt = new Date();
    expiresAt.setDate(expiresAt.getDate() + 7); // 7 days

    return new RefreshToken(RefreshTokenId.create(), userId, token, expiresAt, null, new Date());
  }

  /**
   * Check if token is valid (not expired, not revoked)
   */
  isValid(): boolean {
    return !this.revokedAt && this.expiresAt > new Date();
  }

  /**
   * Revoke token (e.g., on logout)
   */
  revoke(): void {
    this.revokedAt = new Date();
    this.addDomainEvent(new RefreshTokenRevokedEvent(this.id, this.userId));
  }

  /**
   * Check if token is expired
   */
  isExpired(): boolean {
    return this.expiresAt <= new Date();
  }
}
```

**Controller Implementation**:

```rust
// src/presentation/auth/routes.rs
use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use time::Duration;

#[derive(Deserialize)]
pub struct LoginDto {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponseDto {
    access_token: String,
    user: UserDto,
}

/// Login endpoint
/// Returns access token in response body
/// Sets refresh token in httpOnly cookie
pub async fn login(
    State(app_state): State<Arc<AppState>>,
    Json(dto): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
    let result = app_state
        .command_bus
        .execute(LoginCommand::new(dto.email, dto.password))
        .await?;

    // Create refresh token cookie
    let cookie = Cookie::build(("refreshToken", result.refresh_token))
        .http_only(true)  // Prevents JavaScript access (XSS protection)
        .secure(cfg!(not(debug_assertions)))  // HTTPS only in production
        .same_site(SameSite::Strict)  // CSRF protection
        .max_age(Duration::days(7))
        .path("/")
        .finish();

    let mut response = Json(LoginResponseDto {
        access_token: result.access_token,
        user: result.user,
    }).into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}

/// Refresh access token endpoint
/// Reads refresh token from cookie
/// Returns new access token
pub async fn refresh(
    State(app_state): State<Arc<AppState>>,
    cookies: Cookies,
) -> Result<Json<RefreshResponse>, AppError> {
    let refresh_token = cookies
        .get("refreshToken")
        .ok_or_else(|| AppError::Unauthorized("No refresh token provided".into()))?
        .value();

    let result = app_state
        .command_bus
        .execute(RefreshAccessTokenCommand::new(refresh_token.to_string()))
        .await?;

    Ok(Json(RefreshResponse {
        access_token: result.access_token,
    }))
}

/// Logout endpoint
/// Revokes refresh token
/// Clears cookie
pub async fn logout(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
    if let Some(refresh_token) = cookies.get("refreshToken") {
        app_state
            .command_bus
            .execute(LogoutCommand::new(user.user_id, refresh_token.value().to_string()))
            .await?;
    }

    // Clear refresh token cookie
    let cookie = Cookie::build(("refreshToken", ""))
        .max_age(Duration::seconds(0))
        .finish();

    let mut response = Json(LogoutResponse {
        message: "Logged out successfully".into(),
    }).into_response();

    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}
```

**Key Security Features**:

1. **Access tokens are short-lived (15 min)** - Limits damage if compromised
2. **Refresh tokens in httpOnly cookies** - Frontend JavaScript cannot access
3. **Refresh tokens stored in database** - Can be revoked server-side
4. **SameSite=strict cookie** - CSRF protection
5. **Secure flag in production** - HTTPS only

---

#### 1.3 JWT Strategy and Guard Pattern

**Problem**: Need to validate JWT tokens on protected endpoints.

**Solution**: Use Passport JWT strategy with NestJS guards.

**JWT Strategy**:

```rust
// src/infrastructure/auth/jwt_middleware.rs
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub role: String,
}

/// JWT authentication middleware
/// Extracts and validates JWT from Authorization header
pub async fn jwt_auth_middleware(
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let secret = std::env::var("JWT_SECRET")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract token from Authorization header
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Decode and validate token
    let token_data = decode::<JwtPayload>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Attach user to request extensions
    let user = AuthUser {
        user_id: token_data.claims.sub,
        email: token_data.claims.email,
        role: token_data.claims.role,
    };

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
```

**JWT Auth Guard**:

```rust
// src/infrastructure/auth/guards.rs
use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
};

/// JWT Authentication Guard
/// Protects routes by requiring valid JWT access token
pub async fn jwt_auth_guard(
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Check if user is authenticated (added by jwt_auth_middleware)
    let user = req.extensions().get::<AuthUser>();

    if user.is_none() {
        return Err((StatusCode::UNAUTHORIZED, "Invalid or expired token"));
    }

    Ok(next.run(req).await)
}

/// Public route guard - allows unauthenticated access
pub async fn public_route_guard(
    req: Request,
    next: Next,
) -> Response {
    // Always allow access
    next.run(req).await
}
```

**Public Decorator** (for public endpoints):

```rust
// src/presentation/routes.rs
use axum::{
    Router,
    routing::{get, post},
    middleware,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        // Protected routes - require authentication
        .route("/users/profile", get(get_profile))
        .layer(middleware::from_fn(jwt_auth_guard))

        // Public routes - no authentication required
        .route("/users/public-info", get(get_public_info))

        .with_state(app_state)
}

async fn get_profile(
    Extension(user): Extension<AuthUser>,
) -> Json<UserProfile> {
    // Protected route - requires JWT
    Json(UserProfile { /* ... */ })
}

async fn get_public_info() -> Json<PublicInfo> {
    Json(PublicInfo {
        message: "This is public".into(),
    })
}
```

---

#### 1.4 RBAC with CASL Pattern

**Problem**: Need fine-grained authorization beyond simple role checks.

**Solution**: Use CASL for attribute-based access control.

**Abilities Factory**:

```rust
// src/infrastructure/authorization/abilities.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Manage,  // Wildcard for any action
    Create,
    Read,
    Update,
    Delete,
    Approve,
    Submit,
    Export,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Subject {
    User,
    TimeEntry,
    Project,
    Invoice,
    Client,
    All,
}

pub struct AbilityFactory;

impl AbilityFactory {
    pub fn create_for_user(user_role: &str, user_id: &str) -> Abilities {
        let mut abilities = Abilities::new();

        match user_role {
            // Admin - can do anything
            "admin" => {
                abilities.allow(Action::Manage, Subject::All, None);
            }

            // Manager - can manage projects, clients, invoices, and approve time entries
            "manager" => {
                abilities.allow(Action::Read, Subject::All, None);
                abilities.allow(Action::Manage, Subject::Project, None);
                abilities.allow(Action::Manage, Subject::Client, None);
                abilities.allow(Action::Manage, Subject::Invoice, None);
                abilities.allow(Action::Approve, Subject::TimeEntry, None);
                abilities.allow(Action::Export, Subject::TimeEntry, None);

                // Cannot delete users
                abilities.deny(Action::Delete, Subject::User, None);
            }

            // Consultant - can only manage own time entries
            "consultant" => {
                abilities.allow(Action::Read, Subject::Project, None);
                abilities.allow(Action::Read, Subject::Client, None);

                // Own time entries only
                abilities.allow(Action::Create, Subject::TimeEntry, None);
                abilities.allow(
                    Action::Read,
                    Subject::TimeEntry,
                    Some(HashMap::from([("userId", user_id)])),
                );
                abilities.allow(
                    Action::Update,
                    Subject::TimeEntry,
                    Some(HashMap::from([
                        ("userId", user_id),
                        ("status", "draft"),
                    ])),
                );
                abilities.allow(
                    Action::Submit,
                    Subject::TimeEntry,
                    Some(HashMap::from([("userId", user_id)])),
                );

                // Cannot approve or delete time entries
                abilities.deny(Action::Approve, Subject::TimeEntry, None);
                abilities.deny(Action::Delete, Subject::TimeEntry, None);
            }

            _ => {
                // Default: no permissions
            }
        }

        abilities
    }
}

pub struct Abilities {
    rules: Vec<(bool, Action, Subject, Option<HashMap<&'static str, &'static str>>)>,
}

impl Abilities {
    fn new() -> Self {
        Self { rules: Vec::new() }
    }

    fn allow(&mut self, action: Action, subject: Subject, conditions: Option<HashMap<&'static str, &'static str>>) {
        self.rules.push((true, action, subject, conditions));
    }

    fn deny(&mut self, action: Action, subject: Subject, conditions: Option<HashMap<&'static str, &'static str>>) {
        self.rules.push((false, action, subject, conditions));
    }

    pub fn can(&self, action: Action, subject: Subject) -> bool {
        // Check rules in order
        for (allowed, rule_action, rule_subject, _) in &self.rules {
            if (*rule_action == action || *rule_action == Action::Manage) &&
               (*rule_subject == subject || *rule_subject == Subject::All) {
                return *allowed;
            }
        }
        false
    }
}
```

**Permissions Guard**:

```rust
// src/infrastructure/authorization/guards.rs
use axum::{
    extract::{Path, State, Request},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

/// Guard to check permissions
pub async fn permissions_guard(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<AuthUser>,
    Path(params): Path<HashMap<String, String>>,
    required_action: Action,
    required_subject: Subject,
    req: Request,
    next: Next,
) -> Result<Response, impl IntoResponse> {
    // Create abilities for user
    let abilities = AbilityFactory::create_for_user(&user.role, &user.user_id);

    // Check permission
    if !abilities.can(required_action, required_subject) {
        return Err((StatusCode::FORBIDDEN, "Insufficient permissions"));
    }

    Ok(next.run(req).await)
}

/// Macro to easily require permissions on routes
macro_rules! require_permissions {
    ($action:expr, $subject:expr) => {
        middleware::from_fn_with_state(
            app_state.clone(),
            move |state, user, path, req, next| {
                permissions_guard(state, user, path, $action, $subject, req, next)
            },
        )
    };
}

// Usage in routes
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/time-entries", post(create_time_entry))
        .layer(require_permissions!(Action::Create, Subject::TimeEntry))

        .route("/time-entries/:id", patch(update_time_entry))
        .layer(require_permissions!(Action::Update, Subject::TimeEntry))

        .route("/time-entries/:id/approve", post(approve_time_entry))
        .layer(require_permissions!(Action::Approve, Subject::TimeEntry))

        .with_state(app_state)
}
```

**Permissions Decorator**:

```typescript
// apps/api/src/shared/decorators/permissions.decorator.ts
import { Action } from '@/authorization/abilities.factory';

/**
 * Decorator to require specific permissions
 */
export const RequirePermissions = (action: Action, subject: any, conditions?: any) =>
  SetMetadata('permissions', [action, subject, conditions]);
```

**Usage**:

```typescript
@Controller('time-entries')
@UseGuards(JwtAuthGuard, PermissionsGuard)
export class TimeEntryController {
  @Post()
  @RequirePermissions(Action.Create, TimeEntry)
  async create(@Body() dto: CreateTimeEntryDto) {
    // Only users with 'create' permission on TimeEntry can execute
  }

  @Patch(':id')
  @RequirePermissions(Action.Update, TimeEntry)
  async update(@Param('id') id: string, @Body() dto: UpdateTimeEntryDto) {
    // Checks if user can update THIS specific time entry
    // (e.g., only own entries, only draft status)
  }

  @Post(':id/approve')
  @RequirePermissions(Action.Approve, TimeEntry)
  async approve(@Param('id') id: string) {
    // Only managers/admins can approve
  }
}
```

---

### 2. Input Validation & Sanitization

#### 2.1 DTO Validation Pattern

**Problem**: User input can contain malicious data or invalid formats.

**Solution**: Use class-validator and class-transformer for automatic validation.

**Implementation**:

```rust
// src/presentation/user/dto.rs
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(email(message = "Invalid email format"))]
    #[serde(deserialize_with = "deserialize_email")]
    pub email: String,

    #[validate(
        length(min = 8, max = 100, message = "Password must be between 8 and 100 characters"),
        custom = "validate_password_strength"
    )]
    pub password: String,

    #[validate(
        length(min = 2, max = 100, message = "Name must be between 2 and 100 characters")
    )]
    #[serde(deserialize_with = "deserialize_trimmed")]
    pub name: String,

    #[validate(phone)]
    pub phone_number: Option<String>,
}

// Sanitize email: lowercase and trim
fn deserialize_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_lowercase())
}

// Trim whitespace
fn deserialize_trimmed<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.trim().to_string())
}

// Custom password strength validator
fn validate_password_strength(password: &str) -> Result<(), ValidationError> {
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| "@$!%*?&".contains(c));

    if !has_lowercase || !has_uppercase || !has_digit || !has_special {
        return Err(ValidationError::new(
            "Password must contain uppercase, lowercase, number, and special character"
        ));
    }

    Ok(())
}
```

**Global Validation Pipe**:

```rust
// src/main.rs
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use validator::Validate;

// Global validation is handled through the Validate derive macro
// and custom extractors

/// Custom JSON extractor that validates input
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // Validate the deserialized data
        value
            .validate()
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Validation error: {}", e)))?;

        Ok(ValidatedJson(value))
    }
}

#[tokio::main]
async fn main() {
    let app = create_router(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

**Custom Validators**:

```typescript
// apps/api/src/shared/validators/is-api-number.validator.ts
import {
  registerDecorator,
  ValidationOptions,
  ValidatorConstraint,
  ValidatorConstraintInterface,
} from 'class-validator';

/**
 * Custom validator for API numbers (oil & gas well identifiers)
 */
@ValidatorConstraint({ async: false })
export class IsApiNumberConstraint implements ValidatorConstraintInterface {
  validate(apiNumber: any) {
    // API number format: XX-XXX-XXXXX (14 digits with hyphens)
    return typeof apiNumber === 'string' && /^\d{2}-\d{3}-\d{5}$/.test(apiNumber);
  }

  defaultMessage() {
    return 'API number must be in format XX-XXX-XXXXX';
  }
}

export function IsApiNumber(validationOptions?: ValidationOptions) {
  return function (object: Object, propertyName: string) {
    registerDecorator({
      target: object.constructor,
      propertyName: propertyName,
      options: validationOptions,
      constraints: [],
      validator: IsApiNumberConstraint,
    });
  };
}

// Usage
export class CreateWellDto {
  @IsApiNumber()
  apiNumber: string;
}
```

---

#### 2.2 SQL Injection Prevention Pattern

**Problem**: String concatenation in queries allows SQL injection attacks.

**Solution**: Always use parameterized queries with SQLx.

```typescript
// ✅ CORRECT - Parameterized (Drizzle auto-parameterizes)
import { eq, and, like } from 'drizzle-orm';
import { db } from '@/infrastructure/database/connection';
import { users } from '@/infrastructure/database/schema/user.schema';

export class UserRepository {
  async findByEmail(email: string): Promise<User | null> {
    // Drizzle automatically parameterizes
    const result = await db
      .select()
      .from(users)
      .where(eq(users.email, email)) // Safe - parameterized
      .limit(1);

    return result[0] ? this.toDomain(result[0]) : null;
  }

  async searchByName(searchTerm: string): Promise<User[]> {
    // Safe - parameterized
    const results = await db
      .select()
      .from(users)
      .where(like(users.name, `%${searchTerm}%`)); // Drizzle escapes

    return results.map(r => this.toDomain(r));
  }
}

// ❌ NEVER DO THIS - Raw SQL with concatenation
async findByEmailUnsafe(email: string): Promise<User | null> {
  // DANGEROUS - SQL injection vulnerability
  const result = await db.execute(
    `SELECT * FROM users WHERE email = '${email}'` // NEVER DO THIS
  );
  return result[0];
}

// ⚠️ IF YOU MUST USE RAW SQL - Use parameters
async findByEmailRaw(email: string): Promise<User | null> {
  // Acceptable - uses parameters
  const result = await db.execute(
    'SELECT * FROM users WHERE email = $1',
    [email] // Parameterized
  );
  return result[0];
}
```

---

#### 2.3 XSS Prevention Pattern (Backend)

**Problem**: User-generated content can contain malicious scripts.

**Solution**: Sanitize output, set security headers, validate input.

**Security Headers with Helmet**:

```rust
// src/middleware/security_headers.rs
use axum::{
    http::{HeaderValue, Request},
    middleware::Next,
    response::Response,
};

/// Security headers middleware
pub async fn security_headers_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    let mut response = next.run(req).await;

    let headers = response.headers_mut();

    // Content Security Policy
    headers.insert(
        "content-security-policy",
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data: https:; \
             connect-src 'self'; \
             font-src 'self'; \
             object-src 'none'; \
             media-src 'self'; \
             frame-src 'none';"
        ),
    );

    // HTTP Strict Transport Security (HSTS)
    headers.insert(
        "strict-transport-security",
        HeaderValue::from_static("max-age=31536000; includeSubDomains; preload"),
    );

    // X-Content-Type-Options
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );

    // X-XSS-Protection
    headers.insert(
        "x-xss-protection",
        HeaderValue::from_static("1; mode=block"),
    );

    // Referrer Policy
    headers.insert(
        "referrer-policy",
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    response
}

// Apply in main.rs
pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .merge(auth_routes())
        .merge(user_routes())
        .layer(middleware::from_fn(security_headers_middleware))
        .with_state(app_state)
}
```

**Sanitize HTML in DTOs**:

```typescript
import { IsString, MaxLength } from 'class-validator';
import { Transform } from 'class-transformer';
import * as sanitizeHtml from 'sanitize-html';

export class CreateCommentDto {
  @IsString()
  @MaxLength(1000)
  @Transform(({ value }) =>
    sanitizeHtml(value, {
      allowedTags: ['b', 'i', 'em', 'strong', 'a', 'p'],
      allowedAttributes: {
        a: ['href'],
      },
    }),
  )
  content: string;
}
```

---

### 3. CSRF Protection

#### 3.1 SameSite Cookie Pattern

**Problem**: Cross-site request forgery allows attackers to perform actions on behalf of authenticated users.

**Solution**: Use SameSite cookies for automatic CSRF protection.

```typescript
// apps/api/src/presentation/auth/auth.controller.ts
private setRefreshTokenCookie(response: Response, token: string): void {
  response.cookie('refreshToken', token, {
    httpOnly: true,
    secure: process.env.NODE_ENV === 'production',
    sameSite: 'strict', // CSRF protection - only send cookie for same-site requests
    maxAge: 7 * 24 * 60 * 60 * 1000,
    path: '/',
  });
}
```

**SameSite Options**:

- `strict` - Most secure, cookie only sent for same-site requests (recommended)
- `lax` - Cookie sent for top-level navigation (GET requests)
- `none` - Cookie sent for all requests (requires `secure: true`)

---

#### 3.2 CSRF Token Pattern (Alternative)

**Problem**: Some scenarios require additional CSRF protection beyond SameSite cookies.

**Solution**: Use CSRF tokens for state-changing operations.

```rust
// src/middleware/csrf.rs
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use std::sync::Arc;

/// CSRF protection middleware (if not using SameSite strict)
pub async fn csrf_middleware<B>(
    State(app_state): State<Arc<AppState>>,
    jar: CookieJar,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let method = req.method();

    // Skip CSRF check for safe methods
    if matches!(method.as_str(), "GET" | "HEAD" | "OPTIONS") {
        return Ok(next.run(req).await);
    }

    // Verify CSRF token for state-changing requests
    let csrf_token = req
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::FORBIDDEN)?;

    let expected_token = jar
        .get("csrf_token")
        .map(|c| c.value())
        .ok_or(StatusCode::FORBIDDEN)?;

    if csrf_token != expected_token {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}
```

**CSRF Guard**:

```typescript
// apps/api/src/shared/guards/csrf.guard.ts
export class CsrfGuard {
  constructor(private reflector: any) {}

  canActivate(context: ExecutionContext): boolean {
    const request = context.switchToHttp().getRequest();

    // Skip for GET, HEAD, OPTIONS
    if (['GET', 'HEAD', 'OPTIONS'].includes(request.method)) {
      return true;
    }

    // Verify CSRF token
    const csrfToken = request.headers['x-csrf-token'];
    const expectedToken = request.csrfToken();

    if (csrfToken !== expectedToken) {
      throw new ForbiddenException('Invalid CSRF token');
    }

    return true;
  }
}
```

---

### 4. Rate Limiting & Brute Force Prevention

#### 4.1 Throttler Pattern

**Problem**: Brute force attacks can compromise accounts.

**Solution**: Implement rate limiting on sensitive endpoints.

```rust
// src/middleware/rate_limit.rs
use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    ttl: Duration,
    limit: usize,
}

impl RateLimiter {
    pub fn new(ttl_seconds: u64, limit: usize) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(ttl_seconds),
            limit,
        }
    }

    pub async fn check(&self, key: &str) -> bool {
        let mut requests = self.requests.write().await;
        let now = Instant::now();

        // Get or create entry for this key
        let times = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // Remove expired timestamps
        times.retain(|&time| now.duration_since(time) < self.ttl);

        // Check if limit exceeded
        if times.len() >= self.limit {
            return false;
        }

        // Add current request
        times.push(now);
        true
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware<B>(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // Use IP address as key (or user ID for authenticated routes)
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !limiter.check(key).await {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}

// Apply in main.rs with custom limits per route
pub fn create_router(app_state: Arc<AppState>) -> Router {
    let global_limiter = Arc::new(RateLimiter::new(60, 10)); // 10 requests per 60 seconds
    let auth_limiter = Arc::new(RateLimiter::new(60, 5));    // 5 requests per 60 seconds

    Router::new()
        .route("/auth/login", post(login))
        .layer(middleware::from_fn_with_state(
            auth_limiter,
            rate_limit_middleware,
        ))

        .route("/api/users", get(get_users))
        .layer(middleware::from_fn_with_state(
            global_limiter,
            rate_limit_middleware,
        ))

        .with_state(app_state)
}
```

**Custom Rate Limits per Endpoint**:

```rust
// src/presentation/auth/routes.rs
// Rate limiting applied via middleware layers

pub fn auth_routes(app_state: Arc<AppState>) -> Router {
    let login_limiter = Arc::new(RateLimiter::new(60, 5));        // 5 attempts per minute
    let register_limiter = Arc::new(RateLimiter::new(60, 3));     // 3 attempts per minute
    let reset_limiter = Arc::new(RateLimiter::new(900, 3));       // 3 attempts per 15 minutes

    Router::new()
        .route("/login", post(login))
        .layer(middleware::from_fn_with_state(
            login_limiter,
            rate_limit_middleware,
        ))

        .route("/register", post(register))
        .layer(middleware::from_fn_with_state(
            register_limiter,
            rate_limit_middleware,
        ))

        .route("/password-reset/request", post(request_password_reset))
        .layer(middleware::from_fn_with_state(
            reset_limiter,
            rate_limit_middleware,
        ))

        .with_state(app_state)
}
```

**Distributed Rate Limiting with Redis**:

```rust
// src/infrastructure/rate_limit/redis_limiter.rs
use redis::AsyncCommands;
use std::sync::Arc;

pub struct RedisRateLimiter {
    client: Arc<redis::Client>,
    ttl: u64,
    limit: usize,
}

impl RedisRateLimiter {
    pub fn new(redis_url: &str, ttl_seconds: u64, limit: usize) -> Result<Self, redis::RedisError> {
        let client = redis::Client::open(redis_url)?;
        Ok(Self {
            client: Arc::new(client),
            ttl: ttl_seconds,
            limit,
        })
    }

    pub async fn check(&self, key: &str) -> Result<bool, redis::RedisError> {
        let mut conn = self.client.get_async_connection().await?;

        // Increment counter
        let count: usize = conn.incr(key, 1).await?;

        // Set expiry on first increment
        if count == 1 {
            conn.expire(key, self.ttl as usize).await?;
        }

        // Check if limit exceeded
        Ok(count <= self.limit)
    }
}

// Use in middleware
pub async fn redis_rate_limit_middleware<B>(
    State(limiter): State<Arc<RedisRateLimiter>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if !limiter.check(key).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)? {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
```

---

#### 4.2 Account Lockout Pattern

**Problem**: Multiple failed login attempts indicate brute force attack.

**Solution**: Lock account after threshold of failed attempts.

```typescript
// apps/api/src/domain/user/user.entity.ts
export class User extends AggregateRoot {
  private failedLoginAttempts: number = 0;
  private lockedUntil: Date | null = null;
  private static readonly MAX_FAILED_ATTEMPTS = 5;
  private static readonly LOCKOUT_DURATION_MINUTES = 15;

  /**
   * Verify password and handle failed attempts
   */
  async verifyPassword(plainPassword: string): Promise<boolean> {
    // Check if account is locked
    if (this.isLocked()) {
      throw new AccountLockedException(`Account locked until ${this.lockedUntil.toISOString()}`);
    }

    const isValid = await this.passwordHash.verify(plainPassword);

    if (!isValid) {
      this.handleFailedLogin();
      return false;
    }

    // Reset failed attempts on successful login
    this.failedLoginAttempts = 0;
    this.lockedUntil = null;

    return true;
  }

  /**
   * Handle failed login attempt
   */
  private handleFailedLogin(): void {
    this.failedLoginAttempts++;

    // Lock account after max attempts
    if (this.failedLoginAttempts >= User.MAX_FAILED_ATTEMPTS) {
      this.lockAccount();
      this.addDomainEvent(new UserAccountLockedEvent(this.id));
    }
  }

  /**
   * Lock account for specified duration
   */
  private lockAccount(): void {
    this.lockedUntil = new Date();
    this.lockedUntil.setMinutes(this.lockedUntil.getMinutes() + User.LOCKOUT_DURATION_MINUTES);
  }

  /**
   * Check if account is currently locked
   */
  isLocked(): boolean {
    return this.lockedUntil !== null && this.lockedUntil > new Date();
  }

  /**
   * Manually unlock account (admin action)
   */
  unlock(adminId: UserId): void {
    this.lockedUntil = null;
    this.failedLoginAttempts = 0;
    this.addDomainEvent(new UserAccountUnlockedEvent(this.id, adminId));
  }
}

// Custom exception
export class AccountLockedException extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'AccountLockedException';
  }
}
```

---

### 5. Audit Logging Pattern

#### 5.1 Audit Log Entity

**Problem**: Need immutable record of all security-relevant actions for compliance.

**Solution**: Create audit log entries for all mutations.

```typescript
// apps/api/src/domain/audit-log/audit-log.entity.ts
import { AggregateRoot } from '@/domain/shared/aggregate-root';
import { AuditLogId } from './audit-log-id.vo';
import { UserId } from '@/domain/user/user-id.vo';

export class AuditLog extends AggregateRoot {
  private constructor(
    public readonly id: AuditLogId,
    public readonly userId: UserId | null,
    public readonly action: string,
    public readonly entityType: string | null,
    public readonly entityId: string | null,
    public readonly changes: Record<string, any> | null,
    public readonly ipAddress: string | null,
    public readonly userAgent: string | null,
    public readonly timestamp: Date,
  ) {
    super();
  }

  /**
   * Create audit log entry
   */
  static create(params: {
    userId?: UserId;
    action: string;
    entityType?: string;
    entityId?: string;
    changes?: Record<string, any>;
    ipAddress?: string;
    userAgent?: string;
  }): AuditLog {
    return new AuditLog(
      AuditLogId.create(),
      params.userId ?? null,
      params.action,
      params.entityType ?? null,
      params.entityId ?? null,
      params.changes ?? null,
      params.ipAddress ?? null,
      params.userAgent ?? null,
      new Date(),
    );
  }
}
```

---

#### 5.2 Audit Log Interceptor

**Problem**: Manually logging every action is error-prone.

**Solution**: Use interceptor to automatically log all mutations.

```rust
// src/middleware/audit_log.rs
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::Value;
use std::sync::Arc;

/// Audit log middleware
pub async fn audit_log_middleware(
    State(app_state): State<Arc<AppState>>,
    Extension(user): Extension<Option<AuthUser>>,
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = req.method().clone();
    let uri = req.uri().clone();

    // Only log mutations (POST, PATCH, DELETE, PUT)
    if !matches!(method.as_str(), "POST" | "PATCH" | "DELETE" | "PUT") {
        return Ok(next.run(req).await);
    }

    // Extract request info
    let ip_address = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let user_agent = req
        .headers()
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // Execute request
    let response = next.run(req).await;
    let status = response.status();

    // Log audit entry
    let audit_entry = AuditLogEntry {
        user_id: user.as_ref().map(|u| u.user_id.clone()),
        action: format!("{} {}", method, uri.path()),
        entity_type: None, // Can be extracted from route metadata
        entity_id: None,   // Can be extracted from path params
        changes: None,     // Can parse request/response bodies if needed
        ip_address,
        user_agent,
        status_code: status.as_u16(),
        timestamp: chrono::Utc::now(),
    };

    // Async log without blocking response
    tokio::spawn(async move {
        if let Err(e) = app_state.audit_log_service.log(audit_entry).await {
            eprintln!("Failed to log audit entry: {}", e);
        }
    });

    Ok(response)
}

fn sanitize_body(mut body: Value) -> Value {
    if let Value::Object(ref mut map) = body {
        // Remove sensitive fields
        map.remove("password");
        map.remove("passwordConfirm");
        map.remove("token");
        map.remove("refreshToken");
    }
    body
}

fn sanitize_response(mut response: Value) -> Value {
    if let Value::Object(ref mut map) = response {
        // Remove sensitive fields
        map.remove("passwordHash");
        map.remove("refreshToken");
    }
    response
}
```

**Entity Type Decorator**:

```typescript
// apps/api/src/shared/decorators/entity-type.decorator.ts
export const EntityType = (type: string) => ({ entityType: type });
```

**Usage**:

```typescript
@Controller('users')
@UseInterceptors(AuditLogInterceptor)
export class UserController {
  @Patch(':id')
  @EntityType('User')
  async update(@Param('id') id: string, @Body() dto: UpdateUserDto) {
    // Automatically logged to audit trail
  }
}
```

---

### 6. Secret Management Pattern

#### 6.1 Environment Variables

**Problem**: Hardcoding secrets in code exposes them in version control.

**Solution**: Use environment variables with validation.

```rust
// src/infrastructure/config/env.rs
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_environment")]
    pub environment: Environment,

    pub port: u16,

    #[serde(deserialize_with = "validate_min_length_32")]
    pub jwt_secret: String,

    pub database_url: String,

    pub cors_origin: String,

    #[serde(deserialize_with = "validate_min_length_32")]
    pub encryption_key: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
    Test,
}

fn default_environment() -> Environment {
    Environment::Development
}

fn validate_min_length_32<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.len() < 32 {
        return Err(serde::de::Error::custom(
            "Value must be at least 32 characters",
        ));
    }
    Ok(s)
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let config_file = format!(".env.{}", environment);

        config::Config::builder()
            .add_source(config::File::with_name(&config_file).required(false))
            .add_source(config::Environment::default().separator("__"))
            .build()?
            .try_deserialize()
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.jwt_secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters".into());
        }

        if self.encryption_key.len() < 32 {
            return Err("ENCRYPTION_KEY must be at least 32 characters".into());
        }

        if self.database_url.is_empty() {
            return Err("DATABASE_URL is required".into());
        }

        Ok(())
    }
}

// Load and validate config on startup
pub fn load_config() -> Config {
    let config = Config::from_env()
        .expect("Failed to load configuration");

    config.validate()
        .expect("Configuration validation failed");

    config
}
```

**.env.example**:

```bash
# Application
NODE_ENV=development
PORT=3001

# Database
DATABASE_URL=postgresql://user:password@localhost:5432/wellos

# JWT
JWT_SECRET=your-super-secret-jwt-key-min-32-chars
JWT_REFRESH_SECRET=your-refresh-token-secret-min-32-chars

# CORS
CORS_ORIGIN=http://localhost:4000

# Encryption
ENCRYPTION_KEY=your-encryption-key-min-32-chars

# External Services
QUICKBOOKS_CLIENT_ID=
QUICKBOOKS_CLIENT_SECRET=
STRIPE_SECRET_KEY=
```

**Generate Secure Secrets**:

```bash
# Generate JWT secret
node -e "console.log(require('crypto').randomBytes(64).toString('hex'))"
```

---

#### 6.2 Secrets Manager Pattern (Production)

**Problem**: Environment variables in production can be compromised.

**Solution**: Use secrets manager (AWS Secrets Manager, Vault).

```rust
// src/infrastructure/secrets/secrets_service.rs
use aws_sdk_secretsmanager::Client;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use serde_json::Value;

pub struct SecretsService {
    client: Client,
    cache: Arc<RwLock<HashMap<String, CachedSecret>>>,
    cache_ttl: Duration,
}

struct CachedSecret {
    value: Value,
    timestamp: SystemTime,
}

impl SecretsService {
    pub async fn new(region: &str) -> Self {
        let config = aws_config::from_env()
            .region(aws_sdk_secretsmanager::Region::new(region.to_string()))
            .load()
            .await;

        let client = Client::new(&config);

        Self {
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: Duration::from_secs(5 * 60), // 5 minutes
        }
    }

    /// Get secret from AWS Secrets Manager
    /// Caches result for 5 minutes
    pub async fn get_secret(&self, secret_name: &str) -> Result<Value, anyhow::Error> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(secret_name) {
                if cached.timestamp.elapsed()? < self.cache_ttl {
                    return Ok(cached.value.clone());
                }
            }
        }

        // Fetch from Secrets Manager
        let response = self.client
            .get_secret_value()
            .secret_id(secret_name)
            .send()
            .await?;

        let secret_string = response.secret_string()
            .ok_or_else(|| anyhow::anyhow!("Secret has no string value"))?;

        let secret: Value = serde_json::from_str(secret_string)?;

        // Cache result
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                secret_name.to_string(),
                CachedSecret {
                    value: secret.clone(),
                    timestamp: SystemTime::now(),
                },
            );
        }

        Ok(secret)
    }

    /// Clear cache (for secret rotation)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
```

**Usage**:

```typescript
@Injectable()
export class JwtService {
  constructor(
    private readonly secretsService: SecretsService,
    private readonly configService: ConfigService,
  ) {}

  async signAccessToken(user: User): Promise<string> {
    let secret: string;

    if (process.env.NODE_ENV === 'production') {
      // Use Secrets Manager in production
      const secrets = await this.secretsService.getSecret('wellos/jwt');
      secret = secrets.JWT_SECRET;
    } else {
      // Use environment variable in development
      secret = this.configService.get<string>('JWT_SECRET');
    }

    return this.jwtService.sign(payload, { secret, expiresIn: '15m' });
  }
}
```

---

### 7. Error Handling Pattern

#### 7.1 Secure Error Messages

**Problem**: Detailed error messages expose implementation details to attackers.

**Solution**: Generic error messages to users, detailed logs for developers.

```rust
// src/infrastructure/errors/handler.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    BadRequest(String),
    InternalError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    status_code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stack: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };

        // Detailed error for logging
        tracing::error!(
            error = ?self,
            message = %message,
            status_code = status.as_u16(),
            "Error occurred"
        );

        // Get error response based on environment
        let error_response = get_error_response(&self, status);

        (status, Json(error_response)).into_response()
    }
}

fn get_error_response(error: &AppError, status: StatusCode) -> ErrorResponse {
    let is_production = cfg!(not(debug_assertions));

    if is_production {
        // In production, never expose stack traces or internal details
        ErrorResponse {
            status_code: status.as_u16(),
            message: match error {
                AppError::InternalError(_) => "An unexpected error occurred".into(),
                _ => error.to_string(),
            },
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            stack: None,
        }
    } else {
        // In development, include more details
        ErrorResponse {
            status_code: status.as_u16(),
            message: error.to_string(),
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
            stack: Some(format!("{:?}", error)),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not Found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad Request: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal Error: {}", msg),
        }
    }
}
```

**Register Filter**:

```typescript
// apps/api/src/main.ts
import { HttpExceptionFilter } from '@/shared/filters/http-exception.filter';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  app.useGlobalFilters(new HttpExceptionFilter());

  await app.listen(3000);
}
```

**Custom Security Exceptions**:

```typescript
// apps/api/src/shared/exceptions/security.exceptions.ts
export class AccountLockedException extends Error {
  constructor(message: string = 'Account is locked') {
    super(message);
    this.name = 'AccountLockedException';
  }
}

export class WeakPasswordException extends HttpException {
  constructor(message: string = 'Password does not meet requirements') {
    super(message, HttpStatus.BAD_REQUEST);
  }
}

export class InvalidTokenException extends HttpException {
  constructor(message: string = 'Invalid or expired token') {
    super(message, HttpStatus.UNAUTHORIZED);
  }
}

export class InsufficientPermissionsException extends HttpException {
  constructor(message: string = 'Insufficient permissions') {
    super(message, HttpStatus.FORBIDDEN);
  }
}
```

---

## Frontend Security Patterns (Next.js)

### 1. Token Storage Pattern

#### 1.1 In-Memory Access Token Pattern

**Problem**: Storing tokens in localStorage exposes them to XSS attacks.

**Solution**: Store access tokens in memory only.

```typescript
// apps/web/lib/api/client.ts
import axios, { AxiosError, InternalAxiosRequestConfig } from 'axios';

// In-memory token storage (NOT localStorage!)
let accessToken: string | null = null;

/**
 * Set access token in memory
 */
export const setAccessToken = (token: string | null): void => {
  accessToken = token;
};

/**
 * Get current access token
 */
export const getAccessToken = (): string | null => {
  return accessToken;
};

/**
 * Clear access token
 */
export const clearAccessToken = (): void => {
  accessToken = null;
};

// Create axios instance
const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001/api/v1',
  headers: {
    'Content-Type': 'application/json',
  },
  withCredentials: true, // Send httpOnly cookies
});

/**
 * Request interceptor: Add Authorization header
 */
apiClient.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    const token = getAccessToken();
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error: AxiosError) => {
    return Promise.reject(error);
  },
);

export default apiClient;
```

**Why In-Memory is Secure**:

- Not accessible via JavaScript from other tabs/windows
- Cleared on page refresh (requires re-authentication)
- Not vulnerable to XSS attacks reading localStorage
- Cannot be stolen by malicious scripts

---

#### 1.2 Automatic Token Refresh Pattern

**Problem**: Access tokens expire quickly (15 min). Need seamless refresh.

**Solution**: Intercept 401 responses and automatically refresh token.

```typescript
// apps/web/lib/api/client.ts (continued)
let isRefreshing = false;
let failedQueue: Array<{
  resolve: (value?: unknown) => void;
  reject: (reason?: unknown) => void;
}> = [];

/**
 * Process queued requests after token refresh
 */
const processQueue = (error: Error | null, token: string | null = null): void => {
  failedQueue.forEach((prom) => {
    if (error) {
      prom.reject(error);
    } else {
      prom.resolve(token);
    }
  });

  failedQueue = [];
};

/**
 * Response interceptor: Handle 401 by refreshing token
 */
apiClient.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean };

    // If error is 401 and we haven't retried yet
    if (error.response?.status === 401 && !originalRequest._retry) {
      // Don't retry if it's the refresh endpoint itself
      if (originalRequest.url?.includes('/auth/refresh')) {
        clearAccessToken();

        // Redirect to login
        if (typeof window !== 'undefined') {
          window.location.href = '/login';
        }

        return Promise.reject(error);
      }

      if (isRefreshing) {
        // Queue this request to retry after refresh completes
        return new Promise((resolve, reject) => {
          failedQueue.push({ resolve, reject });
        })
          .then(() => {
            return apiClient(originalRequest);
          })
          .catch((err) => {
            return Promise.reject(err);
          });
      }

      originalRequest._retry = true;
      isRefreshing = true;

      try {
        // Attempt to refresh the token (uses httpOnly cookie)
        const response = await axios.post<{ accessToken: string }>(
          `${process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001/api/v1'}/auth/refresh`,
          {},
          {
            withCredentials: true, // Send refresh token cookie
          },
        );

        const newAccessToken = response.data.accessToken;
        setAccessToken(newAccessToken);

        // Update the original request with new token
        if (originalRequest.headers) {
          originalRequest.headers.Authorization = `Bearer ${newAccessToken}`;
        }

        // Process queued requests
        processQueue(null, newAccessToken);

        // Retry the original request
        return apiClient(originalRequest);
      } catch (refreshError) {
        // Refresh failed, clear token and reject all queued requests
        processQueue(refreshError as Error, null);
        clearAccessToken();

        // Redirect to login page
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

**Key Features**:

1. **Automatic retry** - Failed requests are retried with new token
2. **Request queueing** - Multiple simultaneous 401s refresh only once
3. **Graceful degradation** - Redirects to login if refresh fails
4. **Prevents infinite loops** - Doesn't retry refresh endpoint itself

---

### 2. XSS Prevention Pattern (Frontend)

#### 2.1 React Auto-Escaping

**Problem**: User-generated content can contain malicious scripts.

**Solution**: Leverage React's automatic escaping.

```tsx
// ✅ SAFE - React auto-escapes
export function UserProfile({ user }: { user: User }) {
  return (
    <div>
      <h1>{user.name}</h1> {/* Auto-escaped */}
      <p>{user.bio}</p> {/* Auto-escaped */}
      <a href={user.website}>{user.website}</a> {/* Auto-escaped */}
    </div>
  );
}

// ❌ DANGEROUS - Never use dangerouslySetInnerHTML with user input
export function UnsafeComponent({ userHtml }: { userHtml: string }) {
  return (
    <div dangerouslySetInnerHTML={{ __html: userHtml }} />
    // ⚠️ XSS vulnerability if userHtml contains <script> tags
  );
}
```

---

#### 2.2 DOMPurify Sanitization Pattern

**Problem**: Sometimes need to render user HTML (rich text editors).

**Solution**: Sanitize HTML with DOMPurify before rendering.

```tsx
// apps/web/components/rich-text-display.tsx
import DOMPurify from 'isomorphic-dompurify';

interface RichTextDisplayProps {
  html: string;
}

export function RichTextDisplay({ html }: RichTextDisplayProps) {
  // Sanitize HTML before rendering
  const sanitizedHtml = DOMPurify.sanitize(html, {
    ALLOWED_TAGS: [
      'b',
      'i',
      'em',
      'strong',
      'a',
      'p',
      'br',
      'ul',
      'ol',
      'li',
      'h1',
      'h2',
      'h3',
      'h4',
      'h5',
      'h6',
    ],
    ALLOWED_ATTR: ['href', 'title', 'target'],
    ALLOW_DATA_ATTR: false, // Prevent data-* attributes
  });

  return <div className="rich-text-content" dangerouslySetInnerHTML={{ __html: sanitizedHtml }} />;
}
```

**DOMPurify Configuration**:

- **ALLOWED_TAGS** - Whitelist of safe HTML tags
- **ALLOWED_ATTR** - Whitelist of safe attributes
- **ALLOW_DATA_ATTR** - Disable data attributes (can contain JS)
- **FORBID_TAGS** - Blacklist dangerous tags (script, iframe, object)
- **FORBID_ATTR** - Blacklist dangerous attributes (onclick, onerror)

---

#### 2.3 Content Security Policy (CSP)

**Problem**: Need defense-in-depth against XSS.

**Solution**: Set CSP headers in Next.js config.

```javascript
// apps/web/next.config.js
const ContentSecurityPolicy = `
  default-src 'self';
  script-src 'self' 'unsafe-eval' 'unsafe-inline';
  style-src 'self' 'unsafe-inline';
  img-src 'self' data: https:;
  font-src 'self' data:;
  connect-src 'self' ${process.env.NEXT_PUBLIC_API_URL};
  frame-ancestors 'none';
  base-uri 'self';
  form-action 'self';
`;

const securityHeaders = [
  {
    key: 'Content-Security-Policy',
    value: ContentSecurityPolicy.replace(/\s{2,}/g, ' ').trim(),
  },
  {
    key: 'X-DNS-Prefetch-Control',
    value: 'on',
  },
  {
    key: 'Strict-Transport-Security',
    value: 'max-age=63072000; includeSubDomains; preload',
  },
  {
    key: 'X-Frame-Options',
    value: 'DENY',
  },
  {
    key: 'X-Content-Type-Options',
    value: 'nosniff',
  },
  {
    key: 'Referrer-Policy',
    value: 'strict-origin-when-cross-origin',
  },
  {
    key: 'Permissions-Policy',
    value: 'camera=(), microphone=(), geolocation=()',
  },
];

module.exports = {
  async headers() {
    return [
      {
        source: '/:path*',
        headers: securityHeaders,
      },
    ];
  },
};
```

---

### 3. CSRF Protection Pattern (Frontend)

#### 3.1 SameSite Cookie Reliance

**Problem**: Need CSRF protection for state-changing requests.

**Solution**: Backend uses SameSite=strict cookies (automatic protection).

```typescript
// apps/web/lib/api/client.ts
const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL,
  withCredentials: true, // Send SameSite cookies
});

// No additional CSRF token needed!
// SameSite=strict prevents cross-site cookie sending
```

---

#### 3.2 CSRF Token Pattern (Alternative)

**Problem**: Some scenarios require explicit CSRF tokens.

**Solution**: Include CSRF token in request headers.

```typescript
// apps/web/lib/api/client.ts
let csrfToken: string | null = null;

/**
 * Fetch CSRF token from backend
 */
export const fetchCsrfToken = async (): Promise<void> => {
  const response = await axios.get(`${process.env.NEXT_PUBLIC_API_URL}/auth/csrf-token`, {
    withCredentials: true,
  });
  csrfToken = response.data.csrfToken;
};

/**
 * Request interceptor: Add CSRF token
 */
apiClient.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  // Add CSRF token for state-changing requests
  if (['POST', 'PATCH', 'DELETE', 'PUT'].includes(config.method?.toUpperCase() || '')) {
    if (csrfToken && config.headers) {
      config.headers['X-CSRF-Token'] = csrfToken;
    }
  }

  // Add access token
  const token = getAccessToken();
  if (token && config.headers) {
    config.headers.Authorization = `Bearer ${token}`;
  }

  return config;
});

// Initialize CSRF token on app load
fetchCsrfToken();
```

---

### 4. Secure Communication Pattern

#### 4.1 HTTPS Only Pattern

**Problem**: HTTP traffic can be intercepted (man-in-the-middle).

**Solution**: Enforce HTTPS in production.

```typescript
// apps/web/middleware.ts
import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

export function middleware(request: NextRequest) {
  // Enforce HTTPS in production
  if (
    process.env.NODE_ENV === 'production' &&
    request.headers.get('x-forwarded-proto') !== 'https'
  ) {
    return NextResponse.redirect(
      `https://${request.headers.get('host')}${request.nextUrl.pathname}`,
      301,
    );
  }

  return NextResponse.next();
}
```

---

#### 4.2 API Base URL Configuration

**Problem**: Hardcoding API URLs makes environment switching difficult.

**Solution**: Use environment variables for API URLs.

```typescript
// apps/web/.env.local
NEXT_PUBLIC_API_URL=http://localhost:4001/api/v1

// apps/web/.env.production
NEXT_PUBLIC_API_URL=https://api.onwellos.com/api/v1
```

```typescript
// apps/web/lib/api/client.ts
const apiClient = axios.create({
  baseURL: process.env.NEXT_PUBLIC_API_URL || 'http://localhost:4001/api/v1',
  withCredentials: true,
});
```

**Security Notes**:

- Use `NEXT_PUBLIC_*` prefix only for public URLs
- Never expose secrets in `NEXT_PUBLIC_*` variables
- Server-side secrets go in `.env` (without `NEXT_PUBLIC_`)

---

### 5. Input Validation Pattern (Frontend)

#### 5.1 Client-Side Validation

**Problem**: Need immediate feedback for user experience.

**Solution**: Validate on client-side, but ALWAYS validate on backend too.

```tsx
// apps/web/components/forms/login-form.tsx
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';

const loginSchema = z.object({
  email: z.string().email('Invalid email address'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
});

type LoginFormData = z.infer<typeof loginSchema>;

export function LoginForm() {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<LoginFormData>({
    resolver: zodResolver(loginSchema),
  });

  const onSubmit = async (data: LoginFormData) => {
    // Client-side validation passed
    // Backend will validate again (CRITICAL!)
    await authRepository.login(data);
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)}>
      <input {...register('email')} type="email" placeholder="Email" />
      {errors.email && <span className="error">{errors.email.message}</span>}

      <input {...register('password')} type="password" placeholder="Password" />
      {errors.password && <span className="error">{errors.password.message}</span>}

      <button type="submit">Login</button>
    </form>
  );
}
```

**Why Validate on Both Sides?**

- **Client-side** - Better UX, immediate feedback
- **Backend** - Security boundary, prevents bypass

**Never trust client-side validation alone!** Attackers can bypass it.

---

#### 5.2 Schema Validation Pattern

**Problem**: Need consistent validation across forms.

**Solution**: Define reusable validation schemas.

```typescript
// apps/web/lib/validation/auth.schemas.ts
import * as z from 'zod';

export const emailSchema = z.string().email('Invalid email address').min(1, 'Email is required');

export const passwordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters')
  .regex(/[A-Z]/, 'Password must contain an uppercase letter')
  .regex(/[a-z]/, 'Password must contain a lowercase letter')
  .regex(/[0-9]/, 'Password must contain a number')
  .regex(/[^A-Za-z0-9]/, 'Password must contain a special character');

export const loginSchema = z.object({
  email: emailSchema,
  password: z.string().min(1, 'Password is required'), // Don't validate strength on login
});

export const registerSchema = z.object({
  email: emailSchema,
  password: passwordSchema,
  name: z.string().min(2, 'Name must be at least 2 characters'),
});
```

---

### 6. Error Handling Pattern (Frontend)

#### 6.1 Generic Error Messages

**Problem**: Detailed error messages expose implementation details.

**Solution**: Show generic errors to users, log details for developers.

```typescript
// apps/web/lib/api/error-handler.ts
import { AxiosError } from 'axios';
import { toast } from 'sonner';

export function handleApiError(error: unknown): void {
  if (error instanceof AxiosError) {
    // Log detailed error for developers
    console.error('API Error:', {
      status: error.response?.status,
      data: error.response?.data,
      message: error.message,
    });

    // Show generic error to users
    const userMessage = getUserFriendlyMessage(error);
    toast.error(userMessage);
  } else {
    console.error('Unknown error:', error);
    toast.error('An unexpected error occurred. Please try again.');
  }
}

function getUserFriendlyMessage(error: AxiosError): string {
  const status = error.response?.status;

  switch (status) {
    case 400:
      return 'Invalid request. Please check your input.';
    case 401:
      return 'Authentication required. Please log in.';
    case 403:
      return 'You do not have permission to perform this action.';
    case 404:
      return 'The requested resource was not found.';
    case 429:
      return 'Too many requests. Please try again later.';
    case 500:
      return 'Server error. Please try again later.';
    default:
      return 'An error occurred. Please try again.';
  }
}
```

**Usage**:

```typescript
// apps/web/components/forms/login-form.tsx
import { handleApiError } from '@/lib/api/error-handler';

const onSubmit = async (data: LoginFormData) => {
  try {
    await authRepository.login(data);
  } catch (error) {
    handleApiError(error); // Generic error shown to user
  }
};
```

---

### 7. Route Protection Pattern

#### 7.1 Auth Guard Component

**Problem**: Need to protect authenticated routes.

**Solution**: Create reusable AuthGuard component.

```tsx
// apps/web/components/auth/auth-guard.tsx
'use client';

import { useAuth } from '@/lib/auth/auth-context';
import { useRouter } from 'next/navigation';
import { useEffect } from 'react';

export function AuthGuard({ children }: { children: React.ReactNode }) {
  const { isAuthenticated, isLoading } = useAuth();
  const router = useRouter();

  useEffect(() => {
    if (!isLoading && !isAuthenticated) {
      router.push('/login');
    }
  }, [isAuthenticated, isLoading, router]);

  // Show loading state
  if (isLoading) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-gray-900" />
      </div>
    );
  }

  // Show nothing if not authenticated (redirecting)
  if (!isAuthenticated) {
    return null;
  }

  // Render protected content
  return <>{children}</>;
}
```

**Usage**:

```tsx
// apps/web/app/(dashboard)/layout.tsx
import { AuthGuard } from '@/components/auth/auth-guard';

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <div className="dashboard-layout">{children}</div>
    </AuthGuard>
  );
}
```

---

#### 7.2 Permission-Based Rendering

**Problem**: Need to conditionally render UI based on user permissions.

**Solution**: Create permission check hooks.

```tsx
// apps/web/hooks/use-can.ts
import { useAuth } from '@/lib/auth/auth-context';
import { Action, Subject } from '@/types/permissions.types';

export function useCan() {
  const { user } = useAuth();

  const can = (action: Action, subject: Subject): boolean => {
    if (!user) return false;

    // Admin can do anything
    if (user.role === 'admin') return true;

    // Manager permissions
    if (user.role === 'manager') {
      if (action === 'delete' && subject === 'User') return false;
      return true;
    }

    // Consultant permissions
    if (user.role === 'consultant') {
      if (subject === 'TimeEntry') {
        return ['create', 'read', 'update', 'submit'].includes(action);
      }
      if (subject === 'Project' || subject === 'Client') {
        return action === 'read';
      }
      return false;
    }

    return false;
  };

  return { can };
}
```

**Usage**:

```tsx
// apps/web/components/time-entry/time-entry-actions.tsx
import { useCan } from '@/hooks/use-can';

export function TimeEntryActions({ timeEntry }: { timeEntry: TimeEntry }) {
  const { can } = useCan();

  return (
    <div>
      {can('update', 'TimeEntry') && <button onClick={() => handleEdit(timeEntry)}>Edit</button>}

      {can('delete', 'TimeEntry') && (
        <button onClick={() => handleDelete(timeEntry)}>Delete</button>
      )}

      {can('approve', 'TimeEntry') && (
        <button onClick={() => handleApprove(timeEntry)}>Approve</button>
      )}
    </div>
  );
}
```

---

## Implementation Checklist

### Backend Security Checklist

**Authentication**:

- [ ] Passwords hashed with bcrypt (cost 12)
- [ ] Password strength validation enforced
- [ ] JWT access tokens short-lived (15 min)
- [ ] Refresh tokens long-lived (7 days), stored in database
- [ ] Refresh tokens in httpOnly cookies
- [ ] JwtAuthGuard applied to protected routes
- [ ] Public decorator for public endpoints

**Authorization**:

- [ ] CASL AbilityFactory implemented
- [ ] PermissionsGuard applied to protected routes
- [ ] Fine-grained permissions defined for each role
- [ ] Entity-level permission checks

**Input Validation**:

- [ ] Global ValidationPipe configured
- [ ] class-validator decorators on all DTOs
- [ ] Input sanitization with Transform decorator
- [ ] Custom validators for domain-specific validation

**SQL Injection Prevention**:

- [ ] Drizzle ORM used for all queries
- [ ] No raw SQL string concatenation
- [ ] Parameters used if raw SQL is necessary

**XSS Prevention**:

- [ ] Helmet security headers configured
- [ ] CSP policy defined
- [ ] Output sanitization for user content

**CSRF Protection**:

- [ ] SameSite=strict cookies configured
- [ ] CSRF tokens for sensitive operations (if needed)

**Rate Limiting**:

- [ ] ThrottlerModule configured globally
- [ ] Custom rate limits on sensitive endpoints (login, password reset)
- [ ] Account lockout after failed attempts

**Audit Logging**:

- [ ] AuditLog entity created
- [ ] AuditLogInterceptor applied to mutations
- [ ] Sensitive data excluded from logs
- [ ] Logs immutable and timestamped

**Secret Management**:

- [ ] Environment variables validated on startup
- [ ] No secrets in version control
- [ ] .env.example provided
- [ ] Secrets Manager for production (AWS/Vault)

**Error Handling**:

- [ ] HttpExceptionFilter configured
- [ ] Generic error messages to users
- [ ] Detailed error logging
- [ ] No stack traces in production

**CORS**:

- [ ] CORS restricted to frontend origin
- [ ] credentials: true for cookies

**HTTPS**:

- [ ] HTTPS enforced in production
- [ ] Secure cookies in production

### Frontend Security Checklist

**Token Storage**:

- [ ] Access tokens in memory (NOT localStorage)
- [ ] Refresh tokens in httpOnly cookies
- [ ] Automatic token refresh on 401

**XSS Prevention**:

- [ ] React auto-escaping leveraged
- [ ] DOMPurify for user HTML
- [ ] No dangerouslySetInnerHTML without sanitization
- [ ] CSP headers configured

**CSRF Protection**:

- [ ] withCredentials: true for API calls
- [ ] SameSite cookies from backend
- [ ] CSRF tokens if needed

**Input Validation**:

- [ ] Client-side validation with react-hook-form + Zod
- [ ] Backend validation always enforced
- [ ] Reusable validation schemas

**Error Handling**:

- [ ] Generic error messages to users
- [ ] Detailed errors logged to console (dev only)
- [ ] Error boundaries for React errors

**Route Protection**:

- [ ] AuthGuard for protected routes
- [ ] Permission-based rendering
- [ ] Redirect to login if unauthenticated

**Secure Communication**:

- [ ] HTTPS enforced in production
- [ ] API base URL from environment variables
- [ ] No secrets in NEXT*PUBLIC*\* variables

---

## Security Anti-Patterns

### Backend Anti-Patterns

| ❌ Anti-Pattern                        | ✅ Correct Approach                   |
| -------------------------------------- | ------------------------------------- |
| Storing passwords in plain text        | Hash with bcrypt (cost 12)            |
| Long-lived access tokens (24h+)        | Short-lived (15 min) + refresh tokens |
| Storing refresh tokens in localStorage | httpOnly cookies                      |
| Hardcoding JWT secrets                 | Environment variables                 |
| String concatenation in SQL            | Parameterized queries (Drizzle)       |
| Exposing stack traces in production    | Generic error messages                |
| No rate limiting on login              | ThrottlerGuard on auth endpoints      |
| No audit logging                       | AuditLogInterceptor                   |
| Generic authorization (isAdmin?)       | CASL with fine-grained permissions    |
| Skipping input validation              | class-validator on all DTOs           |

### Frontend Anti-Patterns

| ❌ Anti-Pattern                 | ✅ Correct Approach             |
| ------------------------------- | ------------------------------- |
| Storing tokens in localStorage  | In-memory + httpOnly cookies    |
| Trusting client-side validation | Always validate on backend      |
| Using dangerouslySetInnerHTML   | React auto-escaping + DOMPurify |
| Showing detailed error messages | Generic errors + logging        |
| No route protection             | AuthGuard component             |
| Hardcoding API URLs             | Environment variables           |
| No CSP headers                  | Configure in next.config.js     |
| No HTTPS in production          | Enforce HTTPS redirect          |

---

## Testing Security Patterns

### Backend Security Tests

**1. Authentication Tests**:

```typescript
// apps/api/src/application/auth/commands/__tests__/login.handler.spec.ts
describe('LoginHandler', () => {
  it('should hash password before storing', async () => {
    const plainPassword = 'Test123!';
    const hashedPassword = await HashedPassword.fromPlainText(plainPassword);

    expect(hashedPassword.getHash()).not.toBe(plainPassword);
    expect(hashedPassword.getHash()).toMatch(/^\$2[aby]\$/); // bcrypt format
  });

  it('should enforce password strength', async () => {
    await expect(HashedPassword.fromPlainText('weak')).rejects.toThrow(
      'Password must be at least 8 characters',
    );
  });

  it('should lock account after 5 failed attempts', async () => {
    const user = User.create({
      /* ... */
    });

    // Simulate 5 failed login attempts
    for (let i = 0; i < 5; i++) {
      await user.verifyPassword('wrongpassword');
    }

    await expect(user.verifyPassword('wrongpassword')).rejects.toThrow(AccountLockedException);
  });

  it('should return short-lived access token', async () => {
    const token = jwtService.signAccessToken(user);
    const decoded = jwtService.verifyAccessToken(token);

    const expiry = decoded.exp! * 1000; // Convert to milliseconds
    const now = Date.now();
    const expiryDuration = expiry - now;

    expect(expiryDuration).toBeLessThanOrEqual(15 * 60 * 1000); // 15 minutes
  });
});
```

**2. Authorization Tests**:

```typescript
// apps/api/src/authorization/__tests__/abilities.factory.spec.ts
describe('AbilityFactory', () => {
  it('should allow admin to manage all', () => {
    const admin = User.create({ role: 'admin' });
    const ability = abilityFactory.createForUser(admin);

    expect(ability.can('manage', 'all')).toBe(true);
    expect(ability.can('delete', 'User')).toBe(true);
  });

  it('should prevent consultant from approving time entries', () => {
    const consultant = User.create({ role: 'consultant' });
    const ability = abilityFactory.createForUser(consultant);

    expect(ability.can('approve', 'TimeEntry')).toBe(false);
  });

  it('should allow consultant to update own draft time entries only', () => {
    const consultant = User.create({ role: 'consultant', id: 'user-123' });
    const ability = abilityFactory.createForUser(consultant);

    const ownDraftEntry = { userId: 'user-123', status: 'draft' };
    const ownSubmittedEntry = { userId: 'user-123', status: 'submitted' };
    const otherEntry = { userId: 'user-456', status: 'draft' };

    expect(ability.can('update', 'TimeEntry', ownDraftEntry)).toBe(true);
    expect(ability.can('update', 'TimeEntry', ownSubmittedEntry)).toBe(false);
    expect(ability.can('update', 'TimeEntry', otherEntry)).toBe(false);
  });
});
```

**3. Input Validation Tests**:

```typescript
// apps/api/src/presentation/user/__tests__/create-user.dto.spec.ts
describe('CreateUserDto', () => {
  it('should validate email format', async () => {
    const dto = new CreateUserDto();
    dto.email = 'invalid-email';
    dto.password = 'Test123!';
    dto.name = 'John Doe';

    const errors = await validate(dto);
    expect(errors).toHaveLength(1);
    expect(errors[0].property).toBe('email');
  });

  it('should sanitize email', () => {
    const dto = plainToClass(CreateUserDto, {
      email: '  TEST@EXAMPLE.COM  ',
      password: 'Test123!',
      name: 'John Doe',
    });

    expect(dto.email).toBe('test@example.com');
  });

  it('should reject weak passwords', async () => {
    const dto = new CreateUserDto();
    dto.email = 'test@example.com';
    dto.password = 'weak';
    dto.name = 'John Doe';

    const errors = await validate(dto);
    expect(errors.some((e) => e.property === 'password')).toBe(true);
  });
});
```

**4. SQL Injection Tests**:

```typescript
// apps/api/src/infrastructure/database/repositories/__tests__/user.repository.spec.ts
describe('UserRepository', () => {
  it('should prevent SQL injection in findByEmail', async () => {
    const maliciousEmail = "'; DROP TABLE users; --";

    // Should not throw error or execute SQL injection
    const result = await userRepository.findByEmail(maliciousEmail);

    expect(result).toBeNull();

    // Verify users table still exists
    const users = await userRepository.findAll();
    expect(users).toBeDefined();
  });
});
```

### Frontend Security Tests

**1. Token Storage Tests**:

```typescript
// apps/web/lib/api/__tests__/client.spec.ts
describe('API Client', () => {
  it('should store access token in memory only', () => {
    setAccessToken('test-token');

    // Verify NOT in localStorage
    expect(localStorage.getItem('accessToken')).toBeNull();

    // Verify in memory
    expect(getAccessToken()).toBe('test-token');
  });

  it('should clear token on 401 refresh failure', async () => {
    setAccessToken('expired-token');

    // Mock failed refresh
    mock.onPost('/auth/refresh').reply(401);

    try {
      await apiClient.get('/protected');
    } catch (error) {
      // Token should be cleared
      expect(getAccessToken()).toBeNull();
    }
  });
});
```

**2. XSS Prevention Tests**:

```typescript
// apps/web/components/__tests__/rich-text-display.spec.tsx
describe('RichTextDisplay', () => {
  it('should sanitize malicious HTML', () => {
    const maliciousHtml = '<script>alert("XSS")</script><p>Safe content</p>';

    render(<RichTextDisplay html={maliciousHtml} />);

    // Script tag should be removed
    expect(screen.queryByText('alert("XSS")')).not.toBeInTheDocument();

    // Safe content should remain
    expect(screen.getByText('Safe content')).toBeInTheDocument();
  });
});
```

---

## Security Scanning Tools

### Backend Tools

**1. Dependency Scanning**:

```bash
# Check for vulnerable dependencies
pnpm audit

# Auto-fix vulnerabilities
pnpm audit fix
```

**2. Static Analysis**:

```bash
# ESLint with security plugin
pnpm add -D eslint-plugin-security

# .eslintrc.js
module.exports = {
  plugins: ['security'],
  extends: ['plugin:security/recommended'],
};
```

**3. Secret Scanning**:

```bash
# Install trufflehog
brew install trufflesecurity/trufflehog/trufflehog

# Scan repository
trufflehog git file://. --only-verified
```

### Frontend Tools

**1. Dependency Scanning**:

```bash
pnpm audit
```

**2. Content Security Policy Testing**:

```bash
# Install CSP validator
pnpm add -D csp-validator

# Test CSP
csp-validator next.config.js
```

---

## References

### Security Resources

**OWASP**:

- [OWASP Top 10](https://owasp.org/www-project-top-ten/) - Most critical security risks
- [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/) - Security best practices
- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/) - API-specific risks

**Rust + Axum Security**:

- [Axum Documentation](https://docs.rs/axum/latest/axum/) - Official Axum docs
- [Tower HTTP](https://docs.rs/tower-http/latest/tower_http/) - HTTP middleware
- [jsonwebtoken](https://docs.rs/jsonwebtoken/latest/jsonwebtoken/) - JWT library

**Next.js Security**:

- [Next.js Security Headers](https://nextjs.org/docs/advanced-features/security-headers)
- [Next.js Authentication](https://nextjs.org/docs/authentication)
- [Content Security Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/CSP)

**JWT**:

- [JWT.io](https://jwt.io/) - JWT debugger
- [JWT Best Practices](https://tools.ietf.org/html/rfc8725) - RFC 8725

**Bcrypt**:

- [Bcrypt Documentation](https://www.npmjs.com/package/bcrypt)
- [Password Hashing Competition](https://password-hashing.net/)

**CASL**:

- [CASL Documentation](https://casl.js.org/v6/en/)
- [CASL NestJS Integration](https://casl.js.org/v6/en/package/casl-nestjs)

### Related Pattern Documents

- [01-RBAC-CASL-Pattern.md](./01-RBAC-CASL-Pattern.md) - Detailed CASL implementation
- [03-Hexagonal-Architecture.md](./03-Hexagonal-Architecture.md) - Security in infrastructure layer
- [06-Repository-Pattern.md](./06-Repository-Pattern.md) - SQL injection prevention
- [07-DTO-Pattern.md](./07-DTO-Pattern.md) - Input validation
- [19-Soft-Delete-Implementation-Guide.md](./19-Soft-Delete-Implementation-Guide.md) - Data retention

### Related Guides

- [Security Best Practices Guide](../guides/security-guide.md) - Quick reference
- [Testing Guide](../guides/testing-guide.md) - Security testing strategies
- [Development Tools](../guides/development-tools.md) - Security linting setup

---

## Summary

**Security is a layered approach**. No single pattern provides complete protection. Implement all patterns for defense-in-depth:

**Backend Security Layers**:

1. Authentication (bcrypt + JWT + refresh tokens)
2. Authorization (CASL RBAC)
3. Input validation (class-validator)
4. SQL injection prevention (Drizzle ORM)
5. XSS prevention (Helmet, sanitization)
6. CSRF protection (SameSite cookies)
7. Rate limiting (ThrottlerGuard)
8. Audit logging (AuditLogInterceptor)
9. Secret management (environment variables)
10. Error handling (generic messages)

**Frontend Security Layers**:

1. Token storage (in-memory + httpOnly cookies)
2. Automatic token refresh (401 interceptor)
3. XSS prevention (React escaping + DOMPurify)
4. CSRF protection (SameSite cookies)
5. Input validation (react-hook-form + Zod)
6. Error handling (generic messages)
7. Route protection (AuthGuard)
8. HTTPS enforcement (production)

**Remember**: Security is not a feature, it's a requirement. Every line of code must consider security implications.

---

**Version**: 1.0
**Last Updated**: October 6, 2025
**Author**: WellOS Development Team
