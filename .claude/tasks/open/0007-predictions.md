---
id: 0007
title: Predictions UI
status: open
type: feature
adrs: [0007, 0009, 0016]
refs: [0005]
created: 2026-04-06
started: ~
completed: ~
---

## Goal

Let authenticated users submit and edit their tournament predictions before the prediction lock. Predictions cover three areas: group stage match outcomes, knockout round advancement, and top scorer candidates. Once `predictions_locked_at` is set on the active tournament, all prediction writes are rejected.

## Acceptance Criteria

- [ ] Predictions page is only accessible when there is an active tournament
- [ ] **Group stage tab**: displays all group stage matches; user selects home / draw / away for each; saves on submit
- [ ] **Knockout tab**: for each round (R16 → QF → SF → Final → Winner), user selects the expected advancing teams from the full team list; correct number of teams enforced per round (16 / 8 / 4 / 2 / 1)
- [ ] **Top scorer tab**: user selects exactly 3 players from the full player list
- [ ] All three forms support partial save (user does not need to fill everything in one go)
- [ ] Existing predictions are pre-filled when the page loads
- [ ] Any write attempt after `predictions_locked_at` returns 403; UI shows a "predictions locked" state instead of forms
- [ ] Prediction lock check uses `SELECT ... FOR UPDATE` on the tournament row (per ADR-0016 concurrency rules)

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/mod.rs` — new module
- `src/modules/predictions/handlers.rs`
- `src/modules/predictions/db.rs`
- `src/modules/predictions/models.rs`
- `templates/predictions/` — Askama templates (tabbed layout)
- `migrations/0007_predictions.sql` — already written

### ADR constraints

- **ADR-0016 (concurrency)**: Every prediction write must use `SELECT ... FOR UPDATE` on the tournament row before inserting/updating; see the Concurrency Control section of the ADR for the exact pattern
- **ADR-0009**: Return `AppError::Unauthorized` (403) when predictions are locked or user is not authenticated
- **ADR-0005**: `sqlx::query!` for all DB access

### Knockout round team counts

| Round | `knockout_round` value | Teams to predict |
|---|---|---|
| Round of 16 | `r16` | 16 |
| Quarter-finals | `qf` | 8 |
| Semi-finals | `sf` | 4 |
| Finalists | `final` | 2 |
| Winner | `winner` | 1 |

Enforce these counts at the handler level before any DB write.

### Tests

- Unit tests for the round team-count enforcement: pure function that validates the submitted team list length per `knockout_round` value — test all valid counts and at least one invalid count per round.
- `#[sqlx::test]` for prediction lock enforcement: insert a tournament with `predictions_locked_at` in the past, attempt a prediction write, assert it returns `AppError::Unauthorized`.
- `#[sqlx::test]` for group stage upsert: submit a prediction, change it, assert only one row exists with the updated value.
- No tests for template rendering.

### Implementation notes

- Use HTMX for form submissions so each tab saves independently without a full page reload
- Top scorer: player list should be searchable/filterable (simple `<input>` filter on the client is fine; no server-side search needed at this scale)
- Knockout predictions are stored as a set of rows (one per team per round); replacing a round's predictions means deleting existing rows for that round and inserting the new set, within a single transaction
- Group stage: single upsert per match is sufficient; no need to delete-and-reinsert
- Do not implement score display here — predictions page shows only prediction inputs, not points

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
