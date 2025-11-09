# Phase 8: Testing, Performance & Production Polish

**Duration**: Weeks 15-16 (14 days)
**Goal**: Comprehensive Testing → Performance Optimization → Production Readiness
**Dependencies**: Phase 7 complete (Error Recovery)

---

## Epic 1: Integration Testing

### Task 1.1: Backend Integration Tests

**Type**: Testing
**Dependencies**: All backend features complete

**Subtasks**:

- [ ] 1.1.1: Set up test database

```bash
# Create test database
createdb ghostpirates_test

# Set up test environment
cat > .env.test <<EOF
DATABASE_URL=postgresql://localhost/ghostpirates_test
REDIS_URL=redis://localhost:6379/1
CLAUDE_API_KEY=test_key
OPENAI_API_KEY=test_key
JWT_SECRET=test_secret_key_for_testing
EOF
```

- [ ] 1.1.2: Configure sqlx for testing

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
        .expect("DATABASE_URL must be set for tests");

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

pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query!("TRUNCATE TABLE teams, tasks, team_members, messages, checkpoints, cost_tracking, audit_events, escalations CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test database");
}
```

- [ ] 1.1.3: Team creation and management tests

```rust
// apps/api/tests/team_integration_tests.rs
use common::*;

mod common;

#[sqlx::test]
async fn test_create_team_flow(pool: PgPool) -> sqlx::Result<()> {
    let company_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    // Create company
    sqlx::query!(
        "INSERT INTO companies (id, name) VALUES ($1, $2)",
        company_id,
        "Test Company"
    )
    .execute(&pool)
    .await?;

    // Create user
    sqlx::query!(
        "INSERT INTO users (id, company_id, email, password_hash, full_name)
         VALUES ($1, $2, $3, $4, $5)",
        user_id,
        company_id,
        "test@example.com",
        "hashed_password",
        "Test User"
    )
    .execute(&pool)
    .await?;

    // Create team
    let team_service = TeamService::new(pool.clone());
    let team = team_service
        .create_team(company_id, user_id, "Build a web scraper".to_string())
        .await?;

    assert_eq!(team.status, TeamStatus::Pending);
    assert_eq!(team.goal, "Build a web scraper");

    Ok(())
}

#[sqlx::test]
async fn test_team_lifecycle(pool: PgPool) -> sqlx::Result<()> {
    let team_id = create_test_team(&pool).await?;
    let team_service = TeamService::new(pool.clone());

    // Start team
    team_service.start_team(team_id).await?;
    let team = team_service.get_team(team_id).await?.unwrap();
    assert_eq!(team.status, TeamStatus::Planning);

    // Activate team
    team_service.activate_team(team_id).await?;
    let team = team_service.get_team(team_id).await?.unwrap();
    assert_eq!(team.status, TeamStatus::Active);

    // Complete team
    team_service.complete_team(team_id).await?;
    let team = team_service.get_team(team_id).await?.unwrap();
    assert_eq!(team.status, TeamStatus::Completed);
    assert!(team.completed_at.is_some());

    Ok(())
}
```

- [ ] 1.1.4: Task execution and workflow tests

```rust
// apps/api/tests/task_execution_tests.rs
#[sqlx::test]
async fn test_task_assignment_and_completion(pool: PgPool) -> sqlx::Result<()> {
    let team_id = create_test_team(&pool).await?;
    let manager_id = create_test_manager(&pool, team_id).await?;
    let worker_id = create_test_worker(&pool, team_id).await?;

    let task_service = TaskService::new(pool.clone());

    // Create task
    let task = task_service
        .create_task(
            team_id,
            "Implement authentication".to_string(),
            "Add JWT-based authentication".to_string(),
            vec![
                "JWT tokens working".to_string(),
                "Login endpoint functional".to_string(),
            ],
        )
        .await?;

    assert_eq!(task.status, TaskStatus::Pending);

    // Assign task
    task_service.assign_task(task.id, worker_id, manager_id).await?;
    let task = task_service.get_task(task.id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::Assigned);
    assert_eq!(task.assigned_to, Some(worker_id));

    // Start work
    task_service.start_task(task.id).await?;
    let task = task_service.get_task(task.id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);

    // Submit for review
    task_service
        .submit_for_review(task.id, serde_json::json!({"result": "completed"}))
        .await?;
    let task = task_service.get_task(task.id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::Review);

    // Approve
    task_service.approve_task(task.id).await?;
    let task = task_service.get_task(task.id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::Completed);
    assert!(task.completion_time.is_some());

    Ok(())
}

