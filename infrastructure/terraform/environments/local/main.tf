# infrastructure/terraform/environments/local/main.tf
# Local Development Environment (Docker-based)
#
# This configuration documents the local development setup using Docker Compose.
# No actual Terraform resources are created - this serves as Infrastructure as Code
# documentation for the local environment.

terraform {
  required_version = ">= 1.5.0"
}

# Local outputs for documentation purposes
locals {
  environment = "local"

  # Docker Compose Services (postgres, redis)
  database = {
    host     = "localhost"
    port     = 54320
    name     = "ghostpirates_dev"
    user     = "postgres"
    password = "postgres" # From docker-compose.yml
  }

  redis = {
    host = "localhost"
    port = 6379
  }

  # Local Development Servers
  api = {
    port = 3000
    cmd  = "cd apps/api && cargo run"
  }

  web = {
    port = 3001
    cmd  = "cd apps/web && pnpm dev"
    note = "Next.js web app (planned for future sprint)"
  }
}

# Output the local configuration
output "database_url" {
  description = "Local PostgreSQL connection string"
  value       = "postgresql://${local.database.user}:${local.database.password}@${local.database.host}:${local.database.port}/${local.database.name}"
  sensitive   = true
}

output "redis_url" {
  description = "Local Redis connection string"
  value       = "redis://${local.redis.host}:${local.redis.port}"
}

output "api_url" {
  description = "Local API server URL"
  value       = "http://localhost:${local.api.port}"
}

output "web_url" {
  description = "Local web app URL (when implemented)"
  value       = "http://localhost:${local.web.port}"
}

output "services_status" {
  description = "Status of all local services"
  value = {
    docker_services = {
      postgres = "localhost:${local.database.port}"
      redis    = "localhost:${local.redis.port}"
      status   = "docker compose ps"
    }
    development_servers = {
      api  = "http://localhost:${local.api.port}"
      web  = "http://localhost:${local.web.port} (planned)"
    }
  }
}

output "instructions" {
  description = "Instructions for local development"
  value       = <<-EOT
    Ghost Pirates - Local Development Environment
    =============================================

    1. Start Docker Services (PostgreSQL + Redis)
    ────────────────────────────────────────────
      docker compose up -d
      docker compose ps  # Verify running

    2. Run Database Migrations
    ───────────────────────────
      cd apps/api
      sqlx migrate run

    3. Start API Server (Rust)
    ──────────────────────────
      cd apps/api
      cargo run
      # API available at: http://localhost:3000

    4. Start Web App (Next.js - Future Sprint)
    ───────────────────────────────────────────
      cd apps/web
      pnpm install
      pnpm dev
      # Web available at: http://localhost:3001

    Services Running
    ────────────────
      ✅ PostgreSQL:  localhost:54320
      ✅ Redis:       localhost:6379
      ✅ API:         http://localhost:3000
      ⏳ Web:         http://localhost:3001 (planned)

    Environment Configuration (.env)
    ────────────────────────────────
      DATABASE_URL=postgresql://postgres:postgres@localhost:54320/ghostpirates_dev
      REDIS_URL=redis://localhost:6379
      JWT_SECRET=your-secret-key-here-generate-with-openssl-rand-hex-32
      RUST_LOG=info

    Testing
    ───────
      cargo test                              # All tests
      cargo test --test repository_integration  # Repository tests
      cargo test --test api_integration          # E2E API tests

    Cost: $0/month (100% local)
  EOT
}
