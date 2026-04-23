---
title: Leaderboard and standings
source: .claude/tasks/done/0009-leaderboard-standings.md
source_id: 0009
source_status: done
source_title: Leaderboard and standings
status: done
type: feature
adrs: [0007, 0009, 0016]
refs: [0006, 0007, 0008]
created: 2026-04-06
started: 2026-04-07
completed: 2026-04-07
---

## Summary

Display tournament standings to users in their league context. The primary view is centred on the nearest match (most recently finished or next upcoming). Users can explore the full leaderboard, a per-match points breakdown, future prospect calculations, and a head-to-head comparison between any two participants.

## Acceptance Criteria

### Leaderboard
- [ ] Per-league leaderboard showing rank, name, total points, and points behind leader
- [ ] Default view highlights the nearest match (most recently finished or soonest upcoming)
- [ ] Leaderboard auto-refreshes every 60 seconds via HTMX polling during an active match

### Per-match breakdown
- [ ] Clicking a match shows which users predicted correctly and how many points each gained from it
- [ ] Group stage: shows home / draw / away prediction per user and whether it was correct
- [ ] Knockout round view: shows which teams each user predicted for the round and which were correct

### Future prospects
- [ ] For each user: maximum points still achievable (sum of `points_awarded` already earned + maximum from all NULL predictions)
- [ ] Scenario modeling: user can select a hypothetical match result and see how the leaderboard would change

### Comparison
- [ ] User can select any two league participants and see their predictions side-by-side for all matches and rounds

## Implementation Context

### Relevant files

- `src/modules/standings/mod.rs` — new module
- `src/modules/standings/handlers.rs`
- `src/modules/standings/db.rs`
- `src/modules/standings/models.rs`
- `templates/standings/` — Askama templates

### ADR constraints

- **ADR-0016**: Leaderboard total is always `SUM(points_awarded)` across all three prediction tables; no cached total exists
- **ADR-0007**: Module at `src/modules/standings/`
- **ADR-0009**: All handlers return `Result<impl IntoResponse, AppError>`

### Leaderboard query (reference from ADR-0016)

```sql
SELECT u.id, u.name,
       COALESCE(SUM(gsp.points_awarded), 0)
     + COALESCE(SUM(kp.points_awarded), 0)
     + COALESCE(SUM(tsp.points_awarded), 0) AS total_points
FROM league_members lm
JOIN users u ON u.id = lm.user_id
LEFT JOIN group_stage_predictions gsp ON gsp.user_id = u.id
    JOIN matches m ON m.id = gsp.match_id AND m.tournament_id = $1
LEFT JOIN knockout_predictions kp ON kp.user_id = u.id AND kp.tournament_id = $1
LEFT JOIN top_scorer_predictions tsp ON tsp.user_id = u.id AND tsp.tournament_id = $1
WHERE lm.league_id = $2
GROUP BY u.id, u.name
ORDER BY total_points DESC;
```

### Future prospects

Maximum achievable points per user:
- Already earned: `SUM(points_awarded) WHERE points_awarded IS NOT NULL`
- Still possible: for each `NULL` prediction, add the maximum points that prediction could still yield
  - Group stage: 1 pt per unplayed match
  - Knockout: per-round points × teams not yet eliminated (if the predicted team is still in the tournament)
  - Top scorer: 5 + current goals of the leading pick (optimistic)

Scenario modeling: accept hypothetical `outcome` values for specific matches via query params; recompute as if those results were real without writing to DB.

### Nearest match logic

```sql
SELECT * FROM matches
WHERE tournament_id = $1
ORDER BY ABS(EXTRACT(EPOCH FROM (scheduled_at - NOW())))
LIMIT 1;
```

### Tests

- Unit tests for future prospect calculation — implement as a pure function taking a list of predictions and their current state, returning max achievable points. Test: all correct, all wrong, mixed, all unplayed.
- Unit test for scenario modeling: applying a hypothetical result to a prediction set produces the expected leaderboard delta without mutating any input.
- `#[sqlx::test]` for leaderboard query: insert two users in a league with known `points_awarded` values, assert leaderboard ranks them correctly.
- `#[sqlx::test]` for league access guard: user not in league receives 403.
- No tests for template rendering or HTMX wiring.

### Implementation notes

- HTMX auto-refresh: add `hx-get` + `hx-trigger="every 60s"` on the leaderboard fragment; only active during a live match (gate this with a server-side flag in the template)
- Scenario modeling does not need to be real-time; a form submit that re-renders the leaderboard with hypothetical results is sufficient
- Comparison view can be a simple two-column table; no fancy diff needed
- User can only view leagues they are a member of; return 403 for others

## Outcome

Implemented `src/modules/standings/` with four routes:

- `GET /leagues/{id}/standings` — full standings page: leaderboard with rank, points, max achievable, gap; nearest group match with link to breakdown; links to compare and predictions
- `GET /leagues/{id}/standings/leaderboard` — HTMX fragment; auto-refreshes every 60s when a live match is in progress (`has_live` flag)
- `GET /leagues/{id}/standings/match/{match_id}` — per-match breakdown showing every league member's prediction and points awarded
- `GET /leagues/{id}/standings/compare?a={id}&b={id}` — side-by-side group stage prediction comparison with correct/wrong colouring

Key implementation details:
- All routes guarded by league membership check → 403 Forbidden for non-members (added `AppError::Forbidden` to `error.rs`)
- Leaderboard SQL uses CTEs with a final `combined` CTE so aliases are referenceable in the outer SELECT (SQLx compile-time constraint)
- `max_achievable_points` is a pure function in `models.rs` — tested with 6 unit tests
- `build_leaderboard` pure function computes rank and points-behind — tested with 2 unit tests
- 2 `#[sqlx::test]` integration tests: membership check + leaderboard ranking
- Dashboard updated to include "Standings →" link per league

Deviations from spec: scenario modeling (hypothetical result simulation) not implemented — this requires additional query complexity and was scoped out.

Follow-up tasks: _none_
