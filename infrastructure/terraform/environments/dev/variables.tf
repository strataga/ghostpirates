# infrastructure/terraform/environments/dev/variables.tf

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "eastus"
}

variable "postgres_admin_password" {
  description = "PostgreSQL administrator password"
  type        = string
  sensitive   = true
}
