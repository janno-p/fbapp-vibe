---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-15T00:00:00Z
---

# Cavekit: Tournament Management

## Scope

Lifecycle of a tournament: registration from football-data.org competitions list, seeding (teams, groups, matches, players), activation/deactivation, and prediction locking (manual or automatic).

## Requirements

### R1: Tournament Registration
**Description:** Admin users can register a new tournament from football-data.org competition.

**Acceptance Criteria:**
- [ ] Admin visits `/admin/competitions` and sees list of available competitions from football-data.org API
- [ ] Admin submits competition selection with: external_id (from API), name (custom label), season (year)
- [ ] POST `/admin/tournaments` creates tournament record in database
- [ ] Newly created tournament has: id, external_id, name, season, is_active=false, predictions_locked_at=null
- [ ] Created tournament is seeded immediately (see R2)
- [ ] Admin is redirected to `/admin` dashboard after successful registration

**Dependencies:** cavekit-auth (AdminUser extractor)

### R2: Tournament Seeding
**Description:** Teams, groups, matches, players, and group memberships are fetched from football-data.org API and stored locally.

**Acceptance Criteria:**
- [ ] Seeding fetches all teams for the competition from football-data.org
- [ ] Seeding fetches all matches for the competition from football-data.org
- [ ] Teams are stored in `teams` table: id, tournament_id, external_id, name, code (3-letter), tla, flag (ISO 2-letter country code, nullable)
- [ ] Groups are created in `groups` table: id, tournament_id, name (e.g., "Group A")
- [ ] Group memberships are created in `group_memberships` table: group_id, team_id
- [ ] Matches are stored in `matches` table: id, tournament_id, home_team_id, away_team_id, group_id, stage, scheduled_utc, home_score, away_score, outcome
- [ ] Players are stored in `players` table: id, team_id, name, position, number, goals_scored (default 0)
- [ ] Seeding is idempotent: running seed twice does not duplicate records (upsert behavior)
- [ ] Seeding is rate-limited to 7 requests per second (free tier football-data.org limit)

**Dependencies:** R1 (Tournament Registration)

### R3: Tournament Activation
**Description:** Exactly one tournament is active at a time; admins can activate/deactivate tournaments.

**Acceptance Criteria:**
- [ ] POST `/admin/tournaments/{id}/activate` sets tournament.is_active = true
- [ ] Activating a tournament deactivates any currently active tournament (zero or one active at a time)
- [ ] POST `/admin/tournaments/{id}/deactivate` sets tournament.is_active = false
- [ ] Only admin users can activate/deactivate (enforced via AdminUser extractor)
- [ ] Activation/deactivation is logged at info level
- [ ] Users can see the active tournament on prediction form pages

**Dependencies:** R1 (Tournament Registration), cavekit-auth (AdminUser)

### R4: Prediction Locking (Manual)
**Description:** Admin can manually lock/unlock predictions before and after tournament.

**Acceptance Criteria:**
- [ ] POST `/admin/tournaments/{id}/lock` sets tournament.predictions_locked_at = now()
- [ ] POST `/admin/tournaments/{id}/unlock` sets tournament.predictions_locked_at = null
- [ ] When predictions are locked, prediction forms are read-only (no new predictions or edits allowed)
- [ ] Locked prediction forms still show user's current predictions
- [ ] Only admin users can manually lock/unlock (enforced via AdminUser extractor)
- [ ] Lock/unlock is logged at info level

**Dependencies:** R3 (Tournament Activation), cavekit-predictions (lock enforcement)

### R5: Auto-Lock on First Kickoff
**Description:** Predictions are automatically locked when the first match of the tournament kicks off (becomes in-progress).

**Acceptance Criteria:**
- [ ] Background polling task detects when any tournament match status changes to in-progress (or finished)
- [ ] On first in-progress match detected, polling task sets tournament.predictions_locked_at = match.scheduled_utc
- [ ] Auto-lock happens before or at match kickoff time
- [ ] Auto-lock is only set once per tournament (idempotent)
- [ ] Manual lock (R4) takes precedence; auto-lock does not override manual lock
- [ ] Auto-lock is logged at info level

**Dependencies:** R3 (Tournament Activation), cavekit-scoring (polling task touches this)

### R6: Tournament Data Models
**Description:** Tournament and related domain types are available throughout the app.

