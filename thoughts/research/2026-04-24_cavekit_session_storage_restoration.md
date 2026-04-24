---
date: 2026-04-24T15:10:45+03:00
git_commit: 60b28a3a964968a6cc96979f0944302686811fa7
branch: main
repository: fbapp-vibe
topic: "FEATURE-003: Session storage and restoration"
tags: [research, codebase, auth, sessions, postgres, tower-sessions]
last_updated: 2026-04-24
---

## Ticket Synopsis

The ticket requires database-backed auth sessions that persist across requests, restore via `AuthSession`, reject expired sessions with `401`, invalidate on logout, and invalidate when identity state changes (email-backed session auth hash). It also requires `POST /auth/logout` to destroy session state and redirect home.

## Summary

The required behavior is implemented in live code and covered by integration tests. Session storage is configured with PostgreSQL `tower_sessions.session` via `PostgresStore` + `SessionManagerLayer`, restoration occurs through `axum-login` and `AuthBackend::get_user`, and ticket-critical invalidation paths (logout, expiry, email change) are covered in `tests/auth_routes.rs`.

The ticket metadata/checklist has been normalized for close-out: status/checklists now align with implemented behavior and evidence-backed outcome notes.

## Detailed Findings

### Session Persistence and Restoration Pipeline

- PostgreSQL-backed sessions are configured at app startup (`src/main.rs:37`, `src/main.rs:38`, `src/main.rs:47`, `src/main.rs:67`).
- The canonical session table is `tower_sessions.session` (`migrations/0004_fix_sessions.sql:9`), correcting an earlier schema mismatch (`migrations/0004_fix_sessions.sql:5`).
- Session expiry policy is inactivity-based and config-driven (`src/main.rs:41`, `src/config.rs:29`, `src/config.rs:42`).
- Request-time restoration is performed through `AuthSession` + backend `get_user` DB reload (`src/modules/auth/mod.rs:15`, `src/modules/auth/mod.rs:42`).
- Handlers consume restored auth via `auth_session.user` and return unauthorized when absent (`src/modules/auth/handlers.rs:58`).

### Logout and Invalidation Behavior

- Logout endpoint is registered as `POST /auth/logout` (`src/modules/auth/mod.rs:62`).
- Logout destroys session via `auth_session.logout()` then redirects to `/` (`src/modules/auth/handlers.rs:170`, `src/modules/auth/handlers.rs:172`, `src/modules/auth/handlers.rs:175`).
- Session auth hash is derived from email (`src/modules/auth/models.rs:21`), making email changes an explicit invalidation signal.
- Unit tests confirm hash changes with email changes (`src/modules/auth/models.rs:97`).
- Integration test confirms email mutation invalidates current session (`tests/auth_routes.rs:500`, `tests/auth_routes.rs:515`).

### Expiry Handling and 401 Semantics

- Expired sessions are rejected on protected routes (`tests/auth_routes.rs:484`, `tests/auth_routes.rs:496`).
- Background cleanup removes expired rows (`src/session_cleanup.rs:20`) and is started during app startup (`src/main.rs:66`).
- `AppError::Unauthorized` maps to HTTP 401 with the 401 template (`src/error.rs:44`, `src/error.rs:22`).
- This creates a clear split: auth failures are status-based responses, while navigation/login flows use redirects.

### Coverage Against Ticket Acceptance

- Session restoration on subsequent requests: covered by OAuth callback creating session then authenticated dashboard access (`tests/auth_routes.rs:371`, `tests/auth_routes.rs:386`).
- Logout destroys session: covered (`tests/auth_routes.rs:293`, `tests/auth_routes.rs:302`).
- Expired-session rejection with 401: covered (`tests/auth_routes.rs:484`, `tests/auth_routes.rs:496`).
- Session invalidation on email change: covered (`tests/auth_routes.rs:500`, `tests/auth_routes.rs:515`).

## Code References

