### OneSociety Project Plan

This folder contains the living project plan. It reflects a monolith-first approach with a single PostgreSQL database and Flutter frontend, while preserving a clean path to future microservices.

Contents:

- 01_current_state.md — Repo audit: structure, implemented vs missing, risks
- 02_architecture.md — Target architecture: monolith-first, service boundaries, environments
- 03_backend_plan.md — Rust backend milestones, APIs, scaffolding, testing
- 04_database_plan.md — Schemas, migrations, validation, data lifecycle
- 05_frontend_plan.md — Flutter app milestones, API integration, state mgmt
- 06_ci_cd_and_quality.md — CI, code quality gates, workflows
- 07_security_and_compliance.md — AuthN/Z, PII, logging, audits
- 08_release_plan.md — Environments, versioning, rollout, observability
- 09_open_questions.md — Decisions needed and assumptions

Editing guidelines:
- Keep sections concise and actionable
- Use checklists with owners and dates where possible
- Link to code paths and docs using relative paths

Milestones & timeline (tentative):
- M0: Planning finalized (this doc) [Week 0] ✅ COMPLETED
- M1: Monolith bootstrap + DB ready + Health endpoints [Week 1] ✅ COMPLETED
- M2: Auth + User MVP (API + mobile flows) [Weeks 2-3] ✅ COMPLETED
- M3: Product + Rental MVP [Weeks 4-6] ✅ COMPLETED
- M4: Messaging + Reviews (basic) [Weeks 7-8] ✅ COMPLETED
- M5: Payments + Subscription (sandbox) [Weeks 9-10] ✅ COMPLETED
- M6: CI/CD hardening, security pass, beta release [Weeks 11-12] ✅ COMPLETED

## Recent Progress

### Milestone 6: CI/CD Hardening, Security Pass, Beta Release ✅ COMPLETED
- **Comprehensive CI/CD Pipeline**: Complete GitHub Actions workflows for CI, CD, security, and Flutter
- **Security Hardening**: Security scanning, vulnerability assessment, and compliance checks
- **Docker Containerization**: Multi-stage Dockerfile with security best practices
- **Automated Testing**: Database validation, unit tests, integration tests, and code quality checks
- **Dependency Management**: Automated dependency updates with Dependabot
- **Security Policy**: Comprehensive security policy and vulnerability reporting process

**CI/CD Workflows Created:**
- `.github/workflows/ci.yml` - Main CI pipeline with database, Rust testing, and code quality
- `.github/workflows/cd.yml` - CD pipeline for staging and production deployments
- `.github/workflows/security.yml` - Security scanning and vulnerability assessment
- `.github/workflows/flutter.yml` - Flutter mobile app CI/CD (ready for future use)
- `.github/workflows/dependabot.yml` - Automated dependency management

**Security Features:**
- **CodeQL Analysis**: Static code analysis for security vulnerabilities
- **Trivy Container Scanning**: Container vulnerability scanning
- **Semgrep SAST**: Static application security testing
- **Cargo Audit**: Rust dependency vulnerability scanning
- **Secret Scanning**: Detection of hardcoded secrets and credentials
- **License Compliance**: Automated license compliance checking

**Infrastructure:**
- **Dockerfile**: Multi-stage build with security hardening
- **Dockerignore**: Optimized build context
- **SECURITY.md**: Comprehensive security policy
- **Dependabot Configuration**: Automated dependency updates

**Quality Gates:**
- **Code Formatting**: `cargo fmt` checks
- **Linting**: `cargo clippy` with strict warnings
- **File Size Limits**: 500-line limit enforcement
- **TODO/FIXME Detection**: Automated detection of pending work
- **Test Coverage**: Comprehensive test suite execution
- **Database Validation**: Schema validation and migration testing

**Deployment Features:**
- **Multi-Environment Support**: Staging and production deployments
- **Rollback Capability**: Manual rollback workflow
- **Health Checks**: Automated health check validation
- **Artifact Management**: Docker image and build artifact management
- **Environment Protection**: Protected environments with approval workflows

**Files Created/Modified:**
- `.github/workflows/ci.yml` - Main CI pipeline
- `.github/workflows/cd.yml` - CD pipeline
- `.github/workflows/security.yml` - Security scanning
- `.github/workflows/flutter.yml` - Flutter CI/CD
- `.github/workflows/dependabot.yml` - Dependency management
- `.github/dependabot.yml` - Dependabot configuration
- `Dockerfile` - Multi-stage container build
- `.dockerignore` - Docker build optimization
- `SECURITY.md` - Security policy and vulnerability reporting

