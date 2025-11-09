# infrastructure/terraform/modules/database/variables.tf

variable "environment" {
  description = "Environment name (local, dev, staging, production)"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "eastus"
}

variable "resource_group_name" {
  description = "Name of the resource group"
  type        = string
}

variable "server_name" {
  description = "Name suffix for the PostgreSQL server"
  type        = string
  default     = "db"
}

variable "postgres_version" {
  description = "PostgreSQL version"
  type        = string
  default     = "16"
}

variable "sku_name" {
  description = "SKU for PostgreSQL server"
  type        = string
  default     = "B_Standard_B1ms" # 1 vCore, 1GB RAM, ~$30/month
}

variable "storage_mb" {
  description = "Storage in MB"
  type        = number
  default     = 32768 # 32GB minimum
}

variable "admin_username" {
  description = "PostgreSQL admin username"
  type        = string
  default     = "postgres"
}

variable "admin_password" {
  description = "PostgreSQL admin password"
  type        = string
  sensitive   = true
}

variable "database_name" {
  description = "Name of the database to create"
  type        = string
  default     = "ghostpirates"
}

variable "backup_retention_days" {
  description = "Backup retention in days"
  type        = number
  default     = 7
}

variable "geo_redundant_backup_enabled" {
  description = "Enable geo-redundant backups"
  type        = bool
  default     = false
}

variable "high_availability_enabled" {
  description = "Enable high availability (zone redundant)"
  type        = bool
  default     = false
}

variable "allow_azure_services" {
  description = "Allow Azure services to access the database"
  type        = bool
  default     = true
}

variable "public_network_access_enabled" {
  description = "Enable public network access (dev only)"
  type        = bool
  default     = false
}

variable "tags" {
  description = "Additional tags for resources"
  type        = map(string)
  default     = {}
}
