---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, results, scoring]
keywords: [match results, finished matches, correct prediction, incorrect prediction, home_score, away_score]
patterns: [read-only result display, match status gating, correctness highlighting, joined scoring data]
---

# FEATURE-PREDICTIONS-08: Show actual results on the predictions page

## Summary

Display finished match results alongside the user's predictions so they can see which picks were correct or wrong.

## Acceptance Criteria

- [ ] Finished group stage matches display the actual score.
- [ ] The user's prediction is marked correct or incorrect against the actual outcome.
- [ ] Unplayed matches still show only the scheduled kickoff time and prediction form.
- [ ] Pending or future matches do not show score data.
- [ ] Finished-match result display is read-only.
- [ ] The pre-tournament state continues to render correctly.

## Implementation Context

### Relevant files

- `src/modules/predictions/db.rs` — match queries should include result fields
- `src/modules/predictions/handlers.rs` — pass result data into the page view model
- `src/modules/predictions/models.rs` — helper for correctness display
- `templates/predictions/index.html` — show score and correctness badges

### ADR constraints

- **ADR-0005**: Use checked SQL for the match query changes.
- **ADR-0007**: Keep the result display in the predictions module.

### Tests

- [ ] Query test for finished-match result fields.
- [ ] Integration test for correct/incorrect badge rendering.

### Implementation notes

- Only show the result block when the match has finished.
- Use a read-only presentation; do not make finished matches interactive.

## Research Context

### Keywords to Search

- home_score - result field
- away_score - result field
- match results - display data
- correct prediction - success state
- incorrect prediction - failure state

### Patterns to Investigate

- read-only result display - no interaction after finish
- match status gating - show results only when finished
- correctness highlighting - green/red outcome styling
- joined scoring data - current prediction plus actual result

### Key Decisions Made

- Result display is group-stage only for this ticket.
- Finished matches should show immediate feedback on prediction correctness.
- Unplayed matches keep the existing input-focused layout.

## Success Criteria

The ticket is complete when finished matches visibly compare the user's pick to the real result.

### Automated Verification

- [ ] `cargo test` covers the result query and correctness state.
- [ ] Render test covers finished vs unplayed matches.

### Manual Verification

- [ ] Finished matches show score and correctness.
- [ ] Future matches stay scoreless.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R8`

## Notes

Do not broaden this into scoring updates or standings integration work.
