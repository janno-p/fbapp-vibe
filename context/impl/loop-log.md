# Loop Log — Auth Build

Build site: context/plans/build-site.md

### Wave 1 — 2026-04-13
- T-001: User model + AuthUser trait — DONE. Files: auth/models.rs, auth/db.rs. Build P, Tests P (90/90).
- T-002: OAuth flow backend — DONE. Files: auth/mod.rs. Build P, Tests P.
- T-003: Session storage — DONE. Files: auth/models.rs (hash tests). Build P, Tests P.
- T-004: Admin RBAC — DONE. Files: admin/mod.rs. Build P, Tests P.
- T-005: Public pages — DONE. Files: auth/handlers.rs. Build P, Tests P.
- T-006: Session cleanup — DONE. Files: session_cleanup.rs. SQL bug fixed. Build P, Tests P.
- Commit: 39efac8

### Wave 2 — 2026-04-13
- T-007: Dashboard requires auth — DONE. Files: tests/auth_routes.rs.
- T-008: Logout destroys session — DONE. Files: tests/auth_routes.rs.
- T-009: Home redirects authed user — DONE. Files: tests/auth_routes.rs.
- T-010: Admin RBAC HTTP tests — DONE. Files: tests/auth_routes.rs, admin/mod.rs (false-confidence unit tests removed).
- T-011: Expired/invalidated session — DONE. Files: tests/auth_routes.rs.
- Build P, Tests P (7/7). Commit: 29d5d2d
