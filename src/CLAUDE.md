# src/session_cleanup.rs

Implements:
- cavekit-auth.md R5 (Session Cleanup)

Build tasks: T-006 (context/plans/build-site.md)

Note: SQL bug fixed in T-006 — query now correctly targets `tower_sessions.session` (schema.table) instead of the non-existent public `tower_sessions` table.
