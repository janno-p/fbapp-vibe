---
date: 2026-04-24T12:26:03+03:00
git_commit: 0cc6e159fcdeae3f3e399d1c50d15f48a0ff32ff
branch: main
repository: fbapp-vibe
topic: "FEATURE-002: User model for auth integration"
tags: [research, codebase, auth, user-model, database, sessions]
last_updated: 2026-04-24
---

## Ticket Synopsis

`thoughts/tickets/feature_cavekit_user_model.md` asks whether Cavekit has the auth user model required for session-backed Google OAuth: a persisted user record with `id`, `google_id`, `email`, `name`, `avatar_url`, and `is_admin`; a database-level unique email constraint; an `axum_login::AuthUser` implementation; and user restoration by id through `AuthBackend.get_user()`.

The referenced source spec `context/kits/cavekit-auth.md` is not present in this repository, so this research uses live code plus existing `thoughts/` documents as the sources of truth.

## Summary

The core user model for auth integration is already implemented in live code. `User` has all six required fields, implements `axum_login::AuthUser` with `i64` database ids, and uses `email` as the session auth hash so email changes invalidate existing sessions (`src/modules/auth/models.rs:3-24`).

The database schema also exists. `migrations/0002_create_users.sql` creates `users` with unique `google_id` and unique `email`, and `migrations/0008_admin_role.sql` adds `is_admin BOOLEAN NOT NULL DEFAULT FALSE` (`migrations/0002_create_users.sql:1-8`, `migrations/0008_admin_role.sql:1`).

Session restoration is wired through `AuthBackend::get_user`, which loads the full `User` row by id from PostgreSQL and is used by the production `AuthManagerLayerBuilder` stack (`src/modules/auth/mod.rs:17-53`, `src/main.rs:36-67`). Existing tests verify model fields, `AuthUser` behavior, profile upsert behavior, admin flag preservation, and lookup by id. The one acceptance gap found is that there does not appear to be a dedicated test that intentionally violates the unique email constraint, even though the constraint is present in the migration.

## Detailed Findings

### User Model Shape

- `User` is defined in `src/modules/auth/models.rs` and derives `Debug`, `Clone`, and `sqlx::FromRow`, making it usable as the SQLx query result type for auth database reads (`src/modules/auth/models.rs:3-4`).
- The struct contains the ticket-required fields: `id: i64`, `google_id: String`, `email: String`, `name: String`, `avatar_url: Option<String>`, and `is_admin: bool` (`src/modules/auth/models.rs:4-11`).
- Unit coverage explicitly asserts the field values and optional avatar behavior in `user_has_all_required_fields` and `user_avatar_url_is_optional` (`src/modules/auth/models.rs:56-79`).
- `google_id` is currently annotated with `#[allow(dead_code)]`, which suggests some production paths create and return it for completeness while not always reading it directly outside tests (`src/modules/auth/models.rs:6-7`).

### AuthUser Integration

- `User` implements `axum_login::AuthUser` with `type Id = i64`, matching the `BIGSERIAL` primary key in the database (`src/modules/auth/models.rs:14-19`, `migrations/0002_create_users.sql:1-2`).
- `session_auth_hash()` returns `self.email.as_bytes()`, which binds active sessions to the user's current email and invalidates sessions after email changes (`src/modules/auth/models.rs:21-23`).
- Unit tests verify both id extraction and the email-backed auth hash, including a hash change when the email changes (`src/modules/auth/models.rs:81-104`).
- HTTP integration coverage confirms that changing `users.email` after login causes the same browser session to receive `401 Unauthorized` on `/dashboard` (`tests/auth_routes.rs:499-516`).

### Database Schema And Constraints

- `migrations/0002_create_users.sql` creates `users` with `id BIGSERIAL PRIMARY KEY`, `google_id TEXT NOT NULL UNIQUE`, `email TEXT NOT NULL UNIQUE`, `name TEXT NOT NULL`, nullable `avatar_url`, and `created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` (`migrations/0002_create_users.sql:1-8`).
- `migrations/0008_admin_role.sql` adds `is_admin BOOLEAN NOT NULL DEFAULT FALSE`, which satisfies the ticket's binary role-state requirement and ensures new users default to non-admin (`migrations/0008_admin_role.sql:1`).
- The unique email requirement is enforced at the database level by the migration, not just by application logic (`migrations/0002_create_users.sql:4`).
- Searches did not find a dedicated automated test that inserts two different users with the same `email` and asserts a database constraint failure. Existing tests rely on the schema via migrated SQLx test databases, but they do not directly exercise this constraint violation.

