# Security Policy

## Reporting a Vulnerability

Please report vulnerabilities privately via **GitHub's private vulnerability
reporting** (the "Report a vulnerability" button under this repository's
Security tab), which opens a private security advisory with the maintainer.

Please do **not** open a public issue for a suspected vulnerability.

## Scope

This repository contains a medical-tourism marketplace:

- `service/` — Rust (axum) API server and domain/storage crates
- `collectors/` — Rust data-collection crate
- `web/` — Nuxt 4 frontend

Reports are welcome for anything in this codebase that could compromise
confidentiality, integrity, or availability — for example authentication or
session-handling flaws, injection, payment/webhook verification issues
(Stripe), secret leakage, or insecure dependency usage. Issues in third-party
dependencies should also be reported to the upstream project.

## Response Expectations

This is a solo-maintainer project. You can expect an acknowledgement within a
few days and a good-faith effort to assess and remediate verified reports,
with credit in the advisory if you wish.