**Key Features:**
- **Automated Testing**: Database setup, migrations, validation, and comprehensive test suites
- **Security Scanning**: Multiple layers of security scanning and vulnerability detection
- **Code Quality**: Automated code formatting, linting, and quality checks
- **Dependency Management**: Automated dependency updates with security review
- **Container Security**: Secure container builds with vulnerability scanning
- **Multi-Platform Support**: Support for Rust backend and Flutter mobile app
- **Environment Management**: Staging and production deployment workflows
- **Monitoring & Alerting**: Health checks and deployment verification

**Next Steps:** Project is now ready for production deployment and beta release

### Milestone 5: Payments + Subscription (sandbox) ✅ COMPLETED
- **Payment System**: Complete payment method management and payment intent processing
- **Subscription System**: Subscription plans, user subscriptions, and usage tracking
- **Database Schema**: Created payment_schema and subscription_schema with proper relationships
- **API Endpoints**: Comprehensive REST API for payment and subscription operations
- **Mock Integration**: Mock payment provider integration (Stripe/PayPal ready for real implementation)
- **Unit Tests**: Business logic validation tests for request/response models

**Payment Endpoints:**
- `POST /api/v1/payment/payment-methods` - Create payment method
- `GET /api/v1/payment/payment-methods` - List user's payment methods
- `POST /api/v1/payment/payment-intents` - Create payment intent
- `GET /api/v1/payment/payment-intents` - List user's payment intents
- `GET /api/v1/payment/payment-intents/:id` - Get payment intent details
- `POST /api/v1/payment/payment-intents/:id/confirm` - Confirm payment intent

**Subscription Endpoints:**
- `GET /api/v1/subscription/plans` - List available subscription plans
- `GET /api/v1/subscription/plans/:id` - Get subscription plan details
- `POST /api/v1/subscription/subscriptions` - Create subscription
- `GET /api/v1/subscription/subscriptions` - List user's subscriptions
- `GET /api/v1/subscription/subscriptions/:id` - Get subscription details
- `PUT /api/v1/subscription/subscriptions/:id/cancel` - Cancel subscription
- `GET /api/v1/subscription/subscriptions/:id/usage` - Get subscription usage

**Files Created/Modified:**
- `apps/monolith-server/src/payment.rs` - Payment handlers with payment method and intent management
- `apps/monolith-server/src/subscription.rs` - Subscription handlers for plans and user subscriptions
- `apps/monolith-server/src/routes/payment.rs` - Payment API routes
- `apps/monolith-server/src/routes/subscription.rs` - Subscription API routes
- `apps/monolith-server/tests/payment_tests.rs` - Payment business logic tests
- `apps/monolith-server/tests/subscription_tests.rs` - Subscription business logic tests
- `db/migrations/payment/20240405000001_create_payment_tables.up.sql` - Payment schema
- `db/migrations/payment/20240405000001_create_payment_tables.down.sql` - Payment rollback
- `db/migrations/subscription/20240405000001_create_subscription_tables.up.sql` - Subscription schema
- `db/migrations/subscription/20240405000001_create_subscription_tables.down.sql` - Subscription rollback
- `db/seed/initial_data.sql` - Updated with subscription plans seed data

**Key Features:**
- **Payment Methods**: Support for cards, bank accounts, and digital wallets
- **Payment Intents**: Secure payment processing with rental and subscription integration
- **Payment Transactions**: Complete transaction history and audit trail
- **Subscription Plans**: Tiered plans with different features and limits
- **Usage Tracking**: Monitor subscription usage (listings, rentals)
- **Billing Cycles**: Monthly and yearly billing support
- **Mock Providers**: Ready for Stripe/PayPal integration
- **Authorization**: Secure access control for all payment and subscription operations

### Milestone 4: Messaging + Reviews MVP ✅ COMPLETED
- **Messaging System**: Complete conversation and message management
- **Review System**: Product and user review functionality with ratings
- **Database Schema**: Created messaging_schema and review_schema with proper relationships
- **API Endpoints**: Comprehensive REST API for messaging and review operations
- **Authorization**: Secure access control for conversations and reviews
- **Unit Tests**: Business logic validation tests for request/response models

**Messaging Endpoints:**
- `POST /api/v1/messaging/conversations` - Create conversation with initial message
- `GET /api/v1/messaging/conversations` - List user's conversations
- `GET /api/v1/messaging/conversations/:id` - Get conversation details
- `POST /api/v1/messaging/conversations/:id/messages` - Send message
- `GET /api/v1/messaging/conversations/:id/messages` - List conversation messages
- `POST /api/v1/messaging/conversations/:id/read` - Mark messages as read

**Review Endpoints:**
- `POST /api/v1/review/product-reviews` - Create product review
- `POST /api/v1/review/user-reviews` - Create user review (renter/owner)
- `GET /api/v1/review/product-reviews` - List product reviews with filtering
- `GET /api/v1/review/products/:id/review-stats` - Get product review statistics

