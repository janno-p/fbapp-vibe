# ADR-0021: Upgrade to Rust Edition 2024 🦀

## Status

✅ Accepted

## Date

2026-04-08

## Context

Rust releases a new **edition** roughly every three years. An edition is a compatibility-breaking opt-in layer on top of the language: each edition can introduce new keywords, change default semantics, or clean up legacy behaviour, while the compiler continues to compile all editions in the same binary. Crates choose their edition independently in `Cargo.toml`.

The project launched on **edition 2021** (see [ADR-0001](0001-use-rust-as-programming-language.md)). Rust edition 2024 was stabilised with Rust 1.85 (released February 2025). Edition 2024 is the most recent stable edition and the recommended default for new and updated projects.

The migration was performed in commit `1692d98` alongside fixing the Clippy warning set and formatting the full codebase.

**Why upgrade at all?**

| Reason | Detail |
|--------|--------|
| 🏗️ **Stay current** | Edition 2024 is the stable default; staying on 2021 means tracking the diff indefinitely |
| 🔧 **Async ergonomics** | Edition 2024 improves `async fn` in traits and RPIT lifetime capture, aligning better with Axum's async handler patterns |
| 📦 **`[lints]` table** | Stable in edition 2024 and already used in this project's `Cargo.toml` |
| 🛡️ **Forward-compatibility** | New language features that require edition 2024 semantics become available without a future migration |

## Decision

🦀 Upgrade `Cargo.toml` from `edition = "2021"` to `edition = "2024"`.

The upgrade was carried out mechanically via `cargo fix --edition` followed by manual review. No runtime behaviour changed.

## Rationale

### Notable edition 2024 changes and their relevance to this project

1. **`gen` keyword reserved** 🔑
   - `gen` is a reserved keyword in edition 2024 (for future generator/coroutine syntax).
   - This codebase does not use `gen` as an identifier, so no code changes were required.

2. **Stricter `impl Trait` lifetime capture** ⏳
   - Return-position `impl Trait` (RPIT) in edition 2024 captures *all* in-scope lifetimes by default, not just those explicitly referenced.
   - This is a tighter, more correct default. Signatures that need to exclude a lifetime can opt out with `+ use<'a>` bounds.
   - Current Axum handler and extractor signatures in this project are unaffected (they either own their data or use `'static` bounds).

3. **`unsafe extern` blocks** ⚠️
   - In edition 2024, `extern "C"` blocks containing unsafe items must be written as `unsafe extern { ... }`.
   - This project has no FFI (`unsafe extern` blocks), so no changes were needed.

4. **`[lints]` table stability** 📦
   - The `[lints]` table in `Cargo.toml` became stable and is the recommended way to configure `rustc` and `clippy` lints in edition 2024.
   - This project already uses `[lints.clippy]` and `[lints.rust]` in `Cargo.toml`, so this change is already in effect.

5. **Improved `async` ergonomics** 🔄
   - Edition 2024 refines how `async fn` in traits and async closures interact with lifetime capture (see point 2).
   - Axum handler functions (`async fn handler(...)`) and extractor implementations benefit from the more predictable capture rules.

### Migration effort

The upgrade required only compiler-driven changes (`cargo fix --edition`). No manual logic rewrites were needed. The primary mechanical changes were:

- Formatting adjustments (`rustfmt` re-ran with edition 2024 rules)
- Minor Clippy warning fixes unrelated to the edition itself

## Trade-offs and Risks ⚠️

| Trade-off | Mitigation |
|-----------|-----------|
| 🔑 **`gen` keyword reservation** | No identifier named `gen` exists in the codebase. Future contributors cannot use `gen` as a name. |
| ⏳ **Stricter lifetime capture may break dependencies** | Only affects this crate's own code; dependency crates choose their own edition. All existing code compiled cleanly. |
| 🔁 **Edition 2024 requires Rust 1.85+** | `rust-toolchain.toml` (or CI) must pin to ≥ 1.85. This is a well-supported release. |
| 🔮 **Unknown future changes** | Rust's stability guarantees mean edition 2024 code will continue to compile as the toolchain evolves. No known breaking changes on the horizon. |

## Consequences

- ✅ `Cargo.toml` has `edition = "2024"`.
- ✅ The `[lints]` table in `Cargo.toml` is the authoritative lint configuration.
- ✅ `gen` is unavailable as an identifier in new code.
- ✅ New async handler patterns can rely on edition 2024's improved RPIT lifetime capture semantics.
- ℹ️ The project requires a Rust toolchain of version 1.85 or newer.
- ℹ️ External crates continue to use their own declared edition; only `fbapp_vibe` itself uses edition 2024.
