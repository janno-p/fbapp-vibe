---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, invites, security]
keywords: [invite token, random token, league invite token, URL-safe token, creator-only visibility]
patterns: [opaque token generation, persistent field storage, access-controlled display]
---

# FEATURE-LEAGUES-02: Invite token generation

## Description
Generate a persistent invite token for each league so members can share a join link without exposing internal identifiers.

## Context
This token is the shared entry point for league membership and must remain stable over the league's lifetime.

## Requirements
- Each league has a generated invite token.
- The token is random and at least 20 characters long.
- The token is persistent and does not regenerate.
- The token is URL-safe and alphanumeric.
- The token is visible only to league creator/admin users.
- `GET /leagues/{id}` shows the token only to eligible users.

### Functional Requirements
- Store a unique invite token with the league record.
- Expose the token only to authorized viewers.

### Non-Functional Requirements
- Tokens should be opaque and hard to guess.
- Token generation must not break existing league records once assigned.

## Current State
The source spec defines invite tokens as a distinct requirement, but not as a separate ticket yet.

## Desired State
Every league has one stable invite token, and only eligible users can see it.

## Research Context

### Keywords to Search
- `invite token` - league sharing mechanism
- random string - token generation requirement
- URL-safe - token format constraint
- creator/admin users - visibility rule
- `GET /leagues/{id}` - overview page display path

### Patterns to Investigate
- opaque token generation - secure random token strategy
- persistent field storage - token stored on league record
- access-controlled display - hiding token from non-eligible users

### Key Decisions Made
- The token is stored on the league record.
- The token is permanent for the lifetime of the league.
- Only creator/admin users can see it.

## Success Criteria
The ticket is complete when each league has a stable, private invite token.

### Automated Verification
- [ ] Test confirms token is generated for new leagues.
- [ ] Test confirms the token does not regenerate on subsequent reads.
- [ ] Test confirms non-eligible users cannot see the token.

### Manual Verification
- [ ] League overview shows the token to creator/admin users.
- [ ] League overview hides the token from regular members and non-members.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R2`

## Notes
Do not add token expiration or rotation in this ticket.
