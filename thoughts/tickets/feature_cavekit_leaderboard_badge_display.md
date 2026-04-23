---
type: feature
priority: low
created: 2026-04-23T00:00:00Z
status: created
tags: [ui, leaderboard, badges]
keywords: [leaderboard column, top badge, hover tooltip, most recent badge, most rare badge, empty cell]
patterns: [optional table column, tooltip presentation, derived-display selection]
---

# FEATURE-040: Optionally show a badge on the leaderboard

## Description
Add an optional leaderboard badge column that highlights each user’s most notable achievement without changing the core leaderboard behavior.

## Context
The leaderboard can surface a compact achievement signal for users, but this should remain optional so the main standings layout stays lightweight.

## Requirements
- Add an optional `Top Badge` or `Badge` column to the main leaderboard.
- Display the most notable badge earned by the user.
- Show the badge icon or emoji in the cell.
- Show badge name and description on hover.
- Leave the cell empty or show `—` when no badge exists.
- Use a consistent rule for which badge is shown if multiple badges exist.

### Functional Requirements
- Render a compact badge indicator on leaderboard rows.
- Support tooltip-style reveal of badge details.
- Handle users with no badges gracefully.

### Non-Functional Requirements
- Keep the leaderboard readable and uncluttered.
- The badge display must not break the existing standings table.

## Current State
The leaderboard does not expose badge information.

## Desired State
The leaderboard can optionally show a compact achievement badge column.

## Research Context

### Keywords to Search
- leaderboard column - table layout change
- top badge - display concept
- hover tooltip - badge details UI
- most recent badge - selection option
- most rare badge - alternate selection option
- empty cell - no-badge state

### Patterns to Investigate
- optional table column - progressive enhancement pattern
- tooltip presentation - compact metadata reveal
- derived-display selection - choosing one badge from many

### Key Decisions Made
- This is optional, not required for core badge functionality.
- The leaderboard should stay usable even if the badge column is omitted.

## Success Criteria
The ticket is complete when the leaderboard can show one badge per user without harming the existing table layout.

### Automated Verification
- [ ] Integration test covers the badge column when enabled.
- [ ] Integration test covers the empty cell state.

### Manual Verification
- [ ] Hovering a badge shows the correct metadata.
- [ ] The leaderboard remains readable with the new column.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R5`
- Depends on: badge definitions, storage, and display metadata tickets.
- Depends on: leaderboard page existing in standings.

## Notes
If this creates layout risk, keep it behind an implementation choice or omit it from the initial rollout.
