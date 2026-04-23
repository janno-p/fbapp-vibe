---
date: 2026-04-23T16:42:51+03:00
git_commit: 1b291c0347a5d17a36dd4e5b48d7c49ea2ef597c
branch: main
repository: fbapp-vibe
topic: "DEBT-007: Auth integration test coverage"
tags: [research, codebase, auth, sessions, integration-tests, admin]
last_updated: 2026-04-23
---

## Ticket Synopsis

The ticket asks for HTTP-level integration coverage for six auth regressions: unauthenticated `/dashboard`, authenticated `/` redirect, logout destroying the session, non-admin rejection on admin routes, email-change invalidation, and expired-session rejection. The referenced source file `context/kits/cavekit-auth.md` is not present in this repository, so live code and existing ticket history are the primary sources.

## Summary

The requested coverage already exists in the current codebase. `tests/auth_routes.rs` contains integration tests for every acceptance item in the ticket, and those tests build the full app stack with `SessionManagerLayer` and `AuthManagerLayer` so the behavior is exercised at the real HTTP boundary rather than by re-implementing extractor logic (`tests/auth_routes.rs:49-89`, `tests/auth_routes.rs:112-206`).

The runtime auth behavior these tests assert is also present in production code. `/dashboard` requires an authenticated `AuthSession` user (`src/modules/auth/handlers.rs:53-64`), `/` redirects authenticated users to `/dashboard` (`src/modules/auth/handlers.rs:44-51`), logout calls `auth_session.logout()` and redirects home (`src/modules/auth/handlers.rs:169-176`), and the `AdminUser` extractor returns `401` for missing auth and `403` for authenticated non-admin users (`src/modules/admin/mod.rs:18-37`). Session invalidation on email change is driven by `User::session_auth_hash()` being derived from `email` (`src/modules/auth/models.rs:14-24`).

## Detailed Findings

### Existing Integration Coverage

- The auth integration test helper reproduces the production auth/session stack by wiring `PostgresStore`, `SessionManagerLayer`, `AuthManagerLayerBuilder`, the real app router, and a test-only login route into a `TestServer` with persisted cookies (`tests/auth_routes.rs:49-89`).
- Unauthenticated `/dashboard` coverage already exists as `dashboard_requires_auth`, asserting `401 Unauthorized` (`tests/auth_routes.rs:111-116`).
- Authenticated `/` redirect coverage already exists as `home_redirects_authenticated_user_to_dashboard`, asserting `303 See Other` and `Location: /dashboard` (`tests/auth_routes.rs:132-144`).
- Logout invalidation coverage already exists as `logout_destroys_session`, which logs a user in, confirms `/dashboard` works, posts to `/auth/logout`, and then confirms the same cookie can no longer access `/dashboard` (`tests/auth_routes.rs:118-130`).
- Expired-session coverage already exists as `expired_session_returns_401`, which updates `tower_sessions.session.expiry_date` into the past before asserting `/dashboard` becomes unauthorized (`tests/auth_routes.rs:172-187`).
- Email-change invalidation coverage already exists as `email_change_invalidates_session`, which mutates `users.email` and then asserts the current session is rejected (`tests/auth_routes.rs:189-206`).
- Admin HTTP-level gating is already covered in auth integration tests via `non_admin_user_gets_403_on_admin_route` and `admin_user_can_access_admin_route` (`tests/auth_routes.rs:146-170`).

### Runtime Auth Behavior

- The auth router owns `/`, `/dashboard`, `/auth/login`, `/auth/callback`, and `/auth/logout`, and it is merged into the top-level app router in `src/routes.rs` (`src/modules/auth/mod.rs:56-63`, `src/routes.rs:6-16`).
- `home()` redirects authenticated users to `/dashboard` and renders the public home template otherwise (`src/modules/auth/handlers.rs:44-51`).
- `dashboard()` enforces auth by requiring `auth_session.user` and returning `AppError::Unauthorized` when absent (`src/modules/auth/handlers.rs:53-64`).
- `logout()` calls `auth_session.logout()` directly, so invalidation happens through the auth/session framework rather than bespoke DB cleanup (`src/modules/auth/handlers.rs:169-176`).
- The production app stack applies the same session/auth middleware shape used by the integration tests: `PostgresStore` + `SessionManagerLayer` + `AuthManagerLayerBuilder` layered over `routes::router(state)` (`src/main.rs:36-60`).

### Admin Authorization Contract

- `AdminUser` is the central authorization boundary. It restores `AuthSession` from request parts, returns `401 Unauthorized` when no authenticated user is available, and returns `403 Forbidden` when `user.is_admin` is false (`src/modules/admin/mod.rs:18-37`).
- Admin routes in the admin module are all gated through handlers that take `AdminUser` (`src/modules/admin/mod.rs:40-65`, `src/modules/admin/handlers.rs:34-141`).
- The same extractor is reused outside the admin module for league admin routes, showing the repo-wide pattern is reusable extractor-based HTTP gating, not ad hoc handler checks (`src/modules/leagues/mod.rs:14-20`, `src/modules/leagues/handlers.rs:35-54`).

### Session Invalidation Mechanics

