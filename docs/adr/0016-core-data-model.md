# ADR-0016: Core Data Model 🗄️

## Status

✅ Accepted

## Date

2026-04-06

## Context

The application is a football tournament prediction game with the following structural requirements:

- 🏆 **One active tournament** at a time (UEFA Euro or FIFA World Cup), configured by an admin
- 👥 **Leagues** — independent groups of users who compete against each other; users join via invite link
- 🔮 **Predictions** — made once per user, globally (not per league); visible and scored within every league the user belongs to
- 🔒 **Prediction lock** — all predictions become read-only when the tournament starts
- ⚽ **Match results** — pulled from an external API; stored locally for scoring and display
- 📊 **Scoring** — computed from predictions vs. results; must support per-league leaderboards and future-prospect calculations

### Tournament structure

A tournament has two distinct phases with different prediction types:

**Group stage**
- Fixed number of groups (e.g. 6 groups of 4 teams = 24 teams for Euro)
- Each group plays a round-robin; every match has a predicted outcome: home win / draw / away win
- 1 point per correct prediction

**Knockout stage**
- Rounds: R32 → R16 → Quarter-finals → Semi-finals → Final → Winner
- Users predict which teams advance to each round (not match-by-match opponents)
- Points increase per round: 2 / 3 / 4 / 6 / 8 / 10 (R32 counts 2 pts per team)

**Top scorer**
- User selects 3 players as candidates
- If any of the 3 is the tournament's actual top scorer: 5 points + goals scored by that player

### Key modelling challenges

1. **Predictions are global but leaderboards are per-league** — a user's score is the same number regardless of league; the leaderboard just ranks different subsets of users.
2. **Knockout predictions are round-keyed, not match-keyed** — a user predicts "team X reaches the QF", not "team X beats team Y in match Z".
3. **Top scorer has a composite reward** — the 5-point bonus plus variable goals-scored points must both be recorded for auditability.
4. **Scores must be recomputable** — results arrive incrementally; scores are recalculated as results are confirmed.

## Decision

### Entities and relationships

```
users (existing)
  │
  ├─◄─ league_members ─►─ leagues
  │
  └─── predictions (one set per user per tournament)
         ├── group_stage_predictions (one per match)
         ├── knockout_predictions    (one per round per team slot)
         └── top_scorer_predictions  (three rows per user)

tournaments
  ├── teams
  ├── groups
  │     └── group_memberships (team ↔ group)
  ├── matches
  │     ├── group stage matches  (group_id set, round = NULL)
  │     └── knockout matches     (group_id = NULL, round set)
  └── players
```

### Schema

