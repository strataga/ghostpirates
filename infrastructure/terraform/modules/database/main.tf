# infrastructure/terraform/modules/database/main.tf
# PostgreSQL Database Module (Azure Flexible Server)

resource "azurerm_postgresql_flexible_server" "main" {
  name                = "ghostpirates-${var.environment}-${var.server_name}"
  resource_group_name = var.resource_group_name
  location            = var.location

  # Cost optimization: B_Standard_B1ms for dev (~$30/month)
  # Production should use GP_Standard_D2s_v3 or higher
  sku_name   = var.sku_name
  storage_mb = var.storage_mb
  version    = var.postgres_version

  administrator_login    = var.admin_username
  administrator_password = var.admin_password

  backup_retention_days        = var.backup_retention_days
  geo_redundant_backup_enabled = var.geo_redundant_backup_enabled

  # High availability (only for production)
  dynamic "high_availability" {
    for_each = var.high_availability_enabled ? [1] : []
    content {
      mode = "ZoneRedundant"
    }
  }

  tags = merge(
    var.tags,
    {
      Environment = var.environment
      ManagedBy   = "Terraform"
      Project     = "GhostPirates"
    }
  )

  lifecycle {
    prevent_destroy = false # Set to true for production
    ignore_changes = [
      tags["CreatedDate"],
      zone
    ]
  }
}

# Create the main database
resource "azurerm_postgresql_flexible_server_database" "main" {
  name      = var.database_name
  server_id = azurerm_postgresql_flexible_server.main.id
  charset   = "UTF8"
  collation = "en_US.utf8"
}

# Enable PostgreSQL extensions
resource "azurerm_postgresql_flexible_server_configuration" "extensions" {
  name      = "azure.extensions"
  server_id = azurerm_postgresql_flexible_server.main.id
  value     = "UUID-OSSP,PGCRYPTO"
}

# Firewall rule to allow Azure services (for Container Apps, etc.)
resource "azurerm_postgresql_flexible_server_firewall_rule" "azure_services" {
  count = var.allow_azure_services ? 1 : 0

  name             = "AllowAzureServices"
  server_id        = azurerm_postgresql_flexible_server.main.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "0.0.0.0"
}

# Firewall rule for development (allow all IPs - DEV ONLY!)
resource "azurerm_postgresql_flexible_server_firewall_rule" "dev_access" {
  count = var.public_network_access_enabled ? 1 : 0

  name             = "AllowAll"
  server_id        = azurerm_postgresql_flexible_server.main.id
  start_ip_address = "0.0.0.0"
  end_ip_address   = "255.255.255.255"
}