#[sqlx::test]
async fn test_task_revision_workflow(pool: PgPool) -> sqlx::Result<()> {
    let task_id = create_test_task(&pool).await?;
    let task_service = TaskService::new(pool.clone());

    // Submit for review
    task_service
        .submit_for_review(task_id, serde_json::json!({"result": "incomplete"}))
        .await?;

    // Request revision
    task_service
        .request_revision(task_id, "Missing error handling".to_string())
        .await?;

    let task = task_service.get_task(task_id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::RevisionRequested);
    assert_eq!(task.revision_count, 1);

    // Resubmit
    task_service
        .submit_for_review(task_id, serde_json::json!({"result": "now complete"}))
        .await?;

    // Approve
    task_service.approve_task(task_id).await?;
    let task = task_service.get_task(task_id).await?.unwrap();
    assert_eq!(task.status, TaskStatus::Completed);

    Ok(())
}
```

- [ ] 1.1.5: Checkpoint and recovery tests

```rust
// apps/api/tests/recovery_integration_tests.rs
#[sqlx::test]
async fn test_checkpoint_creation_and_resumption(pool: PgPool) -> sqlx::Result<()> {
    let task_id = create_test_task(&pool).await?;
    let checkpoint_manager = CheckpointManager::new(pool.clone());

    // Create checkpoints
    for step in 0..5 {
        checkpoint_manager
            .create_checkpoint(
                task_id,
                step,
                serde_json::json!({"step_output": step}),
                serde_json::json!({"accumulated": format!("Step {}", step)}),
                CheckpointType::Automatic,
            )
            .await?;
    }

    // Get latest checkpoint
    let latest = checkpoint_manager
        .get_latest_checkpoint(task_id)
        .await?
        .unwrap();

    assert_eq!(latest.step_number, 4);

    // Get specific checkpoint
    let checkpoint_2 = checkpoint_manager
        .get_checkpoint_at_step(task_id, 2)
        .await?
        .unwrap();

    assert_eq!(checkpoint_2.step_number, 2);

    Ok(())
}

#[sqlx::test]
async fn test_failure_handling_and_retry(pool: PgPool) -> sqlx::Result<()> {
    let task_id = create_test_task(&pool).await?;
    let checkpoint_manager = CheckpointManager::new(pool.clone());
    let failure_handler = FailureHandler::new(checkpoint_manager.clone());

    // Simulate a retryable failure
    let failure = Failure::new(
        FailureType::NetworkTimeout,
        "API request timed out".to_string(),
    );

    let recovery_action = failure_handler
        .handle_failure(task_id, failure)
        .await
        .unwrap();

    match recovery_action {
        RecoveryAction::Retry { strategy, .. } => {
            assert!(matches!(strategy, RetryStrategy::ExponentialBackoff { .. }));
        }
        _ => panic!("Expected retry action"),
    }

    Ok(())
}
```

- [ ] 1.1.6: Run all integration tests

```bash
# Run integration tests
cargo test --test '*' -- --test-threads=1

# Run with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

**Acceptance Criteria**:

- [ ] All team lifecycle tests pass
- [ ] All task workflow tests pass
- [ ] All checkpoint tests pass
- [ ] All recovery tests pass
- [ ] Test coverage > 80%
- [ ] No flaky tests

---

## Epic 2: End-to-End Tests (Playwright)

### Task 2.1: Frontend E2E Tests

**Type**: Testing
**Dependencies**: Frontend complete

**Subtasks**:

- [ ] 2.1.1: Install Playwright

```bash
cd apps/frontend
npm install -D @playwright/test
npx playwright install
```

- [ ] 2.1.2: Configure Playwright

```typescript
// apps/frontend/playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:3000',
    reuseExistingServer: !process.env.CI,
  },
});
```

- [ ] 2.1.3: Authentication flow tests

