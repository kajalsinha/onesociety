### Security and Compliance

AuthN/Z:
- JWT access/refresh tokens; Argon2 password hashing
- Role model (user, admin); route guards in Axum layers

PII and Data security:
- Store PII within `user_schema`; restrict access paths
- Add audit triggers for critical tables into `audit_schema`
- TLS termination at gateway (in prod)

Secrets:
- Use env vars locally; secret manager for staging/prod

Compliance (future):
- GDPR basics: data export/delete endpoints, data retention policy
- Audit logs retention

Threat model (initial):
- Auth: credential stuffing → rate limiting; strong hashing; MFA later
- API: injection/XSS → input validation, typed queries (`sqlx`), output encoding
- Data: unauthorized access → role checks, schema separation, least privilege
- Infra: secrets exposure → env separation, no secrets in repo, vault later

Audit plan:
- Log auth events, data changes to `audit_schema`
- Retain audit logs per policy; redact sensitive fields


