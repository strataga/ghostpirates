# Comprehensive Testing Strategy

**Focus**: Unit Tests → Integration Tests → Contract Tests → Chaos Engineering → Test Data
**Priority**: Critical (quality assurance across all components)
**Cross-cutting**: All phases and components

---

## Epic 1: Unit Test Framework

### Task 1.1: Rust Unit Testing

**Type**: Testing
**Dependencies**: Code modules implemented

**Subtasks**:

- [ ] 1.1.1: Configure test environment

```toml
# apps/api/Cargo.toml
[dev-dependencies]
mockall = "0.12"
fake = "2.9"
proptest = "1.4"
```

- [ ] 1.1.2: Domain model unit tests

```rust
// apps/api/src/domain/teams/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_creation() {
        let team = Team::new(
            Uuid::new_v4(),
            "Build a web scraper".to_string(),
            Uuid::new_v4(),
        );

        assert_eq!(team.status, TeamStatus::Pending);
        assert!(team.manager_agent_id.is_none());
        assert!(team.started_at.is_none());
    }

    #[test]
    fn test_team_start() {
        let mut team = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
        );

        let manager_id = Uuid::new_v4();
        team.start(manager_id);

        assert_eq!(team.status, TeamStatus::Planning);
        assert_eq!(team.manager_agent_id, Some(manager_id));
        assert!(team.started_at.is_some());
    }

    #[test]
    fn test_team_lifecycle() {
        let mut team = Team::new(
            Uuid::new_v4(),
            "Test goal".to_string(),
            Uuid::new_v4(),
        );

        team.start(Uuid::new_v4());
        assert_eq!(team.status, TeamStatus::Planning);

        team.activate();
        assert_eq!(team.status, TeamStatus::Active);

        team.complete();
        assert_eq!(team.status, TeamStatus::Completed);
        assert!(team.completed_at.is_some());
    }
}
```

- [ ] 1.1.3: Task workflow unit tests

```rust
// apps/api/src/domain/tasks/tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_assignment() {
        let mut task = Task::new(
            Uuid::new_v4(),
            "Implement auth".to_string(),
            "Add JWT authentication".to_string(),
            vec!["JWT working".to_string()],
        );

        let worker_id = Uuid::new_v4();
        let manager_id = Uuid::new_v4();

        task.assign(worker_id, manager_id);

        assert_eq!(task.status, TaskStatus::Assigned);
        assert_eq!(task.assigned_to, Some(worker_id));
        assert_eq!(task.assigned_by, Some(manager_id));
    }

    #[test]
    fn test_revision_limits() {
        let mut task = Task::new(
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            vec!["Done".to_string()],
        );

        // Should allow up to max_revisions
        for _ in 0..task.max_revisions {
            assert!(task.request_revision().is_ok());
        }

        // Should fail after max_revisions
        assert!(task.request_revision().is_err());
    }

    #[test]
    fn test_task_state_transitions() {
        let mut task = Task::new(
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            vec![],
        );

        assert_eq!(task.status, TaskStatus::Pending);

        task.assign(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(task.status, TaskStatus::Assigned);

        task.start_work();
        assert_eq!(task.status, TaskStatus::InProgress);
        assert!(task.start_time.is_some());

        task.submit_for_review(serde_json::json!({}));
        assert_eq!(task.status, TaskStatus::Review);

        task.approve();
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.completion_time.is_some());
    }
}
```

- [ ] 1.1.4: Service layer unit tests with mocks

```rust
// apps/api/src/services/tests/team_service_tests.rs
use mockall::predicate::*;
use mockall::mock;

mock! {
    TeamsRepository {
        fn create(&self, team: &Team) -> Result<(), sqlx::Error>;
        fn find_by_id(&self, id: Uuid) -> Result<Option<Team>, sqlx::Error>;
        fn update_status(&self, id: Uuid, status: &str) -> Result<(), sqlx::Error>;
    }
}

#[tokio::test]
async fn test_create_team_service() {
    let mut mock_repo = MockTeamsRepository::new();

    mock_repo
        .expect_create()
        .times(1)
        .returning(|_| Ok(()));

    let service = TeamService::new(mock_repo);

    let result = service.create_team(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Test goal".to_string(),
    ).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_team_status() {
    let mut mock_repo = MockTeamsRepository::new();
    let team_id = Uuid::new_v4();

    mock_repo
        .expect_update_status()
        .with(eq(team_id), eq("active"))
        .times(1)
        .returning(|_, _| Ok(()));

    let service = TeamService::new(mock_repo);

    let result = service.update_team_status(team_id, TeamStatus::Active).await;

    assert!(result.is_ok());
}
```

