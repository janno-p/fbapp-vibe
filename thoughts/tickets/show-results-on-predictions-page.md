---
title: Show actual match results alongside predictions after kickoff
source: .claude/tasks/open/0048-show-results-on-predictions-page.md
source_id: 0048
source_status: open
source_title: Show actual match results alongside predictions after kickoff
status: open
phase: Phase2
type: feature
adrs: []
refs: [0043]
created: 2026-04-09
started: ~
completed: ~
---

## Summary

Once predictions are locked and matches kick off, the predictions page becomes a read-only form showing only the user's picks — there's no indication of actual results. Users have to navigate to their league's review page to see how their picks fared. Showing the score and outcome inline on the predictions page gives users an at-a-glance view of how they're doing without leaving `/predictions`.

## Acceptance Criteria

- [ ] On the group stage tab, each match card shows the actual score (`home_score – away_score`) when the match is finished (`outcome IS NOT NULL`)
- [ ] The user's prediction is visually distinguished as correct or incorrect relative to the actual outcome
- [ ] Unplayed matches continue to show the scheduled kickoff time only (no score)
- [ ] The page still works correctly when predictions are not yet locked (pre-tournament state: no results to show)

## Implementation Context

### Relevant files

- `src/modules/predictions/models.rs` — `MatchRow`; add `home_score: Option<i32>`, `away_score: Option<i32>`, `outcome: Option<MatchOutcome>` if not already present
- `src/modules/predictions/db.rs` — `get_group_matches_with_predictions` query; needs to select `m.home_score`, `m.away_score`, `m.outcome` from the `matches` table
- `templates/predictions/index.html` — match card in group tab (line 63–142)

### ADR constraints

- `sqlx::query_as!` for compile-time checking
- `MatchRow` fields are added as `Option<T>` since they are NULL until the match finishes

### Tests

No tests — display-only change, derived from existing match data. The DB query change is a straightforward column addition.

### Implementation notes

The `matches` table already has `home_score INT`, `away_score INT`, and `outcome match_outcome` columns (set by the polling task when results come in). The `get_group_matches_with_predictions` query just needs to include them.

In the match card template, add a result display below the dropdown (which is disabled when locked):

```html
{% if let Some(hs) = m.home_score %}
<div class="text-xs text-center mt-1 font-display font-bold
  {% if m.prediction_correct() %}text-goal-400{% else %}text-signal-red{% endif %}">
  {{ hs }} – {{ m.away_score.unwrap() }}
</div>
{% endif %}
```

Add a helper method `prediction_correct() -> bool` on `MatchRow`:
- Returns `true` if `predicted_outcome == outcome` (both Some and equal)
- Returns `false` if both are Some but differ, or if either is None

Keep the kickoff time line as-is for unplayed matches — the result line is only rendered when `home_score` is Some.

Knockout results are out of scope for this task — the knockout tab is team-selection based and doesn't map to individual match scores.

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
