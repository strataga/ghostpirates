# infrastructure/terraform/modules/database/outputs.tf

output "server_id" {
  description = "ID of the PostgreSQL server"
  value       = azurerm_postgresql_flexible_server.main.id
}

output "server_fqdn" {
  description = "Fully qualified domain name of the PostgreSQL server"
  value       = azurerm_postgresql_flexible_server.main.fqdn
}

output "database_name" {
  description = "Name of the database"
  value       = azurerm_postgresql_flexible_server_database.main.name
}

output "connection_string" {
  description = "PostgreSQL connection string"
  value       = "postgresql://${var.admin_username}:${var.admin_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/${azurerm_postgresql_flexible_server_database.main.name}"
  sensitive   = true
}

output "admin_username" {
  description = "Admin username"
  value       = var.admin_username
}
