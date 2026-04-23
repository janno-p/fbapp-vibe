---
title: Read-only predictions review page
source: .claude/tasks/done/0027-predictions-review.md
source_id: 0027
source_status: done
source_title: Read-only predictions review page
status: done
type: feature
adrs: [0007, 0009, 0005]
refs: [0007, 0025]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Summary

Once predictions are locked, users cannot see what they submitted — the prediction forms become read-only inputs but there is no review page that shows each prediction alongside the actual result and points awarded. A review page lets users understand exactly how their score was built up and compare their choices against outcomes.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/predictions/review` renders the viewer's predictions for the active tournament alongside actual results and points awarded
- [ ] Group stage section: each predicted match shows home/away teams, user's predicted outcome, actual outcome, points awarded
- [ ] Knockout section: each predicted team advancement, actual result, points awarded
- [ ] Top scorer section: predicted player, actual goals scored so far, points awarded
- [ ] Unscored/future matches display "—" for points
- [ ] Only accessible to authenticated league members (401 / 403 for others)
- [ ] Page is linked from the league leaderboard and/or league overview

## Implementation Context

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

Added `GET /leagues/{id}/predictions/review` — a read-only page showing the authenticated user's predictions for the active tournament alongside actual results and points.

**What was built:**
- Three new model types in `predictions/models.rs`: `GroupReviewRow`, `KnockoutReviewRow`, `TopScorerReviewRow`, each with a `points_display()` helper returning `"—"` for unscored entries
- Three read-only DB queries in `predictions/db.rs` (group, knockout, top scorer), each joining predictions with match/team/player tables and ordering for subheading grouping in the template
- `predictions_review` handler: auth check → membership check via `standings::db::is_member` → parallel query fetch → render
- `templates/predictions/review.html`: three sections with inline group/round subheadings using `{% let mut current_group %}` pattern; outcome badges coloured green/red by `score_state()`
- Route registered in `predictions/mod.rs`; "My predictions →" link in `standings/index.html` updated to point to this new route

**Deviations from spec:**
- Used `standings::db::is_member` (changed to `pub(crate)`) instead of `leagues::db::is_member` — both exist but `standings::db` was already imported in predictions context; this avoided adding a second cross-module dependency
- No collapsible/tabbed sections — flat layout with subheadings is sufficient and simpler

Follow-up tasks: _none_
