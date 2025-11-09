# Infrastructure Setup: Azure Cloud Deployment

**Duration**: Week 1 (7 days)
**Goal**: Production-ready Azure infrastructure → Terraform managed → CI/CD operational
**Dependencies**: Azure subscription with appropriate permissions

---

## Epic 1: Azure Foundation Setup

### Task 1.1: Azure Subscription and Resource Group Configuration

**Type**: Infrastructure
**Dependencies**: Azure subscription created

**Subtasks**:

- [ ] 1.1.1: Create Azure account and subscription

```bash
# Login to Azure CLI
az login

# Set default subscription
az account set --subscription "Ghost Pirates Production"

# Verify subscription
az account show --output table
```

- [ ] 1.1.2: Create resource groups

```bash
# Production resource group
az group create \
  --name ghostpirates-prod-rg \
  --location eastus \
  --tags Environment=Production Project=GhostPirates

# Staging resource group
az group create \
  --name ghostpirates-staging-rg \
  --location eastus \
  --tags Environment=Staging Project=GhostPirates

# Development resource group
az group create \
  --name ghostpirates-dev-rg \
  --location eastus \
  --tags Environment=Development Project=GhostPirates
```

- [ ] 1.1.3: Create service principal for Terraform

```bash
# Create service principal with Contributor role
az ad sp create-for-rbac \
  --name "ghostpirates-terraform-sp" \
  --role Contributor \
  --scopes /subscriptions/<SUBSCRIPTION_ID>

# Save output - you'll need:
# - appId (client_id)
# - password (client_secret)
# - tenant

# Create GitHub secret for CI/CD
gh secret set AZURE_CREDENTIALS --body '{
  "clientId": "<appId>",
  "clientSecret": "<password>",
  "subscriptionId": "<SUBSCRIPTION_ID>",
  "tenantId": "<tenant>"
}'
```

- [ ] 1.1.4: Configure Azure Container Registry

```bash
# Create ACR for container images
az acr create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpiratesacr \
  --sku Standard \
  --admin-enabled true

# Enable geo-replication (production)
az acr replication create \
  --resource-group ghostpirates-prod-rg \
  --registry ghostpiratesacr \
  --location westus

# Get credentials for Docker login
az acr credential show --name ghostpiratesacr
```

- [ ] 1.1.5: Create storage account for Terraform state

```bash
# Storage account for remote state
az storage account create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpiratesterraform \
  --sku Standard_LRS \
  --encryption-services blob

# Create container for state files
az storage container create \
  --name tfstate \
  --account-name ghostpiratesterraform
```

**Acceptance Criteria**:

- [ ] All resource groups created successfully
- [ ] Service principal has appropriate permissions
- [ ] Container Registry accessible
- [ ] Storage account configured for Terraform state
- [ ] All credentials stored securely in GitHub Secrets

---

### Task 1.2: Virtual Network and Security Configuration

**Type**: Infrastructure
**Dependencies**: Task 1.1 complete

**Subtasks**:

- [ ] 1.2.1: Create virtual network with subnets

```bash
# Create VNet
az network vnet create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-vnet \
  --address-prefix 10.0.0.0/16 \
  --location eastus

# AKS subnet
az network vnet subnet create \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name aks-subnet \
  --address-prefix 10.0.1.0/24

# Database subnet
az network vnet subnet create \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name database-subnet \
  --address-prefix 10.0.2.0/24 \
  --service-endpoints Microsoft.Sql

# Redis subnet
az network vnet subnet create \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name redis-subnet \
  --address-prefix 10.0.3.0/24

# Application Gateway subnet (for ingress)
az network vnet subnet create \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name appgw-subnet \
  --address-prefix 10.0.4.0/24
```

- [ ] 1.2.2: Configure network security groups

