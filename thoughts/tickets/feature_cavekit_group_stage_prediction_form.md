---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, group-stage, htmx]
keywords: [group stage predictions, home draw away, prediction lock, group_stage_predictions, active tournament]
patterns: [tabbed form rendering, per-match upsert, lock-gated form state, grouped match listing]
---

# FEATURE-PREDICTIONS-01: Group stage prediction form

## Summary

Allow authenticated users to submit and update home/draw/away predictions for every group stage match in the active tournament.

## Acceptance Criteria

- [ ] `GET /predictions` renders a group stage tab.
- [ ] Matches are grouped by group and show home team, away team, and kickoff time.
- [ ] Each match has a home/draw/away control.
- [ ] `POST /predictions/group` saves all submitted group stage predictions.
- [ ] Duplicate submissions update the existing prediction for the same match.
- [ ] Stored predictions include `user_id`, `match_id`, `predicted_outcome`, and nullable `points_awarded`.
- [ ] The form is read-only when `predictions_locked_at` is set.
- [ ] Users can change predictions before lock and see their saved values pre-filled.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — page render and group save handler
- `src/modules/predictions/db.rs` — group prediction queries and upsert logic
- `src/modules/predictions/models.rs` — form and display models
- `templates/predictions/index.html` — group stage tab UI
- `migrations/0007_predictions.sql` — `group_stage_predictions` table

### ADR constraints

- **ADR-0007**: Keep the feature inside `src/modules/predictions/` and expose only `router()`.
- **ADR-0009**: Use `Result<impl IntoResponse, AppError>` for handlers.
- **ADR-0005**: Use compile-time checked `sqlx` queries.

### Tests

- [ ] `#[sqlx::test]` for group prediction upsert behavior.
- [ ] Integration test for rendering the group stage tab with match data.
- [ ] Integration test for locked-state read-only rendering.

### Implementation notes

- Keep the update flow partial-save friendly.
- Do not add scoring logic here; `points_awarded` stays unset until the scoring pass runs.

## Research Context

### Keywords to Search

- `group_stage_predictions` - storage table
- `predictions_locked_at` - lock state gate
- `POST /predictions/group` - save endpoint
- group stage tab - UI entry point
- home draw away - outcome control values

### Patterns to Investigate

- tabbed form rendering - current predictions page structure
- per-match upsert - duplicate prediction update flow
- lock-gated form state - read-only rendering when locked
- grouped match listing - how matches are ordered by group

### Key Decisions Made

- Group stage predictions are editable until the tournament lock.
- Existing predictions should be pre-filled instead of forcing a fresh submit.
- The ticket stays focused on group stage only.

## Success Criteria

The ticket is complete when a user can submit, update, and view locked/unlocked group stage predictions correctly.

### Automated Verification

- [ ] `cargo test` covers the upsert and lock-state behavior.
- [ ] Page render test confirms grouped match output.

### Manual Verification

- [ ] Group stage matches render with the expected controls.
- [ ] Updates replace the prior prediction for the same match.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R1`

## Notes

Do not expand this into knockout, top scorer, or review-page work.