```typescript
// apps/frontend/tests/e2e/auth.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Authentication', () => {
  test('should login successfully', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');

    await expect(page).toHaveURL('/dashboard');
    await expect(page.locator('text=Welcome')).toBeVisible();
  });

  test('should show error on invalid credentials', async ({ page }) => {
    await page.goto('/login');

    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'wrongpassword');
    await page.click('button[type="submit"]');

    await expect(page.locator('text=Invalid credentials')).toBeVisible();
  });

  test('should logout successfully', async ({ page }) => {
    // Login first
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Logout
    await page.click('[data-testid="user-menu"]');
    await page.click('text=Logout');

    await expect(page).toHaveURL('/login');
  });
});
```

- [ ] 2.1.4: Team creation flow tests

```typescript
// apps/frontend/tests/e2e/teams.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Team Management', () => {
  test.beforeEach(async ({ page }) => {
    // Login before each test
    await page.goto('/login');
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');
  });

  test('should create a new team', async ({ page }) => {
    await page.goto('/teams');
    await page.click('button:has-text("Create Team")');

    await page.fill('textarea[name="goal"]', 'Build a web scraper for product prices');
    await page.click('button:has-text("Create Team")');

    await expect(page.locator('text=Team created successfully')).toBeVisible();
    await expect(page.locator('text=Build a web scraper')).toBeVisible();
  });

  test('should view team details', async ({ page }) => {
    await page.goto('/teams');
    await page.click('.team-card:first-child');

    await expect(page).toHaveURL(/\/teams\/[0-9a-f-]+/);
    await expect(page.locator('[data-testid="team-goal"]')).toBeVisible();
    await expect(page.locator('[data-testid="team-members"]')).toBeVisible();
    await expect(page.locator('[data-testid="task-list"]')).toBeVisible();
  });

  test('should see real-time team status updates', async ({ page }) => {
    await page.goto('/teams');
    const teamCard = page.locator('.team-card:first-child');

    // Initial status
    await expect(teamCard.locator('[data-testid="status-badge"]')).toHaveText('pending');

    // Wait for status change (this would be triggered by backend)
    await page.waitForSelector('[data-testid="status-badge"]:has-text("active")', {
      timeout: 10000,
    });

    await expect(teamCard.locator('[data-testid="status-badge"]')).toHaveText('active');
  });
});
```

- [ ] 2.1.5: Task workflow tests

```typescript
// apps/frontend/tests/e2e/tasks.spec.ts
test.describe('Task Workflow', () => {
  test.beforeEach(async ({ page }) => {
    await loginAsUser(page);
    await navigateToTeam(page);
  });

  test('should display task list', async ({ page }) => {
    await expect(page.locator('[data-testid="task-list"]')).toBeVisible();
    await expect(page.locator('.task-card')).toHaveCount(3);
  });

  test('should filter tasks by status', async ({ page }) => {
    await page.click('[data-testid="filter-completed"]');
    await expect(page.locator('.task-card')).toHaveCount(1);

    await page.click('[data-testid="filter-in-progress"]');
    await expect(page.locator('.task-card')).toHaveCount(2);
  });

  test('should view task details and history', async ({ page }) => {
    await page.click('.task-card:first-child');

    await expect(page.locator('[data-testid="task-title"]')).toBeVisible();
    await expect(page.locator('[data-testid="task-description"]')).toBeVisible();
    await expect(page.locator('[data-testid="acceptance-criteria"]')).toBeVisible();

    // Check audit history
    await page.click('text=History');
    await expect(page.locator('[data-testid="audit-timeline"]')).toBeVisible();
  });
});
```

- [ ] 2.1.6: WebSocket real-time updates tests

```typescript
// apps/frontend/tests/e2e/realtime.spec.ts
test.describe('Real-time Updates', () => {
  test('should receive WebSocket messages', async ({ page }) => {
    await loginAsUser(page);
    await page.goto('/teams/test-team-id');

    // Wait for WebSocket connection
    await page.waitForSelector('[data-testid="connection-status"]:has-text("Connected")');

    // Trigger an update from backend (via API call in test)
    await triggerTaskUpdate(page);

    // Verify UI updated
    await expect(page.locator('.task-card:first-child [data-testid="status"]'))
      .toHaveText('completed', { timeout: 5000 });
  });

  test('should reconnect after disconnection', async ({ page }) => {
    await loginAsUser(page);
    await page.goto('/teams/test-team-id');

    await expect(page.locator('[data-testid="connection-status"]'))
      .toHaveText('Connected');

    // Simulate network disconnection
    await page.context().setOffline(true);
    await expect(page.locator('[data-testid="connection-status"]'))
      .toHaveText('Disconnected');

    // Restore connection
    await page.context().setOffline(false);
    await expect(page.locator('[data-testid="connection-status"]'))
      .toHaveText('Connected', { timeout: 10000 });
  });
});
```

