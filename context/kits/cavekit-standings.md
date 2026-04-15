---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-15T00:00:00Z
---

# Cavekit: Standings & Leaderboards

## Scope

Leaderboards, match breakdowns (showing all members' predictions for a match), member comparison, fixtures pages, member stats, and per-round leaderboard breakdowns.

## Requirements

### R1: Main Leaderboard
**Description:** League members can view a ranked leaderboard showing each member's total points and rank.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/standings` renders main leaderboard page (league members only, 401/403 enforced)
- [ ] Leaderboard shows table with columns: Rank, Name, Points, (optional: streak, recent form)
- [ ] Entries are sorted by points DESC with tie-breaking as follows:
  - [ ] Primary sort: total points DESC
  - [ ] Tie-breaker 1: correct predictions count DESC
  - [ ] Tie-breaker 2: user ID ASC (stable, deterministic)
- [ ] Points are summed across all predictions for the active tournament
- [ ] Rank is assigned based on sort order (1st, 2nd, etc.)
- [ ] Page shows league name and member count
- [ ] Page displays "No tournament active" if no active tournament
- [ ] Page displays "Predictions locked" indicator and lock time if applicable

**Dependencies:** cavekit-leagues (league membership check), cavekit-scoring (points_awarded), cavekit-auth (user context)

### R2: HTMX Leaderboard Fragment
**Description:** Leaderboard can be updated live via HTMX without full page reload.

**Acceptance Criteria:**
- [ ] Leaderboard table is rendered as a fragment template
- [ ] GET `/standings/leaderboard?league_id={id}` returns HTML fragment (no page wrapper)
- [ ] Fragment includes all columns from R1 (Rank, Name, Points, etc.)
- [ ] Fragment is updated by HTMX `hx-get` requests from client (e.g., every 10 seconds or on user action)
- [ ] Fragment respects league membership (401/403 if user not in league)

**Dependencies:** R1 (Main Leaderboard)

### R3: Match Breakdown
**Description:** Members can see a detailed view of all members' predictions for a single match.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/standings/match/{match_id}` renders match breakdown page (league members only)
- [ ] Page shows: league name, match info (teams, kickoff time, actual result if finished)
- [ ] Table shows each league member's prediction and points awarded (or pending if not scored)
- [ ] Columns: Member Name, Prediction (home/draw/away or score), Points Awarded, Correct/Wrong indicator
- [ ] Rows are sorted by member name or points DESC
- [ ] Actual result is shown (if match is finished): "Final: Home 2-1 Away"
- [ ] Shows consensus (e.g., "18 predicted home, 4 predicted draw, 2 predicted away")
- [ ] Page is blank or 404 if match_id does not exist
- [ ] Access control: only league members can view (401/403 for non-members)

**Dependencies:** R1, cavekit-leagues (league membership), cavekit-scoring (match results), cavekit-predictions (predictions)

### R4: Member Comparison
**Description:** Two league members can be compared side-by-side, showing their predictions and results.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/standings/compare` renders compare page with member selector
- [ ] Dropdown/selector shows all league members
- [ ] User selects two members to compare
- [ ] POST or GET query params select members: `?member_a={id}&member_b={id}`
- [ ] Comparison table shows all matches with both members' predictions side-by-side
- [ ] Columns: Match, Actual Result, Member A Prediction, Member A Points, Member B Prediction, Member B Points
- [ ] Color coding: green for correct, red for incorrect predictions
- [ ] Knockout section: team predictions per round, same format
- [ ] Top scorer section: list of picks, highlighting the actual top scorer
- [ ] Access control: only league members can view (401/403)
- [ ] Invalid member IDs return 404 or clear error

**Dependencies:** R1, cavekit-leagues, cavekit-scoring

### R5: Fixtures Page
**Description:** Members can view upcoming and past matches grouped by stage and date.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/fixtures` renders fixtures page (league members only)
- [ ] Page groups matches by stage: Group Stage, Knockout (R32, R16, QF, SF, Final)
- [ ] Within stage, matches are sorted by kickoff time ASC
- [ ] Each match displays: home team, away team, scheduled kickoff, actual result (if finished), next closest match highlighted
- [ ] Match cards link to match breakdown page (see R3)
- [ ] Page shows date headers for readability (e.g., "Friday, 6 December 2024")
- [ ] Access control: league members only (401/403)

**Dependencies:** cavekit-tournament (match data), cavekit-leagues

### R6: Member Stats Page
**Description:** Individual member statistics are displayed for inspection by league members.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/members/{user_id}` renders member stats page (league members only)
- [ ] Page shows: member name, avatar, tournament name, total points, rank in league
- [ ] Stats section: group stage accuracy (%, correct out of total), knockout accuracy, top scorer picks
- [ ] Streak section: current win streak (consecutive correct predictions)
- [ ] Recent form: last 5 predictions with results and points
- [ ] Profile section: joined date, achievements (if implemented, see cavekit-badges)
- [ ] Access control: league members only (401/403)
- [ ] Page 404 if user_id does not exist or is not in league

**Dependencies:** R1, cavekit-leagues, cavekit-scoring, cavekit-badges (optional)

## Gaps (Open Tasks)

### [GAP] R7: Per-Round Leaderboard Breakdown
**Task:** 0032 — Per-round leaderboard

Shows points breakdown by stage (group, R16, QF, etc.).

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/standings/rounds` renders per-round breakdown page (league members only)
- [ ] Table shows each member in rows, columns for: group stage, R16, QF, SF, Final, Winner bonus, top scorer, total
- [ ] Each cell shows points awarded for that round (or "—" if not yet scored)
- [ ] Rows sorted by total points DESC (same tie-breaker as R1)
- [ ] Only stages with predictions are shown (hide empty columns)
- [ ] Access control: league members only (401/403)
- [ ] Page linked from main leaderboard page (tab or link to "By Round")

**Implementation note:** Query all predictions grouped by (user, round), sum points per round, then aggregate total. Template renders 2D table.

**Dependencies:** R1 (Main Leaderboard), cavekit-scoring (points per prediction)

### [GAP] R8: Group Stage Standings Table
**Task:** 0040 — Group stage standings

Shows FIFA World Cup-style group standings (MP, W, D, L, GF, GA, GD, Pts).

**Acceptance Criteria:**
- [ ] New non-route module `src/group_standings.rs` contains pure computation (no DB, no async)
- [ ] Struct `GroupMatchResult { group_id, home_team_id, away_team_id, home_score, away_score, outcome }`
- [ ] Struct `TeamStanding { team_id, team_name, mp, w, d, l, gf, ga, gd, pts }`
- [ ] Struct `GroupStandings { group_id, standings: Vec<TeamStanding> }`
- [ ] Function `compute_standings(matches: Vec<GroupMatchResult>) -> GroupStandings`:
  - [ ] Calculates: Matches Played, Wins, Draws, Losses, Goals For, Goals Against, Goal Difference, Points (3/1/0)
  - [ ] Pending matches (no outcome) are excluded from stats
  - [ ] Sorts teams by: Pts DESC, GD DESC, GF DESC, H2H pts (if available), H2H GD, H2H GF, alphabetical by team name
- [ ] Head-to-head (H2H) tiebreaker implemented when available (direct matches between tied teams)
- [ ] GET `/leagues/{id}/groups` renders group standings page
- [ ] Page shows standings for each group (A, B, C, D)
- [ ] Access control: league members only (401/403)
- [ ] Unit tests cover: simple group (4 teams), partial group, GD tiebreaker, GF tiebreaker, H2H tiebreaker, alphabetical fallback
- [ ] Tests are in `src/group_standings.rs` and are pure (no DB, no fixtures)

**Implementation note:** `compute_standings()` takes pre-fetched match data and returns computed standings. No DB calls. Handler queries finished group matches, calls `compute_standings()`, renders template.

**Dependencies:** cavekit-tournament (group, team, match data), cavekit-leagues

### [GAP] R9: Scenario Modeling
**Task:** 0018 — Hypothetical results

Users can set hypothetical outcomes for unplayed matches and see projected leaderboard.

**Acceptance Criteria:**
- [ ] Unplayed group stage matches show a hypothetical result picker on standings page
- [ ] User selects hypothetical outcome (home/draw/away) for each match
- [ ] Selecting a result triggers HTMX `hx-get` to re-render leaderboard
- [ ] Leaderboard recalculates points with hypothetical results applied on top of actual
- [ ] Multiple unplayed matches can be hypothesized simultaneously (state via query params)
- [ ] Query params format: `?hypo[{match_id}]=home|draw|away`
- [ ] Hypothetical results do NOT write to database (ephemeral, client-side state via URL)
- [ ] Clearing all hypothetical params returns to actual standings
- [ ] Leaderboard clearly distinguishes actual vs hypothetical points: "(+N projected)" suffix or similar
- [ ] Unplayed matches only; finished matches show actual result and cannot be hypothesized
- [ ] Works for league members only (401/403)

**Implementation note:** Handler accepts query params, parses hypothetical results, applies them in-memory to scoring calculation (not DB), renders leaderboard fragment. Pure function: `apply_hypothetical(predictions, actual_results, hypothetical_results) -> new_leaderboard`.

**Dependencies:** R1 (Main Leaderboard), R2 (HTMX Fragment), cavekit-scoring

### [GAP] R10: Scenario Modeling — Hypo Param Validation
**Description:** Server-side validation ensures hypothetical match IDs are valid (unplayed group-stage matches belonging to the active tournament) before computing projected standings.

**Acceptance Criteria:**
- [ ] Handler rejects hypo param keys that are not valid integers (400 Bad Request)
- [ ] Handler silently ignores hypo match IDs that do not appear in the unplayed_matches whitelist (only IDs returned by `get_unplayed_group_matches` for the active tournament are accepted)
- [ ] Handler enforces a maximum of 20 hypo params per request; excess params beyond the first 20 are dropped
- [ ] Handler silently ignores hypo param values that are not one of: `home`, `draw`, `away` (the match entry is simply dropped, no error returned)
- [ ] Knockout match IDs cannot be hypothesized (they are not in the unplayed group-stage whitelist and are filtered out)
- [ ] Unit tests cover: valid subset accepted, invalid ID filtered out, knockout ID rejected, value=`home`/`draw`/`away` accepted, invalid value ignored, >20 params truncated

**Dependencies:** R9 (Scenario Modeling), R8 (Group Stage Standings — unplayed_matches source)

### [GAP] R11: Potential Points Indicator
**Description:** A 7-tier visual indicator on the leaderboard shows each player's relative ceiling (max_achievable) compared to all other players, using self-hosted Material Symbols icons with color coding.

**Acceptance Criteria:**
- [ ] Each player's `remaining_possible = max_achievable - total_points` is computed and displayed as a secondary value inside the Max cell (e.g., "+42 pts left"), but it does NOT drive band assignment
- [ ] Band assignment is based solely on each player's `max_achievable` absolute value (their ceiling score if all remaining predictions are correct)
- [ ] Dynamic range is derived from `min(max_achievable)` and `max(max_achievable)` across all players in the current render
- [ ] Range is divided into 7 equal bands; each player is assigned a band based on their `max_achievable` value
- [ ] Edge case: if all players have the same `max_achievable` (range = 0), all players are assigned band 4 (middle)
- [ ] Icon assignment:
  - [ ] Band 7 (highest ceiling): triple chevron-up icon with strong green color
  - [ ] Band 6: double chevron-up icon with green color
  - [ ] Band 5: single chevron-up icon with muted green color
  - [ ] Band 4 (middle): horizontal/equal icon with gray color
  - [ ] Band 3: single chevron-down icon with muted red color
  - [ ] Band 2: double chevron-down icon with red color
  - [ ] Band 1 (lowest ceiling): triple chevron-down icon with strong red color
- [ ] Icons use Material Symbols (self-hosted variable font, not CDN)
- [ ] Indicator is displayed inside the existing Max cell in leaderboard table, stacked below the `max_achievable` number
- [ ] No additional column is added to the leaderboard
- [ ] Indicator applies to both the main leaderboard page (R1) and HTMX leaderboard fragment (R2)
- [ ] On main page load, indicator displays correctly
- [ ] On HTMX fragment update (e.g., polling), indicator recomputes and renders correctly

**Dependencies:** R1 (Main Leaderboard), R2 (HTMX Leaderboard Fragment), cavekit-scoring (max_achievable, total_points)

## Source Traceability

### Brownfield Status: Mostly Complete (1 open gap)
R1-R10 are fully implemented. R11 (Potential Points Indicator) is the only open task.

### Source Files
- `src/modules/standings/mod.rs` — router() with standings routes
- `src/modules/standings/handlers.rs` — leaderboard, match breakdown, compare, fixtures, member_stats handlers
- `src/modules/standings/db.rs` — queries for leaderboard, match data, member stats
- `src/modules/standings/models.rs` — LeaderboardEntry, MatchBreakdownRow, MemberStats types, build_leaderboard() function
- `templates/standings/index.html` — main leaderboard page
- `templates/standings/leaderboard.html` — HTMX fragment
- `templates/standings/match.html` — match breakdown page
- `templates/standings/compare.html` — member comparison page
- `templates/standings/member_stats.html` — individual member stats
- `src/national_flags.rs` — TLA-to-ISO-2 mapping used in match breakdown and fixtures templates (via cavekit-tournament.md R7)

### Implementation Notes
- Leaderboard uses a single multi-table query to fetch all user predictions and sum points
- Tie-breaking is done in post-processing (Rust, after query) not in SQL
- Match breakdown queries all predictions for a match grouped by user
- HTMX fragment is lightweight and repeatable for live updates
- Standings pages check league membership before rendering (401/403)
- Member stats are aggregated in-memory from raw prediction data

## Changes
- 2026-04-15: Added R11 (Potential Points Indicator) — 7-tier visual ceiling indicator for leaderboard
- 2026-04-15: Clarified R11 banding metric: band assignment uses `max_achievable` (absolute ceiling); `remaining_possible` is a display value only
- 2026-04-14: Added R10 (Scenario Modeling — Hypo Param Validation) — discovered during inspection (finding F-010, F-002, F-004, F-005)

## Cross-References
- Depends on: **cavekit-auth.md** (user context, AuthSession)
- Depends on: **cavekit-leagues.md** (league membership access control)
- Depends on: **cavekit-predictions.md** (prediction data, review page)
- Depends on: **cavekit-scoring.md** (points_awarded, match results)
- Depends on: **cavekit-tournament.md** (active tournament, match data, team flags R7)
- Related to: **cavekit-badges.md** (achievements displayed on member stats)
