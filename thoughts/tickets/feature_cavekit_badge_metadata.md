---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [metadata, achievements, badges]
keywords: [badge metadata, display name, short description, emoji, icon, constants, no database storage]
patterns: [shared metadata contract, code-defined presentation data, lookup helpers]
---

# FEATURE-041: Define badge metadata for display

## Description
Expose badge metadata as a stable code-level contract so UI surfaces can render names, descriptions, and icons consistently.

## Context
Badge storage only needs the slug, but the UI needs human-readable metadata. This ticket formalizes the display contract for badges.

## Requirements
- Each badge slug must map to a display name.
- Each badge slug must map to a short description.
- Each badge slug must map to an emoji or icon.
- Metadata must be defined in code, not in the database.
- Metadata should be reusable across the member stats page and leaderboard display.

### Functional Requirements
- Provide a lookup path from badge slug to display metadata.
- Keep badge metadata consistent across all badge surfaces.

### Non-Functional Requirements
- Metadata changes should require code review, not database edits.
- The metadata contract must remain stable for UI rendering.

## Current State
The kit repeats badge metadata requirements, but there is no shared implementation contract yet.

## Desired State
Badge presentation metadata is centralized and reusable wherever badges are shown.

## Research Context

### Keywords to Search
- badge metadata - shared presentation contract
- display name - UI label field
- short description - tooltip or helper text field
- emoji - compact badge marker
- icon - alternative badge marker
- constants - code-defined metadata source

### Patterns to Investigate
- shared metadata contract - one source of truth for UI copy
- code-defined presentation data - non-database metadata pattern
- lookup helpers - slug-to-metadata mapping

### Key Decisions Made
- Metadata belongs in code, not in persistent badge rows.
- The same metadata must feed both the member stats and leaderboard views.

## Success Criteria
The ticket is complete when every badge slug has a reusable display metadata record accessible from code.

### Automated Verification
- [ ] Unit test confirms every slug has name, description, and icon metadata.
- [ ] Unit test confirms metadata lookup returns stable values.

### Manual Verification
- [ ] Badge labels are human-readable in the UI.
- [ ] Badge descriptions are concise enough for display and tooltips.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R6`
- Depends on: badge definitions ticket.

## Notes
This is a support ticket for the badge UI and should not introduce a separate storage model.
