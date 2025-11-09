# Ghost Pirates - Azure Development Environment

**Cost-Optimized Azure Infrastructure for Testing**

## ⚠️ Important

This environment is **OPTIONAL** and for Azure testing only. Continue using local Docker for day-to-day development.

**Cost:** ~$30-40/month if left running, ~$2-5/month with on-demand deployment

## When to Use This

- Testing Azure PostgreSQL compatibility
- Testing cloud deployment before production
- Verifying Terraform infrastructure code
- Experimenting with Azure services

## Quick Start

### Prerequisites

```bash
# Install Azure CLI
brew install azure-cli

# Install Terraform
brew install terraform

# Login to Azure
az login
```

### Deploy Infrastructure

```bash
# 1. Copy example variables
cd infrastructure/terraform/environments/dev
cp terraform.tfvars.example terraform.tfvars

# 2. Edit terraform.tfvars with your password
nano terraform.tfvars  # Set postgres_admin_password

# 3. Initialize Terraform
terraform init

# 4. Preview changes
terraform plan

# 5. Deploy (takes ~10 minutes)
terraform apply -auto-approve
```

### Connect to Azure Database

```bash
# Get connection info
terraform output database_fqdn
terraform output -raw database_connection_string

# Connect with psql
PGPASSWORD='YourPassword' psql -h <FQDN> -U postgres -d ghostpirates

# Update .env for API
DATABASE_URL=<connection_string_from_output>
```

### Run Migrations

```bash
cd apps/api
sqlx migrate run
```

### Destroy to Save Costs

**IMPORTANT:** Destroy when not actively testing!

```bash
terraform destroy -auto-approve  # Takes ~5 minutes
```

Redeploy anytime with `terraform apply -auto-approve`

## Cost Breakdown

| Resource             | SKU              | Monthly Cost | Notes                |
| -------------------- | ---------------- | ------------ | -------------------- |
| PostgreSQL Server    | B_Standard_B1ms  | ~$30         | 1 vCore, 1GB RAM     |
| Storage Account      | Standard LRS     | ~$1          | Minimal usage        |
| Redis                | -                | $0           | Using local Docker   |
| **Total (24/7)**     |                  | **~$31**     | If left running      |
| **Total (On-Demand)**|                  | **~$2-5**    | Deploy only when needed |

## Testing Infrastructure

Verify deployment:

```bash
# Check all resources
terraform output

# Test database connection
terraform output -raw database_connection_string | xargs psql

# Verify resource group in Azure Portal
az group show --name ghostpirates-dev
```

## Troubleshooting

### Terraform init fails

```bash
# Remove lock file and retry
rm .terraform.lock.hcl
terraform init
```

### Database connection refused

- Check firewall rules allow your IP
- Verify public_network_access_enabled = true
- Wait 5-10 minutes for server to fully provision

### Can't destroy resources

```bash
# Force delete resource group (WARNING: Deletes everything)
az group delete --name ghostpirates-dev --yes --no-wait
```

## Switching Between Local and Azure

**Use Local (Default):**
```bash
# .env
DATABASE_URL=postgresql://postgres:postgres@localhost:54320/ghostpirates_dev
```

**Use Azure:**
```bash
# .env
DATABASE_URL=<from terraform output>
```

## Next Steps

When ready for production:
1. Create `environments/production/`
2. Use stronger SKU (GP_Standard_D2s_v3+)
3. Enable high availability
4. Use private networking
5. Set prevent_destroy = true
