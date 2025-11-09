# infrastructure/terraform/modules/core/outputs.tf

output "resource_group_name" {
  description = "Name of the resource group"
  value       = azurerm_resource_group.main.name
}

output "resource_group_location" {
  description = "Location of the resource group"
  value       = azurerm_resource_group.main.location
}

output "storage_account_name" {
  description = "Name of the storage account"
  value       = azurerm_storage_account.main.name
}

output "storage_account_primary_key" {
  description = "Primary access key for storage account"
  value       = azurerm_storage_account.main.primary_access_key
  sensitive   = true
}

output "redis_host" {
  description = "Redis hostname (if created)"
  value       = var.create_redis ? azurerm_redis_cache.main[0].hostname : "localhost"
}

output "redis_port" {
  description = "Redis port"
  value       = var.create_redis ? azurerm_redis_cache.main[0].port : 6379
}

output "redis_primary_key" {
  description = "Redis primary access key (if created)"
  value       = var.create_redis ? azurerm_redis_cache.main[0].primary_access_key : ""
  sensitive   = true
}
