---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, tournament, models, templates]
keywords: [Tournament, Match, Team, Group, Player, is_predictions_locked, template serialization]
patterns: [domain model alignment, optional fields, template-safe serialization, nullable timestamps]
---

# FEATURE-CAVEKIT-TOURNAMENT-05: Tournament domain models

## Summary

Provide the shared tournament domain types so handlers, DB code, and templates all speak the same schema.

## Acceptance Criteria

- [ ] `Tournament` includes `id`, `external_id`, `name`, `season`, `is_active`, and `predictions_locked_at`.
- [ ] `Tournament::is_predictions_locked()` returns true when the lock timestamp is set and not in the future.
- [ ] `Match` includes `home_score`, `away_score`, and `outcome` as optional values.
- [ ] `Team` includes `flag: Option<String>`.
- [ ] `Group` includes `id`, `tournament_id`, and `name`.
- [ ] `Player` includes `goals_scored`.
- [ ] All types can be queried and serialized for templates.

## Implementation Context

### Relevant files

- `src/modules/admin/models.rs` - tournament admin-facing types.
- `src/modules/standings/` - consumers of shared tournament data.
- `src/modules/predictions/` - lock-state consumers.
- `src/modules/**/templates` - Askama render targets.

### ADR constraints

- **ADR-0007**: Prefer module-local feature code and shared top-level types where appropriate.
- **ADR-0005**: Use SQLx macro-backed structs for query mapping.

### Tests

- Unit test for `is_predictions_locked()` boundary behavior.
- Compile-time coverage via SQLx and Askama type matching.

### Implementation notes

- Keep the types minimal and aligned with persisted data.
- Optional fields should match real API and DB nullability.

## Research Context

### Keywords to Search

- `Tournament` - primary aggregate type.
- `Match` - fixture model.
- `Team` - team model and flag field.
- `Group` - group-stage model.
- `Player` - roster model.
- `is_predictions_locked` - helper method.

### Patterns to Investigate

- domain model alignment - keep structs aligned with DB and templates.
- optional fields - represent incomplete match data safely.
- template-safe serialization - types rendered by Askama.
- nullable timestamps - lock state storage.

### Key Decisions Made

- The lock helper is derived from the timestamp, not a separate flag.
- Template compatibility is part of the contract.
- Model shape should mirror the tournament persistence layer.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers the lock helper.
- [ ] `cargo build` succeeds with all model consumers.

### Manual Verification

- [ ] Templates can render tournament, team, group, match, and player data.

## Related Information

- Source requirement: `context/kits/cavekit-tournament.md` R6.
- Depends on tournament registration.

## Notes

- Do not add business logic beyond the lock helper here.
