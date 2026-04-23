---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, membership, routing]
keywords: [league join, invite token, league_members, GET /leagues/join/{token}, idempotent join]
patterns: [token-based membership join, idempotent insert, redirect after join, not found handling]
---

# FEATURE-LEAGUES-03: Token-based league joining

## Description
Allow authenticated users to join a league by visiting a shareable invite link.

## Context
This is the user-facing entry point for membership and depends on a valid invite token.

## Requirements
- Invite link uses `GET /leagues/join/{token}`.
- A valid token adds the current user to the league.
- Membership is represented by a row in `league_members(league_id, user_id)`.
- Joining is idempotent and does not create duplicate memberships.
- Successful join redirects to the league overview page.
- Invalid or expired tokens return `404 Not Found` or an equivalent not-found error.
- Any authenticated user can join via token.

### Functional Requirements
- Resolve the league from the token.
- Insert membership once, even if the join URL is visited repeatedly.

### Non-Functional Requirements
- Join behavior must be safe against duplicate requests.
- Unknown tokens should fail cleanly without leaking extra data.

## Current State
The source spec calls for token-based joining, but the ticket does not yet exist separately.

## Desired State
Users can join leagues through a stable invite URL and land on the league page afterward.

## Research Context

### Keywords to Search
- `GET /leagues/join/{token}` - join route
- `league_members` - membership table
- idempotent join - duplicate request behavior
- `404 Not Found` - invalid token response
- authenticated user - access assumption

### Patterns to Investigate
- token-based membership join - routing and DB lookup flow
- idempotent insert - upsert or ignore-on-conflict pattern
- redirect after join - post-join user flow
- not found handling - invalid token response path

### Key Decisions Made
- Only authenticated users can join.
- Joining by the same token more than once is a no-op.
- Unknown tokens map to a not-found response.

## Success Criteria
The ticket is complete when a valid invite token reliably joins the current user to the league.

### Automated Verification
- [ ] Test covers successful join with a valid token.
- [ ] Test covers duplicate join requests without duplicate rows.
- [ ] Test covers invalid token returning not-found.

### Manual Verification
- [ ] Visiting a valid invite link joins the league.
- [ ] Reusing the same link does not create duplicate membership rows.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R3`

## Notes
Do not add membership invites by email or token expiration here.
