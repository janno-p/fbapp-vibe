## Status

⛔ Superseded by [ADR-0022](0022-team-crest-images.md)

> The Unicode emoji approach was replaced by API-provided crest images in task 0036. See ADR-0022 for the current implementation.

## Date

2026-04-08

## Context

Every team displayed on the site should show its country flag alongside the team name, to match the visual style of tournament pages. The original task spec called for custom-designed SVG assets committed to the repo — one per national team. However this was revised: SVG authoring overhead is high, assets would need maintenance as team rosters change, and the visual result depends heavily on SVG quality. Unicode flag emoji are an attractive alternative.

## Decision

🏳 Use **Unicode flag emoji** rendered as inline `<span>` elements instead of SVG image assets.

The implementation:

1. 📦 Added `teams.tla` column via migration `0012_team_tla.sql`.
2. 🦀 Created `src/flags.rs` — a pure Rust module with a `flag_emoji(tla: Option<&str>) -> String` function:
   - UK constituent nations (ENG, SCO, WAL, NIR) use tag-sequence emoji (`🏴󠁧󠁢󠁥󠁮󠁧󠁿` etc.) hardcoded as string literals.
   - All other nations map via `tla_to_alpha2()` (football TLA → ISO 3166-1 alpha-2) then `alpha2_to_emoji()` (alpha-2 → regional indicator pair at `U+1F1E6 + offset`).
   - Unknown/null TLA falls back to `🏳` (white flag).
3. 🔗 Pre-compute emoji strings in Rust (in `db.rs` mapping closures), store as `home_emoji`/`away_emoji`/`emoji` fields on view-model structs — Askama templates cannot call arbitrary Rust functions.
4. 🖼️ Templates use `<span class="text-xl leading-none shrink-0">{{ m.home_emoji }}</span>` alongside the team name.

## Rationale

| Concern | SVG assets | Unicode emoji |
|---------|-----------|---------------|
| 📁 No new files | ✗ Many SVG files | ✓ Zero extra assets |
| 🔧 Maintenance | High — SVGs need updating | None — OS/browser renders |
| 🎨 Visual fidelity | Depends on SVG quality | OS-native, familiar |
| ♿ Accessibility | Needs explicit `alt` text | Native emoji semantics |
| 📦 Bundle size | Grows with team count | Zero |
| 🌍 Coverage | Only committed teams | All nations |
| 🏴 UK sub-nations | Custom SVGs | Tag-sequence emoji ✓ |

## Trade-offs and Risks

- ⚠️ **Rendering varies by OS/font** — emoji appearance differs between Android, iOS, Windows, and Linux. This is acceptable for a friend-group app.
- ⚠️ **Kosovo (`XK`)** uses an unofficial alpha-2 code not recognized by all systems; it may fall back to `🏳` on some platforms.
- ⚠️ **Emoji sizing** relies on `font-size`/`line-height` — layouts use `text-xl leading-none` or `text-2xl leading-none` which is consistent but not pixel-perfect like SVG.

## Consequences

- ✅ Zero asset files to maintain.
- ✅ `src/flags.rs` is fully unit-tested (known TLAs, UK nations, unknown fallback).
- ✅ `cargo test` passes with no changes to the test suite structure.
- ℹ️ If club tournament support is added later, a separate approach will be needed (clubs don't have country TLAs).