```bash
# AKS NSG
az network nsg create \
  --resource-group ghostpirates-prod-rg \
  --name aks-nsg

# Allow HTTPS inbound
az network nsg rule create \
  --resource-group ghostpirates-prod-rg \
  --nsg-name aks-nsg \
  --name allow-https \
  --priority 100 \
  --direction Inbound \
  --access Allow \
  --protocol Tcp \
  --destination-port-ranges 443

# Database NSG (private only)
az network nsg create \
  --resource-group ghostpirates-prod-rg \
  --name database-nsg

# Allow PostgreSQL from AKS subnet only
az network nsg rule create \
  --resource-group ghostpirates-prod-rg \
  --nsg-name database-nsg \
  --name allow-postgres-from-aks \
  --priority 100 \
  --direction Inbound \
  --access Allow \
  --protocol Tcp \
  --source-address-prefixes 10.0.1.0/24 \
  --destination-port-ranges 5432

# Associate NSGs with subnets
az network vnet subnet update \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name aks-subnet \
  --network-security-group aks-nsg

az network vnet subnet update \
  --resource-group ghostpirates-prod-rg \
  --vnet-name ghostpirates-vnet \
  --name database-subnet \
  --network-security-group database-nsg
```

- [ ] 1.2.3: Create Azure Key Vault

```bash
# Key Vault for secrets
az keyvault create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-kv \
  --location eastus \
  --enable-rbac-authorization true \
  --network-acls-default-action Deny

# Grant service principal access
az role assignment create \
  --role "Key Vault Secrets Officer" \
  --assignee <SERVICE_PRINCIPAL_APP_ID> \
  --scope /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.KeyVault/vaults/ghostpirates-kv

# Store initial secrets
az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "claude-api-key" \
  --value "<YOUR_CLAUDE_API_KEY>"

az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "openai-api-key" \
  --value "<YOUR_OPENAI_API_KEY>"

az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "jwt-secret" \
  --value "$(openssl rand -base64 32)"
```

- [ ] 1.2.4: Configure private endpoints for Key Vault

```bash
# Disable public access
az keyvault update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-kv \
  --default-action Deny

# Create private endpoint
az network private-endpoint create \
  --resource-group ghostpirates-prod-rg \
  --name kv-private-endpoint \
  --vnet-name ghostpirates-vnet \
  --subnet aks-subnet \
  --private-connection-resource-id /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.KeyVault/vaults/ghostpirates-kv \
  --group-id vault \
  --connection-name kv-connection
```

**Acceptance Criteria**:

- [ ] Virtual network created with all required subnets
- [ ] Network security groups properly configured
- [ ] Key Vault accessible via private endpoint only
- [ ] All secrets stored securely
- [ ] No public internet access to internal resources

---

## Epic 2: Database Infrastructure

### Task 2.1: Azure Database for PostgreSQL Setup

**Type**: Infrastructure
**Dependencies**: Task 1.2 complete

**Subtasks**:

- [ ] 2.1.1: Create PostgreSQL Flexible Server

```bash
# Create PostgreSQL server
az postgres flexible-server create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-db \
  --location eastus \
  --admin-user ghostadmin \
  --admin-password "<STRONG_PASSWORD>" \
  --sku-name Standard_D4s_v3 \
  --tier GeneralPurpose \
  --storage-size 128 \
  --version 15 \
  --high-availability Enabled \
  --zone 1 \
  --standby-zone 2 \
  --vnet ghostpirates-vnet \
  --subnet database-subnet \
  --private-dns-zone /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.Network/privateDnsZones/privatelink.postgres.database.azure.com

# Store connection string in Key Vault
DB_CONNECTION_STRING="postgresql://ghostadmin:<PASSWORD>@ghostpirates-db.postgres.database.azure.com:5432/ghostpirates?sslmode=require"

az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "database-url" \
  --value "$DB_CONNECTION_STRING"
```

- [ ] 2.1.2: Configure PostgreSQL parameters

