---
title: Improve predictions review page — knockout grouping and correctness display
source: .claude/tasks/done/0039-predictions-review-knockout-ux.md
source_id: 0039
source_status: open
source_title: Improve predictions review page — knockout grouping and correctness display
status: open
phase: MVP
type: feature
adrs: []
refs: [0027]
created: 2026-04-08
started: ~
completed: ~
---

## Summary

The predictions review page (`/leagues/{id}/predictions/review`) has two UX gaps in the knockout section: (1) each knockout prediction is rendered as a standalone row, so the round structure is lost in a long list; (2) there is no visual signal for whether a knockout pick was correct or wrong — only the group stage section has proper correct/wrong/pending styling. This task groups knockout predictions by round into compact labeled blocks and adds the same colour-coded correctness indicators used by the group stage section.

## Acceptance Criteria

- [ ] Knockout predictions are displayed grouped by round — each round has a single header and all team picks for that round appear together in one block
- [ ] Each knockout pick shows a visual correctness state: green (correct), red (wrong), neutral/muted (pending — match not yet scored)
- [ ] Pending state is distinct from wrong — a pick with no result yet must not look the same as an eliminated team
- [ ] Group stage correctness display is unchanged (it already works correctly)
- [ ] `cargo test` passes

## Implementation Context

### How correctness is determined

`KnockoutReviewRow.points_awarded` (an `Option<i32>`) already encodes all three states:

| Value | Meaning | Display |
|---|---|---|
| `None` | Match not yet resolved / scorer hasn't run | Pending (neutral) |
| `Some(0)` | Team was eliminated — prediction was wrong | Wrong (red) |
| `Some(n > 0)` | Team advanced — prediction was correct | Correct (green) |

No DB query changes are needed. Add a `score_state() -> &'static str` method to `KnockoutReviewRow` in `models.rs` that maps these three cases to `"pending"`, `"wrong"`, `"correct"` — mirroring the existing method on `GroupReviewRow`.

### Relevant files

- `src/modules/predictions/models.rs` — add `score_state()` to `KnockoutReviewRow` (lines ~121–138)
- `templates/predictions/review.html` — redesign the knockout section (lines ~103–135); group stage section (lines ~32–101) and top scorer section (lines ~137–169) are **out of scope**
- `src/modules/predictions/db.rs` — read-only; no changes needed
- `src/modules/predictions/handlers.rs` — read-only; no changes needed

### Template redesign guidance

The current template loops over `knockout_rows` one by one and emits a round header only when the round changes. The new template should:

1. **Group by round** — all picks for the same round appear together inside one block. Askama doesn't have a native group-by filter; the simplest approach is to emit the round-header `<div>` when the round changes (same trick as today) and keep team rows visually inset/indented inside it.
2. **Compact layout** — within a round block, each team pick is a shorter row: crest + team name on the left, a correctness badge or dot on the right.
3. **Colour coding** — use the same Tailwind classes as the group stage section: green for correct, red for wrong, muted (e.g. `text-ink-500`) for pending. A coloured left-border or small indicator dot per row is sufficient — no need to change the overall card structure.

### ADR constraints

- No new ADRs required — this is a template + model-method change only.
- Keep using `sqlx::query_as!` patterns in `db.rs` if any query touches are needed (none expected).

### Tests

- Unit test `KnockoutReviewRow::score_state()` in `models.rs` — three cases: `None`, `Some(0)`, `Some(5)`. Keep it inside a `#[cfg(test)]` block in the same file, mirroring the existing `GroupReviewRow` test style if one exists.
- No DB or integration tests needed — the DB query is unchanged.

## Outcome

Added `score_state() -> &'static str` to `KnockoutReviewRow` in `models.rs` (mirrors `GroupReviewRow::score_state()`). Updated the knockout section in `templates/predictions/review.html` to show round headers (same change-detection trick as group stage) and per-team cards with a coloured left-indicator bar (green/red/muted) and text colour matching correctness state.

Simplified approach vs spec: kept individual cards per team rather than nested containers per round, which avoids Askama loop-control complexity while still achieving the visual grouping via the round header. Template is compile-time checked by Askama — no template tests needed.

3 unit tests added for `score_state()`: `None → pending`, `Some(0) → wrong`, `Some(n) → correct`. All 74 tests pass.

Follow-up tasks: _none_
