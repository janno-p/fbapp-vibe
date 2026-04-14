# src/ — top-level modules

## src/session_cleanup.rs

Implements:
- cavekit-auth.md R5 (Session Cleanup)

Build tasks: T-006 (context/plans/build-site.md)

Note: SQL bug fixed in T-006 — query now correctly targets `tower_sessions.session` (schema.table) instead of the non-existent public `tower_sessions` table.

## src/tracing_setup.rs

Implements:
- cavekit-observability.md R1 (OTLP deps)
- cavekit-observability.md R2 (init_tracing with conditional OTLP layer)
- cavekit-observability.md R3 (Batch OTLP exporter)
- cavekit-observability.md R4 (shutdown_tracing on graceful exit)

Build tasks: T-001, T-002 (context/plans/build-site.md)

## src/achievements.rs

Implements:
- cavekit-badges.md R1 (BadgeSlug enum, 5 badge types)
- cavekit-badges.md R2 (user_achievements table queries)
- cavekit-badges.md R3 (run_badge_award_job background evaluation)
- cavekit-badges.md R4 (get_user_badges for member stats display)
- cavekit-badges.md R5 (get_top_badge_per_user for leaderboard column)
- cavekit-badges.md R6 (badge metadata: name, description, emoji)

Build tasks: T-004, T-009, T-010, T-011, T-026 (context/plans/build-site.md)

## src/group_standings.rs

Implements:
- cavekit-standings.md R8 (Pure group standings computation from match results)

Build tasks: T-006, T-007 (context/plans/build-site.md)
