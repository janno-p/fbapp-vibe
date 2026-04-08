---
id: 0034
title: Confidence multiplier on group stage predictions
status: open
phase: Phase2
type: feature
adrs: [0005, 0007, 0009, 0016]
refs: [0007, 0008]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Standard group stage predictions award 1 point per correct outcome. A confidence multiplier lets each user "double down" on up to 3 matches they feel certain about — a correct doubled prediction earns 2 points instead of 1. This adds a strategic layer without requiring schema-heavy changes.

## Acceptance Criteria

- [ ] Each user may mark at most 3 group stage predictions as "confident" (2× multiplier) per tournament
- [ ] The prediction form shows a toggle/checkbox for "I'm confident" on each group match
- [ ] Scoring: a confident correct prediction awards 2 points; a confident wrong prediction awards 0 (same as unconfident wrong)
- [ ] The multiplier choice is stored per prediction and is locked with the match (cannot change after the per-match deadline or global lock)
- [ ] The leaderboard and match breakdown pages show the multiplier where relevant (e.g., "2× ✓ +2 pts" vs "✓ +1 pt")
- [ ] If a user tries to mark more than 3 predictions as confident, the form rejects the submission with a clear error

## Context for Claude 🤖

### Relevant files

- `migrations/` — add migration to add `is_confident BOOLEAN NOT NULL DEFAULT FALSE` column to `group_predictions`
- `src/modules/predictions/models.rs` — add `is_confident: bool` field to the group stage prediction form model
- `src/modules/predictions/db.rs` — update upsert to write `is_confident`; add validation query that counts existing confident predictions before inserting
- `src/modules/predictions/handlers.rs` — validate max 3 confident picks per tournament before upserting
- `src/polling/scorer.rs` — update group stage scoring to multiply by 2 when `is_confident = true`
- `src/modules/standings/models.rs:MatchBreakdownRow` — add `is_confident: bool` field
- `templates/predictions/index.html` — add confident checkbox per match
- `templates/standings/match.html` — display multiplier in breakdown rows

### ADR constraints

- **ADR-0016**: Schema change requires a migration; `is_confident` is a column on `group_predictions`
- **ADR-0005**: Update the `INSERT ... ON CONFLICT DO UPDATE` to include `is_confident`
- **ADR-0009**: Return `AppError::BadRequest` when >3 confident picks are submitted in one request

### Tests

- Unit test in `scorer.rs`: correct confident prediction → 2 pts; incorrect confident prediction → 0 pts; correct unconfident → 1 pt
- Unit test: validation function `count_remaining_confident_slots(existing_count, new_count)` returns error when exceeding 3

### Implementation notes

- Migration: `ALTER TABLE group_predictions ADD COLUMN is_confident BOOLEAN NOT NULL DEFAULT FALSE;`
- Validation: count `SELECT COUNT(*) FROM group_predictions WHERE user_id = $1 AND tournament_id = $2 AND is_confident = true`; allow up to 3 total; on resubmission, count the user's existing confident predictions excluding the current match being updated
- The 3-pick limit is a constant: `const MAX_CONFIDENT_PICKS: i64 = 3;`
- Form: HTML checkbox with name like `confident_<match_id>` — parse in handler alongside existing outcome fields
- This is a moderately complex task; tackle in order: migration → scorer → form → handler validation → template display

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
