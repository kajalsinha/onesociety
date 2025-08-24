### Database Plan (Single Postgres, multi-schema)

Current:
- Postgres 15 + PostGIS via Docker Compose
- Per-domain schemas: `*_schema`; service-specific migrations
- Validation suite in `db/sql/validate_schema.sql`; runner `db/bin/validate.sh`

Plan:
- Finalize base schema in `db/schemas/onesociety.sql` (moved here; ensure present)
- Enforce migration format (single file with Up/Down sections)
- Add foreign key indices and RLS where appropriate
- Seed data via `db/seed/initial_data.sql`

Conventions:
- snake_case names; `uuid` PKs; `created_at/updated_at`; soft-deletes where needed.
- FKs `ON DELETE CASCADE` thoughtfully; indexes on FK and query columns.
- Comment tables/columns with purpose and data classification (PII, sensitive).

Testing:
- Migration verification in CI; `make db-validate` in pre-commit.
- Per-domain repo tests covering CRUD and constraints.

Backups & retention (later stage):
- Nightly logical dumps; weekly base + daily incrementals (to be scripted).
- Retention policy aligned with compliance requirements.

Operational:
- Use `make db-docker-setup` to provision local DB
- Run `make db-setup`, `make db-new`, `make db-migrate service=...`, `make db-validate`
- Pre-commit hook to run `db-validate`

Data lifecycle & PII:
- Separate PII to `user_schema` tables; document access patterns
- Add audit trail to `audit_schema` for critical mutations
- Backups and restore procedure (to be scripted later)


