---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, domain-models, scoring, enums]
keywords: [MatchOutcome, KnockoutRound, string slug, serializable, testable]
patterns: [domain enum mapping, serde model, slug conversion, shared type definition, schema alignment]
---

# FEATURE-SCORING-08: Define scoring domain models

## Summary

Add the shared domain types used by the scoring pipeline so result and round logic stay consistent.

## Acceptance Criteria

- [ ] `MatchOutcome` exists with `Home`, `Draw`, and `Away` variants.
- [ ] `KnockoutRound` exists with `R32`, `R16`, `QF`, `SF`, `Final`, and `Winner` variants.
- [ ] Both enums convert to and from string slugs.
- [ ] The types are serializable.
- [ ] The types are covered by tests.

## Implementation Context

### Relevant files

- `src/db_types.rs` - shared database-backed enums.
- `src/polling/scorer.rs` - scoring functions that consume the types.
- `src/modules/standings/models.rs` - read models that display outcomes and rounds.
- `src/modules/standings/db.rs` - queries that map DB rows to the enums.

### Tests

- Unit tests for string conversion in both directions.
- Unit tests for serde round-tripping.

### Implementation notes

- Keep the database enum names aligned with the API and local schema.
- Prefer a single canonical mapping layer instead of per-call conversion logic.

## Research Context

### Keywords to Search

- `MatchOutcome` - match result enum.
- `KnockoutRound` - knockout round enum.
- `string slug` - external representation.
- `serializable` - data interchange support.

### Patterns to Investigate

- domain enum mapping - convert between DB and app types.
- serde model - stable serialization behavior.
- schema alignment - keep Rust and SQL naming in sync.

### Key Decisions Made

- Scoring uses shared domain enums rather than ad hoc strings.
- Slug conversion is part of the domain model contract.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers all enum conversions.
- [ ] The types compile cleanly anywhere they are referenced.

### Manual Verification

- [ ] Slugs round-trip to the same enum values.
- [ ] Serialized values are stable across the app.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R8.
- This model work underpins the scoring tickets above.

## Notes

- This ticket does not add new business logic; it defines shared value types.
