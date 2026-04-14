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

### Wave 2 — 2026-04-14 (build-site.md)
- T-008: Group standings route+template — DONE. Files: standings/handlers.rs, standings/db.rs, standings/models.rs, templates/standings/groups.html. Build P, Tests P.
- T-010: Badge display member_stats — DONE. Files: standings/handlers.rs, templates/standings/member_stats.html, achievements.rs. Build P.
- T-011: Badge column leaderboard — DONE. Files: standings/models.rs, standings/handlers.rs, achievements.rs, templates/standings/leaderboard.html. Build P.
- T-012: Prediction counter logic — DONE. Files: predictions/handlers.rs, templates/predictions/index.html. Build P.
- T-013: Live counter Alpine.js — DONE. Files: templates/predictions/index.html. Build P.
- T-014: Match results display — DONE. Files: predictions/models.rs, predictions/db.rs, templates/predictions/index.html. Build P.
- T-015: Correctness indicator — DONE. Files: templates/predictions/index.html. Build P.
- T-017: Confidence checkbox — DONE. Files: predictions/handlers.rs, templates/predictions/index.html. Build P.
- T-018: Scoring function update — DONE. Files: polling/scorer.rs, polling/db.rs. Build P, Tests P.
- T-019: Confidence breakdown indicator — DONE. Files: standings/models.rs, standings/db.rs, templates/standings/match.html. Build P.
- T-020: Per-round points query — DONE. Files: standings/db.rs. Build P.
- T-021: Per-round route+template — DONE. Files: standings/handlers.rs, standings/mod.rs, templates/standings/rounds.html. Build P.
- Commit: b3f0e13. Next: T-022, T-023, T-024 (Tier 3), T-025, T-026 (Tier 4)

### Wave 3 — 2026-04-14 (build-site.md)
- T-022: Scenario hypo param parsing — DONE. Files: standings/models.rs (parse_hypo_params, compute_projected_delta). Build P, Tests P.
- T-023: Leaderboard recompute with hypo results — DONE. Files: standings/handlers.rs (leaderboard_fragment accepts Query<HashMap>), standings/db.rs (get_unplayed_group_matches, get_predictions_for_matches). Build P.
- T-024: HTMX scenario UI + projected delta display — DONE. Files: templates/standings/index.html (scenario picker + scenarioPicker Alpine component), templates/standings/leaderboard.html (+N projected delta). Build P.
- T-025: Trace integration verify — DONE. Code wiring verified: TraceLayer in routes.rs, init_tracing() + shutdown_tracing() in main.rs, OTLP layer conditional on OTEL_EXPORTER_OTLP_ENDPOINT. No code changes needed.
- T-026: Badge job E2E tests — DONE. Files: src/achievements.rs (2 sqlx::test integration tests: consistent_predictor awarded at 75% accuracy, not awarded at 0%). Build P, Tests P (107 unit + 8 db + 7 auth = all pass).
- Commit: 11b735a. ALL TASKS COMPLETE.

### Wave 4 — 2026-04-14 (build-site.md)
- T-027: Hypo param whitelist enforcement — DONE. Files: standings/models.rs (MAX_HYPO_MATCHES, filter_hypo_by_whitelist, 9 tests), standings/handlers.rs (whitelist filtering in leaderboard_fragment), standings/db.rs (remove dead user_id field), achievements.rs (from_str→from_slug), group_standings.rs (slice fix), predictions/handlers.rs (let-chain). Build P, Tests P (117 unit). Commit: 0e034e1. ALL TASKS COMPLETE.
