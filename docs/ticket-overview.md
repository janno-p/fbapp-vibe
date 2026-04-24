---
date: 2026-04-24T13:57:21+03:00
git_commit: a0623f2
branch: main
repository: fbapp-vibe
topic: "Generated ticket overview and implementation order"
tags: [tickets, planning, dependencies, roadmap, generated]
last_updated: 2026-04-24
generated: true
---

# 🧭 Ticket Overview

Single source of truth for all tickets. Use this file to understand what exists, what depends on what, and what should be worked on next.

## 🔄 Generation Notes

- This document is generated from the ticket files under `thoughts/tickets/`.
- All tickets are included, even completed and cancelled ones, so history stays visible.
- `estimate` and `complexity` are heuristic until the source tickets define them explicitly.
- Update this overview whenever ticket status, dependencies, or scope changes.

## 📊 Snapshot

- Tickets: 109
- Types: feature 80, bug 8, chore 19, debt/refactor 2
- Statuses: created 57, open 17, in-progress 0, done 27, reviewed 7, cancelled 1
- Priority mix: high 22, medium 37, low 2

## 🧠 Ordering Rules

### Primary order

1. Hard dependencies first
2. Blockers before blocked work
3. Highest priority among available tickets

### Secondary order

- Lower complexity before higher complexity when priority is equal
- Smaller estimate before larger estimate when risk is equal
- Foundation work before feature work before cleanup

### Alternative views

- By `domain`
- By `type`
- By `status`
- By `priority`
- By `complexity`
- By `estimate`

## 🔗 Dependency Notes

- `FEATURE-002: User model for auth integration`: Auth depends on a stable user representation that can be upserted from Google profile data and loaded by ID during session restoration.
- `FEATURE-LEAGUES-03: Token-based league joining`: This is the user-facing entry point for membership and depends on a valid invite token.
- `Install mkcert (once per machine)`: refs: [0002]
- `auth-module`: refs: []
- `code-housekeeping`: refs: []
- `league-join-open-redirect`: refs: [0006]
- `predictions`: refs: [0005]
- `session-cleanup`: refs: []; If implementing manually: `DELETE FROM tower_sessions WHERE expiry_date < NOW()` (exact column name depends on crate version — check the migration or table schema).
- `styled-error-pages`: refs: []
- `FEATURE-037: Persist awarded badges`: Depends on: badge definitions ticket.
- `FEATURE-038: Award badges after scoring completes`: Depends on: badge definitions and storage tickets.; Also depends on: scoring and leaderboard data being finalized.
- `feature_cavekit_main_leaderboard_standings`: refs: []
- `FEATURE-CAVEKIT-TOURNAMENT-01: Tournament registration from football-data.org`: Depends on `cavekit-auth`.
- `FEATURE-CAVEKIT-TOURNAMENT-02: Seed tournament data from football-data.org`: Depends on tournament registration.
- `FEATURE-CAVEKIT-TOURNAMENT-03: Activate exactly one tournament at a time`: Depends on tournament registration and admin auth.
- `FEATURE-CAVEKIT-TOURNAMENT-04: Manual prediction lock and unlock`: Depends on tournament activation and prediction write enforcement.
- `FEATURE-CAVEKIT-TOURNAMENT-05: Tournament domain models`: Depends on tournament registration.
- `FEATURE-CAVEKIT-TOURNAMENT-06: Display team national flags`: Depends on tournament seeding and the team model.
- `FEATURE-SCORING-01: Background polling loop for cavekit scoring`: Depends on the active tournament state existing in the database.
- `FEATURE-SCORING-03: Auto-lock predictions on first kickoff`: Depends on the polling loop and match ingestion tickets.
- `FEATURE-SCORING-06: Score top scorer predictions`: Depends on result ingestion and background polling.
- `feature_cavekit_fixtures_page`: refs: [R3, cavekit-tournament, cavekit-leagues]
- `feature_cavekit_per_round_leaderboard_breakdown`: refs: [R1, cavekit-scoring]
- `admin-route-smoke-tests`: refs: [0005]
- `auto-lock-predictions-at-kickoff`: refs: [0033]
- `batch-seeding-inserts`: refs: [0005]
- `country-flag-images`: refs: [0026]
- `empty-states`: refs: []
- `enforce-lock-server-side`: refs: [0042]
- `fixture-list`: refs: [0021]
- `football-api-integration`: refs: []
- `group-stage-standings`: refs: [0018]
- `hide-predictions-before-lock`: refs: [0042]
- `leaderboard-standings`: refs: [0006, 0007, 0008]
- `per-round-leaderboard`: refs: [0009, 0028]
- `player-goals-display`: refs: []
- `prediction-revision-window`: refs: [0007]
- `result-polling`: refs: [0004, 0005]
- `tournament-aware-knockout-rounds`: refs: []
- `tournament-management`: refs: [0004]
- `validate-prediction-ids`: refs: [0007]
- `feature_cavekit_potential_points_indicator`: refs: [R1, R2, cavekit-scoring]
- `FEATURE-SCORING-02: Ingest finished match results`: Depends on the background polling loop ticket.
- `FEATURE-SCORING-04: Score group stage predictions`: Depends on result ingestion.
- `FEATURE-SCORING-05: Score knockout predictions`: Depends on result ingestion and group-stage scoring.
- `FEATURE-SCORING-07: Sync player goal counts`: Depends on the background polling loop and football API integration.
- `per-user-prediction-stats`: refs: [0025, 0027]
- `feature_cavekit_match_breakdown`: refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-predictions]
- `feature_cavekit_member_comparison`: refs: [R1, cavekit-leagues, cavekit-scoring]
- `feature_cavekit_member_stats_page`: refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-badges]
- `achievement-badges`: refs: [0008, 0030]
- `confidence-multiplier`: refs: [0007, 0008]
- `consensus-view`: refs: [0021, 0025]
- `global-navigation`: refs: []
- `group-prediction-completion-indicator`: refs: []
- `group-save-htmx-feedback`: refs: []
- `kickoff-countdown`: refs: [0021, 0026]
- `leagues`: refs: []
- `match-schedule-display`: refs: []
- `prediction-table-indexes`: refs: [0007]
- `predictions-review`: refs: [0007, 0025]
- `predictions-review-knockout-ux`: refs: [0027]
- `show-results-on-predictions-page`: refs: [0043]
- `feature_cavekit_group_stage_standings_table`: refs: [R9]
- `FEATURE-039: Show earned badges on member stats`: Depends on: badge definitions, storage, and award job tickets.; Depends on: member stats page existing in standings.
- `leaderboard-tiebreaking`: refs: [0009]
- `league-member-browser`: refs: []
- `feature_cavekit_htmx_leaderboard_fragment`: refs: [R1]
- `feature_cavekit_hypo_param_validation`: refs: [R9, R8]
- `feature_cavekit_scenario_modeling`: refs: [R1, R2, cavekit-scoring]
- `FEATURE-040: Optionally show a badge on the leaderboard`: Depends on: badge definitions, storage, and display metadata tickets.; Depends on: leaderboard page existing in standings.
- `scenario-modeling`: refs: [0009]
- `FEATURE-036: Define achievement badge types`: Depends on: badge storage, award job, and display tickets.
- `FEATURE-041: Define badge metadata for display`: Depends on: badge definitions ticket.
- `otlp-jaeger-observability`: refs: []
- `crest-docs-and-fallback-asset`: refs: [0036]
- `rust-2024-edition-docs`: refs: []
- `knockout-topscore-count-ux`: refs: []
- `project-scaffold`: refs: []
- `qsform-body-limit`: refs: []

