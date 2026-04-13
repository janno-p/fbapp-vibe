---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
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

## Source Traceability

### Brownfield Status: Mostly Complete (3 gaps)
R1-R6 are fully implemented. R7, R8, R9 are open tasks.

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

### Implementation Notes
- Leaderboard uses a single multi-table query to fetch all user predictions and sum points
- Tie-breaking is done in post-processing (Rust, after query) not in SQL
- Match breakdown queries all predictions for a match grouped by user
- HTMX fragment is lightweight and repeatable for live updates
- Standings pages check league membership before rendering (401/403)
- Member stats are aggregated in-memory from raw prediction data

## Cross-References
- Depends on: **cavekit-auth.md** (user context, AuthSession)
- Depends on: **cavekit-leagues.md** (league membership access control)
- Depends on: **cavekit-predictions.md** (prediction data, review page)
- Depends on: **cavekit-scoring.md** (points_awarded, match results)
- Depends on: **cavekit-tournament.md** (active tournament, match data)
- Related to: **cavekit-badges.md** (achievements displayed on member stats)
