# Database Provisioning Automation Pattern

## Context

Multi-tenant applications with database-per-tenant architecture require creating and managing multiple databases during development, testing, and production deployment. Manual database provisioning is error-prone, time-consuming, and doesn't scale beyond a few tenants.

In WellOS, we manage:
- **1 Master Database**: Tenant registry, admin users, billing
- **N Tenant Databases**: One per operator (wells, production data, users)

## Problem

How do you automate the complete database provisioning lifecycle for a multi-tenant application while maintaining:

- **Idempotency**: Scripts can be run multiple times safely
- **Self-Documentation**: Scripts query the database to discover what needs to be created
- **Developer Experience**: Simple commands for common workflows
- **Error Handling**: Clear feedback on what succeeded and what failed
- **Clean Slate Testing**: Easy reset to initial state

**Challenges**:
- Tenant databases are registered in master database (dynamic list)
- Different environments (local dev, staging, production) have different database configurations
- PostgreSQL connections must be terminated before dropping databases
- Migration and seeding must happen in correct order
- Need both "create all" and "drop all" operations

## Solution

Create **Infrastructure as Code** automation scripts that:
1. Dynamically discover which databases to create by querying master database
2. Provide both creation and destruction operations
3. Handle errors gracefully with clear user feedback
4. Support optional demo data seeding

### Structure

```
scripts/
├── create-dbs.sh   # Complete database provisioning
└── drop-dbs.sh     # Safe database cleanup
```

### Implementation

#### 1. **Database Creation Script**

```bash
#!/bin/bash
# scripts/create-dbs.sh

##
## Create WellOS Databases
##
## Creates all WellOS databases (master + tenant databases)
## Runs migrations and optionally seeds data
##
## Usage:
##   ./scripts/create-dbs.sh              # Create and migrate only
##   ./scripts/create-dbs.sh --seed       # Create, migrate, and seed demo data
##

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
SEED_DATA=false
if [[ "$1" == "--seed" ]]; then
  SEED_DATA=true
fi

echo "🚀 Creating WellOS databases..."
echo ""

# Step 1: Create master database
echo -e "${BLUE}📦 Step 1: Creating master database${NC}"
echo -n "   Creating wellos_master... "
if psql -h localhost -d postgres -c "CREATE DATABASE wellos_master OWNER wellos;" 2>/dev/null; then
  echo -e "${GREEN}✓${NC}"
else
  echo -e "${RED}✗${NC}"
  echo -e "${RED}Error: Failed to create wellos_master${NC}"
  echo "Tip: Database may already exist. Run './scripts/drop-dbs.sh' first."
  exit 1
fi

# Step 2: Run master migrations
echo -e "${BLUE}📦 Step 2: Running master database migrations${NC}"
cd /path/to/api
pnpm exec tsx src/infrastructure/database/scripts/migrate-master.ts

# Step 3: Seed master database
echo -e "${BLUE}📦 Step 3: Seeding master database (creating tenants)${NC}"
pnpm exec tsx src/infrastructure/database/seeds/master.seed.ts

# Step 4: Dynamic tenant database discovery
echo -e "${BLUE}📦 Step 4: Creating tenant databases${NC}"

# Query master database for tenant database names
TENANT_DBS=$(psql -h localhost -d wellos_master -t -c "SELECT database_name FROM tenants ORDER BY subdomain;")

for db in $TENANT_DBS; do
  db=$(echo $db | xargs) # trim whitespace
  echo -n "   Creating $db... "
  if psql -h localhost -d postgres -c "CREATE DATABASE $db OWNER wellos;" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
  else
    echo -e "${RED}✗${NC}"
    echo -e "${RED}Error: Failed to create $db${NC}"
    exit 1
  fi
done

# Step 5: Run tenant migrations
echo -e "${BLUE}📦 Step 5: Running tenant database migrations${NC}"
pnpm --filter=api db:migrate:tenant

# Step 6: Seed demo tenant (optional)
if [ "$SEED_DATA" = true ]; then
  echo -e "${BLUE}📦 Step 6: Seeding demo tenant database${NC}"
  pnpm exec tsx src/infrastructure/database/seeds/tenant.seed.ts demo
fi

# Summary
echo ""
echo -e "${GREEN}✅ WellOS databases created successfully!${NC}"
echo ""
echo "📊 Database Summary:"
psql -h localhost -d wellos_master -c "
  SELECT subdomain, name, database_name, subscription_tier, status
  FROM tenants ORDER BY subdomain;
"
```

**Key Patterns**:

1. **Dynamic Database Discovery**: Query master database to get tenant database names
   ```bash
   TENANT_DBS=$(psql -h localhost -d wellos_master -t -c "SELECT database_name FROM tenants ORDER BY subdomain;")
   ```

2. **Color-Coded Output**: Use ANSI colors for better UX
   ```bash
   echo -e "${GREEN}✓${NC}"  # Success
   echo -e "${RED}✗${NC}"    # Error
   echo -e "${BLUE}📦 Step 1${NC}"  # Info
   ```