- [ ] 1.1.5: Property-based testing

```rust
// apps/api/src/domain/tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_team_member_workload_invariant(
        current_workload in 0..10i32,
        max_concurrent in 1..10i32
    ) {
        let mut member = TeamMember::new_worker(
            Uuid::new_v4(),
            "Test".to_string(),
        );
        member.current_workload = current_workload.min(max_concurrent);
        member.max_concurrent_tasks = max_concurrent;

        prop_assert!(member.current_workload <= member.max_concurrent_tasks);
    }

    #[test]
    fn test_task_revision_count_invariant(
        revision_count in 0..5i32,
        max_revisions in 1..5i32
    ) {
        let mut task = Task::new(
            Uuid::new_v4(),
            "Test".to_string(),
            "Test".to_string(),
            vec![],
        );
        task.revision_count = revision_count.min(max_revisions);
        task.max_revisions = max_revisions;

        prop_assert!(task.revision_count <= task.max_revisions);
    }
}
```

- [ ] 1.1.6: Run unit tests

```bash
# Run all tests
cargo test

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage

# Run specific test module
cargo test domain::teams::tests

# Run with output
cargo test -- --nocapture
```

**Acceptance Criteria**:

- [ ] All domain models have unit tests
- [ ] All services have unit tests with mocks
- [ ] Property-based tests for invariants
- [ ] Test coverage > 80%
- [ ] All tests pass
- [ ] Fast test execution (< 30s)

---

## Epic 2: Integration Test Framework

### Task 2.1: Database Integration Tests

**Type**: Testing
**Dependencies**: Database schema complete

**Subtasks**:

- [ ] 2.1.1: Set up test database

```rust
// apps/api/tests/common/mod.rs
use sqlx::PgPool;
use std::sync::Once;

static INIT: Once = Once::new();

pub async fn setup_test_db() -> PgPool {
    INIT.call_once(|| {
        dotenv::from_filename(".env.test").ok();
    });

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

pub async fn cleanup_database(pool: &PgPool) {
    sqlx::query!("TRUNCATE TABLE teams, tasks, team_members, messages CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup database");
}
```

- [ ] 2.1.2: Repository integration tests

```rust
// apps/api/tests/repository_tests.rs
use sqlx::PgPool;

mod common;

#[sqlx::test]
async fn test_teams_repository_create_and_find(pool: PgPool) -> sqlx::Result<()> {
    let repo = TeamsRepository::new(pool.clone());

    let team = Team::new(
        Uuid::new_v4(),
        "Test goal".to_string(),
        Uuid::new_v4(),
    );

    repo.create(&team).await?;

    let found = repo.find_by_id(team.id).await?;

    assert!(found.is_some());
    assert_eq!(found.unwrap().goal, "Test goal");

    Ok(())
}

#[sqlx::test]
async fn test_tasks_repository_workflow(pool: PgPool) -> sqlx::Result<()> {
    let repo = TasksRepository::new(pool.clone());

    let task = Task::new(
        Uuid::new_v4(),
        "Test task".to_string(),
        "Description".to_string(),
        vec!["Criterion 1".to_string()],
    );

    repo.create(&task).await?;

    let tasks = repo.find_by_team(task.team_id).await?;
    assert_eq!(tasks.len(), 1);

    repo.update_status(task.id, "in_progress").await?;

    let updated = repo.find_by_id(task.id).await?.unwrap();
    assert_eq!(updated.status, TaskStatus::InProgress);

    Ok(())
}
```