- [ ] 2.1.7: Run E2E tests

```bash
# Run all E2E tests
npx playwright test

# Run with UI mode
npx playwright test --ui

# Run specific browser
npx playwright test --project=chromium

# Generate report
npx playwright show-report
```

**Acceptance Criteria**:

- [ ] All auth flow tests pass
- [ ] All team management tests pass
- [ ] All task workflow tests pass
- [ ] Real-time update tests pass
- [ ] Tests pass in Chrome, Firefox, Safari
- [ ] No flaky tests
- [ ] Test execution time < 5 minutes

---

## Epic 3: Load Testing

### Task 3.1: Performance and Load Tests

**Type**: Testing
**Dependencies**: All features complete

**Subtasks**:

- [ ] 3.1.1: Install k6

```bash
# macOS
brew install k6

# Or download from https://k6.io/
```

- [ ] 3.1.2: Create API load test scenarios

```javascript
// tests/load/api-load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate } from 'k6/metrics';

const errorRate = new Rate('errors');

export const options = {
  stages: [
    { duration: '30s', target: 10 },   // Ramp up to 10 users
    { duration: '1m', target: 50 },    // Ramp up to 50 users
    { duration: '2m', target: 50 },    // Stay at 50 users
    { duration: '1m', target: 100 },   // Ramp up to 100 users
    { duration: '2m', target: 100 },   // Stay at 100 users
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<500', 'p(99)<1000'], // 95% < 500ms, 99% < 1s
    errors: ['rate<0.1'],                            // Error rate < 10%
    http_req_failed: ['rate<0.05'],                  // Failed requests < 5%
  },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

export function setup() {
  // Login and get token
  const loginRes = http.post(`${BASE_URL}/api/auth/login`, JSON.stringify({
    email: 'loadtest@example.com',
    password: 'password123',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });

  const token = loginRes.json('token');
  return { token };
}

export default function(data) {
  const headers = {
    'Authorization': `Bearer ${data.token}`,
    'Content-Type': 'application/json',
  };

  // List teams
  let res = http.get(`${BASE_URL}/api/teams`, { headers });
  check(res, {
    'teams list status 200': (r) => r.status === 200,
    'teams list duration < 200ms': (r) => r.timings.duration < 200,
  }) || errorRate.add(1);

  sleep(1);

  // Get specific team
  const teams = res.json();
  if (teams.length > 0) {
    res = http.get(`${BASE_URL}/api/teams/${teams[0].id}`, { headers });
    check(res, {
      'team detail status 200': (r) => r.status === 200,
      'team detail duration < 300ms': (r) => r.timings.duration < 300,
    }) || errorRate.add(1);
  }

  sleep(1);

  // List tasks
  if (teams.length > 0) {
    res = http.get(`${BASE_URL}/api/teams/${teams[0].id}/tasks`, { headers });
    check(res, {
      'tasks list status 200': (r) => r.status === 200,
    }) || errorRate.add(1);
  }

  sleep(2);
}
```

- [ ] 3.1.3: Create WebSocket connection load test

```javascript
// tests/load/websocket-load-test.js
import ws from 'k6/ws';
import { check } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 50 },
    { duration: '1m', target: 100 },
    { duration: '2m', target: 100 },
    { duration: '30s', target: 0 },
  ],
};

const BASE_URL = __ENV.WS_URL || 'ws://localhost:4000';

export default function() {
  const url = `${BASE_URL}/ws`;
  const params = {
    headers: {
      'Authorization': `Bearer ${__ENV.TOKEN}`,
    },
  };

  const res = ws.connect(url, params, function(socket) {
    socket.on('open', () => {
      console.log('WebSocket connection opened');

      // Subscribe to team updates
      socket.send(JSON.stringify({
        type: 'subscribe',
        team_id: '00000000-0000-0000-0000-000000000001',
      }));
    });

    socket.on('message', (data) => {
      const message = JSON.parse(data);
      check(message, {
        'message received': (m) => m.type !== undefined,
      });
    });

    socket.on('close', () => {
      console.log('WebSocket connection closed');
    });

    socket.on('error', (e) => {
      console.log('WebSocket error:', e);
    });

    // Keep connection open for 30 seconds
    socket.setTimeout(() => {
      socket.close();
    }, 30000);
  });

  check(res, { 'WebSocket connected': (r) => r && r.status === 101 });
}
```

