---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: reviewed
tags: [auth, user-model, database]
keywords: [user record, google_id, email unique, is_admin, AuthUser, AuthBackend.get_user]
patterns: [database entity modeling, auth user trait integration, user lookup by id]
---

# FEATURE-002: User model for auth integration

## Description
Define the user account model used by auth so identity, contact data, and role state are stored consistently and can be loaded for sessions.

## Context
Auth depends on a stable user representation that can be upserted from Google profile data and loaded by ID during session restoration.

## Requirements
- User record includes `id`, `google_id`, `email`, `name`, `avatar_url`, and `is_admin`.
- `email` is unique.
- User model implements `axum_login::AuthUser`.
- Users can be loaded by ID through `AuthBackend.get_user()`.

### Functional Requirements
- Persist the full user identity needed for login and role checks.
- Support loading users from the database during request handling.

### Non-Functional Requirements
- Enforce the unique email constraint at the database level.
- Keep the model compatible with session-backed auth integration.

## Current State
The model is described in the source spec and should be treated as an atomic ticket for planning and implementation.

## Desired State
A user model that cleanly supports OAuth identity, session restoration, and admin checks.

## Research Context

### Keywords to Search
- user record - core auth entity
- google_id - Google identity mapping
- is_admin - binary role flag
- AuthUser - session integration trait
- AuthBackend.get_user - user lookup path

### Patterns to Investigate
- database entity modeling - field shape and constraints
- auth user trait integration - trait implementation requirements
- user lookup by id - retrieval path for sessions

### Key Decisions Made
- Role model is binary: admin or regular user.
- Email must remain unique.
- Avatar URL is optional.

## Success Criteria
The ticket is complete when the model matches the required fields and auth integration points.

### Automated Verification
- [ ] Database tests validate the schema constraints.
- [ ] Unit or integration tests confirm `AuthUser` integration.
- [ ] User lookup by ID is verified.

### Manual Verification
- [ ] A login can create or update a user record.
- [ ] The loaded user contains the expected profile fields.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R2`

## Notes
Do not expand this into account linking or profile editing.

## Outcome

The auth user model was already implemented in `src/modules/auth/models.rs` with the required fields and `AuthUser` integration. `AuthBackend::get_user()` already restores users by id, and the database schema already enforced unique `email`.

This ticket closed by adding `rejects_duplicate_email_for_different_google_id` in `src/modules/auth/db.rs`, which proves a second Google identity cannot reuse an existing user's email. Verification passed with SQLx test databases available via the configured test database environment: `cargo test modules::auth::db::tests -- --nocapture`, `make test`, and `make lint`.
