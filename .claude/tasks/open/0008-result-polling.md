---
id: 0008
title: Background result polling and scoring
status: open
type: feature
adrs: [0016]
refs: [0004, 0005]
created: 2026-04-06
started: ~
completed: ~
---

## Goal

Implement a background task that polls the football API for match results, updates the local database, and recalculates `points_awarded` for all affected predictions. This is the engine that drives live leaderboard updates during the tournament.

## Acceptance Criteria

- [ ] Background task starts with the server and polls on a configurable interval (default: 2 minutes)
- [ ] Polls only when there is an active tournament
- [ ] Polls more frequently (configurable, default: 30 seconds) when a match is currently in progress (scheduled_at within last 2 hours and outcome still NULL)
- [ ] On each poll: fetches all matches from the API, upserts results for completed matches
- [ ] After updating a match result, recalculates `points_awarded` for all `group_stage_predictions` on that match
- [ ] After a knockout round is fully decided (all matches in round have outcomes), recalculates `points_awarded` for `knockout_predictions` for that round
- [ ] After tournament ends, calculates `points_awarded` for `top_scorer_predictions`
- [ ] Concurrent execution is safe: uses `pg_try_advisory_xact_lock` keyed on match ID (per ADR-0016)
- [ ] Scoring is idempotent: re-running on an already-scored match is a no-op
- [ ] Errors from the API (network failure, rate limit) are logged and retried on next poll cycle; they do not crash the server

## Context for Claude 🤖

### Relevant files

- `src/polling/mod.rs` — new module; not a route module, does not expose `router()`
- `src/polling/scorer.rs` — scoring logic (points calculation per prediction type)
- `src/main.rs` — spawn polling task with `tokio::spawn` after server setup
- `src/state.rs` — `AppState` is passed to the polling task (contains `PgPool` and football API client)

### Scoring rules (from ADR-0016)

| Prediction type | Condition | Points |
|---|---|---|
| Group stage | `predicted_outcome = match.outcome` | 1 |
| R16 knockout | team appeared in R16 | 2 per team |
| QF knockout | team appeared in QF | 3 per team |
| SF knockout | team appeared in SF | 4 per team |
| Final knockout | team appeared in Final | 5 per team |
| Winner | team won the tournament | 6 |
| Top scorer | any of 3 picks is top scorer | 5 + goals scored |

### ADR constraints

- **ADR-0016 (concurrency)**: Use `pg_try_advisory_xact_lock($match_id)` before scoring a match; use `AND outcome IS NULL` guard to make scoring idempotent
- **ADR-0010**: Use `tracing::info!` / `tracing::warn!` / `tracing::error!` for all polling events

### Knockout round completion detection

A knockout round is "complete" when all matches for that round have `outcome IS NOT NULL`. Query `matches` grouped by `round` to detect this before running knockout scoring.

For `winner` round: the winner is the team with `outcome = 'home'` or `outcome = 'away'` in the Final match.

### Top scorer detection

Query `players` ordered by `goals_scored DESC LIMIT 1` for the tournament. If tied, all tied players count as top scorer (any user who picked any of the tied players gets the reward).

### Tests

This task has the highest unit test value in the codebase — scoring rules are pure logic with no I/O.

- Unit tests for every scoring rule in `src/polling/scorer.rs` — implement scoring as pure functions that take plain data types (not DB rows):
  - Correct / incorrect group stage prediction → 1 pt / 0 pt
  - Each knockout round correct / incorrect → correct point value per round
  - Winner correct → 6 pt
  - Top scorer: pick matches → 5 + goals; pick doesn't match → 0; tie between players handled correctly
- Unit test for idempotency: calling the scorer on an already-scored prediction returns the same value (not double-counted)
- Unit test for tied top scorer: two players with equal goals, user picked one of them → award granted
- `#[sqlx::test]` for the full scoring pipeline: insert a match result, run the scorer, assert `points_awarded` rows are updated correctly

### Implementation notes

- Use `tokio::time::interval` for the poll loop; adjust interval dynamically based on match schedule
- The polling task receives a clone of `AppState`; `PgPool` is cheaply cloneable (it's an Arc internally)
- Do not implement server-sent events or WebSocket push in this task — the leaderboard page will refresh via HTMX polling (task 0009)
- Rate limit: football-data.org free tier allows 10 req/min; a single poll fetches one `/matches` endpoint call; 2-minute default interval is safely within limits

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
