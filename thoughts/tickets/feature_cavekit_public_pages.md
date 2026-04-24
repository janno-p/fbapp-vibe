---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [auth, public-pages, routing]
keywords: [home page, dashboard redirect, unauthenticated, login link, public route]
patterns: [public route handling, auth-aware redirects, protected dashboard routing]
---

# FEATURE-006: Public pages and auth-aware routing

## Description
Allow unauthenticated users to access the home page while keeping the dashboard protected and redirecting users appropriately based on auth state.

## Context
This defines the basic public-facing routing behavior around the auth system.

## Requirements
- GET `/` renders the home page without authentication.
- Home page displays a login link.
- GET `/dashboard` is protected.
- Unauthenticated users visiting `/dashboard` receive `401 Unauthorized`.

### Functional Requirements
- Provide at least one public landing page.
- Reject unauthenticated access to protected content with `401` at the HTTP layer.

### Non-Functional Requirements
- Keep routing behavior consistent with the auth session layer.
- Avoid exposing dashboard content to unauthenticated users.

## Current State
The source spec calls out public pages as a separate requirement.

## Desired State
The app has a public home page and a protected dashboard with the right redirects.

## Research Context

### Keywords to Search
- home page - public landing route
- dashboard redirect - auth-aware routing
- unauthenticated - redirect condition
- login link - home page action
- public route - route visibility model

### Patterns to Investigate
- public route handling - unauthenticated access
- auth-aware redirects - route behavior by session state
- protected dashboard routing - authenticated-only page guard

### Key Decisions Made
- Home is public.
- Dashboard is protected.
- Unauthenticated dashboard access returns `401 Unauthorized`.

## Success Criteria
The ticket is complete when routing follows the public/protected split.

### Automated Verification
- [ ] Integration test covers public access to `/`.
- [ ] Integration test covers `401 Unauthorized` from `/dashboard` for unauthenticated users.

### Manual Verification
- [ ] Home page loads without signing in.
- [ ] Dashboard access returns `401 Unauthorized` when signed out.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R6`

## Notes
Do not expand this into broader public-site work.

Canonical behavior references:
- Integration test: `tests/auth_routes.rs:286`
- Runtime guard and status mapping: `src/modules/auth/handlers.rs:58`, `src/error.rs:44`