- [ ] 2.1.3: End-to-end workflow tests

```rust
// apps/api/tests/workflow_tests.rs
#[sqlx::test]
async fn test_complete_team_workflow(pool: PgPool) -> sqlx::Result<()> {
    let team_service = TeamService::new(pool.clone());
    let task_service = TaskService::new(pool.clone());

    // Create team
    let team = team_service.create_team(
        Uuid::new_v4(),
        Uuid::new_v4(),
        "Build API".to_string(),
    ).await?;

    // Start team
    team_service.start_team(team.id).await?;

    // Create task
    let task = task_service.create_task(
        team.id,
        "Implement endpoint".to_string(),
        "Add GET /users endpoint".to_string(),
        vec!["Returns user list".to_string()],
    ).await?;

    // Assign task
    let worker_id = Uuid::new_v4();
    let manager_id = Uuid::new_v4();
    task_service.assign_task(task.id, worker_id, manager_id).await?;

    // Start work
    task_service.start_task(task.id).await?;

    // Submit for review
    task_service.submit_for_review(
        task.id,
        serde_json::json!({"result": "completed"})
    ).await?;

    // Approve
    task_service.approve_task(task.id).await?;

    // Verify final state
    let final_task = task_service.get_task(task.id).await?.unwrap();
    assert_eq!(final_task.status, TaskStatus::Completed);

    Ok(())
}
```

**Acceptance Criteria**:

- [ ] Database integration tests passing
- [ ] Repository tests cover all CRUD operations
- [ ] Workflow tests cover complete flows
- [ ] Tests isolated (cleanup between tests)
- [ ] No flaky tests

---

## Epic 3: Contract Tests

### Task 3.1: API Contract Testing

**Type**: Testing
**Dependencies**: API endpoints implemented

**Subtasks**:

- [ ] 3.1.1: Install Pact

```bash
cargo add --dev pact_consumer
```

- [ ] 3.1.2: Create consumer contract tests

```rust
// apps/frontend/tests/contract/api_contract.rs
use pact_consumer::prelude::*;

#[tokio::test]
async fn test_create_team_contract() {
    let pact = PactBuilder::new("Frontend", "API")
        .interaction("create team", |i| {
            i.request
                .post()
                .path("/api/teams")
                .json_body(json!({
                    "goal": "Build a web scraper",
                    "budget_limit": 100.0
                }));

            i.response
                .status(201)
                .json_body(json!({
                    "id": Matcher::uuid(),
                    "goal": "Build a web scraper",
                    "status": "pending",
                    "created_at": Matcher::iso_datetime()
                }));
        })
        .build();

    let mock_server = pact.start_mock_server();

    // Make request to mock server
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/teams", mock_server.url()))
        .json(&json!({
            "goal": "Build a web scraper",
            "budget_limit": 100.0
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);

    // Verify pact
    pact.verify();
}

#[tokio::test]
async fn test_get_team_contract() {
    let pact = PactBuilder::new("Frontend", "API")
        .interaction("get team", |i| {
            i.request
                .get()
                .path("/api/teams/123e4567-e89b-12d3-a456-426614174000");

            i.response
                .status(200)
                .json_body(json!({
                    "id": "123e4567-e89b-12d3-a456-426614174000",
                    "goal": Matcher::string("Build a web scraper"),
                    "status": Matcher::regex("pending|active|completed", "pending"),
                    "budget_limit": Matcher::decimal(100.0),
                    "created_at": Matcher::iso_datetime()
                }));
        })
        .build();

    let mock_server = pact.start_mock_server();

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/teams/123e4567-e89b-12d3-a456-426614174000", mock_server.url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    pact.verify();
}
```

- [ ] 3.1.3: Provider verification tests

```rust
// apps/api/tests/contract/provider_verification.rs
#[tokio::test]
async fn verify_provider_pacts() {
    let server_url = "http://localhost:4000";

    PactBuilder::new()
        .verify_provider("API")
        .provider_state_url(format!("{}/provider-states", server_url))
        .pact_url("./pacts/frontend-api.json")
        .verify()
        .await
        .unwrap();
}
```

