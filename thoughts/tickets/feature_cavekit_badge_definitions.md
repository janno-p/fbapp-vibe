---
type: feature
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [achievements, badges, tournament]
keywords: [BadgeSlug, badge type, slug, name, description, icon, emoji, perfect_group_round, underdog_caller, top_scorer, consistent_predictor, oracle]
patterns: [enum-based definitions, pure badge predicates, static metadata registry]
---

# FEATURE-036: Define achievement badge types

## Description
Create the canonical achievement badge set for Cavekit and define the logic needed to determine whether a user qualifies for each badge.

## Context
The badge system is a new tournament-side feature. Badge definitions are the source of truth for all downstream storage, award, and display work.

## Requirements
- Define at least 5 distinct badge types in code.
- Each badge must expose a unique slug, display name, description, and icon or emoji.
- Badge definitions must be constants or enums, not user-editable data.
- The initial badge set must include `perfect_group_round`, `underdog_caller`, `top_scorer`, `consistent_predictor`, and `oracle`.
- Each badge must have clear awarding logic that can be evaluated independently.

### Functional Requirements
- Provide a canonical list of all badges.
- Expose badge metadata for use by storage, award jobs, and templates.
- Keep award criteria testable as pure logic.

### Non-Functional Requirements
- Badge definitions must be deterministic and stable across tournaments.
- Metadata should be defined in code, not stored in the database.

## Current State
The cavekit badges spec exists only as a monolithic requirements document.

## Desired State
The codebase has a reusable badge definition layer that other badge features can build on.

## Research Context

### Keywords to Search
- `BadgeSlug` - likely enum or constant type
- badge type - core domain concept
- slug - unique identifier contract
- emoji - presentation metadata
- `perfect_group_round` - initial badge slug
- `underdog_caller` - initial badge slug
- `top_scorer` - initial badge slug
- `consistent_predictor` - initial badge slug
- `oracle` - initial badge slug

### Patterns to Investigate
- enum-based definitions - how other domain constants are modeled
- pure badge predicates - testable qualification functions
- static metadata registry - shared badge lookup pattern

### Key Decisions Made
- Badge definitions live in code, not in user-editable storage.
- The initial badge set is fixed to the five named badges from the kit.

## Success Criteria
The ticket is complete when badge types and their metadata exist in code and can be referenced by other badge flows.

### Automated Verification
- [ ] Unit tests cover each badge definition and metadata field.
- [ ] Unit tests validate the badge list contains at least five entries.

### Manual Verification
- [ ] Badge names and slugs are readable and stable.
- [ ] Badge metadata is suitable for UI display.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R1`
- Depends on: badge storage, award job, and display tickets.

## Notes
Keep the badge catalog small and fixed for now; do not add tiering, rarity, or custom badges.
