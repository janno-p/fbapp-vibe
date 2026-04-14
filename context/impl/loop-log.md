# Loop Log — Build Site (context/plans/build-site.md)

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

### Wave 1 — 2026-04-14 (build-site.md)
- T-001: OTLP deps + init_tracing() — DONE. Files: Cargo.toml, src/tracing_setup.rs. Build P, Tests P.
- T-002: Tracer provider init + shutdown — DONE. Files: src/tracing_setup.rs, src/main.rs. Build P, Tests P.
- T-003: Docker Compose + env docs — DONE. Files: docker-compose.yml, .env.example. Build P, Tests P.
- T-004: BadgeSlug enum (5 badges) — DONE. Files: src/achievements.rs. Build P, Tests P.
- T-005: user_achievements migration — DONE. Files: migrations/0013_user_achievements.sql. Build P.
- T-006: group_standings.rs pure module — DONE. Files: src/group_standings.rs. Build P, 11 unit tests P.
- T-007: Group standings unit tests — DONE (embedded in T-006 file). Tests P.
- T-009: run_badge_award_job() — DONE. Files: src/achievements.rs. Build P.
- T-016: is_confident migration — DONE. Files: migrations/0014_confidence_flag.sql. Build P.
- Commit: 91c9536. Next: T-008, T-010, T-012, T-014, T-017, T-018, T-020
