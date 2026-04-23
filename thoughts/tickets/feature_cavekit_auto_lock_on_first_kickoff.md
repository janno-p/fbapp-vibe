---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, predictions, tournament, lock]
keywords: [predictions_locked_at, kickoff time, IN_PLAY, manual lock, idempotent auto-lock]
patterns: [server-side state transition, first-event detection, write guard, precedence rule, timestamp preservation]
---

# FEATURE-SCORING-03: Auto-lock predictions on first kickoff

## Summary

Lock predictions automatically when the first match of the active tournament starts so late edits cannot slip in.

## Acceptance Criteria

- [ ] The polling task detects when any match transitions to `IN_PLAY` or `FINISHED`.
- [ ] On the first in-play match, `tournament.predictions_locked_at` is set to `match.scheduled_utc`.
- [ ] The auto-lock happens exactly once per tournament.
- [ ] If `predictions_locked_at` is already set, auto-lock does not override it.
- [ ] The stored lock timestamp uses the kickoff time, not the current time.
- [ ] The auto-lock event is logged at info level.

## Implementation Context

### Relevant files

- `src/polling/db.rs` - lock transition logic.
- `src/modules/standings/db.rs` - tournament state reads.
- `src/modules/admin/db.rs` - manual lock overrides.
- `src/modules/predictions/db.rs` - prediction write-path lock checks.

### Tests

- `#[sqlx::test]` for auto-locking a tournament once.
- `#[sqlx::test]` for preserving an existing manual lock.

### Implementation notes

- The first kickoff timestamp is authoritative for the auto-lock event.
- Manual lock must always win over the automatic path.

## Research Context

### Keywords to Search

- `predictions_locked_at` - lock field.
- `IN_PLAY` - kickoff detection status.
- `scheduled_utc` - timestamp source.
- `manual lock` - precedence behavior.

### Patterns to Investigate

- server-side state transition - lock state changes in the write path.
- first-event detection - only the first kickoff matters.
- timestamp preservation - keep kickoff time stable.

### Key Decisions Made

- Auto-lock is derived from match kickoff time.
- Manual lock takes precedence over automatic lock.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers first-lock and already-locked cases.
- [ ] The lock update is idempotent under repeated polling.

### Manual Verification

- [ ] The first live match locks the tournament.
- [ ] An admin-set lock is never overwritten.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R3.
- Depends on the polling loop and match ingestion tickets.

## Notes

- This ticket does not cover UI lock indicators.
