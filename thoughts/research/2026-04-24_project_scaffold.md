---
date: 2026-04-24T08:56:56+03:00
git_commit: 9eae645bc6d43a3f7b8d972b5bdd7c7af9bafa50
branch: main
repository: fbapp-vibe
topic: "Project scaffold"
tags: [research, codebase, scaffold, axum, sqlx, askama, tailwind, docker]
last_updated: 2026-04-24
---

## Ticket Synopsis

`thoughts/tickets/project-scaffold.md` describes the initial repository bootstrap: a compiling Rust application with Axum, SQLx/PostgreSQL, Askama templates, Tailwind assets, typed environment configuration, `AppState`, top-level route assembly, tracing, startup migrations, Docker Compose, and Makefile targets. The ticket is marked complete and its outcome notes that the skeleton was created, `/assets` was served through `ServeDir`, `sqlx::migrate!()` ran at startup, and `Config` derived `Clone` so it could live in cloneable Axum state.

## Summary

The live codebase still reflects the scaffold's core architecture: `main.rs` loads config, connects to PostgreSQL, runs SQLx migrations before binding, builds shared `AppState`, starts background tasks, and serves an Axum router assembled in `routes.rs` (`src/main.rs:24-83`, `src/routes.rs:6-16`). The feature-module pattern from ADR-0007 is active: modules are registered in `src/modules/mod.rs`, expose `router()`, and keep handlers and DB code internal except for selected public APIs (`src/modules/mod.rs:1-8`, `src/modules/auth/mod.rs:56-63`, `src/modules/admin/mod.rs:40-65`, `src/modules/leagues/mod.rs:14-20`, `src/modules/predictions/mod.rs:12-25`, `src/modules/standings/mod.rs:9-34`).

Several scaffold assumptions have evolved. Dependencies are now newer than the original ticket examples: Axum `0.8`, Tower `0.5`, Tower HTTP `0.6`, SQLx `0.8`, Askama `0.15`, and Tailwind CSS `4.2.2` (`Cargo.toml:6-36`, `package.json:16-24`). `Config` now includes OAuth, TLS, football API, polling, and session settings beyond the original database/host/port fields (`src/config.rs:10-32`). Docker Compose still declares `build: .`, but no `Dockerfile` exists in the repository, so the scaffold acceptance criterion that `docker compose up` starts the app is not currently satisfiable without adding or restoring a Dockerfile (`docker-compose.yml:1-13`).

Task sub-agent dispatch was attempted for the requested Locate phase but failed with `ProviderModelNotFoundError` in this environment, so this research was completed directly with repository search/read tools while preserving the requested Locate -> Pattern -> Analyze flow.

## Detailed Findings

### Startup And Runtime Wiring

- `main.rs` installs the Rustls crypto provider, initializes tracing, loads config, connects to PostgreSQL, and runs migrations before server bind (`src/main.rs:18-30`). This preserves the scaffold requirement and ADR-0012 constraint that `sqlx::migrate!().run(&pool).await` occurs before `axum::serve`.
- Runtime wiring has grown beyond the scaffold: TLS config is optional, sessions are PostgreSQL-backed via `PostgresStore`, auth is layered with `AuthManagerLayerBuilder`, OAuth and football API clients are built at startup, and polling/session-cleanup jobs are spawned (`src/main.rs:31-67`).
- The server supports HTTPS when both TLS cert and key paths are configured; otherwise it binds a Tokio TCP listener and calls `axum::serve` (`src/main.rs:77-83`).

### Route Assembly And Health Endpoint

- The top-level router keeps infrastructure-only logic in `routes.rs`: `GET /health` returns `StatusCode::OK`, static `/assets` are served with `ServeDir`, `TraceLayer::new_for_http()` is applied, and shared state is attached with `.with_state(state)` (`src/routes.rs:6-20`).
- `routes.rs` only merges module routers and does not call module handlers directly (`src/routes.rs:9-13`). This matches ADR-0007's consequence that `routes.rs` should only call `{module}::router()` (`docs/adr/0007-project-structure-modular-monolith.md:101-103`).

### AppState And Configuration

- `AppState` is the primary shared runtime context and stores `PgPool`, `Config`, OAuth client/endpoints, and the football API client (`src/state.rs:11-18`). Its constructor is simple field assignment (`src/state.rs:20-36`).
- `Config::load()` still follows the scaffold and ADR-0008 pattern: call `dotenvy::dotenv().ok()` and then deserialize the environment with `envy::from_env::<Config>()` (`src/config.rs:54-58`, `docs/adr/0008-configuration-management.md:65-84`).
- Required configuration now includes Google OAuth credentials and a football API key in addition to `DATABASE_URL`, while TLS and tuning settings are optional/defaulted (`src/config.rs:10-32`, `.env.example:1-24`). This means a fresh local run now needs more env vars than the original scaffold ticket listed.
- The default host is now `127.0.0.1` in code and `.env.example`, while Docker Compose overrides it to `0.0.0.0` for container binding (`src/config.rs:46-52`, `.env.example:1-3`, `docker-compose.yml:6-9`).

