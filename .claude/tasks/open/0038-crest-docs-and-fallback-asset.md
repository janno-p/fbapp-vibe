---
id: 0038
title: Update crest/flag docs and add fallback crest asset
status: open
phase: MVP
type: chore
adrs: [0019]
refs: [0036]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Task 0036 was implemented using crest images from the Football Data API instead of the originally planned SVG flag assets or Unicode emoji. ADR-0019 (which documents the emoji approach) is now stale and misleading. Additionally, `src/crests.rs` already references `/assets/default.svg` as the fallback for teams without a crest URL, but that file does not yet exist. This task brings the docs and assets in sync with the actual implementation.

## Acceptance Criteria

- [ ] `docs/adr/0019-unicode-emoji-country-flags.md` is marked as **Superseded by ADR-0022** in its Status section
- [ ] `docs/adr/0022-team-crest-images.md` is created, documenting the current approach
- [ ] `assets/default.svg` exists — a square, generic placeholder used when no crest URL is available
- [ ] `cargo test` continues to pass (no code changes expected)

## Context for Claude 🤖

### Relevant files

- `docs/adr/0019-unicode-emoji-country-flags.md` — change Status to `⛔ Superseded by ADR-0022`; add a one-line note at the top pointing to ADR-0022
- `docs/adr/0022-team-crest-images.md` — new file to create (see format below)
- `assets/default.svg` — new file to create (see spec below)
- `src/crests.rs` — already implemented; `DEFAULT_CREST_URL = "/assets/default.svg"` and `find_crest_url()` with fallback; **no changes needed**
- `src/lib.rs` — `mod crests;` should already be declared; verify only

### ADR-0022 content

Follow the existing ADR style (sections: Status, Date, Context, Decision, Rationale, Trade-offs and Risks, Consequences; use emojis throughout). Cover:

- **Context**: task 0036 originally planned SVG flag assets, then pivoted to Unicode emoji (ADR-0019), then pivoted again to API-provided crest images
- **Decision**: serve team crests directly from Football Data API URLs stored in `teams.crest_url`; fall back to `/assets/default.svg` when the column is NULL
- **Implementation**:
  - `teams.crest_url TEXT` column populated by `upsert_team` in `src/modules/admin/db.rs`
  - `src/crests.rs` — `find_crest_url(Option<&str>) -> String` resolves NULL to the local fallback path
  - Templates render crests via CSS custom property: `style="--crest-url: url('{{ m.home_crest_url }}')"` with a circular mask applied via Tailwind utility classes
- **Rationale**: zero authoring cost, always up-to-date as API provides official club/nation crests, works for both national team and club tournaments
- **Trade-offs**: external image dependency (API CDN must be reachable), no offline support, crest quality/style varies by team
- **Consequences**: `src/flags.rs` was deleted; `src/crests.rs` replaced it; ADR-0019 is superseded

### `assets/default.svg` spec

- **Square** viewBox (`viewBox="0 0 40 40"`)
- Neutral dark background fill matching the app's pitch colour (`#1e2f47` or similar)
- A simple centred icon — a plain shield outline or football circle in a muted lighter colour (`#4a6080` or similar)
- No text, no emoji, no complex geometry
- Same subtle border style as real crests: `stroke="#1e2f47"` (or slightly lighter for contrast) with `stroke-width="1"`
- Consistent with the circular mask applied by the templates (`mask-radial-[circle_at_center,...]`) — keep the icon centred so it looks good when cropped to a circle

### Tests

- No new tests — `src/crests.rs` already has unit tests for the fallback path; the SVG is a static asset

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
