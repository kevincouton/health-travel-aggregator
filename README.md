# Health Travel Aggregator

> A curated marketplace for medical tourism — clinics, treatments, packages, and verified reviews.
>
> **Domain:** `health-travel.lucanian.app`
> **Status:** Early development. Bootstrapped from platform-templates.

The Health Travel Aggregator connects patients with accredited clinics and treatment packages across borders. Providers publish clinics, treatments, and all-inclusive packages; patients search, compare, and request quotes; admins moderate listings and reviews.

---

## Stack

- **Backend:** Rust workspace (`service/`)
  - `crates/chassis` — domain models, storage, and shared service logic (SQLite via `rusqlite`)
  - `crates/server` — HTTP API server built with `topcoat`
- **Frontend:** Nuxt 4 (`web/`) with Vue 3, Tailwind CSS, and Vite+
- **Database:** SQLite
- **E2E Testing:** Playwright
- **Deployment:** systemd + Caddy

## Quick Start

```bash
# 1. Clone / enter the repository
cd /root/health-travel-aggregator

# 2. Build and run the Rust API server
cd service
cargo run --bin server

# 3. In another terminal, install and run the web app
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

# E2E tests (requires Chromium browser)
cd web && npx playwright install chromium && npm run test:e2e
```

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
