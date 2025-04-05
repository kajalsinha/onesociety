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
PROJECT_ROOT="$(dirname "$DB_DIR")"
MIGRATE_SCRIPT="$DB_DIR/bin/migrate.sh"
VALIDATE_SCRIPT="$DB_DIR/bin/validate.sh"
SEED_DIR="$DB_DIR/seed"
CONFIG_DIR="$PROJECT_ROOT/config"

# Default values
DB_CONTAINER="onesociety_postgres"
DB_NAME="onesociety"
DB_USER="onesociety"
DB_PASSWORD="development_password"
DB_PORT="5432"
DB_VERSION="15-3.3"
DOCKER_NETWORK="onesociety_network"

# Function to print usage
print_usage() {
    echo -e "${BLUE}OneSociety Database Management Script${NC}"
    echo
    echo "Usage: $0 [options] <command>"
    echo
    echo "Commands:"
    echo "  setup       Create and initialize the database"
    echo "  teardown    Remove the database container and volumes"
    echo "  restart     Restart the database container"
    echo "  status      Show database container status"
    echo "  reset       Reset database to initial state (destructive)"
    echo
    echo "Options:"
    echo "  -h, --help                 Show this help message"
    echo "  -c, --container NAME       Set container name (default: $DB_CONTAINER)"
    echo "  -d, --database NAME        Set database name (default: $DB_NAME)"
    echo "  -u, --user USERNAME        Set database user (default: $DB_USER)"
    echo "  -p, --password PASSWORD    Set database password (default: $DB_PASSWORD)"
    echo "  -P, --port PORT           Set host port (default: $DB_PORT)"
    echo "  -v, --version VERSION      Set PostgreSQL version (default: $DB_VERSION)"
    echo "  -f, --force               Force operation without confirmation"
    echo "  --no-seed                 Skip seeding initial data"
    echo "  --env ENV                 Use specific environment config (dev, staging, prod)"
    echo
    echo "Environment Variables:"
    echo "  POSTGRES_HOST     Override database host"
    echo "  POSTGRES_PORT     Override database port"
    echo "  POSTGRES_DB       Override database name"
    echo "  POSTGRES_USER     Override database user"
    echo "  POSTGRES_PASSWORD Override database password"
}

# Function to log messages
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

# Function to load environment config
load_env_config() {
    local env=${ENV:-development}
    local config_file="$CONFIG_DIR/$env/database.env"
    
    if [[ -f "$config_file" ]]; then
        log "info" "Loading database configuration from $env environment"
        source "$config_file"
        
        # Override variables if environment variables are set
        DB_NAME=${POSTGRES_DB:-$DB_NAME}
        DB_USER=${POSTGRES_USER:-$DB_USER}
        DB_PASSWORD=${POSTGRES_PASSWORD:-$DB_PASSWORD}
        DB_PORT=${POSTGRES_PORT:-$DB_PORT}
    else
        log "warning" "No environment config found at $config_file, using defaults"
    fi
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        log "error" "Docker is not running or you don't have permission to use it"
        exit 1
    fi
}

# Function to check if container exists
container_exists() {
    docker ps -a --format '{{.Names}}' | grep -q "^${DB_CONTAINER}$"
}

# Function to check if container is running
container_running() {
    docker ps --format '{{.Names}}' | grep -q "^${DB_CONTAINER}$"
}

# Function to wait for PostgreSQL to be ready
wait_for_postgres() {
    local max_attempts=30
    local attempt=1
    
    log "info" "Waiting for PostgreSQL to be ready..."
    while [ $attempt -le $max_attempts ]; do
        if docker exec $DB_CONTAINER pg_isready -U $DB_USER > /dev/null 2>&1; then
            log "success" "PostgreSQL is ready"
            return 0
        fi
        log "info" "Attempt $attempt/$max_attempts: PostgreSQL is not ready yet..."
        sleep 1
        attempt=$((attempt + 1))
    done
    
    log "error" "PostgreSQL failed to become ready in time"
    return 1
}

# Function to create Docker network if it doesn't exist
ensure_network() {
    if ! docker network ls | grep -q "$DOCKER_NETWORK"; then
        log "info" "Creating Docker network: $DOCKER_NETWORK"
        docker network create "$DOCKER_NETWORK"
    fi
}

# Export DATABASE_URL for scripts
export_db_url() {
    export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@localhost:$DB_PORT/$DB_NAME"
    log "info" "Set DATABASE_URL environment variable"
}

# Function to setup database
setup_database() {
    if container_exists; then
        if [ "$FORCE" = true ]; then
            log "warning" "Force flag set, removing existing container..."
            docker rm -f $DB_CONTAINER > /dev/null
        else
            log "error" "Container $DB_CONTAINER already exists. Use -f to force recreation"
            exit 1
        fi
    fi

    ensure_network

    log "info" "Creating PostgreSQL container..."
    docker run -d \
        --name $DB_CONTAINER \
        --network $DOCKER_NETWORK \
        -e POSTGRES_DB=$DB_NAME \
        -e POSTGRES_USER=$DB_USER \
        -e POSTGRES_PASSWORD=$DB_PASSWORD \
        -p $DB_PORT:5432 \
        -v ${DB_CONTAINER}_data:/var/lib/postgresql/data \
        postgis/postgis:$DB_VERSION

    if ! wait_for_postgres; then
        log "error" "Failed to start PostgreSQL"
        exit 1
    fi

    # Initialize database
    log "info" "Initializing database..."
    export_db_url
    
    # Run database setup
    if ! $MIGRATE_SCRIPT setup; then
        log "error" "Failed to initialize database schemas"
        exit 1
    fi

    # Run schema validation
    if ! $VALIDATE_SCRIPT; then
        log "warning" "Schema validation failed"
    fi

    # Seed initial data if not disabled
    if [ "$NO_SEED" != true ]; then
        log "info" "Seeding initial data..."
        if [ -f "$SEED_DIR/initial_data.sql" ]; then
            docker exec -i $DB_CONTAINER psql -U $DB_USER -d $DB_NAME < "$SEED_DIR/initial_data.sql"
            log "success" "Database seeded successfully"
        else
            log "warning" "No initial data file found at $SEED_DIR/initial_data.sql"
        fi
    fi

    log "success" "Database setup completed successfully"
}