```bash
# Optimize for agent workloads
az postgres flexible-server parameter set \
  --resource-group ghostpirates-prod-rg \
  --server-name ghostpirates-db \
  --name max_connections \
  --value 200

az postgres flexible-server parameter set \
  --resource-group ghostpirates-prod-rg \
  --server-name ghostpirates-db \
  --name shared_buffers \
  --value "1GB"

az postgres flexible-server parameter set \
  --resource-group ghostpirates-prod-rg \
  --server-name ghostpirates-db \
  --name work_mem \
  --value "16MB"

az postgres flexible-server parameter set \
  --resource-group ghostpirates-prod-rg \
  --server-name ghostpirates-db \
  --name maintenance_work_mem \
  --value "256MB"
```

- [ ] 2.1.3: Install pgvector extension

```bash
# Connect to database
psql "$DB_CONNECTION_STRING"

# Install extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgvector";

# Verify installation
\dx

# Exit
\q
```

- [ ] 2.1.4: Configure automated backups

```bash
# Set backup retention to 7 days
az postgres flexible-server update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-db \
  --backup-retention 7

# Enable geo-redundant backup
az postgres flexible-server update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-db \
  --geo-redundant-backup Enabled
```

- [ ] 2.1.5: Create read replica for scaling

```bash
# Create read replica in West US
az postgres flexible-server replica create \
  --resource-group ghostpirates-prod-rg \
  --replica-name ghostpirates-db-replica \
  --source-server ghostpirates-db \
  --location westus
```

**Acceptance Criteria**:

- [ ] PostgreSQL Flexible Server running
- [ ] High availability enabled with zone redundancy
- [ ] pgvector extension installed
- [ ] Automated backups configured
- [ ] Read replica created for scaling
- [ ] Connection string stored in Key Vault

---

### Task 2.2: Azure Cache for Redis Setup

**Type**: Infrastructure
**Dependencies**: Task 1.2 complete

**Subtasks**:

- [ ] 2.2.1: Create Azure Cache for Redis

```bash
# Create Redis cache
az redis create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis \
  --location eastus \
  --sku Standard \
  --vm-size C1 \
  --enable-non-ssl-port false \
  --minimum-tls-version 1.2 \
  --redis-version 6 \
  --zones 1 2

# Get connection string
az redis list-keys \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis
```

- [ ] 2.2.2: Configure Redis parameters

```bash
# Set max memory policy
az redis update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis \
  --set redisConfiguration.maxmemory-policy=allkeys-lru

# Enable persistence
az redis update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis \
  --set enableNonSslPort=false \
  --set redisConfiguration.rdb-backup-enabled=true \
  --set redisConfiguration.rdb-backup-frequency=60
```

- [ ] 2.2.3: Store Redis connection string

```bash
# Get primary connection string
REDIS_CONNECTION=$(az redis show \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis \
  --query "hostName" -o tsv)

REDIS_KEY=$(az redis list-keys \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-redis \
  --query "primaryKey" -o tsv)

REDIS_URL="rediss://:$REDIS_KEY@$REDIS_CONNECTION:6380"

# Store in Key Vault
az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "redis-url" \
  --value "$REDIS_URL"
```

- [ ] 2.2.4: Configure private endpoint for Redis

```bash
# Create private endpoint
az network private-endpoint create \
  --resource-group ghostpirates-prod-rg \
  --name redis-private-endpoint \
  --vnet-name ghostpirates-vnet \
  --subnet redis-subnet \
  --private-connection-resource-id /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.Cache/Redis/ghostpirates-redis \
  --group-id redisCache \
  --connection-name redis-connection
```

**Acceptance Criteria**:

- [ ] Redis cache created with Standard tier
- [ ] TLS encryption enabled
- [ ] Persistence configured
- [ ] Private endpoint configured
- [ ] Connection string stored in Key Vault

---

## Epic 3: Kubernetes Cluster Setup

### Task 3.1: Azure Kubernetes Service (AKS) Creation

**Type**: Infrastructure
**Dependencies**: Tasks 1.2 and 2.1 complete

**Subtasks**:

- [ ] 3.1.1: Create AKS cluster

