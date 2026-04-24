---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: reviewed
tags: [auth, google-oauth, sessions]
keywords: [google oauth, auth/login, auth/callback, access token, user info, tower_sessions]
patterns: [oauth authorization code flow, session creation after login, protected route authorization]
---

# FEATURE-001: Google OAuth login flow

## Description
Implement the Google OAuth login flow so users can authenticate, have their account information synchronized, and be redirected into the app with a valid session.

## Context
This is the foundational auth capability for Cavekit. Other modules depend on a successful OAuth login, persisted user identity, and an authenticated session.

## Requirements
- GET `/auth/login` redirects to Google OAuth authorization endpoint.
- GET `/auth/callback` accepts an authorization code and exchanges it for tokens.
- Callback fetches Google user info and stores or updates the user record.
- A PostgreSQL-backed session is created after successful login.
- Successful login redirects to a safe stored continuation target when present, otherwise `/dashboard`.
- Unauthenticated access to protected routes returns `401 Unauthorized`.

### Functional Requirements
- Support a complete Google OAuth authorization-code login flow.
- Persist Google ID, email, name, and avatar URL on login.
- Create a usable app session immediately after authentication.

### Non-Functional Requirements
- Use the existing session infrastructure and PostgreSQL storage.
- Do not introduce other social providers or password auth.

## Current State
The source spec says this flow already exists, but it needs to be represented as its own planning ticket.

## Desired State
A complete Google OAuth login implementation with redirect, callback, user sync, and session creation.

## Research Context

### Keywords to Search
- google oauth - primary provider and flow
- auth/login - login route behavior
- auth/callback - callback route behavior
- tower_sessions - session persistence layer
- access token - token exchange step

### Patterns to Investigate
- oauth authorization code flow - route and token exchange handling
- session creation after login - how auth session is established
- protected route authorization - how unauthenticated requests are blocked

### Key Decisions Made
- Google is the only OAuth provider in scope.
- Session state should be stored in PostgreSQL.
- Post-login redirect target is a safe stored continuation target when present, otherwise `/dashboard`.

## Success Criteria
The ticket is complete when the login flow works end-to-end and protected routes reject unauthenticated access.

### Automated Verification
- [ ] Integration test covers `/auth/login` redirect behavior.
- [ ] Integration test covers `/auth/callback` and session creation.
- [ ] Integration test confirms protected routes return `401` when unauthenticated.

### Manual Verification
- [ ] Login completes with a Google account.
- [ ] User lands on the safe continuation target when one exists, otherwise `/dashboard`.
- [ ] Session persists across requests.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R1`

## Notes
Keep this ticket limited to Google OAuth. Do not expand into account linking or other providers.