## 📋 Ticket Registry

| ID | Type | Status | Priority | Domain | Summary | Estimate | Complexity |
|---|---|---|---|---|---|---:|---|
| `debt_cavekit_auth_integration_tests` | 🧹 debt | 👍 reviewed | 🔴 high | auth | Add real HTTP integration coverage for the critical auth flows so session, authorization, and invalidation behavior is verified against the actual stack. | L | high |
| `feature_cavekit_google_oauth_login_flow` | ✨ feature | 👍 reviewed | 🔴 high | auth | Implement the Google OAuth login flow so users can authenticate, have their account information synchronized, and be redirected into the app with a valid session. | L | high |
| `feature_cavekit_user_model` | ✨ feature | 👍 reviewed | 🔴 high | auth | Define the user account model used by auth so identity, contact data, and role state are stored consistently and can be loaded for sessions.<br><br>**Depends on:** Auth depends on a stable user representation that can be upserted from Google profile data and loaded by ID during session restoration. | L | high |
| `feature_cavekit_session_storage_restoration` | ✨ feature | 👍 reviewed | 🔴 high | auth | Ensure authenticated sessions persist in PostgreSQL and are restored on subsequent requests through the auth session extractor. | M | medium |
| `feature_cavekit_admin_role_access_control` | ✨ feature | 🆕 created | 🔴 high | auth | Add binary admin authorization so only users with `is_admin = true` can reach admin-only routes and management actions. | M | medium |
| `feature_cavekit_league_join_by_token` | ✨ feature | 🆕 created | 🔴 high | auth | Allow authenticated users to join a league by visiting a shareable invite link.<br><br>**Depends on:** This is the user-facing entry point for membership and depends on a valid invite token. | M | medium |
| `feature_cavekit_session_cleanup` | ✨ feature | 🆕 created | 🟠 medium | auth | Run periodic cleanup of expired session rows so the session table does not grow without bound. | M | medium |
| `feature_cavekit_public_pages` | ✨ feature | 🆕 created | 🟠 medium | auth | Allow unauthenticated users to access the home page while keeping the dashboard protected and redirecting users appropriately based on auth state. | M | medium |
| `feature_cavekit_group_stage_prediction_form` | ✨ feature | 🆕 created | 🟠 medium | auth | Allow authenticated users to submit and update home/draw/away predictions for every group stage match in the active tournament. | M | medium |
| `https-dev-tls` | 🧰 chore | ✅ done | ⚪ tbd | auth | Enable the development server to serve traffic over HTTPS using a locally-trusted self-signed certificate. This is required because the OAuth `session_secret` cookie is set with `with_secure(true)` in production mode and browsers refuse to send secure cookies over plain HTTP. Running on HTTPS locally also ensures the development environment matches production behaviour and avoids subtle auth bugs caused by the HTTP/HTTPS mismatch.<br><br>**Depends on:** refs: [0002] | S | low |
| `auth-module` | ✨ feature | ✅ done | ⚪ tbd | auth | Implement Google OAuth authentication and two landing pages: a public home page for unauthenticated visitors (with a "Sign in with Google" button) and a protected dashboard page for authenticated users. This establishes the auth foundation all future features build upon. | L | high |
| `code-housekeeping` | 🧰 chore | ✅ done | ⚪ tbd | auth | Several small quality issues found during code review: unused struct fields suppressed with `#[allow(dead_code)]`, a magic number in the top scorer handler, session expiry hardcoded in `main.rs`, an error message that echoes raw user input, and admin actions performed without any audit log. Fix all of these in one pass. | S | low |
| `league-join-open-redirect` | 🐞 bug | 🔓 open | ⚪ tbd | auth | The league join handler stores the current request URL in the session as `post_login_redirect` when the user is not authenticated. After login, the auth callback reads that value and redirects to it without validation. An attacker can craft a link like `/leagues/join/<token>` from a page that sets an external URL, resulting in a post-login redirect to an attacker-controlled site.<br><br>**Depends on:** refs: [0006] | M | medium |
| `predictions` | ✨ feature | ✅ done | ⚪ tbd | auth | Let authenticated users submit and edit their tournament predictions before the prediction lock. Predictions cover three areas: group stage match outcomes, knockout round advancement, and top scorer candidates. Once `predictions_locked_at` is set on the active tournament, all prediction writes are rejected.<br><br>**Depends on:** refs: [0005] | M | medium |
| `session-cleanup` | 🧰 chore | 🔓 open | ⚪ tbd | auth | `tower_sessions` stores sessions in the `tower_sessions` PostgreSQL table and never cleans up expired rows. Over time this table will grow unboundedly. A periodic cleanup removes rows whose expiry has passed, keeping the table small and index efficient.<br><br>**Depends on:** If implementing manually: `DELETE FROM tower_sessions WHERE expiry_date < NOW()` (exact column name depends on crate version — check the migration or table schema). | S | low |
| `styled-error-pages` | ✨ feature | ✅ done | ⚪ tbd | auth | Replace the plain-text error responses with styled HTML pages that extend the base layout. Currently `AppError::NotFound`, `AppError::Unauthorized`, `AppError::Forbidden`, and `AppError::Unexpected` all return bare text strings. Users hitting a 404 or 403 see a broken, unstyled response with no navigation back to the app. | M | medium |
| `bug_cavekit_prediction_lock_enforcement` | 🐞 bug | 🆕 created | 🔴 high | tournament | Reject every prediction write once the tournament lock is active so users cannot bypass the UI and submit POST requests directly. | M | medium |
| `feature_cavekit_badge_storage` | ✨ feature | 🆕 created | 🔴 high | tournament | Add durable storage for earned badges so the application can query a user’s achievements efficiently within a tournament.<br><br>**Depends on:** badge definitions ticket. | L | high |
| `feature_cavekit_badge_award_job` | ✨ feature | 🆕 created | 🔴 high | tournament | Run a background badge evaluation step after scoring finishes and persist any earned badges for all eligible users in the active tournament.<br><br>**Depends on:** badge definitions and storage tickets.; Also depends on: scoring and leaderboard data being finalized. | L | high |
| `feature_cavekit_main_leaderboard_standings` | ✨ feature | 🆕 created | 🔴 high | tournament | League members need a main standings view that ranks everyone by total points for the active tournament and explains where each user sits in the league. | L | high |
| `feature_cavekit_tournament_registration` | ✨ feature | 🆕 created | 🟠 medium | tournament | Let admins register a new tournament from the football-data.org competitions list so the app can begin managing one competition end to end.<br><br>**Depends on:** `cavekit-auth`. | L | high |
| `feature_cavekit_tournament_seeding` | ✨ feature | 🆕 created | 🟠 medium | tournament | Fetch and persist the tournament structure from football-data.org so the local database has teams, groups, matches, players, and memberships ready for downstream features.<br><br>**Depends on:** tournament registration. | M | medium |
| `feature_cavekit_tournament_activation` | ✨ feature | 🆕 created | 🟠 medium | tournament | Give admins control over which tournament is live so the rest of the app can consistently read one active competition.<br><br>**Depends on:** tournament registration and admin auth. | M | medium |
| `feature_cavekit_manual_prediction_locking` | ✨ feature | 🆕 created | 🟠 medium | tournament | Allow admins to manually lock or unlock tournament predictions so the submission window can be controlled directly.<br><br>**Depends on:** tournament activation and prediction write enforcement. | M | medium |
| `feature_cavekit_tournament_data_models` | ✨ feature | 🆕 created | 🟠 medium | tournament | Provide the shared tournament domain types so handlers, DB code, and templates all speak the same schema.<br><br>**Depends on:** tournament registration. | L | high |
| `feature_cavekit_team_flag_display` | ✨ feature | 🆕 created | 🟠 medium | tournament | Show teams with self-hosted national flags derived from their ISO country codes instead of external crest images.<br><br>**Depends on:** tournament seeding and the team model. | M | medium |
| `feature_cavekit_knockout_prediction_form` | ✨ feature | 🆕 created | 🟠 medium | tournament | Allow users to predict which teams advance through the tournament knockout rounds. | M | medium |
| `feature_cavekit_top_scorer_prediction_form` | ✨ feature | 🆕 created | 🟠 medium | tournament | Allow users to pick up to three players they expect to finish as the tournament top scorer. | M | medium |
| `feature_cavekit_prediction_visibility_controls` | ✨ feature | 🆕 created | 🟠 medium | tournament | Keep prediction data private before lock so users can only see their own submissions until the tournament is revealed. | M | medium |
| `feature_cavekit_background_polling_loop` | ✨ feature | 🆕 created | 🟠 medium | tournament | Run a long-lived background task that periodically polls for tournament results and drives the scoring pipeline.<br><br>**Depends on:** the active tournament state existing in the database. | L | high |
| `feature_cavekit_auto_lock_on_first_kickoff` | ✨ feature | 🆕 created | 🟠 medium | tournament | Lock predictions automatically when the first match of the active tournament starts so late edits cannot slip in.<br><br>**Depends on:** the polling loop and match ingestion tickets. | L | high |
| `feature_cavekit_top_scorer_scoring` | ✨ feature | 🆕 created | 🟠 medium | tournament | Award top scorer points once the tournament ends and the final top scorer is confirmed.<br><br>**Depends on:** result ingestion and background polling. | L | high |
| `feature_cavekit_fixtures_page` | ✨ feature | 🆕 created | 🟠 medium | tournament | Members need a fixtures page that groups upcoming and completed matches by tournament stage and date.<br><br>**Depends on:** refs: [R3, cavekit-tournament, cavekit-leagues] | M | medium |
| `feature_cavekit_per_round_leaderboard_breakdown` | ✨ feature | 🆕 created | 🟠 medium | tournament | Members need a standings view that breaks total points down by tournament round so performance by stage is visible.<br><br>**Depends on:** refs: [R1, cavekit-scoring] | L | high |
| `admin-route-smoke-tests` | 🧰 chore | ✅ done | ⚪ tbd | tournament | Several admin POST endpoints (e.g. `/admin/tournaments/{id}/activate`) are returning 404 in the<br><br>**Depends on:** refs: [0005] | S | low |
| `auto-lock-predictions-at-kickoff` | ✨ feature | 🔓 open | ⚪ tbd | tournament | Predictions should close automatically when the tournament begins — specifically when the scheduled kickoff time of the first match arrives. At that moment the system sets `predictions_locked_at` on the tournament row, preventing any further changes by users. Administrators retain the ability to manually unlock (and re-lock) via the existing admin dashboard controls for exceptional circumstances. No user action or manual admin intervention is required for the common case.<br><br>**Depends on:** refs: [0033] | M | medium |
| `batch-seeding-inserts` | 🧰 chore | 🔓 open | ⚪ tbd | tournament | `seed_tournament_data` in `src/modules/admin/db.rs` issues one SQL query per player and one per group membership inside nested loops. Seeding a 32-team tournament with 26-man squads produces ~850 player inserts plus group membership inserts individually. Batch these with multi-row `UNNEST`-based inserts to reduce round-trips to a fixed number of queries regardless of squad size.<br><br>**Depends on:** refs: [0005] | S | low |
| `country-flag-images` | ✨ feature | ✅ done | ⚪ tbd | tournament | Every team displayed on the site should show its country flag alongside the team name. Flags must be custom-designed static SVG assets committed to the repo — not raw external URLs from the football API. The design should be consistent and polished (think UEFA Euro / FIFA tournament pages): uniform 4:3 rounded-rectangle frames with the real country flag at full fidelity, styled to read well on the dark pitch background. Wherever a team name appears (fixtures, match breakdown, nearest match preview, knockout predictions), the flag appears at a uniform `w-8 h-6` (32×24 px) inline size.<br><br>**Depends on:** refs: [0026] | M | medium |
| `empty-states` | ✨ feature | ✅ done | ⚪ tbd | tournament | Several routes return 404 when there is no active tournament instead of showing a friendly message. `/predictions` and `/leagues/{id}/standings` both call `ok_or(AppError::NotFound)` on the active tournament lookup. This is confusing — the route exists, just nothing to show yet. Users before tournament activation see a cryptic 404. | L | high |
| `enforce-lock-server-side` | 🐞 bug | ✅ done | ⚪ tbd | tournament | The prediction lock introduced by task 0042 is only enforced in the UI (disabled HTML inputs). All three save handlers (`save_group`, `save_knockout`, `save_top_scorer`) fetch the active tournament but never check `tournament.is_predictions_locked()` before writing to the database. Any user can POST directly to these endpoints after kickoff to modify their predictions, bypassing the lock entirely.<br><br>**Depends on:** refs: [0042] | M | medium |
| `fixture-list` | ✨ feature | ✅ done | ⚪ tbd | tournament | Users have no single page to see all matches in the active tournament — who plays who, when, and what the result was. The match breakdown page (`/leagues/{id}/matches/{match_id}`) exists, but there is no index. A fixture list gives every league member an at-a-glance view of the full tournament schedule and live results.<br><br>**Depends on:** refs: [0021] | M | medium |
| `football-api-integration` | 🧰 chore | ✅ done | ⚪ tbd | tournament | Establish the external football data source and implement a typed API client for it. All tournament data — competitions, teams, groups, players, match fixtures, and live results — flows through this client. This task produces the shared infrastructure that tournament management (0005) and result polling (0008) both depend on. | S | low |
| `group-stage-standings` | ✨ feature | 🔓 open | ⚪ tbd | tournament | The API does not provide group stage standings tables. To support scenario modelling (task 0018) and to give users a current view of how each group looks mid-tournament, the application needs to compute group standings from the match results it already stores. Standings must work on partial data (some matches played, some still scheduled), apply the correct football competition tiebreaker rules, and be accessible as a page or component that can later be wired into the scenario modelling feature.<br><br>**Depends on:** refs: [0018] | L | high |
| `hide-predictions-before-lock` | ✨ feature | ✅ done | ⚪ tbd | tournament | League members must not be able to see each other's predictions before the tournament starts (i.e. before `predictions_locked_at` is reached). Seeing others' picks before the lock could influence late submissions and undermines the fairness of the competition. Once the tournament is locked, all prediction data becomes visible as normal — the reveal is part of the game experience.<br><br>**Depends on:** refs: [0042] | M | medium |
| `leaderboard-standings` | ✨ feature | ✅ done | ⚪ tbd | tournament | Display tournament standings to users in their league context. The primary view is centred on the nearest match (most recently finished or next upcoming). Users can explore the full leaderboard, a per-match points breakdown, future prospect calculations, and a head-to-head comparison between any two participants.<br><br>**Depends on:** refs: [0006, 0007, 0008] | L | high |
| `per-round-leaderboard` | ✨ feature | 🔓 open | ⚪ tbd | tournament | The league leaderboard shows cumulative totals but not how points were earned across tournament stages. A round-by-round breakdown lets members see who dominated the group stage, who came alive in the knockouts, and where rankings shifted. This adds narrative to the competition.<br><br>**Depends on:** refs: [0009, 0028] | L | high |
| `player-goals-display` | ✨ feature | ✅ done | ⚪ tbd | tournament | The `players` table has a `goals_scored` column that is updated by the polling loop, but this data is never surfaced in the UI. The top scorer prediction form just shows player names and teams. Showing live goal tallies helps users track their top scorer picks and makes the feature feel alive during the tournament. | L | high |
| `prediction-revision-window` | ✨ feature | ❌ cancelled | ⚪ tbd | tournament | Currently predictions can only be submitted once before the tournament locks. Many real-world prediction games allow updates up until shortly before kickoff (e.g., 15 minutes before the match starts). A per-match revision window lets users correct their group stage predictions based on team news or late information, making the game more engaging without compromising fairness.<br><br>**Depends on:** refs: [0007] | M | medium |
| `result-polling` | ✨ feature | ✅ done | ⚪ tbd | tournament | Implement a background task that polls the football API for match results, updates the local database, and recalculates `points_awarded` for all affected predictions. This is the engine that drives live leaderboard updates during the tournament.<br><br>**Depends on:** refs: [0004, 0005] | L | high |
| `tournament-aware-knockout-rounds` | 🐞 bug | 🔓 open | ⚪ tbd | tournament | The knockout predictions form always shows all six rounds (R32 → Winner) regardless of which rounds the tournament actually has. For a 16-team tournament like UEFA EURO 2024 — which starts at R16 — users see a confusing R32 section where they can submit predictions that will never be scored. The rounds that exist for a given tournament are already stored correctly in the `matches` table (seeded from the API), so the fix is to query those rounds and show only them. | M | medium |
| `tournament-management` | ✨ feature | 🆕 created | ⚪ tbd | tournament | Umbrella ticket for the Cavekit tournament management workstream. It groups the atomic tickets for registration, seeding, activation, manual locking, shared models, and team flag display.<br><br>**Depends on:** refs: [0004] | L | high |
| `validate-prediction-ids` | 🐞 bug | 🔓 open | ⚪ tbd | tournament | `save_knockout_round_predictions` and `save_top_scorer_predictions` accept team/player IDs from form input and insert them directly without checking they belong to the active tournament. A user could submit IDs from a different tournament (or arbitrary integers) and corrupt prediction data.<br><br>**Depends on:** refs: [0007] | M | medium |
| `feature_cavekit_potential_points_indicator` | ✨ feature | 🆕 created | 🔴 high | scoring | The leaderboard needs a visual ceiling indicator so users can see how much scoring headroom each player still has and how strong their remaining path looks relative to others.<br><br>**Depends on:** refs: [R1, R2, cavekit-scoring] | L | high |
| `feature_cavekit_result_ingestion` | ✨ feature | 🆕 created | 🟠 medium | scoring | Fetch finished matches from football-data.org and persist the local match result fields needed for scoring.<br><br>**Depends on:** the background polling loop ticket. | L | high |
| `feature_cavekit_group_stage_scoring` | ✨ feature | 🆕 created | 🟠 medium | scoring | Score group stage predictions using a pure outcome comparison function and persist the awarded points.<br><br>**Depends on:** result ingestion. | L | high |
| `feature_cavekit_knockout_scoring` | ✨ feature | 🆕 created | 🟠 medium | scoring | Score knockout predictions by round using the teams that advance from finished knockout matches.<br><br>**Depends on:** result ingestion and group-stage scoring. | L | high |
| `feature_cavekit_player_goal_tracking` | ✨ feature | 🆕 created | 🟠 medium | scoring | Keep player goal totals in sync with football-data.org so top scorer detection can rely on local data.<br><br>**Depends on:** the background polling loop and football API integration. | L | high |
| `feature_cavekit_scoring_models` | ✨ feature | 🆕 created | 🟠 medium | scoring | Add the shared domain types used by the scoring pipeline so result and round logic stay consistent. | L | high |
| `per-user-prediction-stats` | ✨ feature | ✅ done | ⚪ tbd | scoring | The leaderboard shows total points but gives no insight into *how* a user is scoring — are they getting lucky with a few big knockouts, or consistently predicting group stage outcomes correctly? Per-user stats (group stage accuracy %, current correct streak, breakdown by stage) add depth to the social competition and give users something to talk about.<br><br>**Depends on:** refs: [0025, 0027] | L | high |
| `feature_cavekit_league_prediction_review_page` | ✨ feature | 🆕 created | 🟠 medium | predictions | Give league members a per-league review page that compares every member's predictions with actual results and points. | L | high |
| `feature_cavekit_show_results_on_predictions_page` | ✨ feature | 🆕 created | 🟠 medium | predictions | Display finished match results alongside the user's predictions so they can see which picks were correct or wrong. | M | medium |
| `feature_cavekit_match_breakdown` | ✨ feature | 🆕 created | 🟠 medium | predictions | Members need a per-match breakdown that shows how every league member predicted a fixture and how many points each prediction earned.<br><br>**Depends on:** refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-predictions] | M | medium |
| `feature_cavekit_member_comparison` | ✨ feature | 🆕 created | 🟠 medium | predictions | Members need a comparison view that places two league participants side-by-side so they can compare predictions, results, and overall performance.<br><br>**Depends on:** refs: [R1, cavekit-leagues, cavekit-scoring] | M | medium |
| `feature_cavekit_member_stats_page` | ✨ feature | 🆕 created | 🟠 medium | predictions | Members need an individual stats page that summarizes a participant's performance, streaks, and recent predictions.<br><br>**Depends on:** refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-badges] | M | medium |
| `feature_cavekit_prediction_completion_counter` | ✨ feature | 🆕 created | 🟢 low | predictions | Show a group-stage progress counter so users can see how many matches they have predicted out of the total. | M | medium |
| `achievement-badges` | ✨ feature | 🔓 open | ⚪ tbd | predictions | Prediction games become more social and replayable when users earn visible recognition for notable performances. Achievement badges ("Perfect Round", "Underdog Caller", "Top of the League") give users bragging rights and surface interesting stories from the data.<br><br>**Depends on:** refs: [0008, 0030] | L | high |
| `confidence-multiplier` | ✨ feature | 🔓 open | ⚪ tbd | predictions | Standard group stage predictions award 1 point per correct outcome. A confidence multiplier lets each user "double down" on up to 3 matches they feel certain about — a correct doubled prediction earns 2 points instead of 1. This adds a strategic layer without requiring schema-heavy changes.<br><br>**Depends on:** refs: [0007, 0008] | M | medium |
| `consensus-view` | ✨ feature | ✅ done | ⚪ tbd | predictions | After predictions are locked, it is interesting to see how the league collectively predicted each match — did everyone predict the same outcome, or was the league split? A consensus view on the match breakdown page shows the distribution of predictions (e.g. "Home 60% · Draw 20% · Away 20%") and reveals who went against the grain.<br><br>**Depends on:** refs: [0021, 0025] | M | medium |
| `global-navigation` | ✨ feature | ✅ done | ⚪ tbd | predictions | There is no persistent navigation across the app. Users who are deep in standings or predictions have no visible way to get back to their dashboard or switch leagues without manually editing the URL. The base layout has a minimal header but no nav links. | L | high |
| `group-prediction-completion-indicator` | ✨ feature | 🔓 open | ⚪ tbd | predictions | The group stage tab shows all matches but gives no indication of how many the user has already predicted. For a 36-match group stage, a user who partially fills the form and saves has no way to know they missed matches. A simple "12 / 36 predicted" counter near the save button would surface gaps before submission. | M | medium |
| `group-save-htmx-feedback` | 🐞 bug | ✅ done | ⚪ tbd | predictions | The group stage form posts via HTMX with `hx-target="#group-status" hx-swap="innerHTML"`, expecting an inline snippet to appear in the status span on success. However `save_group` currently returns a redirect response (`htmx_redirect`), which triggers a full page reload instead of swapping content into the target element. The `#group-status` span is never populated, so users receive no confirmation that their predictions were saved. | M | medium |
| `kickoff-countdown` | ✨ feature | 🔓 open | ⚪ tbd | predictions | Match pages and the fixture list show a static formatted kickoff time in UTC, but users have to mentally convert the time themselves. A live countdown ("kicks off in 2h 15m") gives immediate context without requiring time-zone arithmetic and makes the predictions deadline feel tangible.<br><br>**Depends on:** refs: [0021, 0026] | M | medium |
| `leagues` | ✨ feature | ✅ done | ⚪ tbd | predictions | Allow admins to create named leagues and generate invite links, and allow users to join a league via that link. A user may belong to multiple leagues. Leagues provide the competitive grouping used by the leaderboard — users' predictions and scores are global, but the ranking view is per-league. | L | high |
| `match-schedule-display` | ✨ feature | ✅ done | ⚪ tbd | predictions | Match kick-off times are stored as `scheduled_at TIMESTAMPTZ` in the DB and fetched from the football API, but they are either not displayed or shown as raw ISO-8601 strings in the predictions page. Users should see dates and times in a readable format, ideally in their local timezone. | M | medium |
| `prediction-table-indexes` | 🧰 chore | ✅ done | ⚪ tbd | predictions | The three prediction tables (`group_stage_predictions`, `knockout_predictions`, `top_scorer_predictions`) and `league_members` have no secondary indexes beyond their primary keys. Every query that loads a user's predictions does a full table scan. Add indexes so reads stay fast as row counts grow.<br><br>**Depends on:** refs: [0007] | S | low |
| `predictions-review` | ✨ feature | ✅ done | ⚪ tbd | predictions | Once predictions are locked, users cannot see what they submitted — the prediction forms become read-only inputs but there is no review page that shows each prediction alongside the actual result and points awarded. A review page lets users understand exactly how their score was built up and compare their choices against outcomes.<br><br>**Depends on:** refs: [0007, 0025] | L | high |
| `predictions-review-knockout-ux` | ✨ feature | 🔓 open | ⚪ tbd | predictions | The predictions review page (`/leagues/{id}/predictions/review`) has two UX gaps in the knockout section: (1) each knockout prediction is rendered as a standalone row, so the round structure is lost in a long list; (2) there is no visual signal for whether a knockout pick was correct or wrong — only the group stage section has proper correct/wrong/pending styling. This task groups knockout predictions by round into compact labeled blocks and adds the same colour-coded correctness indicators used by the group stage section.<br><br>**Depends on:** refs: [0027] | L | high |
| `show-results-on-predictions-page` | ✨ feature | 🔓 open | ⚪ tbd | predictions | Once predictions are locked and matches kick off, the predictions page becomes a read-only form showing only the user's picks — there's no indication of actual results. Users have to navigate to their league's review page to see how their picks fared. Showing the score and outcome inline on the predictions page gives users an at-a-glance view of how they're doing without leaving `/predictions`.<br><br>**Depends on:** refs: [0043] | L | high |
| `feature_cavekit_league_creation` | ✨ feature | 🆕 created | 🔴 high | leagues | Allow admin users to create leagues from the admin area using a unique league name. | M | medium |
| `feature_cavekit_invite_token_generation` | ✨ feature | 🆕 created | 🔴 high | leagues | Generate a persistent invite token for each league so members can share a join link without exposing internal identifiers. | M | medium |
| `feature_cavekit_league_overview_page` | ✨ feature | 🆕 created | 🔴 high | leagues | Render a members-only league overview page that shows league details and the current membership roster. | M | medium |
| `feature_cavekit_membership_tracking` | ✨ feature | 🆕 created | 🔴 high | leagues | Persist league membership in the database and use it as the source of truth for access control and league-scoped views. | M | medium |
| `feature_cavekit_group_stage_standings_table` | ✨ feature | 🆕 created | 🔴 high | leagues | The app needs a FIFA-style group standings table that can be computed from finished group matches and rendered for league members.<br><br>**Depends on:** refs: [R9] | L | high |
| `feature_cavekit_member_badges_display` | ✨ feature | 🆕 created | 🟠 medium | leagues | Display a user’s earned badges on the member stats page so league members can see achievements prominently.<br><br>**Depends on:** badge definitions, storage, and award job tickets.; Depends on: member stats page existing in standings. | L | high |
| `feature_cavekit_admin_league_list` | ✨ feature | 🆕 created | 🟠 medium | leagues | Show admins a league listing on the admin dashboard so they can review league inventory and basic metadata. | M | medium |
| `leaderboard-tiebreaking` | ✨ feature | ✅ done | ⚪ tbd | leagues | When two or more league members have the same `total_points`, the leaderboard assigns them the same rank but lists them in arbitrary order (database row order). This is confusing and can feel unfair. A deterministic tie-breaking rule makes rankings unambiguous and gives users a consistent experience.<br><br>**Depends on:** refs: [0009] | L | high |
| `league-member-browser` | ✨ feature | ✅ done | ⚪ tbd | leagues | League members have no way to see who else is in their league, when the league was created, or how many members it has. The only league-related page is the join flow. A league overview page would give members context and make the social aspect of the app visible. | L | high |
| `feature_cavekit_htmx_leaderboard_fragment` | ✨ feature | 🆕 created | 🟠 medium | standings | The leaderboard needs a reusable HTML fragment so the standings can refresh without a full page reload.<br><br>**Depends on:** refs: [R1] | L | high |
| `feature_cavekit_hypo_param_validation` | ✨ feature | 🆕 created | 🟠 medium | standings | Hypothetical leaderboard projections need server-side validation so only valid, unplayed group-stage matches can influence the computed projection.<br><br>**Depends on:** refs: [R9, R8] | L | high |
| `feature_cavekit_scenario_modeling` | ✨ feature | 🆕 created | 🟠 medium | standings | Members need to test hypothetical outcomes for unplayed matches and see how the leaderboard would change before those games are decided.<br><br>**Depends on:** refs: [R1, R2, cavekit-scoring] | L | high |
| `feature_cavekit_leaderboard_badge_display` | ✨ feature | 🆕 created | 🟢 low | standings | Add an optional leaderboard badge column that highlights each user’s most notable achievement without changing the core leaderboard behavior.<br><br>**Depends on:** badge definitions, storage, and display metadata tickets.; Depends on: leaderboard page existing in standings. | L | high |
| `scenario-modeling` | ✨ feature | 🔓 open | ⚪ tbd | standings | Let users explore "what if" scenarios on the standings page: select a hypothetical outcome for an upcoming match and see how the leaderboard would shift, without persisting anything to the database. This was scoped out of task 0009.<br><br>**Depends on:** refs: [0009] | L | high |
| `feature_cavekit_badge_definitions` | ✨ feature | 🆕 created | 🔴 high | badges | Create the canonical achievement badge set for Cavekit and define the logic needed to determine whether a user qualifies for each badge.<br><br>**Depends on:** badge storage, award job, and display tickets. | L | high |
| `feature_cavekit_badge_metadata` | ✨ feature | 🆕 created | 🟠 medium | badges | Expose badge metadata as a stable code-level contract so UI surfaces can render names, descriptions, and icons consistently.<br><br>**Depends on:** badge definitions ticket. | L | high |
| `chore_cavekit_otlp_dependency_setup` | 🧰 chore | 🆕 created | 🔴 high | observability | Add the OpenTelemetry crates needed to support optional OTLP trace export to Jaeger. | S | low |
| `chore_cavekit_conditional_trace_export` | 🧰 chore | 🆕 created | 🔴 high | observability | Enable OTLP trace export only when the OTLP endpoint environment variable is present. | S | low |
| `chore_cavekit_tracer_provider_initialization` | 🧰 chore | 🆕 created | 🔴 high | observability | Initialize the OpenTelemetry tracer provider with the correct batch export runtime and development sampling behavior. | S | low |
| `chore_cavekit_no_breaking_changes` | 🧰 chore | 🆕 created | 🔴 high | observability | Verify that adding OTLP/Jaeger observability does not break existing build, test, startup, or stdout tracing behavior. | S | low |
| `chore_cavekit_graceful_tracer_shutdown` | 🧰 chore | 🆕 created | 🟠 medium | observability | Shut down the tracer provider cleanly so pending traces are flushed during application exit. | S | low |
| `chore_cavekit_jaeger_docker_compose` | 🧰 chore | 🆕 created | 🟠 medium | observability | Add local docker-compose support for running a Jaeger all-in-one container. | S | low |
| `chore_cavekit_otel_env_configuration` | 🧰 chore | 🆕 created | 🟠 medium | observability | Document the optional environment configuration needed to enable OTLP trace export. | S | low |
| `chore_cavekit_trace_integration` | 🧰 chore | 🆕 created | 🟠 medium | observability | Connect the existing tracing instrumentation so HTTP, database, and background spans are exported to Jaeger when OTLP is enabled. | S | low |
| `otlp-jaeger-observability` | 🧰 chore | 🔓 open | ⚪ tbd | observability | Wire up OpenTelemetry OTLP trace export so that all existing `tracing` instrumentation (HTTP middleware, handler spans, SQLx queries) is automatically visible in a local Jaeger UI. The change must be fully opt-in — the app must continue to work identically without any environment changes for developers not running Jaeger. | S | low |
| `debt_claude_to_opencode_thoughts` | 🧹 debt | 👍 reviewed | 🔴 high | debt/docs | Remove repo-local Claude-specific guidance files and migrate task tracking into `thoughts/` using an agentic workflow model. The end state should rely on opencode-native conventions where they are needed, with obsolete Claude markdown removed once replacements are in place or confirmed unnecessary. | L | high |
| `crest-docs-and-fallback-asset` | 🧰 chore | ✅ done | ⚪ tbd | debt/docs | Task 0036 was implemented using crest images from the Football Data API instead of the originally planned SVG flag assets or Unicode emoji. ADR-0019 (which documents the emoji approach) is now stale and misleading. Additionally, `src/crests.rs` already references `/assets/default.svg` as the fallback for teams without a crest URL, but that file does not yet exist. This task brings the docs and assets in sync with the actual implementation.<br><br>**Depends on:** refs: [0036] | S | low |
| `rust-2024-edition-docs` | 🧰 chore | ✅ done | ⚪ tbd | debt/docs | The project was migrated from Rust edition 2021 to 2024 (commit `1692d98`) but no documentation reflects this change. Update the existing Rust ADR and write a new ADR recording the upgrade decision so the docs accurately describe the current state of the project. | S | low |
| `knockout-topscore-count-ux` | 🐞 bug | 🔓 open | ⚪ tbd | misc | When a user submits a knockout round form with the wrong number of teams (or top-scorer with fewer than 3 players), the server returns `AppError::BadRequest`, which renders a generic error page. The UI shows "Select X teams" as a hint but provides no client-side guard. Users who accidentally submit early get a hard error page instead of a friendly inline message. | M | medium |
| `project-scaffold` | 🧰 chore | 👍 reviewed | ⚪ tbd | misc | Bootstrap the project from an empty repository into a compiling, runnable Rust application that implements the full structural skeleton defined in the ADRs. No business logic or features are included — the outcome is a working foundation that every subsequent task builds upon. | S | low |
| `qsform-body-limit` | 🐞 bug | 👍 reviewed | ⚪ tbd | misc | The `QsForm<T>` extractor body read is capped at a named 16 KiB limit, oversized bodies return 413 Payload Too Large, and serde_qs parse errors remain 400 Bad Request. | M | medium |