```bash
# Create AKS cluster with managed identity
az aks create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-aks \
  --location eastus \
  --node-count 3 \
  --node-vm-size Standard_D4s_v3 \
  --network-plugin azure \
  --vnet-subnet-id /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.Network/virtualNetworks/ghostpirates-vnet/subnets/aks-subnet \
  --enable-managed-identity \
  --enable-addons monitoring \
  --workspace-resource-id /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.OperationalInsights/workspaces/ghostpirates-logs \
  --enable-cluster-autoscaler \
  --min-count 3 \
  --max-count 10 \
  --kubernetes-version 1.28

# Get credentials
az aks get-credentials \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-aks \
  --overwrite-existing
```

- [ ] 3.1.2: Configure node pools

```bash
# Create system node pool (for system pods)
az aks nodepool add \
  --resource-group ghostpirates-prod-rg \
  --cluster-name ghostpirates-aks \
  --name systempool \
  --node-count 2 \
  --node-vm-size Standard_D2s_v3 \
  --mode System \
  --zones 1 2 3

# Create user node pool (for application pods)
az aks nodepool add \
  --resource-group ghostpirates-prod-rg \
  --cluster-name ghostpirates-aks \
  --name agentpool \
  --node-count 3 \
  --node-vm-size Standard_D4s_v3 \
  --mode User \
  --enable-cluster-autoscaler \
  --min-count 3 \
  --max-count 10 \
  --zones 1 2 3
```

- [ ] 3.1.3: Install essential cluster add-ons

```bash
# Install NGINX Ingress Controller
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.2/deploy/static/provider/cloud/deploy.yaml

# Install cert-manager for TLS certificates
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.2/cert-manager.yaml

# Install Azure CSI Secret Store Driver
helm repo add csi-secrets-store-provider-azure https://azure.github.io/secrets-store-csi-driver-provider-azure/charts
helm install csi-secrets-store-provider-azure/csi-secrets-store-provider-azure \
  --namespace kube-system \
  --generate-name
```

- [ ] 3.1.4: Configure workload identity

```bash
# Enable workload identity on cluster
az aks update \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-aks \
  --enable-workload-identity \
  --enable-oidc-issuer

# Create managed identity for application
az identity create \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-app-identity

# Get identity details
CLIENT_ID=$(az identity show \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-app-identity \
  --query clientId -o tsv)

PRINCIPAL_ID=$(az identity show \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-app-identity \
  --query principalId -o tsv)

# Grant Key Vault access
az role assignment create \
  --role "Key Vault Secrets User" \
  --assignee $PRINCIPAL_ID \
  --scope /subscriptions/<SUBSCRIPTION_ID>/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.KeyVault/vaults/ghostpirates-kv
```

- [ ] 3.1.5: Configure SecretProviderClass

```yaml
# k8s/secret-provider-class.yaml
apiVersion: secrets-store.csi.x-k8s.io/v1
kind: SecretProviderClass
metadata:
  name: ghostpirates-secrets
  namespace: default
spec:
  provider: azure
  parameters:
    usePodIdentity: "false"
    useVMManagedIdentity: "true"
    userAssignedIdentityID: "<CLIENT_ID>"
    keyvaultName: ghostpirates-kv
    cloudName: AzurePublicCloud
    objects: |
      array:
        - |
          objectName: database-url
          objectType: secret
        - |
          objectName: redis-url
          objectType: secret
        - |
          objectName: claude-api-key
          objectType: secret
        - |
          objectName: openai-api-key
          objectType: secret
        - |
          objectName: jwt-secret
          objectType: secret
    tenantId: "<TENANT_ID>"
```

```bash
kubectl apply -f k8s/secret-provider-class.yaml
```

**Acceptance Criteria**:

- [ ] AKS cluster running with 3+ nodes
- [ ] Cluster autoscaler configured
- [ ] NGINX ingress controller installed
- [ ] Cert-manager installed for TLS
- [ ] Workload identity configured
- [ ] Secret provider accessing Key Vault