- [ ] 3.1.4: Create database query performance test

```javascript
// tests/load/db-performance-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 20 },
    { duration: '3m', target: 20 },
  ],
  thresholds: {
    'http_req_duration{endpoint:teams}': ['p(95)<200'],
    'http_req_duration{endpoint:tasks}': ['p(95)<300'],
    'http_req_duration{endpoint:audit}': ['p(95)<500'],
  },
};

const BASE_URL = __ENV.API_URL || 'http://localhost:4000';

export default function(data) {
  const headers = {
    'Authorization': `Bearer ${data.token}`,
  };

  // Test database-heavy endpoints

  // Teams list (with joins)
  let res = http.get(`${BASE_URL}/api/teams?include=members,tasks`, {
    headers,
    tags: { endpoint: 'teams' },
  });
  check(res, { 'teams query fast': (r) => r.timings.duration < 200 });

  sleep(1);

  // Tasks with filters (indexed queries)
  res = http.get(`${BASE_URL}/api/tasks?status=active&limit=50`, {
    headers,
    tags: { endpoint: 'tasks' },
  });
  check(res, { 'tasks query fast': (r) => r.timings.duration < 300 });

  sleep(1);

  // Audit trail (large dataset)
  res = http.get(`${BASE_URL}/api/audit?limit=100`, {
    headers,
    tags: { endpoint: 'audit' },
  });
  check(res, { 'audit query fast': (r) => r.timings.duration < 500 });

  sleep(2);
}
```

- [ ] 3.1.5: Run load tests

```bash
# Run API load test
k6 run tests/load/api-load-test.js

# Run with output to InfluxDB (if available)
k6 run --out influxdb=http://localhost:8086/k6 tests/load/api-load-test.js

# Run WebSocket test
k6 run tests/load/websocket-load-test.js

# Run database performance test
k6 run tests/load/db-performance-test.js
```

**Acceptance Criteria**:

- [ ] System handles 100 concurrent API users
- [ ] 95th percentile response time < 500ms
- [ ] 99th percentile response time < 1s
- [ ] Error rate < 10%
- [ ] WebSocket handles 100 concurrent connections
- [ ] Database queries optimized (no N+1 queries)
- [ ] No memory leaks under sustained load

---

## Epic 4: Performance Profiling

### Task 4.1: Profile and Optimize

**Type**: Performance
**Dependencies**: Load tests identifying bottlenecks

**Subtasks**:

- [ ] 4.1.1: Install profiling tools

```bash
# Install flamegraph
cargo install flamegraph

# Install perf (Linux) or Instruments (macOS)
```

- [ ] 4.1.2: Profile Rust backend

```bash
# Generate flamegraph
cargo flamegraph --bin ghostpirates-api

# Profile with perf
perf record -F 99 -g target/release/ghostpirates-api
perf report

# Profile memory allocations
cargo build --release
valgrind --tool=massif target/release/ghostpirates-api
```

- [ ] 4.1.3: Identify and fix N+1 queries

```rust
// Before: N+1 query problem
for team in teams {
    let members = get_team_members(team.id).await?; // N queries
}

// After: Single query with join
let teams_with_members = sqlx::query!(
    r#"
    SELECT
        t.id, t.name,
        json_agg(json_build_object(
            'id', m.id,
            'agent_id', m.agent_id,
            'role', m.role
        )) as members
    FROM teams t
    LEFT JOIN team_members m ON m.team_id = t.id
    WHERE t.company_id = $1
    GROUP BY t.id
    "#,
    company_id
)
.fetch_all(&pool)
.await?;
```

- [ ] 4.1.4: Add database query caching

