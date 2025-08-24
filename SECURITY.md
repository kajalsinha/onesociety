# Security Policy

## Supported Versions

We release patches to fix security vulnerabilities. Which versions are eligible for receiving such patches depends on the CVSS v3.0 Rating:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of OneSociety seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to [security@onesociety.com](mailto:security@onesociety.com).

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the requested information listed below (as much as you can provide) to help us better understand the nature and scope of the possible issue:

- Type of issue (buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the vulnerability
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

This information will help us triage your report more quickly.

## Preferred Languages

We prefer all communications to be in English.

## Policy

OneSociety follows the principle of [Responsible Disclosure](https://en.wikipedia.org/wiki/Responsible_disclosure).

## Security Best Practices

### For Developers

1. **Never commit secrets** - API keys, passwords, tokens, etc.
2. **Use environment variables** for configuration
3. **Validate all inputs** - especially user-provided data
4. **Use prepared statements** for database queries
5. **Keep dependencies updated** - run `cargo audit` regularly
6. **Follow the principle of least privilege** - minimal permissions
7. **Log security events** - authentication, authorization failures
8. **Use HTTPS** in production
9. **Implement rate limiting** on API endpoints
10. **Regular security reviews** of code changes

### For Users

1. **Keep your system updated** with the latest patches
2. **Use strong, unique passwords** for your accounts
3. **Enable two-factor authentication** when available
4. **Be cautious with API keys** - rotate them regularly
5. **Monitor your account** for suspicious activity
6. **Report security issues** promptly

## Security Features

OneSociety includes several security features:

- **JWT-based authentication** with secure token handling
- **Password hashing** using bcrypt
- **SQL injection protection** via sqlx prepared statements
- **CORS protection** for API endpoints
- **Rate limiting** on authentication endpoints
- **Audit logging** for security-relevant events
- **Input validation** on all API endpoints
- **Secure headers** in HTTP responses

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported versions
4. Release new versions with the fixes
5. Publicly announce the vulnerability

## Credits

We would like to thank all security researchers and users who have responsibly disclosed vulnerabilities to us.

## Contact

- **Security Email**: [security@onesociety.com](mailto:security@onesociety.com)
- **Security Team**: OneSociety Security Team

## Bug Bounty

We currently do not offer a formal bug bounty program, but we do appreciate and acknowledge security researchers who responsibly disclose vulnerabilities to us.
