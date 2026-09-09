# Health Travel Aggregator

> A curated marketplace for medical tourism — clinics, treatments, packages, and verified reviews.
>
> **Domain:** `health-travel.lucanian.app`
> **Status:** Early development. Bootstrapped from platform-templates.

The Health Travel Aggregator connects patients with accredited clinics and treatment packages across borders. Providers publish clinics, treatments, and all-inclusive packages; patients search, compare, and request quotes; admins moderate listings and reviews.

---

## Stack

- **Backend:** Rust workspace (`service/`)
  - `crates/chassis` — domain models, storage (Postgres via `sqlx`), and shared service logic
  - `crates/server` — HTTP API server built with `axum` + `tower-http`
- **Frontend:** Nuxt 4 (`web/`) with Vue 3, Tailwind CSS, and Vite+ — installable as a PWA (web app manifest + service worker; public pages work offline, authenticated surfaces stay network-only)
- **Database:** PostgreSQL 15
- **E2E Testing:** Playwright
- **Deployment:** systemd + Caddy

## Quick Start

```bash
# 1. Clone / enter the repository
cd /root/health-travel-aggregator

# 2. Configure environment (Postgres creds, session key, CORS origin)
cp .env.example .env

# 3. Start Postgres (or point DATABASE_URL at your own instance)
docker compose up -d postgres

# 4. Build and run the Rust API server
cd service
cargo run --bin server

# 5. In another terminal, install and run the web app
cd web
npm ci
npm run dev
```

The API server starts on the configured port (see `service/crates/chassis` config), and the Nuxt dev server serves the frontend.

## Build Verification

```bash
# Backend release build
cd service && cargo build --release

# Frontend install + static generation
cd web && npm ci && npm run build
```

## Development Commands

```bash
# Service checks
cd service && cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Web checks
cd web && npm run check && npm run test

# E2E tests (requires Chromium browser, a seeded API on :8080, and the web
# preview on :3000 — see the e2e job in .github/workflows/ci.yml)
cd web && npx playwright install chromium && npm run test:e2e
```

### Browser → API origin requirements

The API enforces a CSRF origin guard and a restrictive CORS allowlist. When the
frontend and API run on different origins (e.g. Nuxt dev/preview on
`http://localhost:3000` and the API on `http://localhost:8080`), the API must
be started with `CORS_ORIGIN=http://localhost:3000` (and `APP_URL` must match
the frontend origin — its default already is `http://localhost:3000`).
Authenticated requests use the `session` cookie; the web app always calls the
API with `credentials: 'include'`.

## CI

GitHub Actions are defined in `.github/workflows/ci.yml`.

Local CI verification with `act`:

```bash
act --container-daemon-socket /run/podman/podman.sock
```

## Project Layout

```
health-travel-aggregator/
├── collectors/          # Future data collectors for clinic/provider feeds
├── service/             # Rust workspace
│   ├── crates/chassis/  # Domain, storage, config
│   └── crates/server/   # HTTP API binary
├── web/                 # Nuxt 4 frontend
│   ├── pages/           # Route pages
│   ├── components/      # Vue components
│   └── tests/           # Unit + e2e tests
└── .github/workflows/   # CI/CD pipelines
```