**Acceptance Criteria:**
- [ ] Tournament struct: id, external_id, name, season, is_active, predictions_locked_at
- [ ] `is_predictions_locked()` method returns true if predictions_locked_at is set and <= now()
- [ ] Match struct: id, tournament_id, home_team_id, away_team_id, group_id, stage, scheduled_utc, home_score (optional), away_score (optional), outcome (optional)
- [ ] Team struct: id, tournament_id, external_id, name, code, tla, flag (Option<String>)
- [ ] Group struct: id, tournament_id, name
- [ ] Player struct: id, team_id, name, position, number, goals_scored
- [ ] All domain types can be queried and serialized for templates

**Dependencies:** R1 (Tournament Registration)

### R7: Team National Flag Display
**Description:** Teams are displayed with their national flag using self-hosted Iconify circle-flags icons keyed on the team's ISO 2-letter country code, not external crest images.

**Acceptance Criteria:**
- [ ] `teams` table stores a `flag` column (ISO 2-letter code, e.g., `"es"` for Spain); no `crest_url` column
- [ ] `src/national_flags.rs` provides `tla_to_flag(tla: Option<&str>) -> Option<String>` mapping TLA → ISO-2 code
- [ ] Admin seeding populates `flag` via `tla_to_flag()` at seed time; unmapped TLAs store `NULL`
- [ ] All templates referencing a team use `icon-[circle-flags--{flag_code}]` Tailwind class for flag display
- [ ] A fallback (no icon rendered) is used when `flag` is NULL
- [ ] Iconify circle-flags is self-hosted via `@iconify/tailwind4` plugin (no CDN dependency)
- [ ] All active-tournament country flags are listed as `@source inline(...)` directives in `assets/css/input.css`

**Dependencies:** R2 (Tournament Seeding), R6 (Team data model)

## Out of Scope

- Editing tournament details after creation (no update API)
- Deleting tournaments
- Multiple simultaneous active tournaments (exactly one at a time)
- Partial seeding or manual team/match/player entry
- Custom tournament rules or group/knockout formats
- Import from sources other than football-data.org
- Scheduling or rescheduling matches
- Player transfers or roster changes mid-tournament
- Tournament cancellation or postponement workflows

## Source Traceability

### Brownfield Status: Complete
All acceptance criteria are satisfied by existing code.

### Source Files
- `src/modules/admin/mod.rs` — router() with tournament routes, AdminUser extractor
- `src/modules/admin/handlers.rs` — list_competitions, register_tournament, seed_tournament, activate/deactivate, lock/unlock handlers
- `src/modules/admin/db.rs` — tournament CRUD, seeding queries, activation logic
- `src/modules/admin/models.rs` — Tournament, RegisterTournamentForm types
- `src/football_api.rs` — football-data.org HTTP client with rate limiting
- `src/polling/mod.rs` — background polling task, auto-lock detection
- `src/national_flags.rs` — TLA-to-ISO-2 country code mapping for Iconify circle-flags (R7)
- `migrations/0005_tournament_core.sql` — tournaments, teams, groups, group_memberships, matches, players tables
- `migrations/0010_add_r32_knockout_round.sql` — R32 round variant for knockout
- `migrations/0015_remove_team_crest_url.sql` — drops `crest_url` column from teams (R7)
- `migrations/0016_team_flag.sql` — adds `flag TEXT` column to teams (R7)

### Implementation Notes
- Uses `sqlx::query_as!()` and `sqlx::query!()` macros for compile-time query checking
- football-data.org API client has 7s rate limiter via `RateLimiter` (free tier constraint)
- Seeding is implemented as a synchronous operation post-registration (blocking but fast)
- Auto-lock is handled in the polling task; fires when first match transitions to in-progress
- Tournament state is queried on every prediction page load to set form read-only flag

## Changes
- 2026-04-15: Added R7 (Team National Flag Display) — crest_url replaced with self-hosted Iconify circle-flags; added flag column, tla_to_flag() mapping, migrations 0015/0016; updated R2/R6 schema criteria

## Cross-References
- Depends on: **cavekit-auth.md** (AdminUser extractor)
- Consumed by: **cavekit-predictions.md** (tournament lock state, active tournament selection)
- Consumed by: **cavekit-scoring.md** (tournament data for result ingestion and auto-lock)
- Consumed by: **cavekit-standings.md** (active tournament for leaderboard calculation; flag icons used in match breakdown and fixtures templates)