```rust
// apps/api/src/infrastructure/cache/query_cache.rs
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct QueryCache {
    redis: redis::Client,
}

impl QueryCache {
    pub fn new(redis_url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            redis: redis::Client::open(redis_url)?,
        })
    }

    pub async fn get_or_set<T, F>(
        &self,
        key: &str,
        ttl: Duration,
        fetch: F,
    ) -> Result<T, CacheError>
    where
        T: Serialize + for<'de> Deserialize<'de>,
        F: std::future::Future<Output = Result<T, sqlx::Error>>,
    {
        let mut conn = self.redis.get_async_connection().await?;

        // Try cache first
        if let Ok(cached) = conn.get::<_, String>(key).await {
            if let Ok(value) = serde_json::from_str(&cached) {
                return Ok(value);
            }
        }

        // Cache miss - fetch from database
        let value = fetch.await?;

        // Store in cache
        let serialized = serde_json::to_string(&value)?;
        conn.set_ex(key, serialized, ttl.as_secs() as usize).await?;

        Ok(value)
    }

    pub async fn invalidate(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.redis.get_async_connection().await?;
        conn.del(key).await?;
        Ok(())
    }
}
```

- [ ] 4.1.5: Optimize database indexes

```sql
-- Check for missing indexes
SELECT
    schemaname,
    tablename,
    attname,
    null_frac,
    avg_width,
    n_distinct
FROM pg_stats
WHERE schemaname = 'public'
ORDER BY null_frac DESC;

-- Add missing indexes based on query patterns
CREATE INDEX CONCURRENTLY idx_tasks_team_status
ON tasks(team_id, status)
WHERE status != 'completed';

CREATE INDEX CONCURRENTLY idx_messages_team_created
ON messages(team_id, created_at DESC);

CREATE INDEX CONCURRENTLY idx_audit_team_type_created
ON audit_events(team_id, event_type, created_at DESC);
```

- [ ] 4.1.6: Optimize frontend bundle

```bash
# Analyze bundle size
cd apps/frontend
npm run build
npx webpack-bundle-analyzer .next/analyze/client.json

# Optimize images
npm install sharp
# Update next.config.js to use sharp for image optimization

# Enable code splitting
# Components lazy loaded with next/dynamic
```

**Acceptance Criteria**:

- [ ] Flamegraph shows no obvious hotspots
- [ ] No N+1 query patterns
- [ ] Database queries use appropriate indexes
- [ ] Cache hit rate > 70% for common queries
- [ ] Frontend bundle size < 500KB (gzipped)
- [ ] First Contentful Paint < 1.5s
- [ ] Time to Interactive < 3s

---

## Epic 5: Production Deployment Checklist

### Task 5.1: Production Readiness

**Type**: DevOps
**Dependencies**: All testing complete

**Subtasks**:

- [ ] 5.1.1: Create production environment checklist

```markdown
# Production Deployment Checklist

## Security
- [ ] All secrets stored in Azure Key Vault
- [ ] JWT secret is cryptographically random (>256 bits)
- [ ] HTTPS enabled with valid SSL certificate
- [ ] CORS configured for production domain only
- [ ] Rate limiting enabled
- [ ] SQL injection protection verified
- [ ] XSS protection headers set
- [ ] CSRF protection enabled

## Database
- [ ] All migrations tested
- [ ] Database backups configured (daily)
- [ ] Point-in-time recovery enabled
- [ ] Connection pooling optimized (max_connections)
- [ ] Indexes created for all common queries
- [ ] VACUUM and ANALYZE scheduled

## Monitoring
- [ ] Application logging configured (structured JSON)
- [ ] Error tracking set up (Sentry/similar)
- [ ] Metrics collection enabled (Prometheus/similar)
- [ ] Alerting rules configured
- [ ] Database slow query logging enabled
- [ ] API response time monitoring
- [ ] WebSocket connection monitoring

## Performance
- [ ] Redis caching enabled
- [ ] CDN configured for static assets
- [ ] Database query performance validated
- [ ] Load testing completed successfully
- [ ] Memory usage profiled and optimized
- [ ] Auto-scaling configured

## Resilience
- [ ] Health check endpoint working
- [ ] Readiness probe configured
- [ ] Graceful shutdown implemented
- [ ] Circuit breakers tested
- [ ] Retry logic validated
- [ ] Backup/restore procedure documented

## Documentation
- [ ] API documentation complete (OpenAPI/Swagger)
- [ ] Deployment runbook created
- [ ] Rollback procedure documented
- [ ] Incident response plan ready
- [ ] Architecture diagrams updated
```

