---
title: Show group stage prediction completion count
source: .claude/tasks/open/0047-group-prediction-completion-indicator.md
source_id: 0047
source_status: open
source_title: Show group stage prediction completion count
status: open
phase: Phase2
type: feature
adrs: []
refs: []
created: 2026-04-09
started: ~
completed: ~
---

## Summary

The group stage tab shows all matches but gives no indication of how many the user has already predicted. For a 36-match group stage, a user who partially fills the form and saves has no way to know they missed matches. A simple "12 / 36 predicted" counter near the save button would surface gaps before submission.

## Acceptance Criteria

- [ ] A completion counter is shown on the group stage tab (e.g. "18 / 36 predicted")
- [ ] The count reflects the current server-side state on page load (matches with a saved prediction)
- [ ] If all matches are predicted, the counter shows a visually distinct "complete" state
- [ ] The counter is not shown when predictions are locked (the form is read-only anyway)

## Implementation Context

### Relevant files

- `src/modules/predictions/models.rs` — `MatchRow` has `predicted_outcome: Option<MatchOutcome>`; `GroupWithMatches` holds a `Vec<MatchRow>`
- `src/modules/predictions/handlers.rs` — `PredictionsTemplate` has `groups: Vec<GroupWithMatches>`; the template has access to all matches
- `templates/predictions/index.html` — group section (line 44–163); save button area at line 147

### ADR constraints

- Askama templates cannot call arbitrary Rust functions — add a computed field or a template method on `PredictionsTemplate` if arithmetic is needed

### Tests

No tests — pure display logic derived from existing data.

### Implementation notes

The simplest approach is to compute the counts in the Askama template using its loop + filter support, or pre-compute them in the handler and pass as template fields.

Option A — template computation (Askama supports `filter` and `count`-style iteration):
```jinja
{% set total = groups | map(attribute="matches") | flatten | length %}
{% set predicted = groups | map(attribute="matches") | flatten | selectattr("predicted_outcome") | length %}
```
Askama's filter support is limited — verify what's available before choosing this option.

Option B — pre-compute in handler and add fields to `PredictionsTemplate`:
```rust
let total_matches: usize = groups.iter().map(|g| g.matches.len()).sum();
let predicted_matches: usize = groups.iter()
    .flat_map(|g| &g.matches)
    .filter(|m| m.predicted_outcome.is_some())
    .count();
```
Then add `total_matches` and `predicted_matches` fields to the template struct.

Option B is more straightforward given Askama's limited filter set. Place the counter just above the save button (line 147 area):

```html
<p class="text-sm text-ink-500">
  {{ predicted_matches }} / {{ total_matches }} predicted
</p>
```

When `predicted_matches == total_matches`, style the counter in `text-goal-400` to signal completion.

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
