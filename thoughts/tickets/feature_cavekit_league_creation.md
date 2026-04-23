---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, admin, database]
keywords: [admin leagues, league creation form, unique league name, AdminUser, POST /admin/leagues]
patterns: [admin-only form handling, uniqueness enforcement, post-redirect-get, database insert]
---

# FEATURE-LEAGUES-01: Admin league creation

## Description
Allow admin users to create leagues from the admin area using a unique league name.

## Context
This is the entry point for league setup and underpins the rest of the league membership flow.

## Requirements
- GET `/admin/leagues` shows the existing leagues and the creation form.
- Admin submits a league creation form with a unique `name`.
- POST `/admin/leagues` creates a new league record.
- Newly created leagues include `id`, `name`, and `created_at`.
- Successful creation redirects to `/admin`.
- Duplicate submissions are handled predictably by the schema or application logic.

### Functional Requirements
- Support admin-only league creation.
- Prevent duplicate league names from creating ambiguous records.

### Non-Functional Requirements
- Keep creation behavior idempotent or constraint-safe.
- Enforce access control consistently at the route layer.

## Current State
The source spec defines league creation as a requirement, but it is not yet split into a dedicated ticket.

## Desired State
Admins can create leagues from `/admin/leagues` and the system stores the new league reliably.

## Research Context

### Keywords to Search
- `GET /admin/leagues` - admin listing and form surface
- `POST /admin/leagues` - creation handler route
- `AdminUser` - admin access control extractor
- unique league name - validation and schema constraint
- duplicate league name - idempotency / conflict handling

### Patterns to Investigate
- admin-only form handling - how admin routes are gated
- uniqueness enforcement - database constraint or upsert behavior
- post-redirect-get - redirect after create success
- database insert - league creation query path

### Key Decisions Made
- League creation is admin-only.
- League names must be unique.
- A successful create returns the operator to `/admin`.

## Success Criteria
The ticket is complete when admins can create leagues and duplicate handling is deterministic.

### Automated Verification
- [ ] Integration test covers successful league creation.
- [ ] Integration test covers duplicate-name handling.
- [ ] Integration test confirms non-admin users cannot access the create action.

### Manual Verification
- [ ] Admin can create a league from `/admin/leagues`.
- [ ] Duplicate league names do not create inconsistent state.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R1`

## Notes
Keep this ticket focused on creation only; membership and invite behavior belong in separate tickets.
