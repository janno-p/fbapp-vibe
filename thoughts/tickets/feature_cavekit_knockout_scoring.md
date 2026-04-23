---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, scoring, knockout, predictions]
keywords: [knockout_points_per_team, KnockoutRound, advancing teams, winner, round scoring]
patterns: [round-based scoring, derived team advancement, pure scoring function, read-modify-write pipeline, idempotent scoring]
---

# FEATURE-SCORING-05: Score knockout predictions

## Summary

Score knockout predictions by round using the teams that advance from finished knockout matches.

## Acceptance Criteria

- [ ] `knockout_points_per_team(round: KnockoutRound) -> i32` returns the configured round values.
- [ ] The scorer treats a team as advancing when it appears in a finished knockout match.
- [ ] Finished knockout rounds are detected before scoring is attempted.
- [ ] `knockout_predictions.points_awarded` is updated for every matching prediction row.
- [ ] Re-running scoring for the same round is idempotent.
- [ ] The cycle logs how many knockout predictions were scored at debug level.

## Implementation Context

### Relevant files

- `src/polling/scorer.rs` - knockout scoring constants.
- `src/polling/db.rs` - round completion detection and updates.
- `src/modules/standings/db.rs` - leaderboard and round progress reads.
- `src/db_types.rs` - `KnockoutRound` type.

### Tests

- Unit tests for each round value.
- Integration test for scoring a finished round once.

### Implementation notes

- Keep team advancement detection aligned with the knockout match data model.
- The scoring path should only run when the round is complete.

## Research Context

### Keywords to Search

- `knockout_points_per_team` - scoring function.
- `KnockoutRound` - round enum.
- `advancing teams` - winner detection.
- `winner` - final round handling.

### Patterns to Investigate

- round-based scoring - score by tournament phase.
- derived team advancement - infer winners from finished matches.
- idempotent scoring - no double-awards.

### Key Decisions Made

- Knockout scoring is round-based and deterministic.
- Scoring waits until the relevant round is complete.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers all knockout round values.
- [ ] Round completion detection is exercised in integration tests.

### Manual Verification

- [ ] A finished round produces knockout points.
- [ ] Reprocessing the same round does not change the total.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R5.
- Depends on result ingestion and group-stage scoring.

## Notes

- This ticket excludes any tie-breaking or bonus-multiplier behavior.
