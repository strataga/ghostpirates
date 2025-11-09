# Ghost Pirates Infrastructure

Infrastructure as Code for the Ghost Pirates autonomous AI teams platform.

## Structure

```
infrastructure/
├── terraform/              # Terraform IaC
│   ├── environments/       # Environment-specific configs
│   │   ├── local/         # Local Docker (active)
│   │   └── dev/           # Azure dev (optional)
│   └── modules/           # Reusable Terraform modules
│       ├── core/          # Resource group, storage
│       └── database/      # PostgreSQL
└── README.md              # This file
```

## Current Setup

**Active Environment:** Local Development (Docker Compose)

- **PostgreSQL**: localhost:54320
- **Redis**: localhost:6379
- **API**: http://localhost:3000
- **Cost**: $0/month

## Quick Start

### Local Development (Recommended)

```bash
# Start services
docker compose up -d

# Run migrations
cd apps/api
sqlx migrate run

# Start API
cargo run
```

### View Infrastructure Documentation

```bash
cd infrastructure/terraform/environments/local
terraform init
terraform output
```

## Terraform Documentation

All infrastructure is documented as Terraform code:

- **[Terraform Root README](./terraform/README.md)** - Main Terraform guide
- **[Local Environment](./terraform/environments/local/README.md)** - Docker setup
- **[Dev Environment](./terraform/environments/dev/README.md)** - Azure (optional)

## Future Environments

| Environment  | Status     | Cost/Month | Purpose                |
| ------------ | ---------- | ---------- | ---------------------- |
| **local**    | ✅ Active  | $0         | Daily development      |
| **dev**      | 📦 Ready   | $2-5       | Azure testing          |
| staging      | ⏳ Planned | TBD        | Pre-production         |
| production   | ⏳ Planned | TBD        | Live deployment        |

## When to Use Azure (dev environment)

The Azure dev environment is **optional** and should only be used for:

- Testing Azure PostgreSQL compatibility
- Validating cloud deployment
- Experimenting with Azure services
- Load testing at scale

**Continue using local Docker for daily development!**

## Documentation

- [Terraform Overview](./terraform/README.md)
- [Local Setup Guide](./terraform/environments/local/README.md)
- [Azure Dev Guide](./terraform/environments/dev/README.md)
- [Module Documentation](./terraform/modules/)

## Cost Management

### Local Development: $0/month

All services run in Docker:
- PostgreSQL 16
- Redis 7
- No cloud costs

### Azure Development: $2-5/month (on-demand)

Only deploy when actively testing:
```bash
# Deploy
terraform apply -auto-approve

# Test...

# Destroy to save costs!
terraform destroy -auto-approve
```

**If left running 24/7:** ~$30-40/month

## Support

See environment-specific READMEs or main project documentation.
