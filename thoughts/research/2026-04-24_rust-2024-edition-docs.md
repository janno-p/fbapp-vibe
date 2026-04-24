---
date: 2026-04-24T23:06:50+03:00
git_commit: 30783b1c1c04e5cd384d682523d5c95d7a9d2878
branch: main
repository: fbapp-vibe
topic: "Rust 2024 edition documentation consistency and codebase alignment"
tags: [research, codebase, rust, adr, documentation]
last_updated: 2026-04-24
---

## Ticket Synopsis

Ticket `thoughts/tickets/rust-2024-edition-docs.md` documents a docs-only follow-up after the codebase had already migrated from Rust edition 2021 to 2024. The requested outcomes were: update ADR-0001, create ADR-0021, and sweep markdown files for stale edition-2021 references.

## Summary

The live crate configuration and ADR set are mostly aligned on Rust edition 2024, with `Cargo.toml` as the canonical source (`Cargo.toml:4`) and ADR-0001 linking to ADR-0021 (`docs/adr/0001-use-rust-as-programming-language.md:63`).

The most important gap found is an inconsistency inside ADR-0021: it claims the project already uses both `[lints.clippy]` and `[lints.rust]`, but only `[lints.clippy]` exists in `Cargo.toml` (`Cargo.toml:41`). There is also no in-repo toolchain pin file, despite ADR-0021 discussing a 1.85+ requirement (`docs/adr/0021-rust-2024-edition-upgrade.md:72`).

## Detailed Findings

### Rust Edition Source of Truth

- Crate edition is explicitly `2024` in package metadata (`Cargo.toml:4`).
- ADR-0001 consequences now reflect edition 2024 and link forward to ADR-0021 (`docs/adr/0001-use-rust-as-programming-language.md:63`).
- ADR-0021 records migration context and consequences as the dedicated upgrade decision record (`docs/adr/0021-rust-2024-edition-upgrade.md:1`, `docs/adr/0021-rust-2024-edition-upgrade.md:77`).

### Documentation Consistency Across Entry Points

- README identifies Rust as language and links to ADR-0001, but does not explicitly mention edition 2024 or ADR-0021 (`README.md:9`, `README.md:191`).
- AGENTS workflow guidance is Rust-first and lint-command specific, but edition-neutral (`AGENTS.md:3`, `AGENTS.md:4`).
- Generated overview includes a historical ticket summary saying docs were missing at that time (`docs/ticket-overview.md:245`), which is expected because it intentionally includes completed ticket history (`docs/ticket-overview.md:18`).

### ADR-0021 Accuracy Check

- **Mismatch:** ADR-0021 states both `[lints.clippy]` and `[lints.rust]` are already used (`docs/adr/0021-rust-2024-edition-upgrade.md:53`), but `Cargo.toml` only has `[lints.clippy]` (`Cargo.toml:41`).
- Lint policy is still strongly enforced through command workflow (`Makefile:13`, `AGENTS.md:4`), so this is a docs-accuracy issue, not an enforcement failure.
- ADR-0021 says toolchain should pin to 1.85+ (`docs/adr/0021-rust-2024-edition-upgrade.md:72`), but no `rust-toolchain` or `rust-toolchain.toml` is present in repository root.

### Code Pattern Validation for 2024 Notes

- Axum handlers heavily use `Result<impl IntoResponse, AppError>` RPIT style and compile with owned extractor data, supporting the ADR claim that current signatures are unaffected (`src/modules/auth/handlers.rs:57`, `src/modules/standings/handlers.rs:187`).
- Async trait/extractor implementations are actively used (`src/modules/auth/mod.rs:34`, `src/extractors.rs:20`, `src/modules/admin/mod.rs:26`), matching ADR-0021 discussion of async ergonomics.
- No FFI pattern (`extern "C"` / `unsafe extern`) was found in current Rust sources, consistent with ADR-0021's non-applicability statement (`docs/adr/0021-rust-2024-edition-upgrade.md:49`).

## Code References

- `Cargo.toml:4` - Active crate edition is `2024`.
- `Cargo.toml:41` - Only `[lints.clippy]` table exists.
- `docs/adr/0001-use-rust-as-programming-language.md:63` - ADR-0001 consequence explicitly states edition 2024 and links ADR-0021.
- `docs/adr/0021-rust-2024-edition-upgrade.md:53` - States both lint tables are in use (currently inaccurate).
- `docs/adr/0021-rust-2024-edition-upgrade.md:72` - Notes Rust 1.85+ toolchain expectation.
- `README.md:191` - Architecture decisions section links ADR-0001.
- `AGENTS.md:4` - Canonical lint workflow command.
- `Makefile:13` - Lint command enforcement (`fmt` + clippy `-D warnings`).
- `src/modules/auth/handlers.rs:57` - Async handler RPIT return style.
- `src/modules/standings/handlers.rs:187` - Async handler RPIT return style in standings module.
- `src/extractors.rs:20` - Async custom extractor implementation.
- `src/modules/admin/mod.rs:26` - Async `FromRequestParts` extractor.

## Architecture Insights

Rust edition state follows a strong hierarchy in practice: compiler-facing config in `Cargo.toml` is authoritative, ADRs describe rationale/history, and README/AGENTS provide operational guidance. This keeps runtime behavior stable even when narrative docs drift slightly.

The codebase's handler/extractor design (owned values, straightforward async boundaries, RPIT response types) minimizes exposure to the stricter RPIT lifetime-capture semantics introduced in edition 2024.

The largest remaining architecture-doc risk is not runtime correctness but governance drift: ADR assertions about lint/table/toolchain details can age out unless checked against concrete repository config.

## Historical Context (from thoughts/)

- `thoughts/tickets/rust-2024-edition-docs.md` - Ticket records the docs-only migration follow-up and marks all acceptance criteria complete.
- `thoughts/tickets/project-scaffold.md:71` - Later scaffold ticket context references ADR-0021 and notes current edition 2024.
- `thoughts/plans/project_scaffold_closeout.md:64` - Closeout plan explicitly calls for builder compatibility with edition 2024.
- `thoughts/reviews/cavekit_user_model_gap_closure-review.md:25` - Documents ticket lifecycle convention including `researched -> planned -> implemented -> reviewed`.

## Related Research

- `thoughts/research/2026-04-24_project_scaffold.md` - Broader scaffold consistency analysis that overlaps with Rust/tooling/documentation drift.
- `thoughts/research/2026-04-24_qsform_body_limit.md` - Example of current research-document structure and citation style used in this repository.

## Open Questions

- Should ADR-0021 be corrected to remove or add `[lints.rust]` so docs and `Cargo.toml` match?
- Should repository-level toolchain pinning (`rust-toolchain.toml`) or `rust-version` in `Cargo.toml` be added to operationalize the documented 1.85+ expectation?
- Should README architecture section also link ADR-0021 directly for edition discoverability?
