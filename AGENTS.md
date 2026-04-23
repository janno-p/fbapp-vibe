# AGENTS.md

- This repo is Rust-first: use `cargo`/`make` for app work. `package.json` only holds frontend dev deps for Tailwind and vendored JS; `npm test` is a stub.
- Primary commands: `make dev`, `make build`, `make lint` (`cargo fmt --check && cargo clippy -- -D warnings`), `make test`, `make migrate`, `make css`, `make js`.
- `cargo run` already runs SQLx migrations at startup. Use `make migrate` only when you need the DB prepared without starting the server.
- `RUST_LOG` defaults to `fbapp_vibe=debug,tower_http=debug,sqlx=warn`.
- HTTP routes are assembled in `src/routes.rs`. Feature routes live under `src/modules/<name>/` and must expose `router()`. Register new feature modules in `src/modules/mod.rs` and `src/routes.rs`.
- Shared non-route code lives directly under `src/` (for example `polling.rs`, `session_cleanup.rs`, `tracing_setup.rs`), not under `src/modules/`.
- Current feature modules are `auth`, `admin`, `leagues`, `predictions`, and `standings`.
- If you edit under `src/`, `src/modules/<name>/`, or `tests/`, read the relevant ticket or plan under `thoughts/` first. Repo-wide guidance also lives in `thoughts/`.
- Tests: keep pure logic tests inline with `#[cfg(test)]`; use `#[sqlx::test]` or `tests/` for DB-backed cases. `TEST_DATABASE_URL` must point to a separate database on the same host as `DATABASE_URL`.
- Avoid mocking framework glue just to force a test; extract pure logic first. `clippy::unwrap_used` is denied.
- Askama templates compile at build time, so template fields must match the backing structs exactly.
- Cross-cutting changes and architecture decisions belong in `docs/adr/` using the existing ADR format; IDs are sequential and never reused.
- Task work is spec-first in `thoughts/tickets/` and `thoughts/plans/`: copy `thoughts/tickets/TEMPLATE.md`, fill acceptance criteria, keep the linked plan updated, and fill `## Outcome` before closing.
