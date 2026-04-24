# Rust 2024 Edition Docs Consistency Closeout Implementation Plan

## Overview

Close out remaining documentation drift around Rust 2024 edition guidance by reconciling ADR wording with the repository's actual configuration. This is a docs-only consistency pass: no runtime, build, lint, or toolchain configuration changes are in scope.

## Current State Analysis

The crate is already on Rust edition 2024 (`Cargo.toml:4`), and lint enforcement is operational through the standard workflow (`Makefile:13`, `AGENTS.md:4`). ADR-0001 already states edition 2024 and links to ADR-0021 (`docs/adr/0001-use-rust-as-programming-language.md:63`).

The remaining drift is inside ADR-0021:

- It claims the project already uses both `[lints.clippy]` and `[lints.rust]` (`docs/adr/0021-rust-2024-edition-upgrade.md:53`), but `Cargo.toml` currently has only `[lints.clippy]` (`Cargo.toml:41`).
- It describes a 1.85+ toolchain requirement with wording that implies pinning/enforcement (`docs/adr/0021-rust-2024-edition-upgrade.md:72`, `docs/adr/0021-rust-2024-edition-upgrade.md:81`), while no repository toolchain pin file exists.

## Desired End State

Documentation clearly reflects the live repo state without introducing policy claims that are not implemented in config.

### Key Discoveries:

- Rust edition source of truth is `Cargo.toml:4`.
- Lint configuration source of truth is `[lints.clippy]` in `Cargo.toml:41`.
- Contributor lint gate is operationally enforced by `make lint` (`Makefile:13`) and documented in `AGENTS.md:4`.
- ADR-0001 is already aligned and should not be modified for this closeout (`docs/adr/0001-use-rust-as-programming-language.md:63`).
- ADR-0021 contains the only required docs corrections (`docs/adr/0021-rust-2024-edition-upgrade.md:53`, `docs/adr/0021-rust-2024-edition-upgrade.md:72`, `docs/adr/0021-rust-2024-edition-upgrade.md:81`).

## What We're NOT Doing

- Not adding `[lints.rust]` to `Cargo.toml`.
- Not adding `rust-toolchain` or `rust-toolchain.toml`.
- Not adding `rust-version` to `Cargo.toml`.
- Not changing CI/build scripts or lint behavior.
- Not changing application code, database schema, or runtime behavior.

## Implementation Approach

Apply a minimal, evidence-backed documentation reconciliation. Keep the canonical configuration unchanged and update ADR language to match the actual repository state.

## Phase 1: Evidence Lock-In

### Overview

Freeze source-of-truth anchors for edition and lint/toolchain governance before editing docs.

### Changes Required:

#### 1. Confirm canonical configuration anchors
**Files**: `Cargo.toml`, `Makefile`, `AGENTS.md`, `docs/adr/0001-use-rust-as-programming-language.md`, `docs/adr/0021-rust-2024-edition-upgrade.md`
**Changes**: No file content changes in this step; capture anchors used for reconciliation and outcome evidence.

### Success Criteria:

#### Automated Verification:
- [x] Rust edition anchor confirmed: `rg -n 'edition = "2024"' Cargo.toml`
- [x] Lint-table anchor confirmed: `rg -n '^\[lints\.' Cargo.toml`
- [x] ADR drift locations confirmed: `rg -n 'lints\.rust|1\.85|toolchain' docs/adr/0021-rust-2024-edition-upgrade.md`

#### Manual Verification:
- [x] Every planned ADR correction has a concrete config anchor.
- [x] Scope remains docs-only and excludes config/toolchain changes.

---

## Phase 2: ADR Reconciliation

### Overview

Update ADR-0021 so its statements on lint tables and toolchain expectation match repository reality.

### Changes Required:

#### 1. Correct lint-table statement in ADR-0021
**File**: `docs/adr/0021-rust-2024-edition-upgrade.md`
**Changes**: Replace wording that claims both `[lints.clippy]` and `[lints.rust]` are already present with wording that accurately describes current state (`[lints.clippy]` present) and avoids asserting nonexistent config.

```md
Current repo state: `Cargo.toml` currently defines `[lints.clippy]` and uses it as active lint configuration.
```

#### 2. Reword toolchain requirement language in ADR-0021
**File**: `docs/adr/0021-rust-2024-edition-upgrade.md`
**Changes**: Keep factual Rust 2024 compatibility requirement, but remove wording that implies repository-level toolchain pinning is already implemented.

```md
Rust edition 2024 requires a compiler version that supports edition 2024 semantics (Rust 1.85+), while repository-level pinning is a separate operational choice.
```

### Success Criteria:

#### Automated Verification:
- [x] ADR no longer claims `[lints.rust]` exists if absent in `Cargo.toml`: `rg -n 'lints\.rust' docs/adr/0021-rust-2024-edition-upgrade.md`
- [x] ADR wording no longer states pinning as already implemented: `rg -n 'must pin|rust-toolchain\.toml' docs/adr/0021-rust-2024-edition-upgrade.md`
- [x] Repo lint/test baseline remains green: `make lint && make test`

