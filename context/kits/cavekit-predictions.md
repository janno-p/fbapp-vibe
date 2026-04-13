---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit: Predictions

## Scope

Prediction forms for group stage matches, knockout rounds, and top scorers. Lock enforcement, visibility controls, and per-league review pages showing predictions vs actual results.

## Requirements

### R1: Group Stage Prediction Form
**Description:** Users can predict match outcomes (home/draw/away) for all group stage matches.

**Acceptance Criteria:**
- [ ] GET `/predictions` renders prediction page with group stage tab
- [ ] Group stage tab shows all matches grouped by group (A, B, C, D, etc.)
- [ ] Each match displays: home team, away team, scheduled kickoff time
- [ ] Form shows radio buttons or dropdown per match for outcome selection: home, draw, away
- [ ] Submit button saves all group stage predictions via POST `/predictions/group`
- [ ] Predictions are stored in `group_stage_predictions` table: user_id, match_id, predicted_outcome, points_awarded (null until scored)
- [ ] Submitting duplicate prediction for same match updates the existing prediction
- [ ] Form is read-only (disabled) when predictions are locked (tournament.predictions_locked_at is set)
- [ ] User can update predictions before lock; after lock, form is read-only

**Dependencies:** cavekit-auth (AuthSession), cavekit-tournament (tournament.is_predictions_locked())

### R2: Knockout Prediction Form
**Description:** Users can predict which teams advance through knockout rounds.

**Acceptance Criteria:**
- [ ] GET `/predictions` renders knockout tab with sections for each round (R32, R16, QF, SF, Final, Winner)
- [ ] Each round shows available teams (based on seeding or previous predictions if applicable)
- [ ] User selects one team per round (or optionally per-slot if tournament format requires)
- [ ] Knockout predictions are stored in `knockout_predictions` table: user_id, tournament_id, round, team_id, points_awarded
- [ ] POST `/predictions/knockout/{round}` saves predictions for a specific round
- [ ] Form shows user's current prediction for each round (pre-fill from DB)
- [ ] Form is read-only when predictions are locked
- [ ] Rounds with no seeded/available teams are hidden or marked as unavailable
- [ ] User can update predictions before lock; after lock, form is read-only

**Dependencies:** R1, cavekit-tournament (active tournament, knockout rounds)

### R3: Top Scorer Prediction Form
**Description:** Users can predict which player will finish as tournament's top scorer (max 3 picks).

**Acceptance Criteria:**
- [ ] GET `/predictions` renders top scorer tab
- [ ] Tab shows searchable list of all players with: name, team, current goals
- [ ] User can select up to 3 players (toggle or multi-select)
- [ ] Selected players are highlighted or checkmarked
- [ ] Submit button saves up to 3 predictions via POST `/predictions/top-scorer`
- [ ] Predictions stored in `top_scorer_predictions` table: user_id, tournament_id, player_id
- [ ] Selecting more than 3 players shows clear error message and does not save
- [ ] Form is read-only when predictions are locked
- [ ] User can update predictions before lock; after lock, form is read-only

**Dependencies:** R1, cavekit-tournament (active tournament)

### R4: Lock Enforcement (Server-Side)
**Description:** Once tournament predictions are locked, all prediction save handlers reject new submissions and return appropriate responses.

**Acceptance Criteria:**
- [ ] POST `/predictions/group` checks tournament.predictions_locked_at before saving
- [ ] If locked, returns 400 Bad Request with message "Predictions are locked"
- [ ] POST `/predictions/knockout/{round}` checks tournament lock before saving
- [ ] If locked, returns 400 Bad Request
- [ ] POST `/predictions/top-scorer` checks tournament lock before saving
- [ ] If locked, returns 400 Bad Request
- [ ] Forms (GET) always render but are CSS-disabled when locked (form inputs disabled, submit button hidden)
- [ ] User sees clear message "Predictions are locked" on locked form
- [ ] Lock check is done server-side (not just client-side) on every POST

**Dependencies:** R1, R2, R3, cavekit-tournament (predictions_locked_at)

### R5: Prediction Visibility (Hidden Until Lock)
**Description:** Other users' predictions are hidden until the tournament is locked.

**Acceptance Criteria:**
- [ ] Before lock: user can see only their own predictions on `/predictions`
- [ ] Before lock: user cannot see other users' predictions on that page
- [ ] After lock: predictions remain visible only to the user (no change in visibility)
- [ ] Predictions are not exposed in the API or any page before lock (401/403 if attempted)
- [ ] Review page (R6) shows all predictions post-lock

**Dependencies:** R1, R2, R3, cavekit-tournament (predictions_locked_at)

### R6: Per-League Review Page
**Description:** After predictions are locked, league members can see a review of all members' predictions vs actual results.

