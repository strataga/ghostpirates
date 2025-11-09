# infrastructure/terraform/modules/core/main.tf
# Core Azure Resources (Resource Group, Storage, etc.)

resource "azurerm_resource_group" "main" {
  name     = "ghostpirates-${var.environment}"
  location = var.location

  tags = {
    Environment = var.environment
    ManagedBy   = "Terraform"
    Project     = "GhostPirates"
  }
}

# Storage account for logs, backups, etc.
resource "azurerm_storage_account" "main" {
  name                     = "ghostpirates${var.environment}"
  resource_group_name      = azurerm_resource_group.main.name
  location                 = azurerm_resource_group.main.location
  account_tier             = "Standard"
  account_replication_type = var.storage_replication_type

  tags = {
    Environment = var.environment
    ManagedBy   = "Terraform"
  }
}

# Optional: Redis Cache (can use local Docker instead)
resource "azurerm_redis_cache" "main" {
  count = var.create_redis ? 1 : 0

  name                = "ghostpirates-redis-${var.environment}"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  capacity            = 0
  family              = "C"
  sku_name            = "Basic"

  non_ssl_port_enabled = true # For development only

  tags = {
    Environment = var.environment
    ManagedBy   = "Terraform"
  }
}