```sql
-- ── Tournaments ──────────────────────────────────────────────

CREATE TABLE tournaments (
    id              BIGSERIAL PRIMARY KEY,
    external_id     TEXT NOT NULL UNIQUE,   -- football-data.org competition id
    name            TEXT NOT NULL,
    season          TEXT NOT NULL,          -- e.g. '2024'
    is_active       BOOLEAN NOT NULL DEFAULT FALSE,
    predictions_locked_at TIMESTAMPTZ,      -- set when tournament kicks off
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Only one active tournament enforced at application layer.

-- ── Teams ────────────────────────────────────────────────────

CREATE TABLE teams (
    id              BIGSERIAL PRIMARY KEY,
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    external_id     TEXT NOT NULL,
    name            TEXT NOT NULL,
    short_name      TEXT NOT NULL,
    crest_url       TEXT,
    UNIQUE (tournament_id, external_id)
);

-- ── Groups ───────────────────────────────────────────────────

CREATE TABLE groups (
    id              BIGSERIAL PRIMARY KEY,
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    name            TEXT NOT NULL,          -- 'A', 'B', …
    UNIQUE (tournament_id, name)
);

CREATE TABLE group_memberships (
    group_id        BIGINT NOT NULL REFERENCES groups(id),
    team_id         BIGINT NOT NULL REFERENCES teams(id),
    PRIMARY KEY (group_id, team_id)
);

-- ── Players ──────────────────────────────────────────────────

CREATE TABLE players (
    id              BIGSERIAL PRIMARY KEY,
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    external_id     TEXT NOT NULL,
    name            TEXT NOT NULL,
    team_id         BIGINT NOT NULL REFERENCES teams(id),
    goals_scored    INT NOT NULL DEFAULT 0,
    UNIQUE (tournament_id, external_id)
);

-- ── Matches ──────────────────────────────────────────────────

CREATE TYPE match_outcome AS ENUM ('home', 'draw', 'away');
CREATE TYPE knockout_round AS ENUM ('r32', 'r16', 'qf', 'sf', 'final', 'winner');

CREATE TABLE matches (
    id              BIGSERIAL PRIMARY KEY,
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    external_id     TEXT NOT NULL,
    group_id        BIGINT REFERENCES groups(id),   -- NULL for knockout
    round           knockout_round,                  -- NULL for group stage
    home_team_id    BIGINT REFERENCES teams(id),        -- NULL for knockout matches before draw
    away_team_id    BIGINT REFERENCES teams(id),        -- NULL for knockout matches before draw
    scheduled_at    TIMESTAMPTZ NOT NULL,
    home_score      INT,                             -- NULL until played
    away_score      INT,
    outcome         match_outcome,                   -- NULL until played
    UNIQUE (tournament_id, external_id),
    CHECK (
        (group_id IS NOT NULL AND round IS NULL) OR
        (group_id IS NULL AND round IS NOT NULL)
    )
);

-- ── Leagues ──────────────────────────────────────────────────

CREATE TABLE leagues (
    id              BIGSERIAL PRIMARY KEY,
    name            TEXT NOT NULL,
    invite_token    TEXT NOT NULL UNIQUE,
    created_by      BIGINT NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE league_members (
    league_id       BIGINT NOT NULL REFERENCES leagues(id),
    user_id         BIGINT NOT NULL REFERENCES users(id),
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (league_id, user_id)
);

-- ── Predictions ──────────────────────────────────────────────

-- Group stage: one row per user per match
CREATE TABLE group_stage_predictions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    match_id        BIGINT NOT NULL REFERENCES matches(id),
    predicted_outcome match_outcome NOT NULL,
    points_awarded  INT,                        -- NULL until match played
    UNIQUE (user_id, match_id)
);

-- Knockout: one row per user per round per team slot
-- A user predicts N teams reaching a given round (e.g. 16 teams for r16).
-- Each row = "I think this team reaches this round."
CREATE TABLE knockout_predictions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    round           knockout_round NOT NULL,
    team_id         BIGINT NOT NULL REFERENCES teams(id),
    points_awarded  INT,                        -- NULL until round complete
    UNIQUE (user_id, tournament_id, round, team_id)
);

-- Top scorer: exactly 3 rows per user per tournament
CREATE TABLE top_scorer_predictions (
    id              BIGSERIAL PRIMARY KEY,
    user_id         BIGINT NOT NULL REFERENCES users(id),
    tournament_id   BIGINT NOT NULL REFERENCES tournaments(id),
    player_id       BIGINT NOT NULL REFERENCES players(id),
    points_awarded  INT,                        -- NULL until tournament ends
    UNIQUE (user_id, tournament_id, player_id)
);
```

### Scoring storage

`points_awarded` is stored denormalised on each prediction row. This enables:
- Fast leaderboard queries (sum points without re-evaluating rules)
- Per-match breakdowns (filter by match_id or round)
- Future-prospect calculation (count NULL rows = unresolved predictions)

Scores are recalculated by a background job when a match result arrives. The job:
1. Updates `matches.outcome` (and scores)
2. Updates `players.goals_scored`
3. Sets `points_awarded` on affected prediction rows
4. Does **not** store a cached total — leaderboard totals are always `SUM(points_awarded)` at query time

### Leagues and scoring

There is no `user_tournament_score` table. A user's score within a league is:

```sql
SELECT u.id, u.name,
       COALESCE(SUM(gsp.points_awarded), 0)
     + COALESCE(SUM(kp.points_awarded), 0)
     + COALESCE(SUM(tsp.points_awarded), 0) AS total_points
FROM league_members lm
JOIN users u ON u.id = lm.user_id
LEFT JOIN group_stage_predictions gsp ON gsp.user_id = u.id
    JOIN matches m ON m.id = gsp.match_id AND m.tournament_id = :tid
LEFT JOIN knockout_predictions kp ON kp.user_id = u.id AND kp.tournament_id = :tid
LEFT JOIN top_scorer_predictions tsp ON tsp.user_id = u.id AND tsp.tournament_id = :tid
WHERE lm.league_id = :lid
GROUP BY u.id, u.name
ORDER BY total_points DESC;
```

## Concurrency Control

### 1. Prediction upserts — no special handling needed

Each prediction write is an `INSERT ... ON CONFLICT (...) DO UPDATE`. Postgres executes this atomically; duplicate or concurrent submissions from the same user for the same match/round/player are safe by construction.

### 2. Prediction lock — `SELECT ... FOR UPDATE` on tournament row

