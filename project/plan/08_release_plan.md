### Release Plan

Environments:
- Local: docker-compose infra, monolith runs locally
- Staging: to be defined (Docker/K8s); single DB instance
- Production: to be defined; single DB initially, later split possible

Versioning:
- Backend: semantic versioning; API version path `/api/v1`
- Mobile: store versioning scheme; feature flags as needed

Rollout:
- Tag and build artifacts; changelog per release
- Smoke tests + basic load checks

Release checklist (MVP):
- [ ] All milestone endpoints implemented and documented
- [ ] DB migrations applied/validated; backup snapshot taken
- [ ] CI green; quality thresholds met
- [ ] Monitoring dashboards created; alerts configured
- [ ] Security review completed

Rollback strategy:
- Immutable artifacts; deploy previous tag on failure
- DB: roll back last migration set using `make db-rollback` (domain scope)

Observability:
- Metrics via Prometheus, dashboards in Grafana
- Structured logs with tracing correlation IDs