## ✅ Suggested Implementation Order

A coarse order based on dependency clusters and likely build-up sequence:

- auth: FEATURE-004: Admin role access control; FEATURE-LEAGUES-03: Token-based league joining; feature_cavekit_session_cleanup; feature_cavekit_public_pages (+8 more)
- tournament: BUG-PREDICTIONS-04: Enforce prediction lock on all save handlers; FEATURE-037: Persist awarded badges; FEATURE-038: Award badges after scoring completes; feature_cavekit_main_leaderboard_standings; FEATURE-CAVEKIT-TOURNAMENT-01: Tournament registration from football-data.org; FEATURE-CAVEKIT-TOURNAMENT-02: Seed tournament data from football-data.org (+30 more)
- scoring: feature_cavekit_potential_points_indicator; FEATURE-SCORING-02: Ingest finished match results; FEATURE-SCORING-04: Score group stage predictions; FEATURE-SCORING-05: Score knockout predictions; FEATURE-SCORING-07: Sync player goal counts; FEATURE-SCORING-08: Define scoring domain models (+1 more)
- predictions: FEATURE-PREDICTIONS-06: League prediction review page; FEATURE-PREDICTIONS-08: Show actual results on the predictions page; feature_cavekit_match_breakdown; feature_cavekit_member_comparison; feature_cavekit_member_stats_page; FEATURE-PREDICTIONS-07: Prediction completion counter (+13 more)
- leagues: FEATURE-LEAGUES-01: Admin league creation; FEATURE-LEAGUES-02: Invite token generation; FEATURE-LEAGUES-04: League overview page; FEATURE-LEAGUES-05: Membership tracking; feature_cavekit_group_stage_standings_table; FEATURE-039: Show earned badges on member stats (+3 more)
- standings: feature_cavekit_htmx_leaderboard_fragment; feature_cavekit_hypo_param_validation; feature_cavekit_scenario_modeling; FEATURE-040: Optionally show a badge on the leaderboard; scenario-modeling
- badges: FEATURE-036: Define achievement badge types; FEATURE-041: Define badge metadata for display
- observability: CHORE-OBS-01: OTLP/Jaeger dependency setup; CHORE-OBS-02: Conditional OTLP trace export; CHORE-OBS-03: Tracer provider initialization; CHORE-OBS-08: Preserve existing behavior; CHORE-OBS-04: Graceful tracer shutdown; CHORE-OBS-05: Jaeger docker compose support (+3 more)
- debt/docs: crest-docs-and-fallback-asset; rust-2024-edition-docs
- misc: knockout-topscore-count-ux

## ⚠️ Blockers and Risks

- Tickets with explicit dependency notes should be reviewed before scheduling.
- High-complexity items are often the best candidates for early design work.
- Dependency-heavy areas: auth, tournament foundation, scoring pipeline, predictions UX, standings/leaderboard.

## 📝 Fields to Keep

Each ticket row should include at minimum:

- `status`
- `priority`
- `summary`
- `domain`
- `type`
- `estimate`
- `complexity`
- `depends_on`
- `blocks`
- `related`

Recommended extra fields:

- `owner`
- `created`
- `updated`
- `phase`
- `risk`
- `notes`

## 🧾 Update Rules

- Update this file when a ticket is added, closed, reprioritized, or re-scoped.
- Keep dependency links symmetric where possible.
- If order changes because of a new dependency, update the suggested implementation order too.
- Keep summaries short and specific.

## 🧷 Notes

- Use emojis only on high-signal labels so the page stays readable.
- Treat this as a planning aid, not a replacement for the individual ticket files.
