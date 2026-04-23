---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, tournament, admin, activation]
keywords: [is_active, activate, deactivate, single active tournament, AdminUser, active tournament]
patterns: [single-active invariant, transactional toggle, admin-only mutation, state propagation to reads]
---

# FEATURE-CAVEKIT-TOURNAMENT-03: Activate exactly one tournament at a time

## Summary

Give admins control over which tournament is live so the rest of the app can consistently read one active competition.

## Acceptance Criteria

- [ ] `POST /admin/tournaments/{id}/activate` sets `is_active = true`.
- [ ] Activating a tournament deactivates any currently active tournament.
- [ ] `POST /admin/tournaments/{id}/deactivate` sets `is_active = false`.
- [ ] Only admin users can call activate/deactivate endpoints.
- [ ] Activation and deactivation are logged at info level.
- [ ] Prediction pages can read the active tournament.

## Implementation Context

### Relevant files

- `src/modules/admin/handlers.rs` - activation endpoints.
- `src/modules/admin/db.rs` - single-active update logic.
- `src/modules/standings/` - reads that need the active tournament.
- `src/modules/predictions/` - pages that depend on the active tournament.

### ADR constraints

- **ADR-0007**: Keep admin feature code in `src/modules/admin/`.
- **ADR-0009**: Use explicit forbidden/unauthorized handling for non-admins.

### Tests

- `#[sqlx::test]` for activating one tournament deactivates the previous active row.
- Integration test for admin-only access to activation endpoints.

### Implementation notes

- This ticket is about the active-state invariant, not prediction locking.
- The active tournament should be a read-side source for prediction and standings pages.

## Research Context

### Keywords to Search

- `is_active` - tournament state column.
- `activate` - activation route and action name.
- `deactivate` - inverse route and action name.
- `single active tournament` - invariant to enforce.
- `AdminUser` - auth boundary.

### Patterns to Investigate

- single-active invariant - exactly one row may be active.
- transactional toggle - update one row and clear the prior active row together.
- admin-only mutation - state changes restricted to admins.
- state propagation to reads - consumers that need the current active tournament.

### Key Decisions Made

- There can be only one active tournament at a time.
- Activation is an admin-only server-side mutation.
- Other consumers read the active tournament rather than infer it themselves.

## Success Criteria

### Automated Verification

- [ ] `cargo test` proves only one tournament remains active after activation.
- [ ] `cargo test` covers the admin access guard.

### Manual Verification

- [ ] Activating a tournament updates the app-wide active state.
- [ ] Deactivating clears the active state.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R3.
- Depends on tournament registration and admin auth.

## Notes

- Do not add multi-active support here.