---

### Task 3.2: Configure Monitoring and Logging

**Type**: Infrastructure
**Dependencies**: Task 3.1 complete

**Subtasks**:

- [ ] 3.2.1: Create Log Analytics Workspace

```bash
# Create workspace
az monitor log-analytics workspace create \
  --resource-group ghostpirates-prod-rg \
  --workspace-name ghostpirates-logs \
  --location eastus

# Get workspace ID
WORKSPACE_ID=$(az monitor log-analytics workspace show \
  --resource-group ghostpirates-prod-rg \
  --workspace-name ghostpirates-logs \
  --query id -o tsv)
```

- [ ] 3.2.2: Enable Container Insights

```bash
# Enable monitoring
az aks enable-addons \
  --resource-group ghostpirates-prod-rg \
  --name ghostpirates-aks \
  --addons monitoring \
  --workspace-resource-id $WORKSPACE_ID
```

- [ ] 3.2.3: Install Prometheus and Grafana

```bash
# Add Helm repos
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo add grafana https://grafana.github.io/helm-charts
helm repo update

# Install Prometheus
helm install prometheus prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set prometheus.prometheusSpec.retention=30d \
  --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=50Gi

# Install Grafana
helm install grafana grafana/grafana \
  --namespace monitoring \
  --set persistence.enabled=true \
  --set persistence.size=10Gi \
  --set adminPassword=<STRONG_PASSWORD>
```

- [ ] 3.2.4: Configure Application Insights

```bash
# Create Application Insights
az monitor app-insights component create \
  --resource-group ghostpirates-prod-rg \
  --app ghostpirates-insights \
  --location eastus \
  --workspace $WORKSPACE_ID

# Get instrumentation key
INSTRUMENTATION_KEY=$(az monitor app-insights component show \
  --resource-group ghostpirates-prod-rg \
  --app ghostpirates-insights \
  --query instrumentationKey -o tsv)

# Store in Key Vault
az keyvault secret set \
  --vault-name ghostpirates-kv \
  --name "appinsights-key" \
  --value "$INSTRUMENTATION_KEY"
```

**Acceptance Criteria**:

- [ ] Log Analytics Workspace collecting cluster logs
- [ ] Container Insights showing pod metrics
- [ ] Prometheus scraping metrics
- [ ] Grafana dashboards accessible
- [ ] Application Insights configured

---

## Epic 4: Terraform Infrastructure as Code

### Task 4.1: Terraform Configuration

**Type**: Infrastructure
**Dependencies**: All manual setup tasks complete (for reference)

**Subtasks**:

- [ ] 4.1.1: Create Terraform project structure

```bash
mkdir -p terraform/{modules,environments}
cd terraform

# Module structure
mkdir -p modules/{networking,database,redis,aks,monitoring}

# Environment structure
mkdir -p environments/{dev,staging,prod}

# Create files
touch main.tf variables.tf outputs.tf versions.tf
```

- [ ] 4.1.2: Configure Terraform backend

```hcl
# versions.tf
terraform {
  required_version = ">= 1.6.0"

  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = "~> 3.80"
    }
    azuread = {
      source  = "hashicorp/azuread"
      version = "~> 2.45"
    }
  }

  backend "azurerm" {
    resource_group_name  = "ghostpirates-prod-rg"
    storage_account_name = "ghostpiratesterraform"
    container_name       = "tfstate"
    key                  = "prod.terraform.tfstate"
  }
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy = false
    }
  }
}
```

- [ ] 4.1.3: Create networking module

