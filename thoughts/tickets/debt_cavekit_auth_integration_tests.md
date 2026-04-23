---
type: debt
priority: high
created: 2026-04-23T00:00:00Z
status: done
tags: [auth, tests, integration]
keywords: [integration tests, dashboard unauthorized, logout session destruction, expired session, admin route, email change invalidation]
patterns: [HTTP-level integration testing, regression coverage, session invalidation tests]
---

# DEBT-007: Auth integration test coverage

## Description
Add real HTTP integration coverage for the critical auth flows so session, authorization, and invalidation behavior is verified against the actual stack.

## Context
The source spec explicitly calls out gaps where unit tests re-implement extractor logic or miss important auth regressions.

## Requirements
- GET `/dashboard` returns `401 Unauthorized` for unauthenticated requests.
- GET `/` redirects authenticated users to `/dashboard`.
- POST `/auth/logout` destroys the session and future requests with that session token return `401`.
- AdminUser extractor rejects non-admin users at the HTTP level.
- Session is invalidated when the user email changes.
- Expired sessions return `401`.

### Functional Requirements
- Cover core auth regressions with integration tests.
- Exercise the real HTTP stack rather than unit tests duplicating extractor logic.

### Non-Functional Requirements
- Keep tests faithful to production behavior.
- Avoid brittle tests that mirror implementation details instead of public behavior.

## Current State
The source doc notes this as newly discovered missing coverage.

## Desired State
Critical auth behavior is locked down by integration tests that fail on real regressions.

## Research Context

### Keywords to Search
- integration tests - required test layer
- dashboard unauthorized - expected response
- logout session destruction - session invalidation path
- expired session - database row manipulation
- email change invalidation - auth hash behavior

### Patterns to Investigate
- HTTP-level integration testing - real stack verification
- regression coverage - auth behavior safeguards
- session invalidation tests - logout and expiry cases

### Key Decisions Made
- Test the HTTP boundary, not extractor internals.
- Cover logout, expiry, admin rejection, and email-change invalidation.
- Existing unit tests that duplicate extractor logic should not be the primary coverage.

## Success Criteria
The ticket is complete when the listed auth regressions are covered by integration tests.

### Automated Verification
- [ ] Integration test for unauthenticated `/dashboard` access.
- [ ] Integration test for authenticated `/` redirecting to `/dashboard`.
- [ ] Integration test for logout destroying the session.
- [ ] Integration test for admin rejection at the HTTP level.
- [ ] Integration test for email-change invalidation.
- [ ] Integration test for expired-session rejection.

### Manual Verification
- [ ] Confirm tests fail if auth behavior regresses.
- [ ] Confirm tests exercise the actual HTTP stack.

## Related Information
- Source doc: missing in repo (`context/kits/cavekit-auth.md`)
- Requirement: `R7`

## Notes
This is a test-coverage ticket, not a feature change ticket.

## Outcome
The requested HTTP-level auth regression coverage already exists in `tests/auth_routes.rs`.

- `tests/auth_routes.rs:111-116` covers unauthenticated `/dashboard` returning `401`.
- `tests/auth_routes.rs:118-130` covers logout destroying the session.
- `tests/auth_routes.rs:132-144` covers authenticated `/` redirecting to `/dashboard`.
- `tests/auth_routes.rs:146-170` covers non-admin rejection and admin access on `/admin`.
- `tests/auth_routes.rs:172-187` covers expired-session rejection.
- `tests/auth_routes.rs:189-206` covers email-change invalidation.

The tests use the real HTTP stack via `SessionManagerLayer` and `AuthManagerLayer` (`tests/auth_routes.rs:49-89`), so this debt item is satisfied by the current codebase without additional implementation work.
