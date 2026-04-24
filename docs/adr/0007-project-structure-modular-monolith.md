# ADR-0007: Project Structure as a Modular Monolith 🏛️

## Status

✅ Accepted

## Date

2026-04-05

## Context

Before writing application code, the project's module and directory structure must be established. The structure affects how easily features can be added, how well boundaries between domains are enforced, and how straightforward a future migration to a distributed architecture would be.

Two structural approaches were considered:

| | **Single crate + visibility rules** | **Cargo workspace** |
|--|-------------------------------------|-------------------|
| Boundary enforcement | `pub(crate)`, `pub(super)`, `pub` in `mod.rs` | Compiler-enforced — separate crates with explicit `Cargo.toml` dependencies |
| Setup complexity | Low | Medium |
| Inter-module dependencies | Visible but discouraged by convention | Explicitly declared per crate |
| Compile times | Single unit | Parallel per-crate compilation |
| Path to microservices/workspace | Refactor to workspace later | Extract crate and add network layer |
| Best for | Early-stage, small-to-medium apps | Larger teams, stricter boundaries needed from day one |

The desired architectural style is a **modular monolith**: a single deployable unit with well-defined, encapsulated domain modules that communicate through explicit public APIs rather than reaching into each other's internals.

## Decision

We will structure the project as a **modular monolith** 🏛️ using a **single Cargo crate** with module boundaries enforced via Rust's visibility system. The structure is designed to graduate to a Cargo workspace without major refactoring if stricter compile-time boundaries become necessary.

## Directory Layout

```
fbapp-vibe/
├── migrations/               # SQLx migrations (versioned, sequential)
├── templates/                # Askama HTML templates
│   ├── layout/
│   │   └── base.html         # Base layout (navbar, head, footer)
│   └── {module}/             # One subdirectory per domain module
│       └── *.html
├── assets/
│   ├── css/
│   │   └── main.css          # Tailwind CSS compiled output
│   └── js/                   # Vendored HTMX/Alpine and local JS assets
├── tests/                    # HTTP-level integration tests (axum-test)
├── src/
│   ├── main.rs               # Entry point: config loading, server binding, startup
│   ├── lib.rs                # Crate root: re-exports all modules (enables tests/ to import)
│   ├── config.rs             # App configuration loaded from environment variables
│   ├── db_types.rs           # Shared DB enums used across modules (MatchOutcome, KnockoutRound)
│   ├── error.rs              # Global AppError type + IntoResponse implementation
│   ├── extractors.rs         # Shared Axum extractors (e.g. QsForm<T>)
│   ├── state.rs              # AppState struct (PgPool, config, etc.) passed via Axum State
│   ├── routes.rs             # Aggregates routers from all modules into one Router
│   └── modules/
│       ├── mod.rs            # Re-exports module routers; no business logic
│       └── {module}/
│           ├── mod.rs        # Public API: exposes router() and public domain types only
│           ├── handlers.rs   # Axum handlers (private to module)
│           ├── db.rs         # SQLx queries (private to module)
│           └── models.rs     # Domain structs and types (selectively pub)
├── Cargo.toml
└── Makefile                  # Dev tasks: watch, build, migrate, lint
```

## Module Boundary Rules 📐

These rules define the modular monolith contract and must be followed by all modules:

1. 🚪 **Each module exposes a single `router()` function** — this is the only entry point `routes.rs` uses. It never calls handlers or DB functions from another module directly.

2. 🔒 **Handlers and DB functions are module-private** — `handlers.rs` and `db.rs` are not re-exported from `mod.rs`. They are internal implementation details.

3. 📤 **Domain types are selectively public** — structs and enums in `models.rs` that are needed by other modules are re-exported explicitly from `mod.rs`. Everything else stays private.

4. 🚫 **No cross-module DB access** — a module never calls another module's `db.rs` functions. If shared data is needed, it is accessed through the owning module's public API or a shared read model.

5. 🗂️ **Templates mirror module structure** — each module's templates live in `templates/{module}/`, keeping HTML co-located logically with the module that renders it.

## Rationale

1. 🏛️ **Enforces domain boundaries without a framework**: Rust's visibility system (`pub`, `pub(crate)`, `pub(super)`) makes boundary violations a compiler error or a deliberate, visible choice. No external architecture tooling is required.

2. 🔄 **Designed to graduate to a workspace**: Each `modules/{module}/` directory maps directly to a future Cargo crate. If a module needs stronger isolation or independent compilation, it can be extracted with minimal structural change — the public API in `mod.rs` becomes the crate's `lib.rs`.

3. 🧩 **Feature development is self-contained**: Adding a new domain feature means creating a new directory under `modules/`, wiring its `router()` into `routes.rs`, and creating a matching `templates/{module}/` directory. No other files need to change.

4. 🛡️ **Prevents big-ball-of-mud drift**: Explicit module boundaries and the rule against cross-module DB access prevent the codebase from collapsing into an undifferentiated mass of interconnected code as it grows.

5. 📐 **Clear mental model for contributors**: Every developer can answer "where does this code live?" by mapping the domain concept to its module directory, then the technical role (`handlers.rs`, `db.rs`, `models.rs`) within it.

## Trade-offs and Risks ⚠️

- 🔓 **Boundaries are conventional, not fully compiler-enforced**: A single-crate approach relies on team discipline to avoid reaching into private module internals via `pub(crate)` escalation. A Cargo workspace would enforce this at the compiler level. This risk is accepted at the current project scale and can be revisited in a future ADR.
- 📁 **More directories than a flat structure**: The modular layout adds more files and directories than a simple flat `src/handlers.rs` approach. This overhead is justified for any project expected to grow beyond a handful of features.
- 🔧 **Template directory duplication**: Templates live outside `src/` and must be kept in sync with their owning module. Askama's compile-time checking catches missing templates, mitigating drift.

## Consequences

- 📁 All new domain features are added as a subdirectory under `src/modules/` following the `handlers.rs` / `db.rs` / `models.rs` layout.
- 🚪 `routes.rs` only calls `{module}::router()` — it contains no handler logic.
- 📤 `mod.rs` for each module explicitly lists everything it re-exports; nothing is pub by default.
- 🗂️ Askama templates for a module live in `templates/{module}/`.
- 🗄️ SQLx migrations live in `migrations/` at the project root, named sequentially (`0001_create_users.sql`, etc.).
- 📋 If a module grows large enough to warrant a separate compilation unit or team ownership, it is extracted into a Cargo workspace crate in a future ADR.

## Amendment: Current Layout Notes

Date: 2026-04-24

The modular-monolith structure remains accepted. The live application has grown from the initial scaffold and currently includes these registered feature modules in `src/modules/mod.rs`: `auth`, `admin`, `leagues`, `predictions`, and `standings`.

Current shared application code also includes `src/football_api.rs`, `src/polling/`, `src/session_cleanup.rs`, and `src/tracing_setup.rs`. Static assets now include `assets/js/` for vendored HTMX and Alpine files in addition to Tailwind CSS assets. Tailwind v4 CSS-first configuration means the current layout no longer requires a root `tailwind.config.js`; see ADR-0006 and ADR-0020 for the styling and JavaScript asset details.
