---
date: 2026-04-23T13:07:06+03:00
git_commit: d8bdf77c28ef8770f4d4f128b4027ccf2e29ed3b
branch: main
repository: fbapp-vibe
topic: "Analyze architectural structure and issues of the project"
tags: [research, codebase, architecture, modular-monolith, axum, askama, htmx, sqlx, polling]
last_updated: 2026-04-23
---

## Ticket Synopsis
Research the project's architectural structure and call out issues, with findings written to `docs/`.

## Summary
The codebase is a single-crate modular monolith built around Axum, Askama, HTMX, SQLx, and PostgreSQL. The structure mostly matches the documented architecture, but there is one clear boundary leak: `predictions` reaches directly into `standings::db` for membership checks, which violates the modular-monolith rule against cross-module DB access.

## Detailed Findings

### Architecture Layout
- The repo is explicitly organized as a modular monolith in ADR-0007 (`docs/adr/0007-project-structure-modular-monolith.md:26-31`), with route modules under `src/modules/` and shared/non-route code under `src/` (`src/CLAUDE.md:1-39`).
- `src/routes.rs:6-16` assembles the top-level router by merging module routers and serving static assets.
- `src/modules/mod.rs:1-8` is a thin registry of feature modules only.
- `main.rs` wires the runtime stack: config, migrations, TLS, auth/session layers, app state, background jobs, and the server (`src/main.rs:19-84`).

### Module Boundaries
- Each feature module follows the same pattern: `mod.rs` exposes `router()`, while `handlers.rs`, `db.rs`, and `models.rs` remain internal (`src/modules/auth/mod.rs:1-63`, `src/modules/admin/mod.rs:1-65`, `src/modules/leagues/mod.rs:1-20`, `src/modules/predictions/mod.rs:1-25`, `src/modules/standings/mod.rs:1-34`).
- `auth` exposes `User` publicly (`src/modules/auth/mod.rs:12-15`), which is then reused by shared code like `nav.rs` (`src/nav.rs:1-45`).
- `standings` is the one exception: its DB module is `pub(crate)` (`src/modules/standings/mod.rs:5-9`), and another module calls into it directly.

### Boundary Leak
- `predictions_review` checks membership with `crate::modules::standings::db::is_member(...)` (`src/modules/predictions/handlers.rs:231-240`).
- That directly contradicts ADR-0007's rule that modules should not call another module's `db.rs` functions (`docs/adr/0007-project-structure-modular-monolith.md:71-78`).
- This is the main architectural issue found. The code should prefer a public module API or a shared membership helper instead of reaching into `standings::db`.

### Runtime And Cross-Cutting Services
- `AppState` centralizes the pool, config, OAuth client, and football API client (`src/state.rs:11-33`). This keeps handler signatures simple but makes the app-state object the primary coupling point.
- The app spawns two endless background loops: result polling and session cleanup (`src/main.rs:57-61`, `src/polling/mod.rs:10-29`, `src/session_cleanup.rs:6-17`).
- Both loops log errors and continue, which is operationally resilient but has no explicit supervision or shutdown coordination.

### Pure Domain Logic Separation
- The standings computation is well factored into a pure helper in `src/group_standings.rs:1-118`, and the handler delegates to it (`src/modules/standings/handlers.rs:481-501`).
- `src/modules/standings/models.rs` also keeps a lot of pure transformation logic out of handlers: leaderboard ranking, scenario parsing, streak calculation, and fixture grouping (`src/modules/standings/models.rs:55-167`, `src/modules/standings/models.rs:383-543`).
- That split is a strength: the codebase is already extracting business logic away from HTTP handlers where it matters.

### External Dependency And Polling Design
- `FootballApiClient` is a shared, rate-limited wrapper around football-data.org and is stored in `AppState` (`src/football_api.rs:12-278`, `docs/adr/0018-football-api.md:32-38`, `docs/adr/0018-football-api.md:70-73`).
- The polling job is intentionally serial and conservative because the free API tier is rate-limited (`src/football_api.rs:14-16`, `src/polling/mod.rs:10-29`).
- That design is consistent with the ADR, but it means polling throughput is intentionally low and globally serialized.

## Code References
- `src/CLAUDE.md:1-39` - repo-level architecture guidance for route vs non-route modules.
- `docs/adr/0007-project-structure-modular-monolith.md:26-31, 67-106` - modular monolith decision and boundary rules.
- `src/main.rs:19-84` - startup wiring, migrations, auth/session layers, background tasks.
- `src/routes.rs:6-16` - top-level router assembly.
- `src/modules/mod.rs:1-8` - feature-module registry.
- `src/modules/auth/mod.rs:1-63` - auth module router and backend glue.
- `src/modules/admin/mod.rs:1-65` - admin access control and router.
- `src/modules/leagues/mod.rs:1-20` - league routes and exports.
- `src/modules/predictions/mod.rs:1-25` - prediction routes and test coverage.
- `src/modules/standings/mod.rs:1-34` - standings router plus the `pub(crate)` DB module.
- `src/modules/predictions/handlers.rs:231-240` - cross-module DB access to `standings::db`.
- `src/polling/mod.rs:10-141` - background polling loop and scoring pipeline.
- `src/session_cleanup.rs:6-25` - session cleanup loop.
- `src/group_standings.rs:1-118` - pure standings computation.
- `src/modules/standings/handlers.rs:481-501` - handler delegation to pure standings logic.

## Architecture Insights
- The project is deliberately structured for a small-to-medium team: one crate, explicit module APIs, and a single shared `AppState`.
- The architecture is consistent across the codebase, and the docs match the implementation closely.
- The biggest drift from the documented architecture is the direct `predictions` → `standings::db` call.
- Most other cross-cutting concerns are centralized cleanly: auth/session, tracing, config, background tasks, and external API access.

## Historical Context
- `docs/adr/0007-project-structure-modular-monolith.md` explains why the repo is a modular monolith and what the module boundary rules are.
- `docs/adr/0010-observability-with-tracing.md` explains the tracing setup used by `main.rs` and `routes.rs`.
- `docs/adr/0011-authentication-strategy.md` is superseded, but it documents the session-based auth pattern that is still reflected in the runtime wiring.
- `docs/adr/0018-football-api.md` explains the single external data source and the rate-limit-driven polling design.
- No `thoughts/` directory exists in this repo, so there was no separate historical notes tree to mine.

## Related Research
- `docs/adr/0007-project-structure-modular-monolith.md`
- `docs/adr/0010-observability-with-tracing.md`
- `docs/adr/0011-authentication-strategy.md`
- `docs/adr/0018-football-api.md`

## Open Questions
- Should the membership check used by `predictions_review` move behind a public `standings` API or a shared authorization helper?
- Do the long-running background loops need explicit shutdown handling or supervision as the app grows?
- Is the current `AppState` shape still the right coupling point, or should shared services be split further before the next feature wave?
