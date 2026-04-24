---
title: Update docs to reflect Rust 2024 edition
source: .claude/tasks/done/0037-rust-2024-edition-docs.md
source_id: 0037
source_status: done
source_title: Update docs to reflect Rust 2024 edition
status: reviewed
phase: MVP
type: chore
adrs: [0001, 0021]
refs:
  - thoughts/plans/rust_2024_edition_docs_consistency_closeout.md
created: 2026-04-08
started: 2026-04-09
completed: 2026-04-09
---

## Summary

The project was migrated from Rust edition 2021 to 2024 (commit `1692d98`) but no documentation reflects this change. Update the existing Rust ADR and write a new ADR recording the upgrade decision so the docs accurately describe the current state of the project.

## Acceptance Criteria

- [x] `docs/adr/0001-use-rust-as-programming-language.md` explicitly states that edition 2024 is in use
- [x] A new `docs/adr/0021-rust-2024-edition-upgrade.md` documents the upgrade decision (context, rationale, notable 2024 changes that affect the project, trade-offs)
- [x] All `.md` files in the repo are scanned for stale "edition 2021" references and updated where found
- [x] No code changes — documentation only

## Implementation Context

### Relevant files

- `docs/adr/0001-use-rust-as-programming-language.md` — add a sentence/note that edition 2024 is used; link to the new ADR
- `docs/adr/0021-rust-2024-edition-upgrade.md` — new file to create
- `Cargo.toml` — already has `edition = "2024"`; reference as evidence of the decision

### Rust 2024 edition changes relevant to this project

Cover these in the new ADR:

- **`gen` keyword reserved** — `gen` can no longer be used as an identifier; not currently used in this codebase
- **Stricter `impl Trait` lifetime capture** — return-position `impl Trait` now captures all in-scope lifetimes by default; may require `+ use<'a>` bounds in some signatures
- **`unsafe extern` blocks** — `extern "C"` blocks containing unsafe items now require `unsafe extern { ... }`; not applicable here (no FFI), but worth noting
- **`cargo` behaviour changes** — `[lints]` table is now stable and recommended (already used in `Cargo.toml`)
- **Improved `async` ergonomics** — `async fn` in traits and RPIT capture improvements align better with async Axum handler patterns used in this project

### ADR format

Follow the existing ADR style (see any file in `docs/adr/`): sections are Status, Date, Context, Decision, Rationale, Trade-offs and Risks, Consequences. Use emojis throughout.

New ADR ID is **0021** (0020 is the last existing one).

### Tests

- No tests — documentation only

## Outcome

Documentation-only update. Added a sentence to ADR-0001 Consequences noting that edition 2024 is in use with a link to ADR-0021. Created `docs/adr/0021-rust-2024-edition-upgrade.md` covering context, the five notable edition 2024 changes relevant to the project, rationale, trade-offs, and consequences. Updated the one stale "edition 2021" reference found in `.claude/tasks/done/0001-project-scaffold.md`. No code changes.

Follow-up tasks: _none_
