---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, polling, results, matches]
keywords: [football-data.org, finished matches, home_score, away_score, match outcome, idempotent upsert]
patterns: [external API ingestion, idempotent update, status gating, result normalization, write-through sync]
---

# FEATURE-SCORING-02: Ingest finished match results

## Summary

Fetch finished matches from football-data.org and persist the local match result fields needed for scoring.

## Acceptance Criteria

- [ ] The polling task calls football-data.org for the active tournament's matches.
- [ ] Only matches with status `FINISHED` are written locally.
- [ ] `home_score` and `away_score` are stored from the API response.
- [ ] The local `outcome` is computed from the stored scores.
- [ ] Match rows are unchanged when the upstream match is not finished.
- [ ] Re-fetching the same finished match is idempotent and does not duplicate data.
- [ ] Each cycle logs the number of updated matches at info level.

## Implementation Context

### Relevant files

- `src/polling/db.rs` - result upserts and local match updates.
- `src/football_api/mod.rs` - match fetch response types.
- `migrations/0005_tournament_core.sql` - result columns and outcome storage.
- `src/modules/standings/db.rs` - read paths that consume stored results.

### Tests

- `#[sqlx::test]` for ingesting a finished match twice.
- Unit test for score-to-outcome mapping.

### Implementation notes

- Do not update unfinished matches.
- Preserve upstream values as the source of truth for result data.

## Research Context

### Keywords to Search

- `football-data.org` - upstream result source.
- `FINISHED` - status gate.
- `home_score` / `away_score` - persisted fields.
- `match outcome` - derived enum value.

### Patterns to Investigate

- external API ingestion - fetch then normalize then persist.
- idempotent update - safe repeat processing.
- status gating - only completed records are written.

### Key Decisions Made

- Only finished matches are persisted.
- Outcome is derived from scores, not copied blindly.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers finished and unfinished match cases.
- [ ] `sqlx` queries compile against the match schema.

### Manual Verification

- [ ] A finished match updates local score fields.
- [ ] A live match does not get written prematurely.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R2.
- Depends on the background polling loop ticket.

## Notes

- This ticket covers match ingestion only, not scoring the predictions that depend on it.