### Error Handling

- `AppError` remains the single HTTP error boundary and derives `thiserror::Error` (`src/error.rs:7-19`).
- `IntoResponse` maps auth and domain failures to 401/403/404/400 and unexpected errors to rendered 500 templates, logging unexpected errors at `error` and request errors at `warn` (`src/error.rs:37-52`).
- Inline unit tests cover the status mapping for unauthorized, forbidden, not found, bad request, and unexpected errors (`src/error.rs:62-102`).
- ADR-0009's broader rule that handlers return `Result<impl IntoResponse, AppError>` is reflected across feature handlers; grep found this pattern in auth, leagues, predictions, standings, and admin handlers.

### Feature Module Structure

- The current module registry includes `admin`, `auth`, `leagues`, `predictions`, and `standings` (`src/modules/mod.rs:1-8`). This is the expected evolution from the scaffold's empty `modules/mod.rs`.
- Each module exposes a `router() -> Router<AppState>` function consumed by `routes.rs` (`src/modules/auth/mod.rs:56-63`, `src/modules/admin/mod.rs:40-65`, `src/modules/leagues/mod.rs:14-20`, `src/modules/predictions/mod.rs:12-25`, `src/modules/standings/mod.rs:9-34`).
- The modules generally keep `handlers.rs` and `db.rs` private, while selectively exposing model/API types. Examples include `auth` re-exporting `User` and `leagues` exposing membership/list helpers (`src/modules/auth/mod.rs:8-15`, `src/modules/leagues/mod.rs:8-12`).
- Prior architecture research found one boundary leak: `predictions` directly calls `standings::db` for membership, which contradicts ADR-0007's no-cross-module-DB rule (`docs/architecture-structure-issues.md:25-33`). That is historical context for scaffold architecture drift, not part of the initial scaffold itself.

### Assets, Templates, And Tailwind

- The base Askama layout exists and links `/assets/css/main.css`, a vendored HTMX path, and a vendored Alpine path (`templates/layout/base.html:1-18`). This has evolved from the ticket's original CDN-only HTMX requirement.
- The only file currently present under `assets/js` is `countdown.js`; `templates/layout/base.html` references `/assets/js/htmx.js` and `/assets/js/alpine.js`, but those generated/vendor files are not present in the current working tree. `Makefile` has a `js` target that copies them from `node_modules` (`Makefile:27-29`).
- Tailwind has migrated to v4 syntax: `assets/css/input.css` uses `@import "tailwindcss"`, `@plugin`, inline icon sources, and `@theme` tokens rather than the v3 `@tailwind base/components/utilities` directives from the scaffold ticket (`assets/css/input.css:1-105`).
- No `tailwind.config.js` or `tailwind.config.*` file is present, which is consistent with Tailwind v4's CSS-first setup but differs from the original scaffold acceptance criterion.

### Migrations And Database

- The initial migration remains a placeholder (`migrations/0001_initial.sql:1-2`). Later numbered migrations now define users, sessions, tournaments, leagues, predictions, indexes, achievements, and team flags.
- `Cargo.toml` enables SQLx Postgres, Tokio Rustls runtime, macros, and `time` support (`Cargo.toml:11`). `AppState` stores a cloneable `PgPool` (`src/state.rs:1-14`).
- Tests use SQLx migrations through `#[sqlx::test(migrations = "./migrations")]`, consistent with repo guidance for DB-backed cases.

### Docker And Make Targets

- `docker-compose.yml` defines `app`, `db`, and optional `jaeger` services; `app` uses `build: .`, maps port 3000, and supplies database/host/port environment variables (`docker-compose.yml:1-13`, `docker-compose.yml:30-41`).
- There is no `Dockerfile` at the repository root. Because Compose builds `.`, this is a concrete gap for the original scaffold's Docker acceptance criterion.
- The Makefile includes the required scaffold targets `dev`, `build`, `lint`, `test`, `migrate`, and `css`, plus a later `js` target for vendoring HTMX/Alpine (`Makefile:1-29`).

## Code References

