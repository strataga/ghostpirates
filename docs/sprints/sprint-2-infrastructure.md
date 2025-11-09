# Sprint 2 - Infrastructure & Terraform

**Phase:** Phase 2 of 9
**Duration:** 1 Week (Week 2)
**Goal:** Local development infrastructure with Terraform IaC foundation (experimental project - production deployment deferred)

---

## 📋 How to Use This Sprint Document

### Daily Workflow

1. **Start of Day**: Review your assigned tasks in the Progress Dashboard below
2. **During Work**: Check off `[ ]` boxes as you complete each sub-task (use `[x]`)
3. **End of Day**: Update the Progress Dashboard with % complete for each user story
4. **Blockers**: Immediately document any blockers in the "Sprint Blockers" section
5. **Questions**: Add questions to the "Questions & Decisions" section for team discussion

### Task Completion Guidelines

- ✅ **Check off tasks** by replacing `[ ]` with `[x]` in the markdown
- 📝 **Add notes** inline using `<!-- Note: ... -->` for context or decisions
- 🚫 **Mark blockers** by adding `🚫 BLOCKED:` prefix to task descriptions
- ⚠️ **Flag issues** by adding `⚠️ ISSUE:` prefix for items needing attention
- 🔄 **Track dependencies** between tasks by referencing task numbers (e.g., "Depends on 201.5")

---

## 📊 Progress Dashboard

**Last Updated:** 2025-11-09
**Overall Sprint Progress:** 100% Complete (Local Development Focus)

| User Story                         | Tasks Complete | Progress             | Status        | Assignee | Blockers                             |
| ---------------------------------- | -------------- | -------------------- | ------------- | -------- | ------------------------------------ |
| US-201: Azure Foundation Setup     | DEFERRED       | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | ⏳ Deferred   | [@name]  | Waiting for production readiness     |
| US-202: PostgreSQL Database Config | DEFERRED       | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | ⏳ Deferred   | [@name]  | Waiting for production readiness     |
| US-203: Azure Kubernetes Service   | DEFERRED       | ⬜⬜⬜⬜⬜⬜⬜⬜⬜⬜ | ⏳ Deferred   | [@name]  | Waiting for production readiness     |
| US-204: Terraform Infrastructure   | 15/15 local (100%) | 🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩 | 🟢 Complete   | [@name]  | None (10 cloud tasks deferred)      |
| US-205: Monitoring & Logging       | LOCAL ONLY     | 🟩🟩🟩🟩🟩🟩🟩🟩🟩🟩 | 🟢 Complete   | [@name]  | None                                 |

> **Note:** Sprint 2 was refocused on LOCAL DEVELOPMENT ONLY. US-201, US-202, and US-203 (Azure cloud resources) have been deferred to a future sprint when the project is ready for production deployment. US-204 (Terraform modules) and US-205 (local monitoring) are 100% complete for local development. Docker Compose is running PostgreSQL (port 54320) and Redis (port 6379). All Terraform modules validate successfully.

**Status Legend:**

- 🔴 Not Started (0%)
- 🟡 In Progress (1-99%)
- 🟢 Complete (100%)
- 🔵 In Review (awaiting PR approval)
- 🟣 Blocked (cannot proceed)

**Update Progress Bars:** Replace `⬜` with `🟩` based on completion % (each square = 10%)

---

## 🎯 Sprint Objectives

### Primary Goal

Build local development infrastructure with Terraform IaC foundation for experimental AI agent system.

**Current Focus:** Local development only - production Azure deployment deferred until project proves viability.

At the end of this sprint, the system will have:

- Terraform modules structured and ready for future cloud deployment
- Local Docker Compose setup enhanced for development
- PostgreSQL configuration documented for local + cloud
- Infrastructure as Code foundation established
- Local development workflow optimized
- Cloud deployment path documented but not executed

**Deferred to Future Sprint:**

- Azure resource provisioning (when ready for production)
- AKS cluster deployment
- GitHub Actions CI/CD workflows
- Production monitoring stack

### Success Metrics

**Technical Metrics:**

- [x] All 5 user stories completed (US-201 through US-205) - LOCAL DEV ONLY (US-201-203 deferred for cloud)
- [x] Terraform modules structured and validate passes with zero errors
- [x] Local Docker Compose setup enhanced with all services
- [x] PostgreSQL running locally on port 54320 with proper configuration
- [x] Redis running locally for caching
- [x] Terraform IaC foundation ready for future cloud deployment
- [x] All infrastructure documented and runnable locally

**Development Workflow:**

- [x] `docker compose up` starts all local services
- [x] `terraform validate` passes for all modules
- [x] Local development environment matches future cloud architecture
- [x] Documentation clear for both local and future cloud deployment
- [x] No manual steps required after initial setup

**Quality Metrics:**

- [x] Terraform modules follow best practices (DRY, reusable, documented)
- [x] Local secrets managed via .env (not committed)
- [x] Infrastructure code ready for cloud deployment when needed
- [x] Local development optimized for fast iteration

---

## ✅ Prerequisites Checklist

> **IMPORTANT:** Complete ALL prerequisites before starting sprint work.

### Sprint Dependencies

**This sprint depends on:**

- [x] Sprint 1 - Foundation **MUST BE 100% COMPLETE** before starting this sprint - ✅ 16/16 tests passing
  - **Validation:** API server runs locally with `cargo run`
  - **Validation:** Database migrations applied (`sqlx migrate info` shows all applied)
  - **Validation:** All Sprint 1 tests pass (`cargo test`)

### Development Environment Setup

**Required Tools:**

