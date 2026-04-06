---
id: 0009
title: Leaderboard and standings
status: open
type: feature
adrs: [0007, 0009, 0016]
refs: [0006, 0007, 0008]
created: 2026-04-06
started: ~
completed: ~
---

## Goal

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

## Context for Claude 🤖

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

### Implementation notes

- HTMX auto-refresh: add `hx-get` + `hx-trigger="every 60s"` on the leaderboard fragment; only active during a live match (gate this with a server-side flag in the template)
- Scenario modeling does not need to be real-time; a form submit that re-renders the leaderboard with hypothetical results is sufficient
- Comparison view can be a simple two-column table; no fancy diff needed
- User can only view leagues they are a member of; return 403 for others

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
