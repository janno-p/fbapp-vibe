---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, knockout, htmx]
keywords: [knockout predictions, round predictions, bracket, knockout_predictions, active tournament]
patterns: [round-based form rendering, prefilled selections, per-round save endpoint, unavailable-round handling]
---

# FEATURE-PREDICTIONS-02: Knockout prediction form

## Summary

Allow users to predict which teams advance through the tournament knockout rounds.

## Acceptance Criteria

- [ ] `GET /predictions` renders a knockout tab.
- [ ] Each knockout round shows the teams available for that round.
- [ ] The user can select the predicted advancing team(s) for each round.
- [ ] `POST /predictions/knockout/{round}` saves predictions for one round.
- [ ] Existing round predictions are pre-filled from the database.
- [ ] Rounds with no available teams are hidden or clearly marked unavailable.
- [ ] The form is read-only when predictions are locked.
- [ ] Stored predictions include `user_id`, `tournament_id`, `round`, `team_id`, and `points_awarded`.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — knockout render and save handlers
- `src/modules/predictions/db.rs` — round queries and persistence
- `src/modules/predictions/models.rs` — round form and display models
- `templates/predictions/index.html` — knockout tab UI
- `migrations/0007_predictions.sql` — `knockout_predictions` table

### ADR constraints

- **ADR-0007**: Keep the feature inside the existing predictions module.
- **ADR-0009**: Return application errors instead of silent failure.
- **ADR-0005**: Use checked SQL for the persistence layer.

### Tests

- [ ] `#[sqlx::test]` for per-round save and replace behavior.
- [ ] Unit test for round availability/count logic if the UI needs it.
- [ ] Integration test for pre-filled knockout predictions.

### Implementation notes

- Keep the round-specific save flow isolated so one round can be updated without rewriting the others.
- If the tournament format hides certain rounds, the UI should not expose empty slots as valid predictions.

## Research Context

### Keywords to Search

- `knockout_predictions` - storage table
- `/predictions/knockout/` - save route pattern
- knockout tab - UI entry point
- bracket rounds - tournament structure
- prefilled selections - saved prediction restore

### Patterns to Investigate

- round-based form rendering - one section per knockout round
- prefilled selections - loading current state from DB
- per-round save endpoint - isolated HTMX or form submission flow
- unavailable-round handling - hide or disable empty bracket sections

### Key Decisions Made

- Knockout predictions are saved per round rather than as one giant bracket payload.
- The ticket should work with the active tournament format instead of hard-coding one bracket shape.
- This ticket stays focused on knockout advancement only.

## Success Criteria

The ticket is complete when users can save and update knockout picks for the active tournament rounds.

### Automated Verification

- [ ] `cargo test` covers save and reload behavior.
- [ ] Route test confirms the knockout tab renders for the active tournament.

### Manual Verification

- [ ] Each round shows the expected teams.
- [ ] Saved selections reappear after refresh.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R2`

## Notes

Do not add bracket simulation, eliminations logic, or shared prediction statistics here.
