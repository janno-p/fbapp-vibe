---
title: Country flag emoji for teams
source: .claude/tasks/done/0036-country-flag-images.md
source_id: 0036
source_status: done
source_title: Country flag emoji for teams
status: done
type: feature
adrs: [0005, 0007, 0019]
refs: [0026]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Summary

Every team displayed on the site should show its country flag alongside the team name. Flags must be custom-designed static SVG assets committed to the repo — not raw external URLs from the football API. The design should be consistent and polished (think UEFA Euro / FIFA tournament pages): uniform 4:3 rounded-rectangle frames with the real country flag at full fidelity, styled to read well on the dark pitch background. Wherever a team name appears (fixtures, match breakdown, nearest match preview, knockout predictions), the flag appears at a uniform `w-8 h-6` (32×24 px) inline size.

## Acceptance Criteria

- [ ] `teams.tla` column exists and is populated by the admin seeding flow
- [ ] `/assets/flags/{tla}.svg` files exist for every national team in the active tournament (lowercase TLA, e.g. `eng.svg`, `fra.svg`)
- [ ] A `/assets/flags/_unknown.svg` placeholder exists for teams with no matching flag file
- [ ] Flags are visible next to team names on: fixtures list, match breakdown header, nearest match card (standings index), knockout team picker (predictions)
- [ ] Flag images are uniform: `w-8 h-6 rounded object-cover border border-pitch-700` — consistent across all four surfaces
- [ ] Teams seeded before the migration fall back gracefully to the `_unknown.svg` placeholder (no crash, no broken image)
- [ ] Re-seeding via the admin panel populates `tla` for existing teams

## Implementation Context

### Relevant files

- `migrations/` — add `0007_team_tla.sql`: `ALTER TABLE teams ADD COLUMN IF NOT EXISTS tla TEXT;`
- `src/modules/admin/db.rs` — `upsert_team` function: add `tla` to INSERT/UPDATE; bind `team.tla.as_deref()` as new parameter
- `src/football_api.rs` — `Team` struct already has `pub tla: Option<String>` — no change needed
- `src/modules/standings/db.rs` — queries that SELECT team data: add `teams.tla` column to every query that feeds `MatchInfo`, `FixtureRow`, `NearestMatch`
- `src/modules/standings/models.rs` — add `home_tla: Option<String>` / `away_tla: Option<String>` to `MatchInfo`, `FixtureRow`, `NearestMatch`; add `tla: Option<String>` to `TeamInfo` in predictions
- `src/modules/predictions/db.rs` — query that fetches knockout teams: add `teams.tla`
- `src/modules/predictions/models.rs` — `TeamInfo` struct: add `tla: Option<String>`
- `templates/standings/fixtures.html` — wrap team names with flag `<img>`
- `templates/standings/match.html` — wrap home/away names with flag `<img>`
- `templates/standings/index.html` — nearest match card: wrap team names with flag `<img>`
- `templates/predictions/index.html` — knockout team picker: wrap team short_name with flag `<img>`
- `assets/flags/` — new directory; one SVG per national team + `_unknown.svg`

### ADR constraints

- **ADR-0005**: All DB queries use `sqlx::query!` / `sqlx::query_as!` macros — adding `tla` to SELECT lists must stay within that pattern
- **ADR-0007**: No new module needed; changes live within existing `standings`, `predictions`, and `admin` modules

### Flag asset design spec

Each SVG must follow this template so all flags are visually uniform:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 30">
  <defs>
    <clipPath id="r">
      <rect width="40" height="30" rx="4"/>
    </clipPath>
  </defs>
  <!-- flag content clipped to rounded rect -->
  <g clip-path="url(#r)">
    <!-- stripes, canton, emblem etc. -->
  </g>
  <!-- border on top so it renders over flag edges -->
  <rect width="40" height="30" rx="4" fill="none"
        stroke="#1e2f47" stroke-width="1"/>