#### Manual Verification:
- [x] ADR-0021 statements are directly traceable to `Cargo.toml` and current repo layout.
- [x] ADR-0001 remains unchanged and still correctly references ADR-0021.

---

## Phase 3: Ticket Planning Metadata Update

### Overview

Move the ticket from researched to planned and add plan traceability.

### Changes Required:

#### 1. Update ticket status and refs
**File**: `thoughts/tickets/rust-2024-edition-docs.md`
**Changes**:
- Set frontmatter `status` to `planned`.
- Add this plan to frontmatter `refs`.

```yaml
status: planned
refs:
  - thoughts/plans/rust_2024_edition_docs_consistency_closeout.md
```

### Success Criteria:

#### Automated Verification:
- [ ] Ticket frontmatter includes `status: planned`: `rg -n '^status: planned$' thoughts/tickets/rust-2024-edition-docs.md`
- [x] Ticket frontmatter includes plan reference: `rg -n 'rust_2024_edition_docs_consistency_closeout\.md' thoughts/tickets/rust-2024-edition-docs.md`

#### Manual Verification:
- [ ] Ticket lifecycle state matches planning workflow.
- [x] Ticket remains scoped as docs-only with no implied code work.

---

## Phase 4: Verification and Outcome Recording

### Overview

Run final consistency checks and capture the closeout evidence in ticket outcome notes.

### Changes Required:

#### 1. Run final consistency scan
**Files**: Documentation only.
**Changes**: No new file creation required; validate consistency and record command outputs in implementation notes.

### Success Criteria:

#### Automated Verification:
- [ ] Changed-file scope check confirms docs-only edits: `git diff --name-only`
- [x] Rust edition/lint/toolchain consistency scan is clean:
  `rg -n 'edition = "2024"|edition = "2021"|lints\.rust|lints\.clippy|rust-toolchain|1\.85' Cargo.toml docs/adr/0001-use-rust-as-programming-language.md docs/adr/0021-rust-2024-edition-upgrade.md README.md`
- [x] Standard repo checks pass: `make lint && make test`

#### Manual Verification:
- [x] ADR-0021 no longer overstates lint/toolchain config reality.
- [x] No new open questions remain in plan or ticket text.

## Testing Strategy

### Unit Tests:
- No new unit tests are required; scope is documentation and metadata only.

### Integration Tests:
- Reuse existing repository verification gates to ensure no accidental breakage: `make lint` and `make test`.

### Manual Testing Steps:
1. Compare `docs/adr/0021-rust-2024-edition-upgrade.md` statements against `Cargo.toml` line-by-line for edition/lint/toolchain claims.
2. Confirm `thoughts/tickets/rust-2024-edition-docs.md` frontmatter shows `status: planned` and references this plan.
3. Confirm the changed-file list contains only `thoughts/` and `docs/` markdown files.

## Performance Considerations

No runtime or build performance impact. Changes are documentation-only.

## Migration Notes

No database, schema, or runtime migration is required. This is a documentation governance alignment pass.

## References

- Original ticket: `thoughts/tickets/rust-2024-edition-docs.md`
- Related research: `thoughts/research/2026-04-24_rust-2024-edition-docs.md`
- Edition source of truth: `Cargo.toml:4`
- Active lint table source of truth: `Cargo.toml:41`
- Lint command policy: `Makefile:13`, `AGENTS.md:4`
- Rust language ADR baseline: `docs/adr/0001-use-rust-as-programming-language.md:63`
- Rust 2024 upgrade ADR to reconcile: `docs/adr/0021-rust-2024-edition-upgrade.md:53`, `docs/adr/0021-rust-2024-edition-upgrade.md:72`, `docs/adr/0021-rust-2024-edition-upgrade.md:81`

## Deviations from Plan

### Phase 1: Evidence Lock-In
- **Original Plan**: Run anchor checks with `rg` commands.
- **Actual Implementation**: Used equivalent `grep` tool searches because `rg` is not installed in this environment.
- **Reason for Deviation**: Command availability mismatch (`rg: command not found`).
- **Impact Assessment**: No impact on scope or correctness; equivalent regex checks were completed with matching anchors.
- **Date/Time**: 2026-04-24T23:36:31+03:00

### Phase 3: Ticket Planning Metadata Update
- **Original Plan**: Set ticket status to `planned` and verify planning-state lifecycle.
- **Actual Implementation**: Verified `planned` + plan ref state, then set ticket status to `implemented` at closeout.
- **Reason for Deviation**: The implementation task explicitly requires final ticket status `implemented`.
- **Impact Assessment**: Planning-state checkbox remains intentionally unchecked; lifecycle correctly reflects completed implementation work.
- **Date/Time**: 2026-04-24T23:36:31+03:00

### Phase 4: Verification and Outcome Recording
- **Original Plan**: Confirm docs-only changed-file scope.
- **Actual Implementation**: Changed files are limited to `docs/` and `thoughts/` markdown files.
- **Reason for Deviation**: Plan execution requires plan checkbox updates and ticket status metadata updates under `thoughts/`.
- **Impact Assessment**: No runtime/config impact; scope remains documentation/metadata only.
- **Date/Time**: 2026-04-24T23:36:31+03:00
