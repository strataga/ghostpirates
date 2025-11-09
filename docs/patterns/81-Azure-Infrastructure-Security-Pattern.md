# 81. Azure Infrastructure Security Pattern

**Category**: Infrastructure as Code Security
**Complexity**: Intermediate
**Last Updated**: October 30, 2025

## Intent

Implement secure Azure infrastructure deployments using Bicep/ARM templates following Azure security best practices. Prevent credential exposure, enforce least-privilege access, and establish defense-in-depth security controls.

## Context

**Problem**: Infrastructure as Code (IaC) can inadvertently expose secrets, create overly permissive access, and bypass security controls if not properly designed.

**When to Use**:
- Deploying Azure resources via Bicep/ARM templates
- Automating infrastructure provisioning for multi-tenant SaaS
- Implementing SCADA/IoT Hub integrations with external systems
- Storing sensitive configuration (connection strings, API keys, certificates)

**WellOS Use Cases**:
- IoT Hub deployment for SCADA integration (`infrastructure/azure/iot-hub/`)
- Tenant database provisioning (Azure PostgreSQL Flexible Server)
- Event Grid webhook authentication
- Azure Blob Storage for white-label PDF reports

## Solution

### Core Principles

1. **Secrets Never in Code**: Use `@secure()` parameters, Azure Key Vault references, or Managed Identity
2. **Least Privilege**: RBAC with minimal scopes, no owner/contributor unless required
3. **Defense in Depth**: Network isolation, encryption, monitoring, audit logs
4. **Immutable Infrastructure**: Version-controlled Bicep templates, repeatable deployments

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│ Bicep Template (Infrastructure as Code)                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Parameters (Secure)           Modules (Reusable)          │
│  ├─ @secure() apiKey           ├─ iot-hub.bicep            │
│  ├─ @secure() webhookUrl       ├─ event-grid.bicep         │
│  └─ Key Vault reference        └─ storage-account.bicep    │
│                                                             │
│  Resources (Secure by Default)                             │
│  ├─ System-Assigned Managed Identity                       │
│  ├─ Private Endpoints (no public access)                   │
│  ├─ TLS 1.2+ enforcement                                   │
│  ├─ Diagnostic logging → Log Analytics                     │
│  └─ Azure Policy compliance checks                         │
│                                                             │
│  Outputs (NO SECRETS)                                      │
│  ├─ Resource IDs (for RBAC)                                │
│  ├─ Principal IDs (for role assignments)                   │
│  ├─ Hostnames (for DNS)                                    │
│  └─ ❌ NO listKeys() outputs                               │
│                                                             │
└─────────────────────────────────────────────────────────────┘
         ↓                           ↓
    Key Vault                   Managed Identity
    (Runtime Secrets)           (Passwordless Auth)
```

## Implementation

### 1. Secure Parameters

**❌ Bad: Plaintext secret parameter**
```bicep
param apiKey string  // Logged in ARM deployment history
param connectionString string  // Visible in portal
```

**✅ Good: Secure parameter**
```bicep
@secure()
@description('API key for webhook authentication (not logged)')
param apiKey string

@secure()
@description('Database connection string (use Key Vault reference)')
param connectionString string
```

**✅ Best: Key Vault reference**
```bicep
// main.bicep
param apiKeySecretUri string  // Reference to Key Vault secret

resource keyVault 'Microsoft.KeyVault/vaults@2023-07-01' existing = {
  name: keyVaultName
  scope: resourceGroup()
}

// Pass to module without exposing value
module deployment 'module.bicep' = {
  params: {
    apiKey: keyVault.getSecret('api-key')  // Retrieved at deployment time
  }
}
```

### 2. Outputs Without Secrets

**❌ Bad: Connection string in output**
```bicep
output connectionString string = 'HostName=${iotHub.properties.hostName};SharedAccessKey=${iotHub.listKeys().value[0].primaryKey}'
// ⚠️ Stored in ARM deployment history (accessible via API/Portal)
```

**✅ Good: Safe outputs only**
```bicep
// Production-safe outputs
@description('IoT Hub hostname for managed identity authentication')
output hostname string = iotHub.properties.hostName

@description('Resource ID for RBAC assignments')
output resourceId string = iotHub.id

