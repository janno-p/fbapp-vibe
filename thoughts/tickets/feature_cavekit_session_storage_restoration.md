---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
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
- Sessions are stored in PostgreSQL `tower_sessions`.
- Session auth hash is derived from user email.
- `AuthSession` is available in handlers.
- Expired sessions return `401 Unauthorized`.
- `POST `/auth/logout` destroys the session and redirects to homepage.

### Functional Requirements
- Persist sessions across requests.
- Restore the authenticated user from session state.
- Invalidate sessions when logout occurs or they expire.

### Non-Functional Requirements
- Use the existing PostgreSQL session backend.
- Handle email changes as a session invalidation signal.

## Current State
The source spec defines the required behavior; this ticket isolates the session lifecycle work.

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
- [ ] Integration test covers session restoration on subsequent requests.
- [ ] Integration test covers logout destroying the session.
- [ ] Integration test covers expired-session rejection.

### Manual Verification
- [ ] User remains authenticated across page loads.
- [ ] Logout removes access immediately.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R3`

## Notes
This ticket should not include session management UI.
