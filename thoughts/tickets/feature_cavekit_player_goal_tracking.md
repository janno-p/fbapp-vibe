---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, polling, players, scoring]
keywords: [goals_scored, top scorers list, player sync, football-data.org, tournament players]
patterns: [periodic sync, write-through refresh, stats replication, idempotent update, leaderboard support]
---

# FEATURE-SCORING-07: Sync player goal counts

## Summary

Keep player goal totals in sync with football-data.org so top scorer detection can rely on local data.

## Acceptance Criteria

- [ ] The polling task fetches the top scorers list when the API provides it.
- [ ] Each player's `goals_scored` field is updated from the API response.
- [ ] Re-fetching the same list does not lose data or create duplicates.
- [ ] The cycle logs how many players were updated at debug level.
- [ ] Goal counts are refreshed during every polling cycle, not only at tournament end.

## Implementation Context

### Relevant files

- `src/polling/db.rs` - player goal update queries.
- `src/football_api/mod.rs` - scorers response types.
- `src/modules/predictions/models.rs` - player-facing models that may render goals.
- `src/modules/standings/db.rs` - consumers of player stats.

### Tests

- Integration test for updating player goal counts from a scorers payload.
- Idempotency test for repeated goal syncs.

### Implementation notes

- This sync happens on every cycle so top scorer data stays fresh.
- If the API does not expose scorers for a competition, the ticket should record the fallback behavior explicitly.

## Research Context

### Keywords to Search

- `goals_scored` - stored player stat.
- `top scorers list` - upstream response.
- `player sync` - update path.
- `idempotent update` - repeat-safe refresh.

### Patterns to Investigate

- periodic sync - update data during every polling pass.
- write-through refresh - local cache mirrors upstream stats.
- leaderboard support - stats that drive scoring later.

### Key Decisions Made

- Goal totals are refreshed every polling cycle.
- Local player stats are treated as syncable tournament data.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers goal count updates and repeat syncs.
- [ ] Updated stats are visible in the local database after polling.

### Manual Verification

- [ ] A player's goal total changes after an upstream update.
- [ ] Re-running the sync leaves the totals stable.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R7.
- Depends on the background polling loop and football API integration.

## Notes

- This ticket is about syncing stats, not scoring predictions directly.
