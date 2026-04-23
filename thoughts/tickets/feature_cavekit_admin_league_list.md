---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, admin, listing]
keywords: [admin league list, member count, creation date, admin dashboard, list leagues]
patterns: [admin-only listing, summary table rendering, pagination, clickable rows]
---

# FEATURE-LEAGUES-06: Admin league list

## Description
Show admins a league listing on the admin dashboard so they can review league inventory and basic metadata.

## Context
This is the admin-facing visibility layer for league management and complements league creation.

## Requirements
- `GET /admin/leagues` lists all leagues.
- The route is admin-only.
- The list shows league name, member count, and creation date.
- The list is paginated or otherwise kept reasonably short.
- Admins can click through to view or edit a league.

### Functional Requirements
- Provide an admin inventory view for leagues.
- Surface basic operational metadata without exposing private membership details.

### Non-Functional Requirements
- Keep the list usable as league count grows.
- Enforce admin access consistently.

## Current State
The source spec includes admin league listing as a distinct requirement, but it is not yet ticketed separately.

## Desired State
Admins can open `/admin/leagues` and inspect all leagues with useful summary data.

## Research Context

### Keywords to Search
- `GET /admin/leagues` - admin list route
- member count - summary metric
- creation date - displayed metadata
- pagination - list size constraint
- admin dashboard - route location

### Patterns to Investigate
- admin-only listing - route access gate
- summary table rendering - list UI pattern
- pagination - performance and usability strategy
- clickable rows - navigation to view/edit flow

### Key Decisions Made
- The view is admin-only.
- Member count and creation date are the required summary fields.
- The list should stay short or paginated.

## Success Criteria
The ticket is complete when admins can review all leagues from the admin dashboard.

### Automated Verification
- [ ] Test covers admin-only access to the list page.
- [ ] Test covers the displayed league summary fields.
- [ ] Test covers list pagination or bounded rendering.

### Manual Verification
- [ ] Admin can view all leagues from `/admin/leagues`.
- [ ] Each row shows the expected summary data.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R6`

## Notes
Do not add delete actions, bulk edits, or admin transfer behavior here.
