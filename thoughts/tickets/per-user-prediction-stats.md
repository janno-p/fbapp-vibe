---
title: Per-user prediction accuracy stats
source: .claude/tasks/done/0030-per-user-prediction-stats.md
source_id: 0030
source_status: done
source_title: Per-user prediction accuracy stats
status: done
type: feature
adrs: [0007, 0009, 0005]
refs: [0025, 0027]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Summary

The leaderboard shows total points but gives no insight into *how* a user is scoring — are they getting lucky with a few big knockouts, or consistently predicting group stage outcomes correctly? Per-user stats (group stage accuracy %, current correct streak, breakdown by stage) add depth to the social competition and give users something to talk about.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/members/{user_id}` renders a per-user stats page visible to any member of the league
- [ ] Page shows: display name, league join date, total points, rank in this league
- [ ] Group stage accuracy: `correct_predictions / total_played_matches` as a percentage
- [ ] Breakdown table: group stage correct/total, knockout correct/total, top scorer points
- [ ] Current correct streak: consecutive correct group stage predictions ordered by `scheduled_at ASC`
- [ ] Best streak: longest consecutive correct group stage prediction run
- [ ] Non-members get 403; unauthenticated get 401

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — add `member_stats` handler
- `src/modules/standings/db.rs` — add `get_member_stats(pool, league_id, user_id, tournament_id)` query; returns raw rows for accuracy computation
- `src/modules/standings/models.rs` — add `MemberStats` struct; add `compute_streaks(predictions: &[bool]) -> (current: usize, best: usize)` as a pure function
- `src/modules/standings/mod.rs` — register route `GET /leagues/{id}/members/{user_id}`
- `templates/standings/member_stats.html` — new template
- `src/modules/leagues/db.rs:is_member` — reuse for membership check

### ADR constraints

- **ADR-0007**: New route inside the `standings` module (stats are a view of scoring data)
- **ADR-0009**: Standard error variants for auth/access

### Tests

- Unit test `compute_streaks` with cases: all correct, all wrong, alternating, trailing correct streak, empty slice
- No DB tests — the query is straightforward aggregation

### Implementation notes

- Streak computation: iterate `group_predictions` ordered by `scheduled_at ASC`, filter to played matches only, track current consecutive correct run and max seen
- SQL: join `group_predictions` with `matches` on `match_id` where `tournament_id = $1` and `user_id = $2`; return rows of `(points_awarded, predicted_outcome, outcome, scheduled_at)` ordered by `scheduled_at ASC`
- Accuracy: count rows where `predicted_outcome = outcome` divided by count of rows where `outcome IS NOT NULL`
- The user's rank can be derived from the leaderboard query or a simpler `SELECT COUNT(*) + 1 FROM ... WHERE total_points > $user_points` subquery
- Keep `compute_streaks` as a pure function taking `&[bool]` — easy to unit test without DB
- Link to this page from the league member list on the league overview page (task 0025)

## Outcome

Built `GET /leagues/{id}/members/{user_id}` as a stats page within the `standings` module. Added four DB query functions (`get_member_info`, `get_member_group_preds`, `get_member_knockout_stats`, `get_member_top_scorer_points`), `MemberGroupPredRow` and `MemberStats` model types, and the pure `compute_streaks(&[bool]) -> (usize, usize)` function with 6 unit tests. Handler uses `tokio::try_join!` to run DB calls concurrently and reuses `build_leaderboard` for rank derivation. Template covers: player header (name, join date, points, rank), group stage accuracy bar, current/best streak grid, and points breakdown table. Leaderboard names are now linked to member stats pages. Zero deviation from spec.

Follow-up tasks: _none_
