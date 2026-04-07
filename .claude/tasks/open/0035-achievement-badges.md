---
id: 0035
title: Achievement badges for prediction milestones
status: open
type: feature
adrs: [0005, 0007, 0016]
refs: [0008, 0030]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Prediction games become more social and replayable when users earn visible recognition for notable performances. Achievement badges ("Perfect Round", "Underdog Caller", "Top of the League") give users bragging rights and surface interesting stories from the data.

## Acceptance Criteria

- [ ] At least 5 badge types are defined (see implementation notes for initial set)
- [ ] Badges are awarded by a background job or on-demand computation run after results are processed; they are stored in a `user_achievements` table
- [ ] Earned badges are displayed on the per-user stats page (task 0030) and optionally on the leaderboard
- [ ] A badge shows: icon/emoji, name, and a short description
- [ ] The same badge can only be awarded once per user per tournament

## Context for Claude 🤖

### Relevant files

- `migrations/` — add `user_achievements (id, user_id, tournament_id, badge_slug, awarded_at)` table; unique constraint on `(user_id, tournament_id, badge_slug)`
- `src/achievements.rs` (new non-route module) — badge definitions, award logic as pure functions, award-and-upsert DB function
- `src/polling/mod.rs` or `src/polling/scorer.rs` — call achievement award logic after scoring runs
- `src/modules/standings/db.rs` — add `get_user_achievements(pool, user_id, tournament_id)` query
- `src/modules/standings/handlers.rs` — load achievements in `member_stats` handler (task 0030)
- `templates/standings/member_stats.html` — display badge list

### ADR constraints

- **ADR-0016**: New table `user_achievements`; migration required
- **ADR-0007**: Achievement logic is not a route module — place in `src/achievements.rs` (single file, non-route)
- **ADR-0005**: Upsert on `(user_id, tournament_id, badge_slug)` unique constraint

### Tests

- Unit tests for each badge predicate function (pure functions): given mock data, does the user qualify?

### Implementation notes

**Initial badge set:**

| slug | name | trigger |
|------|------|---------|
| `perfect_group_round` | Perfect Round | All group stage predictions correct in a single match day |
| `underdog_caller` | Underdog Caller | Correctly predicted 3+ upsets (home team with >60% "not home win" consensus was wrong) |
| `top_scorer` | Top Scorer | Finished #1 on the leaderboard at the end of the tournament |
| `consistent_predictor` | Consistent Predictor | >70% group stage accuracy across all played matches |
| `oracle` | Oracle | Predicted the tournament winner correctly in the top-scorer/winner prediction |

- Badge award functions should take plain data (scores, predictions, consensus stats) not DB connections — keep them pure and testable
- Award job: can be called at the end of `src/polling/mod.rs` result-processing loop; check all members of all active leagues
- The `underdog_caller` badge requires consensus data (task 0031) — implement that first, or simplify to "correctly predicted 3+ matches where outcome was away win"
- The `perfect_group_round` badge requires grouping predictions by match day — ensure `matches.match_day` (or similar) exists in the schema; if not, group by date (DATE(scheduled_at))

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