The check "is the tournament still open for predictions?" is a TOCTOU race:

```
Request A: reads predictions_locked_at → NULL (open)
Admin:     sets predictions_locked_at = NOW()
Request A: inserts prediction          → should be rejected, isn't
```

Fix: every prediction write must acquire a row-level lock on the tournament before inserting:

```sql
BEGIN;
SELECT id FROM tournaments
WHERE id = $1
  AND (predictions_locked_at IS NULL OR predictions_locked_at > NOW())
FOR UPDATE;
-- zero rows → ROLLBACK and return 403
INSERT INTO group_stage_predictions ...  -- or knockout / top_scorer
COMMIT;
```

The `FOR UPDATE` lock serialises concurrent writes against the same tournament row. Because the lock is held only for the duration of the transaction (milliseconds), contention is negligible.

### 3. Score recalculation — idempotent job + match-level advisory lock

When a match result arrives from the API the background job must:
1. Update `matches.outcome`, `home_score`, `away_score`
2. Set `points_awarded` on all affected prediction rows

If the job crashes and restarts, or if two instances run simultaneously, the same match could be scored twice. Fix: make the job idempotent and guard it with a Postgres advisory lock keyed on the match ID.

```sql
-- Acquire advisory lock (non-blocking; skip if another job holds it)
SELECT pg_try_advisory_xact_lock($match_id);
-- returns FALSE → another job is processing this match; skip

BEGIN;
UPDATE matches SET outcome = $outcome, home_score = $hs, away_score = $as
WHERE id = $match_id AND outcome IS NULL;  -- no-op if already scored

-- Only update predictions if the match row actually changed
UPDATE group_stage_predictions
SET points_awarded = CASE WHEN predicted_outcome = $outcome THEN 1 ELSE 0 END
WHERE match_id = $match_id AND points_awarded IS NULL;
COMMIT;
```

The `AND outcome IS NULL` / `AND points_awarded IS NULL` guards make a second execution a no-op even if the advisory lock is not held (e.g. after a crash).

### 4. Tournament activation — enforced at DB level

The partial unique index `CREATE UNIQUE INDEX one_active_tournament ON tournaments (is_active) WHERE is_active = TRUE` guarantees at most one active tournament without any application-layer coordination.

### Summary

| Scenario | Mechanism |
|---|---|
| Duplicate prediction submit | `ON CONFLICT DO UPDATE` (atomic upsert) |
| Prediction lock race | `SELECT ... FOR UPDATE` on tournament row |
| Duplicate score calculation | `pg_try_advisory_xact_lock` + idempotent `WHERE outcome IS NULL` |
| Duplicate active tournament | Partial unique index |

## Rationale

- 🔑 **Predictions are tournament-scoped, not league-scoped** — matches the stated requirement that predictions are made once and count everywhere.
- 📐 **Knockout predictions are round + team, not match + outcome** — users predict advancement, not head-to-head results; this matches the UX spec.
- 💾 **Denormalised `points_awarded`** — avoids recalculating scoring rules on every leaderboard page load; keeps scoring logic in one place (the background job).
- 🔗 **`invite_token` on leagues** — a random token enables shareable invite links without exposing internal IDs.
- 🏷️ **`external_id` on tournaments/teams/players/matches** — preserves the API's identity for idempotent upserts during polling.

## Trade-offs and Risks ⚠️

- ⚠️ **No cached score totals** — leaderboard queries join across three prediction tables. Acceptable at this scale (tens of users); add a materialised view or cache column if it becomes slow.
- ⚠️ **Knockout prediction completeness not enforced at DB level** — the constraint "exactly N teams per round" is enforced at the application layer. DB only prevents duplicates.
- ⚠️ **Top scorer limit of 3 not enforced at DB level** — application layer must reject a 4th insert.
- ⚠️ **Single active tournament** — enforced by the partial unique index `one_active_tournament`; activating a second tournament while one is active will raise a DB constraint error that must be surfaced clearly in the admin UI.

## Consequences

- 📋 Migrations will add all tables above, versioned from `0004_` onward.
- 🔒 Prediction lock is checked by reading `tournaments.predictions_locked_at IS NOT NULL AND predictions_locked_at <= NOW()` before any write.
- 🧮 The background polling job is responsible for result ingestion and `points_awarded` updates.
- 🏆 Leaderboard, per-match breakdown, and future-prospect views are all derived from `SUM(points_awarded)` and `COUNT(*) WHERE points_awarded IS NULL`.
- 👤 Admin grant uses the `is_admin BOOLEAN` column on `users` (migration `0008`).
