### Open Questions / Decisions Needed

API and Contracts:
- Confirm API envelope `{ data, error }` and error taxonomy
- Decide on path for OpenAPI authoring (single file vs per-domain merged)

Monolith scope:
- Confirm MVP domains (auth, user, product, rental) vs defer (payments, messaging, reviews, subscription)
- Do we include search (Elasticsearch) in MVP or stub it?

Data & Compliance:
- PII boundaries and anonymization strategy
- Retention and deletion policies

Mobile:
- Navigation and state management details; design system tokens
- Offline caching scope (if any)

Infra:
- Target runtime for staging/prod (Docker vs K8s) and timelines
- Logging/monitoring stack in staging from day 1?

Owners and dates (proposed):
- API envelope and error taxonomy — Owner: Backend lead — Due: M1
- MVP domain scope confirmation — Owner: PM — Due: M1
- PII boundaries and retention — Owner: Security/Backend — Due: M2
- Staging runtime decision — Owner: DevOps — Due: M2


