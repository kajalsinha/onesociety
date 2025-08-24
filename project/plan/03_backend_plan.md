### Backend Plan (Rust Monolith)

Milestone 1: Monolith bootstrap
- Create crate `apps/monolith-server` (Axum)
- Add health endpoints: `/healthz`, `/readyz`
- Wire logging (tracing), config loader, graceful shutdown
- Connect to Postgres via `sqlx` with `DATABASE_URL`

Milestone 2: Auth & User domain
- Define OpenAPI spec for auth/user
- Implement endpoints: signup, login, refresh, get-profile, update-profile
- Repos: users, user_profiles; Argon2 hashing; JWT issuance
- Tests: unit (handlers, repos), integration (HTTP + DB)

Milestone 3: Product & Rental domain
- Product CRUD, categories, tags; search scaffolding (later ES integration)
- Rental booking flow (availability, create booking, cancel)
- Validation and transactional logic

Milestone 4: Payments, Messaging, Reviews, Subscription (scoped MVP)
- Payment intent create/capture (mock or Stripe sandbox)
- Messaging threads/messages basic endpoints
- Reviews create/list
- Subscription plans list/purchase (mock)

Cross-cutting:
- Error model, response envelope, pagination
- Middleware: auth, request-id, rate limiting (basic)
- Observability: metrics and structured logs

Deliverables per milestone:
- OpenAPI spec updated + generated Dart models
- Endpoints + tests green
- Migrations + validation pass

API contracts plan:
- Author OpenAPI spec per domain under `docs/api/` and merge into a single `/api/v1` spec.
- Validate with CI; generate Dart clients via `openapi-generator` into Flutter app.
- Keep response envelope `{ data, error }` and consistent error codes.

Definition of Done (DoD):
- Endpoint covered by unit + integration tests; OpenAPI documented.
- Lints pass; logs and metrics present for the new code paths.
- DB migrations applied locally and validated; rollbacks verified.