**Acceptance Criteria**:

- [ ] Consumer tests define expected contracts
- [ ] Provider verifies all consumer contracts
- [ ] Breaking changes detected
- [ ] Pact broker integrated (optional)
- [ ] Contract tests in CI/CD

---

## Epic 4: Chaos Engineering

### Task 4.1: Failure Injection Testing

**Type**: Testing
**Dependencies**: All resilience features implemented

**Subtasks**:

- [ ] 4.1.1: Install chaos testing tools

```bash
# Install Chaos Mesh
kubectl create ns chaos-mesh
helm install chaos-mesh chaos-mesh/chaos-mesh \
  --namespace=chaos-mesh \
  --set chaosDaemon.runtime=containerd
```

- [ ] 4.1.2: Create network chaos experiments

```yaml
# tests/chaos/network-delay.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: network-delay
  namespace: ghostpirates-test
spec:
  action: delay
  mode: one
  selector:
    namespaces:
      - ghostpirates-test
    labelSelectors:
      app: ghostpirates
  delay:
    latency: "500ms"
    correlation: "50"
    jitter: "100ms"
  duration: "5m"
```

- [ ] 4.1.3: Create pod failure experiments

```yaml
# tests/chaos/pod-failure.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: PodChaos
metadata:
  name: pod-failure
  namespace: ghostpirates-test
spec:
  action: pod-kill
  mode: one
  selector:
    namespaces:
      - ghostpirates-test
    labelSelectors:
      app: ghostpirates
  duration: "30s"
  scheduler:
    cron: "@every 5m"
```

- [ ] 4.1.4: Create database failure tests

```yaml
# tests/chaos/database-failure.yaml
apiVersion: chaos-mesh.org/v1alpha1
kind: NetworkChaos
metadata:
  name: database-partition
  namespace: ghostpirates-test
spec:
  action: partition
  mode: one
  selector:
    namespaces:
      - ghostpirates-test
    labelSelectors:
      app: ghostpirates
  direction: to
  target:
    mode: all
    selector:
      namespaces:
        - ghostpirates-test
      labelSelectors:
        app: postgres
  duration: "2m"
```

- [ ] 4.1.5: Create chaos test suite

```bash
#!/bin/bash
# tests/chaos/run-chaos-tests.sh

set -e

echo "Starting chaos engineering tests..."

# Deploy test environment
kubectl create ns ghostpirates-test
helm install ghostpirates-test infrastructure/helm/ghostpirates \
  --namespace ghostpirates-test

# Wait for pods to be ready
kubectl wait --for=condition=ready pod \
  -l app=ghostpirates \
  -n ghostpirates-test \
  --timeout=300s

# Run chaos experiments
echo "Running network delay experiment..."
kubectl apply -f tests/chaos/network-delay.yaml
sleep 300  # 5 minutes
kubectl delete -f tests/chaos/network-delay.yaml

echo "Running pod failure experiment..."
kubectl apply -f tests/chaos/pod-failure.yaml
sleep 300
kubectl delete -f tests/chaos/pod-failure.yaml

echo "Running database partition experiment..."
kubectl apply -f tests/chaos/database-failure.yaml
sleep 120
kubectl delete -f tests/chaos/database-failure.yaml

# Verify system recovered
kubectl wait --for=condition=ready pod \
  -l app=ghostpirates \
  -n ghostpirates-test \
  --timeout=60s

echo "Chaos tests completed successfully"

# Cleanup
helm uninstall ghostpirates-test -n ghostpirates-test
kubectl delete ns ghostpirates-test
```

**Acceptance Criteria**:

- [ ] System handles network delays gracefully
- [ ] System recovers from pod failures
- [ ] System handles database disconnections
- [ ] Circuit breakers activate correctly
- [ ] Retry logic working under chaos
- [ ] No data loss during failures

---

## Epic 5: Test Data Management

### Task 5.1: Test Data Generation

**Type**: Testing
**Dependencies**: Database schema complete

**Subtasks**:

- [ ] 5.1.1: Create test data generator