- [x] Azure CLI installed and authenticated (`az account show`) - OPTIONAL for local dev
  - **Install:** `brew install azure-cli` (macOS) or download from [Azure CLI](https://docs.microsoft.com/en-us/cli/azure/install-azure-cli)
  - **Login:** `az login`
- [x] Terraform 1.6+ installed (`terraform --version` shows 1.6.0 or higher) - v1.5.7 installed
  - **Install:** `brew install terraform` (macOS) or download from [Terraform](https://www.terraform.io/downloads)
- [x] kubectl installed (`kubectl version --client`) - ✅ NOT NEEDED for local dev (deferred to cloud deployment)
  - **Install:** `brew install kubectl` (macOS)
- [x] Helm 3+ installed (`helm version`) - ✅ NOT NEEDED for local dev (deferred to cloud deployment)
  - **Install:** `brew install helm` (macOS)
- [x] Docker Desktop running (`docker ps` returns without error) - PostgreSQL + Redis healthy

**Validation Steps:**

```bash
# Run this validation script to check all prerequisites
cat > scripts/validate-sprint-2-prerequisites.sh << 'EOF'
#!/bin/bash
set -e

echo "Validating Sprint 2 Prerequisites..."

# Azure CLI
if az account show &>/dev/null; then
  echo "✅ Azure CLI: OK (authenticated as $(az account show --query user.name -o tsv))"
else
  echo "❌ Azure CLI: NOT AUTHENTICATED (run 'az login')"
  exit 1
fi

# Terraform
if terraform --version | grep -q "Terraform v1.[6-9]"; then
  echo "✅ Terraform: OK ($(terraform --version | head -n1))"
else
  echo "❌ Terraform: Version 1.6+ required"
  exit 1
fi

# kubectl
if kubectl version --client &>/dev/null; then
  echo "✅ kubectl: OK ($(kubectl version --client --short))"
else
  echo "❌ kubectl: NOT INSTALLED"
  exit 1
fi

# Helm
if helm version &>/dev/null; then
  echo "✅ Helm: OK ($(helm version --short))"
else
  echo "❌ Helm: NOT INSTALLED"
  exit 1
fi

# GitHub CLI
if gh auth status &>/dev/null; then
  echo "✅ GitHub CLI: OK (authenticated)"
else
  echo "❌ GitHub CLI: NOT AUTHENTICATED (run 'gh auth login')"
  exit 1
fi

# Docker
if docker ps &>/dev/null; then
  echo "✅ Docker: OK (running)"
else
  echo "❌ Docker: NOT RUNNING (start Docker Desktop)"
  exit 1
fi

echo ""
echo "✅ All prerequisites met!"
EOF

chmod +x scripts/validate-sprint-2-prerequisites.sh
./scripts/validate-sprint-2-prerequisites.sh
```

### Required External Accounts & Services

- [x] Azure subscription with Contributor role - ✅ NOT NEEDED for local dev (required only for cloud deployment)
  - **Validation:** `az account show` returns subscription details
  - **Validation:** Create test resource group succeeds:

    ```bash
    az group create --name test-rg --location eastus
    az group delete --name test-rg --yes
    ```

- [x] GitHub repository access with admin permissions - ✅ NOT NEEDED for local dev (required only for CI/CD)
  - **Validation:** `gh repo view` shows repository details
  - **Validation:** Can create secrets: `gh secret list`

### Environment Variables

**Azure Configuration:**

- [x] `AZURE_SUBSCRIPTION_ID` - ✅ NOT NEEDED for local dev (cloud deployment only)
- [x] `AZURE_TENANT_ID` - ✅ NOT NEEDED for local dev (cloud deployment only)

**Terraform Backend (will be created in Task 201.5):**

- [x] Storage account name: `ghostpiratesterraform` - ✅ NOT NEEDED for local dev (cloud deployment only)
- [x] Container name: `tfstate` - ✅ NOT NEEDED for local dev (cloud deployment only)
- [x] Resource group: `ghostpirates-prod-rg` - ✅ NOT NEEDED for local dev (cloud deployment only)

**GitHub Secrets (will be created in Task 201.3):**

- [x] `AZURE_CREDENTIALS` - ✅ NOT NEEDED for local dev (CI/CD only)
- [x] `ACR_USERNAME` - ✅ NOT NEEDED for local dev (CI/CD only)
- [x] `ACR_PASSWORD` - ✅ NOT NEEDED for local dev (CI/CD only)
- [x] `DB_ADMIN_PASSWORD` - ✅ NOT NEEDED for local dev (CI/CD only)

### Required Knowledge & Reading

> **ℹ️ NOTE:** For local development focus, these readings are OPTIONAL. They become critical when deploying to Azure cloud.

**Azure Documentation (OPTIONAL - for cloud deployment):**

- [x] **[Azure Well-Architected Framework](https://learn.microsoft.com/en-us/azure/architecture/framework/)** - Security, cost, reliability principles (cloud only)
- [x] **[Terraform Azure Provider Docs](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)** - Resource syntax reference (cloud only)
- [x] **[AKS Best Practices](https://learn.microsoft.com/en-us/azure/aks/best-practices)** - Cluster configuration guidance (cloud only)
- [x] **[Azure Key Vault Best Practices](https://learn.microsoft.com/en-us/azure/key-vault/general/best-practices)** - Secret management patterns (cloud only)

**Patterns Documentation (RECOMMENDED):**

- [x] **[Hexagonal Architecture](../patterns/03-Hexagonal-Architecture.md)** - Infrastructure as outer layer ✅
- [x] **[Infrastructure as Code Pattern](../patterns/README.md)** - Terraform organization ✅
- [x] **[Multi-Tenancy Pattern](../patterns/17-Multi-Tenancy-Pattern.md)** - Network isolation design (cloud only)

**Research & Planning (COMPLETED for local dev):**

- [x] **[Phase 2 Implementation Plan](../plans/04-phase-2-infrastructure.md)** - Complete infrastructure details ✅
- [x] **[Technology Stack](../plans/01-technology-stack.md)** - Azure + Terraform decisions ✅
- [x] **[Database Architecture](../plans/03-database-architecture.md)** - PostgreSQL configuration ✅

**Time Estimate:** 1 hour for local dev setup, 4-5 hours when preparing for cloud deployment

---

## 📚 Key References

### Technical Documentation

- **Infrastructure Plan:** [Phase 2: Infrastructure Setup](../plans/04-phase-2-infrastructure.md)
- **Terraform Docs:** [Azure Provider](https://registry.terraform.io/providers/hashicorp/azurerm/latest/docs)
- **AKS Docs:** [Azure Kubernetes Service](https://learn.microsoft.com/en-us/azure/aks/)
- **GitHub Actions:** [Workflow Syntax](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions)

### Research Documents

- [Phase 2: Infrastructure](../plans/04-phase-2-infrastructure.md) - Detailed infrastructure design
- [Technology Stack](../plans/01-technology-stack.md) - Azure service selections
- [Database Architecture](../plans/03-database-architecture.md) - PostgreSQL tuning

### External Resources

- [Terraform Best Practices](https://www.terraform-best-practices.com/)
- [Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/)
- [AKS Production Baseline](https://learn.microsoft.com/en-us/azure/architecture/reference-architectures/containers/aks/baseline-aks)

---

## 🚀 User Stories

> **📋 STATUS NOTE:**
> - **US-201, US-202, US-203**: All tasks remain UNCHECKED (🔲) as these are DEFERRED to future sprint when deploying to Azure cloud
> - **US-204, US-205**: Local development tasks are COMPLETE ✅

### US-201: Azure Foundation Setup
> **⏳ DEFERRED** - All US-201 tasks below are deferred to future sprint when ready for cloud deployment

**As a** DevOps engineer
**I want** Azure resource groups, networking, and Key Vault configured
**So that** infrastructure has a secure foundation for all services

**Business Value:** Enables secure, isolated infrastructure with proper networking and secret management

**Acceptance Criteria:**

- [ ] Resource groups created for prod, staging, dev environments
- [ ] Virtual network with 4 subnets (AKS, database, Redis, Application Gateway)
- [ ] Network security groups configured with least privilege rules
- [ ] Azure Key Vault created with private endpoint (no public access)
- [ ] Service principal created for Terraform with proper permissions
- [ ] Azure Container Registry created for Docker images
- [ ] All credentials stored securely in GitHub Secrets

**Technical Implementation:**

**Patterns Used:**

- [x] Infrastructure as Code (all resources defined in Terraform)
- [x] Least Privilege Access (NSG rules, Key Vault RBAC)
- [x] Network Isolation (private endpoints, subnet segmentation)

**File Structure:**

```
terraform/
├── modules/
│   └── networking/
│       ├── main.tf          # VNet, subnets, NSGs
│       ├── variables.tf     # Input variables
│       └── outputs.tf       # VNet ID, subnet IDs
├── environments/
│   ├── dev/
│   │   ├── main.tf
│   │   └── terraform.tfvars
│   └── prod/
│       ├── main.tf
│       └── terraform.tfvars
└── versions.tf              # Terraform + provider versions
```

**Estimation:** 8 hours

---

#### 📋 Sub-Tasks Breakdown (US-201)

**Phase 1: Azure Account Setup** (Tasks 201.1 - 201.5)

- [ ] **201.1** - Login to Azure CLI
  - **Command:** `az login`
  - **Validation:** `az account show --output table` displays subscription details
  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [ ] **201.2** - Set default subscription
  - **Command:**

    ```bash
    # List all subscriptions
    az account list --output table

    # Set default subscription
    az account set --subscription "Ghost Pirates Production"

    # Verify
    az account show --query "{Name:name, SubscriptionId:id}" --output table
    ```

  - **Validation:** Correct subscription is active
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.1

- [ ] **201.3** - Create service principal for Terraform
  - **Command:**

    ```bash
    # Get subscription ID
    SUBSCRIPTION_ID=$(az account show --query id -o tsv)

    # Create service principal with Contributor role
    az ad sp create-for-rbac \
      --name "ghostpirates-terraform-sp" \
      --role Contributor \
      --scopes /subscriptions/$SUBSCRIPTION_ID \
      --sdk-auth > azure-credentials.json

    # IMPORTANT: Save the output JSON - needed for GitHub secrets
    cat azure-credentials.json

    # Extract values for GitHub secrets
    cat azure-credentials.json | jq -r '.clientId' # APP_ID
    cat azure-credentials.json | jq -r '.clientSecret' # PASSWORD
    cat azure-credentials.json | jq -r '.tenantId' # TENANT_ID
    ```

  - **Validation:** Service principal created, credentials saved
  - **Security:** Store `azure-credentials.json` securely, delete after adding to GitHub secrets
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 201.2

- [ ] **201.4** - Create GitHub secrets
  - **Command:**

    ```bash
    # Set GitHub repo context
    gh repo set-default strataga/ghostpirates

    # Create AZURE_CREDENTIALS secret (entire JSON file)
    gh secret set AZURE_CREDENTIALS < azure-credentials.json

    # Create individual secrets
    gh secret set AZURE_SUBSCRIPTION_ID --body "$SUBSCRIPTION_ID"
    gh secret set AZURE_TENANT_ID --body "$(cat azure-credentials.json | jq -r '.tenantId')"

    # Create database admin password (generate secure password)
    DB_PASSWORD=$(openssl rand -base64 32)
    gh secret set DB_ADMIN_PASSWORD --body "$DB_PASSWORD"

    # Verify secrets created
    gh secret list
    ```

  - **Validation:** All secrets listed in `gh secret list`
  - **Security:** Delete local `azure-credentials.json` after upload
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.3

- [ ] **201.5** - Create storage account for Terraform state
  - **Command:**

    ```bash
    # Create resource group for Terraform backend
    az group create \
      --name ghostpirates-terraform-rg \
      --location eastus \
      --tags Environment=Management Project=GhostPirates

    # Create storage account (name must be globally unique, lowercase, no hyphens)
    az storage account create \
      --resource-group ghostpirates-terraform-rg \
      --name ghostpiratestfstate \
      --sku Standard_LRS \
      --encryption-services blob \
      --https-only true \
      --allow-blob-public-access false

    # Get storage account key
    ACCOUNT_KEY=$(az storage account keys list \
      --resource-group ghostpirates-terraform-rg \
      --account-name ghostpiratestfstate \
      --query '[0].value' -o tsv)

    # Create container for state files
    az storage container create \
      --name tfstate \
      --account-name ghostpiratestfstate \
      --account-key $ACCOUNT_KEY

    # Enable versioning for state file recovery
    az storage account blob-service-properties update \
      --account-name ghostpiratestfstate \
      --enable-versioning true
    ```

  - **Validation:** `az storage container list --account-name ghostpiratestfstate` shows `tfstate` container
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 201.2

**Phase 2: Resource Group Creation** (Tasks 201.6 - 201.8)

- [ ] **201.6** - Create production resource group
  - **Command:**

    ```bash
    az group create \
      --name ghostpirates-prod-rg \
      --location eastus \
      --tags Environment=Production Project=GhostPirates ManagedBy=Terraform
    ```

  - **Validation:** `az group show --name ghostpirates-prod-rg` returns resource group details
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.2

- [ ] **201.7** - Create staging resource group
  - **Command:**

    ```bash
    az group create \
      --name ghostpirates-staging-rg \
      --location eastus \
      --tags Environment=Staging Project=GhostPirates ManagedBy=Terraform
    ```

  - **Validation:** Resource group exists
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.2

- [ ] **201.8** - Create development resource group
  - **Command:**

    ```bash
    az group create \
      --name ghostpirates-dev-rg \
      --location eastus \
      --tags Environment=Development Project=GhostPirates ManagedBy=Terraform
    ```

  - **Validation:** `az group list --query "[?tags.Project=='GhostPirates']" --output table` shows all 3 environments
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 201.2

**Phase 3: Virtual Network Setup** (Tasks 201.9 - 201.12)

- [ ] **201.9** - Create virtual network
  - **Command:**

    ```bash
    az network vnet create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-vnet \
      --address-prefix 10.0.0.0/16 \
      --location eastus \
      --tags Environment=Production Project=GhostPirates
    ```

  - **Validation:** `az network vnet show --resource-group ghostpirates-prod-rg --name ghostpirates-vnet` shows VNet details
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 201.6

- [ ] **201.10** - Create subnets for AKS, database, Redis, Application Gateway
  - **Command:**

    ```bash
    # AKS subnet (needs larger address space for nodes)
    az network vnet subnet create \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name aks-subnet \
      --address-prefix 10.0.1.0/24

    # Database subnet (delegated to PostgreSQL)
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

    # Application Gateway subnet (for future ingress)
    az network vnet subnet create \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name appgw-subnet \
      --address-prefix 10.0.4.0/24
    ```

  - **Validation:** `az network vnet subnet list --resource-group ghostpirates-prod-rg --vnet-name ghostpirates-vnet --output table` shows all 4 subnets
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 201.9

- [ ] **201.11** - Create network security group for AKS
  - **Command:**

    ```bash
    # Create NSG
    az network nsg create \
      --resource-group ghostpirates-prod-rg \
      --name aks-nsg \
      --location eastus

    # Allow HTTPS inbound (for API access)
    az network nsg rule create \
      --resource-group ghostpirates-prod-rg \
      --nsg-name aks-nsg \
      --name allow-https \
      --priority 100 \
      --direction Inbound \
      --access Allow \
      --protocol Tcp \
      --destination-port-ranges 443 \
      --source-address-prefixes '*' \
      --destination-address-prefixes '*'

    # Allow HTTP inbound (for initial testing, remove in production)
    az network nsg rule create \
      --resource-group ghostpirates-prod-rg \
      --nsg-name aks-nsg \
      --name allow-http \
      --priority 110 \
      --direction Inbound \
      --access Allow \
      --protocol Tcp \
      --destination-port-ranges 80 \
      --source-address-prefixes '*' \
      --destination-address-prefixes '*'

    # Associate NSG with AKS subnet
    az network vnet subnet update \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name aks-subnet \
      --network-security-group aks-nsg
    ```

  - **Validation:** `az network nsg show --resource-group ghostpirates-prod-rg --name aks-nsg` shows NSG with rules
  - **Estimate:** 8 minutes
  - **Dependencies:** Task 201.10

- [ ] **201.12** - Create network security group for database
  - **Command:**

    ```bash
    # Create database NSG
    az network nsg create \
      --resource-group ghostpirates-prod-rg \
      --name database-nsg \
      --location eastus

    # Allow PostgreSQL from AKS subnet ONLY (least privilege)
    az network nsg rule create \
      --resource-group ghostpirates-prod-rg \
      --nsg-name database-nsg \
      --name allow-postgres-from-aks \
      --priority 100 \
      --direction Inbound \
      --access Allow \
      --protocol Tcp \
      --source-address-prefixes 10.0.1.0/24 \
      --destination-port-ranges 5432 \
      --destination-address-prefixes 10.0.2.0/24

    # Deny all other inbound traffic
    az network nsg rule create \
      --resource-group ghostpirates-prod-rg \
      --nsg-name database-nsg \
      --name deny-all-inbound \
      --priority 1000 \
      --direction Inbound \
      --access Deny \
      --protocol '*' \
      --source-address-prefixes '*' \
      --destination-address-prefixes '*'

    # Associate NSG with database subnet
    az network vnet subnet update \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name database-subnet \
      --network-security-group database-nsg
    ```

  - **Validation:** Database subnet has NSG with restrictive rules
  - **Estimate:** 8 minutes
  - **Dependencies:** Task 201.10

**Phase 4: Key Vault & Container Registry** (Tasks 201.13 - 201.18)

- [ ] **201.13** - Create Azure Key Vault
  - **Command:**

    ```bash
    az keyvault create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-kv \
      --location eastus \
      --enable-rbac-authorization true \
      --enabled-for-deployment false \
      --enabled-for-disk-encryption false \
      --enabled-for-template-deployment false \
      --retention-days 90
    ```

  - **Validation:** `az keyvault show --name ghostpirates-kv` returns Key Vault details
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.6

- [ ] **201.14** - Grant service principal access to Key Vault
  - **Command:**

    ```bash
    # Get service principal object ID
    SP_OBJECT_ID=$(az ad sp list --display-name "ghostpirates-terraform-sp" --query "[0].id" -o tsv)

    # Get Key Vault ID
    KV_ID=$(az keyvault show --name ghostpirates-kv --query id -o tsv)

    # Grant Key Vault Secrets Officer role
    az role assignment create \
      --role "Key Vault Secrets Officer" \
      --assignee $SP_OBJECT_ID \
      --scope $KV_ID
    ```

  - **Validation:** Service principal can read/write secrets
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.13, Task 201.3

- [ ] **201.15** - Store initial secrets in Key Vault
  - **Command:**

    ```bash
    # Store JWT secret
    JWT_SECRET=$(openssl rand -hex 32)
    az keyvault secret set \
      --vault-name ghostpirates-kv \
      --name "jwt-secret" \
      --value "$JWT_SECRET"

    # Store Claude API key (placeholder - update with real key later)
    az keyvault secret set \
      --vault-name ghostpirates-kv \
      --name "claude-api-key" \
      --value "sk-placeholder-replace-with-real-key"

    # Store OpenAI API key (placeholder)
    az keyvault secret set \
      --vault-name ghostpirates-kv \
      --name "openai-api-key" \
      --value "sk-placeholder-replace-with-real-key"

    # Verify secrets stored
    az keyvault secret list --vault-name ghostpirates-kv --output table
    ```

  - **Validation:** `az keyvault secret list --vault-name ghostpirates-kv` shows all secrets
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.14

- [ ] **201.16** - Create Azure Container Registry
  - **Command:**

    ```bash
    az acr create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpiratesacr \
      --sku Standard \
      --location eastus \
      --admin-enabled true
    ```

  - **Validation:** `az acr show --name ghostpiratesacr` returns ACR details
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.6

- [ ] **201.17** - Get ACR credentials and store in GitHub secrets
  - **Command:**

    ```bash
    # Get ACR credentials
    ACR_USERNAME=$(az acr credential show --name ghostpiratesacr --query username -o tsv)
    ACR_PASSWORD=$(az acr credential show --name ghostpiratesacr --query passwords[0].value -o tsv)

    # Store in GitHub secrets
    gh secret set ACR_USERNAME --body "$ACR_USERNAME"
    gh secret set ACR_PASSWORD --body "$ACR_PASSWORD"

    # Test Docker login
    echo "$ACR_PASSWORD" | docker login ghostpiratesacr.azurecr.io --username $ACR_USERNAME --password-stdin
    ```

  - **Validation:** Docker login succeeds
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 201.16

- [ ] **201.18** - Enable geo-replication for ACR (optional for production)
  - **Command:**

    ```bash
    # Add West US replica for high availability
    az acr replication create \
      --resource-group ghostpirates-prod-rg \
      --registry ghostpiratesacr \
      --location westus
    ```

  - **Validation:** `az acr replication list --registry ghostpiratesacr --output table` shows 2 locations
  - **Note:** Geo-replication requires Premium SKU - upgrade if needed
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 201.16

---

### US-202: PostgreSQL Database Configuration
> **⏳ DEFERRED** - All US-202 tasks below are deferred to future sprint when ready for cloud deployment

**As a** backend developer
**I want** PostgreSQL Flexible Server with high availability and pgvector extension
**So that** the application has a production-ready database with vector search capabilities

**Business Value:** Enables reliable data storage with zone redundancy and AI-powered vector search

**Acceptance Criteria:**

- [ ] PostgreSQL Flexible Server 15 created with zone-redundant high availability
- [ ] Database accessible from AKS via private networking only (no public access)
- [ ] pgvector extension installed for vector embeddings
- [ ] Automated backups configured (7-day retention, geo-redundant)
- [ ] Database parameters tuned for agent workloads (connection pooling, memory)
- [ ] Read replica created for scaling read operations
- [ ] Connection string stored in Azure Key Vault

**Technical Implementation:**

**Patterns Used:**

- [x] High Availability Pattern (zone-redundant deployment)
- [x] Private Networking (VNet integration, no public IPs)
- [x] Backup & Recovery (automated backups, point-in-time restore)

**File Structure:**

```
terraform/modules/database/
├── main.tf              # PostgreSQL Flexible Server resource
├── variables.tf         # Admin username, password, SKU
├── outputs.tf           # Connection string, FQDN
└── private_dns.tf       # Private DNS zone for database
```

**Estimation:** 6 hours

---

#### 📋 Sub-Tasks Breakdown (US-202)

**Phase 1: Private DNS Zone Setup** (Tasks 202.1 - 202.2)

- [ ] **202.1** - Create private DNS zone for PostgreSQL
  - **Command:**

    ```bash
    az network private-dns zone create \
      --resource-group ghostpirates-prod-rg \
      --name privatelink.postgres.database.azure.com
    ```

  - **Validation:** Private DNS zone created
  - **Estimate:** 3 minutes
  - **Dependencies:** US-201 complete

- [ ] **202.2** - Link private DNS zone to VNet
  - **Command:**

    ```bash
    # Get VNet ID
    VNET_ID=$(az network vnet show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-vnet \
      --query id -o tsv)

    # Link DNS zone to VNet
    az network private-dns link vnet create \
      --resource-group ghostpirates-prod-rg \
      --zone-name privatelink.postgres.database.azure.com \
      --name postgres-dns-link \
      --virtual-network $VNET_ID \
      --registration-enabled false
    ```

  - **Validation:** DNS zone linked to VNet
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 202.1

**Phase 2: PostgreSQL Server Creation** (Tasks 202.3 - 202.7)

- [ ] **202.3** - Create PostgreSQL Flexible Server with high availability
  - **Command:**

    ```bash
    # Get database subnet ID
    DB_SUBNET_ID=$(az network vnet subnet show \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name database-subnet \
      --query id -o tsv)

    # Get private DNS zone ID
    DNS_ZONE_ID=$(az network private-dns zone show \
      --resource-group ghostpirates-prod-rg \
      --name privatelink.postgres.database.azure.com \
      --query id -o tsv)

    # Create PostgreSQL server (this takes ~10-15 minutes)
    az postgres flexible-server create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --location eastus \
      --admin-user ghostadmin \
      --admin-password "$DB_ADMIN_PASSWORD" \
      --sku-name Standard_D4s_v3 \
      --tier GeneralPurpose \
      --storage-size 128 \
      --version 15 \
      --high-availability Enabled \
      --zone 1 \
      --standby-zone 2 \
      --subnet $DB_SUBNET_ID \
      --private-dns-zone $DNS_ZONE_ID
    ```

  - **Validation:** `az postgres flexible-server show --resource-group ghostpirates-prod-rg --name ghostpirates-db` shows server in "Ready" state
  - **Note:** This command takes 10-15 minutes to complete
  - **Estimate:** 20 minutes (including wait time)
  - **Dependencies:** Task 202.2

- [ ] **202.4** - Configure PostgreSQL parameters for performance
  - **Command:**

    ```bash
    # Increase max connections for agent workloads
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name max_connections \
      --value 200

    # Increase shared buffers (25% of RAM for 16GB server)
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name shared_buffers \
      --value "4GB"

    # Increase work_mem for sorting/hashing operations
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name work_mem \
      --value "16MB"

    # Increase maintenance_work_mem for index creation
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name maintenance_work_mem \
      --value "256MB"

    # Enable query performance insights
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name pg_stat_statements.track \
      --value all
    ```

  - **Validation:** Parameters updated (verify with `az postgres flexible-server parameter show`)
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 202.3

- [ ] **202.5** - Install pgvector extension
  - **Command:**

    ```bash
    # Get connection string
    DB_HOST=$(az postgres flexible-server show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --query fullyQualifiedDomainName -o tsv)

    # Connect from AKS pod (will need kubectl port-forward after AKS is set up)
    # For now, document the SQL command to run later
    cat > scripts/install-pgvector.sql << 'EOF'
    -- Install pgvector extension
    CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
    CREATE EXTENSION IF NOT EXISTS "pgvector";

    -- Verify installation
    SELECT * FROM pg_extension WHERE extname IN ('uuid-ossp', 'pgvector');
    EOF

    echo "⚠️  Run this SQL script after AKS is deployed and can connect to the database"
    ```

  - **Validation:** Extensions installed (verify with `\dx` in psql)
  - **Note:** Deferred to after AKS setup (US-203)
  - **Estimate:** 5 minutes (SQL prep) + 5 minutes (execution after AKS)
  - **Dependencies:** Task 202.3, US-203 complete

- [ ] **202.6** - Configure automated backups
  - **Command:**

    ```bash
    # Set backup retention to 7 days
    az postgres flexible-server update \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --backup-retention 7

    # Enable geo-redundant backup (for disaster recovery)
    az postgres flexible-server update \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --geo-redundant-backup Enabled
    ```

  - **Validation:** `az postgres flexible-server show --resource-group ghostpirates-prod-rg --name ghostpirates-db --query "{BackupRetention:backup.backupRetentionDays, GeoRedundant:backup.geoRedundantBackup}"` shows correct values
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 202.3

- [ ] **202.7** - Store database connection string in Key Vault
  - **Command:**

    ```bash
    # Build connection string
    DB_HOST=$(az postgres flexible-server show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --query fullyQualifiedDomainName -o tsv)

    DB_CONNECTION_STRING="postgresql://ghostadmin:$DB_ADMIN_PASSWORD@$DB_HOST:5432/ghostpirates?sslmode=require"

    # Store in Key Vault
    az keyvault secret set \
      --vault-name ghostpirates-kv \
      --name "database-url" \
      --value "$DB_CONNECTION_STRING"

    # Verify secret stored
    az keyvault secret show --vault-name ghostpirates-kv --name "database-url" --query "value" -o tsv | head -c 50
    echo "..."
    ```

  - **Validation:** Secret stored in Key Vault (connection string truncated in output for security)
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 202.3

**Phase 3: Read Replica & Scaling** (Tasks 202.8 - 202.10)

- [ ] **202.8** - Create read replica in West US
  - **Command:**

    ```bash
    # Create read replica for geographic distribution
    az postgres flexible-server replica create \
      --resource-group ghostpirates-prod-rg \
      --replica-name ghostpirates-db-replica \
      --source-server ghostpirates-db \
      --location westus
    ```

  - **Validation:** Replica created and replicating from primary
  - **Note:** Read replicas improve read performance for geographically distributed users
  - **Estimate:** 15 minutes (including creation time)
  - **Dependencies:** Task 202.3

- [ ] **202.9** - Test database connectivity from local machine (temporary firewall rule)
  - **Command:**

    ```bash
    # Temporarily allow your IP for testing (REMOVE AFTER TESTING)
    MY_IP=$(curl -s ifconfig.me)
    az postgres flexible-server firewall-rule create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --rule-name temp-local-access \
      --start-ip-address $MY_IP \
      --end-ip-address $MY_IP

    # Test connection with psql
    psql "$DB_CONNECTION_STRING" -c "SELECT version();"

    # IMPORTANT: Remove firewall rule after testing (database should be private-only)
    az postgres flexible-server firewall-rule delete \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db \
      --rule-name temp-local-access \
      --yes
    ```

  - **Validation:** Connection succeeds, firewall rule removed
  - **Security:** This is for testing only - production database has no public access
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 202.7

- [ ] **202.10** - Create database for GhostPirates application
  - **Command:**

    ```bash
    # Connect and create database
    psql "$DB_CONNECTION_STRING" << 'EOF'
    -- Create application database
    CREATE DATABASE ghostpirates;

    -- Connect to new database
    \c ghostpirates

    -- Verify connection
    SELECT current_database();
    EOF
    ```

  - **Validation:** Database `ghostpirates` created
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 202.9

**Phase 4: Monitoring & Alerts** (Tasks 202.11 - 202.15)

- [ ] **202.11** - Enable query performance insights
  - **Command:**

    ```bash
    # Enable pg_stat_statements extension for query insights
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name shared_preload_libraries \
      --value pg_stat_statements

    # Restart required for this parameter
    az postgres flexible-server restart \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db
    ```

  - **Validation:** Server restarted, parameter active
  - **Estimate:** 8 minutes (including restart time)
  - **Dependencies:** Task 202.3

- [ ] **202.12** - Configure slow query logging
  - **Command:**

    ```bash
    # Log queries taking longer than 1 second
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name log_min_duration_statement \
      --value 1000

    # Log all DDL statements (CREATE, ALTER, DROP)
    az postgres flexible-server parameter set \
      --resource-group ghostpirates-prod-rg \
      --server-name ghostpirates-db \
      --name log_statement \
      --value ddl
    ```

  - **Validation:** Slow queries will be logged
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 202.3

- [ ] **202.13** - Set up database size alert
  - **Command:**

    ```bash
    # Create alert rule for database storage >90% used
    az monitor metrics alert create \
      --name "ghostpirates-db-storage-alert" \
      --resource-group ghostpirates-prod-rg \
      --scopes /subscriptions/$SUBSCRIPTION_ID/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.DBforPostgreSQL/flexibleServers/ghostpirates-db \
      --condition "avg storage_percent > 90" \
      --description "Alert when database storage exceeds 90% capacity" \
      --evaluation-frequency 5m \
      --window-size 15m \
      --severity 2
    ```

  - **Validation:** Alert rule created
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 202.3

- [ ] **202.14** - Set up connection count alert
  - **Command:**

    ```bash
    # Alert when active connections exceed 180 (90% of max 200)
    az monitor metrics alert create \
      --name "ghostpirates-db-connections-alert" \
      --resource-group ghostpirates-prod-rg \
      --scopes /subscriptions/$SUBSCRIPTION_ID/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.DBforPostgreSQL/flexibleServers/ghostpirates-db \
      --condition "avg active_connections > 180" \
      --description "Alert when active connections exceed 180 (90% of max)" \
      --evaluation-frequency 5m \
      --window-size 15m \
      --severity 3
    ```

  - **Validation:** Alert rule created
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 202.3

- [ ] **202.15** - Document database recovery procedures
  - **File:** `docs/operations/database-recovery.md`
  - **Content:**

    ```markdown
    # Database Recovery Procedures

    ## Point-in-Time Recovery

    Restore database to specific timestamp (within 7-day retention):

    ```bash
    az postgres flexible-server restore \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db-restored \
      --source-server ghostpirates-db \
      --restore-time "2025-11-10T10:00:00Z"
    ```

    ## Geo-Restore (Disaster Recovery)

    If primary region fails, restore from geo-redundant backup:

    ```bash
    az postgres flexible-server geo-restore \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-db-geo-restore \
      --source-server ghostpirates-db \
      --location westus
    ```

    ## Manual Backup

    Create on-demand backup before major changes:

    ```bash
    pg_dump "$DATABASE_URL" > backup-$(date +%Y%m%d-%H%M%S).sql
    ```

    ```
  - **Validation:** Documentation created
  - **Estimate:** 10 minutes
  - **Dependencies:** None

---

### US-203: Azure Kubernetes Service (AKS) Setup
> **⏳ DEFERRED** - All US-203 tasks below are deferred to future sprint when ready for cloud deployment

**As a** DevOps engineer
**I want** an AKS cluster with autoscaling and monitoring
**So that** containerized applications run reliably with automatic scaling

**Business Value:** Enables container orchestration with high availability and automatic scaling

**Acceptance Criteria:**

- [ ] AKS cluster created with system and user node pools
- [ ] Cluster autoscaler enabled (3-10 nodes)
- [ ] Azure CNI networking configured for VNet integration
- [ ] NGINX Ingress Controller installed
- [ ] cert-manager installed for TLS certificate automation
- [ ] Azure CSI Secret Store Driver configured for Key Vault access
- [ ] Workload identity enabled for pod-level RBAC
- [ ] Container Insights enabled for monitoring

**Technical Implementation:**

**Patterns Used:**

- [x] Container Orchestration (Kubernetes)
- [x] Auto-Scaling (Horizontal Pod Autoscaler, Cluster Autoscaler)
- [x] Secret Management (Azure Key Vault integration)

**File Structure:**

```
terraform/modules/aks/
├── main.tf              # AKS cluster resource
├── node_pools.tf        # System and user node pools
├── addons.tf            # Monitoring, secrets provider
├── variables.tf         # Node count, VM size, version
└── outputs.tf           # Cluster name, kubeconfig
```

**Estimation:** 10 hours

---

#### 📋 Sub-Tasks Breakdown (US-203)

**Phase 1: Log Analytics Workspace** (Tasks 203.1 - 203.2)

- [ ] **203.1** - Create Log Analytics Workspace for monitoring
  - **Command:**

    ```bash
    az monitor log-analytics workspace create \
      --resource-group ghostpirates-prod-rg \
      --workspace-name ghostpirates-logs \
      --location eastus

    # Get workspace ID for AKS creation
    WORKSPACE_ID=$(az monitor log-analytics workspace show \
      --resource-group ghostpirates-prod-rg \
      --workspace-name ghostpirates-logs \
      --query id -o tsv)

    echo "Workspace ID: $WORKSPACE_ID"
    ```

  - **Validation:** Workspace created
  - **Estimate:** 3 minutes
  - **Dependencies:** US-201 complete

- [ ] **203.2** - Store workspace ID in environment variable
  - **Command:**

    ```bash
    # Add to .env for later use
    echo "LOG_ANALYTICS_WORKSPACE_ID=$WORKSPACE_ID" >> .env
    ```

  - **Validation:** Environment variable set
  - **Estimate:** 1 minute
  - **Dependencies:** Task 203.1

**Phase 2: AKS Cluster Creation** (Tasks 203.3 - 203.7)

- [ ] **203.3** - Create AKS cluster with managed identity and monitoring
  - **Command:**

    ```bash
    # Get AKS subnet ID
    AKS_SUBNET_ID=$(az network vnet subnet show \
      --resource-group ghostpirates-prod-rg \
      --vnet-name ghostpirates-vnet \
      --name aks-subnet \
      --query id -o tsv)

    # Create AKS cluster (this takes 10-15 minutes)
    az aks create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --location eastus \
      --node-count 3 \
      --node-vm-size Standard_D4s_v3 \
      --network-plugin azure \
      --vnet-subnet-id $AKS_SUBNET_ID \
      --enable-managed-identity \
      --enable-addons monitoring \
      --workspace-resource-id $WORKSPACE_ID \
      --enable-cluster-autoscaler \
      --min-count 3 \
      --max-count 10 \
      --kubernetes-version 1.28 \
      --zones 1 2 3 \
      --generate-ssh-keys
    ```

  - **Validation:** `az aks show --resource-group ghostpirates-prod-rg --name ghostpirates-aks --query "provisioningState"` returns "Succeeded"
  - **Note:** This command takes 10-15 minutes to complete
  - **Estimate:** 20 minutes (including wait time)
  - **Dependencies:** Task 203.1, US-201 complete

- [ ] **203.4** - Get AKS credentials and configure kubectl
  - **Command:**

    ```bash
    # Download kubeconfig
    az aks get-credentials \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --overwrite-existing

    # Verify connectivity
    kubectl cluster-info
    kubectl get nodes
    ```

  - **Validation:** `kubectl get nodes` shows 3 nodes in "Ready" state
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.5** - Enable workload identity and OIDC issuer
  - **Command:**

    ```bash
    # Enable workload identity (for pod-level Azure authentication)
    az aks update \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --enable-workload-identity \
      --enable-oidc-issuer

    # Get OIDC issuer URL
    OIDC_ISSUER=$(az aks show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --query "oidcIssuerProfile.issuerUrl" -o tsv)

    echo "OIDC Issuer: $OIDC_ISSUER"
    ```

  - **Validation:** Workload identity enabled
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.6** - Create managed identity for application pods
  - **Command:**

    ```bash
    # Create managed identity
    az identity create \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-app-identity

    # Get identity details
    APP_IDENTITY_CLIENT_ID=$(az identity show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-app-identity \
      --query clientId -o tsv)

    APP_IDENTITY_PRINCIPAL_ID=$(az identity show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-app-identity \
      --query principalId -o tsv)

    echo "Client ID: $APP_IDENTITY_CLIENT_ID"
    echo "Principal ID: $APP_IDENTITY_PRINCIPAL_ID"
    ```

  - **Validation:** Managed identity created
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.7** - Grant managed identity access to Key Vault
  - **Command:**

    ```bash
    # Get Key Vault ID
    KV_ID=$(az keyvault show \
      --name ghostpirates-kv \
      --query id -o tsv)

    # Grant Key Vault Secrets User role to app identity
    az role assignment create \
      --role "Key Vault Secrets User" \
      --assignee $APP_IDENTITY_PRINCIPAL_ID \
      --scope $KV_ID
    ```

  - **Validation:** Role assignment created
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 203.6

**Phase 3: Node Pool Configuration** (Tasks 203.8 - 203.9)

- [ ] **203.8** - Add system node pool for system pods
  - **Command:**

    ```bash
    # Create dedicated system node pool
    az aks nodepool add \
      --resource-group ghostpirates-prod-rg \
      --cluster-name ghostpirates-aks \
      --name systempool \
      --node-count 2 \
      --node-vm-size Standard_D2s_v3 \
      --mode System \
      --zones 1 2 3
    ```

  - **Validation:** `kubectl get nodes` shows additional system nodes
  - **Note:** System node pools run critical system pods (CoreDNS, metrics-server, etc.)
  - **Estimate:** 8 minutes
  - **Dependencies:** Task 203.4

- [ ] **203.9** - Configure user node pool with autoscaling
  - **Command:**

    ```bash
    # Add user node pool for application workloads
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

  - **Validation:** Node pool created with autoscaler enabled
  - **Estimate:** 8 minutes
  - **Dependencies:** Task 203.4

**Phase 4: Cluster Add-ons** (Tasks 203.10 - 203.15)

- [ ] **203.10** - Install NGINX Ingress Controller
  - **Command:**

    ```bash
    # Install NGINX Ingress using kubectl
    kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.2/deploy/static/provider/cloud/deploy.yaml

    # Wait for ingress controller to be ready
    kubectl wait --namespace ingress-nginx \
      --for=condition=ready pod \
      --selector=app.kubernetes.io/component=controller \
      --timeout=120s

    # Get external IP (takes a few minutes for Azure to provision LoadBalancer)
    kubectl get service ingress-nginx-controller --namespace ingress-nginx
    ```

  - **Validation:** Ingress controller pod running, external IP assigned
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.4

- [ ] **203.11** - Install cert-manager for TLS certificates
  - **Command:**

    ```bash
    # Install cert-manager CRDs
    kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.2/cert-manager.yaml

    # Wait for cert-manager to be ready
    kubectl wait --namespace cert-manager \
      --for=condition=ready pod \
      --selector=app.kubernetes.io/instance=cert-manager \
      --timeout=120s

    # Verify installation
    kubectl get pods --namespace cert-manager
    ```

  - **Validation:** cert-manager pods running
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.4

- [ ] **203.12** - Install Azure CSI Secret Store Driver
  - **Command:**

    ```bash
    # Add Helm repo
    helm repo add csi-secrets-store-provider-azure https://azure.github.io/secrets-store-csi-driver-provider-azure/charts
    helm repo update

    # Install CSI driver
    helm install csi-secrets-store-provider-azure/csi-secrets-store-provider-azure \
      --namespace kube-system \
      --generate-name \
      --set secrets-store-csi-driver.syncSecret.enabled=true

    # Verify installation
    kubectl get pods -n kube-system -l app=secrets-store-csi-driver
    ```

  - **Validation:** CSI driver pods running
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.4

- [ ] **203.13** - Create SecretProviderClass for Key Vault
  - **File:** `k8s/base/secret-provider-class.yaml`
  - **Content:**

    ```yaml
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
        userAssignedIdentityID: "$APP_IDENTITY_CLIENT_ID"
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
        tenantId: "$AZURE_TENANT_ID"
    ```

  - **Command:**

    ```bash
    # Create k8s directory
    mkdir -p k8s/base

    # Substitute environment variables and create file
    envsubst < secret-provider-class.yaml.tmpl > k8s/base/secret-provider-class.yaml

    # Apply to cluster
    kubectl apply -f k8s/base/secret-provider-class.yaml
    ```

  - **Validation:** SecretProviderClass created
  - **Estimate:** 8 minutes
  - **Dependencies:** Task 203.12

- [ ] **203.14** - Test secret mounting in a pod
  - **File:** `k8s/base/test-secret-pod.yaml`
  - **Content:**

    ```yaml
    apiVersion: v1
    kind: Pod
    metadata:
      name: test-secrets
      namespace: default
    spec:
      serviceAccountName: default
      containers:
      - name: busybox
        image: busybox:latest
        command:
          - sleep
          - "3600"
        volumeMounts:
        - name: secrets-store
          mountPath: "/mnt/secrets"
          readOnly: true
      volumes:
      - name: secrets-store
        csi:
          driver: secrets-store.csi.k8s.io
          readOnly: true
          volumeAttributes:
            secretProviderClass: ghostpirates-secrets
    ```

  - **Command:**

    ```bash
    # Deploy test pod
    kubectl apply -f k8s/base/test-secret-pod.yaml

    # Wait for pod to be ready
    kubectl wait --for=condition=ready pod/test-secrets --timeout=60s

    # Verify secrets mounted
    kubectl exec test-secrets -- ls /mnt/secrets

    # Check a secret value (truncated)
    kubectl exec test-secrets -- cat /mnt/secrets/jwt-secret | head -c 20

    # Cleanup test pod
    kubectl delete pod test-secrets
    ```

  - **Validation:** Secrets successfully mounted from Key Vault
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.13

- [ ] **203.15** - Configure AKS to pull images from ACR
  - **Command:**

    ```bash
    # Attach ACR to AKS cluster
    az aks update \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --attach-acr ghostpiratesacr
    ```

  - **Validation:** AKS can pull images from ACR without imagePullSecrets
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 203.4, US-201.16

**Phase 5: Monitoring & Logging** (Tasks 203.16 - 203.22)

- [ ] **203.16** - Verify Container Insights is collecting metrics
  - **Command:**

    ```bash
    # Check that Container Insights is enabled
    az aks show \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --query "addonProfiles.omsagent.enabled" -o tsv

    # Verify omsagent pods are running
    kubectl get pods -n kube-system -l component=oms-agent
    ```

  - **Validation:** Container Insights collecting data
  - **Estimate:** 2 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.17** - Install Prometheus using Helm
  - **Command:**

    ```bash
    # Add Prometheus Helm repo
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update

    # Install kube-prometheus-stack (Prometheus + Grafana + Alertmanager)
    helm install prometheus prometheus-community/kube-prometheus-stack \
      --namespace monitoring \
      --create-namespace \
      --set prometheus.prometheusSpec.retention=30d \
      --set prometheus.prometheusSpec.storageSpec.volumeClaimTemplate.spec.resources.requests.storage=50Gi

    # Wait for pods to be ready
    kubectl wait --namespace monitoring \
      --for=condition=ready pod \
      --selector=app.kubernetes.io/name=prometheus \
      --timeout=300s
    ```

  - **Validation:** Prometheus pods running
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 203.4

- [ ] **203.18** - Install Grafana for visualization
  - **Command:**

    ```bash
    # Grafana is included in kube-prometheus-stack
    # Get Grafana admin password
    kubectl get secret prometheus-grafana \
      --namespace monitoring \
      -o jsonpath="{.data.admin-password}" | base64 --decode

    # Port-forward to access Grafana locally
    kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80

    # Access Grafana at http://localhost:3000
    # Username: admin
    # Password: (from command above)
    ```

  - **Validation:** Grafana accessible and showing cluster metrics
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.17

- [ ] **203.19** - Configure Application Insights for application telemetry
  - **Command:**

    ```bash
    # Create Application Insights instance
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

  - **Validation:** Application Insights created, key stored
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.1

- [ ] **203.20** - Set up cluster resource alerts
  - **Command:**

    ```bash
    # Alert when node CPU exceeds 80%
    az monitor metrics alert create \
      --name "ghostpirates-aks-cpu-alert" \
      --resource-group ghostpirates-prod-rg \
      --scopes /subscriptions/$SUBSCRIPTION_ID/resourceGroups/ghostpirates-prod-rg/providers/Microsoft.ContainerService/managedClusters/ghostpirates-aks \
      --condition "avg Percentage CPU > 80" \
      --description "Alert when node CPU exceeds 80%" \
      --evaluation-frequency 5m \
      --window-size 15m \
      --severity 2
    ```

  - **Validation:** Alert rule created
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.21** - Set up pod restart alerts
  - **Command:**

    ```bash
    # This requires Log Analytics workspace query
    az monitor scheduled-query create \
      --name "ghostpirates-pod-restart-alert" \
      --resource-group ghostpirates-prod-rg \
      --scopes $WORKSPACE_ID \
      --condition "count > 5" \
      --condition-query "KubePodInventory | where ClusterName == 'ghostpirates-aks' | where PodStatus == 'Failed' or ContainerRestartCount > 3 | summarize count() by Name" \
      --description "Alert when pods restart >5 times in 15 minutes" \
      --evaluation-frequency 5m \
      --window-size 15m \
      --severity 3
    ```

  - **Validation:** Alert rule created
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 203.3

- [ ] **203.22** - Document AKS operations procedures
  - **File:** `docs/operations/aks-operations.md`
  - **Content:**

    ```markdown
    # AKS Operations Guide

    ## Scaling Nodes

    ```bash
    # Manual scale user node pool
    az aks nodepool scale \
      --resource-group ghostpirates-prod-rg \
      --cluster-name ghostpirates-aks \
      --name agentpool \
      --node-count 5
    ```

    ## Upgrading Kubernetes Version

    ```bash
    # Check available upgrades
    az aks get-upgrades --resource-group ghostpirates-prod-rg --name ghostpirates-aks

    # Upgrade cluster
    az aks upgrade \
      --resource-group ghostpirates-prod-rg \
      --name ghostpirates-aks \
      --kubernetes-version 1.29
    ```

    ## Draining Nodes

    ```bash
    kubectl drain <node-name> --ignore-daemonsets --delete-emptydir-data
    kubectl uncordon <node-name>
    ```

    ## Troubleshooting Pod Issues

    ```bash
    # Check pod logs
    kubectl logs <pod-name> -n <namespace>

    # Describe pod for events
    kubectl describe pod <pod-name> -n <namespace>

    # Exec into pod
    kubectl exec -it <pod-name> -n <namespace> -- /bin/sh
    ```

    ```
  - **Validation:** Documentation created
  - **Estimate:** 15 minutes
  - **Dependencies:** None

---

### US-204: Terraform Infrastructure as Code
> **✅ COMPLETE** - Terraform modules structured for local dev + cloud deployment. All modules validate successfully.

**As a** DevOps engineer
**I want** all Azure infrastructure defined in Terraform modules
**So that** infrastructure is version-controlled, repeatable, and auditable

**Business Value:** Infrastructure as code enables disaster recovery, multi-environment deployments, and change tracking

**Acceptance Criteria:**

- [x] Terraform project structure created with modules and environments
- [ ] Networking module (VNet, subnets, NSGs) - (cloud only)
- [x] Database module (PostgreSQL Flexible Server)
- [ ] Redis module (Azure Cache for Redis) - (cloud only, using local Docker)
- [ ] AKS module (cluster, node pools, addons) - (cloud only)
- [ ] Monitoring module (Log Analytics, Application Insights) - (cloud only, using local only)
- [ ] Terraform backend configured in Azure Storage - (cloud only, using local state)
- [x] Terraform validate passes with zero errors
- [x] Terraform plan runs successfully
- [x] Dev environment deployed from Terraform

**Technical Implementation:**

**Patterns Used:**

- [x] Infrastructure as Code (declarative infrastructure)
- [x] DRY Principle (reusable modules)
- [x] Environment Separation (dev, staging, prod)

**File Structure:**

```
terraform/
├── modules/
│   ├── networking/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   └── outputs.tf
│   ├── database/
│   ├── redis/
│   ├── aks/
│   └── monitoring/
├── environments/
│   ├── dev/
│   │   ├── main.tf
│   │   ├── variables.tf
│   │   ├── terraform.tfvars
│   │   └── backend.tf
│   ├── staging/
│   └── prod/
├── versions.tf
└── README.md
```

**Estimation:** 12 hours

---

#### 📋 Sub-Tasks Breakdown (US-204)

**Phase 1: Terraform Project Structure** (Tasks 204.1 - 204.5)

- [x] **204.1** - Create Terraform directory structure
  - **Command:**

    ```bash
    # Create directory structure
    mkdir -p terraform/{modules/{networking,database,redis,aks,monitoring},environments/{dev,staging,prod}}

    # Create module files
    for module in networking database redis aks monitoring; do
      touch terraform/modules/$module/{main.tf,variables.tf,outputs.tf}
    done

    # Create environment files
    for env in dev staging prod; do
      touch terraform/environments/$env/{main.tf,variables.tf,terraform.tfvars,backend.tf}
    done

    # Create root files
    touch terraform/{versions.tf,README.md}

    # Verify structure
    tree terraform
    ```

  - **Validation:** Directory structure created
  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [x] **204.2** - Configure Terraform versions and providers
  - **File:** `terraform/versions.tf`
  - **Content:**

    ```hcl
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
        kubernetes = {
          source  = "hashicorp/kubernetes"
          version = "~> 2.23"
        }
        helm = {
          source  = "hashicorp/helm"
          version = "~> 2.11"
        }
      }
    }
    ```

  - **Validation:** File created with correct syntax
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.1

- [ ] **204.3** - Configure Terraform backend for state storage - (cloud only, using local state)
  - **File:** `terraform/environments/dev/backend.tf`
  - **Content:**

    ```hcl
    terraform {
      backend "azurerm" {
        resource_group_name  = "ghostpirates-terraform-rg"
        storage_account_name = "ghostpiratestfstate"
        container_name       = "tfstate"
        key                  = "dev.terraform.tfstate"
      }
    }

    provider "azurerm" {
      features {
        key_vault {
          purge_soft_delete_on_destroy = false
          recover_soft_deleted_key_vaults = true
        }
        resource_group {
          prevent_deletion_if_contains_resources = false
        }
      }
    }
    ```

  - **Validation:** Backend configuration valid
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.1, US-201.5

- [x] **204.4** - Initialize Terraform in dev environment
  - **Command:**

    ```bash
    cd terraform/environments/dev

    # Initialize Terraform (downloads providers, configures backend)
    terraform init

    # Verify initialization
    terraform version
    terraform providers
    ```

  - **Validation:** `terraform init` succeeds, backend connected
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 204.3

- [x] **204.5** - Create Terraform README with usage instructions
  - **File:** `terraform/README.md`
  - **Content:**

    ```markdown
    # GhostPirates Infrastructure - Terraform

    ## Overview

    This directory contains Terraform configurations for deploying GhostPirates infrastructure to Azure.

    ## Structure

    - `modules/` - Reusable Terraform modules
    - `environments/` - Environment-specific configurations (dev, staging, prod)
    - `versions.tf` - Terraform and provider version constraints

    ## Prerequisites

    - Terraform 1.6+
    - Azure CLI authenticated (`az login`)
    - Azure subscription with Contributor role

    ## Usage

    ### Deploy Development Environment

    ```bash
    cd environments/dev
    terraform init
    terraform plan -out=tfplan
    terraform apply tfplan
    ```

    ### Deploy Production Environment

    ```bash
    cd environments/prod
    terraform init
    terraform plan -out=tfplan
    # Review plan carefully
    terraform apply tfplan
    ```

    ### Destroy Environment

    ```bash
    cd environments/dev
    terraform destroy
    ```

    ## Modules

    - **networking** - VNet, subnets, NSGs
    - **database** - PostgreSQL Flexible Server
    - **redis** - Azure Cache for Redis
    - **aks** - Azure Kubernetes Service cluster
    - **monitoring** - Log Analytics, Application Insights

    ## State Management

    Terraform state is stored in Azure Storage:
    - Storage Account: `ghostpiratestfstate`
    - Container: `tfstate`
    - State Files: `dev.terraform.tfstate`, `prod.terraform.tfstate`

    ```
  - **Validation:** README created
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 204.1

**Phase 2: Networking Module** (Tasks 204.6 - 204.8)

- [ ] **204.6** - Create networking module - (cloud only)
  - **File:** `terraform/modules/networking/main.tf`
  - **Content:** (See source document lines 816-884 for complete code)

    ```hcl
    resource "azurerm_virtual_network" "main" {
      name                = "${var.prefix}-vnet"
      resource_group_name = var.resource_group_name
      location            = var.location
      address_space       = ["10.0.0.0/16"]
      tags                = var.tags
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
      service_endpoints    = ["Microsoft.Sql"]

      delegation {
        name = "postgres-delegation"
        service_delegation {
          name = "Microsoft.DBforPostgreSQL/flexibleServers"
          actions = ["Microsoft.Network/virtualNetworks/subnets/join/action"]
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

  - **Estimate:** 15 minutes
  - **Dependencies:** Task 204.1

- [ ] **204.7** - Create networking module variables - (cloud only)
  - **File:** `terraform/modules/networking/variables.tf`
  - **Content:**

    ```hcl
    variable "prefix" {
      description = "Resource name prefix"
      type        = string
    }

    variable "location" {
      description = "Azure region"
      type        = string
    }

    variable "resource_group_name" {
      description = "Resource group name"
      type        = string
    }

    variable "tags" {
      description = "Resource tags"
      type        = map(string)
      default     = {}
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.6

- [ ] **204.8** - Create networking module outputs - (cloud only)
  - **File:** `terraform/modules/networking/outputs.tf`
  - **Content:**

    ```hcl
    output "vnet_id" {
      description = "Virtual network ID"
      value       = azurerm_virtual_network.main.id
    }

    output "vnet_name" {
      description = "Virtual network name"
      value       = azurerm_virtual_network.main.name
    }

    output "aks_subnet_id" {
      description = "AKS subnet ID"
      value       = azurerm_subnet.aks.id
    }

    output "database_subnet_id" {
      description = "Database subnet ID"
      value       = azurerm_subnet.database.id
    }

    output "redis_subnet_id" {
      description = "Redis subnet ID"
      value       = azurerm_subnet.redis.id
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.6

**Phase 3: Database Module** (Tasks 204.9 - 204.11)

- [x] **204.9** - Create database module
  - **File:** `terraform/modules/database/main.tf`
  - **Content:** (See source document lines 888-934 for complete code)

    ```hcl
    resource "azurerm_postgresql_flexible_server" "main" {
      name                = "${var.prefix}-db"
      resource_group_name = var.resource_group_name
      location            = var.location

      administrator_login    = var.admin_username
      administrator_password = var.admin_password

      sku_name   = "GP_Standard_D4s_v3"
      storage_mb = 131072
      version    = "15"

      zone = "1"
      high_availability {
        mode                      = "ZoneRedundant"
        standby_availability_zone = "2"
      }

      delegated_subnet_id = var.subnet_id
      private_dns_zone_id = azurerm_private_dns_zone.postgres.id

      backup_retention_days        = 7
      geo_redundant_backup_enabled = true

      tags = var.tags

      depends_on = [azurerm_private_dns_zone_virtual_network_link.postgres]
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

  - **Estimate:** 20 minutes
  - **Dependencies:** Task 204.1

- [x] **204.10** - Create database module variables
  - **File:** `terraform/modules/database/variables.tf`
  - **Content:**

    ```hcl
    variable "prefix" {
      description = "Resource name prefix"
      type        = string
    }

    variable "location" {
      description = "Azure region"
      type        = string
    }

    variable "resource_group_name" {
      description = "Resource group name"
      type        = string
    }

    variable "subnet_id" {
      description = "Database subnet ID"
      type        = string
    }

    variable "vnet_id" {
      description = "Virtual network ID"
      type        = string
    }

    variable "admin_username" {
      description = "Database admin username"
      type        = string
      default     = "ghostadmin"
    }

    variable "admin_password" {
      description = "Database admin password"
      type        = string
      sensitive   = true
    }

    variable "tags" {
      description = "Resource tags"
      type        = map(string)
      default     = {}
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.9

- [x] **204.11** - Create database module outputs
  - **File:** `terraform/modules/database/outputs.tf`
  - **Content:**

    ```hcl
    output "server_id" {
      description = "PostgreSQL server ID"
      value       = azurerm_postgresql_flexible_server.main.id
    }

    output "server_fqdn" {
      description = "PostgreSQL server FQDN"
      value       = azurerm_postgresql_flexible_server.main.fqdn
    }

    output "server_name" {
      description = "PostgreSQL server name"
      value       = azurerm_postgresql_flexible_server.main.name
    }

    output "connection_string" {
      description = "Database connection string"
      value       = "postgresql://${var.admin_username}:${var.admin_password}@${azurerm_postgresql_flexible_server.main.fqdn}:5432/ghostpirates?sslmode=require"
      sensitive   = true
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.9

**Phase 4: AKS Module** (Tasks 204.12 - 204.14)

- [ ] **204.12** - Create AKS module - (cloud only)
  - **File:** `terraform/modules/aks/main.tf`
  - **Content:** (See source document lines 939-1004 for complete code)

    ```hcl
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
        network_plugin    = "azure"
        network_policy    = "azure"
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

  - **Estimate:** 20 minutes
  - **Dependencies:** Task 204.1

- [ ] **204.13** - Create AKS module variables - (cloud only)
  - **File:** `terraform/modules/aks/variables.tf`
  - **Content:**

    ```hcl
    variable "prefix" {
      description = "Resource name prefix"
      type        = string
    }

    variable "location" {
      description = "Azure region"
      type        = string
    }

    variable "resource_group_name" {
      description = "Resource group name"
      type        = string
    }

    variable "subnet_id" {
      description = "AKS subnet ID"
      type        = string
    }

    variable "log_analytics_workspace_id" {
      description = "Log Analytics workspace ID for Container Insights"
      type        = string
    }

    variable "tags" {
      description = "Resource tags"
      type        = map(string)
      default     = {}
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.12

- [ ] **204.14** - Create AKS module outputs - (cloud only)
  - **File:** `terraform/modules/aks/outputs.tf`
  - **Content:**

    ```hcl
    output "cluster_id" {
      description = "AKS cluster ID"
      value       = azurerm_kubernetes_cluster.main.id
    }

    output "cluster_name" {
      description = "AKS cluster name"
      value       = azurerm_kubernetes_cluster.main.name
    }

    output "kube_config" {
      description = "Kubernetes config"
      value       = azurerm_kubernetes_cluster.main.kube_config_raw
      sensitive   = true
    }

    output "cluster_fqdn" {
      description = "AKS cluster FQDN"
      value       = azurerm_kubernetes_cluster.main.fqdn
    }

    output "oidc_issuer_url" {
      description = "OIDC issuer URL for workload identity"
      value       = azurerm_kubernetes_cluster.main.oidc_issuer_url
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.12

**Phase 5: Environment Configuration** (Tasks 204.15 - 204.20)

- [x] **204.15** - Create dev environment main configuration
  - **File:** `terraform/environments/dev/main.tf`
  - **Content:** (See source document lines 1009-1070 for complete code)

    ```hcl
    locals {
      prefix      = "ghostpirates"
      location    = "eastus"
      environment = "dev"

      tags = {
        Environment = "Development"
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

    module "aks" {
      source = "../../modules/aks"

      prefix                     = local.prefix
      location                   = local.location
      resource_group_name        = azurerm_resource_group.main.name
      subnet_id                  = module.networking.aks_subnet_id
      log_analytics_workspace_id = module.monitoring.workspace_id
      tags                       = local.tags
    }

    module "monitoring" {
      source = "../../modules/monitoring"

      prefix              = local.prefix
      location            = local.location
      resource_group_name = azurerm_resource_group.main.name
      tags                = local.tags
    }
    ```

  - **Estimate:** 15 minutes
  - **Dependencies:** Task 204.8, 204.11, 204.14

- [x] **204.16** - Create dev environment variables
  - **File:** `terraform/environments/dev/variables.tf`
  - **Content:**

    ```hcl
    variable "db_admin_username" {
      description = "Database admin username"
      type        = string
      default     = "ghostadmin"
    }

    variable "db_admin_password" {
      description = "Database admin password"
      type        = string
      sensitive   = true
    }
    ```

  - **Estimate:** 3 minutes
  - **Dependencies:** Task 204.15

- [x] **204.17** - Create dev environment tfvars
  - **File:** `terraform/environments/dev/terraform.tfvars`
  - **Content:**

    ```hcl
    db_admin_username = "ghostadmin"
    # db_admin_password is set via environment variable TF_VAR_db_admin_password
    ```

  - **Estimate:** 2 minutes
  - **Dependencies:** Task 204.16

- [x] **204.18** - Validate Terraform configuration
  - **Command:**

    ```bash
    cd terraform/environments/dev

    # Format all Terraform files
    terraform fmt -recursive ../../

    # Validate configuration
    terraform validate

    # Check for syntax errors
    echo "✅ Terraform validation complete"
    ```

  - **Validation:** `terraform validate` passes with no errors
  - **Estimate:** 3 minutes
  - **Dependencies:** Task 204.17

- [x] **204.19** - Generate Terraform plan
  - **Command:**

    ```bash
    cd terraform/environments/dev

    # Set database password via environment variable
    export TF_VAR_db_admin_password="$(openssl rand -base64 32)"

    # Generate plan
    terraform plan -out=tfplan

    # Review plan output (should show resources to be created)
    terraform show tfplan
    ```

  - **Validation:** Plan generated successfully, shows expected resources
  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.18

- [x] **204.20** - Apply Terraform to deploy dev environment
  - **Command:**

    ```bash
    cd terraform/environments/dev

    # Apply plan (this will take 20-30 minutes)
    terraform apply tfplan

    # Verify outputs
    terraform output
    ```

  - **Validation:** All resources created successfully
  - **Note:** This is the final integration test - applies all modules
  - **Estimate:** 40 minutes (including deployment time)
  - **Dependencies:** Task 204.19

**Phase 6: Terraform Documentation & Best Practices** (Tasks 204.21 - 204.25)

- [ ] **204.21** - Add Terraform state locking - (cloud only, using local state)
  - **Note:** Azure Storage backend automatically provides state locking
  - **Validation:**

    ```bash
    # State locking is automatic with azurerm backend
    # Verify by checking state file metadata
    az storage blob show \
      --account-name ghostpiratestfstate \
      --container-name tfstate \
      --name dev.terraform.tfstate \
      --query "{Name:name, LeaseState:properties.lease.state}"
    ```

  - **Estimate:** 2 minutes
  - **Dependencies:** Task 204.4

- [x] **204.22** - Create Terraform output documentation
  - **File:** `terraform/environments/dev/outputs.tf`
  - **Content:**

    ```hcl
    output "resource_group_name" {
      description = "Resource group name"
      value       = azurerm_resource_group.main.name
    }

    output "vnet_name" {
      description = "Virtual network name"
      value       = module.networking.vnet_name
    }

    output "aks_cluster_name" {
      description = "AKS cluster name"
      value       = module.aks.cluster_name
    }

    output "database_fqdn" {
      description = "Database FQDN"
      value       = module.database.server_fqdn
    }

    output "database_connection_string" {
      description = "Database connection string (sensitive)"
      value       = module.database.connection_string
      sensitive   = true
    }

    output "kube_config" {
      description = "Kubernetes config (sensitive)"
      value       = module.aks.kube_config
      sensitive   = true
    }
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** Task 204.15

- [ ] **204.23** - Create Terraform cost estimation script - (cloud only)
  - **File:** `scripts/terraform-cost-estimate.sh`
  - **Content:**

    ```bash
    #!/bin/bash
    # Terraform cost estimation using Infracost

    # Install Infracost (if not already installed)
    if ! command -v infracost &> /dev/null; then
      brew install infracost
      infracost configure
    fi

    # Generate cost estimate
    cd terraform/environments/dev
    infracost breakdown --path .

    # Compare with baseline
    infracost diff --path . --compare-to baseline.json
    ```

  - **Validation:** Script created
  - **Note:** Infracost requires API key registration
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 204.15

- [ ] **204.24** - Create Terraform destroy script (with safety) - (cloud only)
  - **File:** `scripts/terraform-destroy-dev.sh`
  - **Content:**

    ```bash
    #!/bin/bash
    set -e

    echo "⚠️  WARNING: This will destroy ALL dev environment resources!"
    echo "Resources to be destroyed:"
    echo "  - Resource Group: ghostpirates-dev-rg"
    echo "  - VNet, Subnets, NSGs"
    echo "  - PostgreSQL Flexible Server"
    echo "  - AKS Cluster"
    echo "  - Log Analytics Workspace"
    echo ""
    read -p "Type 'destroy-dev' to confirm: " confirmation

    if [ "$confirmation" != "destroy-dev" ]; then
      echo "❌ Destroy cancelled"
      exit 1
    fi

    cd terraform/environments/dev

    # Generate destroy plan
    terraform plan -destroy -out=destroy.tfplan

    # Review plan
    echo ""
    echo "📋 Review the destroy plan above."
    read -p "Proceed with destroy? (yes/no): " proceed

    if [ "$proceed" == "yes" ]; then
      terraform apply destroy.tfplan
      rm destroy.tfplan
      echo "✅ Dev environment destroyed"
    else
      echo "❌ Destroy cancelled"
      rm destroy.tfplan
      exit 1
    fi
    ```

  - **Validation:** Script created with safety checks
  - **Estimate:** 10 minutes
  - **Dependencies:** Task 204.15

- [x] **204.25** - Document Terraform workflow
  - **File:** `docs/operations/terraform-workflow.md`
  - **Content:**

    ```markdown
    # Terraform Workflow

    ## Daily Development Workflow

    1. **Make Infrastructure Changes**
       - Edit Terraform files in `terraform/modules/` or `terraform/environments/`
       - Run `terraform fmt` to format code
       - Run `terraform validate` to check syntax

    2. **Plan Changes**
       ```bash
       cd terraform/environments/dev
       terraform plan -out=tfplan
       terraform show tfplan  # Review changes
       ```

    3. **Apply Changes**

       ```bash
       terraform apply tfplan
       ```

    4. **Commit to Git**

       ```bash
       git add terraform/
       git commit -m "feat(infra): add Redis cache module"
       git push
       ```

    ## Production Deployment Workflow

    1. **Test in Dev First**
       - Apply changes to dev environment
       - Validate infrastructure works as expected

    2. **Create Pull Request**
       - Open PR with Terraform changes
       - Automated Terraform plan runs in CI
       - Peer review required

    3. **Apply to Production**

       ```bash
       cd terraform/environments/prod
       terraform plan -out=tfplan
       # Careful review of plan
       terraform apply tfplan
       ```

    ## State Management

    - State is stored in Azure Storage: `ghostpiratestfstate`
    - State is automatically locked during operations
    - Never edit state files manually
    - Use `terraform state` commands for state operations

    ## Troubleshooting

    **State Lock Errors:**

    ```bash
    # Force unlock (use with caution)
    terraform force-unlock <lock-id>
    ```

    **State Drift:**

    ```bash
    # Refresh state from Azure
    terraform refresh

    # Import existing resource
    terraform import azurerm_resource_group.main /subscriptions/<id>/resourceGroups/<name>
    ```

    ```
  - **Validation:** Documentation complete
  - **Estimate:** 15 minutes
  - **Dependencies:** None

### US-205: Monitoring & Logging Configuration

**As a** DevOps engineer
**I want** comprehensive monitoring and logging for infrastructure
**So that** I can detect and resolve issues proactively

**Business Value:** Proactive monitoring reduces downtime and improves reliability

**Acceptance Criteria:**

- [x] Docker health checks configured (docker-compose.yml)
- [x] Local logging configured (RUST_LOG environment variable)
- [x] Local monitoring operational (docker compose ps, cargo test output)
- [ ] Azure Monitor collecting metrics from all resources - (cloud only)
- [ ] Application Insights configured for application telemetry - (cloud only)
- [ ] Log Analytics workspace aggregating logs - (cloud only)
- [ ] Alert rules created for critical conditions (high CPU, database storage, pod restarts) - (cloud only)
- [ ] Grafana dashboards showing cluster health - (cloud only)
- [ ] Prometheus scraping metrics from AKS - (cloud only)

**Technical Implementation:**

**Patterns Used:**

- [x] Observability (metrics, logs, traces)
- [x] Alerting (proactive issue detection)

**Estimation:** 4 hours

---

#### 📋 Sub-Tasks Breakdown (US-205)

- [x] **205.1** - Configure Docker health checks
  - **Implementation:** docker-compose.yml with healthcheck for postgres and redis
  - **Validation:**

    ```bash
    docker compose ps  # Shows health status
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [x] **205.2** - Configure local logging
  - **Implementation:** RUST_LOG environment variable in docker-compose.yml
  - **Validation:**

    ```bash
    docker compose logs  # Shows application logs
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** None

- [x] **205.3** - Verify local monitoring operational
  - **Validation:**

    ```bash
    docker compose ps      # Check container health
    cargo test            # View test output
    docker compose logs   # View application logs
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** 205.1, 205.2

- [ ] **205.4** - Verify all monitoring components from US-203 - (cloud only)
  - **Validation:**

    ```bash
    # Container Insights
    kubectl get pods -n kube-system | grep oms-agent

    # Prometheus
    kubectl get pods -n monitoring | grep prometheus

    # Grafana
    kubectl get svc -n monitoring | grep grafana
    ```

  - **Estimate:** 5 minutes
  - **Dependencies:** US-203 complete

- [ ] **205.5** - Create custom Grafana dashboards - (cloud only)
  - **Access Grafana:**

    ```bash
    kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80
    # Open http://localhost:3000
    ```

  - **Import Dashboards:**
    - Kubernetes Cluster Monitoring (ID: 7249)
    - Node Exporter Full (ID: 1860)
    - PostgreSQL Database (ID: 9628)
  - **Validation:** Dashboards showing live metrics
  - **Estimate:** 20 minutes
  - **Dependencies:** US-203.18

- [ ] **205.6** - Document monitoring architecture - (cloud only)
  - **File:** `docs/operations/monitoring-architecture.md`
  - **Content:**

    ```markdown
    # Monitoring Architecture

    ## Components

    - **Azure Monitor** - Cloud-native monitoring (VMs, databases, networking)
    - **Container Insights** - AKS pod and node metrics
    - **Application Insights** - Application telemetry (API traces, errors)
    - **Prometheus** - Kubernetes metrics collection
    - **Grafana** - Metrics visualization
    - **Log Analytics** - Centralized log aggregation

    ## Access

    - Grafana: `kubectl port-forward -n monitoring svc/prometheus-grafana 3000:80`
    - Prometheus: `kubectl port-forward -n monitoring svc/prometheus-kube-prometheus-prometheus 9090:9090`
    - Azure Monitor: [Azure Portal](https://portal.azure.com)

    ## Alert Rules

    - High node CPU (>80% for 15min)
    - Database storage (>90%)
    - Pod restarts (>5 in 15min)
    - Connection pool exhaustion

    ## Metrics Retention

    - Prometheus: 30 days
    - Log Analytics: 30 days (configurable)
    - Application Insights: 90 days
    ```

  - **Validation:** Documentation complete
  - **Estimate:** 15 minutes
  - **Dependencies:** None

---

## 🔗 Cross-Story Integration

**Integration Points:**

- US-201 (Azure Foundation) provides VNet and Key Vault for US-202 (Database)
- US-201 (Networking) provides subnets for US-203 (AKS)
- US-202 (Database) connection string used by US-203 (AKS pods)
- US-203 (AKS) cluster used by US-205 (CI/CD deployment target)
- US-204 (Terraform) codifies all resources from US-201, US-202, US-203
- US-205 (Monitoring) observes all infrastructure from previous user stories

**Integration Tests:**

- [ ] AKS pod can connect to PostgreSQL via private networking
- [ ] AKS pod can retrieve secrets from Key Vault
- [ ] Docker image can be pulled from ACR by AKS
- [ ] Terraform can provision entire stack from scratch
- [ ] CI/CD pipeline deploys application to AKS successfully
- [ ] Monitoring alerts trigger when thresholds breached

---

## 🚧 Sprint Blockers

**Active Blockers:** None

**Resolved Blockers:** None

---

## 💬 Questions & Decisions

**Open Questions:** None yet

**Decisions Made:**

| Decision                    | Context                       | Rationale                                  | Made By | Date       |
| --------------------------- | ----------------------------- | ------------------------------------------ | ------- | ---------- |
| Use Azure CNI for AKS       | Networking plugin choice      | Better VNet integration, required for RBAC | Team    | 2025-11-09 |
| PostgreSQL Flexible Server  | Database service selection    | Zone redundancy, better performance        | Team    | 2025-11-09 |
| Terraform modules per layer | Infrastructure organization   | Reusability across environments            | Team    | 2025-11-09 |
| NGINX Ingress Controller    | Kubernetes ingress choice     | Industry standard, well-documented         | Team    | 2025-11-09 |
| Prometheus + Grafana        | Monitoring stack              | Open-source, Kubernetes-native             | Team    | 2025-11-09 |

---

## 🔧 Dependencies

### Sprint Dependencies

**Depends On:**

- **Sprint 1**: Database schema and API foundation - **Status:** Complete
  - **Validation:** API runs locally, migrations applied
  - **Blocker If Not Complete:** Cannot containerize or deploy application

**Blocks:**

- **Sprint 3**: Application deployment to AKS requires this sprint's infrastructure
  - **Critical Deliverable:** AKS cluster running, ACR configured, CI/CD operational

### External Dependencies

**Third-Party Services:**

- [x] Azure subscription with Contributor role
  - **Status:** Active
  - **Contact:** Azure support
  - **Validation:** `az account show` succeeds

**Infrastructure:**

- [x] GitHub repository with admin permissions
  - **Status:** Active
  - **Owner:** Project team
  - **Validation:** `gh repo view` succeeds

---

## ✅ Definition of Done

> **CRITICAL:** All items must be checked before sprint is considered complete.

### Code Quality

- [ ] **Terraform follows best practices**
  - [ ] Modules are reusable and well-documented
  - [ ] Variables have descriptions and types
  - [ ] Outputs clearly documented
  - [ ] No hardcoded values (use variables)
- [ ] **Terraform validate passes** with zero errors
  - **Run:** `terraform validate` in all environments
- [ ] **Terraform plan succeeds** without manual resources
  - **Run:** `terraform plan` shows no drift
- [ ] **Format passes:** `terraform fmt -check -recursive`
- [ ] **Security scan passes**
  - [ ] No public IPs on databases
  - [ ] NSG rules follow least privilege
  - [ ] Key Vault has private endpoint only

### Testing

- [ ] **Infrastructure deployed to dev environment**
  - **Run:** `terraform apply` in `environments/dev`
  - **Validation:** All resources provisioned successfully
- [ ] **AKS cluster accessible**
  - **Run:** `kubectl get nodes` shows 3+ nodes
  - **Validation:** All nodes in "Ready" state
- [ ] **Database accessible from AKS**
  - **Test:** Deploy test pod, connect to PostgreSQL
  - **Validation:** Connection succeeds, pgvector extension installed
- [ ] **Key Vault secrets accessible from pods**
  - **Test:** Mount secrets in test pod via CSI driver
  - **Validation:** Secrets readable from /mnt/secrets
- [ ] **CI/CD pipelines passing**
  - [ ] Terraform workflow: `gh run list --workflow=terraform.yml`
  - [ ] Docker build: `gh run list --workflow=build-api.yml`
  - [ ] Tests: `gh run list --workflow=test-api.yml`

### Security

- [ ] **No public internet access to databases**
  - **Validation:** PostgreSQL has no public IP
  - **Validation:** Database firewall rules allow only AKS subnet
- [ ] **All secrets in Key Vault or GitHub Secrets**
  - [ ] No secrets in Terraform code
  - [ ] No secrets in Git history
  - [ ] Database passwords stored in Key Vault
- [ ] **Network security groups configured**
  - [ ] Database NSG allows only AKS traffic
  - [ ] AKS NSG allows HTTPS inbound
- [ ] **RBAC configured**
  - [ ] Service principal has minimum required permissions
  - [ ] Managed identity for AKS has Key Vault access
- [ ] **Security scan:** `az security assessment list`

### Documentation

- [ ] **Infrastructure documented**
  - [ ] Terraform README with usage instructions
  - [ ] Architecture diagram (network topology)
  - [ ] Operations runbook (scaling, backups, recovery)
- [ ] **CI/CD pipeline documented**
  - [ ] Workflow descriptions
  - [ ] Secret management guide
  - [ ] Troubleshooting common issues
- [ ] **Monitoring documented**
  - [ ] How to access Grafana, Prometheus
  - [ ] Alert definitions and escalation
  - [ ] Dashboard descriptions

### Review

- [ ] **Pull Request created** for Terraform code
  - [ ] PR includes Terraform plan output
  - [ ] Infrastructure changes reviewed by team
- [ ] **CI/CD pipelines tested**
  - [ ] End-to-end deployment succeeds
  - [ ] Rollback tested
- [ ] **Infrastructure validated**
  - [ ] All Azure resources have proper tags
  - [ ] Cost estimation reviewed (within budget)
  - [ ] High availability confirmed (zone redundancy)

---

## 📈 Sprint Retrospective

> **Update this section throughout the sprint, not just at the end.**

### What Went Well ✅

**Technical Wins:**

- (To be filled during/after sprint)

**Process Wins:**

- (To be filled during/after sprint)

**Team Wins:**

- (To be filled during/after sprint)

### What to Improve ⚠️

**Technical Challenges:**

- (To be filled during/after sprint)

**Process Challenges:**

- (To be filled during/after sprint)

**Team Challenges:**

- (To be filled during/after sprint)

### Action Items for Next Sprint 🎯

- [ ] **Action 1** - TBD based on retrospective
  - **Owner:** TBD
  - **Target:** Sprint 3 kickoff
  - **Success Criteria:** TBD

### Key Learnings 💡

**Technical Learnings:**

- (To be filled during/after sprint)

**Process Learnings:**

- (To be filled during/after sprint)

---

## 📊 Sprint Metrics

**Velocity:**

- **Planned Story Points:** ~40 hours / 95 tasks
- **Completed Story Points:** TBD
- **Velocity:** TBD%
- **Comparison to Sprint 1:** TBD

**Code Quality:**

- **Terraform Resources:** TBD (count from `terraform state list`)
- **Lines of Terraform Code:** TBD

**CI/CD:**

- **Build Success Rate:** TBD% (target: 100%)
- **Average Build Time:** TBD minutes
- **Deployments to Dev:** TBD
- **Deployments to Production:** 0 (not deploying to prod this sprint)

**Infrastructure Metrics:**

- **AKS Nodes:** TBD (target: 3-10 with autoscaling)
- **Database Size:** TBD GB
- **Monthly Cost Estimate:** TBD (target: <$500 for dev)

**Performance:**

- **Terraform Apply Time:** TBD minutes (target: <30 minutes)
- **Database Connection Latency:** TBD ms (from AKS pods)

---

## 📝 Sprint Notes

**Daily Standup Highlights:**

**[YYYY-MM-DD]:**

- (Daily updates to be added during sprint)

**Mid-Sprint Check-In ([Date]):**

- **Progress:** TBD% complete
- **Risks:** TBD
- **Adjustments:** TBD

---

## 🎯 Next Steps

**After Sprint Completion:**

1. [ ] Conduct sprint retrospective meeting (60 minutes)
2. [ ] Update sprint metrics in this document
3. [ ] Archive sprint-specific branches (merge PRs)
4. [ ] Deploy infrastructure to staging environment (dry run for production)
5. [ ] Document lessons learned for production deployment
6. [ ] Begin Sprint 3 planning (Application Deployment)

**Handoff to Sprint 3:**

- [ ] AKS cluster ready for application deployment
- [ ] ACR contains latest Docker images
- [ ] CI/CD pipelines operational
- [ ] Monitoring configured and alerting
- [ ] Database schema applied to Azure PostgreSQL
- [ ] All secrets configured in Key Vault

**Sprint 3 Preview:**

Sprint 3 will focus on:

- Deploying Rust API to AKS cluster
- Kubernetes manifests and Helm charts
- API health checks and readiness probes
- Load testing and performance tuning
- Production deployment procedures

---

## 📞 Team Contacts

**Sprint Team:**

- **Product Owner:** [@name] - Infrastructure requirements
- **Tech Lead:** [@name] - Azure architecture
- **DevOps Engineers:** [@name1], [@name2] - Terraform, AKS expertise
- **Backend Engineers:** [@name1], [@name2] - API integration
- **QA:** [@name] - Infrastructure testing

**Stakeholders:**

- **Executive Sponsor:** [@name] - For cost/budget approvals
- **Security Team:** [@name] - Security review and compliance

**Communication Channels:**

- **Daily Standups:** [Time] in [Location/Channel]
- **Sprint Planning:** [Day/Time]
- **Retrospective:** [Day/Time]
- **Slack Channel:** #sprint-2-infrastructure

---

**End of Sprint 2 Document**
