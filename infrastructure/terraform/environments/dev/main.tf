# infrastructure/terraform/environments/dev/main.tf
# Azure Development Environment (Cost-Optimized)
#
# This environment is for testing Azure deployment when ready.
# Currently focusing on local development - deploy this only when needed.

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.80"
    }
  }
}

provider "azurerm" {
  features {
    resource_group {
      prevent_deletion_if_contains_resources = false # Dev only
    }
  }

  skip_provider_registration = true
}

# Core resources (Resource Group, Storage)
module "core" {
  source = "../../modules/core"

  environment = "dev"
  location    = var.location

  # Cost optimization: Use local Docker Redis instead of Azure
  # Run: docker run -d -p 6379:6379 redis:7-alpine
  create_redis = false
}

# PostgreSQL Database
module "database" {
  source = "../../modules/database"

  environment         = "dev"
  location            = var.location
  resource_group_name = module.core.resource_group_name

  # Cost optimization: B_Standard_B1ms (~$30/month)
  # 1 vCore, 1GB RAM - sufficient for development
  sku_name   = "B_Standard_B1ms"
  storage_mb = 32768 # 32GB minimum

  admin_username = "postgres"
  admin_password = var.postgres_admin_password

  postgres_version = "16"

  # Dev settings
  backup_retention_days        = 7
  geo_redundant_backup_enabled = false
  high_availability_enabled    = false

  # Development only: Allow all IPs for testing
  # TODO: Restrict to specific IPs in production
  public_network_access_enabled = true
  allow_azure_services          = true

  tags = {
    Purpose = "Development Testing"
    Note    = "Destroy when not in use to save costs"
  }
}

# Outputs
output "resource_group_name" {
  description = "Name of the resource group"
  value       = module.core.resource_group_name
}

output "database_fqdn" {
  description = "PostgreSQL server FQDN"
  value       = module.database.server_fqdn
}

output "database_connection_string" {
  description = "PostgreSQL connection string"
  value       = module.database.connection_string
  sensitive   = true
}

output "storage_account_name" {
  description = "Storage account name"
  value       = module.core.storage_account_name
}

output "redis_info" {
  description = "Redis connection info (using local Docker)"
  value       = "redis://localhost:6379 (local Docker)"
}

output "deployment_instructions" {
  description = "Instructions for using this environment"
  sensitive   = true
  value       = <<-EOT
    Azure Development Environment Deployed
    =====================================

    Connect to PostgreSQL:
      PGPASSWORD='${var.postgres_admin_password}' psql -h ${module.database.server_fqdn} -U postgres -d ghostpirates

    Update .env with Azure connection:
      DATABASE_URL=${module.database.connection_string}
      REDIS_URL=redis://localhost:6379

    Run migrations:
      cd apps/api
      sqlx migrate run

    Cost Savings:
      - Destroy when not in use: terraform destroy -auto-approve
      - Redeploy when needed: terraform apply -auto-approve
      - Estimated cost: ~$30-40/month if left running 24/7
      - Estimated cost: ~$2-5/month with on-demand deployment
  EOT
}
