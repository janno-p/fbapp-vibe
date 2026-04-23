---
type: bug
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, security, lock]
keywords: [prediction lock, server-side validation, predictions_locked_at, forbidden, bad request]
patterns: [server-side guard, early return on locked state, write-path validation, consistent error handling]
---

# BUG-PREDICTIONS-04: Enforce prediction lock on all save handlers

## Summary

Reject every prediction write once the tournament lock is active so users cannot bypass the UI and submit POST requests directly.

## Acceptance Criteria

- [ ] `POST /predictions/group` rejects writes when `predictions_locked_at` is set.
- [ ] `POST /predictions/knockout/{round}` rejects writes when `predictions_locked_at` is set.
- [ ] `POST /predictions/top-scorer` rejects writes when `predictions_locked_at` is set.
- [ ] Rejected requests return a clear error response instead of saving data.
- [ ] The lock check happens server-side in every write path.
- [ ] `cargo test` passes.

## Implementation Context

### Relevant files

- `src/modules/predictions/handlers.rs` — all save handlers
- `src/modules/predictions/db.rs` — active tournament lookup already exposes the lock field
- `src/error.rs` — error variants for rejected writes
- `src/modules/predictions/models.rs` — lock-state helper use in the UI

### ADR constraints

- **ADR-0009**: Use the existing application error model for rejected writes.
- **ADR-0007**: Keep the change inside the predictions module.

### Tests

- [ ] `#[sqlx::test]` or integration coverage for at least one locked write path.
- [ ] Verification that all three handlers share the same lock guard pattern.

### Implementation notes

- The UI lock state is not enough on its own.
- Apply the check immediately after loading the active tournament and before any DB write.

## Research Context

### Keywords to Search

- `predictions_locked_at` - lock field
- save_group - group write path
- save_knockout - knockout write path
- save_top_scorer - top scorer write path
- forbidden - rejection response

### Patterns to Investigate

- server-side guard - protect write handlers directly
- early return on locked state - cheapest safe path
- write-path validation - consistent business rule enforcement
- consistent error handling - same response shape across handlers

### Key Decisions Made

- Lock enforcement must happen on the server, not only in HTML.
- All prediction write endpoints should fail the same way once locked.
- This ticket is about enforcement only, not UI state.

## Success Criteria

The ticket is complete when direct POSTs cannot modify predictions after lock.

### Automated Verification

- [ ] `cargo test` proves locked writes are rejected.
- [ ] Handler tests cover all three save endpoints or their shared guard.

### Manual Verification

- [ ] A direct POST after lock fails.
- [ ] No prediction rows change after the rejected request.

## Related Information

- Source doc: `context/kits/cavekit-predictions.md`
- Requirement: `R4`

## Notes

Do not broaden this into permission changes, auth redesign, or new lock semantics.