**Acceptance Criteria:**
- [ ] GET `/leagues/{id}/predictions/review` shows review page (league members only)
- [ ] Page displays: league name, tournament name, review data
- [ ] For each prediction (group stage, knockout, top scorer), show:
  - [ ] User's name
  - [ ] User's prediction
  - [ ] Actual result (if available) or "Pending" (if match not finished)
  - [ ] Points awarded (correct/incorrect state, not yet scored = "—")
- [ ] Group stage table: one row per match, columns for each league member showing prediction outcome
- [ ] Knockout table: one row per round, columns for each member showing predicted team
- [ ] Top scorer section: list of all top scorer predictions by member
- [ ] Page is accessible only to league members (401/403 enforced)
- [ ] Review page shows only members of that specific league (filtered by league_members)
- [ ] Page is blank or shows "No tournament active" if no active tournament
- [ ] Page is accessible before lock (shows "Predictions" but not yet locked) and after lock

**Dependencies:** R1, R2, R3, R6, cavekit-leagues (league membership), cavekit-scoring (points_awarded)

## Out of Scope

- Editing predictions after submission (only form shows current state and allows re-submission)
- Prediction comments or notes
- Historical prediction tracking (only current tournament predictions stored)
- Prediction statistics or accuracy per-user
- Exporting predictions to CSV or PDF
- Batch import of predictions from file
- Live prediction updates (no real-time sync, page refresh required)
- Mobile app-specific prediction forms
- Prediction search or filtering

## Gaps (Open Tasks)

### [GAP] R7: Prediction Completion Counter
**Task:** 0047 — Prediction completion counter

Shows how many matches have been predicted out of total. Not shown when locked.

**Acceptance Criteria:**
- [ ] Group stage tab shows "18 / 36 predicted" counter
- [ ] Counter reflects server-side state on page load
- [ ] Counter increments/decrements as user adds/removes predictions (via HTMX)
- [ ] Counter shows visually distinct "complete" state when all matches predicted (e.g., green checkmark)
- [ ] Counter is not shown when predictions are locked
- [ ] Counter is accurate: counts only actual `group_stage_predictions` rows in DB for this user

**Implementation note:** Pre-compute `total_matches` and `predicted_matches` in handler; pass to template. HTMX fragment updates counter on form POST.

**Dependencies:** R1 (Group Stage Prediction Form)

### [GAP] R8: Show Actual Match Results After Kickoff
**Task:** 0048 — Show actual match results on predictions page

After a match is played, show the actual score and outcome alongside user's prediction.

**Acceptance Criteria:**
- [ ] Group stage match cards show actual score (home_score – away_score) when match has finished
- [ ] User's prediction is visually marked correct (green) or incorrect (red) vs actual outcome
- [ ] Unplayed matches show only scheduled kickoff time and prediction form; no score displayed
- [ ] Pre-tournament state (no results at all) continues to work correctly
- [ ] Correct predictions show: "✓ Correct: You predicted home, actual: home"
- [ ] Incorrect predictions show: "✗ Wrong: You predicted draw, actual: away"
- [ ] Pending/unplayed matches show no result message
- [ ] Result display is read-only (no form interaction with finished matches)

**Implementation note:** Add `home_score`, `away_score`, `outcome` to match queries. Add template helper for prediction correctness. Only show result display when `match.status == "FINISHED"` (from API data).

**Dependencies:** R1, cavekit-scoring (match scoring)

## Source Traceability

### Brownfield Status: Mostly Complete (2 gaps)
R1-R6 are fully implemented. R7 and R8 are open tasks.

### Source Files
- `src/modules/predictions/mod.rs` — router() with prediction routes
- `src/modules/predictions/handlers.rs` — predictions_page, save_group, save_knockout, save_top_scorer, review handlers
- `src/modules/predictions/db.rs` — prediction CRUD, queries for display
- `src/modules/predictions/models.rs` — GroupStageForm, KnockoutForm, TopScorerForm, review types
- `migrations/0007_predictions.sql` — group_stage_predictions, knockout_predictions, top_scorer_predictions tables
- `templates/predictions/index.html` — main prediction form
- `templates/predictions/review.html` — per-league review page

### Implementation Notes
- Forms use HTMX for individual round/section submission (not full page POST)
- Lock enforcement happens in every POST handler with early return
- Prediction queries include a filter for the active tournament (skip if multiple tournaments)
- Review page queries all predictions for a tournament + all league members simultaneously

## Cross-References
- Depends on: **cavekit-auth.md** (AuthSession)
- Depends on: **cavekit-tournament.md** (active tournament, lock state)
- Depends on: **cavekit-leagues.md** (league membership for review page access)
- Consumed by: **cavekit-scoring.md** (predictions are scored after results)
- Consumed by: **cavekit-standings.md** (predictions shown in match breakdown and compare pages)
