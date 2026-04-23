---
title: Cavekit tournament management epic
source: .claude/tasks/done/0005-tournament-management.md
source_id: 0005
source_status: done
source_title: Tournament management (admin)
status: created
phase: Backlog
type: feature
adrs: [0007, 0009, 0016]
refs: [0004]
created: 2026-04-06
started: 2026-04-06
completed: 2026-04-06
---

## Summary

Umbrella ticket for the Cavekit tournament management workstream. It groups the atomic tickets for registration, seeding, activation, manual locking, shared models, and team flag display.

## Acceptance Criteria

- [ ] Child tickets exist for each atomic requirement in the tournament kit.
- [ ] This ticket references the full set of child tickets in Related Information.
- [ ] Scope boundaries are preserved: no update/delete, no multi-active tournaments, no manual data entry.
- [ ] Completed child tickets can be tracked independently from this umbrella ticket.

## Implementation Context

### Relevant files

- `thoughts/tickets/feature_cavekit_tournament_registration.md` - registration ticket.
- `thoughts/tickets/feature_cavekit_tournament_seeding.md` - seeding ticket.
- `thoughts/tickets/feature_cavekit_tournament_activation.md` - activation ticket.
- `thoughts/tickets/feature_cavekit_manual_prediction_locking.md` - manual lock ticket.
- `thoughts/tickets/feature_cavekit_tournament_data_models.md` - shared model ticket.
- `thoughts/tickets/feature_cavekit_team_flag_display.md` - flag display ticket.
- `thoughts/tickets/feature_cavekit_auto_lock_on_first_kickoff.md` - existing auto-lock ticket.

### ADR constraints 

- **ADR-0007**: Feature modules stay isolated and expose only `router()`.
- **ADR-0009**: Use explicit auth/error responses for admin-only behavior.
- **ADR-0005**: Keep SQL compile-time checked with SQLx macros.
- **ADR-0016**: Use `external_id` as the conflict key for idempotent upserts.

### Auth guard

Check `auth_session.user.is_admin` at the handler level (or via a middleware applied to the admin router). Return `AppError::Unauthorized` if false.

### Tests

- `#[sqlx::test]` coverage exists in the child tickets for creation, seeding, activation, locking, and models.
- Child tickets should be used as the implementation and verification units.

### Implementation notes

- This file is the parent index, not the implementation ticket for any one subtask.
- Keep the children atomic and avoid re-expanding their scope here.

## Outcome

> Umbrella ticket created to track the tournament workstream. Implementation is split into the linked child tickets.

Follow-up tasks: #feature_cavekit_tournament_registration, #feature_cavekit_tournament_seeding, #feature_cavekit_tournament_activation, #feature_cavekit_manual_prediction_locking, #feature_cavekit_tournament_data_models, #feature_cavekit_team_flag_display, #feature_cavekit_auto_lock_on_first_kickoff
