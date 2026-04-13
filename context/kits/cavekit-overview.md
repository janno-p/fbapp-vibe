---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit Overview

## Purpose

This document indexes all domain cavekits for fbapp-vibe, a server-rendered Rust/Axum football tournament prediction game. Together, these kits form the single source of truth for all downstream implementation and testing work.

## All Kits

| Kit | Purpose | Status |
|-----|---------|--------|
| **cavekit-auth.md** | User authentication via Google OAuth, session management, admin role | Brownfield (complete) |
| **cavekit-tournament.md** | Tournament registration, seeding, activation, prediction locking | Brownfield (complete) |
| **cavekit-leagues.md** | League creation, membership, invite token management | Brownfield (complete) |
| **cavekit-predictions.md** | Prediction forms (group, knockout, top scorer), lock enforcement, review page | Brownfield (2 gaps) |
| **cavekit-scoring.md** | Result polling, auto-lock, prediction scoring, player goal tracking | Brownfield (1 gap) |
| **cavekit-standings.md** | Leaderboards, match breakdown, fixtures, member stats, per-round breakdown | Brownfield (3 gaps) |
| **cavekit-badges.md** | Achievement badges (new domain) | Greenfield (1 task) |
| **cavekit-observability.md** | OTLP/Jaeger trace export infrastructure (new domain) | Greenfield (1 task) |

**Legend:**
- **Brownfield (complete)** — all acceptance criteria satisfied by existing code
- **Brownfield (N gaps)** — N acceptance criteria not yet satisfied; flagged as [GAP]
- **Greenfield** — entirely new domain; all requirements mapped to open tasks

## Total Coverage

| Metric | Count |
|--------|-------|
| Total requirements (R\*) | 62 |
| Total acceptance criteria | 147 |
| Open implementation gaps | 8 |
| Greenfield requirements | 13 |

## Dependency Graph

```
auth (no deps)
  ├── tournament
  │   ├── leagues
  │   └── predictions
  │       ├── scoring
  │       └── standings
  │           ├── badges (new achievement system)
  │           └── observability (infrastructure)
  └── leagues
      └── standings
```

**Read order:** Start with auth, then tournament and leagues in parallel, then predictions, then scoring, then standings (which touches both badges and observability as optional enhancements).

## Cross-Kit Interfaces

### User Model (auth → all)
Every kit references the user model from auth:
- `id: i64` (primary key)
- `email: String`
- `is_admin: bool`

### Tournament State (tournament → predictions, scoring, standings)
Predictions and scoring use tournament state:
- `id: i64`
- `is_active: bool`
- `predictions_locked_at: Option<DateTime<Utc>>`

### Prediction Data (predictions ← scoring, standings)
Scoring computes points on stored predictions (group_stage_predictions, knockout_predictions, top_scorer_predictions).
Standings displays both predictions and results.

### League Membership (leagues → standings)
Standings filters entries by league_id and league_members.user_id to isolate per-league leaderboards.

## Gap Locations

**Predictions gaps (cavekit-predictions.md):**
- Task 0047: Prediction completion counter on group stage tab
- Task 0048: Show actual match results after kickoff on predictions page

**Scoring gaps (cavekit-scoring.md):**
- Task 0034: Confidence multiplier (2× points for confident picks; max 3 per tournament)

**Standings gaps (cavekit-standings.md):**
- Task 0032: Per-round leaderboard breakdown (points by stage)
- Task 0040: Group stage standings table computation
- Task 0018: Scenario modeling (hypothetical results on standings)

## New Domains

**Badges (cavekit-badges.md) — Task 0035:**
- Implement achievement system with 5+ badge types
- Award badges post-match via background job
- Display earned badges on member stats page

**Observability (cavekit-observability.md) — Task 0010:**
- Add optional OTLP/Jaeger trace export
- Non-breaking change (works without env var set)
- Includes docker-compose.yml for local testing

## Implementation Notes

1. **No circular dependencies** — all kits depend on auth; some depend on tournament/leagues/predictions, but nothing depends back on higher levels.

2. **Testability** — all acceptance criteria are either:
   - Verifiable by automated test (unit, integration, E2E)
   - Observable in running app (UI state, log output, database state)

3. **YAGNI** — no "nice to have" features added; only what was explicitly requested in the task backlog.

4. **Brownfield traceability** — each kit references source files that contributed to its requirements.
