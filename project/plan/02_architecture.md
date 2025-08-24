### Target Architecture (Monolith-first, microservice-ready)

Principles:
- Single binary (Rust, Axum) exposing `/api/v1/*`.
- Domain modules: `auth`, `user`, `product`, `rental`, `payment`, `messaging`, `review`, `subscription`.
- Single PostgreSQL database; per-domain schemas with migration tracking.
- Strict boundaries via crates/modules and interfaces in `libs/common`/`libs/models`.

Components:
- Monolith Server: `apps/monolith-server` (to be created)
  - Router composition: mounts each domain router under `/api/v1/<domain>`
  - Middlewares: logging, tracing, auth (JWT), request-id, CORS
  - Observability: Prometheus metrics, healthz/readyz
- Database: Postgres 15 + PostGIS; managed via `db/scripts/manage_db.sh` and `db/bin/*`
- Cache & Messaging (optional now): Redis, RabbitMQ; integrated later as needed
- Search: Elasticsearch (optional; used by product/search services later)

Environments:
- Local: Docker Compose infra; server runs locally
- Staging/Prod: to be defined in `deploy/` (K8s later)

API Contracts:
- OpenAPI 3 for `/api/v1/*`; generated models for Flutter via `openapi-generator`
- Response envelope: `{ data: T, error: string | null }`

Security:
- JWT access/refresh tokens; password hashing via Argon2
- RLS-enabled tables where applicable in future iterations

Dependency map (initial):
- Monolith → PostgreSQL (required), Redis (optional), RabbitMQ (optional), Elasticsearch (optional)
- Flutter app → Monolith HTTP API
- Migrations/Validation → PostgreSQL

Non-functional requirements (NFRs):
- Performance: P50 < 150ms, P95 < 500ms for common endpoints in dev; load targets to be set for staging.
- Reliability: Graceful shutdown, readiness checks; DB connection pool health monitoring.
- Security: Secrets not in repo; TLS in prod; minimal scopes for service accounts.
- Observability: Request tracing with IDs; app and DB metrics; structured logs.


