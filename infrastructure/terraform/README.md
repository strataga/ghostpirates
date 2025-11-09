# Ghost Pirates - Infrastructure as Code

Terraform configuration for Ghost Pirates autonomous AI teams platform.

## Architecture

```
infrastructure/terraform/
├── versions.tf                    # Provider versions
├── environments/
│   ├── local/                     # Local Docker (current)
│   │   ├── main.tf               # Documents local setup
│   │   └── README.md
│   └── dev/                       # Azure dev (optional)
│       ├── main.tf               # Composes modules
│       ├── variables.tf
│       ├── terraform.tfvars.example
│       └── README.md
└── modules/
    ├── core/                      # Resource group, storage
    │   ├── main.tf
    │   ├── variables.tf
    │   └── outputs.tf
    └── database/                  # PostgreSQL
        ├── main.tf
        ├── variables.tf
        └── outputs.tf
```

## Quick Start

### Local Development (Recommended)

**Cost: $0/month**

```bash
# Start local services
docker compose up -d

# View Terraform documentation
cd infrastructure/terraform/environments/local
terraform init
terraform output

# See: environments/local/README.md for full guide
```

### Azure Development (Optional)

**Cost: ~$2-5/month with on-demand deployment**

```bash
# Deploy Azure infrastructure
cd infrastructure/terraform/environments/dev
cp terraform.tfvars.example terraform.tfvars
# Edit terraform.tfvars with password

terraform init
terraform plan
terraform apply -auto-approve

# Destroy when not in use (saves 98% cost)
terraform destroy -auto-approve

# See: environments/dev/README.md for full guide
```

## Environments

| Environment | Purpose                  | Cost/Month | Status   |
| ----------- | ------------------------ | ---------- | -------- |
| **local**   | Daily development        | $0         | ✅ Active |
| **dev**     | Azure testing (optional) | $2-5       | 📦 Ready |
| staging     | Pre-production           | TBD        | ⏳ Future |
| production  | Live system              | TBD        | ⏳ Future |

## Modules

### Core Module

Creates foundational Azure resources:
- Resource Group
- Storage Account
- Redis Cache (optional - use local Docker instead)

### Database Module

Creates PostgreSQL Flexible Server:
- Configurable SKU (B1ms for dev, higher for prod)
- High availability (optional)
- Automatic backups
- Firewall rules

## Design Principles

### 1. Local-First Development

Default to local Docker for day-to-day work:
- Faster iteration
- Zero cost
- Works offline
- No cloud dependencies

### 2. Azure-Ready Infrastructure

When ready for cloud:
- Infrastructure as Code (Terraform)
- Cost-optimized for experimentation
- On-demand deployment (destroy when not using)
- Production-ready modules

### 3. Environment Parity

Same code runs everywhere:
- Local Docker
- Azure development
- Future production

Only configuration changes (connection strings, SKUs).

### 4. Cost Optimization

For experimental project:
- Minimal SKUs (B_Standard_B1ms)
- On-demand deployment
- Local Docker for Redis
- No high availability in dev

**Monthly Costs:**
- Local: $0
- Azure Dev (on-demand): $2-5
- Azure Dev (24/7): $30-40

## Workflow

### Daily Development (Local)

```bash
# 1. Start services
docker compose up -d

# 2. Run migrations
cd apps/api && sqlx migrate run

# 3. Start API
cargo run

# No Terraform needed for local dev!
```

### Azure Testing (When Needed)

```bash
# 1. Deploy infrastructure
cd infrastructure/terraform/environments/dev
terraform apply -auto-approve

# 2. Update .env with Azure connection
DATABASE_URL=<from terraform output>

# 3. Run migrations
cd apps/api && sqlx migrate run

# 4. Test with Azure PostgreSQL
cargo run

# 5. Destroy to save costs
terraform destroy -auto-approve
```

## Best Practices

### Security

- **Never commit `terraform.tfvars`** (in .gitignore)
- Use strong passwords (16+ characters)
- Restrict firewall rules in production
- Enable private networking for production

### Cost Management

- **Destroy dev environment** when not actively testing
- Use B1ms SKU for development
- Disable geo-redundant backups in dev
- Use local Docker Redis instead of Azure

### Infrastructure Changes

1. Make changes in modules or environments
2. Run `terraform plan` to preview
3. Run `terraform apply` to deploy
4. Commit `.tf` files to git
5. **Never commit `.tfvars` files!**

## Troubleshooting

### Terraform init fails

```bash
rm -rf .terraform .terraform.lock.hcl
terraform init
```

### Can't connect to Azure database

- Check firewall rules
- Verify password in terraform.tfvars
- Wait 5-10 minutes after deployment
- Check public_network_access_enabled = true

### High costs

```bash
# Destroy immediately!
terraform destroy -auto-approve

# Only keep running when actively testing
```

## Next Steps

1. ✅ Use local development (current setup)
2. 📚 Learn Terraform basics
3. 🧪 Test Azure deployment when needed
4. 🚀 Plan production environment (future)

## Documentation

- [Local Environment Guide](./environments/local/README.md)
- [Azure Dev Environment Guide](./environments/dev/README.md)
- [Terraform Best Practices](https://www.terraform.io/docs/language/index.html)
- [Azure PostgreSQL Docs](https://docs.microsoft.com/en-us/azure/postgresql/)

## Support

- Check READMEs in each environment folder
- Review module documentation
- See main project README.md
- Azure costs: `az consumption usage list`
