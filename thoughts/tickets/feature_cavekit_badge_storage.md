---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [database, achievements, badges]
keywords: [user_achievements, tournament_id, badge_slug, awarded_at, unique constraint, badge query]
patterns: [relational persistence, unique composite constraint, query-by-user-and-tournament]
---

# FEATURE-037: Persist awarded badges

## Description
Add durable storage for earned badges so the application can query a user’s achievements efficiently within a tournament.

## Context
The award job needs a database-backed record of earned badges, and the UI needs a stable way to query them later.

## Requirements
- Create a `user_achievements` table.
- The table must include `id`, `user_id`, `tournament_id`, `badge_slug`, and `awarded_at`.
- Add a unique constraint on `(user_id, tournament_id, badge_slug)`.
- Allow the same badge to be awarded in different tournaments.
- Allow a user to earn multiple badges in one tournament.
- Support efficient queries for badges by user and by badge type.

### Functional Requirements
- Persist every earned badge once per user per tournament.
- Support reads for a user’s badges in a tournament.
- Support reads for all users who earned a given badge in a tournament.

### Non-Functional Requirements
- Enforce uniqueness at the database level.
- Keep query patterns simple and index-friendly.

## Current State
There is no dedicated achievement storage table.

## Desired State
Earned badges are stored durably and can be queried without duplication or ambiguity.

## Research Context

### Keywords to Search
- `user_achievements` - target table name
- `badge_slug` - stored badge identifier
- `awarded_at` - timestamp field
- unique constraint - duplicate prevention
- tournament_id - tournament scoping key

### Patterns to Investigate
- relational persistence - table shape and keys
- unique composite constraint - award de-duplication strategy
- query-by-user-and-tournament - member badge lookup

### Key Decisions Made
- A badge is earned once per user per tournament.
- The database enforces uniqueness rather than application-only checks.

## Success Criteria
The ticket is complete when badge records can be inserted and queried with the required uniqueness guarantees.

### Automated Verification
- [ ] Migration test or schema check confirms the table and unique constraint exist.
- [ ] Query test confirms user/tournament badge lookup works.

### Manual Verification
- [ ] Duplicate insert attempts are blocked.
- [ ] Badge rows can be retrieved by user and by badge slug.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R2`
- Depends on: badge definitions ticket.

## Notes
Do not add badge rarity, expiration, or transfer mechanics.
