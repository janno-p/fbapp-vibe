---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, tournament, admin, football-data-org]
keywords: [competition list, external_id, season, register_tournament, AdminUser, tournament seeding]
patterns: [admin-only create flow, external source selection, post-create seeding, redirect after write]
---

# FEATURE-CAVEKIT-TOURNAMENT-01: Tournament registration from football-data.org

## Summary

Let admins register a new tournament from the football-data.org competitions list so the app can begin managing one competition end to end.

## Acceptance Criteria

- [ ] `/admin/competitions` lists available competitions from football-data.org.
- [ ] Admin submits `external_id`, custom `name`, and `season`.
- [ ] `POST /admin/tournaments` creates the tournament record in the database.
- [ ] New tournaments start with `is_active = false` and `predictions_locked_at = null`.
- [ ] Tournament seeding runs immediately after successful creation.
- [ ] Successful registration redirects back to `/admin`.

## Implementation Context

### Relevant files

- `src/modules/admin/handlers.rs` - competition list and tournament create handlers.
- `src/modules/admin/db.rs` - tournament insert and post-create seed flow.
- `src/modules/admin/models.rs` - register form payloads.
- `src/football_api.rs` - competition list client.
- `templates/admin/` - admin competition and registration UI.

### ADR constraints

- **ADR-0007**: Keep the feature in `src/modules/admin/` and expose only `router()`.
- **ADR-0009**: Return `Result<impl IntoResponse, AppError>` from handlers.
- **ADR-0005**: Use compile-time checked SQLx query macros.

### Tests

- `#[sqlx::test]` for tournament creation inserts the expected default state.
- Integration test for admin-only access to `/admin/competitions` and `POST /admin/tournaments`.

### Implementation notes

- This ticket only covers registration, not update/delete flows.
- Seeding is part of the post-create workflow, but the seed details live in the seeding ticket.

## Research Context

### Keywords to Search

- `football-data.org competitions` - source list for tournament selection.
- `external_id` - conflict key and tournament identity.
- `AdminUser` - access control extractor.
- `register_tournament` - likely handler or service entry point.

### Patterns to Investigate

- admin-only create flow - guard before tournament creation.
- external source selection - mapping API competition data into local form fields.
- post-create seeding - immediate continuation after insert.
- redirect after write - UX pattern after successful admin mutation.

### Key Decisions Made

- Tournament registration is admin-only.
- A newly created tournament is inactive until explicitly activated.
- Seeding happens immediately after registration.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers the create-and-seed path.
- [ ] `cargo clippy -- -D warnings` passes for the touched module.

### Manual Verification

- [ ] Admin can select a competition and create a tournament.
- [ ] The app returns to the admin dashboard after success.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R1.
- Depends on `cavekit-auth`.

## Notes

- Keep the flow narrow; do not add edit or delete capability here.
