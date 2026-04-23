---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, tournament, seeding, football-data-org]
keywords: [teams, groups, matches, players, group_memberships, upsert, rate limiter]
patterns: [idempotent sync, foreign-key ordered inserts, external-id upsert, rate-limited API sync]
---

# FEATURE-CAVEKIT-TOURNAMENT-02: Seed tournament data from football-data.org

## Summary

Fetch and persist the tournament structure from football-data.org so the local database has teams, groups, matches, players, and memberships ready for downstream features.

## Acceptance Criteria

- [ ] Seeding fetches all teams for the competition.
- [ ] Seeding fetches all matches for the competition.
- [ ] `teams` rows store `id`, `tournament_id`, `external_id`, `name`, `code`, `tla`, and nullable `flag`.
- [ ] `groups` rows store `id`, `tournament_id`, and `name`.
- [ ] `group_memberships` rows are created for group-to-team relationships.
- [ ] `matches` rows store `home_team_id`, `away_team_id`, `group_id`, `stage`, `scheduled_utc`, scores, and outcome.
- [ ] `players` rows store `team_id`, `name`, `position`, `number`, and `goals_scored`.
- [ ] Re-running seed does not duplicate records.
- [ ] Seed requests are rate-limited to 7 requests per second.

## Implementation Context

### Relevant files

- `src/modules/admin/db.rs` - seed orchestration and upserts.
- `src/football_api.rs` - football-data.org client and rate limiting.
- `migrations/0005_tournament_core.sql` - target tables.
- `migrations/0015_remove_team_crest_url.sql` - flag-related schema cleanup.
- `migrations/0016_team_flag.sql` - flag column addition.

### ADR constraints

- **ADR-0005**: Use `sqlx::query!` / `query_as!` macros for checked SQL.
- **ADR-0016**: Use `external_id` as the idempotent upsert key.

### Tests

- `#[sqlx::test]` for a seed run that can be repeated without row duplication.
- `#[sqlx::test]` for the expected FK insert order and relationship rows.

### Implementation notes

- Seed order matters because of foreign keys.
- Treat football-data.org as the source of truth for the tournament snapshot.

## Research Context

### Keywords to Search

- `teams` - target table and API source entity.
- `groups` - tournament stage grouping.
- `matches` - fixture ingestion target.
- `players` - roster ingestion target.
- `RateLimiter` - free-tier request throttling.
- `upsert` - idempotent persistence pattern.

### Patterns to Investigate

- idempotent sync - repeatable seed operations.
- foreign-key ordered inserts - parent rows before child rows.
- external-id upsert - stable conflict key for incoming source data.
- rate-limited API sync - protect third-party quota usage.

### Key Decisions Made

- Seeding is synchronous relative to registration.
- Duplicate protection is required on every seed run.
- The free-tier API limit is 7 requests per second.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers seed idempotency.
- [ ] `cargo test` covers the rate-limited API path or equivalent unit wrapper.

### Manual Verification

- [ ] Tournament data appears in the expected tables after seed.
- [ ] Running seed twice leaves the row counts stable.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R2.
- Depends on tournament registration.

## Notes

- Keep manual entry out of scope; the data should come from the API only.
