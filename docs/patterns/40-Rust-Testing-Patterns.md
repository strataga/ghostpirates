# Rust Testing Patterns

**Version**: 2.0
**Last Updated**: November 3, 2025

Comprehensive guide to testing patterns and best practices for Rust in the WellOS project.

---

## Table of Contents

1. [Overview](#overview)
2. [Core Testing Principles](#core-testing-principles)
3. [Test Organization](#test-organization)
4. [Mocking Patterns](#mocking-patterns)
5. [Domain Testing Patterns](#domain-testing-patterns)
6. [Application Layer Testing](#application-layer-testing)
7. [Infrastructure Testing](#infrastructure-testing)
8. [Presentation Layer Testing](#presentation-layer-testing)
9. [Common Pitfalls](#common-pitfalls)
10. [Best Practices](#best-practices)

---

## Overview

### Testing Strategy

Our testing approach follows the **Test Pyramid**:

```
        /\
       /E2E\       ← Few, high-value tests
      /------\
     /  INT   \    ← Moderate coverage
    /----------\
   /   UNIT     \  ← Extensive coverage
  /--------------\
```

**Coverage Requirements**:

- **Statements**: ≥80%
- **Branches**: ≥80%
- **Functions**: ≥80%
- **Lines**: ≥80%

### Test Types

1. **Unit Tests** - Test individual units in isolation (`cargo test`)
2. **Integration Tests** - Test interactions between components
3. **Benchmark Tests** - Performance testing with Criterion
4. **E2E Tests** - Test complete user flows (future)

---

## Core Testing Principles

### 1. AAA Pattern (Arrange-Act-Assert)

Always structure tests using the AAA pattern:

```rust
#[test]
fn should_create_user_with_valid_email() {
    // Arrange - Set up test data
    let email = "test@example.com";
    let first_name = "John";
    let last_name = "Doe";

    // Act - Execute the code under test
    let user = User::create(email, first_name, last_name).unwrap();

    // Assert - Verify the outcome
    assert_eq!(user.email, email);
    assert_eq!(user.first_name, first_name);
}
```

### 2. Test Isolation

Each test must be independent and not rely on other tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Tests run in parallel and isolated
    #[test]
    fn test_1() {
        let mut data = vec![1, 2, 3];
        data.push(4);
        assert_eq!(data.len(), 4);
    }

    #[test]
    fn test_2() {
        let mut data = vec![1, 2, 3];
        data.clear();
        assert_eq!(data.len(), 0);
    }
}
```

### 3. Test Documentation

Use doc comments to document test modules:

```rust
/// Test Suite: Well Entity
///
/// Tests the well domain entity functionality.
/// Verifies that:
/// - Wells can be created with valid coordinates
/// - Well status transitions work correctly
/// - Production data updates are validated
/// - Proper error handling for invalid operations
#[cfg(test)]
mod tests {
    use super::*;

    // Tests...
}
```

---

## Test Organization

### File Structure

```
src/
├── domain/
│   └── well/
│       ├── mod.rs
│       ├── entity.rs
│       └── tests.rs           ← Unit tests
├── application/
│   └── handlers/
│       ├── mod.rs
│       ├── create_well.rs
│       └── tests.rs           ← Handler tests
├── infrastructure/
│   └── repositories/
│       ├── mod.rs
│       ├── well_repository.rs
│       └── tests.rs           ← Repository tests
tests/
├── integration/               ← Integration tests
│   ├── api_tests.rs
│   └── database_tests.rs
└── common/                    ← Shared test utilities
    └── fixtures.rs
```

### Test Module Structure

```rust
// Inline tests in same file
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        // Test implementation
    }

    #[test]
    fn test_validation() {
        // Test implementation
    }
}

// Separate test file
// tests/integration_test.rs
use myapp::*;

#[test]
fn integration_test_1() {
    // Test implementation
}
```

---

## Mocking Patterns

### 1. Trait-Based Mocking (mockall)

**Problem**: Need to mock dependencies in tests.

**Solution**: Use `mockall` crate for trait-based mocking.

```rust
use mockall::*;

// Define trait
#[automock]
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Result<Option<User>, Error>;
    fn save(&self, user: &User) -> Result<(), Error>;
}

// Use mock in tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_find_user_by_id() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_find_by_id()
            .with(eq("user-123"))
            .times(1)
            .returning(|_| Ok(Some(User::default())));

        // Act
        let result = mock_repo.find_by_id("user-123");

        // Assert
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
```

### 2. Builder Pattern for Test Data

```rust
// Test data builder
pub struct UserBuilder {
    email: String,
    first_name: String,
    last_name: String,
    status: UserStatus,
}

impl UserBuilder {
    pub fn new() -> Self {
        Self {
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            status: UserStatus::Active,
        }
    }

    pub fn with_email(mut self, email: &str) -> Self {
        self.email = email.to_string();
        self
    }

    pub fn with_status(mut self, status: UserStatus) -> Self {
        self.status = status;
        self
    }

    pub fn build(self) -> User {
        User::create(&self.email, &self.first_name, &self.last_name)
            .unwrap()
    }
}

// Usage in tests
#[test]
fn test_with_builder() {
    let user = UserBuilder::new()
        .with_email("custom@example.com")
        .with_status(UserStatus::Inactive)
        .build();

    assert_eq!(user.email, "custom@example.com");
}
```

### 3. Fixture Functions

```rust
// Test fixtures
fn create_test_user() -> User {
    User::create("test@example.com", "John", "Doe").unwrap()
}

fn create_test_well() -> Well {
    Well::create("Well-001", 32.5, -101.2, WellStatus::Active).unwrap()
}

// Usage
#[test]
fn test_with_fixtures() {
    let user = create_test_user();
    let well = create_test_well();

    assert_eq!(user.email, "test@example.com");
    assert_eq!(well.name, "Well-001");
}
```

---

## Domain Testing Patterns

### 1. Entity Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_well_with_valid_data() {
        let well = Well::create(
            "Well-001",
            32.5,
            -101.2,
            WellStatus::Active,
        ).unwrap();

        assert_eq!(well.name, "Well-001");
        assert_eq!(well.latitude, 32.5);
        assert_eq!(well.status, WellStatus::Active);
    }

    #[test]
    fn should_mark_well_as_abandoned() {
        let mut well = Well::create(
            "Well-001",
            32.5,
            -101.2,
            WellStatus::Active,
        ).unwrap();

        well.mark_as_abandoned();

        assert_eq!(well.status, WellStatus::Abandoned);
        assert!(well.abandoned_at.is_some());
    }

    #[test]
    fn should_reject_invalid_latitude() {
        let result = Well::create(
            "Well-001",
            91.0, // Invalid
            -101.2,
            WellStatus::Active,
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Latitude must be between -90 and 90"
        );
    }
}
```

### 2. Value Object Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_valid_email() {
        let email = Email::new("test@example.com");
        assert!(email.is_ok());
        assert_eq!(email.unwrap().value(), "test@example.com");
    }

    #[test]
    fn should_reject_invalid_email_format() {
        let email = Email::new("invalid");
        assert!(email.is_err());
    }

    #[test]
    fn should_reject_too_long_email() {
        let long_email = format!("{}@example.com", "a".repeat(255));
        let email = Email::new(&long_email);
        assert!(email.is_err());
    }

    #[test]
    fn should_compare_emails_case_insensitively() {
        let email1 = Email::new("Test@Example.com").unwrap();
        let email2 = Email::new("test@example.com").unwrap();
        assert_eq!(email1, email2);
    }
}
```

### 3. Result-Based Error Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_ok_for_valid_operation() {
        let result = perform_operation();
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_specific_error() {
        let result = invalid_operation();
        assert!(result.is_err());

        match result {
            Err(Error::NotFound(msg)) => {
                assert_eq!(msg, "User not found");
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    #[should_panic(expected = "Invalid state")]
    fn should_panic_on_invalid_state() {
        dangerous_operation();
    }
}
```

---

## Application Layer Testing

### 1. Handler Testing with Mocks

```rust
use mockall::predicate::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_create_user_when_email_not_exists() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_find_by_email()
            .with(eq("test@example.com"))
            .times(1)
            .returning(|_| Ok(None));

        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let handler = CreateUserHandler::new(mock_repo);
        let command = CreateUserCommand {
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        // Act
        let result = handler.execute(command).await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_return_error_when_email_exists() {
        // Arrange
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_find_by_email()
            .returning(|_| Ok(Some(User::default())));

        let handler = CreateUserHandler::new(mock_repo);
        let command = CreateUserCommand {
            email: "existing@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        // Act
        let result = handler.execute(command).await;

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::Conflict(msg)) => {
                assert_eq!(msg, "Email already in use");
            }
            _ => panic!("Expected Conflict error"),
        }
    }
}
```

### 2. Async Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn should_handle_async_operation() {
        let result = async_function().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn should_timeout_long_operation() {
        use tokio::time::{timeout, Duration};

        let result = timeout(
            Duration::from_millis(100),
            very_long_operation()
        ).await;

        assert!(result.is_err()); // Timeout
    }
}
```

---

## Infrastructure Testing

### 1. Database Repository Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    async fn should_save_and_retrieve_user(pool: PgPool) {
        // Arrange
        let repo = UserRepository::new(pool.clone());
        let user = User::create("test@example.com", "John", "Doe").unwrap();

        // Act - Save
        repo.save(&user).await.unwrap();

        // Act - Retrieve
        let retrieved = repo.find_by_id(&user.id).await.unwrap();

        // Assert
        assert!(retrieved.is_some());
        let retrieved_user = retrieved.unwrap();
        assert_eq!(retrieved_user.email, user.email);
    }

    #[sqlx::test]
    async fn should_return_none_when_not_found(pool: PgPool) {
        let repo = UserRepository::new(pool);
        let result = repo.find_by_id("non-existent").await.unwrap();
        assert!(result.is_none());
    }
}
```

### 2. HTTP Client Testing

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn should_call_external_api() {
    // Arrange - Start mock server
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": "test"
        })))
        .mount(&mock_server)
        .await;

    // Act
    let client = HttpClient::new(&mock_server.uri());
    let result = client.fetch_data().await;

    // Assert
    assert!(result.is_ok());
}
```

---

## Presentation Layer Testing

### 1. HTTP Handler Testing (Actix-web)

```rust
use actix_web::{test, App};