**Files Created/Modified:**
- `apps/monolith-server/src/messaging.rs` - Messaging handlers with conversation and message management
- `apps/monolith-server/src/review.rs` - Review handlers for products and users
- `apps/monolith-server/src/routes/messaging.rs` - Messaging API routes
- `apps/monolith-server/src/routes/review.rs` - Review API routes
- `apps/monolith-server/tests/messaging_tests.rs` - Messaging business logic tests
- `apps/monolith-server/tests/review_tests.rs` - Review business logic tests
- `db/migrations/messaging/20240405000001_create_messaging_tables.up.sql` - Messaging schema
- `db/migrations/messaging/20240405000001_create_messaging_tables.down.sql` - Messaging rollback
- `db/migrations/review/20240405000001_create_review_tables.up.sql` - Review schema
- `db/migrations/review/20240405000001_create_review_tables.down.sql` - Review rollback

**Key Features:**
- **Conversation Management**: Rental-based conversations between owners and renters
- **Message Types**: Support for text, image, file, and system messages
- **Read Status**: Track message read status for better UX
- **Review Types**: Product reviews and user reviews (renter/owner ratings)
- **Rating System**: 1-5 star rating system with validation
- **Verified Rentals**: Mark reviews from actual rentals as verified
- **Review Statistics**: Average ratings and rating distribution
- **Authorization**: Ensure users can only review relevant products/users

### Milestone 3: Product + Rental MVP ✅ COMPLETED
- **Product Management**: Complete CRUD operations for products with categories, tags, and images
- **Rental System**: Booking, availability checking, and rental lifecycle management
- **Category Management**: Hierarchical category system with parent-child relationships
- **Search & Filtering**: Advanced product search with price ranges, categories, and text search
- **Availability Checking**: Real-time availability validation with conflict detection
- **Authorization**: Product ownership validation and rental permissions
- **API Endpoints**: Comprehensive REST API for all product and rental operations
- **Unit Tests**: Business logic validation tests for request/response models

**Product Endpoints:**
- `POST /api/v1/products` - Create product
- `GET /api/v1/products` - List products with filtering
- `GET /api/v1/products/:id` - Get product details
- `PUT /api/v1/products/:id` - Update product
- `DELETE /api/v1/products/:id` - Soft delete product
- `POST /api/v1/categories` - Create category
- `GET /api/v1/categories` - List categories

**Rental Endpoints:**
- `POST /api/v1/rentals` - Create rental request
- `GET /api/v1/rentals` - List user's rentals (as renter or owner)
- `GET /api/v1/rentals/:id` - Get rental details
- `PUT /api/v1/rentals/:id` - Update rental status/notes
- `POST /api/v1/availability` - Check product availability

**Files Created/Modified:**
- `apps/monolith-server/src/product.rs` - Product management with CRUD operations
- `apps/monolith-server/src/rental.rs` - Rental system with booking and availability
- `apps/monolith-server/src/routes/product.rs` - Product API routes
- `apps/monolith-server/src/routes/rental.rs` - Rental API routes
- `apps/monolith-server/tests/product_tests.rs` - Product business logic tests
- `apps/monolith-server/tests/rental_tests.rs` - Rental business logic tests

**Key Features:**
- **Product Tags**: Dynamic tag system with automatic creation
- **Availability Logic**: Complex date range conflict detection
- **Ownership Validation**: Secure product and rental access control
- **Soft Deletes**: Products marked as deleted rather than removed
- **Pagination**: Efficient list endpoints with page/limit support
- **JSON Specifications**: Flexible product specifications and addresses

### Milestone 2: Auth + User MVP ✅ COMPLETED
- **JWT Authentication**: Implemented secure JWT token generation and validation
- **Password Hashing**: Added bcrypt password hashing with verification
- **Auth Endpoints**: Created signup, login, refresh, get_profile, update_profile endpoints
- **Database Integration**: Connected to existing user_schema.users and user_schema.user_profiles tables
- **Unit Tests**: Comprehensive JWT and password hashing tests (6 tests passing)
- **API Routes**: All auth endpoints wired under `/api/v1/auth/*`
- **Error Handling**: Proper HTTP status codes and error messages
- **Security**: Bearer token authentication for protected endpoints

**Files Created/Modified:**
- `apps/monolith-server/src/jwt.rs` - JWT utilities and password hashing
- `apps/monolith-server/src/auth.rs` - Auth handlers with database integration
- `apps/monolith-server/src/routes/auth.rs` - Auth router with all endpoints
- `apps/monolith-server/tests/jwt_tests.rs` - JWT and password hashing tests
- `apps/monolith-server/Cargo.toml` - Added JWT and bcrypt dependencies