- [ ] 5.1.2: Set up health checks

```rust
// apps/api/src/api/handlers/health.rs
use axum::Json;
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    checks: HealthChecks,
}

#[derive(Serialize)]
pub struct HealthChecks {
    database: String,
    redis: String,
}

pub async fn health_check(
    State(pool): State<PgPool>,
    State(redis): State<RedisClient>,
) -> Json<HealthResponse> {
    let db_status = check_database(&pool).await;
    let redis_status = check_redis(&redis).await;

    Json(HealthResponse {
        status: if db_status == "ok" && redis_status == "ok" {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: db_status,
            redis: redis_status,
        },
    })
}

async fn check_database(pool: &PgPool) -> String {
    match sqlx::query("SELECT 1").fetch_one(pool).await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
}

async fn check_redis(client: &RedisClient) -> String {
    match client.ping().await {
        Ok(_) => "ok".to_string(),
        Err(e) => format!("error: {}", e),
    }
}
```

- [ ] 5.1.3: Configure structured logging

```rust
// apps/api/src/main.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            "ghostpirates_api=info,tower_http=info".into()
        }))
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}
```

- [ ] 5.1.4: Add graceful shutdown

```rust
// apps/api/src/main.rs
use tokio::signal;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Shutdown signal received, starting graceful shutdown");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup code ...

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Cleanup
    pool.close().await;
    tracing::info!("Shutdown complete");

    Ok(())
}
```

- [ ] 5.1.5: Configure production environment variables

```bash
# .env.production
DATABASE_URL=postgresql://user:pass@prod-db.postgres.database.azure.com/ghostpirates?sslmode=require
REDIS_URL=redis://prod-redis.redis.cache.windows.net:6380?ssl=true
JWT_SECRET=${VAULT_JWT_SECRET}
CLAUDE_API_KEY=${VAULT_CLAUDE_API_KEY}
OPENAI_API_KEY=${VAULT_OPENAI_API_KEY}
RUST_LOG=ghostpirates_api=info,tower_http=info
ENVIRONMENT=production
SENTRY_DSN=https://...@sentry.io/...
```

- [ ] 5.1.6: Create deployment script

```bash
#!/bin/bash
# scripts/deploy-production.sh

set -e

echo "Starting production deployment..."

# Build backend
echo "Building backend..."
cd apps/api
cargo build --release
cargo test --release

# Build frontend
echo "Building frontend..."
cd ../frontend
npm run build
npm run test:e2e

# Run database migrations
echo "Running database migrations..."
sqlx migrate run --database-url $DATABASE_URL

# Deploy to Azure
echo "Deploying to Azure..."
az containerapp update \
  --name ghostpirates-api \
  --resource-group ghostpirates-prod \
  --image ghostpirates-api:latest

# Health check
echo "Waiting for health check..."
sleep 10
curl -f https://api.ghostpirates.com/health || exit 1

echo "Deployment complete!"
```

**Acceptance Criteria**:

- [ ] Health check endpoint returns 200
- [ ] All checklist items verified
- [ ] Structured logging working
- [ ] Graceful shutdown tested
- [ ] Production environment variables set
- [ ] Deployment script tested in staging
- [ ] Rollback procedure documented and tested

---

## Success Criteria - Phase 8 Complete

- [ ] All integration tests passing (> 80% coverage)
- [ ] All E2E tests passing in all browsers
- [ ] Load tests show system handles 100+ concurrent users
- [ ] Performance profiling complete, bottlenecks resolved
- [ ] Database queries optimized
- [ ] Frontend bundle optimized
- [ ] Production deployment checklist complete
- [ ] Health checks working
- [ ] Monitoring and logging configured
- [ ] Graceful shutdown implemented
- [ ] System production-ready

---

## Next Steps

Proceed to [12-cost-optimization.md](./12-cost-optimization.md) for cost optimization strategies.

---

**Phase 8: Testing, performance, and production readiness complete**
