# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of AgTerm seriously. If you have discovered a security vulnerability, please report it to us privately.

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please send an email to:
- security@coldwoong-moon.dev (preferred)
- Or create a private security advisory via GitHub Security Advisories

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

## Security Update Process

1. The security team will acknowledge receipt of your vulnerability report
2. We will investigate and confirm the vulnerability
3. We will develop and test a fix
4. We will release a security update
5. We will publicly disclose the vulnerability after the fix is deployed

## Security-related Configuration

AgTerm follows security best practices:

- All releases are built with security hardening flags
- Dependencies are automatically scanned for known vulnerabilities using `cargo-audit`
- Regular security audits are performed on dependencies
- Binary artifacts are cryptographically signed (checksums provided)

## Dependencies

We regularly monitor and update our dependencies to address known vulnerabilities. You can view our dependency audit status in our CI/CD pipeline.

## Bug Bounty Program

We currently do not offer a bug bounty program. However, we greatly appreciate security researchers who report vulnerabilities responsibly and we will publicly acknowledge your contribution (with your permission) when we release a fix.
