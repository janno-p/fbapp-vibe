---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, tournament, lock]
keywords: [predictions_locked_at, lock, unlock, read-only, AdminUser, prediction forms]
patterns: [server-side write guard, lock-gated rendering, timestamp toggle, admin-only mutation]
---

# FEATURE-CAVEKIT-TOURNAMENT-04: Manual prediction lock and unlock

## Summary

Allow admins to manually lock or unlock tournament predictions so the submission window can be controlled directly.

## Acceptance Criteria

- [ ] `POST /admin/tournaments/{id}/lock` sets `predictions_locked_at = now()`.
- [ ] `POST /admin/tournaments/{id}/unlock` clears `predictions_locked_at`.
- [ ] Locked prediction forms are read-only and reject new submissions or edits.
- [ ] Locked forms still show the user's existing predictions.
- [ ] Only admin users can lock or unlock.
- [ ] Lock and unlock events are logged at info level.

## Implementation Context

### Relevant files

- `src/modules/admin/handlers.rs` - lock/unlock endpoints.
- `src/modules/admin/db.rs` - lock state persistence.
- `src/modules/predictions/handlers.rs` - read-only gating on the user-facing form.
- `src/modules/predictions/db.rs` - write-path lock checks.
- `templates/predictions/` - locked-state rendering.

### ADR constraints

- **ADR-0007**: Keep the admin mutation in the admin module.
- **ADR-0009**: Deny non-admin access explicitly.

### Tests

- `#[sqlx::test]` for setting and clearing the lock timestamp.
- Integration test for locked forms remaining visible but non-editable.

### Implementation notes

- This is the manual lock path only; auto-lock has its own ticket.
- Existing predictions should remain visible even when the form is locked.

## Research Context

### Keywords to Search

- `predictions_locked_at` - lock timestamp column.
- `lock` - manual lock endpoint.
- `unlock` - manual unlock endpoint.
- `read-only` - locked-form UI state.
- `AdminUser` - access control extractor.

### Patterns to Investigate

- server-side write guard - prevent mutation when locked.
- lock-gated rendering - show form state based on lock timestamp.
- timestamp toggle - set or clear a nullable timestamp field.
- admin-only mutation - mutation restricted to admins.

### Key Decisions Made

- Manual admin control should always be available.
- Locking does not hide the user's own current picks.
- The lock state is stored on the tournament row.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers lock and unlock persistence.
- [ ] `cargo test` covers locked-form write rejection.

### Manual Verification

- [ ] Locked forms cannot accept new predictions.
- [ ] Unlocking restores the editable form.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R4.
- Depends on tournament activation and prediction write enforcement.

## Notes

- Do not fold the auto-lock behavior into this ticket.