# Function to teardown database
teardown_database() {
    if ! container_exists; then
        log "warning" "Container $DB_CONTAINER does not exist"
        return 0
    fi

    if [ "$FORCE" != true ]; then
        read -p "Are you sure you want to remove the database container and all data? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "info" "Operation cancelled"
            exit 0
        fi
    fi

    log "info" "Removing container and volume..."
    docker rm -f $DB_CONTAINER > /dev/null
    docker volume rm ${DB_CONTAINER}_data > /dev/null
    log "success" "Database teardown completed"
}

# Function to restart database
restart_database() {
    if ! container_exists; then
        log "error" "Container $DB_CONTAINER does not exist"
        exit 1
    fi

    log "info" "Restarting PostgreSQL container..."
    docker restart $DB_CONTAINER > /dev/null
    
    if ! wait_for_postgres; then
        log "error" "Failed to restart PostgreSQL"
        exit 1
    fi
    
    log "success" "Database restart completed"
}

# Function to show database status
show_status() {
    if ! container_exists; then
        log "info" "Container status: ${RED}Not created${NC}"
        return
    fi

    local status
    if container_running; then
        status="${GREEN}Running${NC}"
    else
        status="${RED}Stopped${NC}"
    fi

    echo -e "Container name: $DB_CONTAINER"
    echo -e "Status: $status"
    echo -e "Database: $DB_NAME"
    echo -e "User: $DB_USER"
    echo -e "Port: $DB_PORT"
    
    if container_running; then
        echo "Container Info:"
        docker inspect $DB_CONTAINER | jq -r '.[0] | {
            "Image": .Config.Image,
            "Created": .Created,
            "Ports": .NetworkSettings.Ports,
            "Mounts": .Mounts[].Source
        }'
        
        # Show current database size
        echo -e "\nDatabase Size:"
        docker exec $DB_CONTAINER psql -U $DB_USER -d $DB_NAME -c "SELECT pg_size_pretty(pg_database_size('$DB_NAME')) as db_size;"
        
        # Show table count
        echo -e "\nTable Count:"
        docker exec $DB_CONTAINER psql -U $DB_USER -d $DB_NAME -c "SELECT count(*) as table_count FROM information_schema.tables WHERE table_schema NOT IN ('pg_catalog', 'information_schema', 'public');"
    fi
}

# Function to reset database
reset_database() {
    if [ "$FORCE" != true ]; then
        read -p "This will delete all data in the database. Are you sure? [y/N] " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "info" "Operation cancelled"
            exit 0
        fi
    fi

    if container_running; then
        log "info" "Resetting database..."
        export_db_url
        
        # Drop all schemas
        docker exec -i $DB_CONTAINER psql -U $DB_USER -d $DB_NAME << EOF
DO \$\$
DECLARE
    schema_name text;
BEGIN
    FOR schema_name IN SELECT nspname FROM pg_namespace
    WHERE nspname NOT IN ('pg_catalog', 'information_schema', 'public')
    LOOP
        EXECUTE 'DROP SCHEMA IF EXISTS ' || schema_name || ' CASCADE';
    END LOOP;
END \$\$;
EOF

        # Reinitialize database
        if ! $MIGRATE_SCRIPT setup; then
            log "error" "Failed to reinitialize database schemas"
            exit 1
        fi

        # Seed initial data if not disabled
        if [ "$NO_SEED" != true ]; then
            log "info" "Reseeding initial data..."
            if [ -f "$SEED_DIR/initial_data.sql" ]; then
                docker exec -i $DB_CONTAINER psql -U $DB_USER -d $DB_NAME < "$SEED_DIR/initial_data.sql"
                log "success" "Database reseeded successfully"
            fi
        fi

        log "success" "Database reset completed"
    else
        log "error" "Container is not running"
        exit 1
    fi
}

# Parse command line arguments
FORCE=false
NO_SEED=false
COMMAND=""
ENV="development"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            print_usage
            exit 0
            ;;
        -c|--container)
            DB_CONTAINER="$2"
            shift 2
            ;;
        -d|--database)
            DB_NAME="$2"
            shift 2
            ;;
        -u|--user)
            DB_USER="$2"
            shift 2
            ;;
        -p|--password)
            DB_PASSWORD="$2"
            shift 2
            ;;
        -P|--port)
            DB_PORT="$2"
            shift 2
            ;;
        -v|--version)
            DB_VERSION="$2"
            shift 2
            ;;
        -f|--force)
            FORCE=true
            shift
            ;;
        --no-seed)
            NO_SEED=true
            shift
            ;;
        --env)
            ENV="$2"
            shift 2
            ;;
        setup|teardown|restart|status|reset)
            COMMAND="$1"
            shift
            ;;
        *)
            log "error" "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Check if command is provided
if [ -z "$COMMAND" ]; then
    log "error" "No command specified"
    print_usage
    exit 1
fi

# Load environment-specific config
load_env_config

# Check if Docker is running
check_docker

# Execute command
case $COMMAND in
    setup)
        setup_database
        ;;
    teardown)
        teardown_database
        ;;
    restart)
        restart_database
        ;;
    status)
        show_status
        ;;
    reset)
        reset_database
        ;;
esac 