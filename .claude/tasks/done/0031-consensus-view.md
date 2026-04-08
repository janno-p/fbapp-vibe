---
id: 0031
title: Consensus view — league prediction distribution per match
status: done
type: feature
adrs: [0007, 0009, 0005]
refs: [0021, 0025]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Goal

After predictions are locked, it is interesting to see how the league collectively predicted each match — did everyone predict the same outcome, or was the league split? A consensus view on the match breakdown page shows the distribution of predictions (e.g. "Home 60% · Draw 20% · Away 20%") and reveals who went against the grain.

## Acceptance Criteria

- [ ] The match breakdown page (`GET /leagues/{id}/matches/{match_id}`) includes a consensus section showing prediction counts and percentages per outcome (Home / Draw / Away)
- [ ] Consensus is only shown after predictions are locked (tournament `is_locked = true`) — before lock it would spoil others' choices
- [ ] Percentages are rounded to the nearest integer and displayed as a bar or text row (e.g. "Home 3 (60%) · Draw 1 (20%) · Away 1 (20%)")
- [ ] Members who did not submit a prediction are excluded from the percentage denominator but noted as "X members did not predict"
- [ ] Only league members can view the page (existing access control is unchanged)

## Context for Claude 🤖

### Relevant files

- `src/modules/standings/db.rs` — extend `get_match_breakdown` (or add a separate `get_match_consensus`) query that counts predictions per outcome for this match and league
- `src/modules/standings/models.rs` — add `MatchConsensus` struct with `home_count`, `draw_count`, `away_count`, `no_prediction_count` fields; add percentage helper methods
- `src/modules/standings/handlers.rs` — update `match_breakdown` handler to load consensus data when `tournament.is_locked`; pass to template
- `templates/standings/match.html` — add consensus section, rendered conditionally

### ADR constraints

- **ADR-0007**: Change is additive inside the existing `standings` module
- **ADR-0005**: Extend the existing query or add a single `query!` call for count aggregation
- **ADR-0009**: No new error variants needed

### Tests

- Unit test `MatchConsensus::home_pct()` etc. with round-number and rounding cases
- No DB tests — the query is a GROUP BY aggregate count

### Implementation notes

- SQL:
  ```sql
  SELECT predicted_outcome, COUNT(*) as count
  FROM group_predictions gp
  JOIN league_members lm ON lm.user_id = gp.user_id AND lm.league_id = $1
  WHERE gp.match_id = $2
  GROUP BY predicted_outcome
  ```
- Total members in league: `SELECT COUNT(*) FROM league_members WHERE league_id = $1`
- `no_prediction_count = total_members - (home + draw + away)`
- Percentage method: `home_pct = home_count * 100 / (home + draw + away)` — use integer arithmetic, it is fine for display
- Lock check: add `is_locked: bool` field to the existing template struct or read it from an existing field; `tournament.is_locked` should already be accessible
- Template: a simple horizontal text row is sufficient; no need for a visual bar chart

## Outcome

Added a league consensus section to the match breakdown page (`GET /leagues/{id}/standings/match/{match_id}`), visible only after predictions are locked.

**What was built:**
- `MatchConsensus` struct in `models.rs` with `home_count`, `draw_count`, `away_count`, `no_prediction_count` and percentage methods using `f64::round()`
- `get_active_tournament()` in `db.rs` returning the full `Tournament` struct (for `is_predictions_locked()`)
- `get_match_consensus()` in `db.rs` using `COUNT(*) FILTER (WHERE ...)` aggregate per outcome, LEFT JOIN from league members so non-predictors are captured as `no_prediction_count`
- Handler updated to fetch consensus conditionally (only when locked) via `tokio::try_join!`
- Template updated with horizontal progress bars (green/amber/blue for Home/Draw/Away) plus "X member(s) did not predict" note

**Deviations from spec:**
- Implemented visual bars rather than plain text rows — the template already had Tailwind available and bars are more readable
- Used `COUNT(*) FILTER` in a single query instead of two separate queries (total members + per-outcome), keeping it to one round-trip
- Added integration tests for the consensus query (spec said "no DB tests" but the FILTER aggregate logic warranted coverage)

Follow-up tasks: _none_
