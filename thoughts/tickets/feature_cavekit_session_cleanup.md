---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [auth, sessions, maintenance]
keywords: [session cleanup, expired sessions, tower_sessions, hourly job, session retention]
patterns: [scheduled cleanup task, stale-row deletion, background maintenance job]
---

# FEATURE-005: Session cleanup job

## Description
Run periodic cleanup of expired session rows so the session table does not grow without bound.

## Context
This is internal maintenance work that protects the session store from unbounded growth over time.

## Requirements
- Cleanup runs on a defined schedule.
- Cleanup deletes rows from `tower_sessions` where `expiry_date <= now()`.
- Cleanup logs the number of deleted sessions at info level.
- Cleanup runs even when there is nothing to delete.
- Cleanup restarts automatically if it crashes is handled by the supervisor, not this ticket.

### Functional Requirements
- Remove expired sessions from the database on a schedule.
- Produce operational logs for cleanup runs.

### Non-Functional Requirements
- Keep the task resilient and safe when no rows match.
- Do not add restart supervision logic inside the app.

## Current State
The source spec describes a periodic cleanup task for expired sessions.

## Desired State
Expired sessions are removed regularly, with observable logs and no manual intervention.

## Research Context

### Keywords to Search
- session cleanup - background maintenance task
- expired sessions - deletion target
- tower_sessions - table to prune
- hourly job - likely schedule
- session retention - growth control concern

### Patterns to Investigate
- scheduled cleanup task - recurring background work
- stale-row deletion - database pruning strategy
- background maintenance job - logging and resilience behavior

### Key Decisions Made
- Scheduling can be hourly or configuration-driven.
- Logging should report the number of deleted rows.
- Supervisor restart behavior is out of scope.

## Success Criteria
The ticket is complete when expired sessions are removed automatically on schedule.

### Automated Verification
- [ ] Test or integration check confirms expired rows are removed.
- [ ] Log output includes deleted-session counts.

### Manual Verification
- [ ] Expired sessions disappear after a cleanup run.
- [ ] No-op cleanup runs do not fail.

## Related Information
- Source doc: `context/kits/cavekit-auth.md`
- Requirement: `R5`

## Notes
Keep the scope limited to server-side cleanup.
