# Deployment Strategy

**Focus**: Azure Kubernetes Service → CI/CD Pipeline → Blue-Green Deployment → Database Migrations
**Priority**: High (required for production launch)
**Cross-cutting**: Infrastructure and deployment automation

---

## Epic 1: Azure Kubernetes Service Setup

### Task 1.1: Create AKS Cluster

**Type**: DevOps
**Dependencies**: Azure subscription active

**Subtasks**:

- [ ] 1.1.1: Create AKS cluster with Terraform

```hcl
# infrastructure/terraform/aks.tf
resource "azurerm_kubernetes_cluster" "ghostpirates" {
  name                = "ghostpirates-aks"
  location            = azurerm_resource_group.main.location
  resource_group_name = azurerm_resource_group.main.name
  dns_prefix          = "ghostpirates"
  kubernetes_version  = "1.28"

  default_node_pool {
    name                = "default"
    node_count          = 3
    vm_size             = "Standard_D4s_v3"
    enable_auto_scaling = true
    min_count           = 2
    max_count           = 10
    os_disk_size_gb     = 100

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
    log_analytics_workspace_id = azurerm_log_analytics_workspace.main.id
  }

  azure_policy_enabled = true

  tags = {
    Environment = "production"
    Project     = "ghostpirates"
  }
}

# Get AKS credentials
resource "null_resource" "get_credentials" {
  depends_on = [azurerm_kubernetes_cluster.ghostpirates]

  provisioner "local-exec" {
    command = "az aks get-credentials --resource-group ${azurerm_resource_group.main.name} --name ${azurerm_kubernetes_cluster.ghostpirates.name} --overwrite-existing"
  }
}

# Create namespace
resource "kubernetes_namespace" "ghostpirates" {
  metadata {
    name = "ghostpirates-prod"

    labels = {
      environment = "production"
    }
  }
}
```

- [ ] 1.1.2: Create Azure Container Registry

```hcl
# infrastructure/terraform/acr.tf
resource "azurerm_container_registry" "main" {
  name                = "ghostpiratesacr"
  resource_group_name = azurerm_resource_group.main.name
  location            = azurerm_resource_group.main.location
  sku                 = "Premium"
  admin_enabled       = false

  georeplications {
    location = "westus2"
    tags     = {}
  }

  network_rule_set {
    default_action = "Deny"

    ip_rule {
      action   = "Allow"
      ip_range = var.office_ip_range
    }
  }

  tags = {
    Environment = "production"
  }
}

# Attach ACR to AKS
resource "azurerm_role_assignment" "aks_acr_pull" {
  principal_id                     = azurerm_kubernetes_cluster.ghostpirates.kubelet_identity[0].object_id
  role_definition_name             = "AcrPull"
  scope                            = azurerm_container_registry.main.id
  skip_service_principal_aad_check = true
}
```

- [ ] 1.1.3: Apply Terraform configuration

```bash
cd infrastructure/terraform

# Initialize Terraform
terraform init

# Plan deployment
terraform plan -out=tfplan

# Apply
terraform apply tfplan

# Verify AKS cluster
kubectl get nodes
kubectl get namespaces
```

- [ ] 1.1.4: Install necessary add-ons

```bash
# Install NGINX Ingress Controller
helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
helm repo update

helm install ingress-nginx ingress-nginx/ingress-nginx \
  --namespace ingress-nginx \
  --create-namespace \
  --set controller.service.annotations."service\.beta\.kubernetes\.io/azure-load-balancer-health-probe-request-path"=/healthz

# Install cert-manager for SSL
helm repo add jetstack https://charts.jetstack.io
helm install cert-manager jetstack/cert-manager \
  --namespace cert-manager \
  --create-namespace \
  --set installCRDs=true

# Install metrics-server
kubectl apply -f https://github.com/kubernetes-sigs/metrics-server/releases/latest/download/components.yaml
```

**Acceptance Criteria**:

- [ ] AKS cluster running
- [ ] 3 nodes operational
- [ ] Auto-scaling configured
- [ ] ACR created and linked to AKS
- [ ] Ingress controller installed
- [ ] Cert-manager installed

---

## Epic 2: Helm Charts for Ghost Pirates

### Task 2.1: Create Kubernetes Manifests

**Type**: DevOps
**Dependencies**: AKS cluster ready

**Subtasks**:

- [ ] 2.1.1: Create Helm chart structure

```bash
mkdir -p infrastructure/helm/ghostpirates
cd infrastructure/helm/ghostpirates

helm create .
```

- [ ] 2.1.2: Define values.yaml

