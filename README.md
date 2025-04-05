# OneSociety - P2P Rental Marketplace

A modern peer-to-peer rental marketplace platform built with a microservices architecture.

## Project Structure

```
.
├── apps/                           # Application services
│   ├── api-gateway/               # API Gateway service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── auth-service/             # Authentication & Authorization (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── user-service/            # User Management Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── product-service/         # Product Management Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── rental-service/          # Rental Management Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── payment-service/         # Payment & Billing Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── messaging-service/       # Real-time Messaging Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── notification-service/    # Notification Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── search-service/         # Search & Discovery Service (Rust)
│   │   ├── src/
│   │   └── Cargo.toml
│   └── mobile-app/            # Flutter Mobile Application
│       ├── android/
│       ├── ios/
│       └── lib/
├── libs/                      # Shared libraries and utilities
│   ├── common/               # Common utilities and helpers
│   │   ├── src/
│   │   └── Cargo.toml
│   ├── models/              # Shared data models
│   │   ├── src/
│   │   └── Cargo.toml
│   └── proto/               # Protocol Buffers definitions
│       └── src/
├── tools/                    # Development and deployment tools
│   ├── scripts/             # Utility scripts
│   └── ci/                  # CI/CD configuration
├── config/                   # Configuration files
│   ├── development/
│   ├── staging/
│   └── production/
├── deploy/                   # Deployment configurations
│   ├── kubernetes/          # Kubernetes manifests
│   │   ├── base/
│   │   └── overlays/
│   └── docker/             # Docker configurations
│       └── Dockerfile.*
├── docs/                    # Documentation
│   ├── api/                # API documentation
│   ├── architecture/       # Architecture documentation
│   └── guides/            # Development guides
├── db/                     # Database migrations and schemas
│   ├── migrations/        # Database migrations
│   └── schemas/          # Database schemas
├── tests/                 # Integration and E2E tests
│   ├── integration/
│   └── e2e/
└── web/                   # Web frontend (if needed in future)
    └── admin/            # Admin dashboard

Key files:
├── .gitignore
├── docker-compose.yml    # Local development setup
├── Makefile             # Build and development commands
├── README.md            # Project documentation
└── rust-toolchain.toml  # Rust version management
```

## Technology Stack

- **Backend Services**: Rust
- **Mobile App**: Flutter
- **Database**: PostgreSQL
- **Message Broker**: RabbitMQ/Kafka
- **API Gateway**: Custom Rust implementation
- **Service Discovery**: Consul
- **Monitoring**: Prometheus & Grafana
- **Logging**: ELK Stack
- **Caching**: Redis
- **Search**: Elasticsearch
- **Storage**: S3-compatible object storage

## Getting Started

1. Install prerequisites:
   - Rust (latest stable)
   - Flutter SDK
   - Docker & Docker Compose
   - PostgreSQL
   - Make

2. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/onesociety.git
   cd onesociety
   ```

3. Set up development environment:
   ```bash
   make setup
   ```

4. Start local development environment:
   ```bash
   make dev
   ```

## Development Guidelines

- Each service should be self-contained with its own database schema
- Use shared libraries from `libs/` for common functionality
- Follow the API-first approach using Protocol Buffers
- Write tests for all new features
- Document APIs and architectural decisions

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 