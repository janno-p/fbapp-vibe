# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
make dev        # cargo watch + tailwindcss --watch (concurrent)
make build      # cargo build --release
make lint       # cargo fmt --check && cargo clippy -- -D warnings
make test       # cargo test
make migrate    # cargo sqlx migrate run
make css        # compile Tailwind CSS once
```

Run a single test:
```bash
cargo test test_name
```

Logging level is controlled via `RUST_LOG` (default: `fbapp_vibe=debug,tower_http=debug,sqlx=warn`).

## Architecture

Server-rendered Rust web app: **Axum** + **Askama** templates + **HTMX** + **Tailwind CSS**, with **PostgreSQL** via SQLx and **Google OAuth** via axum-login.

### Request lifecycle

1. `main.rs` — wires layers: `AuthManagerLayer` (axum-login) wraps `SessionManagerLayer` (tower-sessions backed by PostgreSQL), then `TraceLayer`.
2. `routes.rs` — assembles the top-level `Router` by merging module routers and serving `assets/` statically.
3. `AppState` (`state.rs`) — holds `PgPool`, `Config`, and `BasicClient` (OAuth); injected into handlers via Axum's `State` extractor.

### Module structure

Two distinct kinds of modules, with different homes:

**Route modules** live under `src/modules/<name>/` and must expose a `router() -> Router<AppState>`. Register them in `src/modules/mod.rs` and merge into the top-level router in `routes.rs`.

Inside a route module:
- `mod.rs` — public API: `router()`, framework glue (e.g. `AuthBackend`), re-exports
- `handlers.rs` — Axum handler functions
- `models.rs` — domain types
- `db.rs` — SQLx queries

**Non-route modules** (API clients, background jobs, shared libraries) live directly under `src/` as a single file (`src/foo.rs`) or a directory with children (`src/foo/mod.rs`) only when they have submodules. Declare them in `main.rs` with `mod foo;`. Do not place them under `src/modules/`.

| Module type | Location | Example |
|---|---|---|
| HTTP feature with routes | `src/modules/<name>/` | `auth`, `admin`, `predictions` |
| External API client | `src/football_api.rs` | football-data.org client |
| Background job | `src/polling.rs` or `src/polling/` | result polling task |
| Infrastructure | `src/config.rs`, `src/error.rs` | config, error types |

Currently implemented modules:
- **`auth`** — Google OAuth login/callback/logout, session management, home and dashboard pages
- **`admin`** — tournament registration, activation, locking, seeding from football API, league management
- **`leagues`** — league creation, invite-token join flow, membership
- **`predictions`** — group stage, knockout, and top scorer prediction forms with lock enforcement

### Auth flow

`/auth/login` → Google OAuth → `/auth/callback` → upsert user in DB → `auth_session.login()` → redirect to `/dashboard`. Session is stored in the `tower_sessions` PostgreSQL table. `axum_login::AuthSession<AuthBackend>` is the extractor used in protected handlers.

### Error handling

`AppError` (`error.rs`) is the single error type returned by all handlers. It implements `IntoResponse`. Use `anyhow::Error` for unexpected errors (maps to 500) and `AppError::Unauthorized` / `AppError::BadRequest` for expected ones. `clippy::unwrap_used` is denied — use `?` or explicit error handling.

### Templates

Askama templates live in `templates/<module>/`. The base layout is in `templates/layout/`. Templates are compiled at build time; struct fields map directly to template variables.

### Migrations

SQL files in `migrations/` are versioned (`0001_...sql`, `0002_...sql`). Migrations run automatically on startup (`sqlx::migrate!().run(&pool)`), so `make migrate` is only needed when running without `cargo run`.

### TLS

Optional dev TLS via `mkcert`. Set `TLS_CERT_PATH` and `TLS_KEY_PATH` in `.env`; if both are present the server binds with Rustls, otherwise plain HTTP.

## Testing

### What to test

| Code type | Test type | Where |
|---|---|---|
| Business logic (scoring, rules, calculations) | Unit test | `#[cfg(test)]` mod in the same file |
| Error response mappings (`AppError`) | Unit test | `#[cfg(test)]` mod in `error.rs` |
| DB queries with non-trivial logic (upserts, concurrency) | Integration test | `tests/` or `#[sqlx::test]` in `db.rs` |
| Trivial handlers, framework wiring | Skip | Not worth the setup cost |

### What not to test

- Functions that just call a framework API and return the result
- Config loading (`envy` behaviour is not our code)
- Template rendering (compile-time checked by Askama)

### Patterns

**Unit tests** — use `#[cfg(test)]` inline in the source file. For async tests use `#[tokio::test]`. No mocking; if a function needs mocking to test, extract the pure logic first.

**Integration tests (DB)** — use the `#[sqlx::test]` macro. It creates an isolated test database per test and rolls back automatically. Requires `TEST_DATABASE_URL` in the environment (same host as `DATABASE_URL`, different DB name).

**Scoring logic** (task 0008 onwards) — implement scoring as pure functions in `src/polling/scorer.rs` that take plain data types, not DB rows. Test those functions exhaustively without any DB or async setup.

### Requirement

Every task that introduces business logic or a non-trivial data transformation **must** include tests as part of its acceptance criteria. The task is not complete until `cargo test` passes.

## Architecture Decision Records

All architectural decisions are documented as ADRs in `docs/adr/`. When making decisions about tech choices, patterns, or constraints, write an ADR first.

**Format:** Each ADR is a markdown file named `{id}-{short-slug}.md` with sections: Status, Date, Context, Decision, Rationale, Trade-offs and Risks, Consequences. Use emojis throughout — in headers, lists, and tables (see existing ADRs for style reference).

**When to write one:** Any decision that affects how the codebase is structured, which libraries are used, or how cross-cutting concerns (auth, error handling, config, observability) are handled. Prefer writing an ADR before implementing, not after.

**IDs** are sequential and never reused.

## Task Management

Work items are tracked as individual markdown files under `.claude/tasks/`, organised by status directory:

```
.claude/tasks/
├── TEMPLATE.md      # canonical template — read before creating a task
├── open/            # ready to be picked up
├── in-progress/     # currently being worked on
└── done/            # completed or cancelled
```

**File naming:** `{id}-{short-slug}.md` (e.g. `0004-user-profile-page.md`). IDs are sequential and never reused.

**Lifecycle:** move the file between directories as status changes (`open` → `in-progress` → `done`). Cancelled tasks go to `done/` with `status: cancelled` in frontmatter.

**Starting a task:** read the task file — it contains the goal, acceptance criteria, relevant file paths, ADR constraints, and implementation notes. Everything needed to implement without clarifying questions.

**Completing a task:** fill in the `## Outcome` section (what was built, deviations from spec, follow-up tasks), then move the file to `.claude/tasks/done/`.

**Creating a task:** copy `.claude/tasks/TEMPLATE.md`, fill in all sections including ADR references, and write acceptance criteria before starting implementation. Tasks are spec-first — no retroactive creation.
