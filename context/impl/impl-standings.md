---
created: "2026-04-15T00:00:00Z"
last_edited: "2026-04-15T00:00:00Z"
---
# Implementation Tracking: Standings

Build site: context/plans/build-site.md

| Task | Status | Notes |
|------|--------|-------|
| T-028 | DONE | Added `remaining_possible` and `ceiling_band` fields to `LeaderboardEntry`; pure `assign_ceiling_bands()` function; 7 unit tests. `src/modules/standings/models.rs`. |
| T-029 | DONE | Updated Max cell in `templates/standings/leaderboard.html` to stack: max_achievable, `+N left`, and MDI chevron icon per ceiling_band. Added MDI `@source inline` directives to `assets/css/input.css`. Applies to both main page (via include) and HTMX fragment. |