```hcl
# modules/networking/main.tf
resource "azurerm_virtual_network" "main" {
  name                = "${var.prefix}-vnet"
  resource_group_name = var.resource_group_name
  location            = var.location
  address_space       = ["10.0.0.0/16"]

  tags = var.tags
}

resource "azurerm_subnet" "aks" {
  name                 = "aks-subnet"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.1.0/24"]
}

resource "azurerm_subnet" "database" {
  name                 = "database-subnet"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.2.0/24"]

  service_endpoints = ["Microsoft.Sql"]

  delegation {
    name = "postgres-delegation"

    service_delegation {
      name = "Microsoft.DBforPostgreSQL/flexibleServers"
      actions = [
        "Microsoft.Network/virtualNetworks/subnets/join/action",
      ]
    }
  }
}

resource "azurerm_subnet" "redis" {
  name                 = "redis-subnet"
  resource_group_name  = var.resource_group_name
  virtual_network_name = azurerm_virtual_network.main.name
  address_prefixes     = ["10.0.3.0/24"]
}

resource "azurerm_network_security_group" "aks" {
  name                = "${var.prefix}-aks-nsg"
  location            = var.location
  resource_group_name = var.resource_group_name

  security_rule {
    name                       = "allow-https"
    priority                   = 100
    direction                  = "Inbound"
    access                     = "Allow"
    protocol                   = "Tcp"
    source_port_range          = "*"
    destination_port_range     = "443"
    source_address_prefix      = "*"
    destination_address_prefix = "*"
  }

  tags = var.tags
}

resource "azurerm_subnet_network_security_group_association" "aks" {
  subnet_id                 = azurerm_subnet.aks.id
  network_security_group_id = azurerm_network_security_group.aks.id
}
```

- [ ] 4.1.4: Create database module

```hcl
# modules/database/main.tf
resource "azurerm_postgresql_flexible_server" "main" {
  name                = "${var.prefix}-db"
  resource_group_name = var.resource_group_name
  location            = var.location

  administrator_login    = var.admin_username
  administrator_password = var.admin_password

  sku_name   = "GP_Standard_D4s_v3"
  storage_mb = 131072
  version    = "15"

  zone                      = "1"
  high_availability {
    mode                      = "ZoneRedundant"
    standby_availability_zone = "2"
  }

  delegated_subnet_id = var.subnet_id
  private_dns_zone_id = azurerm_private_dns_zone.postgres.id

  backup_retention_days        = 7
  geo_redundant_backup_enabled = true

  tags = var.tags
}

resource "azurerm_private_dns_zone" "postgres" {
  name                = "privatelink.postgres.database.azure.com"
  resource_group_name = var.resource_group_name
}

resource "azurerm_private_dns_zone_virtual_network_link" "postgres" {
  name                  = "postgres-dns-link"
  resource_group_name   = var.resource_group_name
  private_dns_zone_name = azurerm_private_dns_zone.postgres.name
  virtual_network_id    = var.vnet_id
}

resource "azurerm_postgresql_flexible_server_configuration" "extensions" {
  name      = "azure.extensions"
  server_id = azurerm_postgresql_flexible_server.main.id
  value     = "UUID-OSSP,VECTOR"
}
```

- [ ] 4.1.5: Create AKS module

```hcl
# modules/aks/main.tf
resource "azurerm_kubernetes_cluster" "main" {
  name                = "${var.prefix}-aks"
  location            = var.location
  resource_group_name = var.resource_group_name
  dns_prefix          = "${var.prefix}-aks"

  kubernetes_version        = "1.28"
  automatic_channel_upgrade = "stable"

  default_node_pool {
    name                = "system"
    vm_size             = "Standard_D2s_v3"
    enable_auto_scaling = true
    min_count           = 2
    max_count           = 5
    vnet_subnet_id      = var.subnet_id
    zones               = ["1", "2", "3"]

    upgrade_settings {
      max_surge = "33%"
    }
  }

  identity {
    type = "SystemAssigned"
  }

  network_profile {
    network_plugin = "azure"
    network_policy = "azure"
    load_balancer_sku = "standard"
  }

  oms_agent {
    log_analytics_workspace_id = var.log_analytics_workspace_id
  }

  key_vault_secrets_provider {
    secret_rotation_enabled  = true
    secret_rotation_interval = "2m"
  }

  workload_identity_enabled = true
  oidc_issuer_enabled       = true

  tags = var.tags
}

resource "azurerm_kubernetes_cluster_node_pool" "agents" {
  name                  = "agents"
  kubernetes_cluster_id = azurerm_kubernetes_cluster.main.id
  vm_size               = "Standard_D4s_v3"
  enable_auto_scaling   = true
  min_count             = 3
  max_count             = 10
  vnet_subnet_id        = var.subnet_id
  zones                 = ["1", "2", "3"]

  upgrade_settings {
    max_surge = "33%"
  }

  tags = var.tags
}
```

