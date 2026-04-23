---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, scoring, group-stage, predictions]
keywords: [group_stage_points, MatchOutcome, points_awarded, pure function, prediction scoring]
patterns: [pure domain function, scoring pipeline, idempotent write, outcome comparison, unit-tested business logic]
---

# FEATURE-SCORING-04: Score group stage predictions

## Summary

Score group stage predictions using a pure outcome comparison function and persist the awarded points.

## Acceptance Criteria

- [ ] `src/polling/scorer.rs` contains pure scoring functions with no DB access or side effects.
- [ ] `group_stage_points(predicted: MatchOutcome, actual: MatchOutcome) -> i32` returns `1` for a correct pick.
- [ ] `group_stage_points` returns `0` when the prediction is wrong.
- [ ] After a finished match is ingested, all matching `group_stage_predictions` rows are scored.
- [ ] `points_awarded` is updated for each prediction row.
- [ ] Re-scoring the same match yields the same result.
- [ ] The cycle logs how many predictions were scored at debug level.

## Implementation Context

### Relevant files

- `src/polling/scorer.rs` - pure scoring logic.
- `src/polling/db.rs` - fetch predictions and persist points.
- `src/modules/standings/db.rs` - leaderboard queries that consume scores.
- `src/db_types.rs` - `MatchOutcome` type used by the scorer.

### Tests

- Unit tests for correct and incorrect predictions.
- Unit test that repeated scoring remains stable.
- `#[sqlx::test]` for the full group-stage scoring path.

### Implementation notes

- Keep the scorer side-effect free so the logic is easy to verify.
- The database update should use the scorer result as the only source of truth.

## Research Context

### Keywords to Search

- `group_stage_points` - core scoring function.
- `MatchOutcome` - input enum.
- `points_awarded` - persisted score field.
- `pure function` - no side effects.

### Patterns to Investigate

- pure domain function - scoring logic isolated from I/O.
- scoring pipeline - read result, score predictions, write points.
- idempotent write - safe repeated processing.

### Key Decisions Made

- Group stage scoring is a simple correct/incorrect rule.
- Scoring logic must stay testable without database setup.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers all group-stage score cases.
- [ ] `cargo clippy -- -D warnings` passes for the scorer.

### Manual Verification

- [ ] A correct prediction gets 1 point.
- [ ] An incorrect prediction gets 0 points.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R4.
- Depends on result ingestion.

## Notes

- This ticket does not cover confidence multipliers; that is a separate requirement.
