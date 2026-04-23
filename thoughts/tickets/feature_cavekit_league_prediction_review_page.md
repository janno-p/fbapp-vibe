---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, leagues, review]
keywords: [league prediction review, predictions vs actual, league members, points_awarded, review page]
patterns: [league-scoped review page, joined predictions/results query, per-member comparison tables, access control]
---

# FEATURE-PREDICTIONS-06: League prediction review page

## Summary

Give league members a per-league review page that compares every member's predictions with actual results and points.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/predictions/review` renders a league review page.
- [ ] The page shows league name, tournament name, and review data.
- [ ] Group stage, knockout, and top scorer predictions are all represented.
- [ ] Each row shows prediction, actual result when available, and points awarded.
- [ ] Pending scores show `—` or an equivalent placeholder.
- [ ] The page is accessible only to league members.
- [ ] The page only shows members from the requested league.
- [ ] The page works before and after lock, with the post-lock reveal being the primary shared view.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — review handler
- `src/modules/predictions/db.rs` — joins for predictions, matches, players, and points
- `src/modules/predictions/models.rs` — review row and aggregate types
- `templates/predictions/review.html` — review page template
- `src/modules/leagues/db.rs` — membership check reuse

### ADR constraints

- **ADR-0007**: Keep the review route in the predictions module.
- **ADR-0009**: Return unauthorized/forbidden responses for non-members.
- **ADR-0005**: Use checked SQL for the review queries.

### Tests

- [ ] Integration test for league-member access control.
- [ ] Query test for review data shaping across all three prediction types.

### Implementation notes

- The page should be league-specific, not a global predictions dump.
- Keep the review data tied to the active tournament and the current league membership set.

## Research Context

### Keywords to Search

- `/leagues/{id}/predictions/review` - review route
- league prediction review - feature scope
- points_awarded - scoring display field
- league members - access boundary
- predictions vs actual - comparison data

### Patterns to Investigate

- league-scoped review page - membership-gated reporting
- joined predictions/results query - aggregate display data
- per-member comparison tables - how to lay out comparisons
- access control - member-only route protection

### Key Decisions Made

- The review page is league-only.
- It should show all three prediction domains together.
- Pending results should remain visible as pending rather than hidden.

## Success Criteria

The ticket is complete when league members can review all predictions against results for the active tournament.

### Automated Verification

- [ ] `cargo test` covers league access control.
- [ ] Review query tests return the expected joined rows.

### Manual Verification

- [ ] League members can open the page.
- [ ] Non-members are blocked.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R6`

## Notes

Do not add export, historical archives, or per-user accuracy stats here.
