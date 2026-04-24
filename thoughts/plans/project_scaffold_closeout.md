# Project Scaffold Closeout Implementation Plan

## Overview

Close out drift from the original project scaffold by restoring full Docker Compose app startup support and aligning README/ADR documentation with the current Rust-first application shape. The live scaffold is mostly intact, but container packaging, Compose runtime configuration, Tailwind documentation, and setup docs no longer match the codebase.

## Current State Analysis

The application still follows the original scaffold architecture: `main.rs` loads configuration, connects to PostgreSQL, runs SQLx migrations, composes shared state, starts background jobs, and serves the Axum router. Route assembly remains centralized in `routes.rs`, and feature modules still expose a `router()` function consumed by the top-level router.

The main scaffold gap is operational packaging. `docker-compose.yml` expects to build the app from the repository root, but there is no root `Dockerfile` or `.dockerignore`. Even with a Dockerfile, the current Compose app service only provides database, host, and port settings; the app also requires Google OAuth credentials and a football API key at startup.

Documentation has also drifted. The README still references a `SESSION_SECRET` that is not part of `Config`, omits `FOOTBALL_API_KEY`, omits `make js`, and does not list the `standings` module. ADR-0006, ADR-0007, and ADR-0012 still describe a Tailwind v3-era `tailwind.config.js` setup, while the live project uses Tailwind v4 CSS-first configuration in `assets/css/input.css`.

## Desired End State

The repository has a working, documented scaffold closeout:

- `docker compose up --build` can build the application image and start the app when `.env` contains the required app secrets.
- `docker compose up db -d` remains valid for local database-only development.
- README setup instructions match `Config`, `Makefile`, current modules, Tailwind v4, and vendored JS workflow.
- ADRs accurately describe the current accepted implementation without reintroducing obsolete Tailwind config files.
- Verification commands clearly separate automated checks from manual smoke tests.

### Key Discoveries:

- `src/main.rs:24-67` performs startup composition: tracing, config load, DB connect, migrations, TLS/session/auth/OAuth/football API setup, `AppState`, and background jobs.
- `src/routes.rs:6-16` keeps top-level route assembly limited to `/health`, module router merges, static assets, `TraceLayer`, and shared state.
- `src/config.rs:17-22` requires Google OAuth credentials and `FOOTBALL_API_KEY`, so Compose app startup must source these from `.env`.
- `docker-compose.yml:1-13` declares an app service with `build: .`, but no `Dockerfile` exists at the repository root.
- `assets/css/input.css:1-105` uses Tailwind v4 CSS-first configuration and `@theme`, not `tailwind.config.js`.
- `Makefile:23-29` already has `css` and `js` targets for generating CSS and vendoring HTMX/Alpine assets.
- `README.md:63` documents `SESSION_SECRET`, but `src/config.rs:10-32` has no such field.
- `README.md:185-189` lists feature modules but omits `standings`, which is registered in `src/modules/mod.rs:4-8`.
- `docs/adr/0020-client-side-javascript-strategy.md:75-83` already establishes the vendored JS pattern for HTMX and Alpine.

## What We're NOT Doing

- Not reintroducing `tailwind.config.js`; Tailwind v4 CSS-first setup is the desired state.
- Not changing application startup behavior, OAuth behavior, session behavior, or polling behavior.
- Not adding new feature modules or refactoring module boundaries.
- Not fixing the known `predictions` to `standings::db` boundary leak; it is architecture debt outside this scaffold closeout.
- Not committing real OAuth credentials, football API keys, TLS certificates, or `.env` contents.
- Not making Docker Compose independent of `.env`; the agreed approach is to read app secrets from `.env`.

## Implementation Approach

Use the smallest set of packaging and documentation changes that restores the original scaffold guarantee: the project can be built, run, and understood from a fresh checkout. Add a current Dockerfile pattern instead of reshaping Rust code. Keep Compose environment handling explicit: container-specific settings stay in Compose, secret app settings come from `.env`.

## Phase 1: Docker Packaging

### Overview

Add container packaging expected by `docker-compose.yml` and ADR-0012, updated for the current dependency stack and Tailwind v4 asset pipeline.

### Changes Required:

#### 1. Root Dockerfile
**File**: `Dockerfile`
**Changes**: Add a multi-stage build that compiles the Rust binary, builds static assets, and copies runtime files into a slim Debian runtime image.

Key requirements:

