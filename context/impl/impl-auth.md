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
