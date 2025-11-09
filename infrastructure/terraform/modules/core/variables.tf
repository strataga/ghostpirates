# infrastructure/terraform/modules/core/variables.tf

variable "environment" {
  description = "Environment name (dev, staging, production)"
  type        = string
}

variable "location" {
  description = "Azure region for resources"
  type        = string
  default     = "eastus"
}

variable "storage_replication_type" {
  description = "Storage replication type"
  type        = string
  default     = "LRS" # Locally redundant storage (cheapest)
}

variable "create_redis" {
  description = "Whether to create Redis cache (set to false to use local Docker)"
  type        = bool
  default     = false # Use local Docker Redis by default
}
