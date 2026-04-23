---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [ui, standings, badges]
keywords: [member stats, badges section, awarded_at, chronological order, no badges earned yet, achievement count]
patterns: [member profile display, chronological list rendering, empty-state handling]
---

# FEATURE-039: Show earned badges on member stats

## Description
Display a user’s earned badges on the member stats page so league members can see achievements prominently.

## Context
Badges are meant to be visible social proof. The member stats page is the primary place where a user’s achievements should appear.

## Requirements
- Render a badges section on `GET /leagues/{id}/members/{user_id}`.
- Display all badges earned in the active tournament.
- Show each badge’s icon or emoji, name, and short description.
- Order badges by `awarded_at` ascending.
- Show a clear empty state when no badges exist.
- Keep the section visible to all league members.
- Show a completed badge count such as `3 / 5 badges earned`.

### Functional Requirements
- Load and display badge data on the member stats page.
- Present badge information in a readable, chronological format.
- Support the no-badges empty state.

### Non-Functional Requirements
- Badge visibility must not be restricted to private views.
- The layout should remain readable with multiple badges.

## Current State
The member stats page does not show achievement badges.

## Desired State
Member stats includes a visible, ordered badge section with a useful empty state and count.

## Research Context

### Keywords to Search
- member stats - target page
- badges section - UI block to add
- awarded_at - sort field
- no badges earned yet - empty-state copy
- achievement count - summary display

### Patterns to Investigate
- member profile display - where to place the section
- chronological list rendering - badge ordering
- empty-state handling - messaging when no achievements exist

### Key Decisions Made
- Badge visibility is public within the league.
- The badge list is ordered by award time.

## Success Criteria
The ticket is complete when member stats surfaces all earned badges with the required metadata and empty state.

### Automated Verification
- [ ] Integration test covers the badges section rendering.
- [ ] Integration test covers the empty state when no badges exist.

### Manual Verification
- [ ] Badges are visible to league members.
- [ ] The count and ordering match the stored achievements.

## Related Information
- Source doc: `context/kits/cavekit-badges.md`
- Requirement: `R4`
- Depends on: badge definitions, storage, and award job tickets.
- Depends on: member stats page existing in standings.

## Notes
Do not add badge privacy controls or member-only visibility rules beyond the existing league access model.