3. **Fail-Fast**: `set -e` ensures script exits on first error

4. **Optional Seeding**: Use `--seed` flag for demo data

#### 2. **Database Cleanup Script**

```bash
#!/bin/bash
# scripts/drop-dbs.sh

##
## Drop WellOS Databases
##
## Drops all WellOS databases (master + tenant databases)
## Usage: ./scripts/drop-dbs.sh
##

set -e

echo "🗑️  Dropping WellOS databases..."
echo ""

# Database list
DATABASES=(
  "wellos_master"
  "wellos_internal"
  "demo_wellos"
)

# Terminate all connections to WellOS databases
echo "⚠️  Terminating active connections..."
for db in "${DATABASES[@]}"; do
  psql -h localhost -d postgres -c "
    SELECT pg_terminate_backend(pid)
    FROM pg_stat_activity
    WHERE datname = '$db' AND pid <> pg_backend_pid();
  " 2>/dev/null || true
done

echo ""
echo "🔥 Dropping databases..."

# Drop each database
for db in "${DATABASES[@]}"; do
  echo -n "   Dropping $db... "
  if psql -h localhost -d postgres -c "DROP DATABASE IF EXISTS $db;" 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
  else
    echo -e "${RED}✗${NC}"
    echo -e "${RED}Error: Failed to drop $db${NC}"
    exit 1
  fi
done

echo ""
echo -e "${GREEN}✅ All WellOS databases dropped successfully!${NC}"
```

**Key Patterns**:

1. **Connection Termination**: Terminate active connections before dropping
   ```bash
   SELECT pg_terminate_backend(pid)
   FROM pg_stat_activity
   WHERE datname = '$db' AND pid <> pg_backend_pid();
   ```

2. **Safe Dropping**: Use `DROP DATABASE IF EXISTS` to avoid errors

3. **Continue on Error**: Use `|| true` for connection termination (may fail if no connections)

### Usage

#### Development Workflows

**Complete reset with demo data**:
```bash
./scripts/drop-dbs.sh && ./scripts/create-dbs.sh --seed
```

**Fresh databases without demo data**:
```bash
./scripts/drop-dbs.sh && ./scripts/create-dbs.sh
```

**First-time setup**:
```bash
./scripts/create-dbs.sh --seed
```

**Clean slate for testing migration changes**:
```bash
./scripts/drop-dbs.sh
# Edit schema files
pnpm --filter=api db:generate:tenant
./scripts/create-dbs.sh
```

## Benefits

### ✅ Developer Experience
- **One command setup**: `./scripts/create-dbs.sh --seed`
- **Clear visual feedback**: Color-coded progress indicators
- **Self-explanatory**: Scripts are documentation

### ✅ Self-Documentation
- Scripts query master database for tenant list (no hardcoding)
- Output shows what was created (credentials, database names, tenant info)
- Comments explain each step

### ✅ Idempotent Operations
- `CREATE DATABASE` checks if already exists
- Seed scripts use `onConflictDoNothing()`
- Can be run multiple times safely

### ✅ Error Handling
- `set -e`: Exit on first error
- Clear error messages: "Database may already exist. Run './scripts/drop-dbs.sh' first."
- Color-coded status indicators

### ✅ Environment Flexibility
- Scripts work on macOS, Linux, Windows (WSL)
- PostgreSQL standard commands (no vendor lock-in)
- Easy to adapt for Docker, Railway, Azure

## Trade-offs

### Advantages
- **Fast onboarding**: New developers set up in 1 command
- **Consistent environments**: Everyone has same database state
- **Easy testing**: Reset to clean slate quickly
- **No drift**: Scripts are source of truth

### Disadvantages
- **Bash dependency**: Requires Bash shell (not native on Windows)
- **Hardcoded database names**: `drop-dbs.sh` has hardcoded list (could query master DB instead)
- **No rollback**: Dropping databases is destructive (could add backup step)
- **Single environment**: Scripts assume localhost PostgreSQL (could parameterize)

## When to Use

✅ **Use this pattern when**:
- You have database-per-tenant architecture
- You need to onboard new developers quickly
- You want consistent development environments
- You frequently reset databases during development
- You have CI/CD pipelines that need fresh databases

❌ **Don't use this pattern when**:
- You have a single database (use migrations only)
- You're managing production databases (use Infrastructure as Code tools like Terraform)
- You need fine-grained control (use manual commands)
- Database list changes frequently (implement dynamic discovery for drop script)

## Real-World Example: WellOS

**Scenario**: New developer joins team and needs to set up WellOS locally.