- Use a Rust builder stage compatible with edition 2024 and current dependencies.
- Use npm/package-lock for reproducible frontend tooling install.
- Run `npm ci`, `make js`, and `make css` or equivalent commands in the asset stage.
- Copy `templates/`, `migrations/`, and `assets/` into the runtime image.
- Run as a non-root user.
- Expose port `3000` and run `./fbapp-vibe`.

Example shape:

```dockerfile
FROM rust:1-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY templates ./templates
COPY migrations ./migrations
RUN cargo build --release

FROM node:20-slim AS assets
WORKDIR /app
COPY package.json package-lock.json ./
COPY assets ./assets
COPY templates ./templates
RUN npm ci
RUN npx @tailwindcss/cli -i assets/css/input.css -o assets/css/main.css --minify
RUN cp node_modules/htmx.org/dist/htmx.min.js assets/js/htmx.js \
    && cp node_modules/alpinejs/dist/cdn.min.js assets/js/alpine.js

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --shell /usr/sbin/nologin appuser
COPY --from=builder /app/target/release/fbapp-vibe ./fbapp-vibe
COPY --from=assets /app/assets ./assets
COPY templates ./templates
COPY migrations ./migrations
USER appuser
EXPOSE 3000
CMD ["./fbapp-vibe"]
```

The final implementation can simplify or adjust build caching, but must preserve these runtime artifacts.

#### 2. Docker Ignore File
**File**: `.dockerignore`
**Changes**: Add build-context exclusions for local-only and heavyweight files.

Include at minimum:

```dockerignore
target/
node_modules/
.git/
.env
.env.local
certs/
.agents/
.opencode/
```

Do not ignore `Cargo.lock` or `package-lock.json`; Docker builds should use the lockfiles.

### Success Criteria:

#### Automated Verification:
- [x] Docker build succeeds: `docker compose build app`
- [x] Compose config validates: `docker compose config`
- [x] Rust release build still succeeds locally: `make build`

#### Manual Verification:
- [x] Runtime image contains `fbapp-vibe`, `assets/`, `templates/`, and `migrations/`.
- [x] No `.env`, `node_modules/`, local certs, or git metadata are copied into the image.

---

## Phase 2: Compose Runtime Wiring

### Overview

Update Compose so the app service gets required secrets from `.env` while retaining container-specific defaults for database networking and bind address.

### Changes Required:

#### 1. App Service Environment
**File**: `docker-compose.yml`
**Changes**: Add `.env` loading and pass required app settings into the container.

Recommended approach:

```yaml
services:
  app:
    build: .
    env_file:
      - .env
    environment:
      DATABASE_URL: postgres://fbapp:fbapp@db:5432/fbapp
      HOST: 0.0.0.0
      PORT: 3000
```

This keeps the Docker network database URL correct while allowing `GOOGLE_CLIENT_ID`, `GOOGLE_CLIENT_SECRET`, `GOOGLE_REDIRECT_URL`, `FOOTBALL_API_KEY`, optional TLS settings, and optional OTLP settings to come from `.env`.

#### 2. Optional Healthcheck
**File**: `docker-compose.yml`
**Changes**: Consider adding an app healthcheck against `/health` if the runtime image includes a lightweight HTTP client.

If adding a healthcheck requires installing extra packages only for `curl`/`wget`, prefer documenting the manual smoke test instead of increasing the runtime image surface area.

### Success Criteria:

#### Automated Verification:
- [x] Compose config validates: `docker compose config`
- [x] Database-only workflow still validates: `docker compose up db -d`

#### Manual Verification:
- [x] With `.env` populated from `.env.example`, `docker compose up --build` starts `db` and `app`.
- [x] `GET http://localhost:3000/health` returns `200 OK` from the containerized app.
- [x] Missing required `.env` values fail startup with a clear config error rather than silently running misconfigured.

---

## Phase 3: Documentation Alignment

### Overview

Update README and setup docs so a new developer can run the app using the current configuration, asset, and Compose workflows.

### Changes Required:

#### 1. Environment Setup
**File**: `README.md`
**Changes**: Replace stale `SESSION_SECRET` guidance with current required variables.

Document:

- `DATABASE_URL`
- `TEST_DATABASE_URL` for tests only
- `HOST`
- `PORT`
- `GOOGLE_CLIENT_ID`
- `GOOGLE_CLIENT_SECRET`
- `GOOGLE_REDIRECT_URL`
- `FOOTBALL_API_KEY`
- Optional TLS paths
- Optional polling/session duration settings
- Optional `OTEL_EXPORTER_OTLP_ENDPOINT`

