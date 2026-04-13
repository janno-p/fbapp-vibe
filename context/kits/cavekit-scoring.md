---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit: Scoring & Result Polling

## Scope

Background polling task that fetches match results from football-data.org, detects prediction lock triggers, scores predictions, and updates player goal counts.

## Requirements

### R1: Background Polling Loop
**Description:** A periodic background task polls for match results and processes tournament data.

**Acceptance Criteria:**
- [ ] Background task runs continuously (spawned in main.rs via tokio::spawn)
- [ ] Task loops with configurable interval: 30 seconds when tournament is active and has live matches, 120 seconds otherwise
- [ ] Task logs polling cycle at debug level
- [ ] If a polling cycle fails (API error, DB error), task logs error at warn level and continues (does not crash)
- [ ] Task is gracefully cancelable (respects tokio shutdown signals)

**Dependencies:** None (infrastructure)

### R2: Result Ingestion
**Description:** The polling task fetches finished matches from football-data.org and stores their results locally.

**Acceptance Criteria:**
- [ ] Task calls football-data.org API for active tournament's matches
- [ ] For each match with status "FINISHED", task updates local match record:
  - [ ] Stores home_score and away_score from API response
  - [ ] Computes outcome (home/draw/away) from scores
  - [ ] Stores outcome in match.outcome column
- [ ] Updates are idempotent: fetching same result twice does not duplicate or overwrite with stale data
- [ ] Match record is unchanged if status is not "FINISHED" (only finished matches are updated)
- [ ] Task logs number of matches updated at info level per cycle

**Dependencies:** R1 (Background Polling Loop), cavekit-tournament (active tournament)

### R3: Auto-Lock on First Kickoff
**Description:** When the first match of the active tournament kicks off, predictions are automatically locked.

**Acceptance Criteria:**
- [ ] Task detects when any match transitions to "IN_PLAY" or "FINISHED" status
- [ ] On detection of first in-play match, task sets tournament.predictions_locked_at = match.scheduled_utc
- [ ] Auto-lock is set exactly once per tournament (idempotent)
- [ ] Manual lock (admin action) takes precedence: if tournament.predictions_locked_at is already set, auto-lock does not override it
- [ ] Auto-lock happens at or before the kickoff time (uses match.scheduled_utc, not current time)
- [ ] Task logs auto-lock event at info level

**Dependencies:** R1 (Polling Loop), cavekit-tournament (tournament lock state)

### R4: Group Stage Scoring
**Description:** Predictions are scored based on match outcomes using pure scoring functions.

**Acceptance Criteria:**
- [ ] Scoring functions are pure (no DB calls, no side effects), located in `src/polling/scorer.rs`
- [ ] Function `group_stage_points(predicted: MatchOutcome, actual: MatchOutcome) -> i32`:
  - [ ] Returns 1 if predicted == actual (correct prediction)
  - [ ] Returns 0 if predicted != actual (incorrect prediction)
- [ ] After each match result is ingested, task scores all predictions for that match:
  - [ ] Queries all `group_stage_predictions` rows for that match
  - [ ] Calls `group_stage_points(predicted, actual)` for each
  - [ ] Updates `group_stage_predictions.points_awarded` with result
- [ ] Scoring is idempotent: scoring same match twice yields same result
- [ ] Task logs number of predictions scored at debug level

**Dependencies:** R2 (Result Ingestion)

### R5: Knockout Scoring
**Description:** Knockout predictions are scored based on teams advancing to each round.

**Acceptance Criteria:**
- [ ] Function `knockout_points_per_team(round: KnockoutRound) -> i32`:
  - [ ] R32: 2 points per correct team
  - [ ] R16: 3 points per correct team
  - [ ] QF: 4 points per correct team
  - [ ] SF: 6 points per correct team
  - [ ] Final: 8 points per correct team
  - [ ] Winner: 10 points
- [ ] A team "advances" to a round if it has any match with group_id = NULL (knockout stage)
- [ ] After knockout matches are ingested, task scores knockout predictions:
  - [ ] For each round with finished matches, determines which teams advanced
  - [ ] Queries all `knockout_predictions` rows for that round
  - [ ] Awards points if predicted team appears in finished match results for that round
  - [ ] Updates `knockout_predictions.points_awarded`
- [ ] Scoring is idempotent
- [ ] Task logs number of knockout predictions scored at debug level

**Dependencies:** R2 (Result Ingestion), R4 (Group Stage Scoring as prerequisite)

### R6: Top Scorer Scoring
**Description:** Top scorer predictions are awarded points when the predicted player is confirmed as tournament's top scorer.

**Acceptance Criteria:**
- [ ] Function `top_scorer_points(goals_scored: i32) -> i32`:
  - [ ] Returns 5 (bonus) + goals_scored (e.g., if top scorer has 6 goals, awards 11 points)