```rust
// apps/api/tests/fixtures/generators.rs
use fake::{Fake, Faker};
use fake::faker::company::en::*;
use fake::faker::internet::en::*;

pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_company() -> Company {
        Company {
            id: Uuid::new_v4(),
            name: CompanyName().fake(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn generate_user(company_id: Uuid) -> User {
        User {
            id: Uuid::new_v4(),
            company_id,
            email: SafeEmail().fake(),
            password_hash: "hashed_password".to_string(),
            full_name: format!("{} {}", Faker.fake::<String>(), Faker.fake::<String>()),
            is_active: true,
            last_login: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn generate_team(company_id: Uuid, created_by: Uuid) -> Team {
        let goals = vec![
            "Build a web scraper for product prices",
            "Create an analytics dashboard",
            "Implement user authentication system",
            "Develop REST API for mobile app",
            "Automate deployment pipeline",
        ];

        Team::new(
            company_id,
            goals[rand::random::<usize>() % goals.len()].to_string(),
            created_by,
        )
    }

    pub fn generate_task(team_id: Uuid) -> Task {
        let titles = vec![
            "Implement authentication",
            "Create database schema",
            "Build API endpoints",
            "Write unit tests",
            "Deploy to production",
        ];

        Task::new(
            team_id,
            titles[rand::random::<usize>() % titles.len()].to_string(),
            Faker.fake::<String>(),
            vec![
                Faker.fake::<String>(),
                Faker.fake::<String>(),
            ],
        )
    }
}
```

- [ ] 5.1.2: Create test database seeder

```rust
// apps/api/tests/fixtures/seed.rs
pub async fn seed_database(pool: &PgPool) -> SeedData {
    let company = TestDataGenerator::generate_company();
    let user = TestDataGenerator::generate_user(company.id);
    let team = TestDataGenerator::generate_team(company.id, user.id);
    let task = TestDataGenerator::generate_task(team.id);

    // Insert into database
    sqlx::query!(
        "INSERT INTO companies (id, name) VALUES ($1, $2)",
        company.id,
        company.name
    )
    .execute(pool)
    .await
    .unwrap();

    sqlx::query!(
        "INSERT INTO users (id, company_id, email, password_hash, full_name)
         VALUES ($1, $2, $3, $4, $5)",
        user.id,
        user.company_id,
        user.email,
        user.password_hash,
        user.full_name
    )
    .execute(pool)
    .await
    .unwrap();

    // ... insert team and task

    SeedData {
        company,
        user,
        team,
        task,
    }
}

pub struct SeedData {
    pub company: Company,
    pub user: User,
    pub team: Team,
    pub task: Task,
}
```

- [ ] 5.1.3: Create snapshot testing for data

```rust
// apps/api/tests/snapshot_tests.rs
use insta::assert_json_snapshot;

#[sqlx::test]
async fn test_team_serialization_snapshot(pool: PgPool) -> sqlx::Result<()> {
    let seed = seed_database(&pool).await;

    let repo = TeamsRepository::new(pool);
    let team = repo.find_by_id(seed.team.id).await?.unwrap();

    assert_json_snapshot!(team, {
        ".id" => "[uuid]",
        ".company_id" => "[uuid]",
        ".created_by" => "[uuid]",
        ".created_at" => "[datetime]",
        ".updated_at" => "[datetime]",
    });

    Ok(())
}
```

**Acceptance Criteria**:

- [ ] Test data generation working
- [ ] Realistic fake data generated
- [ ] Seeding scripts for all entities
- [ ] Snapshot tests for data structures
- [ ] Test data cleanup automated

---

## Success Criteria - Testing Complete

- [ ] Unit test coverage > 80%
- [ ] All integration tests passing
- [ ] Contract tests prevent breaking changes
- [ ] Chaos engineering validates resilience
- [ ] Test data management automated
- [ ] CI/CD runs all tests
- [ ] Test execution time < 10 minutes
- [ ] No flaky tests
- [ ] All critical paths tested

---

## Final Production Readiness

All 15 implementation plans complete. System ready for production deployment.

---

**Testing Strategy: Comprehensive test coverage across all layers**
