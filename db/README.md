# OneSociety Database Management

This directory contains the database management tools for the OneSociety application.

## Directory Structure

```
db/
├── bin/                   # Executable shell scripts
│   ├── migrate.sh         # Migration management script
│   └── validate.sh        # Schema validation script
├── migrations/            # Database migrations organized by service
│   ├── auth/              # Authentication service migrations
│   ├── user/              # User service migrations
│   ├── product/           # Product service migrations
│   ├── rental/            # Rental service migrations
│   ├── payment/           # Payment service migrations
│   ├── messaging/         # Messaging service migrations
│   ├── review/            # Review service migrations
│   └── subscription/      # Subscription service migrations
├── schemas/               # Base schema definitions
│   └── onesociety.sql     # Main schema creation script
├── scripts/               # Management scripts
│   └── manage_db.sh       # Docker PostgreSQL instance management
├── seed/                  # Seed data for development
│   └── initial_data.sql   # Initial seed data
└── sql/                   # SQL utility scripts
    └── validate_schema.sql # Schema validation functions
```

## Usage

### Database Setup and Management

```bash
# Set up a Docker PostgreSQL instance
make db-docker-setup [args]

# Teardown the Docker PostgreSQL instance
make db-docker-teardown [args]

# Restart the Docker PostgreSQL instance
make db-docker-restart [args]

# Show the status of the Docker PostgreSQL instance
make db-docker-status

# Reset the database (destructive operation)
make db-docker-reset [args]
```

Available arguments for Docker management:
- `--container=NAME` - Set container name (default: onesociety_postgres)
- `--database=NAME` - Set database name (default: onesociety)
- `--user=USERNAME` - Set database user (default: onesociety)
- `--password=PASSWORD` - Set database password (default: development_password)
- `--port=PORT` - Set host port (default: 5432)
- `--version=VERSION` - Set PostgreSQL version (default: 15-3.3)
- `--force` - Force operation without confirmation
- `--no-seed` - Skip seeding initial data
- `--env=ENV` - Use specific environment config (dev, staging, prod)

### Database Schema and Migrations

```bash
# Initialize database schemas
make db-setup

# Create a new migration
make db-new service=user name=create_users_table

# Run pending migrations for a service
make db-migrate service=user

# Rollback the last migration for a service
make db-rollback service=user

# Show migration status for all services
make db-status
```

### Schema Validation

```bash
# Run quick schema validation
make db-validate

# Run comprehensive schema validation with detailed reports
make db-validate-full
```

## Database Design

The OneSociety database is designed with logical schemas to support a future microservice architecture. Each service has its own schema, which can be moved to a separate database in the future if needed:

- `user_schema` - User accounts and profiles
- `auth_schema` - Authentication and authorization
- `product_schema` - Product listings and categories
- `rental_schema` - Rental bookings and policies
- `payment_schema` - Payment transactions
- `messaging_schema` - User-to-user messaging
- `review_schema` - Product and user reviews
- `subscription_schema` - Subscription plans and user subscriptions
- `audit_schema` - Audit logs and tracking

## Migration Organization

Migrations are organized by service and follow a consistent structure:
- Each migration has a timestamp prefix (`YYYYMMDDHHMMSS_name.sql`)
- Migrations contain both UP and DOWN sections
- Rollback (DOWN) sections are commented out by default
- Each migration records itself in a service-specific migrations table

## Environment Configuration

The database tools support multiple environments through configuration files located in the `config/` directory:
- `config/development/database.env`
- `config/staging/database.env`
- `config/production/database.env`

Use the `--env` flag to specify which environment configuration to use, or set the `DATABASE_URL` environment variable directly. 