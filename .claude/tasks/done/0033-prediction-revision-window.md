---
id: 0033
title: Prediction revision window before kickoff
status: cancelled
phase: MVP
type: feature
adrs: [0005, 0007, 0009]
refs: [0007]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Currently predictions can only be submitted once before the tournament locks. Many real-world prediction games allow updates up until shortly before kickoff (e.g., 15 minutes before the match starts). A per-match revision window lets users correct their group stage predictions based on team news or late information, making the game more engaging without compromising fairness.

## Acceptance Criteria

- [ ] A user can resubmit a group stage match prediction if `NOW() < match.scheduled_at - INTERVAL '15 minutes'` AND the tournament is not globally locked
- [ ] After the per-match deadline, the prediction form for that match is rendered as read-only (disabled inputs, no submit button)
- [ ] The prediction index page shows each match's individual deadline clearly ("Closes 14 Jun 17:45 UTC") in addition to the global lock state
- [ ] Resubmission is an upsert — same `INSERT ... ON CONFLICT DO UPDATE` pattern already used
- [ ] Attempting to POST after a match's deadline returns a `400 Bad Request` with a user-friendly message ("Predictions for this match are closed")
- [ ] No changes to knockout or top scorer predictions — those remain globally locked only

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/handlers.rs` — update group stage POST handler to check `scheduled_at - 15 min > NOW()` per submitted match; return `AppError::BadRequest` if any submitted match is past deadline
- `src/modules/predictions/db.rs` — the deadline check can be done in Rust after fetching `scheduled_at` values (already loaded), or as a SQL condition
- `templates/predictions/index.html` — per-match deadline display and disabled state for past-deadline matches
- `src/modules/standings/models.rs:MatchInfo` — already has `scheduled_at`; a `is_prediction_open()` method (returns true if >15 min before kickoff) would be useful if MatchInfo is reused here; alternatively add the same method to the predictions model

### ADR constraints

- **ADR-0005**: Deadline check in application layer is fine (no need to add a DB-side constraint)
- **ADR-0009**: Return `AppError::BadRequest` for past-deadline submission attempts; do not silently ignore them

### Tests

- Unit test `is_prediction_open(scheduled_at, now)` with cases: 20 min before (open), 15 min before (boundary — closed), 10 min before (closed), after kickoff (closed)
- No DB tests

### Implementation notes

- Time arithmetic in Rust with `time` crate: `scheduled_at - Duration::minutes(15) > OffsetDateTime::now_utc()`
- The 15-minute window is a constant; define it as `const PREDICTION_CLOSE_BEFORE_KICKOFF: time::Duration = time::Duration::minutes(15);`
- Batch POST handler: the existing group stage form likely submits all predictions in one POST. Check each match's deadline individually; if any are closed, return an error listing which matches could not be updated
- Template: add a `data-deadline-utc` attribute and reuse the countdown JS from task 0029 (or a simpler variant) to show "Closes in Xh Ym"; once expired, the JS can disable the input
- This task intentionally does not change the global lock — both mechanisms coexist: global lock closes everything, per-match deadline closes individual matches

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