```yaml
# infrastructure/helm/ghostpirates/values.yaml
replicaCount: 3

image:
  repository: ghostpiratesacr.azurecr.io/ghostpirates-api
  pullPolicy: IfNotPresent
  tag: "latest"

service:
  type: ClusterIP
  port: 4000

ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
  hosts:
    - host: api.ghostpirates.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: ghostpirates-tls
      hosts:
        - api.ghostpirates.com

resources:
  limits:
    cpu: 2000m
    memory: 4Gi
  requests:
    cpu: 1000m
    memory: 2Gi

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 10
  targetCPUUtilizationPercentage: 70
  targetMemoryUtilizationPercentage: 80

postgresql:
  enabled: false  # Using Azure PostgreSQL
  host: ghostpirates-db.postgres.database.azure.com
  port: 5432
  database: ghostpirates

redis:
  enabled: false  # Using Azure Redis Cache
  host: ghostpirates-redis.redis.cache.windows.net
  port: 6380
  ssl: true

secrets:
  azureKeyVault:
    enabled: true
    vaultName: ghostpirates-vault
    usePodIdentity: true

env:
  - name: RUST_LOG
    value: "info"
  - name: ENVIRONMENT
    value: "production"

livenessProbe:
  httpGet:
    path: /health
    port: 4000
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health
    port: 4000
  initialDelaySeconds: 5
  periodSeconds: 5
```

- [ ] 2.1.3: Create deployment template

```yaml
# infrastructure/helm/ghostpirates/templates/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ include "ghostpirates.fullname" . }}
  labels:
    {{- include "ghostpirates.labels" . | nindent 4 }}
spec:
  {{- if not .Values.autoscaling.enabled }}
  replicas: {{ .Values.replicaCount }}
  {{- end }}
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      {{- include "ghostpirates.selectorLabels" . | nindent 6 }}
  template:
    metadata:
      annotations:
        checksum/config: {{ include (print $.Template.BasePath "/configmap.yaml") . | sha256sum }}
      labels:
        {{- include "ghostpirates.selectorLabels" . | nindent 8 }}
        version: {{ .Values.image.tag }}
    spec:
      serviceAccountName: {{ include "ghostpirates.serviceAccountName" . }}
      containers:
      - name: {{ .Chart.Name }}
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        imagePullPolicy: {{ .Values.image.pullPolicy }}
        ports:
        - name: http
          containerPort: 4000
          protocol: TCP
        envFrom:
        - configMapRef:
            name: {{ include "ghostpirates.fullname" . }}
        - secretRef:
            name: {{ include "ghostpirates.fullname" . }}-secrets
        livenessProbe:
          {{- toYaml .Values.livenessProbe | nindent 10 }}
        readinessProbe:
          {{- toYaml .Values.readinessProbe | nindent 10 }}
        resources:
          {{- toYaml .Values.resources | nindent 10 }}
```

- [ ] 2.1.4: Create service template

```yaml
# infrastructure/helm/ghostpirates/templates/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: {{ include "ghostpirates.fullname" . }}
  labels:
    {{- include "ghostpirates.labels" . | nindent 4 }}
spec:
  type: {{ .Values.service.type }}
  ports:
    - port: {{ .Values.service.port }}
      targetPort: http
      protocol: TCP
      name: http
  selector:
    {{- include "ghostpirates.selectorLabels" . | nindent 4 }}
```

- [ ] 2.1.5: Create HPA template

```yaml
# infrastructure/helm/ghostpirates/templates/hpa.yaml
{{- if .Values.autoscaling.enabled }}
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: {{ include "ghostpirates.fullname" . }}
  labels:
    {{- include "ghostpirates.labels" . | nindent 4 }}
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: {{ include "ghostpirates.fullname" . }}
  minReplicas: {{ .Values.autoscaling.minReplicas }}
  maxReplicas: {{ .Values.autoscaling.maxReplicas }}
  metrics:
  {{- if .Values.autoscaling.targetCPUUtilizationPercentage }}
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: {{ .Values.autoscaling.targetCPUUtilizationPercentage }}
  {{- end }}
  {{- if .Values.autoscaling.targetMemoryUtilizationPercentage }}
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: {{ .Values.autoscaling.targetMemoryUtilizationPercentage }}
  {{- end }}
{{- end }}
```

- [ ] 2.1.6: Install Helm chart