Remove the `SESSION_SECRET` generation section unless a future code change reintroduces that config field.

#### 2. Local Development Flow
**File**: `README.md`
**Changes**: Clarify the two supported local workflows.

Document database-only development:

```bash
docker compose up db -d
npm install
make js
make css
cargo run
```

Document full Compose startup:

```bash
cp .env.example .env
# fill in OAuth and football API values
docker compose up --build
```

#### 3. Make Targets
**File**: `README.md`
**Changes**: Add `make js` to the Make target table and explain it vendors HTMX/Alpine into `assets/js/`.

#### 4. Project Structure
**File**: `README.md`
**Changes**: Update the structure section to include:

- `assets/js/`
- `src/tracing_setup.rs`
- `src/football_api.rs`
- `src/polling/`
- `src/session_cleanup.rs`
- `src/modules/standings/`
- Root `Dockerfile`
- Root `.dockerignore`

#### 5. Environment Example
**File**: `.env.example`
**Changes**: Keep the current values but ensure comments align with README and Compose usage.

If necessary, add a comment that full `docker compose up --build` reads this file and overrides `DATABASE_URL`, `HOST`, and `PORT` in Compose for container networking.

### Success Criteria:

#### Automated Verification:
- [x] README references only config keys that exist in `src/config.rs` or are explicitly test-only, such as `TEST_DATABASE_URL`.
- [x] README Make target list includes every target in `Makefile:1-29`.

#### Manual Verification:
- [x] A new developer can distinguish `docker compose up db -d` from full `docker compose up --build`.
- [x] README setup steps mention `make js` before browser usage that requires HTMX/Alpine.
- [x] README project structure matches current module registry in `src/modules/mod.rs:4-8`.

---

## Phase 4: ADR Drift Notes

### Overview

Align architecture documentation with current accepted implementation while preserving historical context.

### Changes Required:

#### 1. Tailwind ADR
**File**: `docs/adr/0006-use-tailwind-css-for-styling.md`
**Changes**: Add an amendment noting that Tailwind has moved to v4 CSS-first configuration.

Document:

- Design tokens live in `assets/css/input.css` under `@theme`.
- The project uses `@tailwindcss/cli` from npm.
- A root `tailwind.config.js` is no longer required for the current setup.
- `assets/css/main.css` remains compiled output and is still served by Axum.

#### 2. Project Structure ADR
**File**: `docs/adr/0007-project-structure-modular-monolith.md`
**Changes**: Add a short amendment or update the layout to include current shared modules and `assets/js/`.

Document that feature modules currently include `auth`, `admin`, `leagues`, `predictions`, and `standings`, all registered in `src/modules/mod.rs`.

#### 3. Docker ADR
**File**: `docs/adr/0012-deployment-with-docker.md`
**Changes**: Update the Dockerfile pattern to match Tailwind v4 and current asset vendoring.

Document:

- `package-lock.json` is used for npm reproducibility.
- CSS build uses `npx @tailwindcss/cli -i assets/css/input.css -o assets/css/main.css --minify`.
- JS assets are vendored from `node_modules` into `assets/js/`.
- Compose app service reads `.env` for application secrets.

### Success Criteria:

#### Automated Verification:
- [x] ADRs no longer require `tailwind.config.js` as current implementation guidance.
- [x] ADR-0012 Dockerfile pattern does not reference missing files.

#### Manual Verification:
- [x] ADRs clearly distinguish original decisions from later implementation amendments.
- [x] Documentation does not imply secrets are committed or baked into images.

---

## Phase 5: Verification Closeout

### Overview

Run the repo's normal validation loop plus Docker/asset smoke checks.

### Changes Required:

#### 1. Local Validation
**Files**: No code changes required.
**Changes**: Run the commands and capture results in the task outcome.

#### 2. Ticket Outcome
**File**: `thoughts/tickets/project-scaffold.md`
**Changes**: After implementation, update `## Outcome` with the Docker/docs closeout result and any verification caveats.

### Success Criteria:

#### Automated Verification:
- [x] Formatting and clippy pass: `make lint`
- [x] Rust tests pass: `make test`
- [x] CSS builds: `make css`
- [x] JS vendoring runs: `make js`
- [x] Docker Compose config validates: `docker compose config`
- [x] App image builds: `docker compose build app`

