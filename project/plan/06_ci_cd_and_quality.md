### CI/CD and Quality Gates

Pipelines (to add under `tools/ci/`):
- Lint: `cargo clippy --workspace -- -D warnings`, `flutter analyze`
- Format: `cargo fmt --all -- --check`, `dart format --set-exit-if-changed`
- Test: `cargo test --workspace`, `flutter test`
- DB: `make db-validate`
- Build: monolith server binary + Flutter build

Quality thresholds:
- Lints: zero warnings
- Tests: >= 80% unit test coverage on core logic (where measurable)
- API: OpenAPI validation must pass; breaking change check
- DB: Validation no failures; migrations must apply/rollback in CI

Branch policy:
- PRs require green CI, code review, and test coverage targets

Pre-commit (local):
- Run lint, format check, and db-validate

Artifacts:
- Docker image for monolith (later), Flutter APK/IPA artifacts