**Before (Manual Setup)**:
```bash
# 1. Create master database
psql -h localhost -d postgres -c "CREATE DATABASE wellos_master OWNER wellos;"

# 2. Run master migrations
cd apps/api
pnpm exec tsx src/infrastructure/database/scripts/migrate-master.ts

# 3. Seed master database
pnpm exec tsx src/infrastructure/database/seeds/master.seed.ts

# 4. Check what tenants were created
psql -h localhost -d wellos_master -c "SELECT database_name FROM tenants;"

# 5. Create tenant databases manually
psql -h localhost -d postgres -c "CREATE DATABASE wellos_internal OWNER wellos;"
psql -h localhost -d postgres -c "CREATE DATABASE demo_wellos OWNER wellos;"

# 6. Run tenant migrations
pnpm --filter=api db:migrate:tenant

# 7. Seed demo tenant
pnpm exec tsx apps/api/src/infrastructure/database/seeds/tenant.seed.ts demo

# Total time: ~10 minutes
# Error-prone: Forgot to create wellos_internal, migrations fail
```

**After (Automated)**:
```bash
./scripts/create-dbs.sh --seed

# Total time: 30 seconds
# Success rate: 100%
```

**Output**:
```
🚀 Creating WellOS databases...

📦 Step 1: Creating master database
   Creating wellos_master... ✓

📦 Step 2: Running master database migrations
✅ Master database migrations completed successfully!

📦 Step 3: Seeding master database (creating tenants)
✅ Master database seed completed!

📦 Step 4: Creating tenant databases
   Creating demo_wellos... ✓
   Creating wellos_internal... ✓

📦 Step 5: Running tenant database migrations
✅ All tenant migrations completed successfully!

📦 Step 6: Seeding demo tenant database
✅ Created 15 wells in Permian Basin
✅ Created 485 field entries

✅ WellOS databases created successfully!

📊 Database Summary:
 subdomain |         name         |   database_name    | subscription_tier | status
-----------+----------------------+--------------------+-------------------+--------
 demo      | Demo Oil Company     | demo_wellos     | STARTER           | TRIAL
 wellos | WellOS (Internal) | wellos_internal | ENTERPRISE        | ACTIVE

📝 Login Credentials:
  Master Admin:
    Email: admin@onwellos.com
    Password: WellOS2025!

  Demo Tenant Users:
    operator@permianpetroleum.com
    Password (all): Test123!@#
```

## Anti-Patterns

### ❌ Hardcoding Database Names in Drop Script

**Bad**:
```bash
DATABASES=("wellos_master" "acme_db" "demo_db")
```

**Problem**: Must update script every time a tenant is added.

**Better**:
```bash
# Query master database for tenant DBs
TENANT_DBS=$(psql -d wellos_master -t -c "SELECT database_name FROM tenants;")
DATABASES=("wellos_master" $TENANT_DBS)
```

### ❌ Not Terminating Connections Before Drop

**Bad**:
```bash
psql -c "DROP DATABASE wellos_master;"
# ERROR: database "wellos_master" is being accessed by other users
```

**Better**:
```bash
psql -c "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = 'wellos_master';"
psql -c "DROP DATABASE wellos_master;"
```

### ❌ No Color Coding or Progress Indicators

**Bad**:
```bash
Creating master database
Creating tenant databases
Done
```

**Better**:
```bash
echo -e "${BLUE}📦 Step 1: Creating master database${NC}"
echo -n "   Creating wellos_master... "
echo -e "${GREEN}✓${NC}"
```

## Variations

### Cloud Deployment (Azure)

Replace `psql` with Azure CLI:

```bash
# Create master database
az postgres flexible-server db create \
  --resource-group wellos-rg \
  --server-name wellos-master \
  --database-name wellos_master
```

### Docker-Based Development

Add Docker Compose service:

```yaml
services:
  db-setup:
    build: ./scripts
    depends_on:
      - postgres
    command: /scripts/create-dbs.sh --seed
```

### Kubernetes

Use Kubernetes Job:

```yaml
apiVersion: batch/v1
kind: Job
metadata:
  name: db-provision
spec:
  template:
    spec:
      containers:
        - name: provisioner
          image: wellos/db-provisioner:latest
          command: ['/scripts/create-dbs.sh', '--seed']
```

## Related Patterns

- **[73. Migration-Based Schema Management Pattern](./73-Migration-Based-Schema-Management-Pattern.md)**: Scripts run migrations as part of provisioning
- **[74. Database Seeding Pattern](./74-Database-Seeding-Pattern.md)**: Scripts optionally seed demo data
- **[69. Database-Per-Tenant Multi-Tenancy Pattern](./69-Database-Per-Tenant-Multi-Tenancy-Pattern.md)**: Architecture that requires this automation
- **Infrastructure as Code (IaC)**: Same principles (declarative, idempotent, version-controlled)

## Summary

The Database Provisioning Automation pattern uses Bash scripts to automate the complete lifecycle of multi-tenant database setup. By querying the master database to dynamically discover which tenant databases to create, the scripts remain self-documenting and don't require manual updates as tenants are added.

**Key Insights**:
- Scripts are documentation (comments + clear output)
- Dynamic discovery prevents hardcoding
- Color-coded output improves developer experience
- Idempotent operations allow safe re-runs
- Connection termination prevents drop failures

This pattern transforms database setup from a 10-minute manual process into a 30-second automated command, reducing errors and improving developer onboarding.