- `src/main.rs:37` - Constructs PostgreSQL session store.
- `src/main.rs:41` - Configures inactivity expiry.
- `src/main.rs:47` - Builds auth layer with session layer.
- `src/main.rs:67` - Applies auth/session middleware to router.
- `src/modules/auth/mod.rs:42` - `get_user` reloads user from DB for restoration.
- `src/modules/auth/mod.rs:62` - Registers `POST /auth/logout`.
- `src/modules/auth/handlers.rs:155` - Creates authenticated session (`login`).
- `src/modules/auth/handlers.rs:172` - Destroys authenticated session (`logout`).
- `src/modules/auth/models.rs:21` - Email-backed `session_auth_hash`.
- `src/session_cleanup.rs:20` - Deletes expired session rows.
- `migrations/0004_fix_sessions.sql:9` - Canonical `tower_sessions.session` table.
- `tests/auth_routes.rs:293` - Logout invalidation integration test.
- `tests/auth_routes.rs:484` - Expired session rejection integration test.
- `tests/auth_routes.rs:500` - Email-change invalidation integration test.

## Architecture Insights

Auth uses framework-native composition instead of custom session glue: `tower-sessions` handles persistence/expiry transport concerns, while `axum-login` handles user restoration and auth-state continuity through `AuthSession`. Authorization is then expressed as extractor/handler-level checks over `auth_session.user`.

Session invalidation is identity-coupled (`session_auth_hash` from email), so identity mutation immediately revokes prior sessions without bespoke revocation logic. Expired-row cleanup is intentionally operational hygiene, not the primary enforcement path for 401 on expired sessions.

## Historical Context (from thoughts/)

- `thoughts/tickets/feature_cavekit_session_storage_restoration.md` - Defines the exact lifecycle requirements covered here and now includes requirement-to-evidence mapping plus close-out outcome notes (`thoughts/tickets/feature_cavekit_session_storage_restoration.md:19`, `thoughts/tickets/feature_cavekit_session_storage_restoration.md:66`).
- `thoughts/tickets/debt_cavekit_auth_integration_tests.md` - Documents that required HTTP-level regression coverage exists (`thoughts/tickets/debt_cavekit_auth_integration_tests.md:64`, `thoughts/tickets/debt_cavekit_auth_integration_tests.md:69`, `thoughts/tickets/debt_cavekit_auth_integration_tests.md:83`).
- `thoughts/research/2026-04-23_auth_integration_tests.md` - Prior research confirms runtime behavior and test coverage (`thoughts/research/2026-04-23_auth_integration_tests.md:17`, `thoughts/research/2026-04-23_auth_integration_tests.md:29`, `thoughts/research/2026-04-23_auth_integration_tests.md:30`).
- `thoughts/research/2026-04-23_google_oauth_login_flow.md` - Confirms same persistence/restoration stack and invalidation model (`thoughts/research/2026-04-23_google_oauth_login_flow.md:41`, `thoughts/research/2026-04-23_google_oauth_login_flow.md:44`).
- `thoughts/tickets/feature_cavekit_session_cleanup.md` and `thoughts/tickets/session-cleanup.md` - Show parallel/stale documentation for cleanup work despite implemented code (`thoughts/tickets/feature_cavekit_session_cleanup.md:5`, `thoughts/tickets/session-cleanup.md:53`).
- Source spec path in this ticket is explicitly marked as missing from repo (`thoughts/tickets/feature_cavekit_session_storage_restoration.md:89`).

## Related Research

- `thoughts/research/2026-04-23_auth_integration_tests.md`
- `thoughts/research/2026-04-23_google_oauth_login_flow.md`
- `thoughts/research/2026-04-24_cavekit_user_model.md`

## Open Questions

- Should auth/session tickets with implemented behavior be bulk-normalized (status/checklists) to prevent ongoing planning drift?
- Should the missing source spec links (`context/kits/cavekit-auth.md`) be replaced with an in-repo canonical auth/session spec?
