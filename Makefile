.PHONY: setup dev test build clean proto lint format help db-setup db-migrate db-rollback db-status db-validate db-validate-full db-docker-setup db-docker-teardown db-docker-restart db-docker-status db-docker-reset

# Default target
.DEFAULT_GOAL := help

# Variables
DOCKER_COMPOSE = docker-compose
CARGO = cargo
FLUTTER = flutter
MIGRATE = ./db/bin/migrate.sh
VALIDATE = ./db/bin/validate.sh
DB_MANAGE = ./db/scripts/manage_db.sh

help: ## Display this help message
	@echo "Usage: make <target>"
	@echo ""
	@echo "Targets:"
	@awk '/^[a-zA-Z0-9_-]+:.*?## .*$$/ {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Set up development environment
	@echo "Setting up development environment..."
	$(CARGO) install cargo-watch
	$(CARGO) install sqlx-cli
	$(CARGO) install cargo-tarpaulin
	$(CARGO) install cargo-audit
	@echo "Installing Flutter dependencies..."
	cd apps/mobile-app && $(FLUTTER) pub get

dev: ## Start development environment
	@echo "Starting development environment..."
	$(DOCKER_COMPOSE) up -d

stop: ## Stop development environment
	@echo "Stopping development environment..."
	$(DOCKER_COMPOSE) down

build: ## Build all services
	@echo "Building all services..."
	$(CARGO) build --workspace

test: ## Run all tests
	@echo "Running tests..."
	$(CARGO) test --workspace
	cd apps/mobile-app && $(FLUTTER) test

proto: ## Generate Protocol Buffer code
	@echo "Generating Protocol Buffer code..."
	cd libs/proto && $(CARGO) build

db-setup: ## Initialize database schemas and migration table
	@echo "Setting up database schemas..."
	$(MIGRATE) setup

db-migrate: db-validate ## Run migrations for a service (usage: make db-migrate service=<service_name>)
	@if [ "$(service)" = "" ]; then \
		echo "Error: service parameter is required. Usage: make db-migrate service=<service_name>"; \
		exit 1; \
	fi
	@echo "Running migrations for $(service) service..."
	$(MIGRATE) up $(service)

db-rollback: db-validate ## Rollback last migration for a service (usage: make db-rollback service=<service_name>)
	@if [ "$(service)" = "" ]; then \
		echo "Error: service parameter is required. Usage: make db-rollback service=<service_name>"; \
		exit 1; \
	fi
	@echo "Rolling back last migration for $(service) service..."
	$(MIGRATE) down $(service)

db-status: ## Show migration status for all services
	@echo "Showing migration status..."
	$(MIGRATE) status

db-new: ## Create a new migration (usage: make db-new service=<service_name> name=<migration_name>)
	@if [ "$(service)" = "" ] || [ "$(name)" = "" ]; then \
		echo "Error: service and name parameters are required. Usage: make db-new service=<service_name> name=<migration_name>"; \
		exit 1; \
	fi
	@echo "Creating new migration for $(service) service..."
	$(MIGRATE) new $(service) $(name)

db-validate: ## Run quick schema validation checks
	@echo "Running schema validation..."
	$(VALIDATE)

db-validate-full: ## Run comprehensive schema validation with detailed report
	@echo "Running comprehensive schema validation..."
	@echo "Checking extensions..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM check_required_extensions();"
	@echo "\nChecking schemas..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM check_required_schemas();"
	@echo "\nChecking foreign keys..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM validate_foreign_keys();"
	@echo "\nChecking for orphaned records..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM check_orphaned_records();"
	@echo "\nChecking for missing indexes..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM check_missing_fk_indexes();"
	@echo "\nChecking for tables without primary keys..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM check_tables_without_pk();"
	@echo "\nChecking audit triggers..."
	@psql "$${DATABASE_URL}" -c "SELECT * FROM validate_audit_triggers();"

db-docker-setup: ## Set up PostgreSQL in Docker (usage: make db-docker-setup [args])
	$(DB_MANAGE) setup $(args)

db-docker-teardown: ## Teardown PostgreSQL Docker container (usage: make db-docker-teardown [args])
	$(DB_MANAGE) teardown $(args)

db-docker-restart: ## Restart PostgreSQL Docker container (usage: make db-docker-restart [args])
	$(DB_MANAGE) restart $(args)

db-docker-status: ## Show PostgreSQL Docker container status
	$(DB_MANAGE) status

db-docker-reset: ## Reset PostgreSQL Docker container data (usage: make db-docker-reset [args])
	$(DB_MANAGE) reset $(args)

lint: ## Run linters
	@echo "Running linters..."
	$(CARGO) clippy --workspace -- -D warnings
	cd apps/mobile-app && $(FLUTTER) analyze

format: ## Format code
	@echo "Formatting code..."
	$(CARGO) fmt --all
	cd apps/mobile-app && $(FLUTTER) format .

clean: ## Clean build artifacts
	@echo "Cleaning build artifacts..."
	$(CARGO) clean
	cd apps/mobile-app && $(FLUTTER) clean
	$(DOCKER_COMPOSE) down -v

audit: ## Run security audit
	@echo "Running security audit..."
	$(CARGO) audit

docs: ## Generate documentation
	@echo "Generating documentation..."
	$(CARGO) doc --workspace --no-deps

update-deps: ## Update dependencies
	@echo "Updating Rust dependencies..."
	$(CARGO) update
	@echo "Updating Flutter dependencies..."
	cd apps/mobile-app && $(FLUTTER) pub upgrade 