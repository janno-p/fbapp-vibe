---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, top-scorer, search]
keywords: [top scorer predictions, player search, max 3 picks, top_scorer_predictions, goals]
patterns: [searchable selection list, multi-select cap, prefilled predictions, lock-gated form state]
---

# FEATURE-PREDICTIONS-03: Top scorer prediction form

## Summary

Allow users to pick up to three players they expect to finish as the tournament top scorer.

## Acceptance Criteria

- [ ] `GET /predictions` renders a top scorer tab.
- [ ] The tab shows a searchable list of players with name, team, and current goals.
- [ ] The user can select up to three players.
- [ ] Attempting to select more than three players shows a clear error and does not save.
- [ ] `POST /predictions/top-scorer` saves the selection set.
- [ ] Existing selections are pre-filled from the database.
- [ ] The form is read-only when predictions are locked.
- [ ] Stored predictions include `user_id`, `tournament_id`, and `player_id`.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — top scorer render and save handlers
- `src/modules/predictions/db.rs` — player query and persistence logic
- `src/modules/predictions/models.rs` — top scorer form and display models
- `templates/predictions/index.html` — top scorer tab UI
- `migrations/0007_predictions.sql` — `top_scorer_predictions` table

### ADR constraints

- **ADR-0007**: Keep the feature in the existing module.
- **ADR-0009**: Return structured errors for invalid selection counts.
- **ADR-0005**: Use checked SQL for persistence and lookups.

### Tests

- [ ] `#[sqlx::test]` for saving up to three players and rejecting four.
- [ ] Integration test for prefilled selections.
- [ ] Integration test for locked-state read-only rendering.

### Implementation notes

- Keep the client-side filtering simple; this is a searchable list, not a full search feature.
- The error for over-selection should be explicit and user-facing.

## Research Context

### Keywords to Search

- `top_scorer_predictions` - storage table
- top scorer tab - UI entry point
- player search - list filtering behavior
- max 3 picks - selection cap
- current goals - display field

### Patterns to Investigate

- searchable selection list - lightweight client filtering
- multi-select cap - enforcing the three-player limit
- prefilled predictions - loading existing rows into the view
- lock-gated form state - disable writes after lock

### Key Decisions Made

- The selection cap is three players.
- The tab should be searchable but not a separate search feature.
- This ticket covers only top scorer prediction entry.

## Success Criteria

The ticket is complete when users can save, reload, and validate top scorer picks correctly.

### Automated Verification

- [ ] `cargo test` covers the three-pick limit.
- [ ] Route/render test confirms the top scorer tab loads.

### Manual Verification

- [ ] Player search/filtering works.
- [ ] Four selections are rejected with a clear message.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R3`

## Notes

Do not add prediction statistics, export, or historical tracking here.
