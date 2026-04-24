---
title: Friendly inline error for wrong knockout/top-scorer selection count
source: .claude/tasks/done/0046-knockout-topscore-count-ux.md
source_id: 0046
source_status: open
source_title: Friendly inline error for wrong knockout/top-scorer selection count
status: reviewed
phase: MVP
type: bug
adrs: []
refs:
  - thoughts/plans/knockout_topscore_count_ux_closeout.md
created: 2026-04-09
started: ~
completed: ~
---

## Summary

When a user submits a knockout round form with the wrong number of teams (or top-scorer with fewer than 3 players), the server returns `AppError::BadRequest`, which renders a generic error page. The UI shows "Select X teams" as a hint but provides no client-side guard. Users who accidentally submit early get a hard error page instead of a friendly inline message.

## Acceptance Criteria

- [ ] Submitting a knockout round with the wrong team count shows an inline error near the form, not a full error page
- [ ] Submitting top scorer with fewer than 3 players shows an inline error
- [ ] The existing server-side count validation is kept (do not remove it); add client-side prevention as the first line of defence
- [ ] The inline error clearly states what the correct count should be

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — `save_knockout` (line 127, count check at line 138); `save_top_scorer` (line 163, count check at line 170)
- `templates/predictions/index.html` — knockout section (line 165+); each round has its own `<form>` with `hx-post` and `hx-target="#ko-{slug}-status"`; top-scorer section uses Alpine.js `playerPicker()`
- The knockout status target per round: `hx-target="#ko-{{ rs.round.slug() }}-status"` — this span exists but is only populated on successful save currently

### ADR constraints

- HTMX: return an HTML fragment with status 200 (or 422) from the handler for the swap to work
- Client-side validation is progressive enhancement — the server guard must stay

### Tests

No tests — handler and template wiring.

### Implementation notes

**Client-side (preferred first defence):**
Each knockout form already has `hx-target` pointing at a status span. Add a small Alpine.js guard or vanilla JS `hx-confirm`-style check. Alternatively, add a `required` count via `hx-params` is not straightforward — simplest is a short Alpine component or a `submit` event listener that checks checkbox count and calls `event.preventDefault()` with an error message.

**Server-side (fallback):**
Change `save_knockout` and `save_top_scorer` to return an HTMX-compatible HTML fragment instead of `AppError::BadRequest` for the count mismatch case, so the error appears inline:

```rust
if form.team_ids.len() != round.expected_team_count() {
    return Ok(Html(format!(
        r#"<span class="text-signal-red">Select exactly {} teams.</span>"#,
        round.expected_team_count()
    )).into_response());
}
```

The top-scorer Alpine `playerPicker()` already enforces max-3 selection at the UI level — verify whether it also prevents submit below 3. If not, add the same inline-error approach on the server.

## Outcome

- Server: `save_knockout` and `save_top_scorer` now return `Ok(Html(<span class="text-signal-red">Select exactly N teams/players.</span>))` for count mismatches, so HTMX swaps the error inline into the status span instead of rendering an error page.
- Client (knockout): Added `knockoutPicker(expected)` Alpine component on each round `<form>`; tracks checked count via `init()` + `@change="update()"`. Submit button is `:disabled="!valid"` with `disabled:opacity-40 disabled:cursor-not-allowed` styling.
- Client (top scorer): Extended `playerPicker()` with `count` property tracked in `init()` and updated inside `enforceMax3()`. Submit button is `:disabled="count !== 3"` with same disabled styling.

Follow-up tasks: _none_

## Closeout Notes

- Canonical rule for this code path remains **exactly 3** top-scorer picks.
- Cavekit wording that says "up to three" is tracked as wording drift and should be aligned separately from this closeout.
