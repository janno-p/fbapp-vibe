---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, leagues, database, access-control]
keywords: [league_members, joined_at, unique constraint, membership tracking, standings isolation]
patterns: [join table modeling, unique composite constraint, access-control query, league-scoped filtering]
---

# FEATURE-LEAGUES-05: Membership tracking

## Description
Persist league membership in the database and use it as the source of truth for access control and league-scoped views.

## Context
Membership rows are the foundation for join behavior, overview access, and league-specific standings.

## Requirements
- `league_members` exists with `id`, `league_id`, `user_id`, and `joined_at`.
- `(league_id, user_id)` has a unique constraint.
- Joining a league inserts a row into `league_members`.
- League overview and standings pages query `league_members` for access control.
- Leaderboard and standings pages show points only for users in that league.

### Functional Requirements
- Persist membership once per user per league.
- Use membership rows to decide who can view league data.

### Non-Functional Requirements
- Prevent duplicate memberships at the database level.
- Keep access-control queries cheap and predictable.

## Current State
The source spec names membership tracking as its own concern, but the ticket is not yet separated.

## Desired State
`league_members` is the authoritative membership store and all league-scoped views rely on it.

## Research Context

### Keywords to Search
- `league_members` - membership table name
- `joined_at` - persisted membership timestamp
- unique constraint - duplicate protection
- standings pages - consumer of membership data
- access control - membership-based authorization

### Patterns to Investigate
- join table modeling - membership persistence design
- unique composite constraint - one row per user per league
- league-scoped filtering - restrict scoreboard queries
- access-control query - membership check before render

### Key Decisions Made
- Membership is stored as a join table.
- Duplicate membership rows are not allowed.
- Membership data drives both access control and league-scoped standings.

## Success Criteria
The ticket is complete when membership data is persisted correctly and used for access checks.

### Automated Verification
- [ ] Test confirms the membership row is inserted on join.
- [ ] Test confirms duplicate memberships are rejected or ignored.
- [ ] Test confirms access checks rely on membership rows.

### Manual Verification
- [ ] A joined user appears in `league_members`.
- [ ] League-scoped pages only include members of that league.

## Related Information
- Source doc: `context/kits/cavekit-leagues.md`
- Requirement: `R5`

## Notes
Do not add member removal, banning, or role-based membership here.