- [ ] 4.1.6: Create main Terraform configuration

```hcl
# environments/prod/main.tf
locals {
  prefix   = "ghostpirates"
  location = "eastus"
  environment = "prod"

  tags = {
    Environment = "Production"
    Project     = "GhostPirates"
    ManagedBy   = "Terraform"
  }
}

resource "azurerm_resource_group" "main" {
  name     = "${local.prefix}-${local.environment}-rg"
  location = local.location
  tags     = local.tags
}

module "networking" {
  source = "../../modules/networking"

  prefix              = local.prefix
  location            = local.location
  resource_group_name = azurerm_resource_group.main.name
  tags                = local.tags
}

module "database" {
  source = "../../modules/database"

  prefix              = local.prefix
  location            = local.location
  resource_group_name = azurerm_resource_group.main.name
  subnet_id           = module.networking.database_subnet_id
  vnet_id             = module.networking.vnet_id
  admin_username      = var.db_admin_username
  admin_password      = var.db_admin_password
  tags                = local.tags
}

module "redis" {
  source = "../../modules/redis"

  prefix              = local.prefix
  location            = local.location
  resource_group_name = azurerm_resource_group.main.name
  subnet_id           = module.networking.redis_subnet_id
  tags                = local.tags
}

module "aks" {
  source = "../../modules/aks"

  prefix                     = local.prefix
  location                   = local.location
  resource_group_name        = azurerm_resource_group.main.name
  subnet_id                  = module.networking.aks_subnet_id
  log_analytics_workspace_id = module.monitoring.workspace_id
  tags                       = local.tags
}
```

- [ ] 4.1.7: Test Terraform configuration

```bash
cd terraform/environments/prod

# Initialize Terraform
terraform init

# Validate configuration
terraform validate

# Plan deployment
terraform plan -out=tfplan

# Apply (dry run first)
# terraform apply tfplan
```

**Acceptance Criteria**:

- [ ] Terraform modules created for all components
- [ ] Backend configured for remote state
- [ ] All resources defined in code
- [ ] Terraform validate passes
- [ ] Terraform plan generates without errors
- [ ] Code checked into version control

---

## Epic 5: CI/CD Pipeline Setup

### Task 5.1: GitHub Actions Workflows

**Type**: DevOps
**Dependencies**: All infrastructure tasks complete

**Subtasks**:

- [ ] 5.1.1: Create Docker build workflow

```yaml
# .github/workflows/build-api.yml
name: Build and Push API Image

on:
  push:
    branches: [main, develop]
    paths:
      - 'apps/api/**'
      - '.github/workflows/build-api.yml'
  pull_request:
    branches: [main]

env:
  REGISTRY: ghostpiratesacr.azurecr.io
  IMAGE_NAME: ghostpirates-api

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      packages: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Azure Container Registry
        uses: azure/docker-login@v1
        with:
          login-server: ${{ env.REGISTRY }}
          username: ${{ secrets.ACR_USERNAME }}
          password: ${{ secrets.ACR_PASSWORD }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=sha,prefix={{branch}}-

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: apps/api
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

- [ ] 5.1.2: Create Rust testing workflow

```yaml
# .github/workflows/test-api.yml
name: Test Rust API

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: ghostpirates_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432

      redis:
        image: redis:7-alpine
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --all
        env:
          DATABASE_URL: postgresql://postgres:postgres@localhost:5432/ghostpirates_test
          REDIS_URL: redis://localhost:6379

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Check formatting
        run: cargo fmt --check
