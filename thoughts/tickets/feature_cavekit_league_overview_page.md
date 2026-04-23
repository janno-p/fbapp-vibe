---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, ui, access-control]
keywords: [league overview, member list, avatars, league name, members-only page]
patterns: [members-only route, access-gated page rendering, sorted member list]
---

# FEATURE-LEAGUES-04: League overview page

## Description
Render a members-only league overview page that shows league details and the current membership roster.

## Context
This is the main page users see after joining or opening a league they already belong to.

## Requirements
- `GET /leagues/{id}` renders a league overview page.
- The page is members-only.
- The page shows the league name.
- The page shows the member list with user names and avatars.
- The invite token is displayed only to creator/admin users.
- Non-members see a clear "You are not a member of this league" error.
- The member list is sorted consistently.

### Functional Requirements
- Gate the page on league membership.
- Show enough metadata for members to understand the league roster.

### Non-Functional Requirements
- Keep access control strict for non-members.
- Keep member ordering deterministic.

## Current State
The source spec defines a members-only overview page, but it is not yet a separate ticket.

## Desired State
League members can view a roster page with the league name, member list, and eligible invite token visibility.

## Research Context

### Keywords to Search
- `GET /leagues/{id}` - overview route
- league overview - page scope
- member list - roster rendering
- avatars - user display requirement
- members-only page - access control boundary

### Patterns to Investigate
- members-only route - membership gate before rendering
- access-gated page rendering - unauthorized user path
- sorted member list - stable ordering strategy

### Key Decisions Made
- Only league members can view the page.
- The invite token remains hidden from non-eligible users.
- The member list must sort consistently.

## Success Criteria
The ticket is complete when league members can open a roster page and non-members cannot.

### Automated Verification
- [ ] Test covers successful rendering for a league member.
- [ ] Test covers denial for a non-member.
- [ ] Test covers consistent member ordering.

### Manual Verification
- [ ] Member can open `/leagues/{id}` and see roster details.
- [ ] Non-member sees the membership error instead of the page.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R4`

## Notes
Do not expand this into league chat, messaging, or broader social features.