@description('Managed identity principal ID for role assignments')
output principalId string = iotHub.identity.principalId

// Development-only (commented out for production)
// @description('Connection string for LOCAL development ONLY (CONTAINS SECRETS)')
// output connectionString string = iotHub.listKeys().value[0].connectionString
```

**🔧 How to Use in Production**:
```bash
# Deploy infrastructure
az deployment sub create \
  --template-file main.bicep \
  --parameters environment=production

# Get outputs (no secrets)
HOSTNAME=$(az deployment sub show --name main --query properties.outputs.hostname.value -o tsv)
PRINCIPAL_ID=$(az deployment sub show --name main --query properties.outputs.principalId.value -o tsv)

# Assign RBAC role to managed identity (no connection string needed)
az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "IoT Hub Data Contributor" \
  --scope /subscriptions/.../resourceGroups/.../providers/Microsoft.Devices/IotHubs/...
```

### 3. Managed Identity (Passwordless Authentication)

**❌ Bad: Connection string authentication**
```typescript
// Requires storing secrets in environment variables
const client = IotHubClient.fromConnectionString(
  process.env.IOT_HUB_CONNECTION_STRING  // Secret management burden
);
```

**✅ Good: Managed Identity authentication**
```bicep
// Bicep: Enable system-assigned identity
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  identity: {
    type: 'SystemAssigned'  // Azure manages credentials automatically
  }
}

// Assign RBAC role
resource roleAssignment 'Microsoft.Authorization/roleAssignments@2022-04-01' = {
  scope: iotHub
  properties: {
    roleDefinitionId: subscriptionResourceId('Microsoft.Authorization/roleDefinitions', '4fc6c259-987e-4a07-842e-c321cc9d413f')  // IoT Hub Data Contributor
    principalId: apiManagedIdentity.properties.principalId
  }
}
```

```typescript
// Node.js: Use DefaultAzureCredential (auto-detects managed identity)
import { DefaultAzureCredential } from '@azure/identity';
import { IotHubClient } from '@azure/iot-hub';

const credential = new DefaultAzureCredential();
const client = new IotHubClient(
  process.env.IOT_HUB_HOSTNAME,  // Just hostname, no secrets
  credential
);
```

### 4. Network Security

**✅ Private Endpoints & Network Isolation**
```bicep
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  properties: {
    publicNetworkAccess: 'Disabled'  // Force private access only
    networkRuleSets: {
      defaultAction: 'Deny'
      ipRules: []  // No public IPs allowed
    }
    privateEndpointConnections: [
      // Private endpoint from VNet
    ]
  }
}
```

### 5. Encryption & TLS

**✅ Enforce minimum TLS version**
```bicep
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  properties: {
    minTlsVersion: '1.2'  // TLS 1.0/1.1 blocked
  }
}

resource storageAccount 'Microsoft.Storage/storageAccounts@2023-01-01' = {
  properties: {
    minimumTlsVersion: 'TLS1_2'
    supportsHttpsTrafficOnly: true  // HTTP blocked
    encryption: {
      keySource: 'Microsoft.Storage'  // Encryption at rest
      services: {
        blob: { enabled: true }
        file: { enabled: true }
      }
    }
  }
}
```

### 6. Audit Logging & Monitoring

**✅ Enable diagnostic logging**
```bicep
resource diagnosticSettings 'Microsoft.Insights/diagnosticSettings@2021-05-01-preview' = {
  scope: iotHub
  properties: {
    workspaceId: logAnalyticsWorkspace.id
    logs: [
      {
        category: 'Connections'
        enabled: true
        retentionPolicy: { enabled: true, days: 90 }
      }
      {
        category: 'DeviceTelemetry'
        enabled: true
      }
    ]
    metrics: [
      {
        category: 'AllMetrics'
        enabled: true
      }
    ]
  }
}
```

### 7. Resource Tagging & Governance

**✅ Required tags for compliance**
```bicep
@description('Resource tags for governance')
param tags object = {
  Project: 'WellOS'
  Environment: environment
  ManagedBy: 'Bicep'
  CostCenter: 'Engineering'
  DataClassification: 'Confidential'  // For compliance
  BackupPolicy: 'Daily'
}

resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  tags: union(tags, {
    Tenant: tenantSubdomain  // Tenant isolation tracking
  })
}
```

## WellOS Implementation

### IoT Hub Security (SCADA Integration)

**File**: `infrastructure/azure/iot-hub/modules/iot-hub.bicep`

```bicep
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  identity: {
    type: 'SystemAssigned'  // ✅ Managed identity for API authentication
  }
  properties: {
    minTlsVersion: '1.2'  // ✅ TLS enforcement
    publicNetworkAccess: 'Enabled'  // ⚠️ Required for field SCADA devices
    networkRuleSets: {
      defaultAction: 'Deny'
      ipRules: [
        // Whitelist known SCADA IP ranges (Permian Basin well sites)
        { filterName: 'PermianBasinWells', action: 'Allow', ipMask: '203.0.113.0/24' }
      ]
    }
  }
}

// ✅ Outputs without secrets
output hostname string = iotHub.properties.hostName
output principalId string = iotHub.identity.principalId
output resourceId string = iotHub.id
```

### Event Grid Webhook Security

**File**: `infrastructure/azure/iot-hub/modules/event-subscription.bicep`

```bicep
@secure()  // ✅ Prevents logging in ARM deployment history
@description('Webhook endpoint URL (may contain authentication tokens)')
param webhookUrl string

resource eventSubscription 'Microsoft.EventGrid/eventSubscriptions@2023-12-15-preview' = {
  properties: {
    destination: {
      endpointType: 'WebHook'
      properties: {
        endpointUrl: webhookUrl  // Delivered via HTTPS, not logged
        maxEventsPerBatch: 10
      }
    }
    eventDeliverySchema: 'EventGridSchema'
    retryPolicy: {
      maxDeliveryAttempts: 30
      eventTimeToLiveInMinutes: 1440
    }
  }
}
```

### Deployment Script (Production)

**File**: `infrastructure/azure/iot-hub/deploy-production.sh`

```bash
#!/bin/bash
set -euo pipefail

ENVIRONMENT="production"
TENANT="acmeoil"
LOCATION="eastus"
SUBSCRIPTION_ID="..."

# ✅ API webhook URL from Key Vault (not hardcoded)
WEBHOOK_URL=$(az keyvault secret show \
  --vault-name "kv-wellos-prod" \
  --name "api-webhook-url-${TENANT}" \
  --query value -o tsv)

# Deploy infrastructure (subscription scope)
az deployment sub create \
  --name "iot-hub-${TENANT}-${ENVIRONMENT}-$(date +%Y%m%d-%H%M%S)" \
  --location "$LOCATION" \
  --template-file main.bicep \
  --parameters \
    environment="$ENVIRONMENT" \
    tenantSubdomain="$TENANT" \
    location="$LOCATION" \
    apiWebhookUrl="$WEBHOOK_URL" \
    iotHubSku="S1" \
    iotHubCapacity=2

# ✅ Retrieve outputs (no secrets)
HOSTNAME=$(az deployment sub show \
  --name "iot-hub-${TENANT}-${ENVIRONMENT}-*" \
  --query 'properties.outputs.iotHubHostname.value' -o tsv)

PRINCIPAL_ID=$(az deployment sub show \
  --name "iot-hub-${TENANT}-${ENVIRONMENT}-*" \
  --query 'properties.outputs.iotHubPrincipalId.value' -o tsv)

# ✅ Assign RBAC to API managed identity
az role assignment create \
  --assignee "$PRINCIPAL_ID" \
  --role "IoT Hub Data Contributor" \
  --scope "/subscriptions/${SUBSCRIPTION_ID}/resourceGroups/rg-wellos-${TENANT}-${ENVIRONMENT}"

