# Impl Tracking — Auth

Build site: context/plans/build-site.md
Pre-build ref: a0f50a792befffbdc66ea349c080f0b83a89542c

## Tasks

| Task | Title | Status | Notes |
|------|-------|--------|-------|
| T-001 | Verify User Model and AuthUser Trait | DONE | models.rs + db.rs tests; 90 tests pass |
| T-002 | Verify Google OAuth Login Flow | DONE | mod.rs AuthBackend tests |
| T-003 | Verify Session Storage and Restoration | DONE | session hash tests in models.rs |
| T-004 | Verify Admin Role Access Control | DONE | admin/mod.rs unit tests |
| T-005 | Verify Public Pages | DONE | handlers.rs template + route tests |
| T-006 | Verify Session Cleanup Background Task | DONE | session_cleanup.rs; SQL bug fixed (tower_sessions → tower_sessions.session) |
| T-007 | Integration test: dashboard requires auth | DONE | tests/auth_routes.rs |
| T-008 | Integration test: logout destroys session | DONE | tests/auth_routes.rs |
| T-009 | Integration test: home redirects authenticated user | DONE | tests/auth_routes.rs |
| T-010 | Integration test: admin RBAC via HTTP | DONE | tests/auth_routes.rs; removed false-confidence unit tests from admin/mod.rs |
| T-011 | Integration test: expired/invalidated session | DONE | tests/auth_routes.rs |