- `User` implements `axum_login::AuthUser`, and `session_auth_hash()` returns `self.email.as_bytes()`, making email changes a built-in session invalidation signal (`src/modules/auth/models.rs:14-24`).
- Unit tests in `auth/models.rs` explicitly document that the session auth hash is derived from email and changes when email changes (`src/modules/auth/models.rs:88-104`).
- Expired session cleanup is also implemented separately as a background task deleting rows from `tower_sessions.session` whose `expiry_date <= NOW()`; the integration test exercises expiry rejection before cleanup runs, which is the correct HTTP-level behavior (`src/session_cleanup.rs:6-24`).

### Error Semantics

- `AppError::Unauthorized` maps to HTTP 401 and `AppError::Forbidden` maps to HTTP 403 in a single central `IntoResponse` implementation (`src/error.rs:37-52`).
- This means the ticket's required HTTP semantics are enforced both by handler/extractor logic and by shared error rendering (`src/error.rs:44-46`).

## Code References

- `tests/auth_routes.rs:49-89` - Integration test harness that recreates the real auth/session stack and adds a test-only login route.
- `tests/auth_routes.rs:111-116` - Unauthenticated `/dashboard` returns `401`.
- `tests/auth_routes.rs:118-130` - Logout destroys the session and blocks subsequent `/dashboard` access.
- `tests/auth_routes.rs:132-144` - Authenticated `/` redirects to `/dashboard`.
- `tests/auth_routes.rs:146-170` - Non-admin `/admin` access returns `403`; admin access succeeds.
- `tests/auth_routes.rs:172-187` - Expired session is rejected after forcing `expiry_date` into the past.
- `tests/auth_routes.rs:189-206` - Email change invalidates the session.
- `src/modules/auth/handlers.rs:44-64` - Home redirect logic and dashboard auth guard.
- `src/modules/auth/handlers.rs:169-176` - Logout implementation.
- `src/modules/auth/models.rs:14-24` - `AuthUser` implementation with email-backed `session_auth_hash`.
- `src/modules/admin/mod.rs:18-37` - `AdminUser` extractor returning 401/403 at the HTTP boundary.
- `src/error.rs:37-52` - Shared mapping from app errors to 401/403/404/500 responses.
- `src/main.rs:36-60` - Production middleware stack for sessions and auth.

## Architecture Insights

The codebase uses extractor-based HTTP authorization instead of duplicating access checks in tests or handlers. Standard authenticated routes read `auth_session.user` and return `AppError::Unauthorized` when absent, while admin-only routes centralize the stronger authorization rule in `AdminUser`. Integration tests mirror production by building the full router plus auth/session middleware, then asserting public HTTP behavior.

Session invalidation is intentionally tied to identity state through `session_auth_hash()`. Because that hash is derived from `email`, a user email update automatically invalidates older sessions without manual session-table scrubbing. Expiry is represented in `tower_sessions.session.expiry_date`; cleanup is a separate background concern and not required for request-time rejection.

One historical mismatch is worth noting: the older auth module ticket expected unauthenticated `/dashboard` to redirect to `/`, but the live implementation and current debt ticket standardize on `401 Unauthorized` instead (`thoughts/tickets/auth-module.md:28-29`, `src/modules/auth/handlers.rs:53-64`, `tests/auth_routes.rs:111-116`). Live code is the current source of truth.

## Historical Context (from thoughts/)

- `thoughts/tickets/auth-module.md` - Original auth module ticket established the core routes, `session_auth_hash` based on email, and centralized `AppError::Unauthorized` handling (`thoughts/tickets/auth-module.md:63-68`, `thoughts/tickets/auth-module.md:114-133`, `thoughts/tickets/auth-module.md:193-200`).
- `thoughts/tickets/admin-route-smoke-tests.md` - Earlier integration work established the pattern of building the full app stack in tests and asserting 401/404 distinctions at the HTTP boundary (`thoughts/tickets/admin-route-smoke-tests.md:18-30`, `thoughts/tickets/admin-route-smoke-tests.md:58-74`).
- `thoughts/tickets/feature_cavekit_session_storage_restoration.md` - Session lifecycle ticket explicitly called for PostgreSQL-backed persistence, logout invalidation, expiry rejection, and email-change invalidation (`thoughts/tickets/feature_cavekit_session_storage_restoration.md:19-34`, `thoughts/tickets/feature_cavekit_session_storage_restoration.md:63-67`).
- `thoughts/tickets/feature_cavekit_admin_role_access_control.md` - Admin access-control ticket matches the current extractor contract: 401 for missing auth and 403 for authenticated non-admins (`thoughts/tickets/feature_cavekit_admin_role_access_control.md:19-31`, `thoughts/tickets/feature_cavekit_admin_role_access_control.md:61-67`).
- `thoughts/tickets/session-cleanup.md` - Historical note that expired session deletion is a background maintenance task, separate from request-time auth failure (`thoughts/tickets/session-cleanup.md:17-27`, `thoughts/tickets/session-cleanup.md:44-49`).

## Related Research

- No prior documents were found under `thoughts/research/` at the time of this research.

## Open Questions

- The ticket references `context/kits/cavekit-auth.md`, but that file is missing from the repository. If needed, the original source spec should be restored or the ticket should be updated to point at an existing canonical auth document.
- The debt ticket asks for new coverage, but the current codebase already contains that coverage. The remaining question is whether the team wants the ticket treated as already satisfied or wants the tests reorganized/documented differently.