```bash
# Validate chart
helm lint infrastructure/helm/ghostpirates

# Dry run
helm install ghostpirates infrastructure/helm/ghostpirates \
  --namespace ghostpirates-prod \
  --dry-run --debug

# Install
helm install ghostpirates infrastructure/helm/ghostpirates \
  --namespace ghostpirates-prod \
  --create-namespace

# Verify deployment
kubectl get pods -n ghostpirates-prod
kubectl get svc -n ghostpirates-prod
kubectl get ingress -n ghostpirates-prod
```

**Acceptance Criteria**:

- [ ] Helm chart installs successfully
- [ ] All pods running
- [ ] Service accessible within cluster
- [ ] Ingress configured
- [ ] Auto-scaling working
- [ ] Health checks passing

---

## Epic 3: Blue-Green Deployment Strategy

### Task 3.1: Implement Blue-Green Deployments

**Type**: DevOps
**Dependencies**: Helm charts working

**Subtasks**:

- [ ] 3.1.1: Create blue-green deployment script

```bash
#!/bin/bash
# scripts/blue-green-deploy.sh

set -e

NAMESPACE="ghostpirates-prod"
CURRENT_ENV=$(kubectl get svc ghostpirates -n $NAMESPACE -o jsonpath='{.spec.selector.environment}')
NEW_ENV=$([ "$CURRENT_ENV" = "blue" ] && echo "green" || echo "blue")

echo "Current environment: $CURRENT_ENV"
echo "Deploying to: $NEW_ENV"

# Deploy new version to inactive environment
helm upgrade --install ghostpirates-$NEW_ENV infrastructure/helm/ghostpirates \
  --namespace $NAMESPACE \
  --set environment=$NEW_ENV \
  --set image.tag=$1 \
  --wait

# Wait for new deployment to be healthy
echo "Waiting for $NEW_ENV environment to be ready..."
kubectl wait --for=condition=ready pod \
  -l app=ghostpirates,environment=$NEW_ENV \
  -n $NAMESPACE \
  --timeout=300s

# Run smoke tests
echo "Running smoke tests on $NEW_ENV environment..."
./scripts/smoke-test.sh $NEW_ENV

# Switch traffic
echo "Switching traffic to $NEW_ENV environment..."
kubectl patch svc ghostpirates -n $NAMESPACE \
  -p "{\"spec\":{\"selector\":{\"environment\":\"$NEW_ENV\"}}}"

echo "Traffic switched to $NEW_ENV"
echo "Old $CURRENT_ENV environment still running for rollback"
echo "To rollback: kubectl patch svc ghostpirates -n $NAMESPACE -p '{\"spec\":{\"selector\":{\"environment\":\"$CURRENT_ENV\"}}}'"
```

- [ ] 3.1.2: Create smoke test script

```bash
#!/bin/bash
# scripts/smoke-test.sh

ENV=$1
SERVICE_URL="http://ghostpirates-$ENV.ghostpirates-prod.svc.cluster.local:4000"

echo "Running smoke tests against $SERVICE_URL"

# Health check
if ! curl -f $SERVICE_URL/health; then
    echo "Health check failed"
    exit 1
fi

# API check
if ! curl -f $SERVICE_URL/api/health; then
    echo "API health check failed"
    exit 1
fi

echo "Smoke tests passed"
```

- [ ] 3.1.3: Create rollback script

```bash
#!/bin/bash
# scripts/rollback.sh

set -e

NAMESPACE="ghostpirates-prod"
CURRENT_ENV=$(kubectl get svc ghostpirates -n $NAMESPACE -o jsonpath='{.spec.selector.environment}')
PREVIOUS_ENV=$([ "$CURRENT_ENV" = "blue" ] && echo "green" || echo "blue")

echo "Current environment: $CURRENT_ENV"
echo "Rolling back to: $PREVIOUS_ENV"

# Switch traffic back
kubectl patch svc ghostpirates -n $NAMESPACE \
  -p "{\"spec\":{\"selector\":{\"environment\":\"$PREVIOUS_ENV\"}}}"

echo "Rollback complete. Traffic switched to $PREVIOUS_ENV"
```

- [ ] 3.1.4: Update CI/CD pipeline for blue-green

