---
created: "2026-04-13T00:00:00Z"
last_edited: "2026-04-14T00:00:00Z"
---

# Review Findings — Auth Loop Inspection

| Finding | Severity | File | Status |
|---------|----------|------|--------|
| F-001: Session cleanup logs at DEBUG instead of INFO | P2 | src/session_cleanup.rs:13 | FIXED |
| F-002: AdminUser unit tests re-implement extractor logic (false confidence) | P1 | src/modules/admin/mod.rs:56-81 | NEW → T-010 |
| F-003: No integration test for GET /dashboard auth protection | P1 | src/modules/auth/handlers.rs:53-64 | NEW → T-007 |
| F-004: No test for GET / redirect when authenticated | P2 | src/modules/auth/handlers.rs:44-51 | NEW → T-009 |
| F-005: No integration test for POST /auth/logout session destruction | P2 | src/modules/auth/handlers.rs:169-176 | NEW → T-008 |
| F-006: login_route_path_is_correct is compile-time only, not behavioral | P3 | src/modules/auth/handlers.rs:192-199 | NEW (low priority) |
| F-007: No integration test for OAuth callback token exchange | P1 | src/modules/auth/handlers.rs:94-167 | NEW (deferred — requires mock infrastructure) |
| F-008: Session auth hash invalidation not tested end-to-end | P2 | src/modules/auth/models.rs:21-23 | NEW → T-011 |
| F-009: Expired session returning 401 not tested end-to-end | P2 | src/session_cleanup.rs | NEW → T-011 |
| F-010: SQL uses < instead of <= for expiry_date comparison | P2 | src/session_cleanup.rs:21 | FIXED |

## Review Findings — Standings/Badges/Predictions/Scoring Loop Inspection (2026-04-14)

From `/ck:check` run — 2026-04-14. Coverage: 100% (45/45 req, 229/229 criteria). Verdict: REVISE.

| Finding | Severity | File | Status |
|---------|----------|------|--------|
| F-011: Hypo match IDs not checked against unplayed_matches whitelist — knockout IDs accepted | P1 | src/modules/standings/handlers.rs | FIXED (T-027) |
| F-012: No size limit on hypo params (no MAX_HYPO_MATCHES check) | P2 | src/modules/standings/handlers.rs | FIXED (T-027) |
| F-013: Hypo match IDs not validated against active tournament | P2 | src/modules/standings/handlers.rs | FIXED (T-027) |
| F-014: Hypo param values not validated (home/draw/away only) | P2 | src/modules/standings/handlers.rs | FIXED (T-027) |
| F-015: `compute_projected_delta` ignores confidence flag — confident+correct should award 2pts not 1 | P2 | src/modules/standings/models.rs | NEW |
| F-016: Leaderboard delta only for users with predictions; users without silently drop off | P2 | src/modules/standings/models.rs | NEW |
| F-017: Missing unit tests for scenario modeling with invalid/nonexistent match IDs | P2 | src/modules/standings/models.rs | NEW |
| F-018: No integration test for Oracle badge with actual final match data | P2 | src/achievements.rs | NEW |
| F-019: Missing integration test for per-round leaderboard empty-state | P2 | src/modules/standings/ | NEW |
| F-020: Dead code — `user_id` field in `RoundPoints` struct | P3 | src/modules/standings/models.rs | FIXED (T-027) |
| F-021: `RoundPoints` rank relies on query sort order — fragile if ORDER BY changes | P3 | src/modules/standings/models.rs | NEW |
