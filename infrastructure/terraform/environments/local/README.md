# Ghost Pirates - Local Development Environment

**100% Docker-based, Zero Cost**

## Quick Start

### 1. Start Docker Services

```bash
# Start PostgreSQL and Redis
docker compose up -d

# Verify containers are running
docker compose ps
```

### 2. Run Database Migrations

```bash
cd apps/api
sqlx migrate run
```

### 3. Start API Server

```bash
cd apps/api
cargo run
# API running at http://localhost:3000
```

### 4. Start Web App (Future Sprint)

```bash
# When implemented:
cd apps/web
pnpm install
pnpm dev
# Web running at http://localhost:3001
```

## Services

| Service    | Port  | URL                       | Status         |
| ---------- | ----- | ------------------------- | -------------- |
| PostgreSQL | 54320 | localhost:54320           | ✅ Docker      |
| Redis      | 6379  | localhost:6379            | ✅ Docker      |
| API        | 3000  | http://localhost:3000     | ✅ Rust/Cargo  |
| Web        | 3001  | http://localhost:3001     | ⏳ Future      |

## Environment Configuration

Create `apps/api/.env`:

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:54320/ghostpirates_dev
JWT_SECRET=your-secret-key-here-generate-with-openssl-rand-hex-32
RUST_LOG=info
```

## Terraform Documentation

This environment doesn't create real Terraform resources - it documents the local Docker setup as IaC.

View configuration:

```bash
cd infrastructure/terraform/environments/local
terraform init
terraform plan  # Shows local configuration
terraform output  # Display connection strings
```

## Database Access

```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U postgres -d ghostpirates_dev

# Or use psql directly
PGPASSWORD=postgres psql -h localhost -p 54320 -U postgres -d ghostpirates_dev
```

## Redis Access

```bash
# Connect to Redis
docker compose exec redis redis-cli

# Or use redis-cli directly
redis-cli -p 6379
```

## Testing

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test repository_integration

# Run E2E API tests
cargo test --test api_integration
```

## Troubleshooting

### Port 54320 already in use

```bash
# Find what's using the port
lsof -i :54320

# Stop docker compose and restart
docker compose down
docker compose up -d
```

### Database migration errors

```bash
# Reset database
docker compose down -v  # WARNING: Deletes all data
docker compose up -d
cd apps/api
sqlx migrate run
```

## Cost

**$0/month** - Everything runs locally in Docker

## Next Steps

When ready for cloud deployment, see:
- `../dev/` - Azure development environment
- `../../docs/DEPLOYMENT_GUIDE.md` - Full deployment guide