```yaml
# .github/workflows/deploy-production.yml
name: Deploy to Production (Blue-Green)

on:
  push:
    tags:
      - 'v*'

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to ACR
        uses: azure/docker-login@v1
        with:
          login-server: ghostpiratesacr.azurecr.io
          username: ${{ secrets.ACR_USERNAME }}
          password: ${{ secrets.ACR_PASSWORD }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: ghostpiratesacr.azurecr.io/ghostpirates-api:${{ github.ref_name }}

      - name: Set up kubectl
        uses: azure/setup-kubectl@v3

      - name: Get AKS credentials
        run: |
          az aks get-credentials \
            --resource-group ghostpirates-prod \
            --name ghostpirates-aks

      - name: Blue-Green Deploy
        run: |
          chmod +x scripts/blue-green-deploy.sh
          ./scripts/blue-green-deploy.sh ${{ github.ref_name }}

      - name: Cleanup old environment (after 1 hour)
        run: |
          sleep 3600
          OLD_ENV=$(kubectl get svc ghostpirates -n ghostpirates-prod -o jsonpath='{.spec.selector.environment}')
          NEW_ENV=$([ "$OLD_ENV" = "blue" ] && echo "green" || echo "blue")
          helm uninstall ghostpirates-$OLD_ENV -n ghostpirates-prod || true
```

**Acceptance Criteria**:

- [ ] Blue-green deployment working
- [ ] Zero-downtime deployments
- [ ] Smoke tests pass before traffic switch
- [ ] Rollback in < 30 seconds
- [ ] Old environment kept for 1 hour
- [ ] Automated via CI/CD

---

## Epic 4: Database Migration Strategy

### Task 4.1: Safe Database Migrations

**Type**: DevOps/Backend
**Dependencies**: Database access

**Subtasks**:

- [ ] 4.1.1: Create migration job template

```yaml
# infrastructure/helm/ghostpirates/templates/migration-job.yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: {{ include "ghostpirates.fullname" . }}-migration-{{ .Values.image.tag }}
  labels:
    {{- include "ghostpirates.labels" . | nindent 4 }}
  annotations:
    "helm.sh/hook": pre-upgrade,pre-install
    "helm.sh/hook-weight": "1"
    "helm.sh/hook-delete-policy": before-hook-creation
spec:
  backoffLimit: 3
  template:
    spec:
      restartPolicy: Never
      containers:
      - name: migration
        image: "{{ .Values.image.repository }}:{{ .Values.image.tag }}"
        command: ["/usr/local/bin/ghostpirates-api"]
        args: ["migrate"]
        envFrom:
        - secretRef:
            name: {{ include "ghostpirates.fullname" . }}-secrets
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: database-credentials
              key: connection-string
```

- [ ] 4.1.2: Create migration safety checks

```rust
// apps/api/src/db/migrations.rs
use sqlx::PgPool;

pub struct MigrationRunner {
    pool: PgPool,
}

impl MigrationRunner {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn run_migrations_safely(&self) -> Result<(), MigrationError> {
        // Create backup before migration
        self.create_backup().await?;

        // Check migration compatibility
        self.check_compatibility().await?;

        // Run migrations in transaction
        let mut tx = self.pool.begin().await?;

        match sqlx::migrate!("./migrations").run(&mut tx).await {
            Ok(_) => {
                tx.commit().await?;
                tracing::info!("Migrations completed successfully");
                Ok(())
            }
            Err(e) => {
                tx.rollback().await?;
                tracing::error!("Migration failed: {}", e);
                Err(MigrationError::MigrationFailed(e.to_string()))
            }
        }
    }

    async fn create_backup(&self) -> Result<(), MigrationError> {
        // Trigger Azure PostgreSQL backup
        tracing::info!("Creating database backup before migration");
        // Implementation depends on Azure CLI or SDK
        Ok(())
    }

    async fn check_compatibility(&self) -> Result<(), MigrationError> {
        // Check if any migrations would break existing queries
        // Could check for:
        // - Column drops
        // - Table renames
        // - Data type changes
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
}
```

- [ ] 4.1.3: Create rollback procedure

```bash
#!/bin/bash
# scripts/rollback-migration.sh

set -e

echo "Rolling back last migration..."

# Get last successful backup
BACKUP_NAME=$(az postgres flexible-server backup list \
  --resource-group ghostpirates-prod \
  --server-name ghostpirates-db \
  --query "[0].name" -o tsv)

echo "Latest backup: $BACKUP_NAME"

# Restore from backup
az postgres flexible-server restore \
  --resource-group ghostpirates-prod \
  --name ghostpirates-db-restored \
  --source-server ghostpirates-db \
  --restore-time $BACKUP_NAME

echo "Restored database to $BACKUP_NAME"
echo "Update connection string to point to restored database"
```

**Acceptance Criteria**:

- [ ] Migrations run before deployment
- [ ] Backup created before migration
- [ ] Migrations run in transaction
- [ ] Failed migrations rollback automatically
- [ ] Rollback procedure documented
- [ ] Zero downtime for additive changes

---

## Epic 5: Rollback Procedures