```

- [ ] 5.1.3: Create deployment workflow

```yaml
# .github/workflows/deploy-prod.yml
name: Deploy to Production

on:
  push:
    branches: [main]

env:
  REGISTRY: ghostpiratesacr.azurecr.io
  AKS_CLUSTER: ghostpirates-aks
  AKS_RESOURCE_GROUP: ghostpirates-prod-rg

jobs:
  deploy:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Azure Login
        uses: azure/login@v1
        with:
          creds: ${{ secrets.AZURE_CREDENTIALS }}

      - name: Get AKS credentials
        run: |
          az aks get-credentials \
            --resource-group ${{ env.AKS_RESOURCE_GROUP }} \
            --name ${{ env.AKS_CLUSTER }} \
            --overwrite-existing

      - name: Set image tag
        id: image
        run: |
          echo "tag=${{ env.REGISTRY }}/ghostpirates-api:main-${{ github.sha }}" >> $GITHUB_OUTPUT

      - name: Deploy to AKS
        run: |
          kubectl set image deployment/ghostpirates-api \
            api=${{ steps.image.outputs.tag }} \
            --namespace default \
            --record

      - name: Verify deployment
        run: |
          kubectl rollout status deployment/ghostpirates-api \
            --namespace default \
            --timeout=5m

      - name: Run smoke tests
        run: |
          kubectl run smoke-test \
            --image=curlimages/curl:latest \
            --rm \
            --restart=Never \
            --timeout=30s \
            -- curl -f http://ghostpirates-api:4000/health
```

- [ ] 5.1.4: Create Terraform workflow

```yaml
# .github/workflows/terraform.yml
name: Terraform

on:
  push:
    branches: [main]
    paths:
      - 'terraform/**'
  pull_request:
    branches: [main]
    paths:
      - 'terraform/**'

jobs:
  terraform:
    runs-on: ubuntu-latest

    defaults:
      run:
        working-directory: terraform/environments/prod

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Terraform
        uses: hashicorp/setup-terraform@v3
        with:
          terraform_version: 1.6.0

      - name: Azure Login
        uses: azure/login@v1
        with:
          creds: ${{ secrets.AZURE_CREDENTIALS }}

      - name: Terraform Init
        run: terraform init

      - name: Terraform Format Check
        run: terraform fmt -check -recursive

      - name: Terraform Validate
        run: terraform validate

      - name: Terraform Plan
        if: github.event_name == 'pull_request'
        run: terraform plan -out=tfplan
        env:
          TF_VAR_db_admin_password: ${{ secrets.DB_ADMIN_PASSWORD }}

      - name: Terraform Apply
        if: github.ref == 'refs/heads/main' && github.event_name == 'push'
        run: terraform apply -auto-approve
        env:
          TF_VAR_db_admin_password: ${{ secrets.DB_ADMIN_PASSWORD }}
```

**Acceptance Criteria**:

- [ ] Docker build workflow operational
- [ ] Rust testing workflow passing
- [ ] Deployment workflow deploying to AKS
- [ ] Terraform workflow managing infrastructure
- [ ] All workflows tested with sample commits
- [ ] GitHub secrets configured

---

## Success Criteria - Infrastructure Complete

- [ ] All Azure resources provisioned
- [ ] Virtual network with proper subnets configured
- [ ] PostgreSQL database accessible from AKS
- [ ] Redis cache operational
- [ ] AKS cluster running with autoscaling
- [ ] Monitoring and logging configured
- [ ] Terraform managing all infrastructure
- [ ] CI/CD pipelines operational
- [ ] All secrets in Key Vault
- [ ] Private endpoints configured
- [ ] No public internet exposure for databases

---

## Next Steps

Proceed to [03-database-architecture.md](./03-database-architecture.md) for complete database schema implementation.

---

**Infrastructure: Production-Ready Azure Environment 🏗️**
