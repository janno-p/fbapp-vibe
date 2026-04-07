---
id: 0018
title: Scenario modeling — hypothetical result simulation on standings page
status: open
type: feature
adrs: [0016]
refs: [0009]
created: 2026-04-07
started: ~
completed: ~
---

## Goal

Let users explore "what if" scenarios on the standings page: select a hypothetical outcome for an upcoming match and see how the leaderboard would shift, without persisting anything to the database. This was scoped out of task 0009.

## Acceptance Criteria

- [ ] On the standings page, unplayed group stage matches show a hypothetical-result picker (home / draw / away)
- [ ] Selecting a result re-renders the leaderboard with those hypothetical points applied on top of actual scored points
- [ ] Multiple unplayed matches can be hypothesised simultaneously (state is carried via query params or a form POST)
- [ ] Hypothetical results do not write to the database; all computation is in-memory
- [ ] The leaderboard clearly distinguishes actual points from hypothetical points (e.g. a "(+N projected)" suffix)
- [ ] Clearing hypothetical picks returns the leaderboard to actual standings

## Context for Claude 🤖

### Relevant files

- `src/modules/standings/handlers.rs` — add scenario variant of `standings_page` or extend existing handler
- `src/modules/standings/db.rs` — new query to fetch unplayed group matches with predictions per user
- `src/modules/standings/models.rs` — pure function for applying hypothetical outcomes to a prediction set
- `templates/standings/index.html` — add the hypothetical picker form and projected leaderboard section

### Approach

1. Add a query `get_unplayed_group_matches_with_predictions(pool, tournament_id, league_id)` that returns every unplayed group match plus each league member's predicted outcome.

2. Accept hypothetical outcomes as query params: `?hypo[{match_id}]=home|draw|away`. Parse these in the handler using `serde_qs` (already a dependency via `QsForm`).

3. Implement a pure function:
   ```rust
   pub fn apply_hypothetical(
       baseline: &[LeaderboardRawRow],
       unplayed_predictions: &[UnplayedMatchPredictions],
       hypothetical_outcomes: &HashMap<i64, MatchOutcome>,
   ) -> Vec<LeaderboardRawRow>
   ```
   For each hypothetical outcome, add 1 point to every user whose prediction matches.

4. Re-use `build_leaderboard` to produce the final sorted entries with ranks.

5. In the template, render a form with one `<select>` per unplayed match; the form submits via `GET` (preserves bookmarkable URLs). HTMX can submit on change without a submit button.

### Tests

- Unit test for `apply_hypothetical`: baseline with two users, one unplayed match, hypothetical = correct for user A only → user A gains 1 projected point.
- Unit test: multiple matches → points accumulate correctly.
- Unit test: hypothetical outcome matches no one's prediction → leaderboard unchanged.
- `#[sqlx::test]` for the new DB query: insert unplayed match with two users' predictions, assert both rows returned.

### Implementation notes

- Use `serde_qs::from_str` or the existing `QsForm` extractor for parsing the nested `hypo[{id}]` params from the query string (not request body).
- Projected points should be additive on top of `total_points`; keep `total_points` (actual) and add `projected_points` (hypothetical delta) as a separate field on the view model.
- Do not touch `max_achievable` computation — that already accounts for all unplayed matches.
- HTMX: add `hx-get` + `hx-trigger="change"` on each picker; target the leaderboard wrapper to replace it on each change.

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
