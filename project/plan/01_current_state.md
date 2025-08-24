### Current State (Monolith-first with single PostgreSQL DB, Flutter frontend)

Repo overview:
- Backend: multiple Rust crates under `apps/` (api-gateway, domain services). Monolith not yet scaffolded.
- Shared libs: `libs/common`, `libs/models`, `libs/proto` present.
- Database: Single Postgres with per-domain schemas; migrations under `db/migrations/*`.
- Tooling: Makefile targets for db setup/migrate/validate; Docker compose for infra; DB manage script.
- Frontend: Flutter app in `apps/mobile-app` (libs structure to be reviewed/filled).

Implemented (evidence):
- Single DB setup using PostGIS image in `docker-compose.yml`.
- DB migration and validation system (`db/bin/*`, `db/sql/validate_schema.sql`).
- Service-scoped migration dirs: `db/migrations/{auth,user,product,...}`.
- Infra services available: Postgres, Redis, RabbitMQ, Elasticsearch, Prometheus, Grafana.
- Makefile orchestration for dev/build/test and DB lifecycle.

Unimplemented / gaps:
- Monolith server crate (Axum) that aggregates all domains.
- API OpenAPI specs and generation workflow (as per `.cursor/rules/api_contracts.json`).
- Actual domain handlers, repos, and service wiring in Rust crates (placeholders likely).
- CI pipeline files (referenced by rules but not present under `tools/ci` yet).
- Flutter app API integration scaffolding (dio, models generation), feature modules.
- End-to-end tests invoking HTTP against a running server.

Constraints/decisions:
- For now: single Postgres DB, multi-schema; single monolith backend binary; Flutter mobile frontend.
- Prepare clean boundaries per domain for future extraction.

Risks:
- Divergence between planned microservices and current monolith if boundaries aren’t enforced.
- DB migrations consistency across domains when multiple developers contribute.
- Lack of OpenAPI-first may slow mobile integration and regress typing.

Assumptions:
- Single PostgreSQL instance serves all domains for MVP.
- Flutter is the only client in scope for MVP (web later).
- Auth uses email/password first; social login later.

Out of scope (MVP):
- Production-grade K8s deployment (basic Docker deploy acceptable initially).
- Full-text search relevance tuning; advanced analytics.
- Complex payment methods beyond card + sandbox.

Mitigations:
- Enforce domain boundaries in code layout and DB schemas.
- Pre-commit validation for migrations; Makefile workflows.
- Adopt OpenAPI early to enable typed mobile clients.