### Google Profile Upsert

- User synchronization is centralized in `find_or_create_user`, which accepts `google_id`, `email`, `name`, and optional `avatar_url`, inserts a row, and returns `id, google_id, email, name, avatar_url, is_admin` (`src/modules/auth/db.rs:5-31`).
- The upsert key is `ON CONFLICT (google_id)`, so the local account is treated as one row per Google identity (`src/modules/auth/db.rs:15-21`).
- On repeated login for the same Google identity, the code refreshes `email`, `name`, and `avatar_url` from Google profile data (`src/modules/auth/db.rs:17-20`).
- The upsert intentionally does not modify `is_admin`, and the test `preserves_is_admin_flag_on_conflict` verifies that an admin remains admin after profile refresh (`src/modules/auth/db.rs:97-114`).
- Because `email` is separately unique, an upsert for a new `google_id` with an email already attached to another user will fail instead of merging accounts. That matches the ticket's instruction not to expand into account linking.

### AuthBackend.get_user

- `AuthBackend` owns a `sqlx::PgPool` and is documented as restoring users from PostgreSQL for session-backed auth (`src/modules/auth/mod.rs:17-25`).
- Its `AuthnBackend` implementation uses `type User = User`, `type Credentials = models::Credentials`, and `type Error = sqlx::Error` (`src/modules/auth/mod.rs:29-33`).
- `authenticate()` returns `Ok(None)` because OAuth does not authenticate through credential submission; production login calls `auth_session.login(&user)` after token exchange instead (`src/modules/auth/mod.rs:34-40`, `src/modules/auth/handlers.rs:154-158`).
- `get_user()` selects `id, google_id, email, name, avatar_url, is_admin FROM users WHERE id = $1` and returns `fetch_optional`, which supports both valid session restoration and missing-user rejection (`src/modules/auth/mod.rs:42-53`).
- SQLx tests cover both `get_user_returns_user_by_id` and `get_user_returns_none_for_unknown_id` (`src/modules/auth/mod.rs:72-92`).

### Runtime Session Wiring

- Production config creates a PostgreSQL-backed `PostgresStore`, wraps it in `SessionManagerLayer`, builds an auth layer with `AuthManagerLayerBuilder::new(AuthBackend::new(pool.clone()), session_layer)`, and layers it over `routes::router(state)` (`src/main.rs:36-67`).
- The auth router exposes `/`, `/dashboard`, `/auth/login`, `/auth/callback`, and `/auth/logout`, and is merged into the top-level router (`src/modules/auth/mod.rs:56-63`, `src/routes.rs:6-16`).
- The dashboard route requires `auth_session.user` and returns `AppError::Unauthorized` when no restored user is available (`src/modules/auth/handlers.rs:53-64`).
- The Google OAuth callback fetches `GoogleUserInfo`, calls `find_or_create_user`, then logs in that returned `User`, connecting Google profile data to the session-backed model (`src/modules/auth/handlers.rs:133-158`).

### Admin Role Consumers

- `is_admin` is consumed by the `AdminUser` extractor, which restores `AuthSession`, returns `401` if no user is present, and returns `403` if the restored user has `is_admin == false` (`src/modules/admin/mod.rs:18-37`).
- The admin router exposes tournament and competition management routes that use handlers gated by this extractor (`src/modules/admin/mod.rs:40-65`).
- Integration tests create a user, flip `is_admin` through SQL, and verify that non-admin users receive `403` while admin users can access `/admin` (`tests/auth_routes.rs:135-140`, `tests/auth_routes.rs:456-480`).

## Code References

