---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, scoring, top-scorer, players]
keywords: [top_scorer_points, goals_scored, scorers API, final top scorer, tournament complete]
patterns: [aggregate-by-max, end-of-tournament scoring, tie-aware lookup, player result sync, points derivation]
---

# FEATURE-SCORING-06: Score top scorer predictions

## Summary

Award top scorer points once the tournament ends and the final top scorer is confirmed.

## Acceptance Criteria

- [ ] `top_scorer_points(goals_scored: i32) -> i32` returns `5 + goals_scored`.
- [ ] The polling task queries football-data.org for the top scorer list.
- [ ] Top scorer scoring runs after the last match is finished.
- [ ] Matching `top_scorer_predictions` rows are updated with `points_awarded`.
- [ ] Only the final top scorer receives points, and all predictions for that player get the same value.
- [ ] Re-running top scorer scoring is idempotent.

## Implementation Context

### Relevant files

- `src/polling/scorer.rs` - top scorer point formula.
- `src/polling/db.rs` - scorer lookup and prediction updates.
- `src/football_api/mod.rs` - scorer API fetches.
- `src/modules/predictions/db.rs` - top scorer prediction records.

### Tests

- Unit test for the points formula.
- Integration test for final top scorer scoring.

### Implementation notes

- Final scoring should wait until the tournament is complete.
- Multiple users picking the same player should receive identical points.

## Research Context

### Keywords to Search

- `top_scorer_points` - scoring function.
- `goals_scored` - player stat used in the formula.
- `scorers API` - upstream data source.
- `final top scorer` - end-of-tournament trigger.

### Patterns to Investigate

- aggregate-by-max - choose the highest-scoring player.
- end-of-tournament scoring - deferred finalization.
- tie-aware lookup - handle equal goal totals safely.

### Key Decisions Made

- Top scorer points are bonus plus goals scored.
- Scoring is deferred until the tournament completes.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers the points formula and final scoring flow.
- [ ] `cargo clippy -- -D warnings` passes for top scorer logic.

### Manual Verification

- [ ] The final top scorer gets scored once the tournament ends.
- [ ] All users who picked that player receive the same score.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R6.
- Depends on result ingestion and background polling.

## Notes

- This ticket does not cover the UI for displaying top scorer predictions.