- `thoughts/tickets/project-scaffold.md:16-35` - Original scaffold summary and acceptance criteria.
- `thoughts/tickets/project-scaffold.md:157-165` - Outcome notes from scaffold completion.
- `src/main.rs:18-30` - Startup config, DB connection, and migration order.
- `src/main.rs:31-67` - TLS, sessions, auth, OAuth, football API, AppState, and background task wiring.
- `src/routes.rs:6-20` - Health route, module router merges, static assets, TraceLayer, and state.
- `src/config.rs:10-32` - Current expanded typed configuration struct.
- `src/config.rs:54-58` - `dotenvy` + `envy` load implementation.
- `src/state.rs:11-18` - Current `AppState` fields.
- `src/error.rs:7-19` - `AppError` variants.
- `src/error.rs:37-52` - `IntoResponse` implementation and HTTP mapping.
- `src/modules/mod.rs:1-8` - Feature-module registry.
- `src/modules/auth/mod.rs:56-63` - Auth module router.
- `src/modules/admin/mod.rs:40-65` - Admin module router.
- `src/modules/leagues/mod.rs:14-20` - Leagues module router.
- `src/modules/predictions/mod.rs:12-25` - Predictions module router.
- `src/modules/standings/mod.rs:9-34` - Standings module router.
- `templates/layout/base.html:1-18` - Base Askama layout and asset links.
- `assets/css/input.css:1-105` - Tailwind v4 CSS-first configuration and theme tokens.
- `migrations/0001_initial.sql:1-2` - Initial placeholder migration.
- `Cargo.toml:6-42` - Current Rust dependency set and denied `unwrap_used` lint.
- `package.json:16-24` - Tailwind v4, HTMX, Alpine, and Iconify frontend dev dependencies.
- `Makefile:1-29` - Dev/build/lint/test/migrate/css/js targets.
- `docker-compose.yml:1-44` - Compose services for app, PostgreSQL, and Jaeger.
- `.env.example:1-24` - Current documented environment variables.

## Architecture Insights

The scaffold established a durable shape: one Rust crate, route aggregation at the top level, typed shared state, typed configuration, SQLx migrations at startup, global HTTP error mapping, and static assets served from Axum. That foundation has held as the app grew into authentication, admin workflows, leagues, predictions, standings, polling, and observability.

The most important architectural drift is expected product growth rather than scaffold erosion: `Config` and `AppState` now carry real app services; `main.rs` does substantial composition; Tailwind moved to v4; and the module registry now has several feature modules. The code still mostly follows the modular-monolith pattern, but historical research records one cross-module DB access that should be kept in mind when evaluating scaffold conformance.

The main operational scaffold gap is Docker packaging. ADR-0012 documents a multi-stage Dockerfile pattern and `docker-compose.yml` assumes one exists, but the live repository currently lacks a `Dockerfile`.

## Historical Context (from thoughts/)

- `thoughts/tickets/project-scaffold.md` - The original ticket and outcome for the initial scaffold.
- `docs/adr/0007-project-structure-modular-monolith.md` - Defines the modular monolith layout and rules that the scaffold implemented.
- `docs/adr/0008-configuration-management.md` - Establishes `dotenvy` + `envy`, startup-only config loading, and `Config` in `AppState`.
- `docs/adr/0009-error-handling-strategy.md` - Establishes `AppError` as the single HTTP boundary and forbids `unwrap()` in app code.
- `docs/adr/0010-observability-with-tracing.md` - Establishes `tracing`, `TraceLayer`, and `RUST_LOG` defaults.
- `docs/adr/0012-deployment-with-docker.md` - Establishes the Docker Compose pattern and a multi-stage Dockerfile design.
- `docs/architecture-structure-issues.md` - Prior architecture research noting the current modular-monolith shape and one boundary leak.

## Related Research

- `thoughts/research/2026-04-23_google_oauth_login_flow.md` - Explains later auth/session runtime built on the scaffold.
- `thoughts/research/2026-04-23_auth_integration_tests.md` - Covers DB-backed integration-test patterns that rely on scaffolded SQLx/test structure.
- `docs/architecture-structure-issues.md` - Broader architecture review with module-boundary observations.

## Open Questions

- Should the missing root `Dockerfile` be restored so `docker compose up` works as the original scaffold acceptance criterion and ADR-0012 expect?
- Should `make js` be part of `make dev`, `make build`, or documented setup so `/assets/js/htmx.js` and `/assets/js/alpine.js` exist whenever `templates/layout/base.html` references them?
- Should README project structure be updated to include the current `standings` module and Tailwind v4/no-config setup?
- Should the known `predictions` -> `standings::db` boundary leak be addressed before more feature modules depend on internal DB APIs?
