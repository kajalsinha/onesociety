#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script location and paths
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DB_DIR="$(dirname "$SCRIPT_DIR")"
MIGRATIONS_DIR="$DB_DIR/migrations"
SCHEMAS_DIR="$DB_DIR/schemas"
SCHEMA_FILE="$SCHEMAS_DIR/onesociety.sql"
VALIDATE_SCRIPT="$SCRIPT_DIR/validate.sh"

# Functions
print_usage() {
    echo -e "${BLUE}OneSociety Database Migration Manager${NC}"
    echo 
    echo "Usage: $0 <command> [service] [name]"
    echo
    echo "Commands:"
    echo "  setup                   Initialize database schemas"
    echo "  new <service> <name>    Create a new migration"
    echo "  up [service]            Run all pending migrations for a service (or all services if omitted)"
    echo "  down [service]          Rollback last migration for a service (or all services if omitted)"
    echo "  status                  Show migration status for all services"
    echo 
    echo "Examples:"
    echo "  $0 setup                # Initialize database with schemas"
    echo "  $0 new user create_table_users  # Create a new migration for user service"
    echo "  $0 up user              # Run all pending migrations for user service"
    echo "  $0 down user            # Rollback last migration for user service"
    echo "  $0 status               # Show migration status"
    echo
    echo "Environment Variables:"
    echo "  DATABASE_URL            PostgreSQL connection string"
}

log() {
    local level=$1
    shift
    case $level in
        "info")
            echo -e "${BLUE}[INFO]${NC} $*"
            ;;
        "success")
            echo -e "${GREEN}[SUCCESS]${NC} $*"
            ;;
        "warning")
            echo -e "${YELLOW}[WARNING]${NC} $*"
            ;;
        "error")
            echo -e "${RED}[ERROR]${NC} $*" >&2
            ;;
    esac
}

confirm() {
    read -p "Are you sure? [y/N] " -n 1 -r
    echo
    [[ $REPLY =~ ^[Yy]$ ]]
}

ensure_migration_table() {
    local service=$1
    log "info" "Ensuring migrations table exists for $service service..."
    
    psql "$DATABASE_URL" << EOF
    CREATE TABLE IF NOT EXISTS ${service}_schema.migrations (
        id SERIAL PRIMARY KEY,
        version VARCHAR(14) NOT NULL,
        name VARCHAR(255) NOT NULL,
        applied_at TIMESTAMPTZ DEFAULT NOW()
    );
EOF
}

setup_database() {
    log "info" "Setting up database schemas..."
    
    if [ ! -f "$SCHEMA_FILE" ]; then
        log "error" "Schema file not found: $SCHEMA_FILE"
        exit 1
    fi
    
    # Run the schema creation
    log "info" "Applying base schema..."
    if psql "$DATABASE_URL" -f "$SCHEMA_FILE"; then
        log "success" "Database schemas created successfully"
    else
        log "error" "Failed to create database schemas"
        exit 1
    fi
    
    # Create migrations tables for each service
    for service_dir in "$MIGRATIONS_DIR"/*; do
        if [ -d "$service_dir" ]; then
            service=$(basename "$service_dir")
            ensure_migration_table "$service"
        fi
    done
    
    # Validate schema
    log "info" "Validating schema..."
    if "$VALIDATE_SCRIPT"; then
        log "success" "Schema validation passed"
    else
        log "warning" "Schema validation had issues, but setup will continue"
    fi
}

create_migration() {
    local service=$1
    local name=$2
    local timestamp=$(date +%Y%m%d%H%M%S)
    local service_dir="$MIGRATIONS_DIR/$service"
    
    # Verify the service directory exists
    if [ ! -d "$service_dir" ]; then
        log "info" "Creating directory for $service service..."
        mkdir -p "$service_dir"
    fi
    
    # Ensure migrations table exists
    ensure_migration_table "$service"
    
    # Create migration files
    local migration_file="$service_dir/${timestamp}_${name}.sql"
    log "info" "Creating migration: $migration_file"
    
    cat > "$migration_file" << EOF
-- Migration: $name
-- Created at: $(date -u +"%Y-%m-%d %H:%M:%S")
-- Service: $service

-- Up Migration
-- Changes to apply
BEGIN;

-- Set schema to service schema
SET search_path TO ${service}_schema, public;

-- Your migration code here
-- Example: CREATE TABLE users (id SERIAL PRIMARY KEY, name TEXT, email TEXT);

-- Record this migration
INSERT INTO ${service}_schema.migrations (version, name) 
VALUES ('$timestamp', '$name');

COMMIT;

-- Down Migration
-- Changes to rollback (commented out, uncomment to enable rollback)
/*
BEGIN;

-- Set schema to service schema
SET search_path TO ${service}_schema, public;

-- Your rollback code here
-- Example: DROP TABLE users;

-- Remove this migration record
DELETE FROM ${service}_schema.migrations 
WHERE version = '$timestamp';

COMMIT;
*/
EOF

    log "success" "Migration created: $migration_file"
}

