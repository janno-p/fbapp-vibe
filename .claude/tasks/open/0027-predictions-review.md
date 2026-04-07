---
id: 0027
title: Read-only predictions review page
status: open
type: feature
adrs: [0007, 0009, 0005]
refs: [0007, 0025]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Once predictions are locked, users cannot see what they submitted — the prediction forms become read-only inputs but there is no review page that shows each prediction alongside the actual result and points awarded. A review page lets users understand exactly how their score was built up and compare their choices against outcomes.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/predictions/review` renders the viewer's predictions for the active tournament alongside actual results and points awarded
- [ ] Group stage section: each predicted match shows home/away teams, user's predicted outcome, actual outcome, points awarded
- [ ] Knockout section: each predicted team advancement, actual result, points awarded
- [ ] Top scorer section: predicted player, actual goals scored so far, points awarded
- [ ] Unscored/future matches display "—" for points
- [ ] Only accessible to authenticated league members (401 / 403 for others)
- [ ] Page is linked from the league leaderboard and/or league overview

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/handlers.rs` — add `predictions_review` handler; existing handler already has auth pattern
- `src/modules/predictions/db.rs` — add `get_user_predictions_review(pool, tournament_id, user_id)` query that joins predictions with match/player tables and returns scored rows
- `src/modules/predictions/models.rs` — add `GroupPredictionReviewRow`, `KnockoutPredictionReviewRow`, `TopScorerReviewRow`; also a `PredictionsReview` aggregate struct
- `src/modules/predictions/mod.rs` — register new route `GET /leagues/{id}/predictions/review`
- `templates/predictions/review.html` — new template
- `src/modules/leagues/db.rs:is_member` — reuse for membership check

### ADR constraints

- **ADR-0007**: New route inside the existing `predictions` module
- **ADR-0009**: Return `AppError::Unauthorized` / `AppError::Forbidden` for access control

### Tests

- No tests — query is a set of SELECT JOINs; handler is trivial auth + render

### Implementation notes

- Prediction tables: `group_predictions (user_id, match_id, predicted_outcome, points_awarded)`, `knockout_predictions (user_id, team_id, round, points_awarded)`, `top_scorer_predictions (user_id, player_id, points_awarded)`
- Join group predictions with `matches` (for teams and scheduled_at and actual outcome/score) and filter by `tournament_id`
- Join knockout predictions with `teams` and `matches` tables for names and actual round results
- Join top scorer prediction with `players` and `player_tournament_stats` (or similar) for goal tally
- Points awarded of `NULL` means not yet scored — display as "—"
- Route includes `{id}` (league id) for membership check; tournament_id comes from the active tournament (query `tournaments WHERE is_active = true` or use nav context)
- Template sections: three collapsible or tabbed sections for Group / Knockout / Top Scorer
- Askama `|length` filter is unavailable — use `.len()` method calls in template
- Date formatting: add `formatted_kickoff()` to `GroupPredictionReviewRow` using the same pattern as `MatchInfo::formatted_kickoff()`

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
