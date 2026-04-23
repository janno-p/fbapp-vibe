---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [auth, admin, access-control]
keywords: [is_admin, AdminUser, forbidden, admin routes, tournament management, league management]
patterns: [role-based authorization, custom extractor, HTTP 403 gating]
---

# FEATURE-004: Admin role access control

## Description
Add binary admin authorization so only users with `is_admin = true` can reach admin-only routes and management actions.

## Context
Admin access is a cross-cutting auth concern used by tournament and league management flows.

## Requirements
- `AdminUser` extractor returns `403 Forbidden` for non-admin users.
- `AdminUser` can be used in handlers to gate admin-only routes.
- Admin routes use the extractor.
- Regular users receive `403 Forbidden` when accessing admin routes.

### Functional Requirements
- Gate admin pages and management handlers with a reusable authorization check.
- Distinguish authenticated-but-unauthorized users from unauthenticated users.

### Non-Functional Requirements
- Keep authorization logic centralized in the extractor.
- Do not add fine-grained permission systems beyond admin/user split.

## Current State
The source spec already defines the admin gate behavior as a separate concern.

## Desired State
A reusable admin authorization mechanism that blocks non-admin users consistently.

## Research Context

### Keywords to Search
- is_admin - role flag
- AdminUser - extractor name
- forbidden - expected HTTP response
- admin routes - protected endpoints
- tournament management - likely consumer

### Patterns to Investigate
- role-based authorization - admin gating approach
- custom extractor - handler-level access control
- HTTP 403 gating - unauthorized-but-authenticated response path

### Key Decisions Made
- Admin access is binary, not permission-based.
- The extractor is the enforcement point.
- Tournament and league management are the primary consumers.

## Success Criteria
The ticket is complete when non-admin users are consistently blocked with 403 responses.

### Automated Verification
- [ ] Integration test verifies `AdminUser` rejects non-admin users.
- [ ] Integration test verifies admin route access succeeds for admin users.

### Manual Verification
- [ ] Regular user receives `403` on admin routes.
- [ ] Admin user can access admin routes.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R4`

## Notes
Do not broaden this into more granular roles or permissions.
