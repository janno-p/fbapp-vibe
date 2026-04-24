---
title: Project scaffold
source: .claude/tasks/done/0001-project-scaffold.md
source_id: 0001
source_status: done
source_title: Project scaffold
status: implemented
type: chore
adrs: [0001, 0002, 0003, 0004, 0005, 0006, 0007, 0008, 0009, 0010, 0012]
refs: []
created: 2026-04-05
started: 2026-04-06
completed: 2026-04-06
---

## Summary

Bootstrap the project from an empty repository into a compiling, runnable Rust application that implements the full structural skeleton defined in the ADRs. No business logic or features are included — the outcome is a working foundation that every subsequent task builds upon.

## Acceptance Criteria

- [ ] `cargo build` succeeds with zero warnings and zero clippy errors
- [ ] `cargo run` starts an HTTP server on the configured port and responds to `GET /health` with `200 OK`
- [ ] The module structure matches ADR-0007 exactly
- [ ] All dependencies from ADRs 0002–0010 are declared in `Cargo.toml` with pinned minor versions
- [ ] `AppState` is wired into the Axum router via `State`
- [ ] `Config` loads from environment variables via `dotenvy` + `envy`; missing required vars exit the process with a clear error
- [ ] `AppError` implements `IntoResponse` and is the return type of all handlers
- [ ] `tracing-subscriber` is initialised in `main.rs`; `TraceLayer` is applied to the router
- [ ] SQLx migrations directory exists with an initial migration placeholder
- [ ] Tailwind config exists and `assets/css/input.css` compiles to `assets/css/main.css` via `npx tailwindcss`
- [ ] Askama base layout template exists at `templates/layout/base.html`
- [ ] `docker-compose.yml` starts PostgreSQL and the app; `docker compose up` reaches the health endpoint
- [ ] `.env.example` documents all required environment variables
- [ ] `Makefile` has targets: `dev`, `build`, `lint`, `test`, `migrate`, `css`

## Implementation Context

### Relevant files to create

Following ADR-0007, create the full directory and file layout from scratch:

```
fbapp-vibe/
├── migrations/
│   └── 0001_initial.sql          # empty placeholder
├── templates/
│   └── layout/
│       └── base.html             # Askama base layout
├── assets/
│   └── css/
│       └── input.css             # Tailwind entry point (@tailwind directives)
├── src/
│   ├── main.rs                   # startup, tracing init, server bind
│   ├── config.rs                 # Config struct via envy
│   ├── error.rs                  # AppError + IntoResponse
│   ├── state.rs                  # AppState (PgPool + Config)
│   ├── routes.rs                 # top-level router assembly
│   └── modules/
│       └── mod.rs                # empty, ready for first feature module
├── .env.example
├── .gitignore
├── docker-compose.yml
├── tailwind.config.js
├── Makefile
└── Cargo.toml
```

### ADR constraints

- **ADR-0001**: Rust (see ADR-0021 for edition — currently edition 2024)
- **ADR-0002**: `axum` is the HTTP framework; no other HTTP crate used directly
- **ADR-0005**: `sqlx` with features `postgres`, `runtime-tokio-rustls`, `macros`; `PgPool` in `AppState`; run migrations at startup via `sqlx::migrate!()`
- **ADR-0007**: `routes.rs` only calls `module::router()` — no handler logic; module `mod.rs` re-exports only the public API
- **ADR-0008**: `Config::load()` calls `dotenvy::dotenv().ok()` then `envy::from_env()`; `Config` is stored in `AppState`
- **ADR-0009**: `AppError` uses `thiserror`; implements `IntoResponse`; all handlers return `Result<impl IntoResponse, AppError>`; `clippy::unwrap_used` lint enabled in `Cargo.toml`
- **ADR-0010**: `tracing_subscriber` initialised once in `main.rs` with `EnvFilter`; `TraceLayer::new_for_http()` applied on the router
- **ADR-0012**: `sqlx::migrate!().run(&pool).await` called before `axum::serve` in `main.rs`

### Cargo.toml dependencies

```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace", "fs"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls", "macros"] }
askama = "0.12"
askama_axum = "0.4"
serde = { version = "1", features = ["derive"] }
dotenvy = "0.15"
envy = "0.4"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1"
anyhow = "1"

[lints.clippy]
unwrap_used = "deny"
```

### Config struct

```rust
#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> u16 { 3000 }
```

### Health check handler

Add a `GET /health` route directly in `routes.rs` (not in a module — it is infrastructure, not a domain feature) returning `StatusCode::OK`.

### Base Askama template

`templates/layout/base.html` should be a minimal HTML5 document with:
- Tailwind CSS link (`/assets/css/main.css`)
- HTMX script tag (CDN)
- `{% block content %}{% endblock %}` body block

### Tailwind input

`assets/css/input.css`:
```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

### Makefile targets

| Target | Command |
|--------|---------|
| `dev` | Run `cargo watch` + `tailwindcss --watch` concurrently |
| `build` | `cargo build --release` |
| `lint` | `cargo fmt --check && cargo clippy -- -D warnings` |
| `test` | `cargo test` |
| `migrate` | `cargo sqlx migrate run` |
| `css` | `npx tailwindcss -i assets/css/input.css -o assets/css/main.css` |

### .env.example

```
DATABASE_URL=postgres://fbapp:fbapp@localhost:5432/fbapp
HOST=0.0.0.0
PORT=3000
```

## Outcome

Full project skeleton created and all acceptance criteria met. All files match the ADR-0007 directory layout. Key implementation notes:

- `Config` derives `Clone` to allow `AppState` to derive `Clone` (required by Axum's `State` extractor)
- `/assets` is served as a static directory via `tower-http`'s `ServeDir` — required for Tailwind CSS to be reachable at `/assets/css/main.css`
- `sqlx::migrate!()` runs automatically at startup before the server binds
- `Cargo.lock` is gitignored (application binary — include it if preferred for reproducible builds)
- `.sqlx/` is also gitignored with a comment explaining how to opt in for offline CI builds

Closeout update (2026-04-24): Docker and documentation drift from the original scaffold has been closed out.

- Added root Docker packaging with a multi-stage build, non-root runtime image, static asset generation, templates, migrations, and runtime-only app artifacts.
- Added `.dockerignore` exclusions for local build outputs, secrets, certs, git metadata, and agent workspaces while preserving lockfiles for reproducible builds.
- Updated Compose so the app reads application secrets from `.env`, keeps container networking overrides in Compose, and mounts local TLS certs at runtime instead of baking them into the image.
- Updated README, `.env.example`, and ADRs to match current config keys, Tailwind v4 CSS-first setup, vendored HTMX/Alpine assets, current modules, and Docker behavior.
- Verification passed: `make lint`, `make test`, `make css`, `make js`, `docker compose config`, `docker compose build app`, full `docker compose up --build` smoke, `/health`, and containerized CSS/JS asset checks.

Follow-up tasks: _none_