</svg>
```

- `viewBox="0 40 30"` — 4:3 ratio, rendered at `w-8 h-6` = 32×24 px on page
- `rx="4"` clip path for rounded corners
- `stroke="#1e2f47"` — matches `pitch-700`, creates a subtle frame on the dark background
- No drop shadow, no glow — keep it clean
- Flag content should be faithful to the real flag (correct colors, proportions of stripes, canton position) — simplified geometry is fine, photorealism is not needed
- For complex emblems/coats of arms, a simplified shape is acceptable; the stripe pattern and color blocks are what make flags recognizable at small size

### Template pattern

Replace bare team name text with:

```html
<span class="inline-flex items-center gap-2 min-w-0">
  <img src="/assets/flags/{{ home_tla|lower }}.svg"
       alt="{{ match_info.home_name }}"
       class="w-8 h-6 rounded object-cover shrink-0"
       loading="lazy">
  <span class="truncate">{{ match_info.home_name }}</span>
</span>
```

For the `_unknown.svg` fallback: Askama can't call arbitrary Rust functions in templates, so compute a `home_flag` / `away_flag` `String` field in the handler/model rather than computing the path in the template. Add a free function:

```rust
pub fn flag_path(tla: &Option<String>) -> String {
    match tla.as_deref().map(|s| s.to_lowercase()) {
        Some(t) if !t.is_empty() => format!("/assets/flags/{t}.svg"),
        _ => "/assets/flags/_unknown.svg".to_string(),
    }
}
```

Call it when building `MatchInfo`, `FixtureRow`, etc., storing result as `home_flag: String` / `away_flag: String` on the struct. Templates then use `{{ m.home_flag }}` directly — no logic in the template.

### Unknown placeholder design

`_unknown.svg` — neutral gray rectangle with a "?" or a simple globe icon, styled in ink-500 on pitch-800 background, same `rx="4"` rounded corners and `pitch-700` border.

### Tests

- No new unit tests required — `flag_path()` is a trivial string function; the migration is a safe DDL change
- Manually verify: seed a tournament via the admin panel, confirm `tla` is populated in the DB, confirm flags render on the fixtures and match pages

### Implementation notes

- **Seeding existing data:** Teams seeded before this migration have `tla = NULL`. A re-seed via `/admin` → "Seed teams" will call `upsert_team` again with the ON CONFLICT update, filling in `tla`. No data migration script needed.
- **TLA casing from the API:** `football_api.rs` returns `tla` as the API sends it (usually uppercase, e.g. `"ENG"`). Store as-is in the DB; convert to lowercase only at the `flag_path()` call site.
- **Out of scope:** Club team crests (this task is only for national team tournaments). If the app ever supports club tournaments, a separate task should handle crest display.
- **SVG source reference:** For accuracy, reference Wikipedia's flag SVGs or flagpedia.net as visual reference when drawing the simplified SVGs. Do not copy SVG source from Wikipedia directly (license complexity) — redraw using the correct colors and geometry.
- **Priority order for flag SVGs:** Create flags for the teams in the currently active tournament first. A full set can follow as future polish.

## Outcome

Implemented using Unicode flag emoji instead of SVG image assets (spec was revised before implementation began).

- Added `migrations/0012_team_tla.sql` — `ALTER TABLE teams ADD COLUMN IF NOT EXISTS tla TEXT;`
- Created `src/flags.rs` — `flag_emoji(tla)` with TLA→ISO-alpha2 map, UK tag-sequence literals, `🏳` fallback; unit-tested
- Updated `src/modules/admin/db.rs` — `upsert_team` now persists `tla`
- Updated `src/modules/standings/db.rs` — all three queries (`get_nearest_match`, `get_match_info`, `get_all_fixtures`) fetch `ht.tla`/`at.tla` and pre-compute `home_emoji`/`away_emoji`
- Updated `src/modules/standings/models.rs` — added `home_emoji`/`away_emoji` fields to `MatchInfo`, `NearestMatch`, `FixtureRow`
- Updated `src/modules/predictions/models.rs` — added `emoji` to `TeamInfo`; added `home_emoji`/`away_emoji` to `MatchRow`
- Updated `src/modules/predictions/db.rs` — `get_teams` and `get_group_matches_with_predictions` fetch TLA and compute emoji
- Updated 4 templates (`standings/fixtures.html`, `standings/match.html`, `standings/index.html`, `predictions/index.html`) — emoji rendered as `<span class="text-xl leading-none shrink-0">` beside team names
- Created ADR-0019 documenting the emoji-vs-SVG trade-off

Deviations from original spec: used emoji instead of SVG files; no `assets/flags/` directory created; no `_unknown.svg` placeholder needed (falls back to `🏳`).

Follow-up tasks: _none_