#### Manual Verification:
- [x] With valid `.env`, `docker compose up --build` starts the full stack.
- [x] `GET http://localhost:3000/health` returns `200 OK`.
- [x] Browser loads `/assets/css/main.css`, `/assets/js/htmx.js`, and `/assets/js/alpine.js` from the containerized app.
- [x] README instructions are followed once from a clean-ish checkout without discovering undocumented setup steps.

---

## Testing Strategy

### Unit Tests:

- No new Rust unit tests are required because the implementation is packaging and documentation focused.
- Existing unit tests in `src/error.rs` and module tests should continue to pass under `make test`.

### Integration Tests:

- Existing SQLx-backed tests continue to run with `TEST_DATABASE_URL` configured.
- Docker smoke testing covers the scaffold integration path: app image, migrations at startup, static assets, and `/health`.

### Manual Testing Steps:

1. Copy `.env.example` to `.env` and fill OAuth/API placeholders with development-safe values.
2. Run `docker compose up --build`.
3. Request `http://localhost:3000/health` and confirm `200 OK`.
4. Open `http://localhost:3000/` and confirm CSS and JS assets load without 404s.
5. Stop the stack and confirm database-only workflow still works with `docker compose up db -d` followed by `cargo run`.

## Performance Considerations

Docker image build time will increase because the image compiles Rust and builds frontend assets. Use multi-stage builds and lockfile-based dependency layers to preserve as much cache reuse as practical. The runtime image should not include Rust, Node, `node_modules`, or build caches.

## Migration Notes

No database migration is required. The app already runs SQLx migrations at startup through `src/main.rs:29`. Existing local `.env` files may need to add `FOOTBALL_API_KEY` if they were created from older README instructions.

## Deviations from Plan

### Phase 1: Docker Packaging
- **Original Plan**: Add a multi-stage Dockerfile that runs `cargo build --release` in a Rust builder stage.
- **Actual Implementation**: The builder stage installs PostgreSQL, starts a temporary local build database, applies migrations, and runs `cargo build --release` with `DATABASE_URL` pointed at that temporary database.
- **Reason for Deviation**: The repository uses SQLx compile-time query macros and does not commit a `.sqlx` offline cache. A fresh Docker build therefore needs a live database for compile-time query validation, but application secrets and `.env` must not be copied into the image.
- **Impact Assessment**: Runtime image contents and Compose behavior are unchanged. Docker builds are slower because PostgreSQL is installed only in the discarded builder stage, but `docker compose build app` works from a fresh checkout without baking secrets into the image.
- **Date/Time**: 2026-04-24 09:22 EEST

### Phase 2: Compose Runtime Wiring
- **Original Plan**: Read required app secrets from `.env` and keep container-specific settings in Compose.
- **Actual Implementation**: Added `env_file: .env` and mounted `./certs` to `/app/certs:ro` for optional TLS files referenced by `.env`.
- **Reason for Deviation**: Local TLS certs are intentionally excluded from Docker images, but a valid `.env` may include `TLS_CERT_PATH` and `TLS_KEY_PATH`. The runtime mount lets optional TLS work in Compose without copying certificates into the image.
- **Impact Assessment**: Database-only workflow is unchanged. Full Compose startup supports both plain HTTP when TLS env vars are unset and HTTPS when local cert files exist. Secrets and certs remain outside the image.
- **Date/Time**: 2026-04-24 09:28 EEST

## References

- Original ticket: `thoughts/tickets/project-scaffold.md`
- Related research: `thoughts/research/2026-04-24_project_scaffold.md`
- Startup wiring: `src/main.rs:24-67`
- Router assembly: `src/routes.rs:6-16`
- Config requirements: `src/config.rs:10-32`
- Shared state: `src/state.rs:11-18`
- Feature module registry: `src/modules/mod.rs:4-8`
- Tailwind v4 input: `assets/css/input.css:1-105`
- Asset/JS build targets: `Makefile:23-29`
- Compose services: `docker-compose.yml:1-44`
- Environment example: `.env.example:1-24`
- README drift: `README.md:46-66`, `README.md:155-194`
- Tailwind ADR: `docs/adr/0006-use-tailwind-css-for-styling.md`
- Project structure ADR: `docs/adr/0007-project-structure-modular-monolith.md`
- Docker ADR: `docs/adr/0012-deployment-with-docker.md`
- JS strategy ADR: `docs/adr/0020-client-side-javascript-strategy.md`
