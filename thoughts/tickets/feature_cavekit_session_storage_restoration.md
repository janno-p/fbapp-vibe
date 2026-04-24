---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: implemented
tags: [auth, sessions, postgres]
keywords: [tower_sessions, AuthSession, expiry_date, logout, session invalidation, email change]
patterns: [session persistence, session restoration, logout invalidation]
---

# FEATURE-003: Session storage and restoration

## Description
Ensure authenticated sessions persist in PostgreSQL and are restored on subsequent requests through the auth session extractor.

## Context
This ticket covers the core session lifecycle: persistence, restoration, expiration handling, and logout destruction.

## Requirements
- Sessions are stored in PostgreSQL `tower_sessions.session`.
- Session auth hash is derived from user email.
- `AuthSession` is available in handlers.
- Expired sessions return `401 Unauthorized`.
- `POST /auth/logout` destroys the session and redirects to homepage.

### Requirement Evidence
- Session restoration across requests is covered by OAuth callback + subsequent protected-route access in `tests/auth_routes.rs:371` and `tests/auth_routes.rs:386`, with backend restoration via `src/modules/auth/mod.rs:42`.
- Logout invalidation is covered in `tests/auth_routes.rs:293` and `tests/auth_routes.rs:302`, implemented in `src/modules/auth/handlers.rs:170`.
- Expired-session rejection is covered in `tests/auth_routes.rs:484` and `tests/auth_routes.rs:496`, with unauthorized mapping in `src/error.rs:44`.
- Email-change invalidation is covered in `tests/auth_routes.rs:500` and `tests/auth_routes.rs:515`, driven by the email-backed session hash in `src/modules/auth/models.rs:21`.

### Functional Requirements
- Persist sessions across requests.
- Restore the authenticated user from session state.
- Invalidate sessions when logout occurs or they expire.

### Non-Functional Requirements
- Use the existing PostgreSQL session backend.
- Handle email changes as a session invalidation signal.

## Current State
Session lifecycle behavior is implemented in runtime code and covered by integration tests; this ticket now captures evidence-backed close-out.

## Desired State
Users remain signed in across requests until logout, expiration, or identity invalidation.

## Research Context

### Keywords to Search
- tower_sessions - session storage backend
- AuthSession - request extractor
- expiry_date - session expiration field
- logout - session destruction endpoint
- session invalidation - stale session handling

### Patterns to Investigate
- session persistence - database-backed auth state
- session restoration - extractor behavior on request
- logout invalidation - clearing stored sessions safely

### Key Decisions Made
- PostgreSQL is the session store.
- Email changes must invalidate the auth session hash.
- Logout redirects to the homepage.

## Success Criteria
The ticket is complete when sessions survive normal request flow and fail correctly after expiration or logout.

### Automated Verification
- [x] Integration test covers session restoration on subsequent requests (`tests/auth_routes.rs:371`, `tests/auth_routes.rs:386`).
- [x] Integration test covers logout destroying the session (`tests/auth_routes.rs:293`, `tests/auth_routes.rs:302`).
- [x] Integration test covers expired-session rejection (`tests/auth_routes.rs:484`, `tests/auth_routes.rs:496`).
- [x] Integration test covers email-change invalidation (`tests/auth_routes.rs:500`, `tests/auth_routes.rs:515`).

### Manual Verification
- [x] User remains authenticated across page loads.
- [x] Logout removes access immediately.

## Outcome

FEATURE-003 lifecycle behavior is implemented and covered by integration tests.
- Runtime stack: `src/main.rs:37`, `src/main.rs:47`, `src/main.rs:67`
- Logout path: `src/modules/auth/handlers.rs:170`
- Unauthorized mapping: `src/error.rs:44`
- Regression coverage: `tests/auth_routes.rs:293`, `tests/auth_routes.rs:371`, `tests/auth_routes.rs:484`, `tests/auth_routes.rs:500`

## Related Information
- Source doc: `context/kits/cavekit-auth.md` (missing in repository as of 2026-04-24)
- Requirement: `R3`

## Notes
This ticket should not include session management UI.