run_migrations() {
    local service=$1
    local all_services=false
    
    if [ -z "$service" ]; then
        all_services=true
        log "info" "Running migrations for all services..."
    else
        log "info" "Running migrations for $service service..."
    fi
    
    # First validate schema
    log "info" "Validating schema before migrations..."
    if ! "$VALIDATE_SCRIPT"; then
        log "warning" "Schema validation issues detected. Proceeding with caution."
    fi
    
    # Handle running for a specific service or all services
    if $all_services; then
        local success=true
        
        for service_dir in "$MIGRATIONS_DIR"/*; do
            if [ -d "$service_dir" ]; then
                service=$(basename "$service_dir")
                if ! run_service_migrations "$service"; then
                    success=false
                fi
            fi
        done
        
        if $success; then
            log "success" "All migrations applied successfully"
            return 0
        else
            log "error" "Some migrations failed"
            return 1
        fi
    else
        # Run for specific service
        if run_service_migrations "$service"; then
            log "success" "Migrations applied successfully for $service service"
            return 0
        else
            log "error" "Failed to apply migrations for $service service"
            return 1
        fi
    fi
}

run_service_migrations() {
    local service=$1
    local service_dir="$MIGRATIONS_DIR/$service"
    local success=true
    
    # Check if service directory exists
    if [ ! -d "$service_dir" ]; then
        log "error" "Service directory not found: $service_dir"
        return 1
    fi
    
    # Ensure migrations table exists
    ensure_migration_table "$service"
    
    # Get list of applied migrations
    local applied_migrations=$(psql "$DATABASE_URL" -t -c "SELECT version FROM ${service}_schema.migrations ORDER BY version;" | tr -d '[:space:]')
    
    # Run each migration file if not already applied
    for migration_file in "$service_dir"/*.sql; do
        if [ -f "$migration_file" ]; then
            local file_name=$(basename "$migration_file")
            local version=${file_name%%_*}
            
            # Check if this migration has been applied
            if echo "$applied_migrations" | grep -q "$version"; then
                log "info" "Skipping already applied migration: $file_name"
            else
                log "info" "Applying migration: $file_name"
                
                # Extract and run the Up Migration section
                local up_migration=$(sed -n '/^-- Up Migration/,/^-- Down Migration/p' "$migration_file" | sed '1d;$d')
                
                if echo "$up_migration" | psql "$DATABASE_URL"; then
                    log "success" "Applied migration: $file_name"
                else
                    log "error" "Failed to apply migration: $file_name"
                    success=false
                    break
                fi
            fi
        fi
    done
    
    return $([ "$success" = true ] && echo 0 || echo 1)
}

rollback_migration() {
    local service=$1
    local all_services=false
    
    if [ -z "$service" ]; then
        all_services=true
        log "info" "Rolling back last migration for all services..."
    else
        log "info" "Rolling back last migration for $service service..."
    fi
    
    # First validate schema
    log "info" "Validating schema before rollback..."
    if ! "$VALIDATE_SCRIPT"; then
        log "warning" "Schema validation issues detected. Proceeding with caution."
    fi
    
    # Handle rolling back for a specific service or all services
    if $all_services; then
        log "warning" "This will rollback the last migration for ALL services"
        if ! confirm; then
            log "info" "Rollback cancelled"
            return 0
        fi
        
        local success=true
        
        for service_dir in "$MIGRATIONS_DIR"/*; do
            if [ -d "$service_dir" ]; then
                service=$(basename "$service_dir")
                if ! rollback_service_migration "$service"; then
                    success=false
                fi
            fi
        done
        
        if $success; then
            log "success" "All migrations rolled back successfully"
            return 0
        else
            log "error" "Some rollbacks failed"
            return 1
        fi
    else
        # Rollback for specific service
        if rollback_service_migration "$service"; then
            log "success" "Migration rolled back successfully for $service service"
            return 0
        else
            log "error" "Failed to rollback migration for $service service"
            return 1
        fi
    fi
}

rollback_service_migration() {
    local service=$1
    local service_dir="$MIGRATIONS_DIR/$service"
    
    # Check if service directory exists
    if [ ! -d "$service_dir" ]; then
        log "error" "Service directory not found: $service_dir"
        return 1
    fi
    
    # Get the latest applied migration
    local latest_migration=$(psql "$DATABASE_URL" -t -c "
        SELECT version, name FROM ${service}_schema.migrations 
        ORDER BY version DESC LIMIT 1;
    ")
    
    if [ -z "$latest_migration" ]; then
        log "warning" "No migrations to roll back for $service service"
        return 0
    fi
    
    local version=$(echo "$latest_migration" | awk '{print $1}')
    local name=$(echo "$latest_migration" | awk '{$1=""; print $0}' | xargs)
    
    # Find the migration file
    local migration_file="$service_dir/${version}_${name}.sql"
    if [ ! -f "$migration_file" ]; then
        # Try to find by version only
        migration_file=$(find "$service_dir" -name "${version}_*.sql" | head -n 1)
        
        if [ ! -f "$migration_file" ]; then
            log "error" "Migration file not found for version $version"
            return 1
        fi
    fi
    
    log "info" "Rolling back migration: $(basename "$migration_file")"
    
    # Extract and run the Down Migration section
    local down_migration=$(sed -n '/^\/\*$/,/^\*\//p' "$migration_file" | sed '1d;$d')
    
    if [ -z "$down_migration" ]; then
        log "error" "No down migration found in file. Rollback section might be commented out."
        log "info" "Edit the file and uncomment the down migration section, then try again."
        return 1
    fi
    
    if echo "$down_migration" | psql "$DATABASE_URL"; then
        log "success" "Rolled back migration: $(basename "$migration_file")"
        return 0
    else
        log "error" "Failed to roll back migration: $(basename "$migration_file")"
        return 1
    fi
}

show_status() {
    log "info" "Showing migration status..."
    
    echo -e "\n${BLUE}=== Migration Status ===${NC}\n"
    
    for service_dir in "$MIGRATIONS_DIR"/*; do
        if [ -d "$service_dir" ]; then
            service=$(basename "$service_dir")
            
            echo -e "${YELLOW}Service: $service${NC}"
            
            # Check if migrations table exists
            if ! psql "$DATABASE_URL" -t -c "
                SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_schema = '${service}_schema' 
                    AND table_name = 'migrations'
                );
            " | grep -q 't'; then
                echo "  Migration table not found. Run 'setup' command first."
                continue
            fi
            
            # Get count of applied migrations
            local applied_count=$(psql "$DATABASE_URL" -t -c "
                SELECT COUNT(*) FROM ${service}_schema.migrations;
            " | tr -d '[:space:]')
            
            # Get count of migration files
            local file_count=$(find "$service_dir" -name "*.sql" | wc -l | tr -d '[:space:]')
            
            echo "  Applied migrations: $applied_count"
            echo "  Available migrations: $file_count"
            
            if [ "$applied_count" -ne "$file_count" ]; then
                echo -e "  ${YELLOW}Status: $((file_count - applied_count)) migrations pending${NC}"
            else
                echo -e "  ${GREEN}Status: Up to date${NC}"
            fi
            
            # List last 5 applied migrations
            echo -e "\n  ${BLUE}Latest applied migrations:${NC}"
            psql "$DATABASE_URL" -c "
                SELECT version, name, applied_at 
                FROM ${service}_schema.migrations 
                ORDER BY version DESC 
                LIMIT 5;
            "
            echo
        fi
    done
}

# Default DATABASE_URL if not set
DATABASE_URL=${DATABASE_URL:-"postgresql://onesociety:development_password@localhost:5432/onesociety"}

# Check command line arguments
if [ $# -lt 1 ]; then
    print_usage
    exit 1
fi

# Process commands
case "$1" in
    setup)
        setup_database
        ;;
    new)
        if [ $# -lt 3 ]; then
            log "error" "Missing service name or migration name"
            print_usage
            exit 1
        fi
        create_migration "$2" "$3"
        ;;
    up)
        run_migrations "${2:-}"
        ;;
    down)
        rollback_migration "${2:-}"
        ;;
    status)
        show_status
        ;;
    help)
        print_usage
        ;;
    *)
        log "error" "Unknown command: $1"
        print_usage
        exit 1
        ;;
esac

exit 0 