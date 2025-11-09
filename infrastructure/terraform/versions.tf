terraform {
  required_version = ">= 1.5.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.80"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.5"
    }
  }

  # Local state for development (no remote backend required)
  # For production, consider Terraform Cloud or Azure Storage backend
}

provider "azurerm" {
  features {}

  # Only needed when using Azure resources
  # For local development, this provider won't be invoked
  skip_provider_registration = true
}
