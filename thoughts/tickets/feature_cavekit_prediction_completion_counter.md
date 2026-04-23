---
type: feature
priority: low
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, htmx, ux]
keywords: [prediction completion counter, predicted matches, total matches, htmx fragment, group stage tab]
patterns: [server-derived counters, HTMX fragment update, completion state indicator, form progress UI]
---

# FEATURE-PREDICTIONS-07: Prediction completion counter

## Summary

Show a group-stage progress counter so users can see how many matches they have predicted out of the total.

## Acceptance Criteria

- [ ] The group stage tab shows a `predicted / total` counter.
- [ ] The counter is computed from server-side state on page load.
- [ ] The counter updates as predictions are added or removed.
- [ ] The counter shows a distinct complete state when all matches are predicted.
- [ ] The counter is hidden when predictions are locked.
- [ ] The count reflects actual rows in `group_stage_predictions` for the current user.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — pass counter values into the template
- `src/modules/predictions/db.rs` — count predicted matches
- `templates/predictions/index.html` — render the counter and locked state

### ADR constraints

- **ADR-0007**: Keep the UI inside the predictions module.
- **ADR-0005**: Use checked SQL for the count query.

### Tests

- [ ] Query test for accurate predicted match counts.
- [ ] Template/route test for hidden counter state when locked.

### Implementation notes

- Use the server as the source of truth for count values.
- HTMX can refresh only the counter fragment after a save.

## Research Context

### Keywords to Search

- prediction completion counter - UX metric
- predicted matches - user progress count
- total matches - denominator
- HTMX fragment - partial update pattern
- complete state - visual status

### Patterns to Investigate

- server-derived counters - count from DB, not client state
- HTMX fragment update - partial refresh of a single widget
- completion state indicator - success styling when finished
- form progress UI - lightweight progress feedback

### Key Decisions Made

- The counter is group-stage only.
- It should disappear once predictions are locked.
- The counter is informational, not a blocker.

## Success Criteria

The ticket is complete when users can see accurate prediction progress on the group stage tab.

### Automated Verification

- [ ] `cargo test` covers the count query.
- [ ] Render test covers the complete and locked states.

### Manual Verification

- [ ] The displayed count changes after saving a prediction.
- [ ] The counter disappears after lock.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R7`

## Notes

Do not add per-user statistics or broader analytics here.
