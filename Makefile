.PHONY: setup dev test build clean proto lint format help

# Default target
.DEFAULT_GOAL := help

# Variables
DOCKER_COMPOSE = docker-compose
CARGO = cargo
FLUTTER = flutter

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

migrate: ## Run database migrations
	@echo "Running database migrations..."
	cd db && sqlx migrate run

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