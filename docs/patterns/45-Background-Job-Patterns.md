# Pattern 45: Background Job & Task Queue Patterns

**Version**: 1.0
**Last Updated**: October 8, 2025
**Status**: Active

---

## Table of Contents

1. [Overview](#overview)
2. [BullMQ Integration](#bullmq-integration)
3. [Job Types & Patterns](#job-types--patterns)
4. [Job Scheduling](#job-scheduling)
5. [Job Prioritization](#job-prioritization)
6. [Error Handling & Retries](#error-handling--retries)
7. [Progress Tracking](#progress-tracking)
8. [Job Coordination](#job-coordination)
9. [Monitoring & Debugging](#monitoring--debugging)
10. [Best Practices](#best-practices)

---

## Overview

Background jobs handle time-consuming tasks asynchronously, improving responsiveness and enabling scheduled operations.

### Use Cases in WellOS

- **Email Sending**: Welcome emails, password resets, notifications
- **Report Generation**: PDF invoices, time sheets, financial reports
- **Data Aggregation**: Daily revenue calculations, time tracking summaries
- **Cleanup Jobs**: Expired tokens, old audit logs, soft-deleted records
- **External API Calls**: QuickBooks sync, payment processing
- **Bulk Operations**: Mass user imports, batch invoice generation

---

## Background Job Integration with Tokio

### 1. Setup

```bash
cargo add tokio redis serde serde_json chrono uuid
```

```rust
// job_queue.rs
use redis::{Client, Commands, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct JobQueue {
    redis_client: Arc<Mutex<Connection>>,
    queue_name: String,
}

impl JobQueue {
    pub fn new(redis_url: &str, queue_name: &str) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        let conn = client.get_connection()?;

        Ok(Self {
            redis_client: Arc::new(Mutex::new(conn)),
            queue_name: queue_name.to_string(),
        })
    }

    pub async fn enqueue<T: Serialize>(
        &self,
        job_type: &str,
        data: &T,
        options: JobOptions,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let job_id = uuid::Uuid::new_v4().to_string();
        let job = Job {
            id: job_id.clone(),
            job_type: job_type.to_string(),
            data: serde_json::to_value(data)?,
            attempts: 0,
            max_attempts: options.max_attempts,
            priority: options.priority,
            delay_ms: options.delay_ms,
            created_at: chrono::Utc::now(),
        };

        let job_json = serde_json::to_string(&job)?;
        let mut conn = self.redis_client.lock().await;

        // Add to queue (priority queue using sorted set)
        let score = self.calculate_score(&job);
        conn.zadd(&self.queue_name, job_json, score)?;

        Ok(job_id)
    }

    pub async fn enqueue_bulk<T: Serialize>(
        &self,
        jobs: Vec<(&str, T, JobOptions)>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut job_ids = Vec::new();

        for (job_type, data, options) in jobs {
            let job_id = self.enqueue(job_type, &data, options).await?;
            job_ids.push(job_id);
        }

        Ok(job_ids)
    }

    fn calculate_score(&self, job: &Job) -> i64 {
        let now = chrono::Utc::now().timestamp_millis();
        let delay = job.delay_ms.unwrap_or(0);
        let priority_offset = (job.priority as i64) * 1000;

        now + delay + priority_offset
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: String,
    pub data: serde_json::Value,
    pub attempts: u32,
    pub max_attempts: u32,
    pub priority: JobPriority,
    pub delay_ms: Option<i64>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum JobPriority {
    Critical = 1,
    High = 3,
    Normal = 5,
    Low = 7,
    Bulk = 10,
}

pub struct JobOptions {
    pub max_attempts: u32,
    pub priority: JobPriority,
    pub delay_ms: Option<i64>,
}

impl Default for JobOptions {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            priority: JobPriority::Normal,
            delay_ms: None,
        }
    }
}
```

### 2. Creating an Email Queue

```rust
// email_queue.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendEmailJobData {
    pub to: String,
    pub subject: String,
    pub template: String,
    pub data: HashMap<String, serde_json::Value>,
}

pub struct EmailQueue {
    job_queue: JobQueue,
}

impl EmailQueue {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let job_queue = JobQueue::new(redis_url, "email")?;
        Ok(Self { job_queue })
    }

    pub async fn send_welcome_email(
        &self,
        user_id: &str,
        email: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("userId".to_string(), serde_json::json!(user_id));

        let job_data = SendEmailJobData {
            to: email.to_string(),
            subject: "Welcome to WellOS".to_string(),
            template: "welcome".to_string(),
            data,
        };

        self.job_queue
            .enqueue(
                "send-welcome-email",
                &job_data,
                JobOptions {
                    max_attempts: 3,
                    priority: JobPriority::Normal,
                    delay_ms: None,
                },
            )
            .await
    }

    pub async fn send_password_reset_email(
        &self,
        email: &str,
        token: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut data = HashMap::new();
        data.insert("token".to_string(), serde_json::json!(token));

        let job_data = SendEmailJobData {
            to: email.to_string(),
            subject: "Reset Your Password".to_string(),
            template: "password-reset".to_string(),
            data,
        };

        self.job_queue
            .enqueue(
                "send-password-reset",
                &job_data,
                JobOptions {
                    max_attempts: 3,
                    priority: JobPriority::High, // High priority
                    delay_ms: None,
                },
            )
            .await
    }

    pub async fn send_bulk_emails(
        &self,
        emails: Vec<SendEmailJobData>,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let jobs: Vec<_> = emails
            .into_iter()
            .map(|email| {
                (
                    "send-email",
                    email,
                    JobOptions {
                        max_attempts: 3,
                        priority: JobPriority::Normal,
                        delay_ms: None,
                    },
                )
            })
            .collect();

        self.job_queue.enqueue_bulk(jobs).await
    }
}
```

### 3. Processing Jobs

```rust
// email_processor.rs
use tokio::time::{sleep, Duration};

pub struct EmailProcessor {
    job_queue: JobQueue,
    email_service: Arc<EmailService>,
}

impl EmailProcessor {
    pub fn new(
        redis_url: &str,
        email_service: Arc<EmailService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let job_queue = JobQueue::new(redis_url, "email")?;
        Ok(Self {
            job_queue,
            email_service,
        })
    }

    pub async fn start(&self) {
        loop {
            match self.process_next_job().await {
                Ok(processed) => {
                    if !processed {
                        // No jobs available, sleep briefly
                        sleep(Duration::from_millis(100)).await;
                    }
                }
                Err(e) => {
                    eprintln!("Error processing job: {:?}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    }

    async fn process_next_job(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Dequeue job from Redis
        let job = match self.dequeue_job().await? {
            Some(job) => job,
            None => return Ok(false),
        };

        println!("Processing job {} of type {}", job.id, job.job_type);

        let result = match job.job_type.as_str() {
            "send-welcome-email" => self.handle_welcome_email(&job).await,
            "send-password-reset" => self.handle_password_reset(&job).await,
            "send-email" => self.handle_email(&job).await,
            _ => {
                eprintln!("Unknown job type: {}", job.job_type);
                Ok(())
            }
        };

        match result {
            Ok(_) => {
                println!("Job {} completed successfully", job.id);
                self.mark_job_complete(&job).await?;
            }
            Err(e) => {
                eprintln!("Job {} failed: {:?}", job.id, e);
                self.handle_job_failure(&job).await?;
            }
        }

        Ok(true)
    }

    async fn handle_welcome_email(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let data: SendEmailJobData = serde_json::from_value(job.data.clone())?;
        println!("Processing welcome email for: {}", data.to);
        self.email_service.send_welcome_email(&data.to, &data.data).await
    }

    async fn handle_password_reset(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let data: SendEmailJobData = serde_json::from_value(job.data.clone())?;
        let token = data.data.get("token")
            .and_then(|v| v.as_str())
            .ok_or("Missing token")?;
        self.email_service.send_password_reset_email(&data.to, token).await
    }

    async fn handle_email(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let data: SendEmailJobData = serde_json::from_value(job.data.clone())?;
        self.email_service.send(&data.to, &data.subject, &data.template, &data.data).await
    }

    async fn dequeue_job(&self) -> Result<Option<Job>, Box<dyn std::error::Error>> {
        // Implementation to dequeue from Redis sorted set
        // Returns the job with the lowest score (highest priority, oldest first)
        todo!("Implement Redis dequeue logic")
    }

    async fn mark_job_complete(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        // Remove job from Redis
        todo!("Implement job completion logic")
    }

    async fn handle_job_failure(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        // Retry or move to dead letter queue
        todo!("Implement retry/DLQ logic")
    }
}
```

---

## Job Types & Patterns

### 1. One-Time Jobs

```rust
// report_queue.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateInvoiceJob {
    pub invoice_id: String,
    pub format: String,
}

pub struct ReportQueue {
    job_queue: JobQueue,
}

impl ReportQueue {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let job_queue = JobQueue::new(redis_url, "reports")?;
        Ok(Self { job_queue })
    }

    pub async fn generate_invoice(&self, invoice_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let job_data = GenerateInvoiceJob {
            invoice_id: invoice_id.to_string(),
            format: "pdf".to_string(),
        };

        self.job_queue
            .enqueue(
                "generate-invoice",
                &job_data,
                JobOptions::default(),
            )
            .await
    }
}

// report_processor.rs
pub struct ReportProcessor {
    job_queue: JobQueue,
    invoice_service: Arc<InvoiceService>,
    s3_service: Arc<S3Service>,
}

impl ReportProcessor {
    pub async fn handle_generate_invoice(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let data: GenerateInvoiceJob = serde_json::from_value(job.data.clone())?;

        // Generate PDF
        let pdf_buffer = self.invoice_service.generate_pdf(&data.invoice_id).await?;

        // Upload to S3
        let key = format!("invoices/{}.{}", data.invoice_id, data.format);
        self.s3_service.upload_file(&pdf_buffer, &key).await?;

        Ok(())
    }
}
```

### 2. Recurring Jobs (Cron)

```rust
// scheduled_jobs.rs
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

pub struct ScheduledJobs {
    scheduler: JobScheduler,
    cleanup_queue: JobQueue,
}

impl ScheduledJobs {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        let cleanup_queue = JobQueue::new(redis_url, "cleanup")?;

        Ok(Self {
            scheduler,
            cleanup_queue,
        })
    }

    pub async fn schedule_cleanup_jobs(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Run daily at 2 AM
        let cleanup_tokens_job = CronJob::new_async("0 2 * * *", |_uuid, _l| {
            Box::pin(async move {
                println!("Running cleanup-expired-tokens job");
            })
        })?;
        self.scheduler.add(cleanup_tokens_job).await?;

        // Run every hour
        let cleanup_users_job = CronJob::new_async("0 * * * *", |_uuid, _l| {
            Box::pin(async move {
                println!("Running cleanup-pending-users job");
            })
        })?;
        self.scheduler.add(cleanup_users_job).await?;

        self.scheduler.start().await?;
        Ok(())
    }
}

// cleanup_processor.rs
pub struct CleanupProcessor {
    token_repository: Arc<TokenRepository>,
    command_bus: Arc<CommandBus>,
}

impl CleanupProcessor {
    pub async fn cleanup_expired_tokens(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let deleted = self.token_repository.delete_expired().await?;
        println!("Cleaned up {} expired tokens", deleted);
        Ok(deleted)
    }

    pub async fn cleanup_pending_users(&self) -> Result<u64, Box<dyn std::error::Error>> {
        let result = self.command_bus.execute(ExpirePendingUsersCommand {}).await?;
        Ok(result.count)
    }
}
```

### 3. Delayed Jobs

```rust
// notification_queue.rs
use chrono::{DateTime, Utc};

pub struct NotificationQueue {
    queue: JobQueue,
}

impl NotificationQueue {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "notifications")?;
        Ok(Self { queue })
    }

    pub async fn send_reminder(
        &self,
        user_id: &str,
        reminder_text: &str,
        delay_ms: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let data = serde_json::json!({
            "userId": user_id,
            "reminderText": reminder_text,
        });

        self.queue
            .enqueue(
                "send-reminder",
                &data,
                JobOptions {
                    delay_ms: Some(delay_ms),
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn send_trial_expiration_warning(
        &self,
        user_id: &str,
        trial_ends_at: DateTime<Utc>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let three_days_before_ms = trial_ends_at.timestamp_millis() - (3 * 24 * 60 * 60 * 1000);
        let delay_ms = three_days_before_ms - Utc::now().timestamp_millis();

        if delay_ms > 0 {
            let data = serde_json::json!({ "userId": user_id });
            let job_id = self
                .queue
                .enqueue(
                    "trial-expiration-warning",
                    &data,
                    JobOptions {
                        delay_ms: Some(delay_ms),
                        ..Default::default()
                    },
                )
                .await?;
            Ok(Some(job_id))
        } else {
            Ok(None)
        }
    }
}
```

### 4. Child Jobs (Job Flows)

```rust
// job_flow_service.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobFlow {
    pub name: String,
    pub queue_name: String,
    pub data: serde_json::Value,
    pub children: Vec<JobFlow>,
}

pub struct JobFlowService {
    redis_client: Arc<Mutex<redis::Connection>>,
    queues: HashMap<String, JobQueue>,
}

impl JobFlowService {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_connection()?;

        Ok(Self {
            redis_client: Arc::new(Mutex::new(conn)),
            queues: HashMap::new(),
        })
    }

    pub async fn process_monthly_invoicing(
        &mut self,
        organization_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create a flow of dependent jobs
        let flow = JobFlow {
            name: "monthly-invoicing".to_string(),
            queue_name: "invoicing".to_string(),
            data: serde_json::json!({ "organizationId": organization_id }),
            children: vec![JobFlow {
                name: "calculate-billable-hours".to_string(),
                queue_name: "calculations".to_string(),
                data: serde_json::json!({ "organizationId": organization_id }),
                children: vec![
                    JobFlow {
                        name: "generate-invoice-pdf".to_string(),
                        queue_name: "reports".to_string(),
                        data: serde_json::json!({ "organizationId": organization_id }),
                        children: vec![],
                    },
                    JobFlow {
                        name: "send-invoice-email".to_string(),
                        queue_name: "email".to_string(),
                        data: serde_json::json!({ "organizationId": organization_id }),
                        children: vec![],
                    },
                ],
            }],
        };

        self.execute_flow(&flow).await?;
        Ok(())
    }

    async fn execute_flow(&mut self, flow: &JobFlow) -> Result<(), Box<dyn std::error::Error>> {
        // Execute parent job first, then children
        let queue = self
            .queues
            .entry(flow.queue_name.clone())
            .or_insert_with(|| {
                JobQueue::new("redis://localhost:6379", &flow.queue_name).unwrap()
            });

        queue
            .enqueue(&flow.name, &flow.data, JobOptions::default())
            .await?;

        // Execute children
        for child in &flow.children {
            self.execute_flow(child).await?;
        }

        Ok(())
    }
}
```

---

## Job Scheduling

### 1. Cron Jobs

```rust
// scheduler_service.rs
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};

pub struct SchedulerService {
    scheduler: JobScheduler,
    queue: JobQueue,
}

impl SchedulerService {
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        let queue = JobQueue::new(redis_url, "scheduled-tasks")?;

        Ok(Self { scheduler, queue })
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Daily report at 8 AM (America/New_York timezone)
        let daily_report = CronJob::new_async("0 8 * * *", |_uuid, _l| {
            Box::pin(async move {
                println!("Running daily-revenue-report");
            })
        })?;
        self.scheduler.add(daily_report).await?;

        // Weekly summary on Mondays at 9 AM
        let weekly_summary = CronJob::new_async("0 9 * * 1", |_uuid, _l| {
            Box::pin(async move {
                println!("Running weekly-summary");
            })
        })?;
        self.scheduler.add(weekly_summary).await?;

        // Monthly billing on 1st of month at midnight
        let monthly_billing = CronJob::new_async("0 0 1 * *", |_uuid, _l| {
            Box::pin(async move {
                println!("Running monthly-billing");
            })
        })?;
        self.scheduler.add(monthly_billing).await?;

        // Every 15 minutes
        let sync_entries = CronJob::new_async("*/15 * * * *", |_uuid, _l| {
            Box::pin(async move {
                println!("Running sync-time-entries");
            })
        })?;
        self.scheduler.add(sync_entries).await?;

        self.scheduler.start().await?;
        Ok(())
    }

    pub async fn remove_scheduled_job(&mut self, job_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Job removal logic
        println!("Removing scheduled job: {}", job_name);
        Ok(())
    }
}
```

### 2. Dynamic Scheduling

```rust
// dynamic_scheduler.rs
use tokio_cron_scheduler::{Job as CronJob, JobScheduler};
use std::collections::HashMap;

pub struct DynamicScheduler {
    scheduler: JobScheduler,
    queue: JobQueue,
    organization_repository: Arc<OrganizationRepository>,
    scheduled_jobs: HashMap<String, uuid::Uuid>,
}

impl DynamicScheduler {
    pub async fn new(
        redis_url: &str,
        organization_repository: Arc<OrganizationRepository>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let scheduler = JobScheduler::new().await?;
        let queue = JobQueue::new(redis_url, "dynamic")?;

        Ok(Self {
            scheduler,
            queue,
            organization_repository,
            scheduled_jobs: HashMap::new(),
        })
    }

    pub async fn schedule_organization_reports(
        &mut self,
        organization_id: &str,
        schedule: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Store schedule preference in database
        self.organization_repository
            .update_report_schedule(organization_id, schedule)
            .await?;

        // Schedule job with user-defined cron
        let org_id = organization_id.to_string();
        let cron_job = CronJob::new_async(schedule, move |_uuid, _l| {
            let org_id = org_id.clone();
            Box::pin(async move {
                println!("Running org-report for: {}", org_id);
            })
        })?;

        let job_uuid = self.scheduler.add(cron_job).await?;
        self.scheduled_jobs
            .insert(format!("org-report:{}", organization_id), job_uuid);

        Ok(())
    }

    pub async fn update_schedule(
        &mut self,
        organization_id: &str,
        new_schedule: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Remove old schedule
        let job_key = format!("org-report:{}", organization_id);
        if let Some(job_uuid) = self.scheduled_jobs.remove(&job_key) {
            self.scheduler.remove(&job_uuid).await?;
        }

        // Add new schedule
        self.schedule_organization_reports(organization_id, new_schedule)
            .await?;

        Ok(())
    }
}
```

---

## Job Prioritization

```rust
// prioritized_queue.rs (already using Rust JobPriority enum from earlier)

pub struct PrioritizedQueue {
    queue: JobQueue,
}

impl PrioritizedQueue {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "tasks")?;
        Ok(Self { queue })
    }

    pub async fn add_critical_task(
        &self,
        data: &impl Serialize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.queue
            .enqueue(
                "critical-task",
                data,
                JobOptions {
                    priority: JobPriority::Critical,
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn add_user_action(
        &self,
        data: &impl Serialize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.queue
            .enqueue(
                "user-action",
                data,
                JobOptions {
                    priority: JobPriority::High,
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn add_background_sync(
        &self,
        data: &impl Serialize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.queue
            .enqueue(
                "background-sync",
                data,
                JobOptions {
                    priority: JobPriority::Normal,
                    ..Default::default()
                },
            )
            .await
    }

    pub async fn add_cleanup(
        &self,
        data: &impl Serialize,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.queue
            .enqueue(
                "cleanup",
                data,
                JobOptions {
                    priority: JobPriority::Low,
                    delay_ms: Some(60000), // Wait 1 minute before processing
                    ..Default::default()
                },
            )
            .await
    }
}
```

---

## Error Handling & Retries

### 1. Exponential Backoff

```typescript
await this.queue.add(
  'unreliable-api-call',
  { url: 'https://api.example.com/data' },
  {
    attempts: 5,
    backoff: {
      type: 'exponential',
      delay: 2000, // Start with 2s, then 4s, 8s, 16s, 32s
    },
  },
);
```

### 2. Custom Retry Logic

```rust
// external_api_processor.rs
impl ExternalApiProcessor {
    pub async fn handle_external_api_call(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let attempt = job.attempts + 1;

        match self.external_api_service.call(&job.data["url"].as_str().unwrap()).await {
            Ok(response) => Ok(()),
            Err(error) => {
                if self.is_retryable_error(&error) {
                    if attempt < 5 {
                        Err(error) // Will retry
                    } else {
                        // Max retries reached, send to dead letter queue
                        let dlq_data = serde_json::json!({
                            "originalJob": job.data,
                            "error": error.to_string(),
                            "attempts": attempt,
                        });

                        self.dead_letter_queue
                            .enqueue("failed-api-call", &dlq_data, JobOptions::default())
                            .await?;

                        Err("Max retries exceeded".into())
                    }
                } else {
                    // Non-retryable error (e.g., 400 Bad Request)
                    eprintln!("Non-retryable error: {:?}", error);
                    Err(error)
                }
            }
        }
    }

    fn is_retryable_error(&self, error: &Box<dyn std::error::Error>) -> bool {
        // Retry on network errors, 5xx errors, rate limits
        let error_str = error.to_string();
        error_str.contains("ECONNRESET")
            || error_str.contains("ETIMEDOUT")
            || error_str.contains("status: 5")
            || error_str.contains("status: 429")
    }
}
```

### 3. Dead Letter Queue

```rust
// dead_letter_queue_handler.rs
pub struct DeadLetterQueueHandler {
    dlq: JobQueue,
    audit_service: Arc<AuditService>,
    notification_service: Arc<NotificationService>,
}

impl DeadLetterQueueHandler {
    pub fn new(
        redis_url: &str,
        audit_service: Arc<AuditService>,
        notification_service: Arc<NotificationService>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let dlq = JobQueue::new(redis_url, "dead-letter")?;
        Ok(Self {
            dlq,
            audit_service,
            notification_service,
        })
    }

    pub async fn handle_failed_job(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        let original_job = &job.data["originalJob"];
        let job_id = original_job["id"].as_str().unwrap();
        let job_name = original_job["name"].as_str().unwrap();
        let error = job.data["error"].as_str().unwrap();
        let attempts = job.data["attempts"].as_u64().unwrap();

        // Log failure
        self.audit_service
            .log_job_failure(JobFailureLog {
                job_id: job_id.to_string(),
                job_name: job_name.to_string(),
                error: error.to_string(),
                attempts,
                failed_at: chrono::Utc::now(),
            })
            .await?;

        // Notify admin
        let body = format!(
            "Job {} failed after {} attempts.\n\nError: {}",
            job_name, attempts, error
        );
        self.notification_service
            .notify_admin(
                "Job Failed After All Retries".to_string(),
                body,
            )
            .await?;

        Ok(())
    }

    pub async fn retry_failed_job(&self, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Manually retry a job from DLQ
        // Implementation depends on your specific DLQ storage mechanism
        println!("Retrying failed job: {}", job_id);
        Ok(())
    }
}
```

---

## Progress Tracking

### 1. Job Progress Updates

```rust
// report_processor.rs
#[derive(Debug, Deserialize)]
struct GenerateReportData {
    report_type: String,
    date_range: DateRange,
}

impl ReportProcessor {
    pub async fn handle_generate_report(&self, job: &mut Job) -> Result<ReportResult, Box<dyn std::error::Error>> {
        let data: GenerateReportData = serde_json::from_value(job.data.clone())?;

        // Step 1: Fetch data (20%)
        self.update_job_progress(job, 0).await?;
        let report_data = self.fetch_report_data(&data.report_type, &data.date_range).await?;
        self.update_job_progress(job, 20).await?;

        // Step 2: Process data (40%)
        let processed = self.process_data(report_data).await?;
        self.update_job_progress(job, 60).await?;

        // Step 3: Generate PDF (20%)
        let pdf = self.generate_pdf(&processed).await?;
        self.update_job_progress(job, 80).await?;

        // Step 4: Upload (10%)
        let key = format!("reports/{}.pdf", job.id);
        let url = self.s3_service.upload_file(&pdf, &key).await?;
        self.update_job_progress(job, 90).await?;

        // Step 5: Send notification (10%)
        let message = format!("Report ready: {}", url);
        self.notification_service.notify(&message).await?;
        self.update_job_progress(job, 100).await?;

        Ok(ReportResult {
            url,
            size: pdf.len(),
        })
    }

    async fn update_job_progress(&self, job: &mut Job, progress: u8) -> Result<(), Box<dyn std::error::Error>> {
        // Update progress in Redis
        println!("Job {} progress: {}%", job.id, progress);
        Ok(())
    }
}

struct ReportResult {
    url: String,
    size: usize,
}
```

### 2. Client-Side Progress Monitoring

```rust
// Axum API endpoint to get job status
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct JobStatusResponse {
    id: String,
    state: String,
    progress: u8,
    data: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<String>,
}

async fn get_job_status(
    Path(job_id): Path<String>,
    State(queue): State<Arc<JobQueue>>,
) -> Result<Json<JobStatusResponse>, (StatusCode, String)> {
    let job = queue
        .get_job(&job_id)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, "Job not found".to_string()))?;

    let response = JobStatusResponse {
        id: job.id.clone(),
        state: job.state.to_string(),
        progress: job.progress,
        data: job.data.clone(),
        result: job.result.clone(),
        error: job.error.clone(),
    };

    Ok(Json(response))
}

// Router setup
pub fn job_routes() -> Router {
    Router::new()
        .route("/jobs/:job_id/status", get(get_job_status))
}
```

---

## Job Coordination

### 1. Job Locking (Prevent Duplicates)

```rust
// unique_job_service.rs
use redis::Commands;

pub struct UniqueJobService {
    queue: JobQueue,
    redis_client: Arc<Mutex<redis::Connection>>,
}

impl UniqueJobService {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "unique-tasks")?;
        let client = redis::Client::open(redis_url)?;
        let conn = client.get_connection()?;

        Ok(Self {
            queue,
            redis_client: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn add_unique_job(
        &self,
        job_name: &str,
        data: &impl Serialize,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let data_json = serde_json::to_string(data)?;
        let lock_key = format!("job-lock:{}:{}", job_name, data_json);
        let lock_ttl = 3600; // 1 hour

        // Try to acquire lock using SET NX EX
        let mut conn = self.redis_client.lock().await;
        let acquired: bool = redis::cmd("SET")
            .arg(&lock_key)
            .arg("1")
            .arg("EX")
            .arg(lock_ttl)
            .arg("NX")
            .query(&mut *conn)?;

        if !acquired {
            println!("Job already queued, skipping");
            return Ok(None);
        }

        // Add job with lock key as job ID
        let job_id = self
            .queue
            .enqueue(job_name, data, JobOptions::default())
            .await?;

        Ok(Some(job_id))
    }
}
```

### 2. Rate Limiting

```rust
// rate_limited_queue.rs
use tokio::sync::Semaphore;
use std::time::Duration;

pub struct RateLimitedQueue {
    queue: JobQueue,
    semaphore: Arc<Semaphore>,
}

impl RateLimitedQueue {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "rate-limited")?;
        let semaphore = Arc::new(Semaphore::new(10)); // Max 10 concurrent

        Ok(Self { queue, semaphore })
    }

    pub async fn add_job(&self, data: &impl Serialize) -> Result<String, Box<dyn std::error::Error>> {
        self.queue
            .enqueue("limited-job", data, JobOptions::default())
            .await
    }
}

// rate_limited_processor.rs
pub struct RateLimitedProcessor {
    semaphore: Arc<Semaphore>,
}

impl RateLimitedProcessor {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(2)), // Concurrency: 2
        }
    }

    pub async fn handle_job(&self, job: &Job) -> Result<(), Box<dyn std::error::Error>> {
        // Acquire semaphore permit (limits concurrency to 2)
        let _permit = self.semaphore.acquire().await?;

        // Process job
        self.process_job(&job.data).await?;

        // Permit automatically released when _permit is dropped
        Ok(())
    }

    async fn process_job(&self, data: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        // Job processing logic
        println!("Processing job with data: {:?}", data);
        Ok(())
    }
}
```

### 3. Job Groups

```rust
// batch_job_service.rs
use tokio::time::{sleep, Duration};

pub struct BatchJobService {
    queue: JobQueue,
}

impl BatchJobService {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "batch")?;
        Ok(Self { queue })
    }

    pub async fn process_batch(
        &self,
        items: Vec<serde_json::Value>,
        batch_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create jobs for each item
        let jobs: Vec<_> = items
            .into_iter()
            .map(|item| {
                (
                    "process-item",
                    serde_json::json!({
                        "item": item,
                        "batchId": batch_id,
                    }),
                    JobOptions::default(),
                )
            })
            .collect();

        // Enqueue all jobs
        self.queue.enqueue_bulk(jobs).await?;

        // Wait for all jobs in batch to complete
        self.wait_for_batch(batch_id).await?;

        Ok(())
    }

    async fn wait_for_batch(&self, batch_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // Check if any jobs for this batch are still active
            let active_jobs = self.queue.get_active_jobs().await?;
            let batch_jobs: Vec<_> = active_jobs
                .iter()
                .filter(|j| j.data["batchId"] == batch_id)
                .collect();

            if batch_jobs.is_empty() {
                break;
            }

            // Sleep for 1 second before checking again
            sleep(Duration::from_secs(1)).await;
        }

        Ok(())
    }
}
```

---

## Monitoring & Debugging

### 1. Job Events & Logging

```rust
// job_monitoring_service.rs
use tracing::{info, error, warn, debug};

pub struct JobMonitoringService {
    queue: JobQueue,
}

impl JobMonitoringService {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let queue = JobQueue::new(redis_url, "monitored")?;
        Ok(Self { queue })
    }

    pub async fn start_monitoring(&self) {
        // In Rust, we would use channels or event streams to monitor job events
        // This is a simplified example showing the concept
        info!("Job monitoring started");
    }

    pub fn log_job_started(&self, job_id: &str) {
        info!("Job {} started", job_id);
    }

    pub fn log_job_completed(&self, job_id: &str, result: &serde_json::Value) {
        info!("Job {} completed: {:?}", job_id, result);
    }

    pub fn log_job_failed(&self, job_id: &str, error: &str) {
        error!("Job {} failed: {}", job_id, error);
    }

    pub fn log_job_stalled(&self, job_id: &str) {
        warn!("Job {} stalled", job_id);
    }

    pub fn log_job_progress(&self, job_id: &str, progress: u8) {
        debug!("Job {} progress: {}%", job_id, progress);
    }

    pub async fn get_queue_metrics(&self) -> Result<QueueMetrics, Box<dyn std::error::Error>> {
        // Fetch metrics from Redis
        let waiting = self.queue.get_waiting_count().await?;
        let active = self.queue.get_active_count().await?;
        let completed = self.queue.get_completed_count().await?;
        let failed = self.queue.get_failed_count().await?;
        let delayed = self.queue.get_delayed_count().await?;

        Ok(QueueMetrics {
            waiting,
            active,
            completed,
            failed,
            delayed,
            total: waiting + active + completed + failed + delayed,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct QueueMetrics {
    pub waiting: u64,
    pub active: u64,
    pub completed: u64,
    pub failed: u64,
    pub delayed: u64,
    pub total: u64,
}
```

### 2. Bull Board (Admin UI)

```bash
pnpm add @bull-board/express @bull-board/api
```

```rust
// queue_admin_routes.rs - Axum admin panel for job queues
use axum::{
    Router,
    routing::get,
    extract::State,
    response::Html,
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
struct QueueStats {
    name: String,
    waiting: u64,
    active: u64,
    completed: u64,
    failed: u64,
}

async fn queue_admin_panel(
    State(queues): State<Arc<Vec<Arc<JobQueue>>>>,
) -> Html<String> {
    // Simple HTML admin panel
    let html = r#"
        <html>
        <head><title>Queue Admin</title></head>
        <body>
            <h1>Job Queue Administration</h1>
            <div id="queues"></div>
            <script>
                fetch('/admin/queues/stats')
                    .then(r => r.json())
                    .then(data => {
                        document.getElementById('queues').innerHTML =
                            data.map(q => `
                                <div>
                                    <h2>${q.name}</h2>
                                    <p>Waiting: ${q.waiting}</p>
                                    <p>Active: ${q.active}</p>
                                    <p>Completed: ${q.completed}</p>
                                    <p>Failed: ${q.failed}</p>
                                </div>
                            `).join('');
                    });
            </script>
        </body>
        </html>
    "#;
    Html(html.to_string())
}

async fn queue_stats(
    State(queues): State<Arc<Vec<Arc<JobQueue>>>>,
) -> Json<Vec<QueueStats>> {
    let mut stats = Vec::new();

    for queue in queues.iter() {
        stats.push(QueueStats {
            name: queue.queue_name.clone(),
            waiting: queue.get_waiting_count().await.unwrap_or(0),
            active: queue.get_active_count().await.unwrap_or(0),
            completed: queue.get_completed_count().await.unwrap_or(0),
            failed: queue.get_failed_count().await.unwrap_or(0),
        });
    }

    Json(stats)
}

pub fn admin_routes() -> Router {
    Router::new()
        .route("/admin/queues", get(queue_admin_panel))
        .route("/admin/queues/stats", get(queue_stats))
}
```

---

## Best Practices

### ✅ Job Design Checklist

- [ ] Keep jobs idempotent (safe to retry)
- [ ] Make jobs atomic (all or nothing)
- [ ] Store minimal data in job payload
- [ ] Set appropriate timeouts
- [ ] Implement proper error handling
- [ ] Use job IDs for deduplication
- [ ] Clean up completed jobs periodically
- [ ] Monitor queue size and processing time

### ✅ Performance Checklist

- [ ] Use concurrency appropriately
- [ ] Batch similar jobs
- [ ] Implement rate limiting for external APIs
- [ ] Use job priorities effectively
- [ ] Monitor memory usage
- [ ] Scale workers horizontally
- [ ] Use separate queues for different job types
- [ ] Implement circuit breakers for failing jobs

### ✅ Reliability Checklist

- [ ] Configure retries with backoff
- [ ] Implement dead letter queue
- [ ] Monitor failed jobs
- [ ] Set up alerts for queue buildup
- [ ] Log all job failures
- [ ] Handle worker crashes gracefully
- [ ] Use Redis persistence
- [ ] Test job recovery scenarios

---

## Related Patterns

- **Pattern 12**: [Observer Pattern](./12-Observer-Pattern.md)
- **Pattern 15**: [Retry Pattern](./15-Retry-Pattern.md)
- **Pattern 43**: [WebSocket & Real-Time Patterns](./43-WebSocket-RealTime-Patterns.md)
- **Pattern 46**: [Caching Strategy Patterns](./46-Caching-Strategy-Patterns.md)

---

## References

- [BullMQ Documentation](https://docs.bullmq.io/)
- [NestJS Bull Integration](https://docs.nestjs.com/techniques/queues)
- [Redis Documentation](https://redis.io/documentation)
- [Job Queue Patterns](https://www.enterpriseintegrationpatterns.com/)

---

**Last Updated**: October 8, 2025
**Version**: 1.0
**Status**: Active