- [ ] Task queries football-data.org to identify current top scorer (highest goals_scored across all players)
- [ ] After tournament completes (last match finished), task identifies final top scorer
- [ ] For each `top_scorer_predictions` row matching final top scorer, task:
  - [ ] Queries the player's final goals_scored count
  - [ ] Calls `top_scorer_points(goals_scored)`
  - [ ] Updates `top_scorer_predictions.points_awarded`
- [ ] Only one player per tournament can be top scorer (all predictions for same player get same points)
- [ ] Scoring is idempotent

**Dependencies:** R2 (Result Ingestion), R1 (Polling Loop)

### R7: Player Goal Tracking
**Description:** Player goals are synced from football-data.org to detect top scorer changes.

**Acceptance Criteria:**
- [ ] Task calls football-data.org to fetch top scorers list (if available from API)
- [ ] For each player, updates `players.goals_scored` from API response
- [ ] Updates are idempotent: fetching same top scorers list twice does not lose data
- [ ] Task logs number of players updated at debug level
- [ ] Goals are updated during every polling cycle (not just end-of-tournament)

**Dependencies:** R1 (Polling Loop), cavekit-tournament (active tournament teams/players)

### R8: Scoring Models
**Description:** Domain types for scoring and result handling.

**Acceptance Criteria:**
- [ ] MatchOutcome enum: Home, Draw, Away (mirrors DB enum)
- [ ] KnockoutRound enum: R32, R16, QF, SF, Final, Winner
- [ ] Can convert from/to string slugs ("home", "draw", "away", "r32", "r16", etc.)
- [ ] All types are serializable and testable

**Dependencies:** cavekit-tournament (tournament data models)

## Out of Scope

- Manual result entry (only automatic polling from API)
- Result rollback or correction (no undo mechanism)
- Partial tournament scoring (all or nothing per match)
- Bonus point multipliers or tie-breaking rules
- Handicaps or custom scoring per user/league
- Suspended or postponed match handling (treated as not finished)
- Penalties or disqualifications
- Weather-based scoring adjustments
- Live scoring updates to users (no WebSocket or polling client-side)

## Gaps (Open Tasks)

### [GAP] R9: Confidence Multiplier
**Task:** 0034 — Confidence multiplier

Users can mark up to 3 group stage predictions as "confident" to earn 2× points.

**Acceptance Criteria:**
- [ ] `group_stage_predictions` table adds `is_confident BOOLEAN NOT NULL DEFAULT FALSE` column
- [ ] Prediction form shows toggle/checkbox for "I'm confident in this prediction" per match
- [ ] User can mark up to 3 predictions per tournament as confident
- [ ] Submitting >3 confident predictions returns 400 Bad Request with message "Maximum 3 confident picks allowed"
- [ ] Scoring function updated: if is_confident = true and correct, award 2 points instead of 1; if incorrect, award 0 (same as non-confident)
- [ ] Confident multiplier is locked with the match (cannot toggle after lock)
- [ ] Leaderboard and match breakdown pages show indicator (e.g., "2× ✓ +2 pts" for confident correct)
- [ ] Multiplier counts toward tournament's total; user cannot exceed 3 per tournament
- [ ] const MAX_CONFIDENT_PICKS: i64 = 3 defined in code

**Implementation note:** Count confident picks when form is submitted; validate before insert. Scoring updates: `group_stage_points(predicted, actual, is_confident) -> i32 { if predicted == actual { if is_confident { 2 } else { 1 } } else { 0 } }`.

**Dependencies:** R4 (Group Stage Scoring), cavekit-predictions (form updates)

## Source Traceability

### Brownfield Status: Mostly Complete (1 gap)
R1-R8 are fully implemented. R9 (confidence multiplier) is open task 0034.

### Source Files
- `src/polling/mod.rs` — background polling loop, main task orchestration
- `src/polling/db.rs` — queries for result ingestion, auto-lock, scoring updates
- `src/polling/scorer.rs` — pure scoring functions with unit tests
- `migrations/0005_tournament_core.sql` — matches table with home_score, away_score, outcome columns
- `src/db_types.rs` — MatchOutcome, KnockoutRound enums

### Implementation Notes
- Polling task spawned in `main.rs` via `tokio::spawn(polling::run_polling_loop(state))`
- Interval logic uses `tokio::time::sleep()` with conditional duration
- Rate limiting on football-data.org API calls enforced in `football_api.rs` (7s limit)
- Scoring is deferred: results ingested in one cycle, scoring may happen in same or next cycle
- Auto-lock uses match.scheduled_utc (API kick-off time), not current time, to avoid timezone issues
- Pure functions in `scorer.rs` are heavily unit tested (see existing test suite)

## Cross-References
- Depends on: **cavekit-tournament.md** (active tournament, lock state, match data)
- Depends on: **cavekit-predictions.md** (prediction data to be scored)
- Consumed by: **cavekit-standings.md** (leaderboard uses points_awarded from scoring)
- Related to: **cavekit-badges.md** (achievement system may use scoring data)