echo "✅ IoT Hub deployed: $HOSTNAME"
echo "✅ Managed identity configured: $PRINCIPAL_ID"
```

## Security Checklist

**Bicep Templates**:
- [ ] All sensitive parameters marked `@secure()`
- [ ] No `listKeys()` or secrets in outputs
- [ ] System-assigned managed identity enabled
- [ ] Minimum TLS version set to 1.2+
- [ ] Public network access restricted (Deny or IP whitelist)
- [ ] Diagnostic logging enabled → Log Analytics
- [ ] Required tags applied (Project, Environment, Tenant, DataClassification)
- [ ] Private endpoints configured (if applicable)

**Deployment Process**:
- [ ] Secrets retrieved from Key Vault (not hardcoded)
- [ ] RBAC roles assigned with least privilege
- [ ] Deployment names include timestamp (auditability)
- [ ] Output values validated (no accidental secret exposure)
- [ ] Infrastructure changes peer-reviewed
- [ ] Terraform/Bicep state stored securely (Azure Storage with encryption)

**Runtime Security**:
- [ ] Applications use `DefaultAzureCredential` (managed identity)
- [ ] No connection strings in environment variables
- [ ] Network traffic encrypted (TLS 1.2+ only)
- [ ] Security alerts configured (Azure Defender/Microsoft Sentinel)
- [ ] Regular security scans (Defender for Cloud)

## Common Pitfalls

### Pitfall 1: Connection Strings in Outputs

**Problem**: Bicep outputs are stored in ARM deployment history, accessible via Azure Portal/API.

```bicep
// ❌ BAD: Secret exposed in deployment history
output connectionString string = iotHub.listKeys().value[0].connectionString
```

**Solution**: Use managed identity or store in Key Vault.

```bicep
// ✅ GOOD: No secrets in outputs
output hostname string = iotHub.properties.hostName
output principalId string = iotHub.identity.principalId
```

### Pitfall 2: Missing `@secure()` Decorator

**Problem**: Parameter values logged in ARM deployment operations log.

```bicep
// ❌ BAD: Webhook URL (with auth token) logged
param webhookUrl string
```

**Solution**: Mark as `@secure()`.

```bicep
// ✅ GOOD: Parameter not logged
@secure()
param webhookUrl string
```

### Pitfall 3: Overly Permissive RBAC

**Problem**: Assigning `Owner` or `Contributor` when narrower roles exist.

```bash
# ❌ BAD: Full subscription access
az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "Contributor" \
  --scope "/subscriptions/${SUBSCRIPTION_ID}"
```

**Solution**: Use specific roles at narrowest scope.

```bash
# ✅ GOOD: IoT Hub Data Contributor on specific resource
az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "IoT Hub Data Contributor" \
  --scope "/subscriptions/.../resourceGroups/.../providers/Microsoft.Devices/IotHubs/..."
```

### Pitfall 4: Public Network Access Enabled

**Problem**: Resources exposed to internet without IP restrictions.

```bicep
// ❌ BAD: Anyone can connect
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  properties: {
    publicNetworkAccess: 'Enabled'  // No restrictions
  }
}
```

**Solution**: Disable public access or whitelist known IPs.

```bicep
// ✅ GOOD: IP whitelist for SCADA devices
resource iotHub 'Microsoft.Devices/IotHubs@2023-06-30' = {
  properties: {
    publicNetworkAccess: 'Enabled'
    networkRuleSets: {
      defaultAction: 'Deny'
      ipRules: [
        { filterName: 'SCADA-Site-1', action: 'Allow', ipMask: '203.0.113.10/32' }
      ]
    }
  }
}
```

## Related Patterns

- **[69. Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)**: Secure multi-tenant database isolation
- **[72. Database-Agnostic Multi-Tenant Pattern](./72-Database-Agnostic-Multi-Tenant-Pattern.md)**: Secure connections to client-hosted databases
- **[80. White-Label PDF Report Generation Pattern](./80-White-Label-PDF-Report-Generation-Pattern.md)**: Secure Azure Blob Storage for PDF files

## References

- [Azure Bicep Best Practices](https://learn.microsoft.com/en-us/azure/azure-resource-manager/bicep/best-practices)
- [Azure Security Baseline](https://learn.microsoft.com/en-us/security/benchmark/azure/baselines/iot-hub-security-baseline)
- [Managed Identity Documentation](https://learn.microsoft.com/en-us/entra/identity/managed-identities-azure-resources/overview)
- [Azure Key Vault Best Practices](https://learn.microsoft.com/en-us/azure/key-vault/general/best-practices)
- [OWASP Infrastructure as Code Security](https://owasp.org/www-project-devsecops-guideline/latest/01a-Infrastructure-Security)

---

**Tags**: #azure #bicep #iac #security #managed-identity #secrets-management #rbac #network-security #encryption