- `src/modules/auth/models.rs:3-24` - `User` struct and `axum_login::AuthUser` implementation.
- `src/modules/auth/models.rs:56-104` - Unit tests for required fields, optional avatar, auth id, and email-backed auth hash.
- `migrations/0002_create_users.sql:1-8` - Base `users` table with unique `google_id` and unique `email`.
- `migrations/0008_admin_role.sql:1` - `is_admin` role column with default `false`.
- `src/modules/auth/db.rs:5-31` - Google-profile upsert returning the full auth user row.
- `src/modules/auth/db.rs:39-114` - SQLx tests for create, optional avatar, profile update, and admin flag preservation.
- `src/modules/auth/mod.rs:17-53` - `AuthBackend` and `get_user()` implementation.
- `src/modules/auth/mod.rs:72-92` - SQLx tests for lookup by id and unknown id.
- `src/modules/auth/handlers.rs:133-158` - OAuth callback maps Google user info into local user upsert and logs in the returned user.
- `src/main.rs:36-67` - PostgreSQL session store and auth middleware wiring.
- `src/modules/admin/mod.rs:18-37` - `AdminUser` uses `is_admin` for binary role enforcement.
- `tests/auth_routes.rs:370-412` - HTTP-level OAuth callback test creates a user from mocked Google user info.
- `tests/auth_routes.rs:499-516` - Email-change session invalidation integration test.

## Architecture Insights

The implementation separates identity persistence from session restoration. Google OAuth profile data is normalized through `find_or_create_user`, while `AuthBackend::get_user()` is the sole session restoration path used by `axum-login`. This means authenticated request handling gets a fresh database-backed `User` instead of relying only on serialized session data.

The app intentionally uses binary authorization. `is_admin` lives directly on `users`, defaults to `false`, and is enforced through a reusable extractor rather than a separate roles table or permission system. This matches the ticket's key decision to avoid expanding into account linking or fine-grained roles.

Email uniqueness and email-backed `session_auth_hash()` create an important coupling: email is both contact identity and a session invalidation factor. That is already covered by integration tests for email changes, but a dedicated duplicate-email database test would make the explicit schema constraint acceptance criterion stronger.

## Historical Context (from thoughts/)

- `thoughts/tickets/auth-module.md` - Original auth module plan specified the users table shape, Google OAuth flow, and `AuthUser` implementation with email-backed `session_auth_hash`; it predates the later `is_admin` migration (`thoughts/tickets/auth-module.md:20-31`, `thoughts/tickets/auth-module.md:91-133`).
- `thoughts/tickets/feature_cavekit_google_oauth_login_flow.md` - Current OAuth ticket requires callback user sync for Google ID, email, name, and avatar URL and states the session should be PostgreSQL-backed (`thoughts/tickets/feature_cavekit_google_oauth_login_flow.md:19-34`).
- `thoughts/tickets/feature_cavekit_admin_role_access_control.md` - Admin access ticket documents the binary `is_admin` role model and reusable `AdminUser` extractor expectations (`thoughts/tickets/feature_cavekit_admin_role_access_control.md:19-31`, `thoughts/tickets/feature_cavekit_admin_role_access_control.md:53-57`).
- `thoughts/tickets/debt_cavekit_auth_integration_tests.md` - Auth regression ticket is marked done and documents existing HTTP-level coverage for dashboard auth, logout, admin rejection, email-change invalidation, and expired sessions (`thoughts/tickets/debt_cavekit_auth_integration_tests.md:82-92`).
- `thoughts/research/2026-04-23_google_oauth_login_flow.md` - Prior research confirms the Google OAuth flow upserts users by `google_id`, preserves `is_admin`, and restores users through `AuthBackend::get_user()` (`thoughts/research/2026-04-23_google_oauth_login_flow.md:33-45`).
- `thoughts/research/2026-04-23_auth_integration_tests.md` - Prior research confirms email-change invalidation and HTTP-level auth/session behavior (`thoughts/research/2026-04-23_auth_integration_tests.md:47-56`).

## Related Research

- `thoughts/research/2026-04-23_google_oauth_login_flow.md` - OAuth callback, user sync, session creation, and protected route behavior.
- `thoughts/research/2026-04-23_auth_integration_tests.md` - Auth/session integration coverage and admin route behavior.

## Open Questions

- `context/kits/cavekit-auth.md` is referenced by this ticket and related auth tickets, but the file is missing from the repository. If it remains canonical, it should be restored or ticket references should be updated.
- The database-level unique email constraint exists in the migration, but no focused test was found that asserts duplicate email insertion fails. If strict completion requires the automated verification checkbox to be directly exercised, add a small SQLx test for that constraint.
- Sub-agent research could not be executed in this environment because each `Task` call failed with `ProviderModelNotFoundError`; this document is based on direct `Glob`, `Grep`, and `Read` evidence from the live repository.