### Task 5.1: Comprehensive Rollback Strategy

**Type**: DevOps
**Dependencies**: All deployment components ready

**Subtasks**:

- [ ] 5.1.1: Create rollback runbook

```markdown
# Rollback Runbook

## Scenario 1: Bad Deployment (Application Issues)

### Detection
- Health checks failing
- Error rate spike in monitoring
- User reports of issues

### Rollback Steps
1. Identify current and previous environments:
   ```bash
   kubectl get svc ghostpirates -n ghostpirates-prod -o jsonpath='{.spec.selector}'
   ```

2. Switch traffic back to previous environment:
   ```bash
   ./scripts/rollback.sh
   ```

3. Verify rollback:
   ```bash
   curl https://api.ghostpirates.com/health
   ```

4. Check monitoring dashboards for error rate decrease

### Time to rollback: < 1 minute

## Scenario 2: Bad Database Migration

### Detection
- Migration job failed
- Application can't connect to database
- Data inconsistencies

### Rollback Steps
1. Check migration status:
   ```bash
   kubectl logs -n ghostpirates-prod -l job-name=migration
   ```

2. If migration partially completed, restore from backup:
   ```bash
   ./scripts/rollback-migration.sh
   ```

3. Redeploy previous application version:
   ```bash
   ./scripts/blue-green-deploy.sh v1.2.3
   ```

### Time to rollback: 5-10 minutes

## Scenario 3: Infrastructure Issues

### Detection
- Cluster unresponsive
- Node failures
- Network issues

### Rollback Steps
1. Check cluster health:
   ```bash
   kubectl get nodes
   kubectl get pods --all-namespaces
   ```

2. Scale up additional nodes if needed:
   ```bash
   az aks scale --node-count 5 \
     --resource-group ghostpirates-prod \
     --name ghostpirates-aks
   ```

3. Restart affected pods:
   ```bash
   kubectl rollout restart deployment/ghostpirates -n ghostpirates-prod
   ```

### Time to recover: 5-15 minutes
```

- [ ] 5.1.2: Create automated rollback triggers

```yaml
# infrastructure/monitoring/rollback-alerts.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: rollback-alerts
data:
  alerts.yml: |
    groups:
    - name: AutoRollback
      interval: 30s
      rules:
      - alert: HighErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
        for: 2m
        annotations:
          summary: "High error rate detected"
          description: "Error rate is {{ $value }} (threshold: 0.05)"
        labels:
          severity: critical
          auto_rollback: "true"

      - alert: HealthCheckFailure
        expr: up{job="ghostpirates"} == 0
        for: 1m
        annotations:
          summary: "Health check failing"
        labels:
          severity: critical
          auto_rollback: "true"
```

- [ ] 5.1.3: Implement automated rollback webhook

```rust
// apps/rollback-webhook/src/main.rs
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/webhook/alert", post(handle_alert));

    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_alert(Json(alert): Json<Alert>) -> StatusCode {
    if alert.labels.get("auto_rollback") == Some(&"true".to_string()) {
        tracing::warn!("Auto-rollback triggered: {}", alert.annotations.summary);

        // Execute rollback
        match execute_rollback().await {
            Ok(_) => {
                tracing::info!("Rollback completed successfully");
                StatusCode::OK
            }
            Err(e) => {
                tracing::error!("Rollback failed: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    } else {
        StatusCode::OK
    }
}

async fn execute_rollback() -> Result<(), Box<dyn std::error::Error>> {
    let output = tokio::process::Command::new("./scripts/rollback.sh")
        .output()
        .await?;

    if !output.status.success() {
        return Err("Rollback script failed".into());
    }

    Ok(())
}
```

**Acceptance Criteria**:

- [ ] Rollback procedures documented
- [ ] Manual rollback tested
- [ ] Automated rollback working
- [ ] Rollback time < 5 minutes
- [ ] Rollback monitoring in place
- [ ] Incident playbooks ready

---

## Success Criteria - Deployment Complete

- [ ] AKS cluster operational
- [ ] Helm charts deploying successfully
- [ ] Blue-green deployments working
- [ ] Zero-downtime deployments achieved
- [ ] Database migrations safe and tested
- [ ] Rollback procedures validated
- [ ] Auto-scaling functional
- [ ] SSL certificates valid
- [ ] Monitoring integrated
- [ ] Production-ready

---

## Next Steps

Proceed to [15-testing-strategy.md](./15-testing-strategy.md) for comprehensive testing strategy.

---

**Deployment Strategy: Production deployment infrastructure complete**
