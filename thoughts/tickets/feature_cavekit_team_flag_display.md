---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, tournament, flags, tailwind]
keywords: [flag, tla_to_flag, circle-flags, iconify, crest_url, @source inline]
patterns: [derived display attribute, self-hosted icon asset, template fallback, CSS source whitelisting]
---

# FEATURE-CAVEKIT-TOURNAMENT-06: Display team national flags

## Summary

Show teams with self-hosted national flags derived from their ISO country codes instead of external crest images.

## Acceptance Criteria

- [ ] `teams.flag` stores an ISO 2-letter country code.
- [ ] `teams` does not use a `crest_url` column for this display path.
- [ ] `src/national_flags.rs` provides `tla_to_flag(tla: Option<&str>) -> Option<String>`.
- [ ] Seeding writes the derived flag value when a mapping exists.
- [ ] Templates use `icon-[circle-flags--{flag_code}]` for rendered flags.
- [ ] Templates render no icon when `flag` is `NULL`.
- [ ] Iconify circle-flags is self-hosted via `@iconify/tailwind4`.
- [ ] Active-tournament flag classes are whitelisted via `@source inline(...)` in `assets/css/input.css`.

## Implementation Context

### Relevant files

- `src/national_flags.rs` - TLA to ISO mapping helper.
- `src/modules/admin/db.rs` - populate flag during seeding.
- `templates/` - team render locations.
- `assets/css/input.css` - Tailwind source directives.
- `migrations/0015_remove_team_crest_url.sql` - crest cleanup.
- `migrations/0016_team_flag.sql` - flag column addition.

### ADR constraints

- **ADR-0007**: Keep mapping and rendering support inside the existing feature modules.
- **ADR-0005**: SQLx-backed schema and model changes should stay compile-time checked.

### Tests

- Unit test for `tla_to_flag()` mappings and misses.
- Template smoke test or integration render check for the no-icon fallback.

### Implementation notes

- This ticket is about display and derived metadata, not source API changes.
- Prefer self-hosted assets over CDN references.

## Research Context

### Keywords to Search

- `flag` - team flag storage field.
- `tla_to_flag` - mapping helper.
- `circle-flags` - icon set used for display.
- `iconify` - tailwind plugin and asset source.
- `crest_url` - removed external image path.

### Patterns to Investigate

- derived display attribute - compute a render field from source data.
- self-hosted icon asset - avoid remote image dependencies.
- template fallback - render nothing when mapping is absent.
- CSS source whitelisting - include only needed icons.

### Key Decisions Made

- National flags are rendered from ISO codes only.
- Missing mappings should not break rendering.
- External crest images are out of scope.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers the mapping helper.
- [ ] `cargo build` confirms templates and CSS references still compile.

### Manual Verification

- [ ] Teams render with the expected flag icons where mappings exist.
- [ ] Teams with no mapping render without an icon.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R7.
- Depends on tournament seeding and the team model.

## Notes

- Do not reintroduce external crest URLs.