#[actix_web::test]
async fn test_get_users_endpoint() {
    // Arrange
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;

    // Act
    let req = test::TestRequest::get()
        .uri("/api/users")
        .insert_header(("authorization", "Bearer test-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["data"].is_array());
}

#[actix_web::test]
async fn test_create_user_endpoint() {
    let app = test::init_service(
        App::new().configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/users")
        .set_json(json!({
            "email": "test@example.com",
            "first_name": "John",
            "last_name": "Doe"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 201);
}
```

### 2. Serialization Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_user_dto() {
        let dto = UserDto {
            id: "user-123".to_string(),
            email: "test@example.com".to_string(),
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
        };

        let json = serde_json::to_string(&dto).unwrap();
        assert!(json.contains("user-123"));
        assert!(json.contains("test@example.com"));
    }

    #[test]
    fn should_deserialize_user_dto() {
        let json = r#"{
            "id": "user-123",
            "email": "test@example.com",
            "first_name": "John",
            "last_name": "Doe"
        }"#;

        let dto: UserDto = serde_json::from_str(json).unwrap();
        assert_eq!(dto.id, "user-123");
        assert_eq!(dto.email, "test@example.com");
    }
}
```

---

## Common Pitfalls

### 1. Unintended Data Mutation

```rust
// ❌ BAD - Mutable shared state
static mut COUNTER: i32 = 0;

#[test]
fn test_1() {
    unsafe {
        COUNTER += 1;
        assert_eq!(COUNTER, 1); // May fail due to test order
    }
}

// ✅ GOOD - Isolated state
#[test]
fn test_1() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);
}
```

### 2. Forgetting Async Runtime

```rust
// ❌ BAD - Missing async runtime
#[test]
fn test_async() {
    let result = async_function().await; // Won't compile
}

// ✅ GOOD - Use tokio::test
#[tokio::test]
async fn test_async() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### 3. Not Handling Errors

```rust
// ❌ BAD - Unwrapping in tests
#[test]
fn test_operation() {
    let result = risky_operation().unwrap(); // Panics with no context
}

// ✅ GOOD - Proper error handling
#[test]
fn test_operation() {
    let result = risky_operation();
    assert!(result.is_ok(), "Operation failed: {:?}", result.err());
}
```

---

## Best Practices

### 1. Test Naming Conventions

```rust
// ✅ GOOD - Descriptive names
#[test]
fn should_create_user_with_valid_email() { }

#[test]
fn should_reject_user_with_invalid_email() { }

#[test]
fn should_return_not_found_when_user_does_not_exist() { }

// ❌ BAD - Vague names
#[test]
fn test_user() { }

#[test]
fn test_email() { }
```

### 2. Use rstest for Parameterized Tests

```rust
use rstest::rstest;

#[rstest]
#[case("test@example.com", true)]
#[case("user@domain.co.uk", true)]
#[case("invalid", false)]
#[case("@example.com", false)]
#[case("test@", false)]
fn test_email_validation(#[case] email: &str, #[case] expected: bool) {
    let result = Email::new(email);
    assert_eq!(result.is_ok(), expected);
}
```

### 3. Property-Based Testing with proptest

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_email_roundtrip(email in "[a-z]{5,10}@[a-z]{3,7}\\.com") {
        let parsed = Email::new(&email).unwrap();
        assert_eq!(parsed.value(), email);
    }
}
```

### 4. Benchmark Tests with Criterion

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_create_user(c: &mut Criterion) {
    c.bench_function("create_user", |b| {
        b.iter(|| {
            User::create(
                black_box("test@example.com"),
                black_box("John"),
                black_box("Doe")
            )
        })
    });
}

criterion_group!(benches, benchmark_create_user);
criterion_main!(benches);
```

---

## Quick Reference

### Essential Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in release mode (faster)
cargo test --release

# Run with coverage
cargo tarpaulin --out Html

# Run benchmarks
cargo bench

# Run mutation tests
cargo mutants
```

### Common Assertions

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean
assert!(condition);
assert!(!condition);

// Results
assert!(result.is_ok());
assert!(result.is_err());

// Options
assert!(option.is_some());
assert!(option.is_none());

// Panic
#[should_panic]
#[should_panic(expected = "error message")]
```

---

## Resources

- [Rust Book - Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [mockall Documentation](https://docs.rs/mockall/)
- [rstest Documentation](https://docs.rs/rstest/)
- [Criterion Documentation](https://docs.rs/criterion/)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)

---

**Remember**: Good tests are:

1. **Fast** - Run quickly to encourage frequent execution
2. **Independent** - No dependencies between tests
3. **Repeatable** - Same result every time
4. **Self-validating** - Pass/fail, no manual verification
5. **Timely** - Written before or with the code

Write tests that document behavior, not implementation details.
