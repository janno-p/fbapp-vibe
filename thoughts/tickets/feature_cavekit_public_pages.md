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
- Unauthenticated users visiting `/dashboard` are redirected to `/auth/login`.

### Functional Requirements
- Provide at least one public landing page.
- Route users to login when they attempt to view protected content without a session.

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
- Redirect destination for unauthenticated dashboard access is `/auth/login`.

## Success Criteria
The ticket is complete when routing follows the public/protected split.

### Automated Verification
- [ ] Integration test covers public access to `/`.
- [ ] Integration test covers redirect from `/dashboard` to `/auth/login` for unauthenticated users.

### Manual Verification
- [ ] Home page loads without signing in.
- [ ] Dashboard access prompts login when signed out.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R6`

## Notes
Do not expand this into broader public-site work.
