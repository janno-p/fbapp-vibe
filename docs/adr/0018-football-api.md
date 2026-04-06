# ADR-0018: football-data.org as External Data Source ⚽

## Status

✅ Accepted

## Date

2026-04-06

## Context

The application needs an external source for tournament data: competition metadata, team rosters with player lists, match fixtures, live match results, and top scorer statistics. This data drives tournament seeding (task 0005) and live result polling (task 0008).

### Options evaluated

| Option | Coverage | Auth | Rate limit | Notes |
|---|---|---|---|---|
| **football-data.org** | EC, WC, major leagues | API key | 10 req/min (free) | Well-documented v4 API, stable IDs, existing familiarity |
| OpenLigaDB | German Bundesliga focus | None | None stated | Inadequate coverage for EC/WC |
| UEFA/FIFA official feeds | First-party | Partnership required | N/A | Not accessible without commercial agreement |

### football-data.org free tier

- Covers `EC` (UEFA European Championship) and `WC` (FIFA World Cup) ✅
- 10 requests/minute rate limit
- Endpoints used: `/competitions`, `/competitions/{code}/teams` (includes squad), `/competitions/{code}/matches`, `/competitions/{code}/scorers`
- Stable integer IDs on all resources — suitable for `external_id` columns

## Decision

Use **football-data.org v4 API** as the sole external data source.

### Rate limiting

10 req/min = one request per 6 seconds minimum. Enforced in the client via a sequential mutex tracking the last request timestamp and sleeping if the interval has not elapsed. This serialises all API calls through a single timing gate — appropriate for a single-instance server.

Minimum interval is set to **7 seconds** (one second buffer above the 6-second minimum).

### Data mapping

| API field | DB column |
|---|---|
| `team.id` | `teams.external_id` |
| `match.id` | `matches.external_id` |
| `player.id` | `players.external_id` |
| `competition.id` | `tournaments.external_id` |
| `match.score.winner = "HOME_TEAM"` | `matches.outcome = 'home'` |
| `match.score.winner = "AWAY_TEAM"` | `matches.outcome = 'away'` |
| `match.score.winner = "DRAW"` | `matches.outcome = 'draw'` |
| `match.stage = "GROUP_STAGE"` | `matches.group_id` set, `round` NULL |
| `match.stage = "ROUND_OF_16"` | `matches.round = 'r16'` |
| `match.stage = "QUARTER_FINALS"` | `matches.round = 'qf'` |
| `match.stage = "SEMI_FINALS"` | `matches.round = 'sf'` |
| `match.stage = "FINAL"` | `matches.round = 'final'` |

Group memberships are derived from match data: a team belongs to a group if it appears in a `GROUP_STAGE` match with that `group` value. No separate standings endpoint is needed.

Players are seeded from the `squad` array on the teams response. Player IDs from the scorers endpoint are the same identifiers.

## Trade-offs and Risks ⚠️

- ⚠️ **External dependency**: availability and schema are outside our control. The `external_id` design isolates the DB from API ID changes — only the client layer needs updating if the API changes.
- ⚠️ **Free tier limits**: 10 req/min is sufficient for seeding (4–5 calls) and polling (1 call per cycle). If the polling interval is reduced below 1 minute, the rate limit will be exceeded.
- ⚠️ **Squad completeness**: player squads on the teams endpoint may be incomplete before the official tournament squad announcement. Re-seeding (idempotent upsert) can be run again closer to the tournament to pick up late additions.
- ⚠️ **No retry logic**: the client does not retry on failure. The polling job's next cycle will retry naturally. Explicit retry can be added in a follow-up if needed.

## Consequences

- `FOOTBALL_API_KEY` is a required environment variable in all environments.
- `FootballApiClient` is constructed once in `main.rs` and stored in `AppState`.
- All API calls go through a shared `reqwest::Client` with a 30-second timeout.
- Callers (seeder, polling job) receive `anyhow::Result` from the client; they are responsible for mapping errors to `AppError`.
